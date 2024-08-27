use std::{convert::Infallible, io::ErrorKind, sync::Arc};

use axum::{
    extract::Path,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use sandbox::SandboxConfig;
use serde::Serialize;
use tokio::{
    fs,
    sync::{mpsc, watch},
    task::JoinSet,
};
use tokio_stream::wrappers::ReceiverStream;

use super::*;
use crate::sandbox::{ResourceUsage, Sandbox};

/// Number of failed tests at which the subtask is skipped
const SKIP_COUNT: u8 = 3;

#[derive(Debug, Clone, Serialize)]
pub enum Message {
    Queued,
    Building,
    Judging {
        verdict: Verdict,
        total: u32,
    },
    Done {
        report: TaskReport,
        compile_stderr: String,
    },
    Error(String),
}

impl From<Message> for Result<Event, Infallible> {
    fn from(message: Message) -> Self {
        Ok(Event::default().json_data(message).unwrap())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskReport {
    verdict: Verdict,
    subtasks: Vec<SubtaskReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubtaskReport {
    verdict: Verdict,
    tests: Vec<TestReport>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TestReport {
    verdict: Verdict,
    resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Verdict {
    CompileError,
    RuntimeError,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    Skipped,
    Accepted,
}

#[derive(Debug, TryFromMultipart)]
pub struct Submission {
    language: String,
    code: String,
}

type Stream = Sse<ReceiverStream<Result<Event, Infallible>>>;
type RequestError = (StatusCode, String);

#[tracing::instrument]
pub async fn submit(
    Path((contest, task)): Path<(String, usize)>,
    TypedMultipart(Submission { language, code }): TypedMultipart<Submission>,
) -> Result<Stream, RequestError> {
    tracing::info!("submission to {contest}/{task} received");
    let config = CONFIG.get().unwrap();

    let task = config
        .contests
        .get(&contest)
        .ok_or((StatusCode::NOT_FOUND, String::from("contest not found")))?
        .tasks
        .get(task - 1)
        .ok_or((StatusCode::NOT_FOUND, String::from("task not found")))?;

    let language = config.languages.get(&language).ok_or((
        StatusCode::BAD_REQUEST,
        String::from("unsupported language"),
    ))?;

    if task.subtasks.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("task does not require code submission"),
        ));
    }

    let (tx, rx) = mpsc::channel(64);

    tracing::debug!("submission validated, queuing submission");
    tx.send(Message::Queued.into()).await.unwrap();

    let sandbox =
        Arc::new(Sandbox::new().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?);

    tokio::spawn(async move {
        let result = Judge {
            tx,
            sandbox,
            task,
            language,
            code,
            resource_limits_build: config.resource_limits_build,
            resource_limits_run: config.resource_limits_run,
        }
        .submit()
        .await;

        if let Err(e) = result {
            tracing::error!("failed to judge submission: {e:?}");
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

struct Judge {
    tx: mpsc::Sender<Result<Event, Infallible>>,
    sandbox: Arc<Sandbox>,
    task: &'static Task,
    language: &'static Language,
    code: String,
    resource_limits_build: Option<ResourceLimits>,
    resource_limits_run: ResourceLimits,
}

impl Judge {
    #[tracing::instrument(skip(self))]
    async fn submit(mut self) -> color_eyre::Result<()> {
        let (compile_ok, compile_stderr) = self.build().await?;
        if !compile_ok {
            self.tx
                .send(
                    Message::Done {
                        report: TaskReport {
                            verdict: Verdict::CompileError,
                            subtasks: Vec::new(),
                        },
                        compile_stderr,
                    }
                    .into(),
                )
                .await?;

            return Ok(());
        }

        let total = self
            .task
            .subtasks
            .iter()
            .map(|subtask| subtask.tests.len() as u32)
            .sum();
        tracing::debug!("starting judging of {total} tests");

        let mut report = TaskReport {
            verdict: Verdict::Accepted,
            subtasks: vec![
                SubtaskReport {
                    verdict: Verdict::Accepted,
                    tests: vec![],
                };
                self.task.subtasks.len()
            ],
        };

        let mut subtask_set = JoinSet::new();
        for (subtask_idx, subtask) in self.task.subtasks.iter().enumerate() {
            let mut subtask_report = SubtaskReport {
                verdict: Verdict::Accepted,
                tests: vec![
                    TestReport {
                        verdict: Verdict::Skipped,
                        resource_usage: ResourceUsage::default()
                    };
                    subtask.tests.len()
                ],
            };

            let config = SandboxConfig {
                resource_limits: Some(self.resource_limits_run),
                seccomp: true,
                landlock: true,
            };

            let (tx, sandbox, language) =
                (self.tx.clone(), self.sandbox.clone(), self.language.clone());
            subtask_set.spawn(async move {
                let (skip_tx, skip_rx) = watch::channel(0u8);

                let mut test_set = JoinSet::new();
                for (test_idx, test) in subtask.tests.iter().enumerate() {
                    let (skip_tx, tx, sandbox, language) = (skip_tx.clone(), tx.clone(), sandbox.clone(), language.clone());
                    test_set.spawn(async move {
                        let (verdict, resource_usage) = match sandbox.run(&language.run, config, test.input.as_bytes()).await {
                            Ok(output) => {
                                let status = output.exit_status();

                                let verdict = if status.success() {
                                    let stdout = output.stdout_utf8();
                                    if stdout.is_ok() && stdout.unwrap().trim() == test.output.trim() {
                                        Verdict::Accepted
                                    } else {
                                        Verdict::WrongAnswer
                                    }
                                } else if status.code().is_none() {
                                    skip_tx.send_modify(|count| *count += 1);

                                    if output.resource_usage().memory
                                        > self.resource_limits_run.memory
                                    {
                                        Verdict::MemoryLimitExceeded
                                    } else {
                                        Verdict::TimeLimitExceeded
                                    }
                                } else {
                                    Verdict::RuntimeError
                                };

                                (verdict, output.resource_usage())
                            },
                            Err(e) => {
                                if let ErrorKind::BrokenPipe = e.kind() {
                                    tracing::warn!("broken pipe error: possible that code is not reading/writing data correctly");
                                    (Verdict::WrongAnswer, ResourceUsage::default())
                                } else {
                                    return Err(color_eyre::Report::new(e))
                                }
                            }
                        };

                        tx.send(Message::Judging { verdict, total }.into())
                        .await
                        .unwrap();

                        let report = TestReport {
                            verdict,
                            resource_usage,
                        };

                        tracing::trace!(?report);

                        Ok((test_idx, report))
                    });
                }

                while let Some(result) = test_set.join_next().await {
                    let (test_idx, test_report )= result??;

                    subtask_report.verdict = subtask_report.verdict.min(test_report.verdict);
                    subtask_report.tests[test_idx] = test_report;

                    if *skip_rx.borrow() > SKIP_COUNT {
                        tracing::warn!("exceeded skip count for subtask, skipping");
                        test_set.abort_all();
                        subtask_report.verdict = Verdict::Skipped;
                        return Ok((subtask_idx, subtask_report));
                    }
                }

                Ok::<_, color_eyre::Report>((subtask_idx, subtask_report))
            });
        }

        while let Some(result) = subtask_set.join_next().await {
            let (subtask_idx, subtask_report) = result??;
            report.verdict = report.verdict.min(subtask_report.verdict);
            report.subtasks[subtask_idx] = subtask_report;
        }

        tracing::debug!("judging complete");

        self.tx
            .send(
                Message::Done {
                    report,
                    compile_stderr,
                }
                .into(),
            )
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn build(&mut self) -> color_eyre::Result<(bool, String)> {
        let code_path = self.sandbox.path().join(&self.language.filename);
        fs::write(&code_path, &self.code).await?;
        tracing::debug!("code written to {}", code_path.display());

        if let Some(build) = &self.language.build {
            tracing::debug!("starting build step");
            self.tx.send(Message::Building.into()).await?;

            let config = SandboxConfig {
                resource_limits: self.resource_limits_build,
                ..Default::default()
            };

            let output = self.sandbox.run(build, config, &[]).await?;
            let stderr = match output.stderr_utf8() {
                Ok(s) => s.to_owned(),
                Err(e) => format!("compile stderr was not valid UTF-8: {e}"),
            };

            if !output.exit_status().success() {
                tracing::error!("build failed");
                tracing::trace!(?stderr);

                return Ok((false, stderr));
            }

            Ok((true, stderr))
        } else {
            tracing::debug!("no build step");
            Ok((true, String::new()))
        }
    }
}

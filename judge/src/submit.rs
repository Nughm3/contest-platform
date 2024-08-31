use std::{convert::Infallible, io::ErrorKind, path::Path};

use axum::{
    extract,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
};
use color_eyre::eyre::WrapErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    config::{Config, Language},
    contest::Task,
    sandbox::{run, Command, Profile, ResourceUsage},
    CONFIG, CONTESTS,
};

#[derive(Deserialize)]
pub struct LanguageQuery {
    language: String,
}

#[derive(Debug, Error)]
pub enum SubmitError {
    #[error("could not find contest: {0}")]
    ContestNotFound(String),
    #[error("could not find task: {0}")]
    TaskNotFound(usize),
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("could not read submission code: {0}")]
    InvalidCode(#[from] extract::multipart::MultipartError),
    #[error("no code submitted")]
    NoCode,
}

impl IntoResponse for SubmitError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

type Stream = Sse<ReceiverStream<Result<Event, Infallible>>>;

pub async fn handler(
    extract::Path((contest_name, task_index)): extract::Path<(String, usize)>,
    extract::Query(LanguageQuery {
        language: language_name,
    }): extract::Query<LanguageQuery>,
    mut multipart: extract::Multipart,
) -> Result<Stream, SubmitError> {
    let contest = CONTESTS
        .get()
        .unwrap()
        .get(&contest_name)
        .ok_or_else(|| SubmitError::ContestNotFound(contest_name.clone()))?;

    let task = contest
        .tasks
        .get(task_index)
        .ok_or_else(|| SubmitError::TaskNotFound(task_index))?;

    let config = CONFIG.get().unwrap();

    let language = config
        .languages
        .get(&language_name)
        .ok_or_else(|| SubmitError::UnsupportedLanguage(language_name))?;

    let code = multipart
        .next_field()
        .await?
        .ok_or(SubmitError::NoCode)?
        .text()
        .await?;

    tracing::info!(
        "submission received for {} : #{}",
        contest.name,
        task_index + 1
    );

    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        if let Err(report) = submit(&tx, config, task, code, language).await {
            tracing::error!("{report:?}");

            Message::Error {
                reason: report
                    .chain()
                    .enumerate()
                    .map(|(i, e)| format!("{i}: {e}\n"))
                    .collect(),
            }
            .send_to(&tx)
            .await;
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

type Sender = mpsc::Sender<Result<Event, Infallible>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
enum Message {
    /// Indicates that the submission has been received
    Queued { total: u32, uuid: Uuid },
    /// Indicates that the compile step has been started (optional)
    Compiling,
    /// Provides compiler warnings and errors when given (optional)
    CompilerOutput { exit_code: i32, stderr: String },
    /// Judging status
    Judging { verdict: Verdict },
    /// Tests were skipped due to exceeding resource usage
    Skipping { estimated_count: u32 },
    /// Judging completed successfully (final)
    Done { report: Report },
    /// The judge experienced an internal error (final)
    Error { reason: String },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
struct Report {
    task: Verdict,
    subtasks: Vec<Verdict>,
    tests: Vec<Vec<TestReport>>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
struct TestReport {
    verdict: Verdict,
    resource_usage: ResourceUsage,
}

impl Message {
    async fn send_to(self, tx: &Sender) {
        tx.send(Ok(Event::default().json_data(self).unwrap()))
            .await
            .expect("connection closed, message not sent")
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum Verdict {
    CompileError,
    RuntimeError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    WrongAnswer,
    Skipped,
    Accepted,
}

#[tracing::instrument(skip_all)]
async fn submit(
    tx: &Sender,
    config: &Config,
    task: &Task,
    code: String,
    language: &Language,
) -> color_eyre::Result<()> {
    let uuid = Uuid::new_v4();
    Message::Queued {
        total: task.subtasks.iter().map(|s| s.tests.len() as u32).sum(),
        uuid,
    }
    .send_to(tx)
    .await;

    tracing::info!("submission ID: {uuid}");

    let dir = Path::new("submissions").join(uuid.to_string());
    fs::create_dir(&dir)
        .await
        .wrap_err("failed to create submission directory")?;

    let file = dir.join(&language.filename);
    fs::write(&file, code)
        .await
        .wrap_err("failed to write code to disk")?;

    let build_succeeded = if let Some(command) = &language.build {
        build(tx, &dir, command)
            .await
            .wrap_err("failed to execute build command")?
    } else {
        tracing::trace!("skipping build phase");
        true
    };

    if build_succeeded {
        judge(tx, config, task, &dir, &language.run)
            .await
            .wrap_err("failed to judge submission")?;
    } else {
        Message::Done {
            report: Report {
                task: Verdict::CompileError,
                subtasks: vec![Verdict::CompileError; task.subtasks.len()],
                tests: task
                    .subtasks
                    .iter()
                    .map(|s| {
                        vec![
                            TestReport {
                                verdict: Verdict::CompileError,
                                resource_usage: ResourceUsage::default()
                            };
                            s.tests.len()
                        ]
                    })
                    .collect(),
            },
        }
        .send_to(tx)
        .await;
    }

    Ok(())
}

async fn build(tx: &Sender, dir: impl AsRef<Path>, command: &Command) -> color_eyre::Result<bool> {
    Message::Compiling.send_to(tx).await;

    let output = run(dir, command, &[], Profile::Build).await?;
    let status = output.exit_status();
    let exit_code = status.code().unwrap_or(-1);

    if status.success() {
        if !output.stderr().is_empty() {
            tracing::warn!("compiler warnings emitted");
            Message::CompilerOutput {
                exit_code,
                stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
            }
            .send_to(tx)
            .await;
        }

        tracing::trace!("build succeeded");
        Ok(true)
    } else {
        Message::CompilerOutput {
            exit_code,
            stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
        }
        .send_to(tx)
        .await;

        if exit_code != -1 {
            tracing::error!("build failed (exit code: {exit_code})");
        } else {
            tracing::error!("build failed (terminated by signal)");
        }

        Ok(false)
    }
}

async fn judge(
    tx: &Sender,
    config: &Config,
    task: &Task,
    dir: impl AsRef<Path>,
    command: &Command,
) -> color_eyre::Result<()> {
    let mut report = Report {
        task: Verdict::Accepted,
        subtasks: vec![Verdict::Accepted; task.subtasks.len()],
        tests: task
            .subtasks
            .iter()
            .map(|s| {
                vec![
                    TestReport {
                        verdict: Verdict::Skipped,
                        resource_usage: ResourceUsage::default()
                    };
                    s.tests.len()
                ]
            })
            .collect(),
    };

    for (subtask_idx, subtask) in task.subtasks.iter().enumerate() {
        for (test_idx, test) in subtask.tests.iter().enumerate() {
            let test_report = match run(
                &dir,
                command,
                test.input.as_bytes(),
                Profile::Run(config.resource_limits),
            )
            .await
            {
                Ok(output) => {
                    let status = output.exit_status();
                    let resource_usage = output.resource_usage();

                    let verdict = if resource_usage.exceeded(config.resource_limits)
                        && status.code().is_none()
                    {
                        if resource_usage.exceeded_time(config.resource_limits) {
                            Verdict::TimeLimitExceeded
                        } else {
                            Verdict::MemoryLimitExceeded
                        }
                    } else if status.success() {
                        match output.stdout_utf8() {
                            Ok(stdout) if stdout.trim() == test.output.trim() => Verdict::Accepted,
                            _ => Verdict::WrongAnswer,
                        }
                    } else {
                        Verdict::RuntimeError
                    };

                    TestReport {
                        verdict,
                        resource_usage,
                    }
                }
                Err(e) if matches!(e.kind(), ErrorKind::BrokenPipe) => {
                    tracing::warn!(
                        "broken pipe error, code may not be reading/writing data correctly"
                    );
                    TestReport {
                        verdict: Verdict::WrongAnswer,
                        resource_usage: ResourceUsage::default(),
                    }
                }
                Err(e) => return Err(e.into()),
            };

            report.tests[subtask_idx][test_idx] = test_report;
            report.subtasks[subtask_idx] = report.subtasks[subtask_idx].min(test_report.verdict);

            tracing::trace!(
                "{}-{} : {:?}",
                subtask_idx + 1,
                test_idx + 1,
                test_report.verdict
            );

            Message::Judging {
                verdict: test_report.verdict,
            }
            .send_to(tx)
            .await;
        }

        report.task = report.task.min(report.subtasks[subtask_idx]);
    }

    Message::Done { report }.send_to(tx).await;
    Ok(())
}

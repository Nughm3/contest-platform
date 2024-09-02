use std::{io::ErrorKind, path::Path, sync::Arc};

use axum::response::sse::Event;
use color_eyre::eyre::WrapErr;
use schemars::JsonSchema;
use serde::Serialize;
use tokio::{sync::watch, task::JoinSet};
use yansi::Paint;

use crate::{
    contest::{Config, Language, Task, Test},
    sandbox::{run, Output, Profile, ResourceUsage},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Report {
    task: Verdict,
    subtasks: Vec<Verdict>,
    tests: Vec<Vec<TestReport>>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, JsonSchema)]
pub struct TestReport {
    verdict: Verdict,
    resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum Verdict {
    CompileError,
    RuntimeError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    WrongAnswer,
    Skipped,
    Accepted,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, JsonSchema)]
pub enum Message {
    /// Queued for submission
    Queued { tests: u32 },
    /// Indicates that the compile step has been started (optional)
    Compiling,
    /// Provides compiler warnings and errors (optional)
    Compiled { exit_code: i32, stderr: String },
    /// Judging status
    Judging { verdict: Verdict },
    /// Tests were skipped due to exceeding resource usage
    Skipping { estimated_count: u32 },
    /// Broken pipe error encountered, code may not be performing I/O correctly
    BrokenPipe,
    /// Judging completed successfully (final)
    Done { report: Report },
    /// The judge experienced an internal error (final)
    Error { reason: String },
}

type Sender = tokio::sync::mpsc::Sender<Result<Event, std::convert::Infallible>>;

#[derive(Clone)]
struct State {
    tx: Sender,
    dir: Arc<Path>,
    config: &'static Config,
    task: &'static Task,
    language: &'static Language,
}

impl State {
    async fn run(&self, profile: Profile, stdin: &[u8]) -> tokio::io::Result<Output> {
        let command = match profile {
            Profile::Compile => self
                .language
                .compile
                .as_ref()
                .expect("attempted to execute non-existent compile command"),
            Profile::Run(_) => &self.language.run,
        };

        run(&self.dir, command, stdin, profile).await
    }

    async fn send(&self, message: Message) {
        self.tx
            .send(Ok(Event::default().json_data(message).unwrap()))
            .await
            .expect("channel closed");
    }
}

#[tracing::instrument(skip_all)]
pub async fn submit(
    tx: Sender,
    dir: impl AsRef<Path>,
    config: &'static Config,
    task: &'static Task,
    language: &'static Language,
) {
    let state = State {
        tx,
        dir: Arc::from(dir.as_ref()),
        config,
        task,
        language,
    };

    state
        .send(Message::Queued {
            tests: task.subtasks.iter().map(|s| s.tests.len() as u32).sum(),
        })
        .await;

    if let Err(report) = submit_inner(state.clone()).await {
        tracing::error!("{report:?}");

        let mut reason = String::new();
        for (i, e) in report.chain().enumerate() {
            reason.push_str(&format!("{i}: {e}\n"));
        }

        state.send(Message::Error { reason }).await;
    }
}

async fn submit_inner(state: State) -> color_eyre::Result<()> {
    if state.language.compile.is_some() {
        if !compile(state.clone())
            .await
            .wrap_err("failed to compile submission")?
        {
            let report = Report {
                task: Verdict::CompileError,
                subtasks: vec![Verdict::CompileError; state.task.subtasks.len()],
                tests: state
                    .task
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
            };

            state.send(Message::Done { report }).await;
            return Ok(());
        }
    } else {
        tracing::trace!("skipping build step");
    }

    let report = judge(state.clone())
        .await
        .wrap_err("failed to judge submission")?;

    state.send(Message::Done { report }).await;
    Ok(())
}

#[tracing::instrument(skip(state))]
async fn compile(state: State) -> color_eyre::Result<bool> {
    state.send(Message::Compiling).await;

    let output = state
        .run(Profile::Compile, &[])
        .await
        .wrap_err("failed to execute compile command")?;
    let status = output.exit_status();
    let exit_code = status.code().unwrap_or(-1);

    if status.success() {
        if !output.stderr().is_empty() {
            tracing::warn!("compiler warnings emitted");
            state
                .send(Message::Compiled {
                    exit_code,
                    stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
                })
                .await;
        }

        tracing::trace!("compile succeeded");
        Ok(true)
    } else {
        state
            .send(Message::Compiled {
                exit_code,
                stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
            })
            .await;

        if exit_code != -1 {
            tracing::error!("compilation failed (exit code: {exit_code})");
        } else {
            tracing::error!("compilation failed (terminated by signal)");
        }

        Ok(false)
    }
}

#[tracing::instrument(skip(state))]
async fn judge(state: State) -> color_eyre::Result<Report> {
    let mut subtask_set = JoinSet::new();

    for (subtask_idx, subtask) in state.task.subtasks.iter().enumerate() {
        let state = state.clone();
        subtask_set.spawn(async move {
            let (skip_tx, skip_rx) = watch::channel(0u8);
            let mut test_set = JoinSet::new();

            for (test_idx, test) in subtask.tests.iter().enumerate() {
                let (state, skip_tx) = (state.clone(), skip_tx.clone());
                test_set.spawn(async move {
                    let test_report = run_test(state.clone(), skip_tx, test)
                        .await
                        .wrap_err("failed to run test")?;

                    tracing::trace!(
                        "{}-{}: {}",
                        subtask_idx + 1,
                        test_idx + 1,
                        match test_report.verdict {
                            Verdict::CompileError => Paint::yellow("Compile Error"),
                            Verdict::RuntimeError => Paint::yellow("Runtime Error"),
                            Verdict::MemoryLimitExceeded => Paint::magenta("Memory Limit Exceeded"),
                            Verdict::TimeLimitExceeded => Paint::magenta("Time Limit Exceeded"),
                            Verdict::WrongAnswer => Paint::red("Wrong Answer"),
                            Verdict::Skipped => Paint::blue("Skipped"),
                            Verdict::Accepted => Paint::green("Accepted"),
                        }
                        .bold()
                    );

                    state
                        .send(Message::Judging {
                            verdict: test_report.verdict,
                        })
                        .await;

                    Ok::<_, color_eyre::Report>((test_idx, test_report))
                });
            }

            let mut subtask_verdict = Verdict::Accepted;
            let mut subtask_reports = vec![
                TestReport {
                    verdict: Verdict::Skipped,
                    resource_usage: ResourceUsage::default()
                };
                subtask.tests.len()
            ];

            while let Some(result) = test_set.join_next().await {
                let (test_idx, test_report) = result??;
                subtask_verdict = subtask_verdict.min(test_report.verdict);
                subtask_reports[test_idx] = test_report;

                if *skip_rx.borrow() > state.config.skip_count {
                    tracing::warn!("exceeded skip count for subtask, skipping");
                    test_set.abort_all();
                    state
                        .send(Message::Skipping {
                            estimated_count: (subtask.tests.len()
                                - state.config.skip_count as usize)
                                as u32,
                        })
                        .await;
                    return Ok((subtask_idx, subtask_verdict, subtask_reports));
                }
            }

            Ok::<_, color_eyre::Report>((subtask_idx, subtask_verdict, subtask_reports))
        });
    }

    let mut report = Report {
        task: Verdict::Accepted,
        subtasks: vec![Verdict::Accepted; state.task.subtasks.len()],
        tests: vec![vec![]; state.task.subtasks.len()],
    };

    while let Some(result) = subtask_set.join_next().await {
        let (subtask_idx, subtask_verdict, subtask_reports) = result??;
        report.task = report.task.min(subtask_verdict);
        report.subtasks[subtask_idx] = subtask_verdict;
        report.tests[subtask_idx] = subtask_reports;
    }

    Ok(report)
}

async fn run_test(
    state: State,
    skip_tx: watch::Sender<u8>,
    test: &Test,
) -> color_eyre::Result<TestReport> {
    match state
        .run(
            Profile::Run(state.config.resource_limits),
            test.input.as_bytes(),
        )
        .await
    {
        Ok(output) => {
            let status = output.exit_status();
            let resource_usage = output.resource_usage();

            let verdict = if resource_usage.exceeded(state.config.resource_limits)
                && status.code().is_none()
            {
                skip_tx.send_modify(|count| *count += 1);
                if resource_usage.exceeded_time(state.config.resource_limits) {
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

            Ok(TestReport {
                verdict,
                resource_usage,
            })
        }
        Err(e) if matches!(e.kind(), ErrorKind::BrokenPipe) => {
            tracing::warn!("broken pipe error, code may not be reading/writing data correctly");
            state.send(Message::BrokenPipe).await;
            Ok(TestReport {
                verdict: Verdict::WrongAnswer,
                resource_usage: ResourceUsage::default(),
            })
        }
        Err(e) => Err(e.into()),
    }
}

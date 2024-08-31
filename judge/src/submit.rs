use std::{convert::Infallible, io::ErrorKind, path::Path, sync::Arc};

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
use tokio::{
    fs,
    sync::{mpsc, watch},
    task::JoinSet,
};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
use yansi::Paint;

use crate::{
    config::{Config, Language},
    contest::{Task, Test},
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

    let task = task_index
        .checked_sub(1)
        .and_then(|idx| contest.tasks.get(idx))
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
        "submission received for {}: #{}",
        contest.name,
        task_index + 1
    );

    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        if let Err(report) = submit(&tx, config, task, code, language).await {
            tracing::error!("{report:?}");
            let mut reason = String::new();
            for (i, e) in report.chain().enumerate() {
                reason.push_str(&format!("{i}: {e}\n"));
            }
            Message::Error { reason }.send_to(&tx).await;
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
    config: &'static Config,
    task: &'static Task,
    code: String,
    language: &'static Language,
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

    let compilation_succeeded = if let Some(command) = &language.compile {
        compile(tx, &dir, command)
            .await
            .wrap_err("failed to execute compile command")?
    } else {
        tracing::trace!("skipping compile phase");
        true
    };

    if compilation_succeeded {
        judge(tx, config, task, Arc::from(dir), &language.run)
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

    tracing::info!("judging complete");

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn compile(tx: &Sender, dir: &Path, command: &Command) -> color_eyre::Result<bool> {
    Message::Compiling.send_to(tx).await;

    let output = run(dir, command, &[], Profile::Compile).await?;
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

        tracing::trace!("compile succeeded");
        Ok(true)
    } else {
        Message::CompilerOutput {
            exit_code,
            stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
        }
        .send_to(tx)
        .await;

        if exit_code != -1 {
            tracing::error!("compilation failed (exit code: {exit_code})");
        } else {
            tracing::error!("compilation failed (terminated by signal)");
        }

        Ok(false)
    }
}

#[tracing::instrument(skip_all)]
async fn judge(
    tx: &Sender,
    config: &'static Config,
    task: &'static Task,
    dir: Arc<Path>,
    command: &'static Command,
) -> color_eyre::Result<()> {
    let mut subtask_set = JoinSet::new();
    for (subtask_idx, subtask) in task.subtasks.iter().enumerate() {
        let (tx, dir) = (tx.clone(), dir.clone());

        subtask_set.spawn(async move {
            let (skip_tx, skip_rx) = watch::channel(0u8);

            let mut test_set = JoinSet::new();
            for (test_idx, test) in subtask.tests.iter().enumerate() {
                let (tx, skip_tx, dir) = (tx.clone(), skip_tx.clone(), dir.clone());

                test_set.spawn(async move {
                    let test_report = run_test(config, skip_tx, test, dir, command)
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

                    Message::Judging {
                        verdict: test_report.verdict,
                    }
                    .send_to(&tx)
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

                if *skip_rx.borrow() > config.skip_count {
                    tracing::warn!("exceeded skip count for subtask, skipping");
                    test_set.abort_all();
                    Message::Skipping {
                        estimated_count: (subtask.tests.len() - config.skip_count as usize) as u32,
                    }
                    .send_to(&tx)
                    .await;
                    return Ok((subtask_idx, subtask_verdict, subtask_reports));
                }
            }

            Ok::<_, color_eyre::Report>((subtask_idx, subtask_verdict, subtask_reports))
        });
    }

    let mut report = Report {
        task: Verdict::Accepted,
        subtasks: vec![Verdict::Accepted; task.subtasks.len()],
        tests: vec![vec![]; task.subtasks.len()],
    };

    while let Some(result) = subtask_set.join_next().await {
        let (subtask_idx, subtask_verdict, subtask_reports) = result??;
        report.task = report.task.min(subtask_verdict);
        report.subtasks[subtask_idx] = subtask_verdict;
        report.tests[subtask_idx] = subtask_reports;
    }

    Message::Done { report }.send_to(tx).await;
    Ok(())
}

async fn run_test(
    config: &'static Config,
    skip_tx: watch::Sender<u8>,
    test: &Test,
    dir: Arc<Path>,
    command: &'static Command,
) -> color_eyre::Result<TestReport> {
    match run(
        dir,
        command,
        test.input.as_bytes(),
        Profile::Run(config.resource_limits),
    )
    .await
    {
        Ok(output) => {
            let status = output.exit_status();
            let resource_usage = output.resource_usage();

            let verdict =
                if resource_usage.exceeded(config.resource_limits) && status.code().is_none() {
                    skip_tx.send_modify(|count| *count += 1);
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

            Ok(TestReport {
                verdict,
                resource_usage,
            })
        }
        Err(e) if matches!(e.kind(), ErrorKind::BrokenPipe) => {
            tracing::warn!("broken pipe error, code may not be reading/writing data correctly");
            Ok(TestReport {
                verdict: Verdict::WrongAnswer,
                resource_usage: ResourceUsage::default(),
            })
        }
        Err(e) => Err(e.into()),
    }
}

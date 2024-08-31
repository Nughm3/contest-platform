use std::{convert::Infallible, path::Path};

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
    sandbox::{self, Command, Profile},
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
            tracing::error!(%report);

            Message::Error {
                reason: report.to_string(),
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
    Done {
        task: Verdict,
        subtasks: Vec<Verdict>,
        tests: Vec<Vec<Verdict>>,
    },
    /// The judge experienced an internal error (final)
    Error { reason: String },
}

impl Message {
    async fn send_to(self, tx: &Sender) {
        let event_type = match self {
            Message::Queued { .. }
            | Message::Compiling
            | Message::CompilerOutput { .. }
            | Message::Done { .. }
            | Message::Error { .. } => "system",
            Message::Judging { .. } | Message::Skipping { .. } => "judge",
        };

        let event = Event::default().event(event_type).json_data(self).unwrap();
        tx.send(Ok(event)).await.expect("failed to send message");
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
    .send_to(&tx)
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
        build(&tx, &dir, command)
            .await
            .wrap_err("failed to run build command")?
    } else {
        tracing::trace!("skipping build phase");
        true
    };

    if build_succeeded {
        judge(&tx, config, task, &dir, &language.run)
            .await
            .wrap_err("failed to judge submission")?;
    } else {
        Message::Done {
            task: Verdict::CompileError,
            subtasks: vec![Verdict::CompileError; task.subtasks.len()],
            tests: task
                .subtasks
                .iter()
                .map(|s| vec![Verdict::CompileError; s.tests.len()])
                .collect(),
        }
        .send_to(&tx)
        .await;
    }

    Ok(())
}

async fn build(tx: &Sender, dir: impl AsRef<Path>, command: &Command) -> std::io::Result<bool> {
    Message::Compiling.send_to(&tx).await;

    let output = sandbox::run(dir, command, &[], Profile::Build).await?;
    let status = output.exit_status();
    let exit_code = status.code().unwrap_or(-1);

    if status.success() {
        if output.stderr().is_empty() {
            tracing::warn!("compiler warnings emitted");
            Message::CompilerOutput {
                exit_code,
                stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
            }
            .send_to(&tx)
            .await;
        }

        tracing::trace!("build succeeded");
        Ok(true)
    } else {
        Message::CompilerOutput {
            exit_code,
            stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
        }
        .send_to(&tx)
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
    Ok(())
}

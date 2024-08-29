use std::convert::Infallible;

use axum::{
    extract::{multipart::MultipartError, Multipart, Path, Query},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    sandbox::{self, Profile},
    CONFIG, CONTESTS,
};

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
    Skipped { estimated_count: u32 },
    /// Judging completed successfully
    Done {
        task: Verdict,
        subtasks: Vec<Verdict>,
        tests: Vec<Vec<Verdict>>,
    },
}

impl Message {
    async fn send_to(self, tx: &mpsc::Sender<Result<Event, Infallible>>) {
        tx.send(Ok(Event::default().json_data(self).unwrap()))
            .await
            .expect("failed to send message");
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum Verdict {
    CompileError,
    RuntimeError,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    WrongAnswer,
    Accepted,
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
    InvalidCode(#[from] MultipartError),
    #[error("no code submitted")]
    NoCode,
}

impl IntoResponse for SubmitError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

#[derive(Deserialize)]
pub struct LanguageQuery {
    language: String,
}

type Stream = Sse<ReceiverStream<Result<Event, Infallible>>>;

pub async fn handler(
    Path((contest_name, task_index)): Path<(String, usize)>,
    Query(LanguageQuery {
        language: language_name,
    }): Query<LanguageQuery>,
    mut multipart: Multipart,
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

    let language = CONFIG
        .get()
        .unwrap()
        .languages
        .get(&language_name)
        .ok_or_else(|| SubmitError::UnsupportedLanguage(language_name))?;

    let code = multipart
        .next_field()
        .await?
        .ok_or(SubmitError::NoCode)?
        .text()
        .await?;

    tracing::info!("submission received for {contest_name}/{}", task_index + 1);

    let (tx, rx) = mpsc::channel(64);

    let uuid = Uuid::new_v4();
    Message::Queued {
        total: task.subtasks.iter().map(|s| s.tests.len() as u32).sum(),
        uuid,
    }
    .send_to(&tx)
    .await;

    tokio::spawn(async move {
        let submission_path = std::path::Path::new("submissions").join(uuid.to_string());
        fs::create_dir(&submission_path).await?;
        tracing::trace!(
            "created submission directory at {}",
            submission_path.display()
        );

        let code_path = submission_path.join(&language.filename);
        fs::write(&code_path, code).await?;
        tracing::trace!("code written to {}", code_path.display());

        if let Some(build_command) = &language.build {
            Message::Compiling.send_to(&tx).await;

            let output = sandbox::run(submission_path, build_command, &[], Profile::Build).await?;
            let status = output.exit_status();
            let exit_code = status.code().unwrap_or(-1);
            if !status.success() {
                if exit_code != -1 {
                    tracing::error!("build failed with exit code: {exit_code}");
                } else {
                    tracing::error!("build failed, terminated by signal");
                }

                Message::CompilerOutput {
                    exit_code,
                    stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
                }
                .send_to(&tx)
                .await;

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

                return Ok(());
            } else if !output.stderr().is_empty() {
                tracing::warn!("compiler warnings emitted");
                Message::CompilerOutput {
                    exit_code,
                    stderr: output.stderr_utf8().unwrap_or_default().to_owned(),
                }
                .send_to(&tx)
                .await;
            } else {
                tracing::trace!("build succeeded");
            }
        } else {
            tracing::trace!("skipping build phase");
        }

        Ok::<_, tokio::io::Error>(())
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

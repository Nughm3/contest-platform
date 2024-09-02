use std::{ffi::OsStr, net::SocketAddr, path::Path};

use ahash::AHashMap;
use axum::{
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::post,
    Router,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use color_eyre::eyre::WrapErr;
use contest::Contest;
use once_cell::sync::OnceCell;
use thiserror::Error;
use tokio::{fs, net::TcpListener, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tower_http::trace::TraceLayer;
use tracing_error::ErrorLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;
use uuid::Uuid;

mod contest;
mod sandbox;
mod submit;

static CONTESTS: OnceCell<AHashMap<String, Contest>> = OnceCell::new();

#[derive(TryFromMultipart)]
struct SubmitRequest {
    contest: String,
    task: usize,
    language: String,
    code: String,
}

type Stream = Sse<ReceiverStream<Result<Event, std::convert::Infallible>>>;

#[derive(Debug, Error)]
enum SubmitError {
    #[error("contest {0} not found")]
    ContestNotFound(String),
    #[error("task #{1} for contest {0} not found")]
    TaskNotFound(String, usize),
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),
}

impl IntoResponse for SubmitError {
    fn into_response(self) -> Response {
        let status = match self {
            SubmitError::ContestNotFound(_) | SubmitError::TaskNotFound(_, _) => {
                StatusCode::NOT_FOUND
            }
            SubmitError::UnsupportedLanguage(_) => StatusCode::BAD_REQUEST,
            SubmitError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

async fn handler(
    TypedMultipart(SubmitRequest {
        contest: contest_name,
        task: task_index,
        language: language_name,
        code,
    }): TypedMultipart<SubmitRequest>,
) -> Result<Stream, SubmitError> {
    let contests = CONTESTS.get().unwrap();

    let contest = contests
        .get(&contest_name)
        .ok_or_else(|| SubmitError::ContestNotFound(contest_name.clone()))?;

    let task = task_index
        .checked_sub(1)
        .and_then(|idx| contest.tasks.get(idx))
        .ok_or_else(|| SubmitError::TaskNotFound(contest_name, task_index))?;

    let language = contest
        .config
        .languages
        .iter()
        .find(|lang| lang.name == language_name)
        .ok_or_else(|| SubmitError::UnsupportedLanguage(language_name))?;

    let uuid = Uuid::new_v4();
    let dir = Path::new("submissions").join(uuid.to_string());
    fs::create_dir(&dir).await?;
    fs::write(dir.join(&language.filename), code).await?;

    let (tx, rx) = mpsc::channel(64);
    tokio::spawn(submit::submit(tx, dir, &contest.config, task, language));

    Ok(Sse::new(ReceiverStream::new(rx)))
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(HierarchicalLayer::default().with_ansi(true))
        .with(ErrorLayer::default())
        .try_init()
        .wrap_err("failed to initialize tracing")?;

    let contests = {
        let mut contests = AHashMap::new();

        let mut read_dir = fs::read_dir("contests")
            .await
            .wrap_err("failed to scan contests directory")?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(OsStr::to_str) == Some("json") {
                let input = fs::read_to_string(&path).await?;
                let contest = Contest::load(&input)?;
                tracing::info!("loaded contest {} ({})", contest.name, path.display());
                contests.insert(
                    path.file_stem()
                        .unwrap()
                        .to_str()
                        .expect("non UTF-8 filename")
                        .to_owned(),
                    contest,
                );
            }
        }

        contests
    };

    CONTESTS.set(contests).unwrap();

    if !Path::new("submissions").is_dir() {
        tracing::warn!("submissions directory not found, creating it");
        fs::create_dir("submissions").await?;
    }

    let app = Router::new()
        .route("/", post(handler))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0; 4], 8128));
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("failed to bind to TCP port")?;
    tracing::info!("listening on {addr}");

    let quit_signal = async {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("shutting down");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(quit_signal)
        .await
        .wrap_err("server crashed")?;

    Ok(())
}

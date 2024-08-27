use std::{collections::HashMap, env, net::SocketAddr, path::PathBuf};

use axum::{routing::post, Router};
use color_eyre::eyre::WrapErr;
use once_cell::sync::OnceCell;
use sandbox::{Command, ResourceLimits};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_error::ErrorLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;

mod loader;
mod sandbox;
mod submit;

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    contests: HashMap<String, Contest>,
    resource_limits_build: Option<ResourceLimits>,
    resource_limits_run: ResourceLimits,
    languages: HashMap<String, Language>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Contest {
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Task {
    subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Subtask {
    tests: Vec<Test>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Test {
    input: String,
    output: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Language {
    filename: String,
    build: Option<Command>,
    run: Command,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(HierarchicalLayer::new(2))
        .with(ErrorLayer::default())
        .init();

    let config = {
        let dir = if let Some(dir) = env::args().nth(1) {
            PathBuf::from(dir)
        } else {
            env::current_dir()?
        };

        tokio::task::spawn_blocking(move || loader::load(dir))
            .await
            .wrap_err("failed to load contests and judge configuration")??
    };

    CONFIG.set(config).unwrap();

    let app = Router::new()
        .route("/submit/:contest/:task", post(submit::submit))
        .layer(TraceLayer::new_for_http());

    let listener = {
        let addr = SocketAddr::from(([0; 4], 8000));
        tracing::info!("listening on {addr}");
        TcpListener::bind(addr).await?
    };

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

use std::{collections::HashMap, ffi::OsStr, net::SocketAddr, path::Path};

use axum::{routing::post, Router};
use color_eyre::eyre::WrapErr;
use config::Config;
use contest::Contest;
use once_cell::sync::OnceCell;
use tokio::{fs, net::TcpListener};
use tower_http::trace::TraceLayer;
use tracing_error::ErrorLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;

mod config;
mod contest;
mod sandbox;
mod submit;

static CONFIG: OnceCell<Config> = OnceCell::new();
static CONTESTS: OnceCell<HashMap<String, Contest>> = OnceCell::new();

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(HierarchicalLayer::default())
        .with(ErrorLayer::default())
        .try_init()
        .wrap_err("failed to initialize logging")?;

    let config = {
        let input = fs::read_to_string("config.toml")
            .await
            .wrap_err("failed to read config.toml")?;

        let config = Config::load(&input).wrap_err("failed to load config.toml")?;

        tracing::info!("loaded config.toml");
        config
    };

    CONFIG.set(config).unwrap();

    let contests = {
        let mut contests = HashMap::new();

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
        tracing::warn!("submissions directory not found, attempting to create it");
        fs::create_dir("submissions").await?;
    }

    let app = Router::new()
        .route("/:contest/:task", post(submit::handler))
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

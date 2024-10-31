pub mod database;
pub mod error;
pub mod logger;
pub mod middleware;
pub mod redis;
pub mod validation;

use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use tracing_appender::non_blocking::WorkerGuard;

pub type Config = ::config::Config;
pub type AppResult<T> = Result<T, error::Error>;

fn load_config() -> Result<Config> {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?;
    Ok(config)
}

async fn serve(config: config::Config, router: Router) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(
        config
            .get_string("general.listen")
            .unwrap_or("0.0.0.0:8000".to_string()),
    )
    .await?;
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

pub async fn run(router: Router) -> Result<WorkerGuard> {
    let config = load_config().with_context(|| "configuration parsing failed")?;
    database::init(&config)
        .await
        .with_context(|| "database connection failed")?;
    redis::init(&config)
        .await
        .with_context(|| "redis connection failed")?;
    let worker_guard = logger::init(&config);
    serve(config, router)
        .await
        .with_context(|| "service startup failed")?;
    Ok(worker_guard)
}

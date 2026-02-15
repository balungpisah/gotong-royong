mod error;
mod middleware;
mod routes;
mod state;
mod validation;

use gotong_infra::{config::AppConfig, logging::init_tracing};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config)?;

    let state = state::AppState::new(config.clone()).await?;
    let app = routes::router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!(%addr, "starting api");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|err| {
        tracing::error!(error = %err, "server exited");
        err
    })?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}

#[cfg(test)]
mod tests;

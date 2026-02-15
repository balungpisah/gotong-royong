use gotong_infra::{config::AppConfig, logging::init_tracing};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config)?;

    info!("worker starting (stub)");
    let _ = tokio::signal::ctrl_c().await;
    info!("worker shutdown");

    Ok(())
}

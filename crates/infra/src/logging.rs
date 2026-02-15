use crate::config::AppConfig;
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_tracing(config: &AppConfig) -> Result<()> {
    let filter =
        EnvFilter::try_new(config.log_level.clone()).unwrap_or_else(|_| EnvFilter::new("info"));

    if config.is_production() {
        fmt()
            .with_env_filter(filter)
            .json()
            .with_target(false)
            .init();
    } else {
        fmt()
            .with_env_filter(filter)
            .with_target(false)
            .compact()
            .init();
    }

    Ok(())
}

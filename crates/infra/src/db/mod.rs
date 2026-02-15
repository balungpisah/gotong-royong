use gotong_domain::ports::BoxFuture;
use gotong_domain::ports::db::{DbAdapter, DbError};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DbConfig {
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self {
            endpoint: config.surreal_endpoint.clone(),
            namespace: config.surreal_ns.clone(),
            database: config.surreal_db.clone(),
            username: config.surreal_user.clone(),
            password: config.surreal_pass.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SurrealAdapter {
    config: DbConfig,
}

impl SurrealAdapter {
    pub fn new(config: DbConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &DbConfig {
        &self.config
    }
}

impl DbAdapter for SurrealAdapter {
    fn name(&self) -> &'static str {
        "surrealdb"
    }

    fn health_check(&self) -> BoxFuture<'_, Result<(), DbError>> {
        // TODO: implement a real DB health check once the Surreal SDK is wired in.
        Box::pin(async { Ok(()) })
    }
}

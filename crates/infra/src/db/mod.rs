use gotong_domain::ports::BoxFuture;
use gotong_domain::ports::db::{DbAdapter, DbError};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use url::Url;

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
        let endpoint = self.config.endpoint.clone();
        let ns = self.config.namespace.clone();
        let db = self.config.database.clone();

        Box::pin(async move {
            let address = parse_socket_address(&endpoint)?;
            let connect = timeout(Duration::from_secs(2), TcpStream::connect(address))
                .await
                .map_err(|_| {
                    DbError::Unavailable("surreal endpoint connect timed out".to_string())
                })?;
            connect.map_err(|err| {
                DbError::Unavailable(format!("surreal endpoint connect failed: {err}"))
            })?;

            tracing::debug!(
                endpoint,
                namespace = ns,
                database = db,
                "surreal health check succeeded"
            );
            Ok(())
        })
    }
}

fn parse_socket_address(endpoint: &str) -> Result<String, DbError> {
    let normalized = if endpoint.contains("://") {
        endpoint.to_string()
    } else {
        format!("ws://{endpoint}")
    };
    let parsed = Url::parse(&normalized).map_err(|err| {
        DbError::Unavailable(format!("invalid surreal endpoint '{endpoint}': {err}"))
    })?;

    let scheme = parsed.scheme();
    let host = parsed.host_str().ok_or_else(|| {
        DbError::Unavailable(format!("missing surreal host in endpoint '{endpoint}'"))
    })?;
    let port = parsed.port_or_known_default().unwrap_or(match scheme {
        "wss" | "https" => 443,
        "http" | "ws" => 8000,
        _ => 8000,
    });
    Ok(format!("{host}:{port}"))
}

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
            endpoint: normalize_surreal_ws_endpoint(&config.surreal_endpoint),
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

/// Normalize a SurrealDB endpoint string into the `host:port` format expected by the Rust SDK when
/// using `engine::remote::ws::Ws`.
///
/// The SDK will append `/rpc` automatically for the websocket engine when given `host:port`.
fn normalize_surreal_ws_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() {
        return endpoint.to_string();
    }

    if endpoint.contains("://") {
        if let Ok(parsed) = Url::parse(endpoint) {
            if let Some(host) = parsed.host_str() {
                let scheme = parsed.scheme();
                let port = parsed.port_or_known_default().unwrap_or(match scheme {
                    "wss" | "https" => 443,
                    "http" | "ws" => 8000,
                    _ => 8000,
                });
                return format!("{host}:{port}");
            }
        }
        return endpoint.to_string();
    }

    // Allow users to pass `host:port/rpc` or similar; the SDK wants `host:port`.
    endpoint.split('/').next().unwrap_or(endpoint).to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_surreal_ws_endpoint;

    #[test]
    fn normalizes_ws_url_to_host_port() {
        assert_eq!(
            normalize_surreal_ws_endpoint("ws://127.0.0.1:8000"),
            "127.0.0.1:8000"
        );
        assert_eq!(
            normalize_surreal_ws_endpoint("ws://127.0.0.1:8000/rpc"),
            "127.0.0.1:8000"
        );
    }

    #[test]
    fn normalizes_bare_host_port() {
        assert_eq!(
            normalize_surreal_ws_endpoint("127.0.0.1:8000"),
            "127.0.0.1:8000"
        );
        assert_eq!(
            normalize_surreal_ws_endpoint("127.0.0.1:8000/rpc"),
            "127.0.0.1:8000"
        );
    }
}

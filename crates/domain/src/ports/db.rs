use thiserror::Error;

use super::BoxFuture;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("db unavailable: {0}")]
    Unavailable(String),
    #[error("db operation failed: {0}")]
    Operation(String),
}

pub trait DbAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn health_check(&self) -> BoxFuture<'_, Result<(), DbError>>;
}

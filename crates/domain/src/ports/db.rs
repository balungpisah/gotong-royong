use std::future::Future;
use std::pin::Pin;

use thiserror::Error;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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

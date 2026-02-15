pub mod auth;
pub mod error;
pub mod idempotency;
pub mod jobs;
pub mod ports;

pub type DomainResult<T> = Result<T, error::DomainError>;

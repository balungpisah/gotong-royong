pub mod error;
pub mod idempotency;
pub mod ports;

pub type DomainResult<T> = Result<T, error::DomainError>;

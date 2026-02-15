pub mod error;
pub mod ports;

pub type DomainResult<T> = Result<T, error::DomainError>;

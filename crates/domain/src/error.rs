use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
}

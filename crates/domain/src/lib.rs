pub mod auth;
pub mod contributions;
pub mod error;
pub mod evidence;
pub mod idempotency;
pub mod identity;
pub mod jobs;
pub mod ports;
pub mod transitions;
pub mod util;
pub mod vouches;

pub type DomainResult<T> = Result<T, error::DomainError>;

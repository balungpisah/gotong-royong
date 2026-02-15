pub mod auth;
pub mod chat;
pub mod contributions;
pub mod discovery;
pub mod error;
pub mod evidence;
pub mod idempotency;
pub mod identity;
pub mod jobs;
pub mod moderation;
pub mod ports;
pub mod siaga;
pub mod transitions;
pub mod util;
pub mod vault;
pub mod vouches;
pub mod webhook;

pub type DomainResult<T> = Result<T, error::DomainError>;

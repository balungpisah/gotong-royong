use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub mod chat;
pub mod contributions;
pub mod db;
pub mod discovery;
pub mod evidence;
pub mod idempotency;
pub mod jobs;
pub mod moderation;
pub mod siaga;
pub mod transitions;
pub mod vault;
pub mod vouches;
pub mod webhook;

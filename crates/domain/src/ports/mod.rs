use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub mod contributions;
pub mod db;
pub mod evidence;
pub mod idempotency;
pub mod jobs;
pub mod transitions;
pub mod vouches;

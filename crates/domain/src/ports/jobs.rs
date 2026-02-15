use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use super::BoxFuture;

#[derive(Debug, Error)]
pub enum JobQueueError {
    #[error("job queue unavailable: {0}")]
    Unavailable(String),
    #[error("job queue serialization error: {0}")]
    Serialization(String),
    #[error("job queue operation failed: {0}")]
    Operation(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    TransitionClose,
    ModerationAutoRelease,
    WebhookRetry,
    DigestSend,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JobEnvelope {
    pub job_id: String,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub request_id: String,
    pub correlation_id: String,
    pub attempt: u32,
    pub max_attempts: u32,
    pub run_at_ms: i64,
    pub created_at_ms: i64,
}

impl JobEnvelope {
    pub fn with_run_at(mut self, run_at_ms: i64) -> Self {
        self.run_at_ms = run_at_ms;
        self
    }

    pub fn next_attempt(&self) -> u32 {
        self.attempt.saturating_add(1)
    }
}

pub trait JobQueue: Send + Sync {
    fn enqueue(&self, job: &JobEnvelope) -> BoxFuture<'_, Result<(), JobQueueError>>;
    fn dequeue(
        &self,
        timeout: Duration,
    ) -> BoxFuture<'_, Result<Option<JobEnvelope>, JobQueueError>>;
    fn ack(&self, job_id: &str) -> BoxFuture<'_, Result<(), JobQueueError>>;
    fn promote_due(&self, now_ms: i64, limit: usize)
    -> BoxFuture<'_, Result<usize, JobQueueError>>;
    fn requeue_processing(&self, limit: usize) -> BoxFuture<'_, Result<usize, JobQueueError>>;
}

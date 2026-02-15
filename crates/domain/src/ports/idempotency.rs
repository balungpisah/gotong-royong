use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use super::BoxFuture;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyKey {
    pub entity_type: String,
    pub entity_id: String,
    pub request_id: String,
}

impl IdempotencyKey {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            request_id: request_id.into(),
        }
    }

    pub fn cache_key(&self, prefix: &str) -> String {
        format!(
            "{prefix}:{}:{}:{}",
            self.entity_type, self.entity_id, self.request_id
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IdempotencyResponse {
    pub status_code: u16,
    pub body: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdempotencyState {
    InProgress,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IdempotencyRecord {
    pub state: IdempotencyState,
    pub response: Option<IdempotencyResponse>,
}

impl IdempotencyRecord {
    pub fn in_progress() -> Self {
        Self {
            state: IdempotencyState::InProgress,
            response: None,
        }
    }

    pub fn completed(response: IdempotencyResponse) -> Self {
        Self {
            state: IdempotencyState::Completed,
            response: Some(response),
        }
    }
}

#[derive(Debug, Error)]
pub enum IdempotencyError {
    #[error("idempotency store unavailable: {0}")]
    Unavailable(String),
    #[error("idempotency serialization error: {0}")]
    Serialization(String),
    #[error("idempotency store error: {0}")]
    Store(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PutOutcome {
    Stored,
    Existing(IdempotencyRecord),
}

pub trait IdempotencyStore: Send + Sync {
    fn get(
        &self,
        key: &IdempotencyKey,
    ) -> BoxFuture<'_, Result<Option<IdempotencyRecord>, IdempotencyError>>;
    fn put_if_absent(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<PutOutcome, IdempotencyError>>;
    fn update(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> BoxFuture<'_, Result<(), IdempotencyError>>;
}

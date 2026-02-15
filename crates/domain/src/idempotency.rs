use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::ports::idempotency::{
    IdempotencyError, IdempotencyKey, IdempotencyRecord, IdempotencyResponse, IdempotencyState,
    IdempotencyStore, PutOutcome,
};

#[derive(Clone, Debug)]
pub struct IdempotencyConfig {
    pub in_progress_ttl: Duration,
    pub completed_ttl: Duration,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            in_progress_ttl: Duration::from_secs(60),
            completed_ttl: Duration::from_secs(60 * 60 * 24),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BeginOutcome {
    Started,
    InProgress,
    Replay(IdempotencyResponse),
}

#[derive(Clone)]
pub struct IdempotencyService {
    store: Arc<dyn IdempotencyStore>,
    config: IdempotencyConfig,
}

impl IdempotencyService {
    pub fn new(store: Arc<dyn IdempotencyStore>, config: IdempotencyConfig) -> Self {
        Self { store, config }
    }

    pub fn config(&self) -> &IdempotencyConfig {
        &self.config
    }

    pub async fn begin(&self, key: &IdempotencyKey) -> Result<BeginOutcome, IdempotencyError> {
        let record = IdempotencyRecord::in_progress();
        match self
            .store
            .put_if_absent(key, &record, self.config.in_progress_ttl)
            .await?
        {
            PutOutcome::Stored => Ok(BeginOutcome::Started),
            PutOutcome::Existing(existing) => match existing.state {
                IdempotencyState::InProgress => Ok(BeginOutcome::InProgress),
                IdempotencyState::Completed => {
                    let response = existing.response.ok_or_else(|| {
                        IdempotencyError::Store("completed record missing response".into())
                    })?;
                    Ok(BeginOutcome::Replay(response))
                }
            },
        }
    }

    pub async fn complete(
        &self,
        key: &IdempotencyKey,
        response: IdempotencyResponse,
    ) -> Result<(), IdempotencyError> {
        let record = IdempotencyRecord::completed(response);
        self.store
            .update(key, &record, self.config.completed_ttl)
            .await
    }
}

pub fn timer_request_id(transition_id: &str, closes_at: i64) -> String {
    format!("timer:{transition_id}:{closes_at}")
}

pub fn job_request_id(job_name: &str, entity_id: &str, scheduled_at: &str) -> String {
    format!("job:{job_name}:{entity_id}:{scheduled_at}")
}

#[derive(Clone, Debug)]
pub struct InMemoryIdempotencyStore {
    prefix: String,
    inner: Arc<Mutex<HashMap<String, MemoryEntry>>>,
}

#[derive(Clone, Debug)]
struct MemoryEntry {
    record: IdempotencyRecord,
    expires_at: Option<Instant>,
}

impl InMemoryIdempotencyStore {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn cache_key(&self, key: &IdempotencyKey) -> String {
        key.cache_key(&self.prefix)
    }

    fn is_expired(expires_at: Option<Instant>) -> bool {
        match expires_at {
            Some(deadline) => Instant::now() >= deadline,
            None => false,
        }
    }
}

impl IdempotencyStore for InMemoryIdempotencyStore {
    fn get(
        &self,
        key: &IdempotencyKey,
    ) -> crate::ports::BoxFuture<'_, Result<Option<IdempotencyRecord>, IdempotencyError>> {
        let cache_key = self.cache_key(key);
        let inner = self.inner.clone();
        Box::pin(async move {
            let mut guard = inner.lock().expect("idempotency store lock");
            if let Some(entry) = guard.get(&cache_key) {
                if Self::is_expired(entry.expires_at) {
                    guard.remove(&cache_key);
                    return Ok(None);
                }
                return Ok(Some(entry.record.clone()));
            }
            Ok(None)
        })
    }

    fn put_if_absent(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> crate::ports::BoxFuture<'_, Result<PutOutcome, IdempotencyError>> {
        let cache_key = self.cache_key(key);
        let inner = self.inner.clone();
        let record = record.clone();
        Box::pin(async move {
            let mut guard = inner.lock().expect("idempotency store lock");
            if let Some(entry) = guard.get(&cache_key) {
                if Self::is_expired(entry.expires_at) {
                    guard.remove(&cache_key);
                } else {
                    return Ok(PutOutcome::Existing(entry.record.clone()));
                }
            }

            let expires_at = if ttl.is_zero() {
                None
            } else {
                Some(Instant::now() + ttl)
            };
            guard.insert(cache_key, MemoryEntry { record, expires_at });
            Ok(PutOutcome::Stored)
        })
    }

    fn update(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> crate::ports::BoxFuture<'_, Result<(), IdempotencyError>> {
        let cache_key = self.cache_key(key);
        let inner = self.inner.clone();
        let record = record.clone();
        Box::pin(async move {
            let mut guard = inner.lock().expect("idempotency store lock");
            let expires_at = if ttl.is_zero() {
                None
            } else {
                Some(Instant::now() + ttl)
            };
            guard.insert(cache_key, MemoryEntry { record, expires_at });
            Ok(())
        })
    }
}

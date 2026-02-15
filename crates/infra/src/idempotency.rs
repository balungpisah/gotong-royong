use std::sync::Arc;
use std::time::Duration;

use gotong_domain::ports::idempotency::{
    IdempotencyError, IdempotencyKey, IdempotencyRecord, IdempotencyStore, PutOutcome,
};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tokio::sync::Mutex;

const DEFAULT_PREFIX: &str = "gotong:idemp";

#[derive(Clone)]
pub struct RedisIdempotencyStore {
    manager: Arc<Mutex<ConnectionManager>>,
    prefix: String,
}

impl RedisIdempotencyStore {
    pub async fn connect(redis_url: &str) -> Result<Self, IdempotencyError> {
        Self::connect_with_prefix(redis_url, DEFAULT_PREFIX).await
    }

    pub async fn connect_with_prefix(
        redis_url: &str,
        prefix: impl Into<String>,
    ) -> Result<Self, IdempotencyError> {
        let client = redis::Client::open(redis_url)
            .map_err(|err| IdempotencyError::Unavailable(err.to_string()))?;
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|err| IdempotencyError::Unavailable(err.to_string()))?;
        Ok(Self {
            manager: Arc::new(Mutex::new(manager)),
            prefix: prefix.into(),
        })
    }

    fn cache_key(&self, key: &IdempotencyKey) -> String {
        key.cache_key(&self.prefix)
    }

    fn ttl_ms(ttl: Duration) -> u64 {
        let ms = ttl.as_millis() as u64;
        if ms == 0 { 1 } else { ms }
    }

    async fn serialize_record(record: &IdempotencyRecord) -> Result<String, IdempotencyError> {
        serde_json::to_string(record)
            .map_err(|err| IdempotencyError::Serialization(err.to_string()))
    }

    async fn deserialize_record(value: &str) -> Result<IdempotencyRecord, IdempotencyError> {
        serde_json::from_str(value).map_err(|err| IdempotencyError::Serialization(err.to_string()))
    }
}

impl IdempotencyStore for RedisIdempotencyStore {
    fn get(
        &self,
        key: &IdempotencyKey,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<Option<IdempotencyRecord>, IdempotencyError>>
    {
        let cache_key = self.cache_key(key);
        let manager = self.manager.clone();
        Box::pin(async move {
            let mut conn = manager.lock().await;
            let value: Option<String> = conn
                .get(cache_key)
                .await
                .map_err(|err| IdempotencyError::Store(err.to_string()))?;
            match value {
                Some(payload) => Ok(Some(Self::deserialize_record(&payload).await?)),
                None => Ok(None),
            }
        })
    }

    fn put_if_absent(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<PutOutcome, IdempotencyError>> {
        let cache_key = self.cache_key(key);
        let manager = self.manager.clone();
        let record = record.clone();
        Box::pin(async move {
            let payload = Self::serialize_record(&record).await?;
            let ttl_ms = Self::ttl_ms(ttl);
            let mut conn = manager.lock().await;
            let result: Option<String> = redis::cmd("SET")
                .arg(&cache_key)
                .arg(payload)
                .arg("NX")
                .arg("PX")
                .arg(ttl_ms)
                .query_async(&mut *conn)
                .await
                .map_err(|err| IdempotencyError::Store(err.to_string()))?;

            if result.is_some() {
                return Ok(PutOutcome::Stored);
            }

            let existing: Option<String> = conn
                .get(&cache_key)
                .await
                .map_err(|err| IdempotencyError::Store(err.to_string()))?;
            let record = match existing {
                Some(payload) => Some(Self::deserialize_record(&payload).await?),
                None => None,
            };
            Ok(record
                .map(PutOutcome::Existing)
                .unwrap_or(PutOutcome::Stored))
        })
    }

    fn update(
        &self,
        key: &IdempotencyKey,
        record: &IdempotencyRecord,
        ttl: Duration,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<(), IdempotencyError>> {
        let cache_key = self.cache_key(key);
        let manager = self.manager.clone();
        let record = record.clone();
        Box::pin(async move {
            let payload = Self::serialize_record(&record).await?;
            let ttl_ms = Self::ttl_ms(ttl);
            let mut conn = manager.lock().await;
            let result: Option<String> = redis::cmd("SET")
                .arg(&cache_key)
                .arg(&payload)
                .arg("XX")
                .arg("PX")
                .arg(ttl_ms)
                .query_async(&mut *conn)
                .await
                .map_err(|err| IdempotencyError::Store(err.to_string()))?;

            if result.is_none() {
                let _: String = redis::cmd("SET")
                    .arg(&cache_key)
                    .arg(payload)
                    .arg("PX")
                    .arg(ttl_ms)
                    .query_async(&mut *conn)
                    .await
                    .map_err(|err| IdempotencyError::Store(err.to_string()))?;
            }

            Ok(())
        })
    }
}

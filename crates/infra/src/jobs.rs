use std::time::Duration;

use gotong_domain::ports::jobs::{JobEnvelope, JobQueue, JobQueueError};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

const DEFAULT_PREFIX: &str = "gotong:jobs";

#[derive(Clone)]
pub struct RedisJobQueue {
    manager: ConnectionManager,
    ready_key: String,
    delayed_key: String,
}

impl RedisJobQueue {
    pub async fn connect(redis_url: &str) -> Result<Self, JobQueueError> {
        Self::connect_with_prefix(redis_url, DEFAULT_PREFIX).await
    }

    pub async fn connect_with_prefix(
        redis_url: &str,
        prefix: impl Into<String>,
    ) -> Result<Self, JobQueueError> {
        let client = redis::Client::open(redis_url)
            .map_err(|err| JobQueueError::Unavailable(err.to_string()))?;
        let manager = ConnectionManager::new(client)
            .await
            .map_err(|err| JobQueueError::Unavailable(err.to_string()))?;
        let prefix = prefix.into();
        Ok(Self {
            manager,
            ready_key: format!("{prefix}:ready"),
            delayed_key: format!("{prefix}:delayed"),
        })
    }

    fn serialize(job: &JobEnvelope) -> Result<String, JobQueueError> {
        serde_json::to_string(job).map_err(|err| JobQueueError::Serialization(err.to_string()))
    }

    fn deserialize(payload: &str) -> Result<JobEnvelope, JobQueueError> {
        serde_json::from_str(payload).map_err(|err| JobQueueError::Serialization(err.to_string()))
    }
}

impl JobQueue for RedisJobQueue {
    fn enqueue(
        &self,
        job: &JobEnvelope,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<(), JobQueueError>> {
        let payload = match Self::serialize(job) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let ready_key = self.ready_key.clone();
        let delayed_key = self.delayed_key.clone();
        let run_at_ms = job.run_at_ms;
        Box::pin(async move {
            let mut conn = self.manager.clone();
            if run_at_ms <= gotong_domain::jobs::now_ms() {
                let _: i64 = conn
                    .rpush(ready_key, payload)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            } else {
                let _: i64 = redis::cmd("ZADD")
                    .arg(&delayed_key)
                    .arg(run_at_ms)
                    .arg(payload)
                    .query_async(&mut conn)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            }
            Ok(())
        })
    }

    fn dequeue(
        &self,
        timeout: Duration,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<Option<JobEnvelope>, JobQueueError>> {
        let ready_key = self.ready_key.clone();
        let timeout_secs = timeout.as_secs() as usize;
        Box::pin(async move {
            let mut conn = self.manager.clone();
            let result: Option<(String, String)> = redis::cmd("BRPOP")
                .arg(&ready_key)
                .arg(timeout_secs)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            match result {
                Some((_key, payload)) => Ok(Some(Self::deserialize(&payload)?)),
                None => Ok(None),
            }
        })
    }

    fn promote_due(
        &self,
        now_ms: i64,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<usize, JobQueueError>> {
        let ready_key = self.ready_key.clone();
        let delayed_key = self.delayed_key.clone();
        Box::pin(async move {
            let mut conn = self.manager.clone();
            let mut moved = 0usize;
            for _ in 0..limit {
                let result: Vec<(String, f64)> = redis::cmd("ZPOPMIN")
                    .arg(&delayed_key)
                    .arg(1)
                    .query_async(&mut conn)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                let Some((payload, score)) = result.into_iter().next() else {
                    break;
                };
                if score as i64 > now_ms {
                    let _: i64 = redis::cmd("ZADD")
                        .arg(&delayed_key)
                        .arg(score)
                        .arg(payload)
                        .query_async(&mut conn)
                        .await
                        .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                    break;
                }
                let _: i64 = conn
                    .lpush(&ready_key, payload)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                moved += 1;
            }
            Ok(moved)
        })
    }
}

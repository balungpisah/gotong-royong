use std::time::Duration;

use gotong_domain::ports::jobs::{JobEnvelope, JobQueue, JobQueueError};
use redis::AsyncCommands;
use redis::Value;
use redis::aio::ConnectionManager;

const DEFAULT_PREFIX: &str = "gotong:jobs";

#[derive(Clone)]
pub struct RedisJobQueue {
    manager: ConnectionManager,
    ready_key: String,
    delayed_key: String,
    processing_key: String,
    payload_key: String,
}

#[derive(Debug, Clone)]
pub struct JobQueueMetricsSnapshot {
    pub ready: u64,
    pub delayed: u64,
    pub processing: u64,
    pub oldest_delayed_ms: Option<i64>,
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
            processing_key: format!("{prefix}:processing"),
            payload_key: format!("{prefix}:payloads"),
        })
    }

    fn serialize(job: &JobEnvelope) -> Result<String, JobQueueError> {
        serde_json::to_string(job).map_err(|err| JobQueueError::Serialization(err.to_string()))
    }

    fn deserialize(payload: &str) -> Result<JobEnvelope, JobQueueError> {
        serde_json::from_str(payload).map_err(|err| JobQueueError::Serialization(err.to_string()))
    }

    pub async fn enqueue_if_absent(
        &self,
        job: &JobEnvelope,
        dedupe_ttl_ms: u64,
    ) -> Result<bool, JobQueueError> {
        let payload = Self::serialize(job)?;
        let payload_key = self.payload_key.clone();
        let ready_key = self.ready_key.clone();
        let delayed_key = self.delayed_key.clone();
        let dedupe_key = format!("{}:dedupe:{job_id}", self.payload_key, job_id = job.job_id);
        let run_at_ms = job.run_at_ms;
        let job_id = job.job_id.clone();
        let now_ms = gotong_domain::jobs::now_ms();
        let dedupe_ttl_ms = dedupe_ttl_ms.max(1);

        let mut conn = self.manager.clone();
        let script = redis::Script::new(
            r#"
                local marker_key = KEYS[4]
                local payload_key = KEYS[1]
                local ready_key = KEYS[2]
                local delayed_key = KEYS[3]
                local job_id = ARGV[1]
                local payload = ARGV[2]
                local run_at_ms = tonumber(ARGV[3])
                local now_ms = tonumber(ARGV[4])
                local dedupe_ttl_ms = tonumber(ARGV[5])

                if redis.call('SET', marker_key, 1, 'PX', dedupe_ttl_ms, 'NX') == false then
                    return 0
                end

                redis.call('HSET', payload_key, job_id, payload)
                if run_at_ms <= now_ms then
                    redis.call('RPUSH', ready_key, job_id)
                else
                    redis.call('ZADD', delayed_key, run_at_ms, job_id)
                end
                return 1
            "#,
        );
        let inserted: i32 = script
            .key(&payload_key)
            .key(&ready_key)
            .key(&delayed_key)
            .key(&dedupe_key)
            .arg(&job_id)
            .arg(payload)
            .arg(run_at_ms)
            .arg(now_ms)
            .arg(dedupe_ttl_ms as i64)
            .invoke_async(&mut conn)
            .await
            .map_err(|err| JobQueueError::Operation(err.to_string()))?;

        Ok(inserted == 1)
    }

    pub async fn restore_processing_with_retry_delay(
        &self,
        job: &JobEnvelope,
    ) -> Result<(), JobQueueError> {
        let payload = Self::serialize(job)?;
        let payload_key = self.payload_key.clone();
        let processing_key = self.processing_key.clone();
        let ready_key = self.ready_key.clone();
        let delayed_key = self.delayed_key.clone();
        let job_id = job.job_id.clone();
        let run_at_ms = job.run_at_ms;
        let now_ms = gotong_domain::jobs::now_ms();
        let mut conn = self.manager.clone();

        let mut pipeline = redis::pipe();
        pipeline.atomic();
        pipeline
            .cmd("HSET")
            .arg(&payload_key)
            .arg(&job_id)
            .arg(payload);
        pipeline
            .cmd("LREM")
            .arg(&processing_key)
            .arg(1)
            .arg(&job_id);
        if run_at_ms <= now_ms {
            pipeline.cmd("LPUSH").arg(&ready_key).arg(&job_id);
        } else {
            pipeline
                .cmd("ZADD")
                .arg(&delayed_key)
                .arg(run_at_ms)
                .arg(&job_id);
        }

        let _: Vec<Value> = pipeline
            .query_async(&mut conn)
            .await
            .map_err(|err| JobQueueError::Operation(err.to_string()))?;
        Ok(())
    }

    pub async fn metrics_snapshot(&self) -> Result<JobQueueMetricsSnapshot, JobQueueError> {
        let mut conn = self.manager.clone();
        let ready: u64 = conn
            .llen(&self.ready_key)
            .await
            .map_err(|err| JobQueueError::Operation(err.to_string()))?;
        let delayed: u64 = conn
            .zcard(&self.delayed_key)
            .await
            .map_err(|err| JobQueueError::Operation(err.to_string()))?;
        let processing: u64 = conn
            .llen(&self.processing_key)
            .await
            .map_err(|err| JobQueueError::Operation(err.to_string()))?;

        let oldest_delayed_ms: Option<i64> = if delayed == 0 {
            None
        } else {
            let mut conn = self.manager.clone();
            let result: Vec<(String, f64)> = redis::cmd("ZRANGE")
                .arg(&self.delayed_key)
                .arg(0)
                .arg(0)
                .arg("WITHSCORES")
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            result.into_iter().next().map(|(_, score)| score as i64)
        };

        Ok(JobQueueMetricsSnapshot {
            ready,
            delayed,
            processing,
            oldest_delayed_ms,
        })
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
        let payload_key = self.payload_key.clone();
        let run_at_ms = job.run_at_ms;
        let job_id = job.job_id.clone();
        Box::pin(async move {
            let mut conn = self.manager.clone();
            let _: i64 = redis::cmd("HSET")
                .arg(&payload_key)
                .arg(&job_id)
                .arg(payload)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            if run_at_ms <= gotong_domain::jobs::now_ms() {
                let _: i64 = conn
                    .rpush(ready_key, job_id)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            } else {
                let _: i64 = redis::cmd("ZADD")
                    .arg(&delayed_key)
                    .arg(run_at_ms)
                    .arg(job_id)
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
        let processing_key = self.processing_key.clone();
        let payload_key = self.payload_key.clone();
        let timeout_secs = timeout.as_secs() as usize;
        Box::pin(async move {
            let mut conn = self.manager.clone();
            let result: Option<String> = redis::cmd("BRPOPLPUSH")
                .arg(&ready_key)
                .arg(&processing_key)
                .arg(timeout_secs)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            match result {
                Some(job_id) => {
                    let payload: Option<String> = redis::cmd("HGET")
                        .arg(&payload_key)
                        .arg(&job_id)
                        .query_async(&mut conn)
                        .await
                        .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                    let Some(payload) = payload else {
                        let _: i64 = redis::cmd("LREM")
                            .arg(&processing_key)
                            .arg(1)
                            .arg(&job_id)
                            .query_async(&mut conn)
                            .await
                            .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                        return Err(JobQueueError::Operation(format!(
                            "missing payload for job_id {job_id}"
                        )));
                    };
                    Ok(Some(Self::deserialize(&payload)?))
                }
                None => Ok(None),
            }
        })
    }

    fn ack(&self, job_id: &str) -> gotong_domain::ports::BoxFuture<'_, Result<(), JobQueueError>> {
        let processing_key = self.processing_key.clone();
        let payload_key = self.payload_key.clone();
        let job_id = job_id.to_string();
        Box::pin(async move {
            let mut conn = self.manager.clone();
            let _: i64 = redis::cmd("LREM")
                .arg(&processing_key)
                .arg(1)
                .arg(&job_id)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            let _: i64 = redis::cmd("HDEL")
                .arg(&payload_key)
                .arg(&job_id)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            Ok(())
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
                let Some((job_id, score)) = result.into_iter().next() else {
                    break;
                };
                if score as i64 > now_ms {
                    let _: i64 = redis::cmd("ZADD")
                        .arg(&delayed_key)
                        .arg(score)
                        .arg(job_id)
                        .query_async(&mut conn)
                        .await
                        .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                    break;
                }
                let _: i64 = conn
                    .lpush(&ready_key, job_id)
                    .await
                    .map_err(|err| JobQueueError::Operation(err.to_string()))?;
                moved += 1;
            }
            Ok(moved)
        })
    }

    fn requeue_processing(
        &self,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, Result<usize, JobQueueError>> {
        let processing_key = self.processing_key.clone();
        let ready_key = self.ready_key.clone();
        Box::pin(async move {
            if limit == 0 {
                return Ok(0);
            }
            let mut conn = self.manager.clone();
            let job_ids: Vec<String> = redis::cmd("LRANGE")
                .arg(&processing_key)
                .arg(0)
                .arg((limit.saturating_sub(1)) as i64)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            if job_ids.is_empty() {
                return Ok(0);
            }
            let _: i64 = redis::cmd("RPUSH")
                .arg(&ready_key)
                .arg(job_ids.clone())
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            let _: String = redis::cmd("LTRIM")
                .arg(&processing_key)
                .arg(job_ids.len() as i64)
                .arg(-1)
                .query_async(&mut conn)
                .await
                .map_err(|err| JobQueueError::Operation(err.to_string()))?;
            Ok(job_ids.len())
        })
    }
}

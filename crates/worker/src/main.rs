use std::time::Duration;

use gotong_domain::jobs::{backoff_ms, now_ms};
use gotong_domain::ports::jobs::{JobEnvelope, JobQueue, JobQueueError, JobType};
use gotong_infra::{config::AppConfig, jobs::RedisJobQueue, logging::init_tracing};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config)?;

    let queue =
        RedisJobQueue::connect_with_prefix(&config.redis_url, config.worker_queue_prefix.clone())
            .await?;

    if std::env::var("WORKER_ENQUEUE_SAMPLE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        let sample = JobEnvelope {
            job_id: Uuid::now_v7().to_string(),
            job_type: JobType::WebhookRetry,
            payload: json!({ "note": "sample job" }),
            request_id: format!("job:sample:{}", Uuid::now_v7()),
            correlation_id: format!("corr:{}", Uuid::now_v7()),
            attempt: 1,
            max_attempts: 5,
            run_at_ms: now_ms(),
            created_at_ms: now_ms(),
        };
        queue.enqueue(&sample).await?;
        info!(job_id = %sample.job_id, "enqueued sample job");
    }

    let worker = Worker::new(queue, config);
    info!("worker starting");
    worker.run().await?;

    Ok(())
}

struct Worker {
    queue: RedisJobQueue,
    config: AppConfig,
}

impl Worker {
    fn new(queue: RedisJobQueue, config: AppConfig) -> Self {
        Self { queue, config }
    }

    async fn run(&self) -> Result<(), JobQueueError> {
        loop {
            let moved = self
                .queue
                .requeue_processing(self.config.worker_promote_batch)
                .await?;
            if moved == 0 {
                break;
            }
        }

        loop {
            let now = now_ms();
            let _ = self
                .queue
                .promote_due(now, self.config.worker_promote_batch)
                .await?;

            match self.queue.dequeue(Duration::from_secs(2)).await? {
                Some(job) => {
                    if let Err(err) = handle_job(&job).await {
                        self.handle_failure(job, err).await?;
                    } else {
                        self.queue.ack(&job.job_id).await?;
                    }
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(self.config.worker_poll_interval_ms))
                        .await;
                }
            }
        }
    }

    async fn handle_failure(
        &self,
        mut job: JobEnvelope,
        err: anyhow::Error,
    ) -> Result<(), JobQueueError> {
        self.queue.ack(&job.job_id).await?;
        if job.attempt >= job.max_attempts {
            error!(
                job_id = %job.job_id,
                attempt = job.attempt,
                error = %err,
                "job failed permanently"
            );
            return Ok(());
        }

        let delay = backoff_ms(
            self.config.worker_backoff_base_ms,
            job.attempt,
            self.config.worker_backoff_max_ms,
        );
        job.attempt = job.attempt.saturating_add(1);
        job.run_at_ms = now_ms() + delay as i64;
        self.queue.enqueue(&job).await?;

        error!(
            job_id = %job.job_id,
            attempt = job.attempt,
            next_run_at_ms = job.run_at_ms,
            error = %err,
            "job failed, scheduled retry"
        );
        Ok(())
    }
}

async fn handle_job(job: &JobEnvelope) -> anyhow::Result<()> {
    match job.job_type {
        JobType::TransitionClose => {
            info!(job_id = %job.job_id, "handling transition close (stub)");
        }
        JobType::ModerationAutoRelease => {
            info!(job_id = %job.job_id, "handling moderation auto-release (stub)");
        }
        JobType::WebhookRetry => {
            info!(job_id = %job.job_id, "handling webhook retry (stub)");
        }
        JobType::DigestSend => {
            info!(job_id = %job.job_id, "handling digest send (stub)");
        }
    }
    Ok(())
}

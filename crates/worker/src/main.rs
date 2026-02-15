use std::sync::Arc;
use std::time::Duration;

use gotong_domain::ports::jobs::{JobQueue, JobQueueError, JobType};
use gotong_domain::{
    auth::Role,
    identity::ActorIdentity,
    jobs::{TransitionClosePayload, backoff_ms, now_ms},
    moderation::{ModerationAutoReleaseCommand, ModerationService},
    ports::{
        jobs::JobEnvelope, moderation::ModerationRepository, transitions::TrackTransitionRepository,
    },
    transitions::{
        TrackTransitionInput, TrackTransitionService, TransitionAction, TransitionMechanism,
    },
};
use gotong_infra::{
    config::AppConfig,
    db::DbConfig,
    jobs::RedisJobQueue,
    logging::init_tracing,
    repositories::{SurrealModerationRepository, SurrealTrackTransitionRepository},
};
use serde_json::json;
use tracing::{error, info, warn};
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

    let mut transition_repo = None;
    let mut moderation_repo = None;
    let backend = config.data_backend.trim().to_ascii_lowercase();
    if matches!(backend.as_str(), "surreal" | "surrealdb" | "tikv") {
        let db_config = DbConfig::from_app_config(&config);
        let repository = SurrealTrackTransitionRepository::new(&db_config).await?;
        transition_repo = Some(Arc::new(repository) as Arc<dyn TrackTransitionRepository>);
        let moderation_repository = SurrealModerationRepository::new(&db_config).await?;
        moderation_repo = Some(Arc::new(moderation_repository) as Arc<dyn ModerationRepository>);
    }

    let worker = Worker::new(queue, config, transition_repo, moderation_repo);
    info!("worker starting");
    worker.run().await?;

    Ok(())
}

struct Worker {
    queue: RedisJobQueue,
    config: AppConfig,
    transition_repo: Option<Arc<dyn TrackTransitionRepository>>,
    moderation_repo: Option<Arc<dyn ModerationRepository>>,
}

impl Worker {
    fn new(
        queue: RedisJobQueue,
        config: AppConfig,
        transition_repo: Option<Arc<dyn TrackTransitionRepository>>,
        moderation_repo: Option<Arc<dyn ModerationRepository>>,
    ) -> Self {
        Self {
            queue,
            config,
            transition_repo,
            moderation_repo,
        }
    }

    async fn run(&self) -> Result<(), JobQueueError> {
        loop {
            let moved = match self
                .queue
                .requeue_processing(self.config.worker_promote_batch)
                .await
            {
                Ok(moved) => moved,
                Err(err) => {
                    warn!(
                        error = %err,
                        "failed to requeue processing jobs, retrying shortly"
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if moved == 0 {
                break;
            }
        }

        loop {
            let now = now_ms();
            if let Err(err) = self
                .queue
                .promote_due(now, self.config.worker_promote_batch)
                .await
            {
                warn!(error = %err, "failed to promote due jobs, continuing");
            }

            match self.queue.dequeue(Duration::from_secs(2)).await {
                Ok(Some(job)) => {
                    if let Err(err) = handle_job(
                        &job,
                        self.transition_repo.as_ref(),
                        self.moderation_repo.as_ref(),
                    )
                    .await
                    {
                        let job_id = job.job_id.clone();
                        if let Err(handle_err) = self.handle_failure(job, err).await {
                            warn!(
                                error = %handle_err,
                                job_id = %job_id,
                                "failed to handle job failure path"
                            );
                        }
                    } else if let Err(err) = self.queue.ack(&job.job_id).await {
                        warn!(
                            error = %err,
                            job_id = %job.job_id,
                            "failed to acknowledge successful job"
                        );
                    }
                }
                Ok(None) => {
                    tokio::time::sleep(Duration::from_millis(self.config.worker_poll_interval_ms))
                        .await;
                }
                Err(err) => {
                    warn!(error = %err, "failed to dequeue job, retrying shortly");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn handle_failure(
        &self,
        mut job: JobEnvelope,
        err: anyhow::Error,
    ) -> Result<(), JobQueueError> {
        if job.attempt >= job.max_attempts {
            self.queue.ack(&job.job_id).await?;
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
        if let Err(enqueue_err) = self.queue.enqueue(&job).await {
            warn!(
                job_id = %job.job_id,
                attempt = %job.attempt,
                error = %enqueue_err,
                "failed to enqueue retry job; attempting to move processing job back to ready queue"
            );
            if let Err(requeue_err) = self.queue.restore_processing_with_retry_delay(&job).await {
                warn!(
                    job_id = %job.job_id,
                    error = %requeue_err,
                    "failed to restore processing job for retry"
                );
            }
            return Err(enqueue_err);
        }

        self.queue.ack(&job.job_id).await?;

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

async fn handle_job(
    job: &JobEnvelope,
    transition_repo: Option<&Arc<dyn TrackTransitionRepository>>,
    moderation_repo: Option<&Arc<dyn ModerationRepository>>,
) -> anyhow::Result<()> {
    match job.job_type {
        JobType::TransitionClose => {
            let Some(repo) = transition_repo else {
                warn!(
                    job_id = %job.job_id,
                    "skipping transition close job: transition repository is unavailable"
                );
                return Ok(());
            };
            let payload = parse_transition_close_payload(job)?;

            let actor = ActorIdentity {
                user_id: "system".to_string(),
                username: "system".to_string(),
            };
            let service = TrackTransitionService::new(repo.clone());
            let input = TrackTransitionInput {
                track: payload.track,
                entity_id: payload.entity_id,
                from_stage: payload.from_stage,
                to_stage: payload.to_stage,
                transition_action: TransitionAction::Object,
                transition_type: TransitionMechanism::Timer,
                mechanism: TransitionMechanism::Timer,
                request_id: payload.request_id,
                correlation_id: payload.correlation_id,
                track_roles: vec![],
                gate_status: payload.gate_status,
                gate_metadata: payload.gate_metadata,
                occurred_at_ms: Some(payload.closes_at_ms),
                request_ts_ms: Some(payload.request_ts_ms),
                closes_at_ms: Some(payload.closes_at_ms),
            };
            service
                .track_state_transition(actor, Role::System, input)
                .await?;
        }
        JobType::ModerationAutoRelease => {
            let Some(repo) = moderation_repo else {
                warn!(
                    job_id = %job.job_id,
                    "skipping moderation auto-release job: moderation repository is unavailable"
                );
                return Ok(());
            };
            let payload = parse_moderation_auto_release_payload(job)?;
            let actor = ActorIdentity {
                user_id: "system".to_string(),
                username: "system".to_string(),
            };
            let service = ModerationService::new(repo.clone());
            let command = ModerationAutoReleaseCommand {
                content_id: payload.content_id,
                hold_decision_request_id: payload.hold_decision_request_id,
                request_id: payload.request_id,
                correlation_id: payload.correlation_id,
                scheduled_ms: payload.scheduled_ms,
                request_ts_ms: Some(payload.request_ts_ms),
            };
            service
                .apply_auto_release(actor, Role::System, command)
                .await?;
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

fn parse_transition_close_payload(job: &JobEnvelope) -> anyhow::Result<TransitionClosePayload> {
    let payload: TransitionClosePayload = serde_json::from_value(job.payload.clone())
        .map_err(|err| anyhow::anyhow!("invalid transition close payload: {err}"))?;

    if payload.closes_at_ms < 0 {
        return Err(anyhow::anyhow!(
            "invalid close payload: closes_at_ms must be non-negative"
        ));
    }

    Ok(payload)
}

fn parse_moderation_auto_release_payload(
    job: &JobEnvelope,
) -> anyhow::Result<gotong_domain::jobs::ModerationAutoReleasePayload> {
    let payload: gotong_domain::jobs::ModerationAutoReleasePayload =
        serde_json::from_value(job.payload.clone())
            .map_err(|err| anyhow::anyhow!("invalid moderation auto-release payload: {err}"))?;
    if payload.scheduled_ms < 0 {
        return Err(anyhow::anyhow!(
            "invalid moderation auto-release payload: scheduled_ms must be non-negative"
        ));
    }
    if payload.content_id.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid moderation auto-release payload: content_id is required"
        ));
    }
    if payload.hold_decision_request_id.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid moderation auto-release payload: hold_decision_request_id is required"
        ));
    }
    if payload.request_id.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid moderation auto-release payload: request_id is required"
        ));
    }
    Ok(payload)
}

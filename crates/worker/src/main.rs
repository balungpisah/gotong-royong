use std::sync::Arc;
use std::time::Duration;

use gotong_domain::ports::jobs::{JobQueue, JobQueueError, JobType};
use gotong_domain::ports::webhook::WebhookOutboxRepository;
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
    webhook::{
        WebhookDeliveryLog, WebhookDeliveryResult, WebhookOutboxEvent, WebhookOutboxStatus,
        WebhookOutboxUpdate,
    },
};
use gotong_infra::{
    config::AppConfig,
    db::DbConfig,
    jobs::RedisJobQueue,
    logging::init_tracing,
    repositories::{
        SurrealModerationRepository, SurrealTrackTransitionRepository,
        SurrealWebhookOutboxRepository,
    },
};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use tracing::{error, info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

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
    let mut webhook_outbox_repo = None;
    let backend = config.data_backend.trim().to_ascii_lowercase();
    if matches!(backend.as_str(), "surreal" | "surrealdb" | "tikv") {
        let db_config = DbConfig::from_app_config(&config);
        let repository = SurrealTrackTransitionRepository::new(&db_config).await?;
        transition_repo = Some(Arc::new(repository) as Arc<dyn TrackTransitionRepository>);
        let moderation_repository = SurrealModerationRepository::new(&db_config).await?;
        moderation_repo = Some(Arc::new(moderation_repository) as Arc<dyn ModerationRepository>);
        let webhook_repository = SurrealWebhookOutboxRepository::new(&db_config).await?;
        webhook_outbox_repo =
            Some(Arc::new(webhook_repository) as Arc<dyn WebhookOutboxRepository>);
    }

    let worker = Worker::new(
        queue,
        config,
        transition_repo,
        moderation_repo,
        webhook_outbox_repo,
    );
    info!("worker starting");
    worker.run().await?;

    Ok(())
}

struct Worker {
    queue: RedisJobQueue,
    config: AppConfig,
    transition_repo: Option<Arc<dyn TrackTransitionRepository>>,
    moderation_repo: Option<Arc<dyn ModerationRepository>>,
    webhook_outbox_repo: Option<Arc<dyn WebhookOutboxRepository>>,
}

impl Worker {
    fn new(
        queue: RedisJobQueue,
        config: AppConfig,
        transition_repo: Option<Arc<dyn TrackTransitionRepository>>,
        moderation_repo: Option<Arc<dyn ModerationRepository>>,
        webhook_outbox_repo: Option<Arc<dyn WebhookOutboxRepository>>,
    ) -> Self {
        Self {
            queue,
            config,
            transition_repo,
            moderation_repo,
            webhook_outbox_repo,
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
                        &self.config,
                        &job,
                        self.transition_repo.as_ref(),
                        self.moderation_repo.as_ref(),
                        self.webhook_outbox_repo.as_ref(),
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
    config: &AppConfig,
    job: &JobEnvelope,
    transition_repo: Option<&Arc<dyn TrackTransitionRepository>>,
    moderation_repo: Option<&Arc<dyn ModerationRepository>>,
    webhook_outbox_repo: Option<&Arc<dyn WebhookOutboxRepository>>,
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
            let Some(repo) = webhook_outbox_repo else {
                warn!(
                    job_id = %job.job_id,
                    "skipping webhook retry job: webhook outbox repository is unavailable"
                );
                return Ok(());
            };
            handle_webhook_retry(config, repo.clone(), job).await?;
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

#[derive(Debug)]
struct WebhookRequestEnvelope {
    request_body: Vec<u8>,
    request_body_sha256: String,
}

enum WebhookDeliveryResultClass {
    Success,
    Retryable,
    Terminal,
}

struct DeliveryResponse {
    status_code: Option<u16>,
    response_body_sha256: Option<String>,
    error_message: String,
}

async fn handle_webhook_retry(
    config: &AppConfig,
    repo: Arc<dyn WebhookOutboxRepository>,
    job: &JobEnvelope,
) -> anyhow::Result<()> {
    if !config.webhook_enabled {
        return Ok(());
    }

    let payload = parse_webhook_retry_payload(job)?;
    let event = repo
        .get(&payload.event_id)
        .await
        .map_err(|err| anyhow::anyhow!("failed to load webhook outbox event: {err}"))?
        .ok_or_else(|| anyhow::anyhow!("webhook event not found: {}", payload.event_id))?;

    if matches!(
        event.status,
        WebhookOutboxStatus::Delivered | WebhookOutboxStatus::DeadLetter
    ) {
        return Ok(());
    }

    let request = build_webhook_request(&event)?;
    let response = send_webhook_request(config, &request, &event).await;

    let (delivery_class, status_code, response_body_sha256, error_message) = match response {
        Ok(delivery) => (
            classify_webhook_response(delivery.status_code),
            delivery.status_code,
            delivery.response_body_sha256,
            delivery.error_message,
        ),
        Err(err) => (
            WebhookDeliveryResultClass::Retryable,
            None,
            None,
            err.to_string(),
        ),
    };

    let update = webhook_outbox_update(&event, &delivery_class, status_code, &error_message, job);
    repo.update(&event.event_id, &update)
        .await
        .map_err(|err| anyhow::anyhow!("failed to update webhook outbox event: {err}"))?;

    let log = webhook_delivery_log(
        &event,
        job.attempt,
        &delivery_class,
        status_code,
        request.request_body_sha256,
        response_body_sha256,
        error_message,
    );
    repo.append_log(&log)
        .await
        .map_err(|err| anyhow::anyhow!("failed to append webhook delivery log: {err}"))?;

    if matches!(delivery_class, WebhookDeliveryResultClass::Retryable)
        && job.attempt < job.max_attempts
    {
        return Err(anyhow::anyhow!(
            "webhook delivery is retryable (status: {status_code:?})"
        ));
    }

    Ok(())
}

fn parse_webhook_retry_payload(
    job: &JobEnvelope,
) -> anyhow::Result<gotong_domain::jobs::WebhookRetryPayload> {
    let payload: gotong_domain::jobs::WebhookRetryPayload =
        serde_json::from_value(job.payload.clone())
            .map_err(|err| anyhow::anyhow!("invalid webhook retry payload: {err}"))?;
    if payload.event_id.trim().is_empty() {
        return Err(anyhow::anyhow!("webhook retry payload missing event_id"));
    }
    Ok(payload)
}

fn build_webhook_request(event: &WebhookOutboxEvent) -> anyhow::Result<WebhookRequestEnvelope> {
    let request_body = serde_json::to_vec(&event.payload)
        .map_err(|err| anyhow::anyhow!("failed to serialize webhook payload: {err}"))?;
    let request_body_sha256 = hash_sha256_hex(&request_body);
    Ok(WebhookRequestEnvelope {
        request_body,
        request_body_sha256,
    })
}

async fn send_webhook_request(
    config: &AppConfig,
    request: &WebhookRequestEnvelope,
    event: &WebhookOutboxEvent,
) -> anyhow::Result<DeliveryResponse> {
    let signature = webhook_signature(&config.webhook_secret, &request.request_body)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client
        .post(&config.webhook_markov_url)
        .header("Content-Type", "application/json")
        .header("X-GR-Signature", format!("sha256={signature}"))
        .header("X-Request-ID", event.request_id.clone())
        .body(request.request_body.clone())
        .send()
        .await
        .map_err(|err| anyhow::anyhow!("webhook request failed: {err}"))?;
    let status_code = response.status().as_u16();
    let response_bytes = response
        .bytes()
        .await
        .map_err(|err| anyhow::anyhow!("failed reading webhook response body: {err}"))?
        .to_vec();
    let response_body_sha256 = if response_bytes.is_empty() {
        None
    } else {
        Some(hash_sha256_hex(&response_bytes))
    };
    Ok(DeliveryResponse {
        status_code: Some(status_code),
        response_body_sha256,
        error_message: format!("status code {status_code}"),
    })
}

fn classify_webhook_response(status_code: Option<u16>) -> WebhookDeliveryResultClass {
    match status_code {
        Some(200 | 202) => WebhookDeliveryResultClass::Success,
        Some(429) => WebhookDeliveryResultClass::Retryable,
        Some(code) if (500..=599).contains(&code) => WebhookDeliveryResultClass::Retryable,
        Some(_) => WebhookDeliveryResultClass::Terminal,
        None => WebhookDeliveryResultClass::Retryable,
    }
}

fn webhook_outbox_update(
    event: &WebhookOutboxEvent,
    delivery_class: &WebhookDeliveryResultClass,
    status_code: Option<u16>,
    error_message: &str,
    job: &JobEnvelope,
) -> WebhookOutboxUpdate {
    let next_attempt_at_ms = if matches!(delivery_class, WebhookDeliveryResultClass::Retryable)
        && job.attempt < event.max_attempts
    {
        let delay_ms = backoff_ms(1_000, job.attempt, 60_000);
        Some(now_ms() + delay_ms as i64)
    } else {
        None
    };

    let status = match delivery_class {
        WebhookDeliveryResultClass::Success => WebhookOutboxStatus::Delivered,
        WebhookDeliveryResultClass::Retryable if job.attempt < event.max_attempts => {
            WebhookOutboxStatus::Retrying
        }
        WebhookDeliveryResultClass::Retryable | WebhookDeliveryResultClass::Terminal => {
            WebhookOutboxStatus::DeadLetter
        }
    };

    let mut last_error = Some(error_message.to_string());
    if matches!(delivery_class, WebhookDeliveryResultClass::Success) {
        last_error = None;
    }

    WebhookOutboxUpdate {
        status,
        attempts: job.attempt,
        max_attempts: event.max_attempts,
        next_attempt_at_ms,
        last_status_code: status_code,
        last_error,
        request_id: None,
        correlation_id: None,
    }
}

fn webhook_delivery_log(
    event: &WebhookOutboxEvent,
    attempt: u32,
    delivery_class: &WebhookDeliveryResultClass,
    status_code: Option<u16>,
    request_body_sha256: String,
    response_body_sha256: Option<String>,
    error_message: String,
) -> WebhookDeliveryLog {
    let outcome = match delivery_class {
        WebhookDeliveryResultClass::Success => WebhookDeliveryResult::Success,
        WebhookDeliveryResultClass::Retryable => WebhookDeliveryResult::RetryableFailure,
        WebhookDeliveryResultClass::Terminal => WebhookDeliveryResult::TerminalFailure,
    };

    WebhookDeliveryLog {
        log_id: Uuid::now_v7().to_string(),
        event_id: event.event_id.clone(),
        attempt,
        outcome,
        status_code,
        request_id: event.request_id.clone(),
        correlation_id: event.correlation_id.clone(),
        request_body_sha256,
        response_body_sha256,
        error_message: Some(error_message),
        created_at_ms: now_ms(),
    }
}

fn webhook_signature(secret: &str, payload: &[u8]) -> anyhow::Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| anyhow::anyhow!("invalid webhook secret: {err}"))?;
    mac.update(payload);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn hash_sha256_hex(payload: &[u8]) -> String {
    use sha2::Digest;
    hex::encode(Sha256::digest(payload))
}

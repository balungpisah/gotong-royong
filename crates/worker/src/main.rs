use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

use gotong_domain::ports::jobs::{JobQueue, JobQueueError, JobType};
use gotong_domain::ports::ontology::OntologyRepository;
use gotong_domain::ports::webhook::WebhookOutboxRepository;
use gotong_domain::{
    auth::Role,
    identity::ActorIdentity,
    jobs::{
        backoff_ms, new_job, now_ms, ConceptVerificationPayload, JobDefaults, TTLCleanupPayload,
        WebhookRetryPayload,
    },
    moderation::{ModerationAutoReleaseCommand, ModerationService},
    ontology::OntologyConcept,
    ports::{jobs::JobEnvelope, moderation::ModerationRepository},
    webhook::{
        WebhookDeliveryLog, WebhookDeliveryResult, WebhookOutboxEvent, WebhookOutboxListQuery,
        WebhookOutboxStatus, WebhookOutboxUpdate,
    },
};
use gotong_infra::{
    config::AppConfig,
    db::DbConfig,
    jobs::{JobQueueMetricsSnapshot, RedisJobQueue},
    logging::init_tracing,
    repositories::{
        SurrealModerationRepository, SurrealOntologyRepository, SurrealWebhookOutboxRepository,
    },
};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;
mod observability;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config)?;
    observability::init_metrics()?;

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Some(command) = args.first().map(String::as_str) {
        match command {
            "webhook-backfill" => {
                run_webhook_backfill_mode(&config, &args[1..]).await?;
                return Ok(());
            }
            "webhook-replay-dlq" => {
                run_webhook_replay_mode(&config, &args[1..]).await?;
                return Ok(());
            }
            _ => {}
        }
    }

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

    let mut moderation_repo = None;
    let mut ontology_repo = None;
    let mut webhook_outbox_repo = None;
    let backend = config.data_backend.trim().to_ascii_lowercase();
    if matches!(backend.as_str(), "surreal" | "surrealdb" | "tikv") {
        let db_config = DbConfig::from_app_config(&config);
        let moderation_repository = SurrealModerationRepository::new(&db_config).await?;
        moderation_repo = Some(Arc::new(moderation_repository) as Arc<dyn ModerationRepository>);
        let ontology_repository = SurrealOntologyRepository::new(&db_config).await?;
        ontology_repo = Some(Arc::new(ontology_repository) as Arc<dyn OntologyRepository>);
        let webhook_repository = SurrealWebhookOutboxRepository::new(&db_config).await?;
        webhook_outbox_repo =
            Some(Arc::new(webhook_repository) as Arc<dyn WebhookOutboxRepository>);
    }

    let worker = Worker::new(
        queue,
        config,
        moderation_repo,
        ontology_repo,
        webhook_outbox_repo,
    );
    info!("worker starting");
    worker.run().await?;

    Ok(())
}

struct Worker {
    queue: RedisJobQueue,
    config: AppConfig,
    moderation_repo: Option<Arc<dyn ModerationRepository>>,
    ontology_repo: Option<Arc<dyn OntologyRepository>>,
    webhook_outbox_repo: Option<Arc<dyn WebhookOutboxRepository>>,
}

#[derive(Debug, Clone)]
struct WebhookBackfillOptions {
    file: String,
    dry_run: bool,
    progress_every: usize,
    max_attempts: Option<u32>,
}

impl Default for WebhookBackfillOptions {
    fn default() -> Self {
        Self {
            file: String::new(),
            dry_run: false,
            progress_every: 100,
            max_attempts: None,
        }
    }
}

#[derive(Debug, Clone)]
struct WebhookReplayOptions {
    status: WebhookOutboxStatus,
    limit: usize,
    dry_run: bool,
    progress_every: usize,
}

impl Default for WebhookReplayOptions {
    fn default() -> Self {
        Self {
            status: WebhookOutboxStatus::DeadLetter,
            limit: 500,
            dry_run: false,
            progress_every: 100,
        }
    }
}

#[derive(Debug, Default)]
struct WebhookBackfillSummary {
    processed: usize,
    created: usize,
    enqueued: usize,
    duplicates: usize,
    failed: usize,
}

#[derive(Debug, Default)]
struct WebhookReplaySummary {
    processed: usize,
    replayed: usize,
    failed: usize,
}

fn parse_webhook_backfill_options(args: &[String]) -> anyhow::Result<WebhookBackfillOptions> {
    let mut opts = WebhookBackfillOptions::default();
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--file" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --file"))?;
                opts.file = value.to_string();
                idx += 2;
            }
            "--dry-run" => {
                opts.dry_run = true;
                idx += 1;
            }
            "--progress-every" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --progress-every"))?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|err| anyhow::anyhow!("invalid --progress-every value: {err}"))?;
                if parsed == 0 {
                    return Err(anyhow::anyhow!("--progress-every must be >= 1"));
                }
                opts.progress_every = parsed;
                idx += 2;
            }
            "--max-attempts" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --max-attempts"))?;
                let parsed = value
                    .parse::<u32>()
                    .map_err(|err| anyhow::anyhow!("invalid --max-attempts value: {err}"))?;
                if parsed == 0 {
                    return Err(anyhow::anyhow!("--max-attempts must be >= 1"));
                }
                opts.max_attempts = Some(parsed);
                idx += 2;
            }
            other => {
                return Err(anyhow::anyhow!(
                    "unknown argument for webhook-backfill: {other}"
                ));
            }
        }
    }

    if opts.file.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "missing required --file argument for webhook-backfill"
        ));
    }
    Ok(opts)
}

fn parse_webhook_replay_options(args: &[String]) -> anyhow::Result<WebhookReplayOptions> {
    let mut opts = WebhookReplayOptions::default();
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--status" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --status"))?;
                opts.status = WebhookOutboxStatus::parse(value).ok_or_else(|| {
                    anyhow::anyhow!(
                        "invalid --status; use pending|in_flight|delivered|retrying|dead_letter"
                    )
                })?;
                idx += 2;
            }
            "--limit" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --limit"))?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|err| anyhow::anyhow!("invalid --limit value: {err}"))?;
                if parsed == 0 {
                    return Err(anyhow::anyhow!("--limit must be >= 1"));
                }
                opts.limit = parsed.min(10_000);
                idx += 2;
            }
            "--dry-run" => {
                opts.dry_run = true;
                idx += 1;
            }
            "--progress-every" => {
                let value = args
                    .get(idx + 1)
                    .ok_or_else(|| anyhow::anyhow!("missing value for --progress-every"))?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|err| anyhow::anyhow!("invalid --progress-every value: {err}"))?;
                if parsed == 0 {
                    return Err(anyhow::anyhow!("--progress-every must be >= 1"));
                }
                opts.progress_every = parsed;
                idx += 2;
            }
            other => {
                return Err(anyhow::anyhow!(
                    "unknown argument for webhook-replay-dlq: {other}"
                ));
            }
        }
    }
    Ok(opts)
}

async fn run_webhook_backfill_mode(config: &AppConfig, args: &[String]) -> anyhow::Result<()> {
    let options = parse_webhook_backfill_options(args)?;
    let db_config = DbConfig::from_app_config(config);
    let repo = SurrealWebhookOutboxRepository::new(&db_config).await?;

    let queue = if options.dry_run {
        None
    } else {
        Some(
            RedisJobQueue::connect_with_prefix(
                &config.redis_url,
                config.worker_queue_prefix.clone(),
            )
            .await?,
        )
    };

    let file = File::open(&options.file)
        .map_err(|err| anyhow::anyhow!("failed opening backfill file '{}': {err}", options.file))?;
    let reader = BufReader::new(file);
    let mut summary = WebhookBackfillSummary::default();
    let max_attempts = options
        .max_attempts
        .unwrap_or(config.webhook_max_attempts.max(1));

    println!(
        "[webhook-backfill] start file={} dry_run={} progress_every={}",
        options.file, options.dry_run, options.progress_every
    );

    for line in reader.lines() {
        let line =
            line.map_err(|err| anyhow::anyhow!("failed reading backfill file line: {err}"))?;
        let payload_line = line.trim();
        if payload_line.is_empty() {
            continue;
        }
        summary.processed = summary.processed.saturating_add(1);

        let payload: serde_json::Value = match serde_json::from_str(payload_line) {
            Ok(value) => value,
            Err(err) => {
                summary.failed = summary.failed.saturating_add(1);
                warn!(error = %err, "invalid json payload in backfill input");
                continue;
            }
        };

        let event_id = payload
            .get("event_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        let request_id = payload
            .get("request_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        if event_id.is_empty() || request_id.is_empty() {
            summary.failed = summary.failed.saturating_add(1);
            warn!("backfill payload is missing event_id or request_id");
            continue;
        }

        let correlation_id = payload
            .get("correlation_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("backfill:{event_id}"));

        if options.dry_run {
            match repo.get(&event_id).await {
                Ok(Some(_)) => {
                    summary.duplicates = summary.duplicates.saturating_add(1);
                }
                Ok(None) => {
                    summary.created = summary.created.saturating_add(1);
                }
                Err(err) => {
                    summary.failed = summary.failed.saturating_add(1);
                    warn!(error = %err, event_id = %event_id, "failed checking backfill duplicate");
                }
            }
        } else {
            let outbox_event = match WebhookOutboxEvent::new(
                payload,
                request_id.clone(),
                correlation_id.clone(),
                max_attempts,
            ) {
                Ok(event) => event,
                Err(err) => {
                    summary.failed = summary.failed.saturating_add(1);
                    warn!(error = %err, event_id = %event_id, "invalid backfill payload");
                    continue;
                }
            };

            match repo.create(&outbox_event).await {
                Ok(created) => {
                    summary.created = summary.created.saturating_add(1);
                    if let Some(queue) = queue.as_ref() {
                        let payload = serde_json::to_value(WebhookRetryPayload {
                            event_id: created.event_id.clone(),
                        })
                        .map_err(|err| {
                            anyhow::anyhow!(
                                "failed serializing webhook retry payload for {}: {err}",
                                created.event_id
                            )
                        })?;
                        let replay_request_id = format!("backfill:req:{}", created.event_id);
                        let replay_correlation_id = format!("backfill:corr:{}", created.event_id);
                        let job = new_job(
                            format!("backfill:{}:{}", created.event_id, now_ms()),
                            JobType::WebhookRetry,
                            payload,
                            replay_request_id,
                            replay_correlation_id,
                            JobDefaults {
                                max_attempts: config.webhook_max_attempts.max(1),
                            },
                        );
                        if let Err(err) = queue.enqueue(&job).await {
                            summary.failed = summary.failed.saturating_add(1);
                            warn!(
                                error = %err,
                                event_id = %created.event_id,
                                "failed to enqueue webhook retry during backfill"
                            );
                            continue;
                        }
                        summary.enqueued = summary.enqueued.saturating_add(1);
                    }
                }
                Err(gotong_domain::error::DomainError::Conflict) => {
                    summary.duplicates = summary.duplicates.saturating_add(1);
                }
                Err(err) => {
                    summary.failed = summary.failed.saturating_add(1);
                    warn!(error = %err, event_id = %event_id, "failed creating outbox event");
                }
            }
        }

        if summary.processed % options.progress_every == 0 {
            println!(
                "[webhook-backfill] progress processed={} created={} enqueued={} duplicates={} failed={}",
                summary.processed,
                summary.created,
                summary.enqueued,
                summary.duplicates,
                summary.failed
            );
        }
    }

    println!(
        "[webhook-backfill] done processed={} created={} enqueued={} duplicates={} failed={} dry_run={}",
        summary.processed,
        summary.created,
        summary.enqueued,
        summary.duplicates,
        summary.failed,
        options.dry_run
    );
    Ok(())
}

async fn run_webhook_replay_mode(config: &AppConfig, args: &[String]) -> anyhow::Result<()> {
    let options = parse_webhook_replay_options(args)?;
    let db_config = DbConfig::from_app_config(config);
    let repo = SurrealWebhookOutboxRepository::new(&db_config).await?;

    let queue = if options.dry_run {
        None
    } else {
        Some(
            RedisJobQueue::connect_with_prefix(
                &config.redis_url,
                config.worker_queue_prefix.clone(),
            )
            .await?,
        )
    };

    let events = repo
        .list(&WebhookOutboxListQuery {
            status: Some(options.status.clone()),
            limit: options.limit,
        })
        .await
        .map_err(|err| anyhow::anyhow!("failed listing outbox replay candidates: {err}"))?;

    println!(
        "[webhook-replay-dlq] start status={} limit={} matched={} dry_run={}",
        options.status.as_str(),
        options.limit,
        events.len(),
        options.dry_run
    );

    let mut summary = WebhookReplaySummary::default();
    for event in events {
        summary.processed = summary.processed.saturating_add(1);

        if options.dry_run {
            summary.replayed = summary.replayed.saturating_add(1);
        } else {
            let request_id = format!("replay:req:{}:{}", event.event_id, now_ms());
            let correlation_id = format!("replay:corr:{}", event.event_id);
            let update = WebhookOutboxUpdate {
                status: WebhookOutboxStatus::Pending,
                attempts: 0,
                max_attempts: config.webhook_max_attempts.max(1),
                next_attempt_at_ms: None,
                last_status_code: None,
                last_error: None,
                request_id: Some(request_id.clone()),
                correlation_id: Some(correlation_id.clone()),
            };
            if let Err(err) = repo.update(&event.event_id, &update).await {
                summary.failed = summary.failed.saturating_add(1);
                warn!(
                    error = %err,
                    event_id = %event.event_id,
                    "failed updating outbox event for replay"
                );
                continue;
            }

            if let Some(queue) = queue.as_ref() {
                let payload = serde_json::to_value(WebhookRetryPayload {
                    event_id: event.event_id.clone(),
                })
                .map_err(|err| {
                    anyhow::anyhow!(
                        "failed serializing replay payload for {}: {err}",
                        event.event_id
                    )
                })?;
                let job = new_job(
                    format!("replay:{}:{}", event.event_id, now_ms()),
                    JobType::WebhookRetry,
                    payload,
                    request_id,
                    correlation_id,
                    JobDefaults {
                        max_attempts: config.webhook_max_attempts.max(1),
                    },
                );
                if let Err(err) = queue.enqueue(&job).await {
                    summary.failed = summary.failed.saturating_add(1);
                    warn!(
                        error = %err,
                        event_id = %event.event_id,
                        "failed enqueuing replay job"
                    );
                    continue;
                }
            }

            summary.replayed = summary.replayed.saturating_add(1);
        }

        if summary.processed % options.progress_every == 0 {
            println!(
                "[webhook-replay-dlq] progress processed={} replayed={} failed={}",
                summary.processed, summary.replayed, summary.failed
            );
        }
    }

    println!(
        "[webhook-replay-dlq] done processed={} replayed={} failed={} dry_run={}",
        summary.processed, summary.replayed, summary.failed, options.dry_run
    );
    Ok(())
}

impl Worker {
    fn new(
        queue: RedisJobQueue,
        config: AppConfig,
        moderation_repo: Option<Arc<dyn ModerationRepository>>,
        ontology_repo: Option<Arc<dyn OntologyRepository>>,
        webhook_outbox_repo: Option<Arc<dyn WebhookOutboxRepository>>,
    ) -> Self {
        Self {
            queue,
            config,
            moderation_repo,
            ontology_repo,
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

        self.emit_queue_metrics().await;
        let mut next_ttl_cleanup_at_ms = 0_i64;
        let mut next_concept_verification_at_ms = 0_i64;
        let mut next_dead_letter_metric_at_ms = 0_i64;
        loop {
            self.emit_queue_metrics().await;
            let now = now_ms();
            if now >= next_dead_letter_metric_at_ms {
                self.emit_dead_letter_metrics().await;
                next_dead_letter_metric_at_ms = now + 10_000;
            }
            if let Err(err) = self
                .queue
                .promote_due(now, self.config.worker_promote_batch)
                .await
            {
                warn!(error = %err, "failed to promote due jobs, continuing");
            }

            self.enqueue_periodic_jobs(
                now,
                &mut next_ttl_cleanup_at_ms,
                &mut next_concept_verification_at_ms,
            )
            .await;

            match self.queue.dequeue(Duration::from_secs(2)).await {
                Ok(Some(job)) => {
                    let started_at = now_ms();
                    let job_type_label = job_type_label(&job.job_type);
                    if let Err(err) = handle_job(
                        &self.config,
                        &job,
                        self.moderation_repo.as_ref(),
                        self.ontology_repo.as_ref(),
                        self.webhook_outbox_repo.as_ref(),
                    )
                    .await
                    {
                        let duration_ms = now_ms() - started_at;
                        let job_id = job.job_id.clone();
                        if let Err(handle_err) = self.handle_failure(job, err).await {
                            warn!(
                                error = %handle_err,
                                job_id = %job_id,
                                "failed to handle job failure path"
                            );
                        }
                        observability::register_job_processed(
                            job_type_label,
                            "failed_processing",
                            duration_ms as f64,
                        );
                    } else if let Err(err) = self.queue.ack(&job.job_id).await {
                        warn!(
                            error = %err,
                            job_id = %job.job_id,
                            "failed to acknowledge successful job"
                        );
                        observability::register_job_processed(
                            job_type_label,
                            "failed_ack",
                            (now_ms() - started_at) as f64,
                        );
                    } else {
                        observability::register_job_processed(
                            job_type_label,
                            "success",
                            (now_ms() - started_at) as f64,
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

    async fn emit_queue_metrics(&self) {
        match self.queue.metrics_snapshot().await {
            Ok(JobQueueMetricsSnapshot {
                ready,
                delayed,
                processing,
                oldest_delayed_ms,
            }) => {
                observability::set_queue_depth_gauge(ready, delayed, processing);
                let lag_ms = oldest_delayed_ms.map_or(0, |score_ms| now_ms() - score_ms);
                observability::set_queue_lag_ms(lag_ms);
            }
            Err(err) => {
                warn!(error = %err, "failed to collect queue metrics snapshot");
            }
        }
    }

    async fn emit_dead_letter_metrics(&self) {
        let Some(repo) = self.webhook_outbox_repo.as_ref() else {
            observability::set_webhook_dead_letter_depth(0);
            return;
        };
        match repo.count_by_status(WebhookOutboxStatus::DeadLetter).await {
            Ok(total) => {
                observability::set_webhook_dead_letter_depth(total);
            }
            Err(err) => {
                warn!(
                    error = %err,
                    "failed to collect dead-letter outbox depth metric"
                );
            }
        }
    }

    async fn enqueue_periodic_jobs(
        &self,
        now: i64,
        next_ttl_cleanup_at_ms: &mut i64,
        next_concept_verification_at_ms: &mut i64,
    ) {
        let ttl_interval_ms = self.config.worker_ttl_cleanup_interval_ms.max(60_000);
        if now >= *next_ttl_cleanup_at_ms {
            let slot_start_ms = periodic_slot_start_ms(now, ttl_interval_ms);
            let job_id = format!("system:ttl_cleanup:{slot_start_ms}");
            let payload = TTLCleanupPayload {
                scheduled_ms: now,
                cutoff_ms: now,
            };
            self.enqueue_periodic_job(
                JobType::TTLCleanup,
                job_id,
                json!(payload),
                now,
                1,
                "ttl_cleanup",
                ttl_interval_ms,
            )
            .await;
            *next_ttl_cleanup_at_ms = slot_start_ms + ttl_interval_ms as i64;
        }

        let concept_interval_ms = self
            .config
            .worker_concept_verification_interval_ms
            .max(60_000);
        if now >= *next_concept_verification_at_ms {
            let slot_start_ms = periodic_slot_start_ms(now, concept_interval_ms);
            let qids = concept_verification_qids(&self.config.worker_concept_verification_qids);
            if qids.is_empty() {
                warn!(
                    concept_verification_qids = %self.config.worker_concept_verification_qids,
                    "skipping periodic concept verification: no qids configured"
                );
            } else {
                for qid in qids {
                    let job_id = format!("system:concept_verification:{slot_start_ms}:{qid}");
                    let payload = ConceptVerificationPayload {
                        qid,
                        scheduled_ms: now,
                    };
                    self.enqueue_periodic_job(
                        JobType::ConceptVerification,
                        job_id,
                        json!(payload),
                        now,
                        1,
                        "concept_verification",
                        concept_interval_ms,
                    )
                    .await;
                }
            }
            *next_concept_verification_at_ms = slot_start_ms + concept_interval_ms as i64;
        }
    }

    async fn enqueue_periodic_job(
        &self,
        job_type: JobType,
        job_id: String,
        payload: serde_json::Value,
        now: i64,
        max_attempts: u32,
        job_type_label: &str,
        dedupe_window_ms: u64,
    ) {
        let envelope = JobEnvelope {
            job_id: job_id.clone(),
            job_type,
            payload,
            request_id: job_id.clone(),
            correlation_id: format!("corr:{job_id}"),
            attempt: 1,
            max_attempts,
            run_at_ms: now,
            created_at_ms: now,
        };

        match self
            .queue
            .enqueue_if_absent(&envelope, dedupe_window_ms)
            .await
        {
            Ok(true) => {
                info!(job_id = %job_id, job_type = %job_type_label, "enqueued periodic job");
            }
            Ok(false) => {
                debug!(
                    job_id = %job_id,
                    job_type = %job_type_label,
                    "skipping duplicate periodic job enqueue"
                );
            }
            Err(err) => {
                warn!(
                    error = %err,
                    job_id = %job_id,
                    job_type = %job_type_label,
                    "failed to enqueue periodic job"
                );
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
    moderation_repo: Option<&Arc<dyn ModerationRepository>>,
    ontology_repo: Option<&Arc<dyn OntologyRepository>>,
    webhook_outbox_repo: Option<&Arc<dyn WebhookOutboxRepository>>,
) -> anyhow::Result<()> {
    match job.job_type {
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
        JobType::TTLCleanup => {
            handle_ttl_cleanup(ontology_repo, job).await?;
        }
        JobType::ConceptVerification => {
            handle_concept_verification(ontology_repo, job).await?;
        }
    }

    Ok(())
}

fn job_type_label(job_type: &JobType) -> &'static str {
    match job_type {
        JobType::ModerationAutoRelease => "moderation_auto_release",
        JobType::WebhookRetry => "webhook_retry",
        JobType::DigestSend => "digest_send",
        JobType::TTLCleanup => "ttl_cleanup",
        JobType::ConceptVerification => "concept_verification",
    }
}

fn periodic_slot_start_ms(now: i64, interval_ms: u64) -> i64 {
    if interval_ms == 0 {
        return now;
    }
    let interval = interval_ms as i64;
    now - now.rem_euclid(interval)
}

fn concept_verification_qids(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
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

fn parse_ttl_cleanup_payload(job: &JobEnvelope) -> anyhow::Result<TTLCleanupPayload> {
    let payload: TTLCleanupPayload = serde_json::from_value(job.payload.clone())
        .map_err(|err| anyhow::anyhow!("invalid ttl cleanup payload: {err}"))?;
    if payload.scheduled_ms < 0 {
        return Err(anyhow::anyhow!(
            "invalid ttl cleanup payload: scheduled_ms must be non-negative"
        ));
    }
    if payload.cutoff_ms < 0 {
        return Err(anyhow::anyhow!(
            "invalid ttl cleanup payload: cutoff_ms must be non-negative"
        ));
    }
    Ok(payload)
}

fn parse_concept_verification_payload(
    job: &JobEnvelope,
) -> anyhow::Result<ConceptVerificationPayload> {
    let payload: ConceptVerificationPayload = serde_json::from_value(job.payload.clone())
        .map_err(|err| anyhow::anyhow!("invalid concept verification payload: {err}"))?;
    if payload.scheduled_ms < 0 {
        return Err(anyhow::anyhow!(
            "invalid concept verification payload: scheduled_ms must be non-negative"
        ));
    }
    if payload.qid.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "invalid concept verification payload: qid is required"
        ));
    }
    Ok(payload)
}

async fn handle_ttl_cleanup(
    ontology_repo: Option<&Arc<dyn OntologyRepository>>,
    job: &JobEnvelope,
) -> anyhow::Result<()> {
    let payload = parse_ttl_cleanup_payload(job)?;
    let Some(repo) = ontology_repo else {
        warn!(
            job_id = %job.job_id,
            "skipping ttl cleanup job: ontology repository is unavailable"
        );
        return Ok(());
    };

    let removed = repo
        .cleanup_expired_notes(payload.cutoff_ms)
        .await
        .map_err(|err| anyhow::anyhow!("ttl cleanup ontology cleanup failed: {err}"))?;
    info!(
        job_id = %job.job_id,
        cutoff_ms = payload.cutoff_ms,
        removed,
        "handled ttl cleanup job"
    );
    Ok(())
}

async fn handle_concept_verification(
    ontology_repo: Option<&Arc<dyn OntologyRepository>>,
    job: &JobEnvelope,
) -> anyhow::Result<()> {
    let payload = parse_concept_verification_payload(job)?;
    let Some(repo) = ontology_repo else {
        warn!(
            job_id = %job.job_id,
            "skipping concept verification job: ontology repository is unavailable"
        );
        return Ok(());
    };

    let current = repo
        .get_concept_by_qid(&payload.qid)
        .await
        .map_err(|err| anyhow::anyhow!("failed to fetch concept by qid: {err}"))?;
    let concept = if let Some(mut concept) = current {
        concept.verified = true;
        let concept_id_missing = match concept.concept_id.split_once(':') {
            Some((_, id_part)) => id_part.trim().is_empty(),
            None => concept.concept_id.trim().is_empty(),
        };
        if concept_id_missing {
            concept.concept_id = concept.qid.clone();
        }
        concept
    } else {
        OntologyConcept {
            concept_id: payload.qid.clone(),
            qid: payload.qid.clone(),
            label_id: None,
            label_en: None,
            verified: true,
        }
    };
    repo.upsert_concept(&concept)
        .await
        .map_err(|err| anyhow::anyhow!("failed to upsert verified concept: {err}"))?;
    info!(
        job_id = %job.job_id,
        qid = %payload.qid,
        "handled concept verification job"
    );
    Ok(())
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

impl WebhookDeliveryResultClass {
    fn as_label(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Retryable => "retryable_failure",
            Self::Terminal => "terminal_failure",
        }
    }
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
    let delivery_started_at = now_ms();
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
    observability::register_webhook_delivery(
        delivery_class.as_label(),
        status_code,
        (now_ms() - delivery_started_at).max(0) as f64,
    );

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

#[cfg(test)]
mod periodic_job_tests {
    use super::*;

    use gotong_domain::ontology::{
        OntologyConcept, OntologyEdgeKind, OntologyNoteCreate, OntologyTripleCreate,
    };
    use gotong_infra::repositories::InMemoryOntologyRepository;

    fn moderation_auto_release_job(payload: serde_json::Value) -> JobEnvelope {
        JobEnvelope {
            job_id: "job-1".to_string(),
            job_type: JobType::ModerationAutoRelease,
            payload,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            attempt: 1,
            max_attempts: 1,
            run_at_ms: 1,
            created_at_ms: 1,
        }
    }

    fn ttl_cleanup_job(payload: serde_json::Value) -> JobEnvelope {
        JobEnvelope {
            job_id: "job-1".to_string(),
            job_type: JobType::TTLCleanup,
            payload,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            attempt: 1,
            max_attempts: 1,
            run_at_ms: 1,
            created_at_ms: 1,
        }
    }

    fn concept_verification_job(payload: serde_json::Value) -> JobEnvelope {
        JobEnvelope {
            job_id: "job-1".to_string(),
            job_type: JobType::ConceptVerification,
            payload,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            attempt: 1,
            max_attempts: 1,
            run_at_ms: 1,
            created_at_ms: 1,
        }
    }

    #[test]
    fn periodic_slot_start_ms_rounds_down_to_interval_boundary() {
        assert_eq!(periodic_slot_start_ms(12_345, 1_000), 12_000);
    }

    #[test]
    fn periodic_slot_start_ms_zero_interval_passthrough() {
        assert_eq!(periodic_slot_start_ms(12_345, 0), 12_345);
    }

    #[test]
    fn concept_verification_qids_trims_and_filters() {
        let qids = concept_verification_qids("Q2095,  Q93189 , ,Q5,");
        assert_eq!(qids, vec!["Q2095", "Q93189", "Q5"]);
    }

    #[test]
    fn parse_ttl_cleanup_payload_rejects_negative_cutoff() {
        let payload = serde_json::json!({
            "scheduled_ms": 1_000,
            "cutoff_ms": -1
        });
        let job = ttl_cleanup_job(payload);
        assert!(parse_ttl_cleanup_payload(&job).is_err());
    }

    #[test]
    fn parse_ttl_cleanup_payload_accepts_valid_payload() {
        let payload = serde_json::json!({
            "scheduled_ms": 1_000,
            "cutoff_ms": 2_000
        });
        let job = ttl_cleanup_job(payload);
        let parsed = parse_ttl_cleanup_payload(&job).expect("valid ttl cleanup payload");
        assert_eq!(parsed.scheduled_ms, 1_000);
        assert_eq!(parsed.cutoff_ms, 2_000);
    }

    #[test]
    fn parse_ttl_cleanup_payload_rejects_missing_required_fields() {
        let job = ttl_cleanup_job(serde_json::json!({
            "scheduled_ms": 1_000
        }));
        assert!(parse_ttl_cleanup_payload(&job).is_err());
    }

    #[test]
    fn parse_ttl_cleanup_payload_rejects_missing_payload() {
        let job = ttl_cleanup_job(serde_json::json!({}));
        assert!(parse_ttl_cleanup_payload(&job).is_err());
    }

    #[test]
    fn parse_ttl_cleanup_payload_rejects_negative_scheduled_ms() {
        let payload = serde_json::json!({
            "scheduled_ms": -1,
            "cutoff_ms": 1_000,
        });
        let job = ttl_cleanup_job(payload);
        assert!(parse_ttl_cleanup_payload(&job).is_err());
    }

    #[test]
    fn parse_concept_verification_payload_requires_qid() {
        let payload = serde_json::json!({
            "qid": " ",
            "scheduled_ms": 1_000
        });
        let mut job = ttl_cleanup_job(payload);
        job.job_type = JobType::ConceptVerification;
        assert!(parse_concept_verification_payload(&job).is_err());
    }

    #[test]
    fn parse_concept_verification_payload_rejects_missing_payload() {
        let mut job = ttl_cleanup_job(serde_json::json!({}));
        job.job_type = JobType::ConceptVerification;
        assert!(parse_concept_verification_payload(&job).is_err());
    }

    #[test]
    fn parse_concept_verification_payload_rejects_negative_scheduled_ms() {
        let payload = serde_json::json!({
            "qid": "Q123",
            "scheduled_ms": -1,
        });
        let job = concept_verification_job(payload);
        assert!(parse_concept_verification_payload(&job).is_err());
    }

    #[test]
    fn parse_concept_verification_payload_accepts_valid_payload() {
        let payload = serde_json::json!({
            "qid": "Q2095",
            "scheduled_ms": 1_000
        });
        let job = concept_verification_job(payload);
        let parsed =
            parse_concept_verification_payload(&job).expect("valid concept verification payload");
        assert_eq!(parsed.qid, "Q2095");
        assert_eq!(parsed.scheduled_ms, 1_000);
    }

    #[test]
    fn parse_webhook_retry_payload_rejects_missing_event_id() {
        let payload = serde_json::json!({});
        let job = JobEnvelope {
            job_id: "job-1".to_string(),
            job_type: JobType::WebhookRetry,
            payload,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            attempt: 1,
            max_attempts: 1,
            run_at_ms: 1,
            created_at_ms: 1,
        };
        assert!(parse_webhook_retry_payload(&job).is_err());
    }

    #[tokio::test]
    async fn handle_ttl_cleanup_removes_expired_notes_and_edges() {
        let repo: Arc<dyn OntologyRepository> = Arc::new(InMemoryOntologyRepository::new());
        let now = gotong_domain::jobs::now_ms();

        repo.upsert_concept(&OntologyConcept {
            concept_id: "Q93189".to_string(),
            qid: "Q93189".to_string(),
            label_id: Some("Telur".to_string()),
            label_en: Some("Egg".to_string()),
            verified: true,
        })
        .await
        .expect("upsert concept");

        let expired = repo
            .create_note(&OntologyNoteCreate {
                note_id: Some("note-expired".to_string()),
                content: "expired note".to_string(),
                author_id: "author-1".to_string(),
                community_id: "rt-1".to_string(),
                temporal_class: "ephemeral".to_string(),
                ttl_expires_ms: Some(now - 1),
                ai_readable: true,
                rahasia_level: 0,
                confidence: 0.9,
            })
            .await
            .expect("create expired note");

        let active = repo
            .create_note(&OntologyNoteCreate {
                note_id: Some("note-active".to_string()),
                content: "active note".to_string(),
                author_id: "author-2".to_string(),
                community_id: "rt-1".to_string(),
                temporal_class: "ephemeral".to_string(),
                ttl_expires_ms: Some(now + 60_000),
                ai_readable: true,
                rahasia_level: 0,
                confidence: 0.9,
            })
            .await
            .expect("create active note");

        repo.write_triples(&[
            OntologyTripleCreate {
                edge: OntologyEdgeKind::About,
                from_id: format!("note:{}", expired.note_id),
                to_id: "concept:Q93189".to_string(),
                predicate: Some("schema:price".to_string()),
                metadata: None,
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Vouches,
                from_id: "warga:user-1".to_string(),
                to_id: format!("note:{}", expired.note_id),
                predicate: None,
                metadata: Some(serde_json::json!({"reason": "expired"})),
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Challenges,
                from_id: "warga:user-2".to_string(),
                to_id: format!("note:{}", active.note_id),
                predicate: None,
                metadata: Some(serde_json::json!({"reason": "active"})),
            },
        ])
        .await
        .expect("write triples");

        let job = ttl_cleanup_job(serde_json::json!({
            "scheduled_ms": now,
            "cutoff_ms": now,
        }));

        handle_ttl_cleanup(Some(&repo), &job)
            .await
            .expect("handle ttl cleanup");

        let expired_feedback = repo
            .note_feedback_counts(&expired.note_id)
            .await
            .expect("expired feedback counts");
        assert_eq!(expired_feedback.vouch_count, 0);
        assert_eq!(expired_feedback.challenge_count, 0);

        let active_feedback = repo
            .note_feedback_counts(&active.note_id)
            .await
            .expect("active feedback counts");
        assert_eq!(active_feedback.challenge_count, 1);
    }

    #[tokio::test]
    async fn handle_ttl_cleanup_without_repo_is_noop() {
        let job = ttl_cleanup_job(serde_json::json!({
            "scheduled_ms": 1_000,
            "cutoff_ms": 1_000,
        }));
        assert!(handle_ttl_cleanup(None, &job).await.is_ok());
    }

    #[tokio::test]
    async fn handle_concept_verification_updates_and_creates_concepts() {
        let repo: Arc<dyn OntologyRepository> = Arc::new(InMemoryOntologyRepository::new());
        let existing = repo
            .upsert_concept(&OntologyConcept {
                concept_id: "Q111".to_string(),
                qid: "Q111".to_string(),
                label_id: Some("Existing".to_string()),
                label_en: Some("Existing".to_string()),
                verified: false,
            })
            .await
            .expect("upsert concept");

        let update_job = concept_verification_job(serde_json::json!({
            "qid": "Q111",
            "scheduled_ms": 1_000,
        }));
        handle_concept_verification(Some(&repo), &update_job)
            .await
            .expect("update existing concept");

        let updated = repo
            .get_concept_by_qid(&existing.qid)
            .await
            .expect("get updated concept")
            .expect("concept exists");
        assert!(updated.verified);

        let create_job = concept_verification_job(serde_json::json!({
            "qid": "Q222",
            "scheduled_ms": 1_000,
        }));
        handle_concept_verification(Some(&repo), &create_job)
            .await
            .expect("create missing concept");

        let created = repo
            .get_concept_by_qid("Q222")
            .await
            .expect("get created concept")
            .expect("concept exists");
        assert!(created.concept_id.ends_with("Q222"));
        assert!(created.verified);
    }

    #[tokio::test]
    async fn handle_concept_verification_fills_empty_concept_id() {
        let repo: Arc<dyn OntologyRepository> = Arc::new(InMemoryOntologyRepository::new());
        repo.upsert_concept(&OntologyConcept {
            concept_id: "".to_string(),
            qid: "Q_MISSING_ID".to_string(),
            label_id: None,
            label_en: None,
            verified: false,
        })
        .await
        .expect("upsert concept");

        let job = concept_verification_job(serde_json::json!({
            "qid": "Q_MISSING_ID",
            "scheduled_ms": 1_000,
        }));
        handle_concept_verification(Some(&repo), &job)
            .await
            .expect("verify concept");

        let updated = repo
            .get_concept_by_qid("Q_MISSING_ID")
            .await
            .expect("get updated concept")
            .expect("concept exists");
        assert!(updated.concept_id.ends_with("Q_MISSING_ID"));
        assert!(updated.verified);
    }

    #[tokio::test]
    async fn handle_concept_verification_without_repo_is_noop() {
        let job = concept_verification_job(serde_json::json!({
            "qid": "Q2095",
            "scheduled_ms": 1_000,
        }));
        assert!(handle_concept_verification(None, &job).await.is_ok());
    }

    #[test]
    fn parse_moderation_auto_release_payload_rejects_blank_content_id() {
        let payload = serde_json::json!({
            "content_id": " ",
            "scheduled_ms": 1_000,
            "hold_decision_request_id": "hold-1",
            "release_ms": 2_000
        });
        let job = moderation_auto_release_job(payload);
        assert!(parse_moderation_auto_release_payload(&job).is_err());
    }

    #[test]
    fn parse_moderation_auto_release_payload_rejects_missing_payload() {
        let job = moderation_auto_release_job(serde_json::json!({}));
        assert!(parse_moderation_auto_release_payload(&job).is_err());
    }

    #[test]
    fn periodic_slot_start_ms_with_negative_now_rounds_down() {
        assert_eq!(periodic_slot_start_ms(-1, 1_000), -1000);
    }
}

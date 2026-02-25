use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::db::DbConfig;
use gotong_domain::DomainResult;
use gotong_domain::adaptive_path::{
    AdaptivePathEvent, AdaptivePathPlan, AdaptivePathSuggestion, SuggestionDecisionStatus,
};
use gotong_domain::chat::{
    ChatDeliveryEvent, ChatMember, ChatMemberRole, ChatMessage, ChatReadCursor, ChatThread,
    ChatThreadQuery, ChatThreadWithMembers, MessageCatchup,
};
use gotong_domain::contributions::{Contribution, ContributionType};
use gotong_domain::discovery::FEED_SOURCE_VAULT;
use gotong_domain::discovery::{FeedItem, InAppNotification};
use gotong_domain::error::DomainError;
use gotong_domain::evidence::{Evidence, EvidenceType};
use gotong_domain::mode::Mode;
use gotong_domain::moderation::{
    ContentModeration, ModerationAction, ModerationActorSnapshot, ModerationDecision,
    ModerationStatus, ModerationViolation,
};
use gotong_domain::ontology::{
    NoteFeedbackCounts, OntologyActionRef, OntologyConcept, OntologyEdgeKind, OntologyNote,
    OntologyNoteCreate, OntologyPlaceRef, OntologyTripleCreate,
};
use gotong_domain::ports::adaptive_path::AdaptivePathRepository;
use gotong_domain::ports::chat::ChatRepository as ChatRepositoryPort;
use gotong_domain::ports::contributions::ContributionRepository;
use gotong_domain::ports::discovery::{
    FeedRepository, FeedRepositoryQuery, FeedRepositorySearchQuery, NotificationRepository,
    NotificationRepositoryListQuery,
};
use gotong_domain::ports::evidence::EvidenceRepository;
use gotong_domain::ports::group::{
    GroupJoinRequestRecord, GroupMemberRecord, GroupRecord, GroupRepository,
};
use gotong_domain::ports::moderation::ModerationRepository;
use gotong_domain::ports::ontology::OntologyRepository;
use gotong_domain::ports::siaga::SiagaRepository;
use gotong_domain::ports::vault::VaultRepository;
use gotong_domain::ports::vouches::VouchRepository;
use gotong_domain::ports::webhook::WebhookOutboxRepository;
use gotong_domain::siaga::{
    SiagaActorSnapshot, SiagaBroadcast, SiagaClosure, SiagaResponder, SiagaState,
    SiagaTimelineEvent, SiagaTimelineEventType,
};
use gotong_domain::vault::{
    VaultActorSnapshot, VaultEntry, VaultState, VaultTimelineEvent, VaultTimelineEventType,
};
use gotong_domain::vouches::{Vouch, VouchWeightHint};
use gotong_domain::webhook::{
    WebhookDeliveryLog, WebhookDeliveryResult, WebhookOutboxEvent, WebhookOutboxListQuery,
    WebhookOutboxStatus, WebhookOutboxUpdate,
};
use metrics::counter;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use tokio::sync::RwLock;

const FEED_INVOLVEMENT_LANE_REQUESTS_TOTAL: &str =
    "gotong_api_feed_involvement_lane_requests_total";
const FEED_INVOLVEMENT_SHADOW_MISMATCH_TOTAL: &str =
    "gotong_api_feed_involvement_shadow_mismatch_total";

#[derive(Default)]
pub struct InMemoryContributionRepository {
    store: Arc<RwLock<HashMap<String, Contribution>>>,
}

#[derive(Default)]
pub struct InMemoryGroupRepository {
    store: Arc<RwLock<HashMap<String, GroupRecord>>>,
}

#[derive(Default)]
pub struct InMemoryWebhookOutboxRepository {
    events: Arc<RwLock<HashMap<String, WebhookOutboxEvent>>>,
    by_request: Arc<RwLock<HashMap<String, String>>>,
    logs: Arc<RwLock<HashMap<String, Vec<WebhookDeliveryLog>>>>,
}

impl InMemoryWebhookOutboxRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl WebhookOutboxRepository for InMemoryWebhookOutboxRepository {
    fn create(
        &self,
        event: &WebhookOutboxEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookOutboxEvent>> {
        let event = event.clone();
        let events = self.events.clone();
        let by_request = self.by_request.clone();
        Box::pin(async move {
            let mut events = events.write().await;
            if events.contains_key(&event.event_id) {
                return Err(DomainError::Conflict);
            }
            let mut by_request = by_request.write().await;
            if by_request.contains_key(&event.request_id) {
                return Err(DomainError::Conflict);
            }
            by_request.insert(event.request_id.clone(), event.event_id.clone());
            events.insert(event.event_id.clone(), event.clone());
            Ok(event)
        })
    }

    fn get(
        &self,
        event_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>> {
        let event_id = event_id.to_string();
        let events = self.events.clone();
        Box::pin(async move {
            let events = events.read().await;
            Ok(events.get(&event_id).cloned())
        })
    }

    fn get_by_request_id(
        &self,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>> {
        let request_id = request_id.to_string();
        let by_request = self.by_request.clone();
        let events = self.events.clone();
        Box::pin(async move {
            let by_request = by_request.read().await;
            let Some(event_id) = by_request.get(&request_id) else {
                return Ok(None);
            };
            Ok(events.read().await.get(event_id).cloned())
        })
    }

    fn list(
        &self,
        query: &WebhookOutboxListQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<WebhookOutboxEvent>>> {
        let query = query.clone();
        let events = self.events.clone();
        Box::pin(async move {
            let mut events: Vec<_> = events.read().await.values().cloned().collect();
            if let Some(status) = query.status {
                events.retain(|event| event.status == status);
            }
            events.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.event_id.cmp(&left.event_id))
            });
            if query.limit > 0 {
                events.truncate(query.limit);
            }
            Ok(events)
        })
    }

    fn count_by_status(
        &self,
        status: WebhookOutboxStatus,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<u64>> {
        let events = self.events.clone();
        Box::pin(async move {
            let count = events
                .read()
                .await
                .values()
                .filter(|event| event.status == status)
                .count();
            Ok(count as u64)
        })
    }

    fn update(
        &self,
        event_id: &str,
        update: &WebhookOutboxUpdate,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookOutboxEvent>> {
        let event_id = event_id.to_string();
        let update = update.clone();
        let events = self.events.clone();
        Box::pin(async move {
            let mut events = events.write().await;
            let event = events.get_mut(&event_id).ok_or(DomainError::NotFound)?;
            event.status = update.status;
            event.attempts = update.attempts;
            event.max_attempts = update.max_attempts;
            event.next_attempt_at_ms = update.next_attempt_at_ms;
            event.last_status_code = update.last_status_code;
            event.last_error = update.last_error.clone();
            if let Some(request_id) = update.request_id {
                event.request_id = request_id;
            }
            if let Some(correlation_id) = update.correlation_id {
                event.correlation_id = correlation_id;
            }
            event.updated_at_ms = gotong_domain::jobs::now_ms();
            Ok(event.clone())
        })
    }

    fn append_log(
        &self,
        log: &WebhookDeliveryLog,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookDeliveryLog>> {
        let log = log.clone();
        let logs = self.logs.clone();
        Box::pin(async move {
            let mut logs = logs.write().await;
            logs.entry(log.event_id.clone())
                .or_default()
                .push(log.clone());
            Ok(log)
        })
    }

    fn list_logs(
        &self,
        event_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<WebhookDeliveryLog>>> {
        let event_id = event_id.to_string();
        let logs = self.logs.clone();
        Box::pin(async move {
            let mut logs = logs
                .read()
                .await
                .get(&event_id)
                .cloned()
                .unwrap_or_default();
            logs.sort_by(|left, right| left.attempt.cmp(&right.attempt));
            Ok(logs)
        })
    }
}

#[cfg(test)]
mod webhook_outbox_repository_tests {
    use super::*;
    use serde_json::json;

    fn valid_payload(event_id: &str, request_id: &str) -> serde_json::Value {
        json!({
            "event_id": event_id,
            "event_type": "contribution_created",
            "schema_version": "1",
            "request_id": request_id,
            "actor": {
                "user_id": "user-123",
                "username": "user-123-name"
            },
            "subject": {
                "contribution_type": "task_completion",
                "title": "test"
            }
        })
    }

    #[tokio::test]
    async fn in_memory_webhook_outbox_count_by_status_tracks_dead_letter_depth() {
        let repo = InMemoryWebhookOutboxRepository::new();

        let first = WebhookOutboxEvent::new(valid_payload("evt-1", "req-1"), "req-1", "corr-1", 3)
            .expect("create first event");
        let second = WebhookOutboxEvent::new(valid_payload("evt-2", "req-2"), "req-2", "corr-2", 3)
            .expect("create second event");

        repo.create(&first).await.expect("persist first");
        repo.create(&second).await.expect("persist second");

        repo.update(
            &first.event_id,
            &WebhookOutboxUpdate {
                status: WebhookOutboxStatus::DeadLetter,
                attempts: 1,
                max_attempts: first.max_attempts,
                next_attempt_at_ms: None,
                last_status_code: Some(500),
                last_error: Some("upstream failed".to_string()),
                request_id: None,
                correlation_id: None,
            },
        )
        .await
        .expect("mark first as dead_letter");

        let dead_letter = repo
            .count_by_status(WebhookOutboxStatus::DeadLetter)
            .await
            .expect("count dead_letter after first update");
        assert_eq!(dead_letter, 1);

        repo.update(
            &second.event_id,
            &WebhookOutboxUpdate {
                status: WebhookOutboxStatus::DeadLetter,
                attempts: 2,
                max_attempts: second.max_attempts,
                next_attempt_at_ms: None,
                last_status_code: Some(500),
                last_error: Some("retry exhausted".to_string()),
                request_id: None,
                correlation_id: None,
            },
        )
        .await
        .expect("mark second as dead_letter");

        let dead_letter = repo
            .count_by_status(WebhookOutboxStatus::DeadLetter)
            .await
            .expect("count dead_letter after second update");
        assert_eq!(dead_letter, 2);

        let pending = repo
            .count_by_status(WebhookOutboxStatus::Pending)
            .await
            .expect("count pending");
        assert_eq!(pending, 0);
    }
}

#[derive(Clone)]
pub struct SurrealWebhookOutboxRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealWebhookOutboxRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn parse_rfc3339(value: &str) -> DomainResult<i64> {
        let dt = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok((dt.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn to_rfc3339(epoch_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(epoch_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid ms timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_status(value: &str) -> DomainResult<WebhookOutboxStatus> {
        value
            .parse::<WebhookOutboxStatus>()
            .map_err(|_| DomainError::Validation(format!("invalid webhook status '{value}'")))
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn decode_event_rows(rows: Vec<Value>) -> DomainResult<Vec<WebhookOutboxEvent>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealWebhookOutboxRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid webhook outbox row: {err}"))
                    })
                    .and_then(|row| {
                        Ok(WebhookOutboxEvent {
                            event_id: row.event_id,
                            event_type: row.event_type,
                            payload: row.payload,
                            actor_id: row.actor_id,
                            actor_username: row.actor_username,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            status: Self::parse_status(&row.status)?,
                            attempts: row.attempts,
                            max_attempts: row.max_attempts,
                            next_attempt_at_ms: row
                                .next_attempt_at
                                .as_deref()
                                .map(Self::parse_rfc3339)
                                .transpose()?,
                            last_status_code: row.last_status_code,
                            last_error: row.last_error,
                            created_at_ms: Self::parse_rfc3339(&row.created_at)?,
                            updated_at_ms: Self::parse_rfc3339(&row.updated_at)?,
                        })
                    })
            })
            .collect()
    }

    fn decode_delivery_logs(rows: Vec<Value>) -> DomainResult<Vec<WebhookDeliveryLog>> {
        rows.into_iter()
            .map(|row| -> DomainResult<WebhookDeliveryLog> {
                let row =
                    serde_json::from_value::<SurrealWebhookDeliveryLogRow>(row).map_err(|err| {
                        DomainError::Validation(format!("invalid webhook delivery log row: {err}"))
                    })?;
                Ok(WebhookDeliveryLog {
                    log_id: row.log_id,
                    event_id: row.event_id,
                    attempt: row.attempt,
                    outcome: row.outcome.parse::<WebhookDeliveryResult>().map_err(|_| {
                        DomainError::Validation(format!("invalid webhook outcome: {}", row.outcome))
                    })?,
                    status_code: row.status_code,
                    request_id: row.request_id,
                    correlation_id: row.correlation_id,
                    request_body_sha256: row.request_body_sha256,
                    response_body_sha256: row.response_body_sha256,
                    error_message: row.error_message,
                    created_at_ms: Self::parse_rfc3339(&row.created_at)?,
                })
            })
            .collect()
    }

    fn build_event_payload(
        event: &WebhookOutboxEvent,
    ) -> DomainResult<SurrealWebhookOutboxCreateRow> {
        let created_at = Self::to_rfc3339(event.created_at_ms)?;
        let updated_at = Self::to_rfc3339(event.updated_at_ms)?;
        Ok(SurrealWebhookOutboxCreateRow {
            event_id: event.event_id.clone(),
            event_type: event.event_type.clone(),
            payload: event.payload.clone(),
            actor_id: event.actor_id.clone(),
            actor_username: event.actor_username.clone(),
            request_id: event.request_id.clone(),
            correlation_id: event.correlation_id.clone(),
            status: event.status.as_str().to_string(),
            attempts: event.attempts,
            max_attempts: event.max_attempts,
            next_attempt_at: event.next_attempt_at_ms.map(Self::to_rfc3339).transpose()?,
            last_status_code: event.last_status_code,
            last_error: event.last_error.clone(),
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealWebhookOutboxCreateRow {
    event_id: String,
    event_type: String,
    payload: serde_json::Value,
    actor_id: String,
    actor_username: String,
    request_id: String,
    correlation_id: String,
    status: String,
    attempts: u32,
    max_attempts: u32,
    next_attempt_at: Option<String>,
    last_status_code: Option<u16>,
    last_error: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealWebhookOutboxRow {
    event_id: String,
    event_type: String,
    payload: serde_json::Value,
    actor_id: String,
    actor_username: String,
    request_id: String,
    correlation_id: String,
    status: String,
    attempts: u32,
    max_attempts: u32,
    #[serde(rename = "next_attempt_at")]
    next_attempt_at: Option<String>,
    last_status_code: Option<u16>,
    last_error: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealWebhookDeliveryLogCreateRow {
    log_id: String,
    event_id: String,
    attempt: u32,
    outcome: String,
    status_code: Option<u16>,
    request_id: String,
    correlation_id: String,
    request_body_sha256: String,
    response_body_sha256: Option<String>,
    error_message: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealWebhookDeliveryLogRow {
    log_id: String,
    event_id: String,
    attempt: u32,
    outcome: String,
    status_code: Option<u16>,
    request_id: String,
    correlation_id: String,
    request_body_sha256: String,
    response_body_sha256: Option<String>,
    error_message: Option<String>,
    created_at: String,
}

impl WebhookOutboxRepository for SurrealWebhookOutboxRepository {
    fn create(
        &self,
        event: &WebhookOutboxEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookOutboxEvent>> {
        let row = match Self::build_event_payload(event) {
            Ok(row) => row,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let event_id = row.event_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let mut query = String::from(
                "CREATE webhook_outbox_event SET \
                    event_id = $event_id, \
                    event_type = $event_type, \
                    payload = $payload, \
                    actor_id = $actor_id, \
                    actor_username = $actor_username, \
                    request_id = $request_id, \
                    correlation_id = $correlation_id, \
                    status = $status, \
                    attempts = $attempts, \
                    max_attempts = $max_attempts, \
                    last_status_code = $last_status_code, \
                    last_error = $last_error, \
                    created_at = <datetime>$created_at, \
                    updated_at = <datetime>$updated_at",
            );
            if row.next_attempt_at.is_some() {
                query.push_str(", next_attempt_at = <datetime>$next_attempt_at");
            } else {
                query.push_str(", next_attempt_at = NONE");
            }
            query.push_str(";");
            let mut pending = client.query(&query);
            pending = pending.bind(("event_id", row.event_id));
            pending = pending.bind(("event_type", row.event_type));
            pending = pending.bind(("payload", row.payload));
            pending = pending.bind(("actor_id", row.actor_id));
            pending = pending.bind(("actor_username", row.actor_username));
            pending = pending.bind(("request_id", row.request_id));
            pending = pending.bind(("correlation_id", row.correlation_id));
            pending = pending.bind(("status", row.status));
            pending = pending.bind(("attempts", row.attempts as i64));
            pending = pending.bind(("max_attempts", row.max_attempts as i64));
            pending = pending.bind(("last_status_code", row.last_status_code.map(i64::from)));
            pending = pending.bind(("last_error", row.last_error));
            pending = pending.bind(("created_at", row.created_at));
            pending = pending.bind(("updated_at", row.updated_at));
            if let Some(next_attempt_at) = row.next_attempt_at {
                pending = pending.bind(("next_attempt_at", next_attempt_at));
            }
            pending.await.map_err(Self::map_surreal_error)?;
            let mut response = client
                .query(
                    "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                            status, attempts, max_attempts, \
                            IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                            last_status_code, last_error, <string>created_at AS created_at, \
                            <string>updated_at AS updated_at \
                     FROM webhook_outbox_event WHERE event_id = $event_id LIMIT 1",
                )
                .bind(("event_id", event_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::decode_event_rows(rows)?;
            events
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn get(
        &self,
        event_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>> {
        let event_id = event_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                            status, attempts, max_attempts, \
                            IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                            last_status_code, last_error, <string>created_at AS created_at, \
                            <string>updated_at AS updated_at \
                     FROM webhook_outbox_event WHERE event_id = $event_id LIMIT 1",
                )
                .bind(("event_id", event_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::decode_event_rows(rows)?;
            Ok(events.pop())
        })
    }

    fn get_by_request_id(
        &self,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<WebhookOutboxEvent>>> {
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                            status, attempts, max_attempts, \
                            IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                            last_status_code, last_error, <string>created_at AS created_at, \
                            <string>updated_at AS updated_at \
                     FROM webhook_outbox_event WHERE request_id = $request_id LIMIT 1",
                )
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::decode_event_rows(rows)?;
            Ok(events.pop())
        })
    }

    fn list(
        &self,
        query: &WebhookOutboxListQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<WebhookOutboxEvent>>> {
        let query = query.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = match query.status {
                Some(status) => client
                    .query(
                        "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                                status, attempts, max_attempts, \
                                IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                                last_status_code, last_error, <string>created_at AS created_at, \
                                <string>updated_at AS updated_at \
                         FROM webhook_outbox_event WHERE status = $status \
                         ORDER BY created_at DESC, event_id DESC LIMIT $limit",
                    )
                    .bind(("status", status.as_str()))
                    .bind(("limit", query.limit as i64))
                    .await,
                None => client
                    .query(
                        "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                                status, attempts, max_attempts, \
                                IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                                last_status_code, last_error, <string>created_at AS created_at, \
                                <string>updated_at AS updated_at \
                         FROM webhook_outbox_event ORDER BY created_at DESC, event_id DESC LIMIT $limit",
                    )
                    .bind(("limit", query.limit as i64))
                    .await,
            }
            .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_event_rows(rows)
        })
    }

    fn count_by_status(
        &self,
        status: WebhookOutboxStatus,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<u64>> {
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT count() AS total FROM webhook_outbox_event WHERE status = $status GROUP ALL",
                )
                .bind(("status", status.as_str()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let total = rows
                .first()
                .and_then(|row| row.get("total").or_else(|| row.get("count")))
                .and_then(|value| {
                    value
                        .as_u64()
                        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
                })
                .unwrap_or(0);
            Ok(total)
        })
    }

    fn update(
        &self,
        event_id: &str,
        update: &WebhookOutboxUpdate,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookOutboxEvent>> {
        let event_id = event_id.to_string();
        let client = self.client.clone();
        let event_id_for_query = event_id.clone();
        let update = update.clone();
        Box::pin(async move {
            let mut query = String::from("UPDATE webhook_outbox_event");
            query.push_str(
                " SET status = $status, attempts = $attempts, max_attempts = $max_attempts, ",
            );
            query.push_str(
                "last_status_code = $last_status_code, last_error = $last_error, updated_at = <datetime>$updated_at",
            );
            if update.next_attempt_at_ms.is_some() {
                query.push_str(", next_attempt_at = <datetime>$next_attempt_at");
            } else {
                query.push_str(", next_attempt_at = NONE");
            }
            if update.request_id.is_some() {
                query.push_str(", request_id = $request_id");
            }
            if update.correlation_id.is_some() {
                query.push_str(", correlation_id = $correlation_id");
            }
            query.push_str(" WHERE event_id = $event_id");
            let updated_at = Self::to_rfc3339(gotong_domain::jobs::now_ms())?;
            let next_attempt_at = update
                .next_attempt_at_ms
                .map(Self::to_rfc3339)
                .transpose()?;
            let mut pending = client
                .query(&query)
                .bind(("status", update.status.as_str()));
            pending = pending.bind(("attempts", update.attempts as i64));
            pending = pending.bind(("max_attempts", update.max_attempts as i64));
            pending = pending.bind(("last_status_code", update.last_status_code));
            pending = pending.bind(("last_error", update.last_error));
            pending = pending.bind(("updated_at", updated_at));
            if let Some(next_attempt_at) = next_attempt_at {
                pending = pending.bind(("next_attempt_at", next_attempt_at));
            }
            if let Some(request_id) = update.request_id.as_ref() {
                pending = pending.bind(("request_id", request_id.to_string()));
            }
            if let Some(correlation_id) = update.correlation_id.as_ref() {
                pending = pending.bind(("correlation_id", correlation_id.to_string()));
            }
            pending = pending.bind(("event_id", event_id_for_query.clone()));
            pending.await.map_err(Self::map_surreal_error)?;
            let mut response = client
                .query(
                    "SELECT event_id, event_type, payload, actor_id, actor_username, request_id, correlation_id, \
                            status, attempts, max_attempts, \
                            IF next_attempt_at = NONE { NONE } ELSE { <string>next_attempt_at } AS next_attempt_at, \
                            last_status_code, last_error, <string>created_at AS created_at, \
                            <string>updated_at AS updated_at \
                     FROM webhook_outbox_event WHERE event_id = $event_id LIMIT 1",
                )
                .bind(("event_id", event_id_for_query))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::decode_event_rows(rows)?;
            events.pop().ok_or(DomainError::NotFound)
        })
    }

    fn append_log(
        &self,
        log: &WebhookDeliveryLog,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<WebhookDeliveryLog>> {
        let row = SurrealWebhookDeliveryLogCreateRow {
            log_id: log.log_id.clone(),
            event_id: log.event_id.clone(),
            attempt: log.attempt,
            outcome: log.outcome.as_str().to_string(),
            status_code: log.status_code,
            request_id: log.request_id.clone(),
            correlation_id: log.correlation_id.clone(),
            request_body_sha256: log.request_body_sha256.clone(),
            response_body_sha256: log.response_body_sha256.clone(),
            error_message: log.error_message.clone(),
            created_at: match SurrealWebhookOutboxRepository::to_rfc3339(log.created_at_ms) {
                Ok(created_at) => created_at,
                Err(err) => return Box::pin(async move { Err(err) }),
            },
        };
        let log_id = row.log_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            client
                .query(
                    "CREATE webhook_delivery_log SET \
                        log_id = $log_id, \
                        event_id = $event_id, \
                        attempt = $attempt, \
                        outcome = $outcome, \
                        status_code = $status_code, \
                        request_id = $request_id, \
                        correlation_id = $correlation_id, \
                        request_body_sha256 = $request_body_sha256, \
                        response_body_sha256 = $response_body_sha256, \
                        error_message = $error_message, \
                        created_at = <datetime>$created_at",
                )
                .bind(("log_id", row.log_id))
                .bind(("event_id", row.event_id))
                .bind(("attempt", row.attempt as i64))
                .bind(("outcome", row.outcome))
                .bind(("status_code", row.status_code.map(i64::from)))
                .bind(("request_id", row.request_id))
                .bind(("correlation_id", row.correlation_id))
                .bind(("request_body_sha256", row.request_body_sha256))
                .bind(("response_body_sha256", row.response_body_sha256))
                .bind(("error_message", row.error_message))
                .bind(("created_at", row.created_at))
                .await
                .map_err(Self::map_surreal_error)?;
            let mut response = client
                .query(
                    "SELECT log_id, event_id, attempt, outcome, status_code, request_id, correlation_id, \
                            request_body_sha256, response_body_sha256, error_message, \
                            <string>created_at AS created_at \
                     FROM webhook_delivery_log WHERE log_id = $log_id LIMIT 1",
                )
                .bind(("log_id", log_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut logs = Self::decode_delivery_logs(rows)?;
            logs.pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn list_logs(
        &self,
        event_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<WebhookDeliveryLog>>> {
        let event_id = event_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT log_id, event_id, attempt, outcome, status_code, request_id, correlation_id, \
                            request_body_sha256, response_body_sha256, error_message, \
                            <string>created_at AS created_at \
                     FROM webhook_delivery_log WHERE event_id = $event_id ORDER BY attempt DESC",
                )
                .bind(("event_id", event_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_delivery_logs(rows)
        })
    }
}

impl InMemoryGroupRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl GroupRepository for InMemoryGroupRepository {
    fn create_group(
        &self,
        group: &GroupRecord,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<GroupRecord>> {
        let group = group.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut store = store.write().await;
            if store.contains_key(&group.group_id) {
                return Err(DomainError::Conflict);
            }
            store.insert(group.group_id.clone(), group.clone());
            Ok(group)
        })
    }

    fn get_group(
        &self,
        group_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<GroupRecord>>> {
        let group_id = group_id.to_string();
        let store = self.store.clone();
        Box::pin(async move { Ok(store.read().await.get(&group_id).cloned()) })
    }

    fn list_groups(&self) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<GroupRecord>>> {
        let store = self.store.clone();
        Box::pin(async move { Ok(store.read().await.values().cloned().collect::<Vec<_>>()) })
    }

    fn update_group(
        &self,
        group: &GroupRecord,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<GroupRecord>> {
        let group = group.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut store = store.write().await;
            if !store.contains_key(&group.group_id) {
                return Err(DomainError::NotFound);
            }
            store.insert(group.group_id.clone(), group.clone());
            Ok(group)
        })
    }
}

#[cfg(test)]
mod group_repository_tests {
    use super::*;

    fn sample_group(group_id: &str) -> GroupRecord {
        GroupRecord {
            group_id: group_id.to_string(),
            name: "Karang Taruna".to_string(),
            description: "Kelompok warga".to_string(),
            entity_type: "kelompok".to_string(),
            join_policy: "persetujuan".to_string(),
            member_count: 1,
            witness_count: 0,
            members: vec![GroupMemberRecord {
                user_id: "user-1".to_string(),
                name: "user-1".to_string(),
                avatar_url: None,
                role: "admin".to_string(),
                joined_at_ms: 1_700_000_000_000,
            }],
            pending_requests: Vec::new(),
            updated_at_ms: 1_700_000_000_000,
        }
    }

    #[tokio::test]
    async fn in_memory_group_repository_create_get_update_list_roundtrip() {
        let repo = InMemoryGroupRepository::new();
        let group = sample_group("g-1");

        let created = repo.create_group(&group).await.expect("create");
        assert_eq!(created.group_id, "g-1");
        assert_eq!(created.member_count, 1);

        let fetched = repo.get_group("g-1").await.expect("get").expect("exists");
        assert_eq!(fetched.join_policy, "persetujuan");

        let mut updated = fetched.clone();
        updated.join_policy = "terbuka".to_string();
        updated.updated_at_ms += 1000;
        let saved = repo.update_group(&updated).await.expect("update");
        assert_eq!(saved.join_policy, "terbuka");

        let listed = repo.list_groups().await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].group_id, "g-1");
        assert_eq!(listed[0].join_policy, "terbuka");
    }

    #[tokio::test]
    async fn in_memory_group_repository_create_conflict() {
        let repo = InMemoryGroupRepository::new();
        let group = sample_group("g-dup");
        repo.create_group(&group).await.expect("first create");
        let err = repo
            .create_group(&group)
            .await
            .expect_err("duplicate should conflict");
        assert!(matches!(err, DomainError::Conflict));
    }
}

#[derive(Clone)]
pub struct SurrealGroupRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealGroupRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let datetime =
            OffsetDateTime::from_unix_timestamp_nanos((timestamp_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(datetime
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn decode_summary_rows(rows: Vec<Value>) -> DomainResult<Vec<SurrealGroupSummaryRow>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealGroupSummaryRow>(row).map_err(|err| {
                    DomainError::Validation(format!("invalid group summary row: {err}"))
                })
            })
            .collect()
    }

    fn decode_member_rows(rows: Vec<Value>) -> DomainResult<Vec<GroupMemberRecord>> {
        rows.into_iter()
            .map(|row| {
                let row = serde_json::from_value::<SurrealGroupMemberRow>(row).map_err(|err| {
                    DomainError::Validation(format!("invalid group member row: {err}"))
                })?;
                Ok(GroupMemberRecord {
                    user_id: row.user_id,
                    name: row.name,
                    avatar_url: row.avatar_url,
                    role: row.role,
                    joined_at_ms: Self::parse_datetime_ms(&row.joined_at)?,
                })
            })
            .collect()
    }

    fn decode_request_rows(rows: Vec<Value>) -> DomainResult<Vec<GroupJoinRequestRecord>> {
        rows.into_iter()
            .map(|row| {
                let row =
                    serde_json::from_value::<SurrealGroupJoinRequestRow>(row).map_err(|err| {
                        DomainError::Validation(format!("invalid group request row: {err}"))
                    })?;
                Ok(GroupJoinRequestRecord {
                    request_id: row.request_id,
                    user_id: row.user_id,
                    name: row.name,
                    avatar_url: row.avatar_url,
                    message: row.message,
                    status: row.status,
                    requested_at_ms: Self::parse_datetime_ms(&row.requested_at)?,
                })
            })
            .collect()
    }

    fn map_group_record(
        summary: SurrealGroupSummaryRow,
        members: Vec<GroupMemberRecord>,
        pending_requests: Vec<GroupJoinRequestRecord>,
    ) -> DomainResult<GroupRecord> {
        Ok(GroupRecord {
            group_id: summary.group_id,
            name: summary.name,
            description: summary.description,
            entity_type: summary.entity_type,
            join_policy: summary.join_policy,
            member_count: summary.member_count,
            witness_count: summary.witness_count,
            members,
            pending_requests,
            updated_at_ms: Self::parse_datetime_ms(&summary.updated_at)?,
        })
    }
}

#[derive(Debug, Serialize)]
struct SurrealGroupSummaryCreateRow {
    group_id: String,
    name: String,
    description: String,
    entity_type: String,
    join_policy: String,
    member_count: usize,
    witness_count: usize,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealGroupSummaryRow {
    group_id: String,
    name: String,
    description: String,
    entity_type: String,
    join_policy: String,
    member_count: usize,
    witness_count: usize,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SurrealGroupMemberCreateRow {
    group_id: String,
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    role: String,
    joined_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealGroupMemberRow {
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    role: String,
    joined_at: String,
}

#[derive(Debug, Serialize)]
struct SurrealGroupJoinRequestCreateRow {
    request_id: String,
    group_id: String,
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    message: Option<String>,
    status: String,
    requested_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealGroupJoinRequestRow {
    request_id: String,
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    message: Option<String>,
    status: String,
    requested_at: String,
}

impl GroupRepository for SurrealGroupRepository {
    fn create_group(
        &self,
        group: &GroupRecord,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<GroupRecord>> {
        let group = group.clone();
        let summary = match SurrealGroupRepository::to_rfc3339(group.updated_at_ms) {
            Ok(updated_at) => SurrealGroupSummaryCreateRow {
                group_id: group.group_id.clone(),
                name: group.name.clone(),
                description: group.description.clone(),
                entity_type: group.entity_type.clone(),
                join_policy: group.join_policy.clone(),
                member_count: group.member_count,
                witness_count: group.witness_count,
                updated_at,
            },
            Err(err) => return Box::pin(async move { Err(err) }),
        };

        let members = match group
            .members
            .iter()
            .map(|member| {
                Ok(SurrealGroupMemberCreateRow {
                    group_id: group.group_id.clone(),
                    user_id: member.user_id.clone(),
                    name: member.name.clone(),
                    avatar_url: member.avatar_url.clone(),
                    role: member.role.clone(),
                    joined_at: SurrealGroupRepository::to_rfc3339(member.joined_at_ms)?,
                })
            })
            .collect::<DomainResult<Vec<_>>>()
        {
            Ok(members) => members,
            Err(err) => return Box::pin(async move { Err(err) }),
        };

        let requests = match group
            .pending_requests
            .iter()
            .map(|request| {
                Ok(SurrealGroupJoinRequestCreateRow {
                    request_id: request.request_id.clone(),
                    group_id: group.group_id.clone(),
                    user_id: request.user_id.clone(),
                    name: request.name.clone(),
                    avatar_url: request.avatar_url.clone(),
                    message: request.message.clone(),
                    status: request.status.clone(),
                    requested_at: SurrealGroupRepository::to_rfc3339(request.requested_at_ms)?,
                })
            })
            .collect::<DomainResult<Vec<_>>>()
        {
            Ok(requests) => requests,
            Err(err) => return Box::pin(async move { Err(err) }),
        };

        let client = self.client.clone();
        Box::pin(async move {
            let summary_payload = to_value(summary).map_err(|err| {
                DomainError::Validation(format!("invalid summary payload: {err}"))
            })?;
            let mut summary_response = client
                .query("CREATE group_summary CONTENT $payload")
                .bind(("payload", summary_payload))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let summary_rows: Vec<Value> = summary_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if summary_rows.is_empty() {
                return Err(DomainError::Validation(
                    "create group summary returned no row".to_string(),
                ));
            }

            for member in members {
                let payload = to_value(member).map_err(|err| {
                    DomainError::Validation(format!("invalid member payload: {err}"))
                })?;
                client
                    .query("CREATE group_member CONTENT $payload")
                    .bind(("payload", payload))
                    .await
                    .map_err(SurrealGroupRepository::map_surreal_error)?;
            }

            for request in requests {
                let payload = to_value(request).map_err(|err| {
                    DomainError::Validation(format!("invalid request payload: {err}"))
                })?;
                client
                    .query("CREATE group_join_request CONTENT $payload")
                    .bind(("payload", payload))
                    .await
                    .map_err(SurrealGroupRepository::map_surreal_error)?;
            }

            Ok(group)
        })
    }

    fn get_group(
        &self,
        group_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<GroupRecord>>> {
        let group_id = group_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut summary_response = client
                .query(
                    "SELECT group_id, name, description, entity_type, join_policy, member_count, witness_count, \
                     <string>updated_at AS updated_at \
                     FROM group_summary WHERE group_id = $group_id LIMIT 1",
                )
                .bind(("group_id", group_id.clone()))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let summary_rows: Vec<Value> = summary_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut summaries = SurrealGroupRepository::decode_summary_rows(summary_rows)?;
            let Some(summary) = summaries.pop() else {
                return Ok(None);
            };

            let mut member_response = client
                .query(
                    "SELECT user_id, name, avatar_url, role, <string>joined_at AS joined_at \
                     FROM group_member WHERE group_id = $group_id \
                     ORDER BY joined_at ASC, user_id ASC",
                )
                .bind(("group_id", group_id.clone()))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let member_rows: Vec<Value> = member_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let members = SurrealGroupRepository::decode_member_rows(member_rows)?;

            let mut request_response = client
                .query(
                    "SELECT request_id, user_id, name, avatar_url, message, status, \
                     <string>requested_at AS requested_at \
                     FROM group_join_request WHERE group_id = $group_id \
                     ORDER BY requested_at DESC, request_id DESC",
                )
                .bind(("group_id", group_id))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let request_rows: Vec<Value> = request_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let requests = SurrealGroupRepository::decode_request_rows(request_rows)?;

            let record = SurrealGroupRepository::map_group_record(summary, members, requests)?;
            Ok(Some(record))
        })
    }

    fn list_groups(&self) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<GroupRecord>>> {
        let client = self.client.clone();
        Box::pin(async move {
            let mut summary_response = client
                .query(
                    "SELECT group_id, name, description, entity_type, join_policy, member_count, witness_count, \
                     <string>updated_at AS updated_at FROM group_summary",
                )
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let summary_rows: Vec<Value> = summary_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let summaries = SurrealGroupRepository::decode_summary_rows(summary_rows)?;

            let mut member_response = client
                .query(
                    "SELECT group_id, user_id, name, avatar_url, role, <string>joined_at AS joined_at \
                     FROM group_member ORDER BY joined_at ASC, user_id ASC",
                )
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let member_rows: Vec<Value> = member_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let parsed_members = member_rows
                .into_iter()
                .map(|row| {
                    #[derive(Deserialize)]
                    struct GroupMemberByGroupRow {
                        group_id: String,
                        user_id: String,
                        name: String,
                        avatar_url: Option<String>,
                        role: String,
                        joined_at: String,
                    }
                    let row =
                        serde_json::from_value::<GroupMemberByGroupRow>(row).map_err(|err| {
                            DomainError::Validation(format!("invalid group member row: {err}"))
                        })?;
                    Ok((
                        row.group_id,
                        GroupMemberRecord {
                            user_id: row.user_id,
                            name: row.name,
                            avatar_url: row.avatar_url,
                            role: row.role,
                            joined_at_ms: SurrealGroupRepository::parse_datetime_ms(
                                &row.joined_at,
                            )?,
                        },
                    ))
                })
                .collect::<DomainResult<Vec<_>>>()?;
            let mut members_by_group = HashMap::<String, Vec<GroupMemberRecord>>::new();
            for (group_id, member) in parsed_members {
                members_by_group.entry(group_id).or_default().push(member);
            }

            let mut request_response = client
                .query(
                    "SELECT group_id, request_id, user_id, name, avatar_url, message, status, \
                     <string>requested_at AS requested_at FROM group_join_request \
                     ORDER BY requested_at DESC, request_id DESC",
                )
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let request_rows: Vec<Value> = request_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let parsed_requests = request_rows
                .into_iter()
                .map(|row| {
                    #[derive(Deserialize)]
                    struct GroupRequestByGroupRow {
                        group_id: String,
                        request_id: String,
                        user_id: String,
                        name: String,
                        avatar_url: Option<String>,
                        message: Option<String>,
                        status: String,
                        requested_at: String,
                    }
                    let row =
                        serde_json::from_value::<GroupRequestByGroupRow>(row).map_err(|err| {
                            DomainError::Validation(format!("invalid group request row: {err}"))
                        })?;
                    Ok((
                        row.group_id,
                        GroupJoinRequestRecord {
                            request_id: row.request_id,
                            user_id: row.user_id,
                            name: row.name,
                            avatar_url: row.avatar_url,
                            message: row.message,
                            status: row.status,
                            requested_at_ms: SurrealGroupRepository::parse_datetime_ms(
                                &row.requested_at,
                            )?,
                        },
                    ))
                })
                .collect::<DomainResult<Vec<_>>>()?;
            let mut requests_by_group = HashMap::<String, Vec<GroupJoinRequestRecord>>::new();
            for (group_id, request) in parsed_requests {
                requests_by_group.entry(group_id).or_default().push(request);
            }

            summaries
                .into_iter()
                .map(|summary| {
                    let members = members_by_group
                        .remove(&summary.group_id)
                        .unwrap_or_default();
                    let requests = requests_by_group
                        .remove(&summary.group_id)
                        .unwrap_or_default();
                    SurrealGroupRepository::map_group_record(summary, members, requests)
                })
                .collect::<DomainResult<Vec<_>>>()
        })
    }

    fn update_group(
        &self,
        group: &GroupRecord,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<GroupRecord>> {
        let group = group.clone();
        let summary = match SurrealGroupRepository::to_rfc3339(group.updated_at_ms) {
            Ok(updated_at) => SurrealGroupSummaryCreateRow {
                group_id: group.group_id.clone(),
                name: group.name.clone(),
                description: group.description.clone(),
                entity_type: group.entity_type.clone(),
                join_policy: group.join_policy.clone(),
                member_count: group.member_count,
                witness_count: group.witness_count,
                updated_at,
            },
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let members = match group
            .members
            .iter()
            .map(|member| {
                Ok(SurrealGroupMemberCreateRow {
                    group_id: group.group_id.clone(),
                    user_id: member.user_id.clone(),
                    name: member.name.clone(),
                    avatar_url: member.avatar_url.clone(),
                    role: member.role.clone(),
                    joined_at: SurrealGroupRepository::to_rfc3339(member.joined_at_ms)?,
                })
            })
            .collect::<DomainResult<Vec<_>>>()
        {
            Ok(members) => members,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let requests = match group
            .pending_requests
            .iter()
            .map(|request| {
                Ok(SurrealGroupJoinRequestCreateRow {
                    request_id: request.request_id.clone(),
                    group_id: group.group_id.clone(),
                    user_id: request.user_id.clone(),
                    name: request.name.clone(),
                    avatar_url: request.avatar_url.clone(),
                    message: request.message.clone(),
                    status: request.status.clone(),
                    requested_at: SurrealGroupRepository::to_rfc3339(request.requested_at_ms)?,
                })
            })
            .collect::<DomainResult<Vec<_>>>()
        {
            Ok(requests) => requests,
            Err(err) => return Box::pin(async move { Err(err) }),
        };

        let client = self.client.clone();
        Box::pin(async move {
            let summary_payload = to_value(summary).map_err(|err| {
                DomainError::Validation(format!("invalid summary payload: {err}"))
            })?;
            let mut update_response = client
                .query(
                    "UPDATE group_summary SET \
                        name = $name, \
                        description = $description, \
                        entity_type = $entity_type, \
                        join_policy = $join_policy, \
                        member_count = $member_count, \
                        witness_count = $witness_count, \
                        updated_at = <datetime>$updated_at \
                     WHERE group_id = $group_id \
                     RETURN AFTER",
                )
                .bind(("group_id", group.group_id.clone()))
                .bind(("name", summary_payload["name"].clone()))
                .bind(("description", summary_payload["description"].clone()))
                .bind(("entity_type", summary_payload["entity_type"].clone()))
                .bind(("join_policy", summary_payload["join_policy"].clone()))
                .bind(("member_count", summary_payload["member_count"].clone()))
                .bind(("witness_count", summary_payload["witness_count"].clone()))
                .bind(("updated_at", summary_payload["updated_at"].clone()))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            let updated_rows: Vec<Value> = update_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if updated_rows.is_empty() {
                return Err(DomainError::NotFound);
            }

            client
                .query("DELETE group_member WHERE group_id = $group_id")
                .bind(("group_id", group.group_id.clone()))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            for member in members {
                let payload = to_value(member).map_err(|err| {
                    DomainError::Validation(format!("invalid member payload: {err}"))
                })?;
                client
                    .query("CREATE group_member CONTENT $payload")
                    .bind(("payload", payload))
                    .await
                    .map_err(SurrealGroupRepository::map_surreal_error)?;
            }

            client
                .query("DELETE group_join_request WHERE group_id = $group_id")
                .bind(("group_id", group.group_id.clone()))
                .await
                .map_err(SurrealGroupRepository::map_surreal_error)?;
            for request in requests {
                let payload = to_value(request).map_err(|err| {
                    DomainError::Validation(format!("invalid request payload: {err}"))
                })?;
                client
                    .query("CREATE group_join_request CONTENT $payload")
                    .bind(("payload", payload))
                    .await
                    .map_err(SurrealGroupRepository::map_surreal_error)?;
            }

            Ok(group)
        })
    }
}

#[derive(Clone)]
pub struct SurrealContributionRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealContributionRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(timestamp_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
            DomainError::Validation(format!("invalid contribution datetime '{value}': {err}"))
        })?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn contribution_type_to_string(value: &ContributionType) -> &'static str {
        match value {
            ContributionType::TaskCompletion => "task_completion",
            ContributionType::CodeReview => "code_review",
            ContributionType::Documentation => "documentation",
            ContributionType::Mentoring => "mentoring",
            ContributionType::EventOrganization => "event_organization",
            ContributionType::CommunityService => "community_service",
            ContributionType::Custom => "custom",
        }
    }

    fn parse_contribution_type(value: &str) -> DomainResult<ContributionType> {
        match value {
            "task_completion" => Ok(ContributionType::TaskCompletion),
            "code_review" => Ok(ContributionType::CodeReview),
            "documentation" => Ok(ContributionType::Documentation),
            "mentoring" => Ok(ContributionType::Mentoring),
            "event_organization" => Ok(ContributionType::EventOrganization),
            "community_service" => Ok(ContributionType::CommunityService),
            "custom" => Ok(ContributionType::Custom),
            _ => Err(DomainError::Validation(format!(
                "invalid contribution_type '{value}'"
            ))),
        }
    }

    fn mode_to_string(value: &Mode) -> &'static str {
        value.as_str()
    }

    fn parse_mode(value: &str) -> DomainResult<Mode> {
        match value {
            "komunitas" => Ok(Mode::Komunitas),
            "catatan_komunitas" => Ok(Mode::CatatanKomunitas),
            "catatan_saksi" => Ok(Mode::CatatanSaksi),
            "siaga" => Ok(Mode::Siaga),
            _ => Err(DomainError::Validation(format!("invalid mode '{value}'"))),
        }
    }

    fn build_payload(contribution: &Contribution) -> DomainResult<SurrealContributionCreateRow> {
        let created_at = Self::to_rfc3339(contribution.created_at_ms)?;
        let updated_at = Self::to_rfc3339(contribution.updated_at_ms)?;
        Ok(SurrealContributionCreateRow {
            contribution_id: contribution.contribution_id.clone(),
            author_id: contribution.author_id.clone(),
            author_username: contribution.author_username.clone(),
            mode: Self::mode_to_string(&contribution.mode).to_string(),
            contribution_type: Self::contribution_type_to_string(&contribution.contribution_type)
                .to_string(),
            title: contribution.title.clone(),
            description: contribution.description.clone(),
            evidence_url: contribution.evidence_url.clone(),
            skill_ids: contribution.skill_ids.clone(),
            metadata: contribution.metadata.clone(),
            request_id: contribution.request_id.clone(),
            correlation_id: contribution.correlation_id.clone(),
            created_at,
            updated_at,
        })
    }

    fn map_row(row: SurrealContributionRow) -> DomainResult<Contribution> {
        Ok(Contribution {
            contribution_id: row.contribution_id,
            author_id: row.author_id,
            author_username: row.author_username,
            mode: Self::parse_mode(&row.mode)?,
            contribution_type: Self::parse_contribution_type(&row.contribution_type)?,
            title: row.title,
            description: row.description,
            evidence_url: row.evidence_url,
            skill_ids: row.skill_ids,
            metadata: row.metadata,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
        })
    }

    fn decode_rows(rows: Vec<Value>) -> DomainResult<Vec<Contribution>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealContributionRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid contribution row: {err}"))
                    })
                    .and_then(Self::map_row)
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct SurrealContributionCreateRow {
    contribution_id: String,
    author_id: String,
    author_username: String,
    mode: String,
    contribution_type: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_url: Option<String>,
    skill_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Value>>,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealContributionRow {
    contribution_id: String,
    author_id: String,
    author_username: String,
    mode: String,
    contribution_type: String,
    title: String,
    description: Option<String>,
    evidence_url: Option<String>,
    skill_ids: Vec<String>,
    metadata: Option<HashMap<String, Value>>,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

impl ContributionRepository for SurrealContributionRepository {
    fn create(
        &self,
        contribution: &Contribution,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Contribution>> {
        let payload = match Self::build_payload(contribution) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let contribution_id = contribution.contribution_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query(
                    "CREATE type::record('contribution', $contribution_id) SET \
                        contribution_id = $payload.contribution_id, \
                        author_id = $payload.author_id, \
                        author_username = $payload.author_username, \
                        mode = $payload.mode, \
                        contribution_type = $payload.contribution_type, \
                        title = $payload.title, \
                        description = $payload.description, \
                        evidence_url = $payload.evidence_url, \
                        skill_ids = $payload.skill_ids, \
                        metadata = $payload.metadata, \
                        request_id = $payload.request_id, \
                        correlation_id = $payload.correlation_id, \
                        created_at = <datetime>$payload.created_at, \
                        updated_at = <datetime>$payload.updated_at; \
                     SELECT contribution_id, author_id, author_username, mode, contribution_type, title, \
                            description, evidence_url, skill_ids, metadata, request_id, correlation_id, \
                            <string>created_at AS created_at, <string>updated_at AS updated_at \
                     FROM contribution WHERE contribution_id = $contribution_id LIMIT 1",
                )
                .bind(("contribution_id", contribution_id))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut contributions = Self::decode_rows(rows)?;
            contributions
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn get(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Contribution>>> {
        let contribution_id = contribution_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM contribution WHERE contribution_id = $contribution_id LIMIT 1",
                )
                .bind(("contribution_id", contribution_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut contributions = Self::decode_rows(rows)?;
            Ok(contributions.pop())
        })
    }

    fn list_by_author(
        &self,
        author_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM contribution \
                     WHERE author_id = $author_id \
                     ORDER BY created_at DESC, contribution_id DESC",
                )
                .bind(("author_id", author_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut contributions = Self::decode_rows(rows)?;
            contributions.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.contribution_id.cmp(&left.contribution_id))
            });
            Ok(contributions)
        })
    }

    fn list_recent(
        &self,
        author_id: &str,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let limit = limit as i64;
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM contribution \
                     WHERE author_id = $author_id \
                     ORDER BY created_at DESC, contribution_id DESC \
                     LIMIT $limit",
                )
                .bind(("author_id", author_id))
                .bind(("limit", limit))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut contributions = Self::decode_rows(rows)?;
            contributions.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.contribution_id.cmp(&left.contribution_id))
            });
            Ok(contributions)
        })
    }
}

#[derive(Clone)]
pub struct SurrealEvidenceRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealEvidenceRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(timestamp_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
            DomainError::Validation(format!("invalid evidence datetime '{value}': {err}"))
        })?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn evidence_type_to_string(value: &EvidenceType) -> &'static str {
        match value {
            EvidenceType::PhotoWithTimestamp => "photo_with_timestamp",
            EvidenceType::GpsVerification => "gps_verification",
            EvidenceType::WitnessAttestation => "witness_attestation",
        }
    }

    fn parse_evidence_type(value: &str) -> DomainResult<EvidenceType> {
        match value {
            "photo_with_timestamp" => Ok(EvidenceType::PhotoWithTimestamp),
            "gps_verification" => Ok(EvidenceType::GpsVerification),
            "witness_attestation" => Ok(EvidenceType::WitnessAttestation),
            _ => Err(DomainError::Validation(format!(
                "invalid evidence_type '{value}'"
            ))),
        }
    }

    fn build_payload(evidence: &Evidence) -> DomainResult<SurrealEvidenceCreateRow> {
        let created_at = Self::to_rfc3339(evidence.created_at_ms)?;
        let updated_at = Self::to_rfc3339(evidence.updated_at_ms)?;
        Ok(SurrealEvidenceCreateRow {
            evidence_id: evidence.evidence_id.clone(),
            contribution_id: evidence.contribution_id.clone(),
            actor_id: evidence.actor_id.clone(),
            actor_username: evidence.actor_username.clone(),
            evidence_type: Self::evidence_type_to_string(&evidence.evidence_type).to_string(),
            evidence_data: evidence.evidence_data.clone(),
            proof: evidence.proof.clone(),
            request_id: evidence.request_id.clone(),
            correlation_id: evidence.correlation_id.clone(),
            created_at,
            updated_at,
        })
    }

    fn map_row(row: SurrealEvidenceRow) -> DomainResult<Evidence> {
        Ok(Evidence {
            evidence_id: row.evidence_id,
            contribution_id: row.contribution_id,
            actor_id: row.actor_id,
            actor_username: row.actor_username,
            evidence_type: Self::parse_evidence_type(&row.evidence_type)?,
            evidence_data: row.evidence_data,
            proof: row.proof,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
        })
    }

    fn decode_rows(rows: Vec<Value>) -> DomainResult<Vec<Evidence>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealEvidenceRow>(row)
                    .map_err(|err| DomainError::Validation(format!("invalid evidence row: {err}")))
                    .and_then(Self::map_row)
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct SurrealEvidenceCreateRow {
    evidence_id: String,
    contribution_id: String,
    actor_id: String,
    actor_username: String,
    evidence_type: String,
    evidence_data: Value,
    proof: Value,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealEvidenceRow {
    evidence_id: String,
    contribution_id: String,
    actor_id: String,
    actor_username: String,
    evidence_type: String,
    evidence_data: Value,
    proof: Value,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

impl EvidenceRepository for SurrealEvidenceRepository {
    fn create(
        &self,
        evidence: &Evidence,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Evidence>> {
        let payload = match Self::build_payload(evidence) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let evidence_id = evidence.evidence_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query(
                    "CREATE type::record('evidence', $evidence_id) SET \
                        evidence_id = $payload.evidence_id, \
                        contribution_id = $payload.contribution_id, \
                        actor_id = $payload.actor_id, \
                        actor_username = $payload.actor_username, \
                        evidence_type = $payload.evidence_type, \
                        evidence_data = $payload.evidence_data, \
                        proof = $payload.proof, \
                        request_id = $payload.request_id, \
                        correlation_id = $payload.correlation_id, \
                        created_at = <datetime>$payload.created_at, \
                        updated_at = <datetime>$payload.updated_at; \
                     SELECT evidence_id, contribution_id, actor_id, actor_username, evidence_type, \
                            evidence_data, proof, request_id, correlation_id, \
                            <string>created_at AS created_at, <string>updated_at AS updated_at \
                     FROM evidence WHERE evidence_id = $evidence_id LIMIT 1",
                )
                .bind(("evidence_id", evidence_id))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut evidences = Self::decode_rows(rows)?;
            evidences
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn get(
        &self,
        evidence_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Evidence>>> {
        let evidence_id = evidence_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM evidence WHERE evidence_id = $evidence_id LIMIT 1")
                .bind(("evidence_id", evidence_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut evidences = Self::decode_rows(rows)?;
            Ok(evidences.pop())
        })
    }

    fn list_by_contribution(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Evidence>>> {
        let contribution_id = contribution_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM evidence \
                     WHERE contribution_id = $contribution_id \
                     ORDER BY created_at DESC, evidence_id DESC",
                )
                .bind(("contribution_id", contribution_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut evidences = Self::decode_rows(rows)?;
            evidences.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.evidence_id.cmp(&left.evidence_id))
            });
            Ok(evidences)
        })
    }
}

#[derive(Clone)]
pub struct SurrealVouchRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealVouchRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(timestamp_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
            DomainError::Validation(format!("invalid vouch datetime '{value}': {err}"))
        })?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn weight_hint_to_string(value: &Option<VouchWeightHint>) -> Option<&'static str> {
        value.as_ref().map(|hint| match hint {
            VouchWeightHint::Strong => "strong",
            VouchWeightHint::Moderate => "moderate",
            VouchWeightHint::Light => "light",
        })
    }

    fn parse_weight_hint(value: &str) -> DomainResult<VouchWeightHint> {
        match value {
            "strong" => Ok(VouchWeightHint::Strong),
            "moderate" => Ok(VouchWeightHint::Moderate),
            "light" => Ok(VouchWeightHint::Light),
            _ => Err(DomainError::Validation(format!(
                "invalid vouch weight_hint '{value}'"
            ))),
        }
    }

    fn build_payload(vouch: &Vouch) -> DomainResult<SurrealVouchCreateRow> {
        let created_at = Self::to_rfc3339(vouch.created_at_ms)?;
        let updated_at = Self::to_rfc3339(vouch.updated_at_ms)?;
        Ok(SurrealVouchCreateRow {
            vouch_id: vouch.vouch_id.clone(),
            voucher_id: vouch.voucher_id.clone(),
            voucher_username: vouch.voucher_username.clone(),
            vouchee_id: vouch.vouchee_id.clone(),
            skill_id: vouch.skill_id.clone(),
            weight_hint: Self::weight_hint_to_string(&vouch.weight_hint)
                .map(|value| value.to_string()),
            message: vouch.message.clone(),
            request_id: vouch.request_id.clone(),
            correlation_id: vouch.correlation_id.clone(),
            created_at,
            updated_at,
        })
    }

    fn map_row(row: SurrealVouchRow) -> DomainResult<Vouch> {
        Ok(Vouch {
            vouch_id: row.vouch_id,
            voucher_id: row.voucher_id,
            voucher_username: row.voucher_username,
            vouchee_id: row.vouchee_id,
            skill_id: row.skill_id,
            weight_hint: row
                .weight_hint
                .as_ref()
                .map(|value| Self::parse_weight_hint(value))
                .transpose()?,
            message: row.message,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
        })
    }

    fn decode_rows(rows: Vec<Value>) -> DomainResult<Vec<Vouch>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealVouchRow>(row)
                    .map_err(|err| DomainError::Validation(format!("invalid vouch row: {err}")))
                    .and_then(Self::map_row)
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct SurrealVouchCreateRow {
    vouch_id: String,
    voucher_id: String,
    voucher_username: String,
    vouchee_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    skill_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct SurrealVouchRow {
    vouch_id: String,
    voucher_id: String,
    voucher_username: String,
    vouchee_id: String,
    skill_id: Option<String>,
    weight_hint: Option<String>,
    message: Option<String>,
    request_id: String,
    correlation_id: String,
    created_at: String,
    updated_at: String,
}

impl VouchRepository for SurrealVouchRepository {
    fn create(&self, vouch: &Vouch) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vouch>> {
        let payload = match Self::build_payload(vouch) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let vouch_id = vouch.vouch_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query(
                    "CREATE type::record('vouch', $vouch_id) SET \
                        vouch_id = $payload.vouch_id, \
                        voucher_id = $payload.voucher_id, \
                        voucher_username = $payload.voucher_username, \
                        vouchee_id = $payload.vouchee_id, \
                        skill_id = $payload.skill_id, \
                        weight_hint = $payload.weight_hint, \
                        message = $payload.message, \
                        request_id = $payload.request_id, \
                        correlation_id = $payload.correlation_id, \
                        created_at = <datetime>$payload.created_at, \
                        updated_at = <datetime>$payload.updated_at; \
                     SELECT vouch_id, voucher_id, voucher_username, vouchee_id, skill_id, weight_hint, \
                            message, request_id, correlation_id, \
                            <string>created_at AS created_at, <string>updated_at AS updated_at \
                     FROM vouch WHERE vouch_id = $vouch_id LIMIT 1",
                )
                .bind(("vouch_id", vouch_id))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut vouches = Self::decode_rows(rows)?;
            vouches
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn list_by_vouchee(
        &self,
        vouchee_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let vouchee_id = vouchee_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM vouch \
                     WHERE vouchee_id = $vouchee_id \
                     ORDER BY created_at DESC, vouch_id DESC",
                )
                .bind(("vouchee_id", vouchee_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut vouches = Self::decode_rows(rows)?;
            vouches.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.vouch_id.cmp(&left.vouch_id))
            });
            Ok(vouches)
        })
    }

    fn list_by_voucher(
        &self,
        voucher_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let voucher_id = voucher_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM vouch \
                     WHERE voucher_id = $voucher_id \
                     ORDER BY created_at DESC, vouch_id DESC",
                )
                .bind(("voucher_id", voucher_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut vouches = Self::decode_rows(rows)?;
            vouches.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.vouch_id.cmp(&left.vouch_id))
            });
            Ok(vouches)
        })
    }
}

impl InMemoryContributionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ContributionRepository for InMemoryContributionRepository {
    fn create(
        &self,
        contribution: &Contribution,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Contribution>> {
        let contribution = contribution.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&contribution.contribution_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(contribution.contribution_id.clone(), contribution.clone());
            Ok(contribution)
        })
    }

    fn get(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Contribution>>> {
        let contribution_id = contribution_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            Ok(items.get(&contribution_id).cloned())
        })
    }

    fn list_by_author(
        &self,
        author_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            let mut contributions: Vec<_> = items
                .values()
                .filter(|item| item.author_id == author_id)
                .cloned()
                .collect();
            contributions.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.contribution_id.cmp(&a.contribution_id))
            });
            Ok(contributions)
        })
    }

    fn list_recent(
        &self,
        author_id: &str,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Contribution>>> {
        let author_id = author_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = self_recent_filtered(&store, &author_id, limit).await;
            items.truncate(limit);
            Ok(items)
        })
    }
}

async fn self_recent_filtered(
    store: &Arc<RwLock<HashMap<String, Contribution>>>,
    author_id: &str,
    limit: usize,
) -> Vec<Contribution> {
    let mut contributions: Vec<_> = store
        .read()
        .await
        .values()
        .filter(|item| item.author_id == author_id)
        .cloned()
        .collect();
    contributions.sort_by(|a, b| {
        b.created_at_ms
            .cmp(&a.created_at_ms)
            .then_with(|| b.contribution_id.cmp(&a.contribution_id))
    });
    contributions.into_iter().take(limit).collect()
}

#[derive(Default)]
pub struct InMemoryOntologyRepository {
    concepts_by_id: Arc<RwLock<HashMap<String, OntologyConcept>>>,
    concepts_by_qid: Arc<RwLock<HashMap<String, String>>>,
    broader_edges: Arc<RwLock<HashMap<String, Vec<String>>>>,
    notes: Arc<RwLock<HashMap<String, OntologyNote>>>,
    triples: Arc<RwLock<Vec<OntologyTripleCreate>>>,
}

impl InMemoryOntologyRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn normalize_record_id(raw: &str, table: &str) -> String {
        let value = raw.trim();
        if value.contains(':') {
            value.to_string()
        } else {
            format!("{table}:{value}")
        }
    }

    fn id_part(raw: &str) -> String {
        raw.split_once(':')
            .map(|(_, id)| id.to_string())
            .unwrap_or_else(|| raw.to_string())
    }
}

impl OntologyRepository for InMemoryOntologyRepository {
    fn upsert_concept(
        &self,
        concept: &OntologyConcept,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<OntologyConcept>> {
        let mut concept = concept.clone();
        let concepts_by_id = self.concepts_by_id.clone();
        let concepts_by_qid = self.concepts_by_qid.clone();
        Box::pin(async move {
            if concept.qid.trim().is_empty() {
                return Err(DomainError::Validation("qid is required".to_string()));
            }
            concept.concept_id = Self::normalize_record_id(&concept.concept_id, "concept");
            concepts_by_qid
                .write()
                .await
                .insert(concept.qid.clone(), concept.concept_id.clone());
            concepts_by_id
                .write()
                .await
                .insert(concept.concept_id.clone(), concept.clone());
            Ok(concept)
        })
    }

    fn add_broader_edge(
        &self,
        narrower_concept_id: &str,
        broader_concept_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let broader_edges = self.broader_edges.clone();
        let narrower = Self::normalize_record_id(narrower_concept_id, "concept");
        let broader = Self::normalize_record_id(broader_concept_id, "concept");
        Box::pin(async move {
            let mut broader_edges = broader_edges.write().await;
            let edges = broader_edges.entry(narrower).or_default();
            if !edges.iter().any(|item| item == &broader) {
                edges.push(broader);
            }
            Ok(())
        })
    }

    fn create_note(
        &self,
        note: &OntologyNoteCreate,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<OntologyNote>> {
        let note = note.clone();
        let notes = self.notes.clone();
        Box::pin(async move {
            if note.content.trim().is_empty() {
                return Err(DomainError::Validation(
                    "note content is required".to_string(),
                ));
            }
            let note_id = note
                .note_id
                .clone()
                .unwrap_or_else(gotong_domain::util::uuid_v7_without_dashes);
            let normalized_note_id = Self::id_part(&note_id);
            let created_note = OntologyNote {
                note_id: normalized_note_id.clone(),
                content: note.content.trim().to_string(),
                author_id: note.author_id.trim().to_string(),
                community_id: note.community_id.trim().to_string(),
                temporal_class: note.temporal_class.trim().to_string(),
                ttl_expires_ms: note.ttl_expires_ms,
                ai_readable: note.ai_readable,
                rahasia_level: note.rahasia_level,
                confidence: note.confidence,
                created_at_ms: gotong_domain::jobs::now_ms(),
            };
            notes
                .write()
                .await
                .insert(normalized_note_id, created_note.clone());
            Ok(created_note)
        })
    }

    fn write_triples(
        &self,
        triples: &[OntologyTripleCreate],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let triples = triples.to_vec();
        let store = self.triples.clone();
        Box::pin(async move {
            for triple in &triples {
                if triple.from_id.trim().is_empty() || triple.to_id.trim().is_empty() {
                    return Err(DomainError::Validation(
                        "triple endpoints are required".to_string(),
                    ));
                }
            }
            let mut store = store.write().await;
            for triple in triples {
                let is_unique_feedback = matches!(
                    triple.edge,
                    OntologyEdgeKind::Vouches | OntologyEdgeKind::Challenges
                );
                if is_unique_feedback {
                    let from_id = Self::normalize_record_id(&triple.from_id, "warga");
                    let to_id = Self::normalize_record_id(&triple.to_id, "note");
                    if store.iter().any(|existing| {
                        existing.edge == triple.edge
                            && Self::normalize_record_id(&existing.from_id, "warga") == from_id
                            && Self::normalize_record_id(&existing.to_id, "note") == to_id
                    }) {
                        continue;
                    }
                }
                store.push(triple);
            }
            Ok(())
        })
    }

    fn list_note_edge_targets(
        &self,
        note_id: &str,
        edge: OntologyEdgeKind,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<String>>> {
        let note_record = Self::normalize_record_id(note_id, "note");
        let triples = self.triples.clone();
        Box::pin(async move {
            let triples = triples.read().await;
            Ok(triples
                .iter()
                .filter(|triple| {
                    triple.edge == edge
                        && Self::normalize_record_id(&triple.from_id, "note") == note_record
                })
                .map(|triple| triple.to_id.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect())
        })
    }

    fn get_concept_by_qid(
        &self,
        qid: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<OntologyConcept>>> {
        let qid = qid.to_string();
        let concepts_by_qid = self.concepts_by_qid.clone();
        let concepts_by_id = self.concepts_by_id.clone();
        Box::pin(async move {
            let concepts_by_qid = concepts_by_qid.read().await;
            let Some(concept_id) = concepts_by_qid.get(&qid) else {
                return Ok(None);
            };
            let concepts_by_id = concepts_by_id.read().await;
            Ok(concepts_by_id.get(concept_id).cloned())
        })
    }

    fn get_concepts_by_qids(
        &self,
        qids: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyConcept>>> {
        let qids = qids.to_vec();
        let concepts_by_qid = self.concepts_by_qid.clone();
        let concepts_by_id = self.concepts_by_id.clone();
        Box::pin(async move {
            let by_qid = concepts_by_qid.read().await;
            let by_id = concepts_by_id.read().await;
            Ok(qids
                .into_iter()
                .filter_map(|qid| by_qid.get(&qid).and_then(|id| by_id.get(id)).cloned())
                .collect())
        })
    }

    fn get_actions_by_types(
        &self,
        action_types: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyActionRef>>> {
        let action_types = action_types.to_vec();
        Box::pin(async move {
            let mut refs = Vec::new();
            for raw in action_types {
                let action_type = raw.trim();
                if action_type.is_empty() {
                    continue;
                }
                let (maps_to_mode, display_label) = match action_type {
                    "schema:InformAction" => ("catatan_komunitas", Some("Informasi")),
                    "schema:RepairAction" => ("komunitas", Some("Tuntaskan")),
                    "schema:CreateAction" => ("komunitas", Some("Wujudkan")),
                    "schema:SearchAction" => ("komunitas", Some("Telusuri")),
                    "schema:AchieveAction" => ("komunitas", Some("Rayakan")),
                    "schema:AssessAction" => ("komunitas", Some("Musyawarah")),
                    "schema:AlertAction" => ("siaga", Some("Siaga")),
                    _ => continue,
                };
                refs.push(OntologyActionRef {
                    action_type: action_type.to_string(),
                    maps_to_mode: maps_to_mode.to_string(),
                    display_label: display_label.map(ToString::to_string),
                });
            }
            Ok(refs)
        })
    }

    fn get_places_by_ids(
        &self,
        _place_ids: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyPlaceRef>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn list_broader_concepts(
        &self,
        concept_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyConcept>>> {
        let concept_id = Self::normalize_record_id(concept_id, "concept");
        let broader_edges = self.broader_edges.clone();
        let concepts_by_id = self.concepts_by_id.clone();
        Box::pin(async move {
            let broader_ids = broader_edges
                .read()
                .await
                .get(&concept_id)
                .cloned()
                .unwrap_or_default();
            let concepts_by_id = concepts_by_id.read().await;
            let mut rows = broader_ids
                .iter()
                .filter_map(|id| concepts_by_id.get(id).cloned())
                .collect::<Vec<_>>();
            rows.sort_by(|left, right| left.qid.cmp(&right.qid));
            Ok(rows)
        })
    }

    fn note_feedback_counts(
        &self,
        note_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<NoteFeedbackCounts>> {
        let note_record = Self::normalize_record_id(note_id, "note");
        let triples = self.triples.clone();
        Box::pin(async move {
            let triples = triples.read().await;
            let vouch_count = triples
                .iter()
                .filter(|triple| {
                    triple.edge == OntologyEdgeKind::Vouches
                        && Self::normalize_record_id(&triple.to_id, "note") == note_record
                })
                .count();
            let challenge_count = triples
                .iter()
                .filter(|triple| {
                    triple.edge == OntologyEdgeKind::Challenges
                        && Self::normalize_record_id(&triple.to_id, "note") == note_record
                })
                .count();
            Ok(NoteFeedbackCounts {
                vouch_count,
                challenge_count,
            })
        })
    }

    fn cleanup_expired_notes(
        &self,
        cutoff_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<String>>> {
        let notes = self.notes.clone();
        let triples = self.triples.clone();
        Box::pin(async move {
            if cutoff_ms < 0 {
                return Err(DomainError::Validation(
                    "cleanup cutoff_ms must be non-negative".to_string(),
                ));
            }

            let mut expired_notes = Vec::new();
            {
                let mut notes = notes.write().await;
                let mut keys = Vec::new();
                for (key, note) in notes.iter() {
                    if note.ttl_expires_ms.is_some_and(|value| value <= cutoff_ms) {
                        expired_notes.push(note.note_id.clone());
                        keys.push(key.clone());
                    }
                }
                for key in keys {
                    notes.remove(&key);
                }
            }

            if expired_notes.is_empty() {
                return Ok(Vec::new());
            }

            let mut normalized_expired = Vec::with_capacity(expired_notes.len() * 2);
            for raw in &expired_notes {
                let id = Self::id_part(raw);
                normalized_expired.push(id.clone());
                normalized_expired.push(format!("note:{id}"));
            }

            let mut triples = triples.write().await;
            let before = triples.len();
            triples.retain(|triple| {
                !normalized_expired
                    .iter()
                    .any(|candidate| triple.from_id == *candidate || triple.to_id == *candidate)
            });
            let _removed_triples = before.saturating_sub(triples.len());
            expired_notes.sort();
            expired_notes.dedup();
            Ok(expired_notes)
        })
    }
}

#[derive(Clone)]
pub struct SurrealOntologyRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealOntologyRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn normalize_record_id(raw: &str, table: &str) -> (String, String) {
        let value = raw.trim();
        if let Some((tbl, id)) = value.split_once(':') {
            (tbl.to_string(), id.to_string())
        } else {
            (table.to_string(), value.to_string())
        }
    }

    fn normalize_id_part(raw: &str) -> String {
        raw.split_once(':')
            .map(|(_, id)| id.to_string())
            .unwrap_or_else(|| raw.to_string())
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(timestamp_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
            DomainError::Validation(format!("invalid ontology datetime '{value}': {err}"))
        })?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let message = err.to_string().to_lowercase();
        if message.contains("already exists")
            || message.contains("duplicate")
            || message.contains("unique")
            || message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {message}"))
    }

    fn decode_rows<T>(rows: Vec<Value>, entity: &str) -> DomainResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value(row).map_err(|err| {
                    DomainError::Validation(format!("invalid {entity} row payload: {err}"))
                })
            })
            .collect()
    }

    fn decode_count(rows: Vec<Value>, key: &str) -> DomainResult<usize> {
        if rows.is_empty() {
            return Ok(0);
        }
        let first = rows
            .first()
            .ok_or_else(|| DomainError::Validation("missing count row".to_string()))?;
        let count = first
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| DomainError::Validation(format!("missing count '{key}'")))?;
        Ok(count as usize)
    }

    pub async fn note_ttl_expires_ms_by_note_ids(
        &self,
        note_ids: &[String],
    ) -> DomainResult<HashMap<String, i64>> {
        let normalized_note_ids = {
            let mut values: Vec<String> = note_ids
                .iter()
                .map(|value| Self::normalize_id_part(value.trim()))
                .filter(|value| !value.is_empty())
                .collect();
            values.sort();
            values.dedup();
            values
        };
        if normalized_note_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let record_ids: Vec<String> = normalized_note_ids
            .iter()
            .map(|note_id| format!("note:{note_id}"))
            .collect();
        let mut response = self
            .client
            .query(
                "SELECT note_id, type::string(id) AS record_id, <string>ttl_expires AS ttl_expires\n\
                 FROM note\n\
                 WHERE ttl_expires IS NOT NONE\n\
                 AND (note_id IN $note_ids OR type::string(id) IN $record_ids)",
            )
            .bind(("note_ids", normalized_note_ids))
            .bind(("record_ids", record_ids))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let rows: Vec<SurrealOntologyNoteTtlRow> = Self::decode_rows(rows, "note ttl")?;

        let mut ttl_by_note_id = HashMap::new();
        for row in rows {
            let Some(ttl_expires) = row.ttl_expires else {
                continue;
            };
            let ttl_expires_ms = Self::parse_datetime_ms(&ttl_expires)?;
            let key = row
                .note_id
                .as_deref()
                .map(Self::normalize_id_part)
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    row.record_id
                        .as_deref()
                        .map(Self::normalize_id_part)
                        .filter(|value| !value.is_empty())
                });
            if let Some(key) = key {
                ttl_by_note_id.insert(key, ttl_expires_ms);
            }
        }
        Ok(ttl_by_note_id)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SurrealOntologyConceptRow {
    qid: String,
    label_id: Option<String>,
    label_en: Option<String>,
    verified: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SurrealOntologyActionRow {
    action_type: String,
    maps_to_mode: String,
    display_label: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SurrealOntologyPlaceRow {
    place_id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SurrealOntologyNoteTtlRow {
    note_id: Option<String>,
    record_id: Option<String>,
    ttl_expires: Option<String>,
}

impl OntologyRepository for SurrealOntologyRepository {
    fn upsert_concept(
        &self,
        concept: &OntologyConcept,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<OntologyConcept>> {
        let client = self.client.clone();
        let concept = concept.clone();
        Box::pin(async move {
            if concept.qid.trim().is_empty() {
                return Err(DomainError::Validation("qid is required".to_string()));
            }
            let concept_id = if concept.concept_id.trim().is_empty() {
                concept.qid.clone()
            } else {
                Self::normalize_id_part(&concept.concept_id)
            };
            let mut response = client
                .query(
                    "UPDATE type::record('concept', $concept_id) MERGE {\n\
                       qid: $qid,\n\
                       label_id: $label_id,\n\
                       label_en: $label_en,\n\
                       verified: $verified,\n\
                       last_referenced: time::now()\n\
                     } RETURN AFTER",
                )
                .bind(("concept_id", concept_id.clone()))
                .bind(("qid", concept.qid.clone()))
                .bind(("label_id", concept.label_id.clone()))
                .bind(("label_en", concept.label_en.clone()))
                .bind(("verified", concept.verified))
                .await
                .map_err(Self::map_surreal_error)?;

            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows: Vec<SurrealOntologyConceptRow> = Self::decode_rows(rows, "concept")?;
            let row = rows
                .pop()
                .ok_or_else(|| DomainError::Validation("upsert returned no concept".to_string()))?;
            Ok(OntologyConcept {
                concept_id,
                qid: row.qid,
                label_id: row.label_id,
                label_en: row.label_en,
                verified: row.verified.unwrap_or(false),
            })
        })
    }

    fn add_broader_edge(
        &self,
        narrower_concept_id: &str,
        broader_concept_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let client = self.client.clone();
        let narrower_id = Self::normalize_id_part(narrower_concept_id);
        let broader_id = Self::normalize_id_part(broader_concept_id);
        Box::pin(async move {
            client
                .query(
                    "CREATE BROADER SET \
                     in = type::record('concept', $narrower_id), \
                     out = type::record('concept', $broader_id)",
                )
                .bind(("narrower_id", narrower_id))
                .bind(("broader_id", broader_id))
                .await
                .map_err(Self::map_surreal_error)?;
            Ok(())
        })
    }

    fn create_note(
        &self,
        note: &OntologyNoteCreate,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<OntologyNote>> {
        let client = self.client.clone();
        let note = note.clone();
        Box::pin(async move {
            if note.content.trim().is_empty() {
                return Err(DomainError::Validation(
                    "note content is required".to_string(),
                ));
            }
            let note_id = note
                .note_id
                .clone()
                .unwrap_or_else(gotong_domain::util::uuid_v7_without_dashes);
            let note_id = Self::normalize_id_part(&note_id);
            let content = note.content.trim().to_string();
            let author_id = note.author_id.trim().to_string();
            let community_id = note.community_id.trim().to_string();
            let temporal_class = note.temporal_class.trim().to_string();
            let created_at_ms = gotong_domain::jobs::now_ms();
            let created_at = Self::to_rfc3339(created_at_ms)?;
            let ttl_expires = note.ttl_expires_ms.map(Self::to_rfc3339).transpose()?;
            client
                .query(
                    "CREATE type::record('note', $note_id) SET\n\
                       note_id = $note_id,\n\
                       content = $content,\n\
                       author = type::record('warga', $author_id),\n\
                       community_id = $community_id,\n\
                       created_at = <datetime>$created_at,\n\
                       temporal_class = $temporal_class,\n\
                       ttl_expires = IF $ttl_expires = NONE THEN NONE ELSE <datetime>$ttl_expires END,\n\
                       ai_readable = $ai_readable,\n\
                       rahasia_level = $rahasia_level,\n\
                       confidence = $confidence RETURN AFTER",
                )
                .bind(("note_id", note_id.clone()))
                .bind(("content", content.clone()))
                .bind(("author_id", author_id.clone()))
                .bind(("community_id", community_id.clone()))
                .bind(("created_at", created_at))
                .bind(("temporal_class", temporal_class.clone()))
                .bind(("ttl_expires", ttl_expires))
                .bind(("ai_readable", note.ai_readable))
                .bind(("rahasia_level", note.rahasia_level))
                .bind(("confidence", note.confidence))
                .await
                .map_err(Self::map_surreal_error)?;

            Ok(OntologyNote {
                note_id,
                content,
                author_id,
                community_id,
                temporal_class,
                ttl_expires_ms: note.ttl_expires_ms,
                ai_readable: note.ai_readable,
                rahasia_level: note.rahasia_level,
                confidence: note.confidence,
                created_at_ms,
            })
        })
    }

    fn write_triples(
        &self,
        triples: &[OntologyTripleCreate],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let client = self.client.clone();
        let triples = triples.to_vec();
        Box::pin(async move {
            for triple in &triples {
                let (from_table, from_id) = Self::normalize_record_id(&triple.from_id, "note");
                let (to_table, to_id) = Self::normalize_record_id(&triple.to_id, "concept");
                let statement = format!(
                    "CREATE {} SET \
                     in = type::record('{from_table}', $from_id), \
                     out = type::record('{to_table}', $to_id), \
                     predicate = IF $predicate = NULL THEN NONE ELSE $predicate END, \
                     metadata = IF $metadata = NULL THEN NONE ELSE $metadata END",
                    triple.edge.as_table_name()
                );
                client
                    .query(statement)
                    .bind(("from_id", from_id))
                    .bind(("to_id", to_id))
                    .bind(("predicate", triple.predicate.clone()))
                    .bind(("metadata", triple.metadata.clone()))
                    .await
                    .map_err(Self::map_surreal_error)?;
            }
            Ok(())
        })
    }

    fn list_note_edge_targets(
        &self,
        note_id: &str,
        edge: OntologyEdgeKind,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<String>>> {
        let client = self.client.clone();
        let note_id = Self::normalize_id_part(note_id);
        let edge_table = edge.as_table_name().to_string();
        Box::pin(async move {
            let mut response = client
                .query(format!(
                    "SELECT VALUE out FROM {edge_table} WHERE in = type::record('note', $note_id)",
                ))
                .bind(("note_id", note_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(rows
                .into_iter()
                .filter_map(|value| {
                    value.as_str().map(ToString::to_string).or_else(|| {
                        value
                            .get("id")
                            .and_then(Value::as_str)
                            .map(ToString::to_string)
                    })
                })
                .collect())
        })
    }

    fn get_concept_by_qid(
        &self,
        qid: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<OntologyConcept>>> {
        let client = self.client.clone();
        let qid = qid.to_string();
        Box::pin(async move {
            let mut response = client
                .query("SELECT qid, label_id, label_en, verified FROM concept WHERE qid = $qid LIMIT 1")
                .bind(("qid", qid.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows: Vec<SurrealOntologyConceptRow> = Self::decode_rows(rows, "concept")?;
            Ok(rows.into_iter().next().map(|row| OntologyConcept {
                concept_id: row.qid.clone(),
                qid: row.qid,
                label_id: row.label_id,
                label_en: row.label_en,
                verified: row.verified.unwrap_or(false),
            }))
        })
    }

    fn get_concepts_by_qids(
        &self,
        qids: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyConcept>>> {
        let client = self.client.clone();
        let qids = qids.to_vec();
        Box::pin(async move {
            if qids.is_empty() {
                return Ok(Vec::new());
            }
            let mut response = client
                .query("SELECT qid, label_id, label_en, verified FROM concept WHERE qid IN $qids")
                .bind(("qids", qids))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows: Vec<SurrealOntologyConceptRow> = Self::decode_rows(rows, "concept")?;
            Ok(rows
                .into_iter()
                .map(|row| OntologyConcept {
                    concept_id: row.qid.clone(),
                    qid: row.qid,
                    label_id: row.label_id,
                    label_en: row.label_en,
                    verified: row.verified.unwrap_or(false),
                })
                .collect())
        })
    }

    fn get_actions_by_types(
        &self,
        action_types: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyActionRef>>> {
        let client = self.client.clone();
        let action_types = action_types.to_vec();
        Box::pin(async move {
            if action_types.is_empty() {
                return Ok(Vec::new());
            }
            let mut response = client
                .query(
                    "SELECT action_type, maps_to_mode, display_label FROM action WHERE action_type IN $action_types",
                )
                .bind(("action_types", action_types))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows: Vec<SurrealOntologyActionRow> = Self::decode_rows(rows, "action")?;
            Ok(rows
                .into_iter()
                .map(|row| OntologyActionRef {
                    action_type: row.action_type,
                    maps_to_mode: row.maps_to_mode,
                    display_label: row.display_label,
                })
                .collect())
        })
    }

    fn get_places_by_ids(
        &self,
        place_ids: &[String],
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyPlaceRef>>> {
        let client = self.client.clone();
        let place_ids = place_ids.to_vec();
        Box::pin(async move {
            let place_ids: Vec<String> = place_ids
                .into_iter()
                .map(|id| id.trim().to_string())
                .filter(|id| id.starts_with("place:"))
                .collect();
            if place_ids.is_empty() {
                return Ok(Vec::new());
            }
            let mut response = client
                .query(
                    "SELECT type::string(id) AS place_id, name FROM place WHERE id IN $place_ids",
                )
                .bind(("place_ids", place_ids))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows: Vec<SurrealOntologyPlaceRow> = Self::decode_rows(rows, "place")?;
            Ok(rows
                .into_iter()
                .map(|row| OntologyPlaceRef {
                    place_id: row.place_id,
                    name: row.name,
                })
                .collect())
        })
    }

    fn list_broader_concepts(
        &self,
        concept_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<OntologyConcept>>> {
        let client = self.client.clone();
        let concept_id = Self::normalize_id_part(concept_id);
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT out.qid AS qid, out.label_id AS label_id, out.label_en AS label_en, out.verified AS verified \
                     FROM BROADER WHERE in = type::record('concept', $concept_id)",
                )
                .bind(("concept_id", concept_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows: Vec<SurrealOntologyConceptRow> = Self::decode_rows(rows, "concept")?;
            Ok(rows
                .into_iter()
                .map(|row| OntologyConcept {
                    concept_id: row.qid.clone(),
                    qid: row.qid,
                    label_id: row.label_id,
                    label_en: row.label_en,
                    verified: row.verified.unwrap_or(false),
                })
                .collect())
        })
    }

    fn note_feedback_counts(
        &self,
        note_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<NoteFeedbackCounts>> {
        let client = self.client.clone();
        let note_id = Self::normalize_id_part(note_id);
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT count() AS vouch_count FROM VOUCHES WHERE out = type::record('note', $note_id) GROUP ALL;\n\
                     SELECT count() AS challenge_count FROM CHALLENGES WHERE out = type::record('note', $note_id) GROUP ALL;",
                )
                .bind(("note_id", note_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let vouch_rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let challenge_rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(NoteFeedbackCounts {
                vouch_count: Self::decode_count(vouch_rows, "vouch_count")?,
                challenge_count: Self::decode_count(challenge_rows, "challenge_count")?,
            })
        })
    }

    fn cleanup_expired_notes(
        &self,
        cutoff_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<String>>> {
        let client = self.client.clone();
        Box::pin(async move {
            if cutoff_ms < 0 {
                return Err(DomainError::Validation(
                    "cleanup cutoff_ms must be non-negative".to_string(),
                ));
            }

            let cutoff = Self::to_rfc3339(cutoff_ms)?;
            let expired_rows: Vec<Value> = client
                .query(
                    "SELECT VALUE id FROM note WHERE ttl_expires IS NOT NONE AND ttl_expires < $cutoff",
                )
                .bind(("cutoff", cutoff))
                .await
                .map_err(Self::map_surreal_error)?
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let expired_note_ids: Vec<String> = expired_rows
                .into_iter()
                .filter_map(|value| {
                    value.as_str().map(ToString::to_string).or_else(|| {
                        value
                            .get("id")
                            .and_then(Value::as_str)
                            .map(ToString::to_string)
                    })
                })
                .collect();
            if expired_note_ids.is_empty() {
                return Ok(Vec::new());
            }

            for raw_note_id in &expired_note_ids {
                let note_id = Self::normalize_id_part(raw_note_id);
                let _deleted_note_rows: Vec<Value> = client
                    .query("DELETE note WHERE id = type::record('note', $note_id)")
                    .bind(("note_id", note_id.clone()))
                    .await
                    .map_err(Self::map_surreal_error)?
                    .take(0)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid query result: {err}"))
                    })?;

                for edge in [
                    "ABOUT",
                    "LOCATED_AT",
                    "HAS_ACTION",
                    "INSTANCE_OF",
                    "VOUCHES",
                    "CHALLENGES",
                ] {
                    client
                        .query(format!(
                            "DELETE {edge} WHERE in = type::record('note', $note_id) OR out = type::record('note', $note_id)"
                        ))
                        .bind(("note_id", note_id.clone()))
                        .await
                        .map_err(Self::map_surreal_error)?
                        .take::<Vec<Value>>(0)
                        .map_err(|err| {
                            DomainError::Validation(format!("invalid query result: {err}"))
                        })?;
                }

                let _deleted_measurement_rows: Vec<Value> = client
                    .query("DELETE measurement WHERE note = type::record('note', $note_id)")
                    .bind(("note_id", note_id))
                    .await
                    .map_err(Self::map_surreal_error)?
                    .take(0)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid query result: {err}"))
                    })?;
            }

            let mut normalized_expired_note_ids = expired_note_ids
                .into_iter()
                .map(|raw_note_id| Self::normalize_id_part(&raw_note_id))
                .collect::<Vec<_>>();
            normalized_expired_note_ids.sort();
            normalized_expired_note_ids.dedup();
            Ok(normalized_expired_note_ids)
        })
    }
}

#[cfg(test)]
mod ontology_repository_tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_ontology_repository_supports_core_flows() {
        let repo = InMemoryOntologyRepository::new();

        let concept_a = repo
            .upsert_concept(&OntologyConcept {
                concept_id: "Q93189".to_string(),
                qid: "Q93189".to_string(),
                label_id: Some("Telur".to_string()),
                label_en: Some("Egg".to_string()),
                verified: true,
            })
            .await
            .expect("upsert concept A");
        let _concept_b = repo
            .upsert_concept(&OntologyConcept {
                concept_id: "Q2095".to_string(),
                qid: "Q2095".to_string(),
                label_id: Some("Makanan".to_string()),
                label_en: Some("Food".to_string()),
                verified: true,
            })
            .await
            .expect("upsert concept B");

        repo.add_broader_edge(&concept_a.concept_id, "Q2095")
            .await
            .expect("add broader");

        let note = repo
            .create_note(&OntologyNoteCreate {
                note_id: Some("note-1".to_string()),
                content: "Telur Rp 28k".to_string(),
                author_id: "damar".to_string(),
                community_id: "rt05".to_string(),
                temporal_class: "ephemeral".to_string(),
                ttl_expires_ms: None,
                ai_readable: true,
                rahasia_level: 0,
                confidence: 0.92,
            })
            .await
            .expect("create note");

        repo.write_triples(&[
            OntologyTripleCreate {
                edge: OntologyEdgeKind::About,
                from_id: format!("note:{}", note.note_id),
                to_id: "concept:Q93189".to_string(),
                predicate: Some("schema:price".to_string()),
                metadata: None,
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Vouches,
                from_id: "warga:u1".to_string(),
                to_id: format!("note:{}", note.note_id),
                predicate: None,
                metadata: Some(serde_json::json!({"reason": "valid"})),
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Challenges,
                from_id: "warga:u2".to_string(),
                to_id: format!("note:{}", note.note_id),
                predicate: None,
                metadata: Some(serde_json::json!({"reason": "unsure"})),
            },
        ])
        .await
        .expect("write triples");

        let by_qid = repo
            .get_concept_by_qid("Q93189")
            .await
            .expect("get concept")
            .expect("concept exists");
        assert_eq!(by_qid.qid, "Q93189");

        let broader = repo
            .list_broader_concepts("Q93189")
            .await
            .expect("list broader");
        assert_eq!(broader.len(), 1);
        assert_eq!(broader[0].qid, "Q2095");

        let counts = repo
            .note_feedback_counts(&note.note_id)
            .await
            .expect("feedback counts");
        assert_eq!(counts.vouch_count, 1);
        assert_eq!(counts.challenge_count, 1);
    }

    #[tokio::test]
    async fn in_memory_ontology_repository_cleanup_expired_notes_removes_notes_and_edges() {
        let repo = InMemoryOntologyRepository::new();
        let now = gotong_domain::jobs::now_ms();

        let concept = repo
            .upsert_concept(&OntologyConcept {
                concept_id: "Q93189".to_string(),
                qid: "Q93189".to_string(),
                label_id: Some("Telur".to_string()),
                label_en: Some("Egg".to_string()),
                verified: true,
            })
            .await
            .expect("upsert concept");

        let expired_note = repo
            .create_note(&OntologyNoteCreate {
                note_id: Some("expired-note".to_string()),
                content: "expired".to_string(),
                author_id: "warga-expired".to_string(),
                community_id: "rt05".to_string(),
                temporal_class: "ephemeral".to_string(),
                ttl_expires_ms: Some(now - 1),
                ai_readable: true,
                rahasia_level: 0,
                confidence: 0.82,
            })
            .await
            .expect("create expired note");

        let active_note = repo
            .create_note(&OntologyNoteCreate {
                note_id: Some("active-note".to_string()),
                content: "active".to_string(),
                author_id: "warga-active".to_string(),
                community_id: "rt05".to_string(),
                temporal_class: "ephemeral".to_string(),
                ttl_expires_ms: Some(now + 60_000),
                ai_readable: true,
                rahasia_level: 0,
                confidence: 0.85,
            })
            .await
            .expect("create active note");

        repo.write_triples(&[
            OntologyTripleCreate {
                edge: OntologyEdgeKind::About,
                from_id: format!("note:{}", expired_note.note_id),
                to_id: format!("concept:{}", concept.qid),
                predicate: None,
                metadata: None,
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::About,
                from_id: format!("note:{}", active_note.note_id),
                to_id: format!("concept:{}", concept.qid),
                predicate: None,
                metadata: None,
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Vouches,
                from_id: "warga:alpha".to_string(),
                to_id: format!("note:{}", expired_note.note_id),
                predicate: None,
                metadata: None,
            },
            OntologyTripleCreate {
                edge: OntologyEdgeKind::Challenges,
                from_id: "warga:beta".to_string(),
                to_id: format!("note:{}", active_note.note_id),
                predicate: None,
                metadata: None,
            },
        ])
        .await
        .expect("write triples");

        let removed_note_ids = repo
            .cleanup_expired_notes(now)
            .await
            .expect("cleanup expired notes");
        assert_eq!(removed_note_ids, vec!["expired-note".to_string()]);

        let expired_feedback = repo
            .note_feedback_counts(&expired_note.note_id)
            .await
            .expect("expired feedback");
        assert_eq!(expired_feedback.vouch_count, 0);
        assert_eq!(expired_feedback.challenge_count, 0);

        let active_feedback = repo
            .note_feedback_counts(&active_note.note_id)
            .await
            .expect("active feedback");
        assert_eq!(active_feedback.vouch_count, 0);
        assert_eq!(active_feedback.challenge_count, 1);
    }
}

#[derive(Default)]
pub struct InMemoryEvidenceRepository {
    store: Arc<RwLock<HashMap<String, Evidence>>>,
}

impl InMemoryEvidenceRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EvidenceRepository for InMemoryEvidenceRepository {
    fn create(
        &self,
        evidence: &Evidence,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Evidence>> {
        let evidence = evidence.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&evidence.evidence_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(evidence.evidence_id.clone(), evidence.clone());
            Ok(evidence)
        })
    }

    fn get(
        &self,
        evidence_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<Evidence>>> {
        let evidence_id = evidence_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            Ok(items.get(&evidence_id).cloned())
        })
    }

    fn list_by_contribution(
        &self,
        contribution_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Evidence>>> {
        let contribution_id = contribution_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let items = store.read().await;
            let mut evidence: Vec<_> = items
                .values()
                .filter(|item| item.contribution_id == contribution_id)
                .cloned()
                .collect();
            evidence.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.evidence_id.cmp(&a.evidence_id))
            });
            Ok(evidence)
        })
    }
}

#[derive(Default)]
pub struct InMemoryVouchRepository {
    store: Arc<RwLock<HashMap<String, Vouch>>>,
}

impl InMemoryVouchRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VouchRepository for InMemoryVouchRepository {
    fn create(&self, vouch: &Vouch) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vouch>> {
        let vouch = vouch.clone();
        let store = self.store.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            if items.contains_key(&vouch.vouch_id) {
                return Err(DomainError::Conflict);
            }
            items.insert(vouch.vouch_id.clone(), vouch.clone());
            Ok(vouch)
        })
    }

    fn list_by_vouchee(
        &self,
        vouchee_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let vouchee_id = vouchee_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut vouches = store
                .read()
                .await
                .values()
                .filter(|item| item.vouchee_id == vouchee_id)
                .cloned()
                .collect::<Vec<_>>();
            vouches.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.vouch_id.cmp(&a.vouch_id))
            });
            Ok(vouches)
        })
    }

    fn list_by_voucher(
        &self,
        voucher_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<Vouch>>> {
        let voucher_id = voucher_id.to_string();
        let store = self.store.clone();
        Box::pin(async move {
            let mut vouches = store
                .read()
                .await
                .values()
                .filter(|item| item.voucher_id == voucher_id)
                .cloned()
                .collect::<Vec<_>>();
            vouches.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.vouch_id.cmp(&a.vouch_id))
            });
            Ok(vouches)
        })
    }
}

#[derive(Default)]
struct InMemoryAdaptivePathState {
    plans: HashMap<String, AdaptivePathPlan>,
    plan_by_entity: HashMap<String, String>,
    plan_by_entity_request: HashMap<(String, String), String>,
    events_by_plan: HashMap<String, Vec<AdaptivePathEvent>>,
    event_by_request: HashMap<String, (String, String)>,
    suggestions: HashMap<String, AdaptivePathSuggestion>,
    suggestion_by_plan_request: HashMap<(String, String), String>,
}

#[derive(Default)]
pub struct InMemoryAdaptivePathRepository {
    state: Arc<RwLock<InMemoryAdaptivePathState>>,
}

impl InMemoryAdaptivePathRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn plan_request_key(entity_id: &str, request_id: &str) -> (String, String) {
        (entity_id.to_string(), request_id.to_string())
    }

    fn suggestion_request_key(plan_id: &str, request_id: &str) -> (String, String) {
        (plan_id.to_string(), request_id.to_string())
    }
}

impl AdaptivePathRepository for InMemoryAdaptivePathRepository {
    fn create_plan(
        &self,
        plan: &AdaptivePathPlan,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
        let plan = plan.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.write().await;
            let request_key = Self::plan_request_key(&plan.entity_id, &plan.request_id);
            if state.plan_by_entity_request.contains_key(&request_key) {
                return Err(DomainError::Conflict);
            }

            if state.plans.contains_key(&plan.plan_id) {
                return Err(DomainError::Conflict);
            }

            if state.plan_by_entity.contains_key(&plan.entity_id) {
                return Err(DomainError::Conflict);
            }
            state
                .plan_by_entity
                .insert(plan.entity_id.clone(), plan.plan_id.clone());
            state
                .plan_by_entity_request
                .insert(request_key, plan.plan_id.clone());
            state.plans.insert(plan.plan_id.clone(), plan.clone());
            Ok(plan)
        })
    }

    fn get_plan(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let plan_id = plan_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            Ok(state.plans.get(&plan_id).cloned())
        })
    }

    fn get_plan_by_entity(
        &self,
        entity_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let entity_id = entity_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let Some(plan_id) = state.plan_by_entity.get(&entity_id) else {
                return Ok(None);
            };
            Ok(state.plans.get(plan_id).cloned())
        })
    }

    fn get_plan_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let key = Self::plan_request_key(entity_id, request_id);
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let Some(plan_id) = state.plan_by_entity_request.get(&key) else {
                return Ok(None);
            };
            Ok(state.plans.get(plan_id).cloned())
        })
    }

    fn update_plan(
        &self,
        plan: &AdaptivePathPlan,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
        let plan = plan.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.write().await;
            let request_key = Self::plan_request_key(&plan.entity_id, &plan.request_id);
            if let Some(existing_plan_id) = state.plan_by_entity_request.get(&request_key) {
                if let Some(existing) = state.plans.get(existing_plan_id) {
                    return Ok(existing.clone());
                }
                return Err(DomainError::Conflict);
            }

            let Some(current) = state.plans.get(&plan.plan_id) else {
                return Err(DomainError::NotFound);
            };
            if plan.version <= current.version {
                return Err(DomainError::Conflict);
            }
            state.plans.insert(plan.plan_id.clone(), plan.clone());
            state
                .plan_by_entity_request
                .insert(request_key, plan.plan_id.clone());
            Ok(plan)
        })
    }

    fn create_event(
        &self,
        event: &AdaptivePathEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathEvent>> {
        let event = event.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.write().await;
            if state.event_by_request.contains_key(&event.request_id) {
                return Err(DomainError::Conflict);
            }

            let rows = state
                .events_by_plan
                .entry(event.plan_id.clone())
                .or_default();
            if rows.iter().any(|row| row.event_id == event.event_id) {
                return Err(DomainError::Conflict);
            }
            rows.push(event.clone());
            state.event_by_request.insert(
                event.request_id.clone(),
                (event.plan_id.clone(), event.event_id.clone()),
            );
            Ok(event)
        })
    }

    fn list_events(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathEvent>>> {
        let plan_id = plan_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let mut rows = state
                .events_by_plan
                .get(&plan_id)
                .cloned()
                .unwrap_or_default();
            rows.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.event_id.cmp(&right.event_id))
            });
            Ok(rows)
        })
    }

    fn get_event_by_request_id(
        &self,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathEvent>>> {
        let request_id = request_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let Some((plan_id, event_id)) = state.event_by_request.get(&request_id) else {
                return Ok(None);
            };
            let Some(events) = state.events_by_plan.get(plan_id) else {
                return Ok(None);
            };
            Ok(events
                .iter()
                .find(|event| event.event_id == *event_id)
                .cloned())
        })
    }

    fn create_suggestion(
        &self,
        suggestion: &AdaptivePathSuggestion,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
        let suggestion = suggestion.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.write().await;
            let request_key =
                Self::suggestion_request_key(&suggestion.plan_id, &suggestion.request_id);
            if state.suggestion_by_plan_request.contains_key(&request_key) {
                return Err(DomainError::Conflict);
            }
            if state.suggestions.contains_key(&suggestion.suggestion_id) {
                return Err(DomainError::Conflict);
            }
            state
                .suggestions
                .insert(suggestion.suggestion_id.clone(), suggestion.clone());
            state
                .suggestion_by_plan_request
                .insert(request_key, suggestion.suggestion_id.clone());
            Ok(suggestion)
        })
    }

    fn list_suggestions(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathSuggestion>>> {
        let plan_id = plan_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let mut rows: Vec<_> = state
                .suggestions
                .values()
                .filter(|suggestion| suggestion.plan_id == plan_id)
                .cloned()
                .collect();
            rows.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.suggestion_id.cmp(&left.suggestion_id))
            });
            Ok(rows)
        })
    }

    fn get_suggestion(
        &self,
        suggestion_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
        let suggestion_id = suggestion_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            Ok(state.suggestions.get(&suggestion_id).cloned())
        })
    }

    fn get_suggestion_by_request_id(
        &self,
        plan_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
        let key = Self::suggestion_request_key(plan_id, request_id);
        let state = self.state.clone();
        Box::pin(async move {
            let state = state.read().await;
            let Some(suggestion_id) = state.suggestion_by_plan_request.get(&key) else {
                return Ok(None);
            };
            Ok(state.suggestions.get(suggestion_id).cloned())
        })
    }

    fn update_suggestion_status(
        &self,
        suggestion_id: &str,
        status: SuggestionDecisionStatus,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
        let suggestion_id = suggestion_id.to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.write().await;
            let suggestion = state
                .suggestions
                .get_mut(&suggestion_id)
                .ok_or(DomainError::NotFound)?;
            suggestion.status = status;
            suggestion.updated_at_ms =
                (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
            Ok(suggestion.clone())
        })
    }
}

#[derive(Clone)]
pub struct SurrealAdaptivePathRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealAdaptivePathRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let message = err.to_string().to_lowercase();
        if message.contains("already exists")
            || message.contains("duplicate")
            || message.contains("unique")
            || message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {message}"))
    }

    fn decode_one<T>(rows: Vec<Value>, context: &str) -> DomainResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let decoded = serde_json::from_value::<T>(row)
            .map_err(|err| DomainError::Validation(format!("invalid {context} row: {err}")))?;
        Ok(Some(decoded))
    }

    fn decode_many<T>(rows: Vec<Value>, context: &str) -> DomainResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<T>(row)
                    .map_err(|err| DomainError::Validation(format!("invalid {context} row: {err}")))
            })
            .collect()
    }
}

impl AdaptivePathRepository for SurrealAdaptivePathRepository {
    fn create_plan(
        &self,
        plan: &AdaptivePathPlan,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
        let client = self.client.clone();
        let plan = plan.clone();
        Box::pin(async move {
            let payload = to_value(&plan)
                .map_err(|err| DomainError::Validation(format!("invalid plan payload: {err}")))?;
            let mut response = client
                .query("CREATE path_plan CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan")?
                .ok_or_else(|| DomainError::Validation("create returned no plan row".to_string()))
        })
    }

    fn get_plan(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let client = self.client.clone();
        let plan_id = plan_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM path_plan WHERE plan_id = $plan_id LIMIT 1")
                .bind(("plan_id", plan_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan")
        })
    }

    fn get_plan_by_entity(
        &self,
        entity_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let client = self.client.clone();
        let entity_id = entity_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM path_plan \
                     WHERE entity_id = $entity_id \
                     ORDER BY version DESC, updated_at_ms DESC, plan_id DESC \
                     LIMIT 1",
                )
                .bind(("entity_id", entity_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan")
        })
    }

    fn get_plan_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
        let client = self.client.clone();
        let entity_id = entity_id.to_string();
        let request_id = request_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM path_plan \
                     WHERE entity_id = $entity_id AND request_id = $request_id \
                     LIMIT 1",
                )
                .bind(("entity_id", entity_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan")
        })
    }

    fn update_plan(
        &self,
        plan: &AdaptivePathPlan,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
        let client = self.client.clone();
        let plan = plan.clone();
        Box::pin(async move {
            let expected_version = plan.version.saturating_sub(1);
            let payload = to_value(&plan)
                .map_err(|err| DomainError::Validation(format!("invalid plan payload: {err}")))?;
            let mut response = client
                .query(
                    "UPDATE path_plan MERGE $payload \
                     WHERE plan_id = $plan_id AND version = $expected_version \
                     RETURN AFTER",
                )
                .bind(("payload", payload))
                .bind(("plan_id", plan.plan_id.clone()))
                .bind(("expected_version", expected_version))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan")?.ok_or(DomainError::Conflict)
        })
    }

    fn create_event(
        &self,
        event: &AdaptivePathEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathEvent>> {
        let client = self.client.clone();
        let event = event.clone();
        Box::pin(async move {
            let payload = to_value(&event)
                .map_err(|err| DomainError::Validation(format!("invalid event payload: {err}")))?;
            let mut response = client
                .query("CREATE path_plan_event CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan_event")?
                .ok_or_else(|| DomainError::Validation("create returned no event row".to_string()))
        })
    }

    fn list_events(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathEvent>>> {
        let client = self.client.clone();
        let plan_id = plan_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM path_plan_event \
                     WHERE plan_id = $plan_id \
                     ORDER BY occurred_at_ms ASC, event_id ASC",
                )
                .bind(("plan_id", plan_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_many(rows, "path_plan_event")
        })
    }

    fn get_event_by_request_id(
        &self,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathEvent>>> {
        let client = self.client.clone();
        let request_id = request_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM path_plan_event \
                     WHERE request_id = $request_id LIMIT 1",
                )
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "path_plan_event")
        })
    }

    fn create_suggestion(
        &self,
        suggestion: &AdaptivePathSuggestion,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
        let client = self.client.clone();
        let suggestion = suggestion.clone();
        Box::pin(async move {
            let payload = to_value(&suggestion).map_err(|err| {
                DomainError::Validation(format!("invalid suggestion payload: {err}"))
            })?;
            let mut response = client
                .query("CREATE plan_suggestion CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "plan_suggestion")?.ok_or_else(|| {
                DomainError::Validation("create returned no suggestion row".to_string())
            })
        })
    }

    fn list_suggestions(
        &self,
        plan_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathSuggestion>>> {
        let client = self.client.clone();
        let plan_id = plan_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM plan_suggestion \
                     WHERE plan_id = $plan_id \
                     ORDER BY created_at_ms DESC, suggestion_id DESC",
                )
                .bind(("plan_id", plan_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_many(rows, "plan_suggestion")
        })
    }

    fn get_suggestion(
        &self,
        suggestion_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
        let client = self.client.clone();
        let suggestion_id = suggestion_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM plan_suggestion \
                     WHERE suggestion_id = $suggestion_id LIMIT 1",
                )
                .bind(("suggestion_id", suggestion_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "plan_suggestion")
        })
    }

    fn get_suggestion_by_request_id(
        &self,
        plan_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
        let client = self.client.clone();
        let plan_id = plan_id.to_string();
        let request_id = request_id.to_string();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM plan_suggestion \
                     WHERE plan_id = $plan_id AND request_id = $request_id \
                     LIMIT 1",
                )
                .bind(("plan_id", plan_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "plan_suggestion")
        })
    }

    fn update_suggestion_status(
        &self,
        suggestion_id: &str,
        status: SuggestionDecisionStatus,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
        let client = self.client.clone();
        let suggestion_id = suggestion_id.to_string();
        Box::pin(async move {
            let status = to_value(&status)
                .map_err(|err| DomainError::Validation(format!("invalid status value: {err}")))?;
            let updated_at_ms =
                (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
            let mut response = client
                .query(
                    "UPDATE plan_suggestion \
                     SET status = $status, updated_at_ms = $updated_at_ms \
                     WHERE suggestion_id = $suggestion_id \
                     RETURN AFTER",
                )
                .bind(("status", status))
                .bind(("updated_at_ms", updated_at_ms))
                .bind(("suggestion_id", suggestion_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_one(rows, "plan_suggestion")?.ok_or(DomainError::NotFound)
        })
    }
}

#[derive(Default)]
pub struct InMemoryVaultRepository {
    entries: Arc<RwLock<HashMap<String, VaultEntry>>>,
    by_actor_request: Arc<RwLock<HashMap<(String, String), String>>>,
    by_entry_request: Arc<RwLock<HashMap<(String, String), String>>>,
    timeline: Arc<RwLock<HashMap<String, Vec<VaultTimelineEvent>>>>,
}

impl InMemoryVaultRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn actor_request_key(actor_id: &str, request_id: &str) -> (String, String) {
        (actor_id.to_string(), request_id.to_string())
    }

    fn entry_request_key(vault_entry_id: &str, request_id: &str) -> (String, String) {
        (vault_entry_id.to_string(), request_id.to_string())
    }
}

impl VaultRepository for InMemoryVaultRepository {
    fn create_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<VaultEntry>> {
        let entry = entry.clone();
        let event = event.clone();
        let entries = self.entries.clone();
        let by_actor_request = self.by_actor_request.clone();
        let by_entry_request = self.by_entry_request.clone();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let actor_key = Self::actor_request_key(&entry.author_id, &event.request_id);
            if by_actor_request.read().await.contains_key(&actor_key) {
                return Err(DomainError::Conflict);
            }
            let mut entries = entries.write().await;
            if entries.contains_key(&entry.vault_entry_id) {
                return Err(DomainError::Conflict);
            }
            entries.insert(entry.vault_entry_id.clone(), entry.clone());
            by_actor_request
                .write()
                .await
                .insert(actor_key, entry.vault_entry_id.clone());
            by_entry_request.write().await.insert(
                Self::entry_request_key(&entry.vault_entry_id, &event.request_id),
                entry.vault_entry_id.clone(),
            );
            timeline
                .write()
                .await
                .entry(entry.vault_entry_id.clone())
                .or_default()
                .push(event);
            Ok(entry)
        })
    }

    fn update_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<VaultEntry>> {
        let entry = entry.clone();
        let event = event.clone();
        let entries = self.entries.clone();
        let by_entry_request = self.by_entry_request.clone();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let entry_request_key =
                Self::entry_request_key(&entry.vault_entry_id, &event.request_id);
            if by_entry_request
                .read()
                .await
                .contains_key(&entry_request_key)
            {
                let entries = entries.read().await;
                return entries
                    .get(&entry.vault_entry_id)
                    .cloned()
                    .ok_or(DomainError::Conflict);
            }
            let mut entries = entries.write().await;
            if !entries.contains_key(&entry.vault_entry_id) {
                return Err(DomainError::NotFound);
            }
            entries.insert(entry.vault_entry_id.clone(), entry.clone());
            by_entry_request
                .write()
                .await
                .insert(entry_request_key, entry.vault_entry_id.clone());
            timeline
                .write()
                .await
                .entry(entry.vault_entry_id.clone())
                .or_default()
                .push(event);
            Ok(entry)
        })
    }

    fn delete_entry(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<bool>> {
        let vault_entry_id = vault_entry_id.to_string();
        let entries = self.entries.clone();
        let by_actor_request = self.by_actor_request.clone();
        let by_entry_request = self.by_entry_request.clone();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let removed = entries.write().await.remove(&vault_entry_id).is_some();
            if !removed {
                return Ok(false);
            }
            by_actor_request
                .write()
                .await
                .retain(|_, existing_entry_id| existing_entry_id != &vault_entry_id);
            by_entry_request
                .write()
                .await
                .retain(|_, existing_entry_id| existing_entry_id != &vault_entry_id);
            timeline.write().await.remove(&vault_entry_id);
            Ok(true)
        })
    }

    fn get_entry(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let entries = self.entries.clone();
        Box::pin(async move {
            let entries = entries.read().await;
            Ok(entries.get(&vault_entry_id).cloned())
        })
    }

    fn list_by_author(
        &self,
        author_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<VaultEntry>>> {
        let author_id = author_id.to_string();
        let entries = self.entries.clone();
        Box::pin(async move {
            let mut by_author: Vec<VaultEntry> = entries
                .read()
                .await
                .values()
                .filter(|entry| entry.author_id == author_id)
                .cloned()
                .collect();
            by_author.sort_by(|a, b| {
                b.created_at_ms
                    .cmp(&a.created_at_ms)
                    .then_with(|| b.vault_entry_id.cmp(&a.vault_entry_id))
            });
            Ok(by_author)
        })
    }

    fn list_timeline(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<VaultTimelineEvent>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let mut events = timeline
                .read()
                .await
                .get(&vault_entry_id)
                .cloned()
                .unwrap_or_default();
            events.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.event_id.cmp(&right.event_id))
            });
            Ok(events)
        })
    }

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let actor_id = actor_id.to_string();
        let request_id = request_id.to_string();
        let by_actor_request = self.by_actor_request.clone();
        let entries = self.entries.clone();
        Box::pin(async move {
            let key = Self::actor_request_key(&actor_id, &request_id);
            let by_actor_request = by_actor_request.read().await;
            let Some(vault_entry_id) = by_actor_request.get(&key) else {
                return Ok(None);
            };
            Ok(entries.read().await.get(vault_entry_id).cloned())
        })
    }

    fn get_by_request(
        &self,
        vault_entry_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let request_id = request_id.to_string();
        let by_entry_request = self.by_entry_request.clone();
        let entries = self.entries.clone();
        Box::pin(async move {
            let key = Self::entry_request_key(&vault_entry_id, &request_id);
            let by_entry_request = by_entry_request.read().await;
            let Some(vault_entry_id) = by_entry_request.get(&key) else {
                return Ok(None);
            };
            Ok(entries.read().await.get(vault_entry_id).cloned())
        })
    }
}

type FeedSourceRequestKey = (String, String, String);
type FeedParticipantEdgeKey = (String, String);

fn feed_participant_actor_ids(item: &FeedItem) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut actor_ids = Vec::new();
    for actor_id in std::iter::once(item.actor_id.as_str())
        .chain(item.participant_ids.iter().map(String::as_str))
    {
        let actor_id = actor_id.trim();
        if actor_id.is_empty() {
            continue;
        }
        if seen.insert(actor_id.to_string()) {
            actor_ids.push(actor_id.to_string());
        }
    }
    actor_ids
}

fn deep_merge_json(base: &mut Value, patch: Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, patch_value) in patch_map {
                match base_map.get_mut(&key) {
                    Some(existing) => deep_merge_json(existing, patch_value),
                    None => {
                        base_map.insert(key, patch_value);
                    }
                }
            }
        }
        (base_slot, patch_value) => {
            *base_slot = patch_value;
        }
    }
}

#[derive(Default)]
pub struct InMemoryDiscoveryFeedRepository {
    by_id: Arc<RwLock<HashMap<String, FeedItem>>>,
    by_source_request: Arc<RwLock<HashMap<FeedSourceRequestKey, String>>>,
    by_participant_edge: Arc<RwLock<HashSet<FeedParticipantEdgeKey>>>,
}

impl InMemoryDiscoveryFeedRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FeedRepository for InMemoryDiscoveryFeedRepository {
    fn create_feed_item(
        &self,
        item: &FeedItem,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<FeedItem>> {
        let item = item.clone();
        let by_id = self.by_id.clone();
        let by_source_request = self.by_source_request.clone();
        Box::pin(async move {
            let mut by_id = by_id.write().await;
            if by_id.contains_key(&item.feed_id) {
                return Err(DomainError::Conflict);
            }
            let source_key: FeedSourceRequestKey = (
                item.source_type.clone(),
                item.source_id.clone(),
                item.request_id.clone(),
            );
            let mut by_source_request = by_source_request.write().await;
            if by_source_request.contains_key(&source_key) {
                return Err(DomainError::Conflict);
            }
            by_source_request.insert(source_key, item.feed_id.clone());
            by_id.insert(item.feed_id.clone(), item.clone());
            Ok(item)
        })
    }

    fn upsert_participant_edges_for_item(
        &self,
        item: &FeedItem,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let actor_ids = feed_participant_actor_ids(item);
        let feed_id = item.feed_id.clone();
        let by_participant_edge = self.by_participant_edge.clone();
        Box::pin(async move {
            let mut by_participant_edge = by_participant_edge.write().await;
            for actor_id in actor_ids {
                by_participant_edge.insert((actor_id, feed_id.clone()));
            }
            Ok(())
        })
    }

    fn get_by_source_request(
        &self,
        source_type: &str,
        source_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let source_key = (
            source_type.to_string(),
            source_id.to_string(),
            request_id.to_string(),
        );
        let by_source_request = self.by_source_request.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let by_source_request = by_source_request.read().await;
            let Some(feed_id) = by_source_request.get(&source_key) else {
                return Ok(None);
            };
            Ok(by_id.read().await.get(feed_id).cloned())
        })
    }

    fn get_by_feed_id(
        &self,
        feed_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let feed_id = feed_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move { Ok(by_id.read().await.get(&feed_id).cloned()) })
    }

    fn get_latest_by_source(
        &self,
        source_type: &str,
        source_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let source_type = source_type.to_string();
        let source_id = source_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let items = by_id.read().await;
            Ok(items
                .values()
                .filter(|item| item.source_type == source_type && item.source_id == source_id)
                .max_by(|left, right| {
                    left.occurred_at_ms
                        .cmp(&right.occurred_at_ms)
                        .then_with(|| left.feed_id.cmp(&right.feed_id))
                })
                .cloned())
        })
    }

    fn merge_payload(
        &self,
        feed_id: &str,
        payload_patch: Value,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<FeedItem>> {
        let feed_id = feed_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut by_id = by_id.write().await;
            let item = by_id.get_mut(&feed_id).ok_or(DomainError::NotFound)?;
            let mut payload = item
                .payload
                .clone()
                .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
            deep_merge_json(&mut payload, payload_patch);
            item.payload = Some(payload);
            Ok(item.clone())
        })
    }

    fn list_feed(
        &self,
        query: &FeedRepositoryQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
        let query = query.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut items: Vec<FeedItem> = by_id
                .read()
                .await
                .values()
                .filter(|item| {
                    if let Some(scope_id) = query.scope_id.as_ref() {
                        if item.scope_id.as_deref() != Some(scope_id.as_str()) {
                            return false;
                        }
                    }
                    if let Some(privacy_level) = query.privacy_level.as_ref() {
                        if item.privacy_level.as_deref() != Some(privacy_level.as_str()) {
                            return false;
                        }
                    }
                    if let Some(from_ms) = query.from_ms {
                        if item.occurred_at_ms < from_ms {
                            return false;
                        }
                    }
                    if let Some(to_ms) = query.to_ms {
                        if item.occurred_at_ms > to_ms {
                            return false;
                        }
                    }
                    if query.involvement_only
                        && item.actor_id != query.actor_id
                        && !item.participant_ids.iter().any(|id| id == &query.actor_id)
                    {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();
            items.sort_by(|left, right| {
                right
                    .occurred_at_ms
                    .cmp(&left.occurred_at_ms)
                    .then_with(|| right.feed_id.cmp(&left.feed_id))
            });
            let items = if let (Some(cursor_ms), Some(cursor_feed_id)) =
                (query.cursor_occurred_at_ms, query.cursor_feed_id.as_ref())
            {
                items
                    .into_iter()
                    .filter(|item| {
                        item.occurred_at_ms < cursor_ms
                            || (item.occurred_at_ms == cursor_ms && item.feed_id < *cursor_feed_id)
                    })
                    .collect()
            } else {
                items
            };
            Ok(items.into_iter().take(query.limit).collect())
        })
    }

    fn search_feed(
        &self,
        query: &FeedRepositorySearchQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
        let query = query.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut items: Vec<FeedItem> = by_id
                .read()
                .await
                .values()
                .filter(|item| {
                    if let Some(scope_id) = query.scope_id.as_ref() {
                        if item.scope_id.as_deref() != Some(scope_id.as_str()) {
                            return false;
                        }
                    }
                    if let Some(privacy_level) = query.privacy_level.as_ref() {
                        if item.privacy_level.as_deref() != Some(privacy_level.as_str()) {
                            return false;
                        }
                    }
                    if query.exclude_vault && item.source_type == FEED_SOURCE_VAULT {
                        return false;
                    }
                    if let Some(from_ms) = query.from_ms {
                        if item.occurred_at_ms < from_ms {
                            return false;
                        }
                    }
                    if let Some(to_ms) = query.to_ms {
                        if item.occurred_at_ms > to_ms {
                            return false;
                        }
                    }
                    if query.involvement_only
                        && item.actor_id != query.actor_id
                        && !item.participant_ids.iter().any(|id| id == &query.actor_id)
                    {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();
            items.sort_by(|left, right| {
                right
                    .occurred_at_ms
                    .cmp(&left.occurred_at_ms)
                    .then_with(|| right.feed_id.cmp(&left.feed_id))
            });
            Ok(items.into_iter().take(query.limit).collect())
        })
    }
}

#[derive(Default)]
pub struct InMemoryDiscoveryNotificationRepository {
    by_id: Arc<RwLock<HashMap<String, InAppNotification>>>,
    by_dedupe_key: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl InMemoryDiscoveryNotificationRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NotificationRepository for InMemoryDiscoveryNotificationRepository {
    fn create_notification(
        &self,
        notification: &InAppNotification,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<InAppNotification>> {
        let notification = notification.clone();
        let by_id = self.by_id.clone();
        let by_dedupe_key = self.by_dedupe_key.clone();
        Box::pin(async move {
            let dedupe_key = (
                notification.user_id.clone(),
                notification.dedupe_key.clone(),
            );
            let mut by_dedupe_key = by_dedupe_key.write().await;
            if by_dedupe_key.contains_key(&dedupe_key) {
                return Err(DomainError::Conflict);
            }
            by_dedupe_key.insert(dedupe_key, notification.notification_id.clone());
            by_id
                .write()
                .await
                .insert(notification.notification_id.clone(), notification.clone());
            Ok(notification)
        })
    }

    fn get_by_dedupe_key(
        &self,
        user_id: &str,
        dedupe_key: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<InAppNotification>>> {
        let by_dedupe_key = self.by_dedupe_key.clone();
        let by_id = self.by_id.clone();
        let key = (user_id.to_string(), dedupe_key.to_string());
        Box::pin(async move {
            let by_dedupe_key = by_dedupe_key.read().await;
            let Some(notification_id) = by_dedupe_key.get(&key) else {
                return Ok(None);
            };
            Ok(by_id.read().await.get(notification_id).cloned())
        })
    }

    fn list_notifications(
        &self,
        query: &NotificationRepositoryListQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
        let query = query.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut items: Vec<InAppNotification> = by_id
                .read()
                .await
                .values()
                .filter(|notification| {
                    if notification.user_id != query.user_id {
                        return false;
                    }
                    if !query.include_read && notification.read_at_ms.is_some() {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();
            items.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.notification_id.cmp(&left.notification_id))
            });
            let items = if let (Some(cursor_ms), Some(cursor_notification_id)) = (
                query.cursor_created_at_ms,
                query.cursor_notification_id.as_ref(),
            ) {
                items
                    .into_iter()
                    .filter(|item| {
                        item.created_at_ms < cursor_ms
                            || (item.created_at_ms == cursor_ms
                                && item.notification_id < *cursor_notification_id)
                    })
                    .collect()
            } else {
                items
            };
            Ok(items.into_iter().take(query.limit).collect())
        })
    }

    fn list_notifications_in_window(
        &self,
        user_id: &str,
        window_start_ms: i64,
        window_end_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
        let user_id = user_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut items: Vec<InAppNotification> = by_id
                .read()
                .await
                .values()
                .filter(|notification| {
                    notification.user_id == user_id
                        && notification.created_at_ms >= window_start_ms
                        && notification.created_at_ms <= window_end_ms
                })
                .cloned()
                .collect();
            items.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.notification_id.cmp(&left.notification_id))
            });
            Ok(items)
        })
    }

    fn mark_as_read(
        &self,
        user_id: &str,
        notification_id: &str,
        read_at_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<InAppNotification>> {
        let user_id = user_id.to_string();
        let notification_id = notification_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut by_id = by_id.write().await;
            let Some(mut notification) = by_id.remove(&notification_id) else {
                return Err(DomainError::NotFound);
            };
            if notification.user_id != user_id {
                by_id.insert(notification_id, notification);
                return Err(DomainError::Forbidden(
                    "notification belongs to another user".into(),
                ));
            }
            notification.read_at_ms = Some(read_at_ms);
            by_id.insert(notification.notification_id.clone(), notification.clone());
            Ok(notification)
        })
    }

    fn unread_count(
        &self,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<usize>> {
        let user_id = user_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let count = by_id
                .read()
                .await
                .values()
                .filter(|notification| {
                    notification.user_id == user_id && notification.read_at_ms.is_none()
                })
                .count();
            Ok(count)
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurrealDiscoveryFeedRepositoryOptions {
    pub involvement_fallback_enabled: bool,
}

impl Default for SurrealDiscoveryFeedRepositoryOptions {
    fn default() -> Self {
        Self {
            involvement_fallback_enabled: true,
        }
    }
}

#[derive(Clone)]
pub struct SurrealDiscoveryFeedRepository {
    client: Arc<Surreal<Client>>,
    options: SurrealDiscoveryFeedRepositoryOptions,
}

impl SurrealDiscoveryFeedRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self::with_client_and_options(client, SurrealDiscoveryFeedRepositoryOptions::default())
    }

    pub fn with_client_and_options(
        client: Arc<Surreal<Client>>,
        options: SurrealDiscoveryFeedRepositoryOptions,
    ) -> Self {
        Self { client, options }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        Self::new_with_options(db_config, SurrealDiscoveryFeedRepositoryOptions::default()).await
    }

    pub async fn new_with_options(
        db_config: &DbConfig,
        options: SurrealDiscoveryFeedRepositoryOptions,
    ) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
            options,
        })
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn parse_datetime_value(value: &Value) -> DomainResult<i64> {
        if let Some(raw) = value.as_str() {
            return Self::parse_datetime_ms(raw);
        }
        if let Some(obj) = value.as_object() {
            for nested in obj.values() {
                if let Some(raw) = nested.as_str() {
                    return Self::parse_datetime_ms(raw);
                }
            }
        }
        Err(DomainError::Validation(format!(
            "invalid datetime payload: {value}"
        )))
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let datetime =
            OffsetDateTime::from_unix_timestamp_nanos((timestamp_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(datetime
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn feed_select_projection() -> &'static str {
        "feed_id, source_type, source_id, actor_id, actor_username, title, summary, scope_id, \
         privacy_level, <string>occurred_at AS occurred_at, <string>created_at AS created_at, \
         request_id, correlation_id, participant_ids, payload"
    }

    fn map_rows(rows: Vec<Value>) -> DomainResult<Vec<FeedItem>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealDiscoveryFeedRow>(row)
                    .map_err(|err| DomainError::Validation(format!("invalid feed row: {err}")))
                    .and_then(|row| {
                        let occurred_at_ms = Self::parse_datetime_value(&row.occurred_at)?;
                        let created_at_ms = Self::parse_datetime_value(&row.created_at)?;
                        Ok(FeedItem {
                            feed_id: row.feed_id,
                            source_type: row.source_type,
                            source_id: row.source_id,
                            actor_id: row.actor_id,
                            actor_username: row.actor_username,
                            title: row.title,
                            summary: row.summary,
                            scope_id: row.scope_id,
                            privacy_level: row.privacy_level,
                            occurred_at_ms,
                            created_at_ms,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            participant_ids: row.participant_ids,
                            payload: row.payload,
                        })
                    })
            })
            .collect()
    }

    fn to_create_payload(item: &FeedItem) -> DomainResult<SurrealDiscoveryFeedCreateRow> {
        let occurred_at = Self::to_rfc3339(item.occurred_at_ms)?;
        let created_at = Self::to_rfc3339(item.created_at_ms)?;
        Ok(SurrealDiscoveryFeedCreateRow {
            feed_id: item.feed_id.clone(),
            source_type: item.source_type.clone(),
            source_id: item.source_id.clone(),
            actor_id: item.actor_id.clone(),
            actor_username: item.actor_username.clone(),
            title: item.title.clone(),
            summary: item.summary.clone(),
            scope_id: item.scope_id.clone(),
            privacy_level: item.privacy_level.clone(),
            occurred_at,
            created_at,
            request_id: item.request_id.clone(),
            correlation_id: item.correlation_id.clone(),
            participant_ids: item.participant_ids.clone(),
            payload: item.payload.clone(),
        })
    }

    fn register_involvement_lane(endpoint: &str, lane: &str) {
        counter!(
            FEED_INVOLVEMENT_LANE_REQUESTS_TOTAL,
            "endpoint" => endpoint.to_string(),
            "lane" => lane.to_string()
        )
        .increment(1);
    }

    fn register_involvement_shadow_mismatch(endpoint: &str, mismatch: &str) {
        counter!(
            FEED_INVOLVEMENT_SHADOW_MISMATCH_TOTAL,
            "endpoint" => endpoint.to_string(),
            "mismatch" => mismatch.to_string()
        )
        .increment(1);
    }

    fn sort_feed_items_desc(items: &mut [FeedItem]) {
        items.sort_by(|left, right| {
            right
                .occurred_at_ms
                .cmp(&left.occurred_at_ms)
                .then_with(|| right.feed_id.cmp(&left.feed_id))
        });
    }

    fn merge_feed_items(
        primary: Vec<FeedItem>,
        fallback: Vec<FeedItem>,
        limit: usize,
    ) -> Vec<FeedItem> {
        let mut merged = Vec::with_capacity(primary.len().saturating_add(fallback.len()));
        let mut seen_feed_ids = HashSet::new();

        for item in primary.into_iter().chain(fallback.into_iter()) {
            if seen_feed_ids.insert(item.feed_id.clone()) {
                merged.push(item);
            }
        }

        Self::sort_feed_items_desc(&mut merged);
        if merged.len() > limit {
            merged.truncate(limit);
        }
        merged
    }

    async fn hydrate_feed_rows_by_ids(&self, feed_ids: &[String]) -> DomainResult<Vec<FeedItem>> {
        if feed_ids.is_empty() {
            return Ok(Vec::new());
        }

        let projection = Self::feed_select_projection();
        let mut response = self
            .client
            .query(format!(
                "SELECT {projection} FROM discovery_feed_item WHERE feed_id IN $feed_ids",
            ))
            .bind(("feed_ids", feed_ids.to_vec()))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let hydrated_rows = Self::map_rows(rows)?;
        let mut by_feed_id: HashMap<String, FeedItem> = hydrated_rows
            .into_iter()
            .map(|item| (item.feed_id.clone(), item))
            .collect();

        let mut ordered = Vec::with_capacity(feed_ids.len());
        for feed_id in feed_ids {
            if let Some(item) = by_feed_id.remove(feed_id) {
                ordered.push(item);
            }
        }
        Ok(ordered)
    }

    async fn list_participant_edge_feed_ids(
        &self,
        actor_id: &str,
        scope_id: Option<&str>,
        privacy_level: Option<&str>,
        from_ms: Option<i64>,
        to_ms: Option<i64>,
        cursor_occurred_at_ms: Option<i64>,
        cursor_feed_id: Option<&str>,
        exclude_source_type: Option<&str>,
        limit: usize,
    ) -> DomainResult<Vec<String>> {
        let mut clauses = vec!["actor_id = $actor_id"];
        if scope_id.is_some() {
            clauses.push("scope_id = $scope_id");
        }
        if privacy_level.is_some() {
            clauses.push("privacy_level = $privacy_level");
        }
        if from_ms.is_some() {
            clauses.push("occurred_at >= <datetime>$from_occurred_at");
        }
        if to_ms.is_some() {
            clauses.push("occurred_at <= <datetime>$to_occurred_at");
        }
        if exclude_source_type.is_some() {
            clauses.push("source_type != $exclude_source_type");
        }
        if cursor_occurred_at_ms.is_some() && cursor_feed_id.is_some() {
            clauses.push(
                "(occurred_at < <datetime>$cursor_occurred_at OR (occurred_at = <datetime>$cursor_occurred_at AND feed_id < $cursor_feed_id))",
            );
        }

        let mut statement = String::from(
            "SELECT feed_id, <string>occurred_at AS occurred_at FROM feed_participant_edge",
        );
        if !clauses.is_empty() {
            statement.push_str(" WHERE ");
            statement.push_str(&clauses.join(" AND "));
        }
        statement.push_str(" ORDER BY occurred_at DESC, feed_id DESC LIMIT $limit");

        let mut db_query = self
            .client
            .query(statement)
            .bind(("actor_id", actor_id.to_string()))
            .bind(("limit", limit as i64));

        if let Some(scope_id) = scope_id {
            db_query = db_query.bind(("scope_id", scope_id.to_string()));
        }
        if let Some(privacy_level) = privacy_level {
            db_query = db_query.bind(("privacy_level", privacy_level.to_string()));
        }
        if let Some(from_ms) = from_ms {
            let from_occurred_at = Self::to_rfc3339(from_ms)?;
            db_query = db_query.bind(("from_occurred_at", from_occurred_at));
        }
        if let Some(to_ms) = to_ms {
            let to_occurred_at = Self::to_rfc3339(to_ms)?;
            db_query = db_query.bind(("to_occurred_at", to_occurred_at));
        }
        if let Some(exclude_source_type) = exclude_source_type {
            db_query = db_query.bind(("exclude_source_type", exclude_source_type.to_string()));
        }
        if let (Some(cursor_ms), Some(cursor_feed_id)) = (cursor_occurred_at_ms, cursor_feed_id) {
            let cursor_occurred_at = Self::to_rfc3339(cursor_ms)?;
            db_query = db_query
                .bind(("cursor_occurred_at", cursor_occurred_at))
                .bind(("cursor_feed_id", cursor_feed_id.to_string()));
        }

        let mut response = db_query.await.map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;

        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealFeedParticipantEdgeCandidateRow>(row)
                    .map(|row| row.feed_id)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid feed participant row: {err}"))
                    })
            })
            .collect()
    }

    async fn list_feed_legacy(&self, query: &FeedRepositoryQuery) -> DomainResult<Vec<FeedItem>> {
        let mut clauses = Vec::new();
        if query.scope_id.is_some() {
            clauses.push("scope_id = $scope_id");
        }
        if query.privacy_level.is_some() {
            clauses.push("privacy_level = $privacy_level");
        }
        if query.from_ms.is_some() {
            clauses.push("occurred_at >= <datetime>$from_occurred_at");
        }
        if query.to_ms.is_some() {
            clauses.push("occurred_at <= <datetime>$to_occurred_at");
        }
        if query.involvement_only {
            clauses.push("(actor_id = $actor_id OR $actor_id IN participant_ids)");
        }
        if query.cursor_occurred_at_ms.is_some() && query.cursor_feed_id.is_some() {
            clauses.push(
                "(occurred_at < <datetime>$cursor_occurred_at OR (occurred_at = <datetime>$cursor_occurred_at AND feed_id < $cursor_feed_id))",
            );
        }

        let projection = Self::feed_select_projection();
        let mut statement = format!("SELECT {projection} FROM discovery_feed_item");
        if !clauses.is_empty() {
            statement.push_str(" WHERE ");
            statement.push_str(&clauses.join(" AND "));
        }
        statement.push_str(" ORDER BY occurred_at DESC, feed_id DESC LIMIT $limit");

        let mut db_query = self
            .client
            .query(statement)
            .bind(("limit", query.limit as i64));

        if let Some(scope_id) = query.scope_id.as_deref() {
            db_query = db_query.bind(("scope_id", scope_id.to_string()));
        }
        if let Some(privacy_level) = query.privacy_level.as_deref() {
            db_query = db_query.bind(("privacy_level", privacy_level.to_string()));
        }
        if let Some(from_ms) = query.from_ms {
            let cursor = Self::to_rfc3339(from_ms)?;
            db_query = db_query.bind(("from_occurred_at", cursor));
        }
        if let Some(to_ms) = query.to_ms {
            let cursor = Self::to_rfc3339(to_ms)?;
            db_query = db_query.bind(("to_occurred_at", cursor));
        }
        if query.involvement_only {
            db_query = db_query.bind(("actor_id", query.actor_id.clone()));
        }
        if let (Some(cursor_ms), Some(cursor_feed_id)) =
            (query.cursor_occurred_at_ms, query.cursor_feed_id.as_deref())
        {
            let cursor = Self::to_rfc3339(cursor_ms)?;
            db_query = db_query
                .bind(("cursor_occurred_at", cursor))
                .bind(("cursor_feed_id", cursor_feed_id.to_string()));
        }

        let mut response = db_query.await.map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        Self::map_rows(rows)
    }

    async fn list_feed_involvement_edge_first(
        &self,
        query: &FeedRepositoryQuery,
    ) -> DomainResult<Vec<FeedItem>> {
        let edge_feed_ids = match self
            .list_participant_edge_feed_ids(
                &query.actor_id,
                query.scope_id.as_deref(),
                query.privacy_level.as_deref(),
                query.from_ms,
                query.to_ms,
                query.cursor_occurred_at_ms,
                query.cursor_feed_id.as_deref(),
                None,
                query.limit,
            )
            .await
        {
            Ok(feed_ids) => feed_ids,
            Err(err) => {
                if !self.options.involvement_fallback_enabled {
                    Self::register_involvement_lane("feed", "edge_error");
                    return Err(err);
                }
                tracing::warn!(
                    actor_id = %query.actor_id,
                    error = %err,
                    "failed edge candidate query for involvement feed; falling back to legacy lane"
                );
                Self::register_involvement_lane("feed", "legacy");
                return self.list_feed_legacy(query).await;
            }
        };
        let edge_rows = match self.hydrate_feed_rows_by_ids(&edge_feed_ids).await {
            Ok(rows) => rows,
            Err(err) => {
                if !self.options.involvement_fallback_enabled {
                    Self::register_involvement_lane("feed", "edge_error");
                    return Err(err);
                }
                tracing::warn!(
                    actor_id = %query.actor_id,
                    error = %err,
                    "failed edge hydration for involvement feed; falling back to legacy lane"
                );
                Self::register_involvement_lane("feed", "legacy");
                return self.list_feed_legacy(query).await;
            }
        };
        if edge_rows.len() >= query.limit {
            Self::register_involvement_lane("feed", "edge");
            return Ok(edge_rows);
        }
        if !self.options.involvement_fallback_enabled {
            tracing::warn!(
                actor_id = %query.actor_id,
                edge_count = edge_rows.len(),
                requested_limit = query.limit,
                "discovery feed involvement edge lane returned partial page with fallback disabled"
            );
            Self::register_involvement_lane("feed", "edge_partial");
            return Ok(edge_rows);
        }

        let mut fallback_query = query.clone();
        fallback_query.limit = query
            .limit
            .saturating_mul(2)
            .max(query.limit.saturating_add(1))
            .max(1);
        let legacy_rows = self.list_feed_legacy(&fallback_query).await?;
        Self::register_involvement_lane("feed", "fallback");

        if !edge_rows.is_empty() {
            let edge_feed_ids: Vec<&str> =
                edge_rows.iter().map(|item| item.feed_id.as_str()).collect();
            let legacy_head: Vec<&str> = legacy_rows
                .iter()
                .take(edge_feed_ids.len())
                .map(|item| item.feed_id.as_str())
                .collect();
            if edge_feed_ids != legacy_head {
                Self::register_involvement_shadow_mismatch("feed", "ordering_prefix");
                tracing::warn!(
                    actor_id = %query.actor_id,
                    edge_count = edge_feed_ids.len(),
                    fallback_count = legacy_rows.len(),
                    "discovery feed involvement edge lane diverges from legacy ordering"
                );
            }
        }

        Ok(Self::merge_feed_items(edge_rows, legacy_rows, query.limit))
    }

    async fn search_feed_legacy(
        &self,
        query: &FeedRepositorySearchQuery,
    ) -> DomainResult<Vec<FeedItem>> {
        let mut clauses = Vec::new();
        if query.scope_id.is_some() {
            clauses.push("scope_id = $scope_id");
        }
        if query.privacy_level.is_some() {
            clauses.push("privacy_level = $privacy_level");
        }
        if query.exclude_vault {
            clauses.push("source_type != $exclude_source_type");
        }
        if query.from_ms.is_some() {
            clauses.push("occurred_at >= <datetime>$from_occurred_at");
        }
        if query.to_ms.is_some() {
            clauses.push("occurred_at <= <datetime>$to_occurred_at");
        }
        if query.involvement_only {
            clauses.push("(actor_id = $actor_id OR $actor_id IN participant_ids)");
        }

        let projection = Self::feed_select_projection();
        let mut statement = format!("SELECT {projection} FROM discovery_feed_item");
        if !clauses.is_empty() {
            statement.push_str(" WHERE ");
            statement.push_str(&clauses.join(" AND "));
        }
        statement.push_str(" ORDER BY occurred_at DESC, feed_id DESC LIMIT $limit");

        let mut db_query = self
            .client
            .query(statement)
            .bind(("limit", query.limit as i64));

        if let Some(scope_id) = query.scope_id.as_deref() {
            db_query = db_query.bind(("scope_id", scope_id.to_string()));
        }
        if let Some(privacy_level) = query.privacy_level.as_deref() {
            db_query = db_query.bind(("privacy_level", privacy_level.to_string()));
        }
        if query.exclude_vault {
            db_query = db_query.bind(("exclude_source_type", FEED_SOURCE_VAULT.to_string()));
        }
        if let Some(from_ms) = query.from_ms {
            let from = Self::to_rfc3339(from_ms)?;
            db_query = db_query.bind(("from_occurred_at", from));
        }
        if let Some(to_ms) = query.to_ms {
            let to = Self::to_rfc3339(to_ms)?;
            db_query = db_query.bind(("to_occurred_at", to));
        }
        if query.involvement_only {
            db_query = db_query.bind(("actor_id", query.actor_id.clone()));
        }

        let mut response = db_query.await.map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let mut rows = Self::map_rows(rows)?;
        Self::sort_feed_items_desc(&mut rows);
        Ok(rows)
    }

    async fn search_feed_involvement_edge_first(
        &self,
        query: &FeedRepositorySearchQuery,
    ) -> DomainResult<Vec<FeedItem>> {
        let edge_feed_ids = match self
            .list_participant_edge_feed_ids(
                &query.actor_id,
                query.scope_id.as_deref(),
                query.privacy_level.as_deref(),
                query.from_ms,
                query.to_ms,
                None,
                None,
                if query.exclude_vault {
                    Some(FEED_SOURCE_VAULT)
                } else {
                    None
                },
                query.limit,
            )
            .await
        {
            Ok(feed_ids) => feed_ids,
            Err(err) => {
                if !self.options.involvement_fallback_enabled {
                    Self::register_involvement_lane("search", "edge_error");
                    return Err(err);
                }
                tracing::warn!(
                    actor_id = %query.actor_id,
                    error = %err,
                    "failed edge candidate query for involvement search; falling back to legacy lane"
                );
                Self::register_involvement_lane("search", "legacy");
                return self.search_feed_legacy(query).await;
            }
        };
        let edge_rows = match self.hydrate_feed_rows_by_ids(&edge_feed_ids).await {
            Ok(rows) => rows,
            Err(err) => {
                if !self.options.involvement_fallback_enabled {
                    Self::register_involvement_lane("search", "edge_error");
                    return Err(err);
                }
                tracing::warn!(
                    actor_id = %query.actor_id,
                    error = %err,
                    "failed edge hydration for involvement search; falling back to legacy lane"
                );
                Self::register_involvement_lane("search", "legacy");
                return self.search_feed_legacy(query).await;
            }
        };
        if edge_rows.len() >= query.limit {
            Self::register_involvement_lane("search", "edge");
            return Ok(edge_rows);
        }
        if !self.options.involvement_fallback_enabled {
            tracing::warn!(
                actor_id = %query.actor_id,
                edge_count = edge_rows.len(),
                requested_limit = query.limit,
                "discovery search involvement edge lane returned partial page with fallback disabled"
            );
            Self::register_involvement_lane("search", "edge_partial");
            return Ok(edge_rows);
        }

        let mut fallback_query = query.clone();
        fallback_query.limit = query
            .limit
            .saturating_mul(2)
            .max(query.limit.saturating_add(1))
            .max(1);
        let legacy_rows = self.search_feed_legacy(&fallback_query).await?;
        Self::register_involvement_lane("search", "fallback");

        if !edge_rows.is_empty() {
            let edge_feed_ids: Vec<&str> =
                edge_rows.iter().map(|item| item.feed_id.as_str()).collect();
            let legacy_head: Vec<&str> = legacy_rows
                .iter()
                .take(edge_feed_ids.len())
                .map(|item| item.feed_id.as_str())
                .collect();
            if edge_feed_ids != legacy_head {
                Self::register_involvement_shadow_mismatch("search", "ordering_prefix");
                tracing::warn!(
                    actor_id = %query.actor_id,
                    edge_count = edge_feed_ids.len(),
                    fallback_count = legacy_rows.len(),
                    "discovery search involvement edge lane diverges from legacy ordering"
                );
            }
        }

        Ok(Self::merge_feed_items(edge_rows, legacy_rows, query.limit))
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }
}

impl FeedRepository for SurrealDiscoveryFeedRepository {
    fn create_feed_item(
        &self,
        item: &FeedItem,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<FeedItem>> {
        let item = item.clone();
        let payload = match Self::to_create_payload(&item) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            client
                .query(
                    "CREATE discovery_feed_item SET \
                     feed_id = $payload.feed_id, \
                     source_type = $payload.source_type, \
                     source_id = $payload.source_id, \
                     actor_id = $payload.actor_id, \
                     actor_username = $payload.actor_username, \
                     title = $payload.title, \
                     summary = IF $payload.summary = NULL THEN NONE ELSE $payload.summary END, \
                     scope_id = IF $payload.scope_id = NULL THEN NONE ELSE $payload.scope_id END, \
                     privacy_level = IF $payload.privacy_level = NULL THEN NONE ELSE $payload.privacy_level END, \
                     occurred_at = <datetime>$payload.occurred_at, \
                     created_at = <datetime>$payload.created_at, \
                     request_id = $payload.request_id, \
                     correlation_id = $payload.correlation_id, \
                     participant_ids = $payload.participant_ids, \
                     payload = IF $payload.payload = NULL THEN NONE ELSE $payload.payload END",
                )
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            Ok(item)
        })
    }

    fn upsert_participant_edges_for_item(
        &self,
        item: &FeedItem,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<()>> {
        let item = item.clone();
        let actor_ids = feed_participant_actor_ids(&item);
        let occurred_at = match Self::to_rfc3339(item.occurred_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let created_at = match Self::to_rfc3339(item.created_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            for actor_id in actor_ids {
                let edge_id = format!("{actor_id}:{}", item.feed_id);
                let result = client
                    .query(
                        "CREATE feed_participant_edge SET \
                         edge_id = $edge_id, \
                         actor_id = $actor_id, \
                         feed_id = $feed_id, \
                         occurred_at = <datetime>$occurred_at, \
                         scope_id = IF $scope_id = NULL THEN NONE ELSE $scope_id END, \
                         privacy_level = IF $privacy_level = NULL THEN NONE ELSE $privacy_level END, \
                         source_type = $source_type, \
                         source_id = $source_id, \
                         created_at = <datetime>$created_at, \
                         request_id = $request_id",
                    )
                    .bind(("edge_id", edge_id))
                    .bind(("actor_id", actor_id))
                    .bind(("feed_id", item.feed_id.clone()))
                    .bind(("occurred_at", occurred_at.clone()))
                    .bind(("scope_id", item.scope_id.clone()))
                    .bind(("privacy_level", item.privacy_level.clone()))
                    .bind(("source_type", item.source_type.clone()))
                    .bind(("source_id", item.source_id.clone()))
                    .bind(("created_at", created_at.clone()))
                    .bind(("request_id", item.request_id.clone()))
                    .await
                    .map_err(Self::map_surreal_error);

                match result {
                    Ok(_) | Err(DomainError::Conflict) => {}
                    Err(err) => return Err(err),
                }
            }
            Ok(())
        })
    }

    fn get_by_source_request(
        &self,
        source_type: &str,
        source_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let source_type = source_type.to_string();
        let source_id = source_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let projection = Self::feed_select_projection();
            let mut response = client
                .query(format!(
                    "SELECT {projection} FROM discovery_feed_item \
                     WHERE source_type = $source_type AND source_id = $source_id AND request_id = $request_id LIMIT 1",
                ))
                .bind(("source_type", source_type))
                .bind(("source_id", source_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            Ok(rows.pop())
        })
    }

    fn get_by_feed_id(
        &self,
        feed_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let feed_id = feed_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let projection = Self::feed_select_projection();
            let mut response = client
                .query(format!(
                    "SELECT {projection} FROM discovery_feed_item WHERE feed_id = $feed_id LIMIT 1",
                ))
                .bind(("feed_id", feed_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            Ok(rows.pop())
        })
    }

    fn get_latest_by_source(
        &self,
        source_type: &str,
        source_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<FeedItem>>> {
        let source_type = source_type.to_string();
        let source_id = source_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let projection = Self::feed_select_projection();
            let mut response = client
                .query(format!(
                    "SELECT {projection} FROM discovery_feed_item \
                     WHERE source_type = $source_type AND source_id = $source_id \
                     ORDER BY occurred_at DESC, feed_id DESC LIMIT 1",
                ))
                .bind(("source_type", source_type))
                .bind(("source_id", source_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            Ok(rows.pop())
        })
    }

    fn merge_payload(
        &self,
        feed_id: &str,
        payload_patch: Value,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<FeedItem>> {
        let feed_id = feed_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT payload FROM discovery_feed_item WHERE feed_id = $feed_id LIMIT 1")
                .bind(("feed_id", feed_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let existing_payload = rows
                .into_iter()
                .next()
                .and_then(|row| row.get("payload").cloned())
                .unwrap_or(Value::Null);
            let mut merged_payload = match existing_payload {
                Value::Null => Value::Object(serde_json::Map::new()),
                other => other,
            };
            deep_merge_json(&mut merged_payload, payload_patch);

            client
                .query(
                    "UPDATE discovery_feed_item SET payload = $payload \
                     WHERE feed_id = $feed_id",
                )
                .bind(("feed_id", feed_id.clone()))
                .bind(("payload", merged_payload))
                .await
                .map_err(Self::map_surreal_error)?;

            let projection = Self::feed_select_projection();
            let mut response = client
                .query(format!(
                    "SELECT {projection} FROM discovery_feed_item WHERE feed_id = $feed_id LIMIT 1",
                ))
                .bind(("feed_id", feed_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            rows.pop().ok_or(DomainError::NotFound)
        })
    }

    fn list_feed(
        &self,
        query: &FeedRepositoryQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
        let query = query.clone();
        let repository = self.clone();
        Box::pin(async move {
            if query.involvement_only {
                return repository.list_feed_involvement_edge_first(&query).await;
            }
            repository.list_feed_legacy(&query).await
        })
    }

    fn search_feed(
        &self,
        query: &FeedRepositorySearchQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<FeedItem>>> {
        let query = query.clone();
        let repository = self.clone();
        Box::pin(async move {
            if query.involvement_only {
                return repository.search_feed_involvement_edge_first(&query).await;
            }
            repository.search_feed_legacy(&query).await
        })
    }
}

#[derive(Clone)]
pub struct SurrealDiscoveryNotificationRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealDiscoveryNotificationRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let datetime =
            OffsetDateTime::from_unix_timestamp_nanos((timestamp_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(datetime
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn map_rows(rows: Vec<Value>) -> DomainResult<Vec<InAppNotification>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealDiscoveryNotificationRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid notification row: {err}"))
                    })
                    .and_then(|row| {
                        let read_at_ms = match row.read_at {
                            Some(value) => Some(Self::parse_datetime_ms(&value)?),
                            None => None,
                        };
                        Ok(InAppNotification {
                            notification_id: row.notification_id,
                            user_id: row.user_id,
                            actor_id: row.actor_id,
                            actor_username: row.actor_username,
                            notification_type: row.notification_type,
                            source_type: row.source_type,
                            source_id: row.source_id,
                            title: row.title,
                            body: row.body,
                            payload: row.payload,
                            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
                            read_at_ms,
                            privacy_level: row.privacy_level,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            dedupe_key: row.dedupe_key,
                        })
                    })
            })
            .collect()
    }

    fn to_create_payload(
        notification: &InAppNotification,
    ) -> DomainResult<SurrealDiscoveryNotificationCreateRow> {
        let created_at = Self::to_rfc3339(notification.created_at_ms)?;
        Ok(SurrealDiscoveryNotificationCreateRow {
            notification_id: notification.notification_id.clone(),
            user_id: notification.user_id.clone(),
            actor_id: notification.actor_id.clone(),
            actor_username: notification.actor_username.clone(),
            notification_type: notification.notification_type.clone(),
            source_type: notification.source_type.clone(),
            source_id: notification.source_id.clone(),
            title: notification.title.clone(),
            body: notification.body.clone(),
            payload: notification.payload.clone(),
            created_at,
            read_at: notification.read_at_ms.map(Self::to_rfc3339).transpose()?,
            privacy_level: notification.privacy_level.clone(),
            request_id: notification.request_id.clone(),
            correlation_id: notification.correlation_id.clone(),
            dedupe_key: notification.dedupe_key.clone(),
        })
    }

    fn decode_count(rows: Vec<Value>, field: &str, label: &str) -> DomainResult<usize> {
        let Some(row) = rows.into_iter().next() else {
            return Err(DomainError::Validation(format!("missing {label}")));
        };
        let Some(value) = row.get(field) else {
            return Err(DomainError::Validation(format!("{label} missing")));
        };
        let count = value
            .as_u64()
            .or_else(|| value.as_i64().and_then(|value| value.try_into().ok()))
            .ok_or_else(|| DomainError::Validation(format!("invalid {label}")))?;
        Ok(count as usize)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }
}

impl NotificationRepository for SurrealDiscoveryNotificationRepository {
    fn create_notification(
        &self,
        notification: &InAppNotification,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<InAppNotification>> {
        let notification = notification.clone();
        let payload = match Self::to_create_payload(&notification) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE discovery_notification CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            rows.pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn get_by_dedupe_key(
        &self,
        user_id: &str,
        dedupe_key: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<InAppNotification>>> {
        let user_id = user_id.to_string();
        let dedupe_key = dedupe_key.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM discovery_notification \
                     WHERE user_id = $user_id AND dedupe_key = $dedupe_key LIMIT 1",
                )
                .bind(("user_id", user_id))
                .bind(("dedupe_key", dedupe_key))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            Ok(rows.pop())
        })
    }

    fn list_notifications(
        &self,
        query: &NotificationRepositoryListQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
        let query = query.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let mut filters = vec!["user_id = $user_id".to_string()];
            if query.cursor_created_at_ms.is_some() && query.cursor_notification_id.is_some() {
                filters.push(
                    "((created_at < $cursor_created_at) OR (created_at = $cursor_created_at AND notification_id < $cursor_notification_id))".to_string(),
                );
            }
            if !query.include_read {
                filters.push("read_at IS NONE".to_string());
            }

            let mut statement = String::from("SELECT * FROM discovery_notification");
            if !filters.is_empty() {
                statement.push_str(" WHERE ");
                statement.push_str(&filters.join(" AND "));
            }
            statement.push_str(" ORDER BY created_at DESC, notification_id DESC LIMIT $limit");

            let mut db_query = client
                .query(statement)
                .bind(("user_id", query.user_id.clone()))
                .bind(("limit", query.limit as i64));

            if let Some(cursor_ms) = query.cursor_created_at_ms {
                if let Some(cursor_notification_id) = query.cursor_notification_id.as_deref() {
                    let cursor_created_at = Self::to_rfc3339(cursor_ms)?;
                    db_query = db_query
                        .bind(("cursor_created_at", cursor_created_at))
                        .bind(("cursor_notification_id", cursor_notification_id.to_string()));
                }
            }

            let mut response = db_query.await.map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let rows = Self::map_rows(rows)?;
            Ok(rows)
        })
    }

    fn list_notifications_in_window(
        &self,
        user_id: &str,
        window_start_ms: i64,
        window_end_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<InAppNotification>>> {
        let user_id = user_id.to_string();
        let window_start = match Self::to_rfc3339(window_start_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let window_end = match Self::to_rfc3339(window_end_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM discovery_notification \
                     WHERE user_id = $user_id AND created_at >= $window_start AND created_at <= $window_end \
                     ORDER BY created_at DESC, notification_id DESC")
                .bind(("user_id", user_id))
                .bind(("window_start", window_start))
                .bind(("window_end", window_end))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::map_rows(rows)
        })
    }

    fn mark_as_read(
        &self,
        user_id: &str,
        notification_id: &str,
        read_at_ms: i64,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<InAppNotification>> {
        let user_id = user_id.to_string();
        let notification_id = notification_id.to_string();
        let client = self.client.clone();
        let read_at = match Self::to_rfc3339(read_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM discovery_notification \
                     WHERE notification_id = $notification_id LIMIT 1",
                )
                .bind(("notification_id", notification_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            let Some(notification) = rows.pop() else {
                return Err(DomainError::NotFound);
            };
            if notification.user_id != user_id {
                return Err(DomainError::Forbidden(
                    "notification belongs to another user".into(),
                ));
            }
            if notification.read_at_ms.is_some() {
                return Ok(notification);
            }

            let mut response = client
                .query(
                    "UPDATE discovery_notification \
                     SET read_at = $read_at \
                     WHERE notification_id = $notification_id \
                     RETURN AFTER *",
                )
                .bind(("read_at", read_at))
                .bind(("notification_id", notification_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut rows = Self::map_rows(rows)?;
            rows.pop().ok_or(DomainError::NotFound)
        })
    }

    fn unread_count(
        &self,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<usize>> {
        let user_id = user_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT count() AS unread_count \
                     FROM discovery_notification \
                     WHERE user_id = $user_id AND read_at IS NONE",
                )
                .bind(("user_id", user_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_count(rows, "unread_count", "unread_count")
        })
    }
}

#[derive(Debug, Deserialize)]
struct SurrealDiscoveryFeedRow {
    feed_id: String,
    source_type: String,
    source_id: String,
    actor_id: String,
    actor_username: String,
    title: String,
    summary: Option<String>,
    scope_id: Option<String>,
    privacy_level: Option<String>,
    occurred_at: Value,
    created_at: Value,
    request_id: String,
    correlation_id: String,
    participant_ids: Vec<String>,
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealDiscoveryFeedCreateRow {
    feed_id: String,
    source_type: String,
    source_id: String,
    actor_id: String,
    actor_username: String,
    title: String,
    summary: Option<String>,
    scope_id: Option<String>,
    privacy_level: Option<String>,
    occurred_at: String,
    created_at: String,
    request_id: String,
    correlation_id: String,
    participant_ids: Vec<String>,
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SurrealFeedParticipantEdgeCandidateRow {
    feed_id: String,
}

#[derive(Debug, Deserialize)]
struct SurrealDiscoveryNotificationRow {
    notification_id: String,
    user_id: String,
    actor_id: String,
    actor_username: String,
    notification_type: String,
    source_type: String,
    source_id: String,
    title: String,
    body: String,
    payload: Option<serde_json::Value>,
    created_at: String,
    read_at: Option<String>,
    privacy_level: Option<String>,
    request_id: String,
    correlation_id: String,
    dedupe_key: String,
}

#[derive(Debug, Serialize)]
struct SurrealDiscoveryNotificationCreateRow {
    notification_id: String,
    user_id: String,
    actor_id: String,
    actor_username: String,
    notification_type: String,
    source_type: String,
    source_id: String,
    title: String,
    body: String,
    payload: Option<serde_json::Value>,
    created_at: String,
    read_at: Option<String>,
    privacy_level: Option<String>,
    request_id: String,
    correlation_id: String,
    dedupe_key: String,
}

#[derive(Clone)]
pub struct SurrealVaultRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealVaultRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn parse_state(value: &str) -> DomainResult<VaultState> {
        match value {
            "draft" => Ok(VaultState::Draft),
            "sealed" => Ok(VaultState::Sealed),
            "published" => Ok(VaultState::Published),
            "revoked" => Ok(VaultState::Revoked),
            "expired" => Ok(VaultState::Expired),
            other => Err(DomainError::Validation(format!(
                "invalid vault state '{other}'"
            ))),
        }
    }

    fn state_to_string(value: &VaultState) -> &'static str {
        match value {
            VaultState::Draft => "draft",
            VaultState::Sealed => "sealed",
            VaultState::Published => "published",
            VaultState::Revoked => "revoked",
            VaultState::Expired => "expired",
        }
    }

    fn parse_event_type(value: &str) -> DomainResult<VaultTimelineEventType> {
        match value {
            "witness_drafted" => Ok(VaultTimelineEventType::WitnessDrafted),
            "witness_sealed" => Ok(VaultTimelineEventType::WitnessSealed),
            "witness_trustee_added" => Ok(VaultTimelineEventType::WitnessTrusteeAdded),
            "witness_trustee_removed" => Ok(VaultTimelineEventType::WitnessTrusteeRemoved),
            "witness_published" => Ok(VaultTimelineEventType::WitnessPublished),
            "witness_revoked" => Ok(VaultTimelineEventType::WitnessRevoked),
            "witness_expired" => Ok(VaultTimelineEventType::WitnessExpired),
            other => Err(DomainError::Validation(format!(
                "invalid vault event type '{other}'"
            ))),
        }
    }

    fn event_type_to_string(value: &VaultTimelineEventType) -> &'static str {
        vault_timeline_event_type_to_string(value)
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn map_entry_rows(rows: Vec<Value>) -> DomainResult<Vec<VaultEntry>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealVaultEntryRow>(row)
                    .map_err(|err| DomainError::Validation(format!("invalid vault row: {err}")))
                    .and_then(|row| {
                        let vault_entry_id = row.vault_entry_id.clone();
                        let retention_tag = row
                            .retention_tag
                            .clone()
                            .unwrap_or_else(|| vault_entry_retention_tag(&vault_entry_id));
                        let event_hash = match row.event_hash {
                            Some(event_hash) => event_hash,
                            None => Self::vault_entry_audit_hash(&row, &retention_tag)
                                .map_err(|err| {
                                    DomainError::Validation(format!(
                                        "missing vault entry event_hash for entry '{}' and recompute failed: {err}",
                                        row.vault_entry_id
                                    ))
                                })?,
                        };
                        let created_at_ms = Self::parse_datetime_ms(&row.created_at)?;
                        let updated_at_ms = Self::parse_datetime_ms(&row.updated_at)?;
                        let sealed_at_ms = match row.sealed_at {
                            Some(value) => Some(Self::parse_datetime_ms(&value)?),
                            None => None,
                        };
                        Ok(VaultEntry {
                            vault_entry_id,
                            author_id: row.author_id,
                            author_username: row.author_username,
                            state: Self::parse_state(&row.state)?,
                            created_at_ms,
                            updated_at_ms,
                            sealed_at_ms,
                            sealed_hash: row.sealed_hash,
                            encryption_key_id: row.encryption_key_id,
                            attachment_refs: row.attachment_refs,
                            wali: row.wali,
                            payload: row.payload,
                            publish_target: row.publish_target,
                            retention_policy: row.retention_policy,
                            audit: row.audit,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            event_hash,
                            retention_tag,
                        })
                    })
            })
            .collect()
    }

    fn map_timeline_rows(rows: Vec<Value>) -> DomainResult<Vec<VaultTimelineEvent>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealVaultTimelineRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid vault timeline row: {err}"))
                    })
                    .and_then(|row| {
                        let occurred_at_ms = Self::parse_datetime_ms(&row.occurred_at)?;
                        let event_type = Self::parse_event_type(&row.event_type)?;
                        let vault_entry_id = row.vault_entry_id.clone();
                        let retention_tag = row.retention_tag.clone().unwrap_or_else(|| {
                            vault_timeline_retention_tag(&vault_entry_id, &event_type)
                        });
                        let event_hash = match row.event_hash {
                            Some(event_hash) => event_hash,
                            None => Self::vault_timeline_audit_hash(&row, &retention_tag).map_err(
                                |err| {
                                    DomainError::Validation(format!(
                                        "missing vault timeline event_hash for event '{}' and recompute failed: {err}",
                                        row.event_id
                                    ))
                                },
                            )?,
                        };
                        Ok(VaultTimelineEvent {
                            event_id: row.event_id,
                            vault_entry_id,
                            event_type: event_type.clone(),
                            actor: row.actor,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            occurred_at_ms,
                            metadata: row.metadata,
                            event_hash,
                            retention_tag,
                        })
                    })
            })
            .collect()
    }

    fn to_entry_payload(entry: &VaultEntry) -> DomainResult<SurrealVaultEntryCreateRow> {
        let created_at =
            OffsetDateTime::from_unix_timestamp_nanos((entry.created_at_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid created_at_ms: {err}")))?;
        let updated_at =
            OffsetDateTime::from_unix_timestamp_nanos((entry.updated_at_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid updated_at_ms: {err}")))?;
        let sealed_at = entry
            .sealed_at_ms
            .map(|sealed_at_ms| {
                OffsetDateTime::from_unix_timestamp_nanos((sealed_at_ms as i128) * 1_000_000)
                    .map_err(|err| DomainError::Validation(format!("invalid sealed_at_ms: {err}")))
                    .map(|datetime| {
                        datetime
                            .format(&Rfc3339)
                            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
                    })
            })
            .transpose()?;
        Ok(SurrealVaultEntryCreateRow {
            vault_entry_id: entry.vault_entry_id.clone(),
            author_id: entry.author_id.clone(),
            author_username: entry.author_username.clone(),
            state: Self::state_to_string(&entry.state).to_string(),
            created_at: created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            updated_at: updated_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            sealed_at,
            sealed_hash: entry.sealed_hash.clone(),
            encryption_key_id: entry.encryption_key_id.clone(),
            attachment_refs: entry.attachment_refs.clone(),
            wali: entry.wali.clone(),
            payload: entry.payload.clone(),
            publish_target: entry.publish_target.clone(),
            retention_policy: entry.retention_policy.clone(),
            audit: entry.audit.clone(),
            request_id: entry.request_id.clone(),
            correlation_id: entry.correlation_id.clone(),
            event_hash: entry.event_hash.clone(),
            retention_tag: entry.retention_tag.clone(),
        })
    }

    fn to_timeline_payload(
        event: &VaultTimelineEvent,
    ) -> DomainResult<SurrealVaultTimelineCreateRow> {
        let occurred_at =
            OffsetDateTime::from_unix_timestamp_nanos((event.occurred_at_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid occurred_at_ms: {err}")))?;
        Ok(SurrealVaultTimelineCreateRow {
            vault_entry_id: event.vault_entry_id.clone(),
            event_id: event.event_id.clone(),
            event_type: Self::event_type_to_string(&event.event_type).to_string(),
            actor: event.actor.clone(),
            request_id: event.request_id.clone(),
            correlation_id: event.correlation_id.clone(),
            occurred_at: occurred_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            metadata: event.metadata.clone(),
            event_hash: event.event_hash.clone(),
            retention_tag: event.retention_tag.clone(),
        })
    }

    async fn get_by_request_from_store(
        client: &Surreal<Client>,
        vault_entry_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<VaultEntry>> {
        let mut response = client
            .query(
                "SELECT * FROM vault_entry \
                 WHERE vault_entry_id = $vault_entry_id AND request_id = $request_id LIMIT 1",
            )
            .bind(("vault_entry_id", vault_entry_id.to_string()))
            .bind(("request_id", request_id.to_string()))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let mut entries = Self::map_entry_rows(vec![row])?;
        Ok(entries.pop())
    }

    async fn get_by_actor_request_from_store(
        client: &Surreal<Client>,
        actor_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<VaultEntry>> {
        let mut response = client
            .query(
                "SELECT * FROM vault_entry \
                 WHERE author_id = $author_id AND request_id = $request_id LIMIT 1",
            )
            .bind(("author_id", actor_id.to_string()))
            .bind(("request_id", request_id.to_string()))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let mut entries = Self::map_entry_rows(vec![row])?;
        Ok(entries.pop())
    }

    async fn get_by_entry_from_store(
        client: &Surreal<Client>,
        vault_entry_id: &str,
    ) -> DomainResult<Option<VaultEntry>> {
        let mut response = client
            .query("SELECT * FROM vault_entry WHERE vault_entry_id = $vault_entry_id LIMIT 1")
            .bind(("vault_entry_id", vault_entry_id.to_string()))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let mut entries = Self::map_entry_rows(vec![row])?;
        Ok(entries.pop())
    }

    async fn resolve_entry_id_from_actor_request(
        client: &Surreal<Client>,
        actor_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<String>> {
        let mut response = client
            .query(
                "SELECT vault_entry_id, actor FROM vault_timeline_event \
                 WHERE request_id = $request_id AND event_type = $event_type \
                 ORDER BY occurred_at ASC, event_id ASC",
            )
            .bind(("request_id", request_id.to_string()))
            .bind(("event_type", "witness_drafted"))
            .await
            .map_err(Self::map_surreal_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        for row in rows {
            let row =
                serde_json::from_value::<SurrealVaultTimelineRequestRow>(row).map_err(|err| {
                    DomainError::Validation(format!("invalid vault timeline row: {err}"))
                })?;
            if row.actor.user_id == actor_id {
                return Ok(Some(row.vault_entry_id));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
struct SurrealVaultEntryRow {
    vault_entry_id: String,
    author_id: String,
    author_username: String,
    state: String,
    created_at: String,
    updated_at: String,
    sealed_at: Option<String>,
    sealed_hash: Option<String>,
    encryption_key_id: Option<String>,
    attachment_refs: Vec<String>,
    wali: Vec<String>,
    payload: Option<serde_json::Value>,
    publish_target: Option<String>,
    retention_policy: Option<serde_json::Value>,
    audit: Option<serde_json::Value>,
    request_id: String,
    correlation_id: String,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Debug, Serialize)]
struct SurrealVaultEntryCreateRow {
    vault_entry_id: String,
    author_id: String,
    author_username: String,
    state: String,
    created_at: String,
    updated_at: String,
    sealed_at: Option<String>,
    sealed_hash: Option<String>,
    encryption_key_id: Option<String>,
    attachment_refs: Vec<String>,
    wali: Vec<String>,
    payload: Option<serde_json::Value>,
    publish_target: Option<String>,
    retention_policy: Option<serde_json::Value>,
    audit: Option<serde_json::Value>,
    request_id: String,
    correlation_id: String,
    event_hash: String,
    retention_tag: String,
}

#[derive(Debug, Deserialize)]
struct SurrealVaultTimelineRow {
    event_id: String,
    vault_entry_id: String,
    event_type: String,
    actor: VaultActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at: String,
    metadata: Option<serde_json::Value>,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SurrealVaultTimelineRequestRow {
    vault_entry_id: String,
    actor: VaultActorSnapshot,
}

#[derive(Debug, Serialize)]
struct SurrealVaultTimelineCreateRow {
    event_id: String,
    vault_entry_id: String,
    event_type: String,
    actor: VaultActorSnapshot,
    request_id: String,
    correlation_id: String,
    #[allow(dead_code)]
    #[serde(rename = "occurred_at")]
    occurred_at: String,
    metadata: Option<serde_json::Value>,
    event_hash: String,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct VaultEntryAuditPayload {
    vault_entry_id: String,
    author_id: String,
    author_username: String,
    state: String,
    created_at_ms: i64,
    updated_at_ms: i64,
    sealed_at_ms: Option<i64>,
    sealed_hash: Option<String>,
    encryption_key_id: Option<String>,
    attachment_refs: Vec<String>,
    wali: Vec<String>,
    payload: Option<serde_json::Value>,
    publish_target: Option<String>,
    retention_policy: Option<serde_json::Value>,
    audit: Option<serde_json::Value>,
    request_id: String,
    correlation_id: String,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct VaultTimelineAuditPayload {
    event_id: String,
    vault_entry_id: String,
    event_type: String,
    actor: VaultActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at_ms: i64,
    metadata: Option<serde_json::Value>,
    retention_tag: String,
}

fn vault_entry_retention_tag(vault_entry_id: &str) -> String {
    format!("vault_entry:{vault_entry_id}")
}

fn vault_timeline_retention_tag(
    vault_entry_id: &str,
    event_type: &VaultTimelineEventType,
) -> String {
    format!(
        "vault_timeline:{vault_entry_id}:{}",
        vault_timeline_event_type_to_string(event_type)
    )
}

fn vault_timeline_event_type_to_string(value: &VaultTimelineEventType) -> &'static str {
    match value {
        VaultTimelineEventType::WitnessDrafted => "witness_drafted",
        VaultTimelineEventType::WitnessSealed => "witness_sealed",
        VaultTimelineEventType::WitnessTrusteeAdded => "witness_trustee_added",
        VaultTimelineEventType::WitnessTrusteeRemoved => "witness_trustee_removed",
        VaultTimelineEventType::WitnessPublished => "witness_published",
        VaultTimelineEventType::WitnessRevoked => "witness_revoked",
        VaultTimelineEventType::WitnessExpired => "witness_expired",
    }
}

impl SurrealVaultRepository {
    fn vault_entry_audit_hash(
        row: &SurrealVaultEntryRow,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let payload = VaultEntryAuditPayload {
            vault_entry_id: row.vault_entry_id.clone(),
            author_id: row.author_id.clone(),
            author_username: row.author_username.clone(),
            state: row.state.clone(),
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
            sealed_at_ms: match &row.sealed_at {
                Some(value) => Some(Self::parse_datetime_ms(value)?),
                None => None,
            },
            sealed_hash: row.sealed_hash.clone(),
            encryption_key_id: row.encryption_key_id.clone(),
            attachment_refs: row.attachment_refs.clone(),
            wali: row.wali.clone(),
            payload: row.payload.clone(),
            publish_target: row.publish_target.clone(),
            retention_policy: row.retention_policy.clone(),
            audit: row.audit.clone(),
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }

    fn vault_timeline_audit_hash(
        row: &SurrealVaultTimelineRow,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let parsed_state = Self::parse_event_type(&row.event_type)?;
        let payload = VaultTimelineAuditPayload {
            event_id: row.event_id.clone(),
            vault_entry_id: row.vault_entry_id.clone(),
            event_type: Self::event_type_to_string(&parsed_state).to_string(),
            actor: row.actor.clone(),
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            occurred_at_ms: Self::parse_datetime_ms(&row.occurred_at)?,
            metadata: row.metadata.clone(),
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }
}

impl VaultRepository for SurrealVaultRepository {
    fn create_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<VaultEntry>> {
        let entry = entry.clone();
        let event = event.clone();
        let payload = match Self::to_entry_payload(&entry) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let event_payload = match Self::to_timeline_payload(&event) {
            Ok(event_payload) => event_payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let vault_entry_id = entry.vault_entry_id.clone();
        let client = self.client.clone();
        let author_id = entry.author_id.clone();
        let request_id = event.request_id.clone();
        let event_id = event.event_id.clone();
        Box::pin(async move {
            if let Some(existing_entry_id) =
                Self::resolve_entry_id_from_actor_request(&client, &author_id, &request_id).await?
            {
                if let Some(existing_entry) =
                    Self::get_by_entry_from_store(&client, &existing_entry_id).await?
                {
                    return Ok(existing_entry);
                }
            }
            if let Some(existing_entry) =
                Self::get_by_actor_request_from_store(&client, &author_id, &request_id).await?
            {
                return Ok(existing_entry);
            }

            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let event_payload = to_value(event_payload).map_err(|err| {
                DomainError::Validation(format!("invalid timeline payload: {err}"))
            })?;

            let mut response = client
                .query("CREATE type::record('vault_entry', $vault_entry_id) CONTENT $payload")
                .bind(("payload", payload))
                .bind(("vault_entry_id", vault_entry_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))?;
            let mut entries = Self::map_entry_rows(vec![row])?;
            let created = entries.pop().ok_or_else(|| {
                DomainError::Validation("create returned malformed row".to_string())
            })?;

            let mut timeline_response = match client
                .query(
                    "CREATE type::record('vault_timeline_event', $event_id) CONTENT $event_payload",
                )
                .bind(("event_id", event_id.clone()))
                .bind(("event_payload", event_payload))
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    let _ = client
                        .query("DELETE vault_entry WHERE vault_entry_id = $vault_entry_id")
                        .bind(("vault_entry_id", vault_entry_id.clone()))
                        .await;
                    return Err(Self::map_surreal_error(err));
                }
            };
            let _rows: Vec<Value> = timeline_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(created)
        })
    }

    fn update_entry(
        &self,
        entry: &VaultEntry,
        event: &VaultTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<VaultEntry>> {
        let entry = entry.clone();
        let event = event.clone();
        let payload = match Self::to_entry_payload(&entry) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let event_payload = match Self::to_timeline_payload(&event) {
            Ok(event_payload) => event_payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let vault_entry_id = entry.vault_entry_id.clone();
        let request_id = event.request_id.clone();
        let event_id = event.event_id.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let event_payload = to_value(event_payload).map_err(|err| {
                DomainError::Validation(format!("invalid timeline payload: {err}"))
            })?;

            let mut existing_by_request = client
                .query(
                    "SELECT * FROM vault_entry WHERE vault_entry_id = $vault_entry_id AND request_id = $request_id LIMIT 1",
                )
                .bind(("vault_entry_id", vault_entry_id.clone()))
                .bind(("request_id", request_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let existing_by_request_rows: Vec<Value> = existing_by_request
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if let Some(row) = existing_by_request_rows.into_iter().next() {
                let mut entries = Self::map_entry_rows(vec![row])?;
                return entries.pop().ok_or_else(|| {
                    DomainError::Validation("existing entry is malformed".to_string())
                });
            }

            let mut existing_entry = client
                .query("SELECT * FROM vault_entry WHERE vault_entry_id = $vault_entry_id LIMIT 1")
                .bind(("vault_entry_id", vault_entry_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let mut existing_entry_rows: Vec<Value> = existing_entry
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if existing_entry_rows.pop().is_none() {
                return Err(DomainError::NotFound);
            }

            let mut response = client
                .query("UPDATE type::record('vault_entry', $vault_entry_id) CONTENT $payload")
                .bind(("vault_entry_id", vault_entry_id.clone()))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("update returned no row".to_string()))?;
            let mut entries = Self::map_entry_rows(vec![row])?;
            let updated = entries.pop().ok_or_else(|| {
                DomainError::Validation("update returned malformed row".to_string())
            })?;

            let mut timeline_response = client
                .query(
                    "CREATE type::record('vault_timeline_event', $event_id) CONTENT $event_payload",
                )
                .bind(("event_id", event_id.clone()))
                .bind(("event_payload", event_payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let _rows: Vec<Value> = timeline_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(updated)
        })
    }

    fn delete_entry(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<bool>> {
        let vault_entry_id = vault_entry_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("DELETE vault_entry WHERE vault_entry_id = $vault_entry_id")
                .bind(("vault_entry_id", vault_entry_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let removed = if rows.is_empty() {
                false
            } else {
                rows.into_iter().next().is_some()
            };
            let _ = client
                .query("DELETE vault_timeline_event WHERE vault_entry_id = $vault_entry_id")
                .bind(("vault_entry_id", vault_entry_id))
                .await
                .map_err(Self::map_surreal_error)?;
            Ok(removed)
        })
    }

    fn get_entry(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM vault_entry WHERE vault_entry_id = $vault_entry_id LIMIT 1")
                .bind(("vault_entry_id", vault_entry_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let mut rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let Some(row) = rows.pop() else {
                return Ok(None);
            };
            let mut entries = Self::map_entry_rows(vec![row])?;
            Ok(entries.pop())
        })
    }

    fn list_by_author(
        &self,
        author_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<VaultEntry>>> {
        let author_id = author_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM vault_entry \
                     WHERE author_id = $author_id \
                     ORDER BY created_at DESC, vault_entry_id ASC",
                )
                .bind(("author_id", author_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut entries = Self::map_entry_rows(rows)?;
            entries.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.vault_entry_id.cmp(&left.vault_entry_id))
            });
            Ok(entries)
        })
    }

    fn list_timeline(
        &self,
        vault_entry_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<VaultTimelineEvent>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM vault_timeline_event \
                     WHERE vault_entry_id = $vault_entry_id \
                     ORDER BY occurred_at ASC, event_id ASC",
                )
                .bind(("vault_entry_id", vault_entry_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::map_timeline_rows(rows)?;
            events.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.event_id.cmp(&right.event_id))
            });
            Ok(events)
        })
    }

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let actor_id = actor_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let Some(vault_entry_id) =
                Self::resolve_entry_id_from_actor_request(&client, &actor_id, &request_id).await?
            else {
                return Self::get_by_actor_request_from_store(&client, &actor_id, &request_id)
                    .await;
            };
            Self::get_by_entry_from_store(&client, &vault_entry_id).await
        })
    }

    fn get_by_request(
        &self,
        vault_entry_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
        let vault_entry_id = vault_entry_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            Self::get_by_request_from_store(&client, &vault_entry_id, &request_id).await
        })
    }
}

#[derive(Default)]
pub struct InMemorySiagaRepository {
    by_id: Arc<RwLock<HashMap<String, SiagaBroadcast>>>,
    by_actor_request: Arc<RwLock<HashMap<(String, String), String>>>,
    by_request: Arc<RwLock<HashMap<(String, String), String>>>,
    timeline: Arc<RwLock<HashMap<String, VecDeque<SiagaTimelineEvent>>>>,
}

impl InMemorySiagaRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn actor_request_key(actor_id: &str, request_id: &str) -> (String, String) {
        (actor_id.to_string(), request_id.to_string())
    }

    fn broadcast_request_key(siaga_id: &str, request_id: &str) -> (String, String) {
        (siaga_id.to_string(), request_id.to_string())
    }
}

impl SiagaRepository for InMemorySiagaRepository {
    fn create_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<SiagaBroadcast>> {
        let broadcast = broadcast.clone();
        let event = event.clone();
        let by_id = self.by_id.clone();
        let by_actor_request = self.by_actor_request.clone();
        let by_request = self.by_request.clone();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            if let Some(existing_id) = by_actor_request
                .read()
                .await
                .get(&Self::actor_request_key(
                    &broadcast.author_id,
                    &event.request_id,
                ))
                .cloned()
            {
                let by_id = by_id.read().await;
                return by_id
                    .get(&existing_id)
                    .cloned()
                    .ok_or(DomainError::Conflict);
            }

            if by_id.read().await.contains_key(&broadcast.siaga_id) {
                return Err(DomainError::Conflict);
            }

            by_id
                .write()
                .await
                .insert(broadcast.siaga_id.clone(), broadcast.clone());
            by_actor_request.write().await.insert(
                Self::actor_request_key(&broadcast.author_id, &event.request_id),
                broadcast.siaga_id.clone(),
            );
            by_request.write().await.insert(
                Self::broadcast_request_key(&broadcast.siaga_id, &event.request_id),
                broadcast.siaga_id.clone(),
            );
            timeline
                .write()
                .await
                .entry(broadcast.siaga_id.clone())
                .or_default()
                .push_back(event);
            Ok(broadcast)
        })
    }

    fn update_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<SiagaBroadcast>> {
        let broadcast = broadcast.clone();
        let event = event.clone();
        let by_id = self.by_id.clone();
        let by_request = self.by_request.clone();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let request_key = Self::broadcast_request_key(&broadcast.siaga_id, &event.request_id);
            if by_request.read().await.contains_key(&request_key) {
                let by_id = by_id.read().await;
                return by_id
                    .get(&broadcast.siaga_id)
                    .cloned()
                    .ok_or(DomainError::Conflict);
            }

            if !by_id.read().await.contains_key(&broadcast.siaga_id) {
                return Err(DomainError::NotFound);
            }

            by_id
                .write()
                .await
                .insert(broadcast.siaga_id.clone(), broadcast.clone());
            by_request
                .write()
                .await
                .insert(request_key, broadcast.siaga_id.clone());
            timeline
                .write()
                .await
                .entry(broadcast.siaga_id.clone())
                .or_default()
                .push_back(event);
            Ok(broadcast)
        })
    }

    fn get_broadcast(
        &self,
        siaga_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let siaga_id = siaga_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let by_id = by_id.read().await;
            Ok(by_id.get(&siaga_id).cloned())
        })
    }

    fn list_by_scope(
        &self,
        scope_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<SiagaBroadcast>>> {
        let scope_id = scope_id.to_string();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let mut broadcasts: Vec<_> = by_id
                .read()
                .await
                .values()
                .filter(|broadcast| broadcast.scope_id == scope_id)
                .cloned()
                .collect();
            broadcasts.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.siaga_id.cmp(&left.siaga_id))
            });
            Ok(broadcasts)
        })
    }

    fn list_timeline(
        &self,
        siaga_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<SiagaTimelineEvent>>> {
        let siaga_id = siaga_id.to_string();
        let timeline = self.timeline.clone();
        Box::pin(async move {
            let mut events = timeline
                .read()
                .await
                .get(&siaga_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            events.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.event_id.cmp(&right.event_id))
            });
            Ok(events)
        })
    }

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let actor_id = actor_id.to_string();
        let request_id = request_id.to_string();
        let by_actor_request = self.by_actor_request.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let by_actor_request = by_actor_request.read().await;
            let Some(siaga_id) =
                by_actor_request.get(&Self::actor_request_key(&actor_id, &request_id))
            else {
                return Ok(None);
            };
            Ok(by_id.read().await.get(siaga_id).cloned())
        })
    }

    fn get_by_request(
        &self,
        siaga_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let siaga_id = siaga_id.to_string();
        let request_id = request_id.to_string();
        let by_request = self.by_request.clone();
        let by_id = self.by_id.clone();
        Box::pin(async move {
            let by_request = by_request.read().await;
            let Some(stored_id) = by_request.get(&(siaga_id.clone(), request_id)) else {
                return Ok(None);
            };
            Ok(by_id.read().await.get(stored_id).cloned())
        })
    }
}

#[derive(Clone)]
pub struct SurrealSiagaRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealSiagaRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn parse_datetime_ms(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn state_to_string(value: &SiagaState) -> &'static str {
        match value {
            SiagaState::Draft => "draft",
            SiagaState::Active => "active",
            SiagaState::Resolved => "resolved",
            SiagaState::Cancelled => "cancelled",
        }
    }

    fn parse_state(value: &str) -> DomainResult<SiagaState> {
        match value {
            "draft" => Ok(SiagaState::Draft),
            "active" => Ok(SiagaState::Active),
            "resolved" => Ok(SiagaState::Resolved),
            "cancelled" => Ok(SiagaState::Cancelled),
            _ => Err(DomainError::Validation(format!(
                "invalid siaga state '{value}'"
            ))),
        }
    }

    fn parse_event_type(value: &str) -> DomainResult<SiagaTimelineEventType> {
        match value {
            "siaga_broadcast_created" => Ok(SiagaTimelineEventType::SiagaBroadcastCreated),
            "siaga_broadcast_activated" => Ok(SiagaTimelineEventType::SiagaBroadcastActivated),
            "siaga_broadcast_updated" => Ok(SiagaTimelineEventType::SiagaBroadcastUpdated),
            "siaga_responder_joined" => Ok(SiagaTimelineEventType::SiagaResponderJoined),
            "siaga_responder_updated" => Ok(SiagaTimelineEventType::SiagaResponderUpdated),
            "siaga_broadcast_closed" => Ok(SiagaTimelineEventType::SiagaBroadcastClosed),
            "siaga_broadcast_cancelled" => Ok(SiagaTimelineEventType::SiagaBroadcastCancelled),
            _ => Err(DomainError::Validation(format!(
                "invalid siaga timeline event '{value}'"
            ))),
        }
    }

    fn event_type_to_string(value: &SiagaTimelineEventType) -> &'static str {
        match value {
            SiagaTimelineEventType::SiagaBroadcastCreated => "siaga_broadcast_created",
            SiagaTimelineEventType::SiagaBroadcastActivated => "siaga_broadcast_activated",
            SiagaTimelineEventType::SiagaBroadcastUpdated => "siaga_broadcast_updated",
            SiagaTimelineEventType::SiagaResponderJoined => "siaga_responder_joined",
            SiagaTimelineEventType::SiagaResponderUpdated => "siaga_responder_updated",
            SiagaTimelineEventType::SiagaBroadcastClosed => "siaga_broadcast_closed",
            SiagaTimelineEventType::SiagaBroadcastCancelled => "siaga_broadcast_cancelled",
        }
    }

    fn map_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }

    fn map_row_to_broadcast(row: SurrealSiagaBroadcastRow) -> DomainResult<SiagaBroadcast> {
        let siaga_id = row.siaga_id.clone();
        let retention_tag = row
            .retention_tag
            .clone()
            .unwrap_or_else(|| siaga_broadcast_retention_tag(&siaga_id));
        let event_hash = match row.event_hash {
            Some(event_hash) => event_hash,
            None => Self::siaga_broadcast_audit_hash(&row, &retention_tag).map_err(|err| {
                DomainError::Validation(format!(
                    "missing siaga broadcast event_hash for '{}' and recompute failed: {err}",
                    row.siaga_id
                ))
            })?,
        };
        Ok(SiagaBroadcast {
            siaga_id,
            scope_id: row.scope_id,
            author_id: row.author_id,
            author_username: row.author_username,
            emergency_type: row.emergency_type,
            severity: row.severity,
            location: row.location,
            title: row.title,
            text: row.text,
            state: Self::parse_state(&row.state)?,
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            responders: row.responders,
            closure: row.closure,
            event_hash,
            retention_tag,
        })
    }

    fn map_broadcast_rows(rows: Vec<Value>) -> DomainResult<Vec<SiagaBroadcast>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealSiagaBroadcastRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid siaga broadcast row: {err}"))
                    })
                    .and_then(Self::map_row_to_broadcast)
            })
            .collect()
    }

    fn map_timeline_row(row: SurrealSiagaTimelineRow) -> DomainResult<SiagaTimelineEvent> {
        let occurred_at_ms = Self::parse_datetime_ms(&row.occurred_at)?;
        let event_type = Self::parse_event_type(&row.event_type)?;
        let siaga_id = row.siaga_id.clone();
        let retention_tag = row
            .retention_tag
            .clone()
            .unwrap_or_else(|| siaga_timeline_retention_tag(&siaga_id, &event_type));
        let event_hash = match row.event_hash {
            Some(event_hash) => event_hash,
            None => Self::siaga_timeline_audit_hash(&row, &event_type, &retention_tag)
                .map_err(|err| {
                    DomainError::Validation(format!(
                        "missing siaga timeline event_hash for event '{}' and recompute failed: {err}",
                        row.event_id
                    ))
                })?,
        };
        Ok(SiagaTimelineEvent {
            event_id: row.event_id,
            siaga_id,
            event_type,
            actor: row.actor,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            occurred_at_ms,
            metadata: row.metadata,
            event_hash,
            retention_tag,
        })
    }

    fn map_timeline_rows(rows: Vec<Value>) -> DomainResult<Vec<SiagaTimelineEvent>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealSiagaTimelineRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid siaga timeline row: {err}"))
                    })
                    .and_then(Self::map_timeline_row)
            })
            .collect()
    }

    fn broadcast_payload_to_store(
        broadcast: &SiagaBroadcast,
    ) -> DomainResult<SurrealSiagaBroadcastCreateRow> {
        let created_at =
            OffsetDateTime::from_unix_timestamp_nanos(broadcast.created_at_ms as i128 * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid created_at_ms: {err}")))?;
        let updated_at =
            OffsetDateTime::from_unix_timestamp_nanos(broadcast.updated_at_ms as i128 * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid updated_at_ms: {err}")))?;
        Ok(SurrealSiagaBroadcastCreateRow {
            siaga_id: broadcast.siaga_id.clone(),
            scope_id: broadcast.scope_id.clone(),
            author_id: broadcast.author_id.clone(),
            author_username: broadcast.author_username.clone(),
            emergency_type: broadcast.emergency_type.clone(),
            severity: broadcast.severity as i64,
            location: broadcast.location.clone(),
            title: broadcast.title.clone(),
            text: broadcast.text.clone(),
            state: Self::state_to_string(&broadcast.state).to_string(),
            created_at: created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            updated_at: updated_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            request_id: broadcast.request_id.clone(),
            correlation_id: broadcast.correlation_id.clone(),
            responders: broadcast.responders.clone(),
            closure: broadcast.closure.clone(),
            event_hash: broadcast.event_hash.clone(),
            retention_tag: broadcast.retention_tag.clone(),
        })
    }

    fn timeline_payload_to_store(
        event: &SiagaTimelineEvent,
    ) -> DomainResult<SurrealSiagaTimelineCreateRow> {
        let occurred_at =
            OffsetDateTime::from_unix_timestamp_nanos(event.occurred_at_ms as i128 * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid occurred_at_ms: {err}")))?;
        Ok(SurrealSiagaTimelineCreateRow {
            siaga_id: event.siaga_id.clone(),
            event_id: event.event_id.clone(),
            event_type: Self::event_type_to_string(&event.event_type).to_string(),
            actor: event.actor.clone(),
            request_id: event.request_id.clone(),
            correlation_id: event.correlation_id.clone(),
            occurred_at: occurred_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            metadata: event.metadata.clone(),
            event_hash: event.event_hash.clone(),
            retention_tag: event.retention_tag.clone(),
        })
    }

    async fn get_from_broadcast_id(
        client: &Surreal<Client>,
        siaga_id: &str,
    ) -> DomainResult<Option<SiagaBroadcast>> {
        let mut response = client
            .query("SELECT * FROM siaga_broadcast WHERE siaga_id = $siaga_id LIMIT 1")
            .bind(("siaga_id", siaga_id.to_string()))
            .await
            .map_err(Self::map_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let mut broadcasts = Self::map_broadcast_rows(rows)?;
        Ok(broadcasts.pop())
    }

    async fn get_from_timeline_request(
        client: &Surreal<Client>,
        siaga_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<SiagaBroadcast>> {
        let mut response = client
            .query(
                "SELECT * FROM siaga_timeline_event \
                 WHERE siaga_id = $siaga_id AND request_id = $request_id LIMIT 1",
            )
            .bind(("siaga_id", siaga_id.to_string()))
            .bind(("request_id", request_id.to_string()))
            .await
            .map_err(Self::map_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        if rows.is_empty() {
            return Ok(None);
        }
        Self::get_from_broadcast_id(client, siaga_id).await
    }

    async fn resolve_broadcast_id_from_actor_request(
        client: &Surreal<Client>,
        actor_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<String>> {
        let mut response = client
            .query(
                "SELECT siaga_id FROM siaga_timeline_event \
                 WHERE event_type = $event_type \
                 AND request_id = $request_id \
                 AND actor.user_id = $actor_id \
                 ORDER BY occurred_at ASC, event_id ASC",
            )
            .bind(("event_type", "siaga_broadcast_created"))
            .bind(("request_id", request_id.to_string()))
            .bind(("actor_id", actor_id.to_string()))
            .await
            .map_err(Self::map_error)?;
        let rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let row = serde_json::from_value::<SurrealSiagaTimelineRequestRow>(row)
            .map_err(|err| DomainError::Validation(format!("invalid siaga timeline row: {err}")))?;
        Ok(Some(row.siaga_id))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealSiagaBroadcastCreateRow {
    siaga_id: String,
    scope_id: String,
    author_id: String,
    author_username: String,
    emergency_type: String,
    severity: i64,
    location: String,
    title: String,
    text: String,
    state: String,
    created_at: String,
    updated_at: String,
    request_id: String,
    correlation_id: String,
    responders: Vec<SiagaResponder>,
    closure: Option<SiagaClosure>,
    event_hash: String,
    retention_tag: String,
}

#[derive(Debug, Deserialize)]
struct SurrealSiagaBroadcastRow {
    siaga_id: String,
    scope_id: String,
    author_id: String,
    author_username: String,
    emergency_type: String,
    severity: u8,
    location: String,
    title: String,
    text: String,
    state: String,
    created_at: String,
    updated_at: String,
    request_id: String,
    correlation_id: String,
    responders: Vec<SiagaResponder>,
    closure: Option<SiagaClosure>,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Debug, Serialize)]
struct SurrealSiagaTimelineCreateRow {
    siaga_id: String,
    event_id: String,
    event_type: String,
    actor: SiagaActorSnapshot,
    request_id: String,
    correlation_id: String,
    #[allow(dead_code)]
    #[serde(rename = "occurred_at")]
    occurred_at: String,
    metadata: Option<serde_json::Value>,
    event_hash: String,
    retention_tag: String,
}

#[derive(Debug, Deserialize)]
struct SurrealSiagaTimelineRow {
    event_id: String,
    siaga_id: String,
    event_type: String,
    actor: SiagaActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at: String,
    metadata: Option<serde_json::Value>,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SurrealSiagaTimelineRequestRow {
    siaga_id: String,
}

#[derive(Clone, Serialize)]
struct SiagaBroadcastAuditPayload {
    siaga_id: String,
    scope_id: String,
    author_id: String,
    author_username: String,
    emergency_type: String,
    severity: u8,
    location: String,
    title: String,
    text: String,
    state: String,
    created_at_ms: i64,
    updated_at_ms: i64,
    request_id: String,
    correlation_id: String,
    responders: Vec<SiagaResponder>,
    closure: Option<SiagaClosure>,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct SiagaTimelineAuditPayload {
    event_id: String,
    siaga_id: String,
    event_type: String,
    actor: SiagaActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at_ms: i64,
    metadata: Option<serde_json::Value>,
    retention_tag: String,
}

fn siaga_broadcast_retention_tag(siaga_id: &str) -> String {
    format!("siaga_broadcast:{siaga_id}")
}

fn siaga_timeline_retention_tag(siaga_id: &str, event_type: &SiagaTimelineEventType) -> String {
    format!(
        "siaga_timeline:{siaga_id}:{}",
        match event_type {
            SiagaTimelineEventType::SiagaBroadcastCreated => "siaga_broadcast_created",
            SiagaTimelineEventType::SiagaBroadcastActivated => "siaga_broadcast_activated",
            SiagaTimelineEventType::SiagaBroadcastUpdated => "siaga_broadcast_updated",
            SiagaTimelineEventType::SiagaResponderJoined => "siaga_responder_joined",
            SiagaTimelineEventType::SiagaResponderUpdated => "siaga_responder_updated",
            SiagaTimelineEventType::SiagaBroadcastClosed => "siaga_broadcast_closed",
            SiagaTimelineEventType::SiagaBroadcastCancelled => "siaga_broadcast_cancelled",
        }
    )
}

impl SurrealSiagaRepository {
    fn siaga_broadcast_audit_hash(
        row: &SurrealSiagaBroadcastRow,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let payload = SiagaBroadcastAuditPayload {
            siaga_id: row.siaga_id.clone(),
            scope_id: row.scope_id.clone(),
            author_id: row.author_id.clone(),
            author_username: row.author_username.clone(),
            emergency_type: row.emergency_type.clone(),
            severity: row.severity,
            location: row.location.clone(),
            title: row.title.clone(),
            text: row.text.clone(),
            state: row.state.clone(),
            created_at_ms: Self::parse_datetime_ms(&row.created_at)?,
            updated_at_ms: Self::parse_datetime_ms(&row.updated_at)?,
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            responders: row.responders.clone(),
            closure: row.closure.clone(),
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }

    fn siaga_timeline_audit_hash(
        row: &SurrealSiagaTimelineRow,
        event_type: &SiagaTimelineEventType,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let payload = SiagaTimelineAuditPayload {
            event_id: row.event_id.clone(),
            siaga_id: row.siaga_id.clone(),
            event_type: Self::event_type_to_string(event_type).to_string(),
            actor: row.actor.clone(),
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            occurred_at_ms: Self::parse_datetime_ms(&row.occurred_at)?,
            metadata: row.metadata.clone(),
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }
}

impl SurrealSiagaRepository {
    async fn build_actor_request_query(
        client: &Surreal<Client>,
        actor_id: &str,
        request_id: &str,
    ) -> DomainResult<Option<SiagaBroadcast>> {
        let request_id = request_id.to_string();
        let actor_id = actor_id.to_string();

        let Some(siaga_id) =
            Self::resolve_broadcast_id_from_actor_request(client, &actor_id, &request_id).await?
        else {
            return Ok(None);
        };

        Self::get_broadcast_from_store(client, &siaga_id).await
    }

    async fn get_broadcast_from_store(
        client: &Surreal<Client>,
        siaga_id: &str,
    ) -> DomainResult<Option<SiagaBroadcast>> {
        let mut response = client
            .query("SELECT * FROM siaga_broadcast WHERE siaga_id = $siaga_id LIMIT 1")
            .bind(("siaga_id", siaga_id.to_string()))
            .await
            .map_err(Self::map_error)?;
        let mut rows: Vec<Value> = response
            .take(0)
            .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
        let Some(row) = rows.pop() else {
            return Ok(None);
        };
        let row = serde_json::from_value::<SurrealSiagaBroadcastRow>(row).map_err(|err| {
            DomainError::Validation(format!("invalid siaga broadcast row: {err}"))
        })?;
        Ok(Some(Self::map_row_to_broadcast(row)?))
    }
}

impl SiagaRepository for SurrealSiagaRepository {
    fn create_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<SiagaBroadcast>> {
        let broadcast = broadcast.clone();
        let event = event.clone();
        let payload = match Self::broadcast_payload_to_store(&broadcast) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let event_payload = match Self::timeline_payload_to_store(&event) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let siaga_id = broadcast.siaga_id.clone();
        let author_id = broadcast.author_id.clone();
        let request_id = event.request_id.clone();
        Box::pin(async move {
            if let Some(existing) =
                Self::build_actor_request_query(&client, &author_id, &request_id).await?
            {
                if let Some(existing) =
                    Self::get_broadcast_from_store(&client, &existing.siaga_id).await?
                {
                    return Ok(existing);
                }
            }

            let payload = to_value(payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga broadcast payload: {err}"))
            })?;
            let event_payload = to_value(event_payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga timeline payload: {err}"))
            })?;

            let mut response = client
                .query("CREATE type::record('siaga_broadcast', $siaga_id) CONTENT $payload")
                .bind(("siaga_id", siaga_id.clone()))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let Some(row) = rows.into_iter().next() else {
                return Err(DomainError::Validation(
                    "create returned no row".to_string(),
                ));
            };
            let created = serde_json::from_value::<SurrealSiagaBroadcastRow>(row)
                .map_err(|err| DomainError::Validation(format!("invalid create response: {err}")))
                .and_then(Self::map_row_to_broadcast)?;

            let mut timeline_response = match client
                .query(
                    "CREATE type::record('siaga_timeline_event', $event_id) CONTENT $event_payload",
                )
                .bind(("event_id", event.event_id.clone()))
                .bind(("event_payload", event_payload))
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    let _ = client
                        .query("DELETE siaga_broadcast WHERE siaga_id = $siaga_id")
                        .bind(("siaga_id", siaga_id.clone()))
                        .await;
                    return Err(Self::map_error(err));
                }
            };
            let _rows: Vec<Value> = timeline_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(created)
        })
    }

    fn update_broadcast(
        &self,
        broadcast: &SiagaBroadcast,
        event: &SiagaTimelineEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<SiagaBroadcast>> {
        let broadcast = broadcast.clone();
        let event = event.clone();
        let payload = match Self::broadcast_payload_to_store(&broadcast) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let event_payload = match Self::timeline_payload_to_store(&event) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let siaga_id = broadcast.siaga_id.clone();
        Box::pin(async move {
            if let Some(existing) =
                Self::get_from_timeline_request(&client, &siaga_id, &event.request_id).await?
            {
                return Ok(existing);
            }

            if Self::get_broadcast_from_store(&client, &siaga_id)
                .await?
                .is_none()
            {
                return Err(DomainError::NotFound);
            }

            let payload = to_value(payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga broadcast payload: {err}"))
            })?;
            let event_payload = to_value(event_payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga timeline payload: {err}"))
            })?;

            let mut response = client
                .query("UPDATE type::record('siaga_broadcast', $siaga_id) CONTENT $payload")
                .bind(("siaga_id", siaga_id.clone()))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let updated_row = rows.into_iter().next().ok_or_else(|| {
                DomainError::Validation("update returned no siaga row".to_string())
            })?;
            let updated = serde_json::from_value::<SurrealSiagaBroadcastRow>(updated_row)
                .map_err(|err| DomainError::Validation(format!("invalid siaga row: {err}")))
                .and_then(Self::map_row_to_broadcast)?;

            let mut timeline_response = client
                .query(
                    "CREATE type::record('siaga_timeline_event', $event_id) CONTENT $event_payload",
                )
                .bind(("event_id", event.event_id.clone()))
                .bind(("event_payload", event_payload))
                .await
                .map_err(Self::map_error)?;
            let _rows: Vec<Value> = timeline_response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(updated)
        })
    }

    fn get_broadcast(
        &self,
        siaga_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let siaga_id = siaga_id.to_string();
        let client = self.client.clone();
        Box::pin(async move { Self::get_broadcast_from_store(&client, &siaga_id).await })
    }

    fn list_by_scope(
        &self,
        scope_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<SiagaBroadcast>>> {
        let scope_id = scope_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM siaga_broadcast WHERE scope_id = $scope_id ORDER BY created_at DESC")
                .bind(("scope_id", scope_id))
                .await
                .map_err(Self::map_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut broadcasts = Self::map_broadcast_rows(rows)?;
            broadcasts.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.siaga_id.cmp(&left.siaga_id))
            });
            Ok(broadcasts)
        })
    }

    fn list_timeline(
        &self,
        siaga_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<SiagaTimelineEvent>>> {
        let siaga_id = siaga_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM siaga_timeline_event \
                     WHERE siaga_id = $siaga_id \
                     ORDER BY occurred_at ASC, event_id ASC",
                )
                .bind(("siaga_id", siaga_id))
                .await
                .map_err(Self::map_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::map_timeline_rows(rows)
        })
    }

    fn get_by_actor_request(
        &self,
        actor_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let actor_id = actor_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(
            async move { Self::build_actor_request_query(&client, &actor_id, &request_id).await },
        )
    }

    fn get_by_request(
        &self,
        siaga_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<SiagaBroadcast>>> {
        let siaga_id = siaga_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(
            async move { Self::get_from_timeline_request(&client, &siaga_id, &request_id).await },
        )
    }
}

#[derive(Default)]
pub struct InMemoryModerationRepository {
    content_by_id: Arc<RwLock<HashMap<String, ContentModeration>>>,
    decisions_by_id: Arc<RwLock<HashMap<String, ModerationDecision>>>,
    decisions_by_request: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl InMemoryModerationRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ModerationRepository for InMemoryModerationRepository {
    fn upsert_content_moderation(
        &self,
        content: &ContentModeration,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ContentModeration>> {
        let content = content.clone();
        let store = self.content_by_id.clone();
        Box::pin(async move {
            let mut items = store.write().await;
            items.insert(content.content_id.clone(), content.clone());
            Ok(content)
        })
    }

    fn get_content_moderation(
        &self,
        content_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ContentModeration>>> {
        let content_id = content_id.to_string();
        let store = self.content_by_id.clone();
        Box::pin(async move { Ok(store.read().await.get(&content_id).cloned()) })
    }

    fn list_content_by_status(
        &self,
        status: &str,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ContentModeration>>> {
        let status = status.to_string();
        let store = self.content_by_id.clone();
        Box::pin(async move {
            let mut items: Vec<_> = store
                .read()
                .await
                .values()
                .filter(|content| content.moderation_status.to_string() == status)
                .cloned()
                .collect();
            items.sort_by(|left, right| {
                left.decided_at_ms
                    .cmp(&right.decided_at_ms)
                    .then_with(|| left.content_id.cmp(&right.content_id))
            });
            items.truncate(limit);
            Ok(items)
        })
    }

    fn create_decision(
        &self,
        decision: &ModerationDecision,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ModerationDecision>> {
        let decision = decision.clone();
        let decisions = self.decisions_by_id.clone();
        let by_request = self.decisions_by_request.clone();
        Box::pin(async move {
            let mut decisions_by_id = decisions.write().await;
            if decisions_by_id.contains_key(&decision.decision_id) {
                return Err(DomainError::Conflict);
            }

            let mut by_request = by_request.write().await;
            let key = (decision.content_id.clone(), decision.request_id.clone());
            if by_request.contains_key(&key) {
                return Err(DomainError::Conflict);
            }

            by_request.insert(key, decision.decision_id.clone());
            decisions_by_id.insert(decision.decision_id.clone(), decision.clone());
            Ok(decision)
        })
    }

    fn get_decision_by_request(
        &self,
        content_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ModerationDecision>>> {
        let key = (content_id.to_string(), request_id.to_string());
        let by_request = self.decisions_by_request.clone();
        let decisions_by_id = self.decisions_by_id.clone();
        Box::pin(async move {
            let Some(decision_id) = by_request.read().await.get(&key).cloned() else {
                return Ok(None);
            };
            let decisions_by_id = decisions_by_id.read().await;
            Ok(decisions_by_id.get(&decision_id).cloned())
        })
    }

    fn list_decisions(
        &self,
        content_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ModerationDecision>>> {
        let content_id = content_id.to_string();
        let decisions = self.decisions_by_id.clone();
        Box::pin(async move {
            let mut decisions: Vec<_> = decisions
                .read()
                .await
                .values()
                .filter(|decision| decision.content_id == content_id)
                .cloned()
                .collect();
            decisions.sort_by(|left, right| {
                left.decided_at_ms
                    .cmp(&right.decided_at_ms)
                    .then_with(|| left.decision_id.cmp(&right.decision_id))
            });
            Ok(decisions)
        })
    }
}

#[derive(Clone)]
pub struct SurrealModerationRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealModerationRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(timestamp_ms: i64) -> DomainResult<String> {
        let dt = OffsetDateTime::from_unix_timestamp_nanos(timestamp_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(dt
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_timestamp(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
            DomainError::Validation(format!("invalid moderation datetime '{value}': {err}"))
        })?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn status_to_string(status: &ModerationStatus) -> String {
        match status {
            ModerationStatus::Processing => "processing".to_string(),
            ModerationStatus::UnderReview => "under_review".to_string(),
            ModerationStatus::Published => "published".to_string(),
            ModerationStatus::Rejected => "rejected".to_string(),
        }
    }

    fn action_to_string(action: &ModerationAction) -> String {
        match action {
            ModerationAction::PublishNow => "publish_now".to_string(),
            ModerationAction::PublishWithWarning => "publish_with_warning".to_string(),
            ModerationAction::HoldForReview => "hold_for_review".to_string(),
            ModerationAction::Block => "block".to_string(),
        }
    }

    fn parse_status(status: &str) -> DomainResult<ModerationStatus> {
        match status {
            "processing" => Ok(ModerationStatus::Processing),
            "under_review" => Ok(ModerationStatus::UnderReview),
            "published" => Ok(ModerationStatus::Published),
            "rejected" => Ok(ModerationStatus::Rejected),
            _ => Err(DomainError::Validation(format!(
                "invalid moderation status '{status}'"
            ))),
        }
    }

    fn parse_action(action: &str) -> DomainResult<ModerationAction> {
        match action {
            "publish_now" => Ok(ModerationAction::PublishNow),
            "publish_with_warning" => Ok(ModerationAction::PublishWithWarning),
            "hold_for_review" => Ok(ModerationAction::HoldForReview),
            "block" => Ok(ModerationAction::Block),
            _ => Err(DomainError::Validation(format!(
                "invalid moderation action '{action}'"
            ))),
        }
    }

    fn to_content_payload(
        content: &ContentModeration,
    ) -> DomainResult<SurrealModerationContentCreateRow> {
        let decided_at = Self::to_rfc3339(content.decided_at_ms)?;
        let hold_expires_at = content
            .hold_expires_at_ms
            .map(Self::to_rfc3339)
            .transpose()?;
        let appeal_window_until = content
            .appeal_window_until_ms
            .map(Self::to_rfc3339)
            .transpose()?;

        Ok(SurrealModerationContentCreateRow {
            content_id: content.content_id.clone(),
            content_type: content.content_type.clone(),
            author_id: content.author_id.clone(),
            author_username: content.author_username.clone(),
            moderation_status: Self::status_to_string(&content.moderation_status),
            moderation_action: Self::action_to_string(&content.moderation_action),
            reason_code: content.reason_code.clone(),
            confidence: content.confidence,
            decided_at,
            decided_by: content.decided_by.clone(),
            hold_expires_at,
            auto_release_if_no_action: content.auto_release_if_no_action,
            appeal_window_until,
            reasoning: content.reasoning.clone(),
            violations: content.violations.clone(),
            last_decision_id: content.last_decision_id.clone(),
            request_id: content.request_id.clone(),
            correlation_id: content.correlation_id.clone(),
            request_ts_ms: content.request_ts_ms,
            event_hash: content.event_hash.clone(),
            retention_tag: content.retention_tag.clone(),
        })
    }

    fn to_decision_payload(
        decision: &ModerationDecision,
    ) -> DomainResult<SurrealModerationDecisionCreateRow> {
        let decided_at = Self::to_rfc3339(decision.decided_at_ms)?;
        let hold_expires_at = decision
            .hold_expires_at_ms
            .map(Self::to_rfc3339)
            .transpose()?;
        let appeal_window_until = decision
            .appeal_window_until_ms
            .map(Self::to_rfc3339)
            .transpose()?;

        Ok(SurrealModerationDecisionCreateRow {
            decision_id: decision.decision_id.clone(),
            content_id: decision.content_id.clone(),
            content_type: decision.content_type.clone(),
            moderation_status: Self::status_to_string(&decision.moderation_status),
            moderation_action: Self::action_to_string(&decision.moderation_action),
            reason_code: decision.reason_code.clone(),
            confidence: decision.confidence,
            decided_at,
            actor: decision.actor.clone(),
            hold_expires_at,
            auto_release_if_no_action: decision.auto_release_if_no_action,
            appeal_window_until,
            reasoning: decision.reasoning.clone(),
            violations: decision.violations.clone(),
            request_id: decision.request_id.clone(),
            correlation_id: decision.correlation_id.clone(),
            event_hash: decision.event_hash.clone(),
            retention_tag: decision.retention_tag.clone(),
        })
    }

    fn map_content_row(row: SurrealModerationContentRow) -> DomainResult<ContentModeration> {
        let retention_tag = row
            .retention_tag
            .as_ref()
            .cloned()
            .unwrap_or_else(|| moderation_content_retention_tag(&row.content_id));
        let event_hash = match row.event_hash {
            Some(event_hash) => event_hash,
            None => Self::moderation_content_audit_hash(&row, &retention_tag).map_err(|err| {
                DomainError::Validation(format!(
                    "missing moderation content event_hash for '{}' and recompute failed: {err}",
                    row.content_id
                ))
            })?,
        };
        Ok(ContentModeration {
            content_id: row.content_id,
            content_type: row.content_type,
            author_id: row.author_id,
            author_username: row.author_username,
            moderation_status: Self::parse_status(&row.moderation_status)?,
            moderation_action: Self::parse_action(&row.moderation_action)?,
            reason_code: row.reason_code,
            confidence: row.confidence,
            decided_at_ms: Self::parse_timestamp(&row.decided_at)?,
            decided_by: row.decided_by,
            hold_expires_at_ms: row
                .hold_expires_at
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            auto_release_if_no_action: row.auto_release_if_no_action,
            violations: row.violations,
            appeal_window_until_ms: row
                .appeal_window_until
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            reasoning: row.reasoning,
            last_decision_id: row.last_decision_id,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            request_ts_ms: row.request_ts_ms,
            event_hash,
            retention_tag,
        })
    }

    fn map_decision_row(row: SurrealModerationDecisionRow) -> DomainResult<ModerationDecision> {
        let retention_tag =
            row.retention_tag.as_ref().cloned().unwrap_or_else(|| {
                moderation_decision_retention_tag(&row.content_id, &row.request_id)
            });
        let event_hash = match row.event_hash {
            Some(event_hash) => event_hash,
            None => Self::moderation_decision_audit_hash(&row, &retention_tag).map_err(|err| {
                DomainError::Validation(format!(
                    "missing moderation decision event_hash for decision '{}' and recompute failed: {err}",
                    row.decision_id
                ))
            })?,
        };
        Ok(ModerationDecision {
            decision_id: row.decision_id,
            content_id: row.content_id,
            content_type: row.content_type,
            moderation_status: Self::parse_status(&row.moderation_status)?,
            moderation_action: Self::parse_action(&row.moderation_action)?,
            reason_code: row.reason_code,
            confidence: row.confidence,
            decided_at_ms: Self::parse_timestamp(&row.decided_at)?,
            actor: row.actor,
            hold_expires_at_ms: row
                .hold_expires_at
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            auto_release_if_no_action: row.auto_release_if_no_action,
            appeal_window_until_ms: row
                .appeal_window_until
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            reasoning: row.reasoning,
            violations: row.violations,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            event_hash,
            retention_tag,
        })
    }

    fn decode_content_rows(rows: Vec<Value>) -> DomainResult<Vec<ContentModeration>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealModerationContentRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid content moderation row: {err}"))
                    })
                    .and_then(Self::map_content_row)
            })
            .collect()
    }

    fn decode_decision_rows(rows: Vec<Value>) -> DomainResult<Vec<ModerationDecision>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealModerationDecisionRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid moderation decision row: {err}"))
                    })
                    .and_then(Self::map_decision_row)
            })
            .collect()
    }

    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }

        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }
}

#[derive(Debug, Serialize)]
struct SurrealModerationContentCreateRow {
    content_id: String,
    content_type: Option<String>,
    author_id: String,
    author_username: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at: String,
    decided_by: String,
    hold_expires_at: Option<String>,
    auto_release_if_no_action: bool,
    appeal_window_until: Option<String>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    last_decision_id: Option<String>,
    request_id: String,
    correlation_id: String,
    request_ts_ms: i64,
    event_hash: String,
    retention_tag: String,
}

#[derive(Debug, Deserialize)]
struct SurrealModerationContentRow {
    content_id: String,
    content_type: Option<String>,
    author_id: String,
    author_username: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at: String,
    decided_by: String,
    hold_expires_at: Option<String>,
    auto_release_if_no_action: bool,
    appeal_window_until: Option<String>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    last_decision_id: Option<String>,
    request_id: String,
    correlation_id: String,
    request_ts_ms: i64,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Debug, Serialize)]
struct SurrealModerationDecisionCreateRow {
    decision_id: String,
    content_id: String,
    content_type: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at: String,
    actor: ModerationActorSnapshot,
    hold_expires_at: Option<String>,
    auto_release_if_no_action: bool,
    appeal_window_until: Option<String>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    request_id: String,
    correlation_id: String,
    event_hash: String,
    retention_tag: String,
}

#[derive(Debug, Deserialize)]
struct SurrealModerationDecisionRow {
    decision_id: String,
    content_id: String,
    content_type: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at: String,
    actor: ModerationActorSnapshot,
    hold_expires_at: Option<String>,
    auto_release_if_no_action: bool,
    appeal_window_until: Option<String>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    request_id: String,
    correlation_id: String,
    #[serde(default)]
    event_hash: Option<String>,
    #[serde(default)]
    retention_tag: Option<String>,
}

#[derive(Clone, Serialize)]
struct ModerationContentAuditPayload {
    content_id: String,
    content_type: Option<String>,
    author_id: String,
    author_username: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at_ms: i64,
    decided_by: String,
    hold_expires_at_ms: Option<i64>,
    auto_release_if_no_action: bool,
    appeal_window_until_ms: Option<i64>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    last_decision_id: Option<String>,
    request_id: String,
    correlation_id: String,
    request_ts_ms: i64,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct ModerationDecisionAuditPayload {
    decision_id: String,
    content_id: String,
    content_type: Option<String>,
    moderation_status: String,
    moderation_action: String,
    reason_code: Option<String>,
    confidence: f64,
    decided_at_ms: i64,
    actor: ModerationActorSnapshot,
    hold_expires_at_ms: Option<i64>,
    auto_release_if_no_action: bool,
    appeal_window_until_ms: Option<i64>,
    reasoning: Option<String>,
    violations: Vec<ModerationViolation>,
    request_id: String,
    correlation_id: String,
    retention_tag: String,
}

fn moderation_content_retention_tag(content_id: &str) -> String {
    format!("moderation_content:{content_id}")
}

fn moderation_decision_retention_tag(content_id: &str, request_id: &str) -> String {
    format!("moderation_decision:{content_id}:{request_id}")
}

impl SurrealModerationRepository {
    fn moderation_content_audit_hash(
        row: &SurrealModerationContentRow,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let payload = ModerationContentAuditPayload {
            content_id: row.content_id.clone(),
            content_type: row.content_type.clone(),
            author_id: row.author_id.clone(),
            author_username: row.author_username.clone(),
            moderation_status: row.moderation_status.clone(),
            moderation_action: row.moderation_action.clone(),
            reason_code: row.reason_code.clone(),
            confidence: row.confidence,
            decided_at_ms: Self::parse_timestamp(&row.decided_at)?,
            decided_by: row.decided_by.clone(),
            hold_expires_at_ms: row
                .hold_expires_at
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            auto_release_if_no_action: row.auto_release_if_no_action,
            appeal_window_until_ms: row
                .appeal_window_until
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            reasoning: row.reasoning.clone(),
            violations: row.violations.clone(),
            last_decision_id: row.last_decision_id.clone(),
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            request_ts_ms: row.request_ts_ms,
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }

    fn moderation_decision_audit_hash(
        row: &SurrealModerationDecisionRow,
        retention_tag: &str,
    ) -> DomainResult<String> {
        let payload = ModerationDecisionAuditPayload {
            decision_id: row.decision_id.clone(),
            content_id: row.content_id.clone(),
            content_type: row.content_type.clone(),
            moderation_status: row.moderation_status.clone(),
            moderation_action: row.moderation_action.clone(),
            reason_code: row.reason_code.clone(),
            confidence: row.confidence,
            decided_at_ms: Self::parse_timestamp(&row.decided_at)?,
            actor: row.actor.clone(),
            hold_expires_at_ms: row
                .hold_expires_at
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            auto_release_if_no_action: row.auto_release_if_no_action,
            appeal_window_until_ms: row
                .appeal_window_until
                .as_deref()
                .map(Self::parse_timestamp)
                .transpose()?,
            reasoning: row.reasoning.clone(),
            violations: row.violations.clone(),
            request_id: row.request_id.clone(),
            correlation_id: row.correlation_id.clone(),
            retention_tag: retention_tag.to_string(),
        };
        gotong_domain::util::immutable_event_hash(&payload)
    }
}

impl ModerationRepository for SurrealModerationRepository {
    fn upsert_content_moderation(
        &self,
        content: &ContentModeration,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ContentModeration>> {
        let payload = match Self::to_content_payload(content) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let content_id = content.content_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("UPSERT type::record('content_moderation', $content_id) CONTENT $payload")
                .bind(("content_id", content_id))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("upsert returned no row".to_string()))?;
            Self::decode_content_rows(vec![row])?
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("upsert returned malformed row".to_string()))
        })
    }

    fn get_content_moderation(
        &self,
        content_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ContentModeration>>> {
        let content_id = content_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM type::record('content_moderation', $content_id)")
                .bind(("content_id", content_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let mut rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if rows.is_empty() {
                return Ok(None);
            }
            let row = rows.remove(0);
            Ok(Some(
                Self::decode_content_rows(vec![row])?
                    .into_iter()
                    .next()
                    .ok_or(DomainError::Validation("invalid row shape".to_string()))?,
            ))
        })
    }

    fn list_content_by_status(
        &self,
        status: &str,
        limit: usize,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ContentModeration>>> {
        let status = status.to_string();
        let limit = limit as i64;
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM content_moderation \
                     WHERE moderation_status = $status \
                     ORDER BY decided_at ASC, content_id ASC LIMIT $limit",
                )
                .bind(("status", status))
                .bind(("limit", limit))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_content_rows(rows)
        })
    }

    fn create_decision(
        &self,
        decision: &ModerationDecision,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ModerationDecision>> {
        let payload = match Self::to_decision_payload(decision) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let decision_id = decision.decision_id.clone();
        let content_id = decision.content_id.clone();
        let request_id = decision.request_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let mut existing = client
                .query(
                    "SELECT decision_id FROM moderation_decision \
                     WHERE content_id = $content_id AND request_id = $request_id LIMIT 1",
                )
                .bind(("content_id", content_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = existing
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if !rows.is_empty() {
                return Err(DomainError::Conflict);
            }

            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE type::record('moderation_decision', $decision_id) CONTENT $payload")
                .bind(("decision_id", decision_id))
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))?;
            Self::decode_decision_rows(vec![row])?
                .into_iter()
                .next()
                .ok_or_else(|| DomainError::Validation("create returned malformed row".to_string()))
        })
    }

    fn get_decision_by_request(
        &self,
        content_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ModerationDecision>>> {
        let content_id = content_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM moderation_decision \
                     WHERE content_id = $content_id AND request_id = $request_id LIMIT 1",
                )
                .bind(("content_id", content_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if rows.is_empty() {
                return Ok(None);
            }
            let mut decisions = Self::decode_decision_rows(rows)?;
            Ok(decisions.pop())
        })
    }

    fn list_decisions(
        &self,
        content_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ModerationDecision>>> {
        let content_id = content_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM moderation_decision \
                     WHERE content_id = $content_id ORDER BY decided_at ASC, decision_id ASC",
                )
                .bind(("content_id", content_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_decision_rows(rows)
        })
    }
}

#[derive(Default)]
pub struct InMemoryChatRepository {
    threads: Arc<RwLock<HashMap<String, ChatThread>>>,
    members: Arc<RwLock<HashMap<(String, String), ChatMember>>>,
    messages: Arc<RwLock<HashMap<(String, String), ChatMessage>>>,
    message_by_request: Arc<RwLock<HashMap<(String, String), String>>>,
    cursors: Arc<RwLock<HashMap<(String, String), ChatReadCursor>>>,
    events: Arc<RwLock<HashMap<(String, String), ChatDeliveryEvent>>>,
}

impl InMemoryChatRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn request_key(thread_id: &str, request_id: &str) -> (String, String) {
        (thread_id.to_string(), request_id.to_string())
    }

    fn list_active_members_for_thread<'a>(
        members: tokio::sync::RwLockReadGuard<'a, HashMap<(String, String), ChatMember>>,
        thread_id: &str,
    ) -> Vec<ChatMember> {
        members
            .values()
            .filter(|member| member.thread_id == thread_id && member.left_at_ms.is_none())
            .cloned()
            .collect()
    }
}

impl ChatRepositoryPort for InMemoryChatRepository {
    fn create_thread(
        &self,
        thread: &ChatThread,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatThread>> {
        let thread = thread.clone();
        let threads = self.threads.clone();
        Box::pin(async move {
            let mut threads = threads.write().await;
            if threads.contains_key(&thread.thread_id) {
                return Err(DomainError::Conflict);
            }
            threads.insert(thread.thread_id.clone(), thread.clone());
            Ok(thread)
        })
    }

    fn get_thread(
        &self,
        thread_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatThread>>> {
        let thread_id = thread_id.to_string();
        let threads = self.threads.clone();
        Box::pin(async move {
            let threads = threads.read().await;
            Ok(threads.get(&thread_id).cloned())
        })
    }

    fn list_threads_by_scope(
        &self,
        query: &ChatThreadQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatThread>>> {
        let query = query.clone();
        let threads = self.threads.clone();
        let members = self.members.clone();
        Box::pin(async move {
            let mut output: Vec<_> = threads
                .read()
                .await
                .values()
                .filter(|thread| {
                    if let Some(scope_id) = &query.scope_id {
                        thread.scope_id == *scope_id
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            if let Some(actor_id) = query.actor_id {
                let active_members = members.read().await;
                let allowed: Vec<String> = active_members
                    .iter()
                    .filter(|((_, member_user), member)| {
                        member_user == &actor_id && member.left_at_ms.is_none()
                    })
                    .map(|((thread_id, _), _)| thread_id.clone())
                    .collect();
                output.retain(|thread| {
                    thread.privacy_level == "public" || allowed.contains(&thread.thread_id)
                });
            }

            output.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
            Ok(output)
        })
    }

    fn list_threads_by_user(
        &self,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatThreadWithMembers>>> {
        let user_id = user_id.to_string();
        let members = self.members.clone();
        let threads = self.threads.clone();
        Box::pin(async move {
            let members = members.read().await;
            let thread_ids: Vec<String> = members
                .iter()
                .filter(|((_, member_user), member)| {
                    member_user == &user_id && member.left_at_ms.is_none()
                })
                .map(|((thread_id, _), _)| thread_id.clone())
                .collect();
            let threads = threads.read().await;
            let mut output = Vec::new();
            for thread_id in thread_ids {
                let Some(thread) = threads.get(&thread_id).cloned() else {
                    continue;
                };
                let member_count = members
                    .values()
                    .filter(|member| member.thread_id == thread_id && member.left_at_ms.is_none())
                    .count();
                output.push(ChatThreadWithMembers {
                    thread,
                    member_count,
                });
            }
            output.sort_by(|a, b| b.thread.created_at_ms.cmp(&a.thread.created_at_ms));
            Ok(output)
        })
    }

    fn create_member(
        &self,
        member: &ChatMember,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatMember>> {
        let member = member.clone();
        let members = self.members.clone();
        Box::pin(async move {
            let mut members = members.write().await;
            let key = (member.thread_id.clone(), member.user_id.clone());
            if let Some(existing) = members.get(&key) {
                if existing.left_at_ms.is_none() {
                    return Err(DomainError::Conflict);
                }
            }
            members.insert(key, member.clone());
            Ok(member)
        })
    }

    fn list_members(
        &self,
        thread_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatMember>>> {
        let thread_id = thread_id.to_string();
        let members = self.members.clone();
        Box::pin(async move {
            let members = members.read().await;
            let mut members: Vec<_> = Self::list_active_members_for_thread(members, &thread_id);
            members.sort_by(|a, b| {
                a.joined_at_ms
                    .cmp(&b.joined_at_ms)
                    .then_with(|| a.user_id.cmp(&b.user_id))
            });
            Ok(members)
        })
    }

    fn get_member(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMember>>> {
        let key = (thread_id.to_string(), user_id.to_string());
        let members = self.members.clone();
        Box::pin(async move {
            let members = members.read().await;
            Ok(members.get(&key).cloned())
        })
    }

    fn create_message(
        &self,
        message: &ChatMessage,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatMessage>> {
        let message = message.clone();
        let messages = self.messages.clone();
        let by_request = self.message_by_request.clone();
        Box::pin(async move {
            let key = Self::request_key(&message.thread_id, &message.request_id);
            let mut by_request = by_request.write().await;
            if let Some(message_id) = by_request.get(&key) {
                let messages = messages.read().await;
                if let Some(existing) =
                    messages.get(&(message.thread_id.clone(), message_id.clone()))
                {
                    return Ok(existing.clone());
                }
                by_request.remove(&key);
            }

            let mut messages = messages.write().await;
            if messages.contains_key(&(message.thread_id.clone(), message.message_id.clone())) {
                return Err(DomainError::Conflict);
            }
            messages.insert(
                (message.thread_id.clone(), message.message_id.clone()),
                message.clone(),
            );
            by_request.insert(key, message.message_id.clone());
            Ok(message)
        })
    }

    fn get_message(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
        let message_id = message_id.to_string();
        let thread_id = thread_id.to_string();
        let messages = self.messages.clone();
        Box::pin(async move {
            let messages = messages.read().await;
            Ok(messages.get(&(thread_id, message_id)).cloned())
        })
    }

    fn get_message_by_request_id(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
        let thread_id = thread_id.to_string();
        let request_id = request_id.to_string();
        let messages = self.messages.clone();
        let by_request = self.message_by_request.clone();
        Box::pin(async move {
            let by_request = by_request.read().await;
            let Some(message_id) = by_request.get(&(thread_id.clone(), request_id)) else {
                return Ok(None);
            };
            let messages = messages.read().await;
            Ok(messages.get(&(thread_id, message_id.clone())).cloned())
        })
    }

    fn list_messages(
        &self,
        thread_id: &str,
        cursor: &MessageCatchup,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatMessage>>> {
        let thread_id = thread_id.to_string();
        let cursor = cursor.clone();
        let messages = self.messages.clone();
        Box::pin(async move {
            let mut messages: Vec<_> = messages
                .read()
                .await
                .values()
                .filter(|message| message.thread_id == thread_id)
                .filter(|message| {
                    cursor.since_created_at_ms.is_none_or(|threshold| {
                        message.created_at_ms > threshold
                            || cursor.since_message_id.as_ref().is_none_or(|cursor_id| {
                                message.created_at_ms != threshold
                                    || message.message_id.as_str() > cursor_id.as_str()
                            })
                    })
                })
                .cloned()
                .collect();
            messages.sort_by(|a, b| {
                a.created_at_ms
                    .cmp(&b.created_at_ms)
                    .then_with(|| a.message_id.cmp(&b.message_id))
            });
            if cursor.since_created_at_ms.is_none() && cursor.since_message_id.is_none() {
                if messages.len() > cursor.limit {
                    let start = messages.len().saturating_sub(cursor.limit);
                    messages = messages.split_off(start);
                }
            } else {
                messages.truncate(cursor.limit);
            }
            Ok(messages)
        })
    }

    fn set_read_cursor(
        &self,
        cursor: &ChatReadCursor,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatReadCursor>> {
        let cursor = cursor.clone();
        let cursors = self.cursors.clone();
        Box::pin(async move {
            let mut cursors = cursors.write().await;
            cursors.insert(
                (cursor.thread_id.clone(), cursor.user_id.clone()),
                cursor.clone(),
            );
            Ok(cursor)
        })
    }

    fn get_read_cursor(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatReadCursor>>> {
        let key = (thread_id.to_string(), user_id.to_string());
        let cursors = self.cursors.clone();
        Box::pin(async move {
            let cursors = cursors.read().await;
            Ok(cursors.get(&key).cloned())
        })
    }

    fn create_delivery_event(
        &self,
        event: &ChatDeliveryEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatDeliveryEvent>> {
        let event = event.clone();
        let events = self.events.clone();
        Box::pin(async move {
            let key = Self::request_key(&event.thread_id, &event.request_id);
            let mut events = events.write().await;
            if events.contains_key(&key) {
                return Err(DomainError::Conflict);
            }
            events.insert(key, event.clone());
            Ok(event)
        })
    }

    fn get_delivery_event_by_request(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatDeliveryEvent>>> {
        let key = Self::request_key(thread_id, request_id);
        let events = self.events.clone();
        Box::pin(async move {
            let events = events.read().await;
            Ok(events.get(&key).cloned())
        })
    }
}

#[derive(Clone)]
pub struct SurrealChatRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealChatRepository {
    pub fn with_client(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn new(db_config: &DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&db_config.endpoint).await?;
        db.signin(Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await?;
        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await?;
        Ok(Self {
            client: Arc::new(db),
        })
    }

    fn to_rfc3339(created_at_ms: i64) -> DomainResult<String> {
        let instant = OffsetDateTime::from_unix_timestamp_nanos(created_at_ms as i128 * 1_000_000)
            .map_err(|err| DomainError::Validation(format!("invalid timestamp: {err}")))?;
        Ok(instant
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()))
    }

    fn parse_datetime(value: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(value, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid datetime: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn decode_thread_row(rows: Vec<Value>) -> DomainResult<Vec<ChatThread>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealChatThreadRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid chat thread row: {err}"))
                    })
                    .and_then(Self::map_chat_thread_row)
            })
            .collect()
    }

    fn map_chat_thread_row(row: SurrealChatThreadRow) -> DomainResult<ChatThread> {
        Ok(ChatThread {
            thread_id: row.thread_id,
            scope_id: row.scope_id,
            created_by: row.created_by,
            privacy_level: row.privacy_level,
            created_at_ms: Self::parse_datetime(&row.created_at)?,
            updated_at_ms: Self::parse_datetime(&row.updated_at)?,
        })
    }

    fn decode_member_row(rows: Vec<Value>) -> DomainResult<Vec<ChatMember>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealChatMemberRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid chat member row: {err}"))
                    })
                    .and_then(Self::map_chat_member_row)
            })
            .collect()
    }

    fn map_chat_member_row(row: SurrealChatMemberRow) -> DomainResult<ChatMember> {
        Ok(ChatMember {
            thread_id: row.thread_id,
            user_id: row.user_id,
            role: row.role,
            joined_at_ms: Self::parse_datetime(&row.joined_at)?,
            left_at_ms: row
                .left_at
                .as_deref()
                .map(Self::parse_datetime)
                .transpose()?,
            mute_until_ms: row
                .mute_until
                .as_deref()
                .map(Self::parse_datetime)
                .transpose()?,
        })
    }

    fn decode_message_row(rows: Vec<Value>) -> DomainResult<Vec<ChatMessage>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealChatMessageRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid chat message row: {err}"))
                    })
                    .and_then(Self::map_chat_message_row)
            })
            .collect()
    }

    fn map_chat_message_row(row: SurrealChatMessageRow) -> DomainResult<ChatMessage> {
        Ok(ChatMessage {
            thread_id: row.thread_id,
            message_id: row.message_id,
            author_id: row.author_id,
            body: row.body,
            attachments: row.attachments,
            created_at_ms: Self::parse_datetime(&row.created_at)?,
            edited_at_ms: row
                .edited_at
                .as_deref()
                .map(Self::parse_datetime)
                .transpose()?,
            deleted_at_ms: row
                .deleted_at
                .as_deref()
                .map(Self::parse_datetime)
                .transpose()?,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
        })
    }

    fn decode_read_cursor_row(rows: Vec<Value>) -> DomainResult<Vec<ChatReadCursor>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealChatReadCursorRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid chat read cursor row: {err}"))
                    })
                    .and_then(Self::map_chat_read_cursor_row)
            })
            .collect()
    }

    fn map_chat_read_cursor_row(row: SurrealChatReadCursorRow) -> DomainResult<ChatReadCursor> {
        Ok(ChatReadCursor {
            thread_id: row.thread_id,
            user_id: row.user_id,
            last_read_message_id: row.last_read_message_id,
            last_read_at_ms: Self::parse_datetime(&row.last_read_at)?,
        })
    }

    fn decode_delivery_event_row(rows: Vec<Value>) -> DomainResult<Vec<ChatDeliveryEvent>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealChatDeliveryEventRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid chat delivery event row: {err}"))
                    })
                    .and_then(Self::map_chat_delivery_event_row)
            })
            .collect()
    }

    fn map_chat_delivery_event_row(
        row: SurrealChatDeliveryEventRow,
    ) -> DomainResult<ChatDeliveryEvent> {
        Ok(ChatDeliveryEvent {
            event_id: row.event_id,
            thread_id: row.thread_id,
            message_id: row.message_id,
            event_type: row.event_type,
            occurred_at_ms: Self::parse_datetime(&row.occurred_at)?,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatThreadRow {
    thread_id: String,
    scope_id: String,
    created_by: String,
    privacy_level: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatMemberRow {
    thread_id: String,
    user_id: String,
    role: ChatMemberRole,
    joined_at: String,
    left_at: Option<String>,
    mute_until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatMemberCreateRow {
    thread_id: String,
    user_id: String,
    role: ChatMemberRole,
    joined_at: String,
    left_at: Option<String>,
    mute_until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatMessageRow {
    thread_id: String,
    message_id: String,
    author_id: String,
    body: String,
    attachments: Vec<serde_json::Value>,
    created_at: String,
    edited_at: Option<String>,
    deleted_at: Option<String>,
    request_id: String,
    correlation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatMessageCreateRow {
    thread_id: String,
    message_id: String,
    author_id: String,
    body: String,
    attachments: Vec<serde_json::Value>,
    created_at: String,
    edited_at: Option<String>,
    deleted_at: Option<String>,
    request_id: String,
    correlation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatReadCursorRow {
    thread_id: String,
    user_id: String,
    last_read_message_id: String,
    last_read_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatReadCursorCreateRow {
    thread_id: String,
    user_id: String,
    last_read_message_id: String,
    last_read_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatDeliveryEventRow {
    event_id: String,
    thread_id: String,
    message_id: String,
    event_type: String,
    occurred_at: String,
    request_id: String,
    correlation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealChatDeliveryEventCreateRow {
    event_id: String,
    thread_id: String,
    message_id: String,
    event_type: String,
    occurred_at: String,
    request_id: String,
    correlation_id: String,
}

impl SurrealChatRepository {
    fn map_surreal_error(err: surrealdb::Error) -> DomainError {
        let error_message = err.to_string().to_lowercase();
        if error_message.contains("already exists")
            || error_message.contains("duplicate")
            || error_message.contains("unique")
            || error_message.contains("conflict")
        {
            return DomainError::Conflict;
        }
        DomainError::Validation(format!("surreal query failed: {error_message}"))
    }
}

impl ChatRepositoryPort for SurrealChatRepository {
    fn create_thread(
        &self,
        thread: &ChatThread,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatThread>> {
        let created_at = match Self::to_rfc3339(thread.created_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let updated_at = match Self::to_rfc3339(thread.updated_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let thread_value = thread.clone();
        let thread_id = thread_value.thread_id.clone();
        let scope_id = thread_value.scope_id.clone();
        let created_by = thread_value.created_by.clone();
        let privacy_level = thread_value.privacy_level.clone();
        Box::pin(async move {
            let response = client
                .query(
                    "CREATE chat_thread CONTENT {\n\
                        thread_id: $thread_id,\n\
                        scope_id: $scope_id,\n\
                        created_by: $created_by,\n\
                        privacy_level: $privacy_level,\n\
                        created_at: <datetime>$created_at,\n\
                        updated_at: <datetime>$updated_at\n\
                    };",
                )
                .bind(("thread_id", thread_id))
                .bind(("scope_id", scope_id))
                .bind(("created_by", created_by))
                .bind(("privacy_level", privacy_level))
                .bind(("created_at", created_at))
                .bind(("updated_at", updated_at))
                .await
                .map_err(Self::map_surreal_error)?;
            response.check().map_err(Self::map_surreal_error)?;
            Ok(thread_value)
        })
    }

    fn get_thread(
        &self,
        thread_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatThread>>> {
        let thread_id = thread_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        scope_id,\n\
                        created_by,\n\
                        privacy_level,\n\
                        type::string(created_at) AS created_at,\n\
                        type::string(updated_at) AS updated_at\n\
                     FROM chat_thread\n\
                     WHERE thread_id = $thread_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_thread_row(rows)?.into_iter().next())
        })
    }

    fn list_threads_by_scope(
        &self,
        query: &ChatThreadQuery,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatThread>>> {
        let query = query.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let mut conditions = Vec::<String>::new();
            let actor_id = query.actor_id.clone();
            if query.scope_id.is_some() {
                conditions.push("scope_id = $scope_id".to_string());
            }
            if actor_id.is_some() {
                conditions.push(
                "(privacy_level = \"public\" OR thread_id IN (SELECT thread_id FROM chat_member WHERE user_id = $actor_id AND left_at IS NONE))"
                    .to_string(),
            );
            }

            let mut query_sql = String::from(
                "SELECT\n\
                    thread_id,\n\
                    scope_id,\n\
                    created_by,\n\
                    privacy_level,\n\
                    type::string(created_at) AS created_at,\n\
                    type::string(updated_at) AS updated_at\n\
                 FROM chat_thread",
            );
            if !conditions.is_empty() {
                query_sql.push_str(" WHERE ");
                query_sql.push_str(&conditions.join(" AND "));
            }
            query_sql.push_str(" ORDER BY created_at DESC, thread_id DESC");

            let mut query_handle = client.query(&query_sql);
            if let Some(scope_id) = query.scope_id {
                query_handle = query_handle.bind(("scope_id", scope_id));
            }
            if let Some(actor_id) = actor_id {
                query_handle = query_handle.bind(("actor_id", actor_id));
            }
            let mut response = query_handle.await.map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_thread_row(rows)
        })
    }

    fn list_threads_by_user(
        &self,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatThreadWithMembers>>> {
        let user_id = user_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        scope_id,\n\
                        created_by,\n\
                        privacy_level,\n\
                        type::string(created_at) AS created_at,\n\
                        type::string(updated_at) AS updated_at\n\
                     FROM chat_thread\n\
                     WHERE thread_id IN (\n\
                        SELECT thread_id FROM chat_member\n\
                        WHERE user_id = $user_id AND left_at IS NONE\n\
                     )\n\
                     ORDER BY created_at DESC, thread_id DESC",
                )
                .bind(("user_id", user_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let threads = Self::decode_thread_row(rows)?;
            let mut result = Vec::new();
            for thread in threads {
                let mut count_response = client
                    .query("SELECT count() AS count FROM chat_member WHERE thread_id = $thread_id AND left_at IS NONE")
                    .bind(("thread_id", thread.thread_id.clone()))
                    .await
                    .map_err(Self::map_surreal_error)?;
                let count_rows: Vec<Value> = count_response.take(0).map_err(|err| {
                    DomainError::Validation(format!("invalid query result: {err}"))
                })?;
                let member_count: usize = count_rows
                    .first()
                    .and_then(|row| row.get("count"))
                    .and_then(|value| value.as_u64())
                    .and_then(|count| usize::try_from(count).ok())
                    .or_else(|| {
                        count_rows
                            .first()?
                            .get("count")
                            .and_then(|value| value.as_i64())
                            .and_then(|count| usize::try_from(count).ok())
                    })
                    .unwrap_or_default();
                result.push(ChatThreadWithMembers {
                    thread,
                    member_count,
                });
            }
            Ok(result)
        })
    }

    fn create_member(
        &self,
        member: &ChatMember,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatMember>> {
        let joined_at = match Self::to_rfc3339(member.joined_at_ms) {
            Ok(value) => value,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let left_at = match member
            .left_at_ms
            .as_ref()
            .map(|value| Self::to_rfc3339(*value))
        {
            Some(Ok(value)) => Some(value),
            Some(Err(err)) => return Box::pin(async move { Err(err) }),
            None => None,
        };
        let mute_until = match member
            .mute_until_ms
            .as_ref()
            .map(|value| Self::to_rfc3339(*value))
        {
            Some(Ok(value)) => Some(value),
            Some(Err(err)) => return Box::pin(async move { Err(err) }),
            None => None,
        };
        let payload = SurrealChatMemberCreateRow {
            thread_id: member.thread_id.clone(),
            user_id: member.user_id.clone(),
            role: member.role.clone(),
            joined_at,
            left_at,
            mute_until,
        };
        let member_value = member.clone();
        let thread_id = member_value.thread_id.clone();
        let user_id = member_value.user_id.clone();
        let role = match member.role {
            ChatMemberRole::Owner => "owner",
            ChatMemberRole::Admin => "admin",
            ChatMemberRole::Member => "member",
        }
        .to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut existing_active = client
                .query(
                    "SELECT thread_id FROM chat_member \
                     WHERE thread_id = $thread_id AND user_id = $user_id AND left_at IS NONE LIMIT 1",
                )
                .bind(("thread_id", thread_id.clone()))
                .bind(("user_id", user_id.clone()))
                .await
                .map_err(Self::map_surreal_error)?;
            let existing: Vec<Value> = existing_active
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            if !existing.is_empty() {
                return Err(DomainError::Conflict);
            }

            let response = client
                .query(
                    "CREATE chat_member CONTENT {\n\
                        thread_id: $thread_id,\n\
                        user_id: $user_id,\n\
                        role: $role,\n\
                        joined_at: <datetime>$joined_at,\n\
                        left_at: IF $left_at IS NONE THEN NONE ELSE <datetime>$left_at END,\n\
                        mute_until: IF $mute_until IS NONE THEN NONE ELSE <datetime>$mute_until END\n\
                    };",
                )
                .bind(("thread_id", thread_id))
                .bind(("user_id", user_id))
                .bind(("role", role))
                .bind(("joined_at", payload.joined_at))
                .bind(("left_at", payload.left_at))
                .bind(("mute_until", payload.mute_until))
                .await
                .map_err(Self::map_surreal_error)?;
            response.check().map_err(Self::map_surreal_error)?;
            Ok(member_value)
        })
    }

    fn list_members(
        &self,
        thread_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatMember>>> {
        let thread_id = thread_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        user_id,\n\
                        role,\n\
                        type::string(joined_at) AS joined_at,\n\
                        IF left_at IS NONE THEN NONE ELSE type::string(left_at) END AS left_at,\n\
                        IF mute_until IS NONE THEN NONE ELSE type::string(mute_until) END AS mute_until\n\
                     FROM chat_member\n\
                     WHERE thread_id = $thread_id AND left_at IS NONE",
                )
                .bind(("thread_id", thread_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_member_row(rows)
        })
    }

    fn get_member(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMember>>> {
        let thread_id = thread_id.to_string();
        let user_id = user_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        user_id,\n\
                        role,\n\
                        type::string(joined_at) AS joined_at,\n\
                        IF left_at IS NONE THEN NONE ELSE type::string(left_at) END AS left_at,\n\
                        IF mute_until IS NONE THEN NONE ELSE type::string(mute_until) END AS mute_until\n\
                     FROM chat_member\n\
                     WHERE thread_id = $thread_id AND user_id = $user_id AND left_at IS NONE\n\
                     ORDER BY joined_at DESC\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .bind(("user_id", user_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_member_row(rows)?.into_iter().next())
        })
    }

    fn create_message(
        &self,
        message: &ChatMessage,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatMessage>> {
        let payload = match Self::to_rfc3339(message.created_at_ms).map(|created_at| {
            SurrealChatMessageCreateRow {
                thread_id: message.thread_id.clone(),
                message_id: message.message_id.clone(),
                author_id: message.author_id.clone(),
                body: message.body.clone(),
                attachments: message.attachments.clone(),
                created_at,
                edited_at: None,
                deleted_at: None,
                request_id: message.request_id.clone(),
                correlation_id: message.correlation_id.clone(),
            }
        }) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let request_id = message.request_id.clone();
        let thread_id = message.thread_id.clone();
        Box::pin(async move {
            let mut existing = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        message_id,\n\
                        author_id,\n\
                        body,\n\
                        attachments,\n\
                        request_id,\n\
                        correlation_id,\n\
                        type::string(created_at) AS created_at,\n\
                        IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                        IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                     FROM chat_message\n\
                     WHERE thread_id = $thread_id AND request_id = $request_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id.clone()))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let existing_rows: Vec<Value> = existing
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut existing_messages = Self::decode_message_row(existing_rows)?;
            if let Some(existing_message) = existing_messages.pop() {
                return Ok(existing_message);
            }

            let mut response = client
                .query(
                    "CREATE chat_message CONTENT {\n\
                        thread_id: $thread_id,\n\
                        message_id: $message_id,\n\
                        author_id: $author_id,\n\
                        body: $body,\n\
                        attachments: $attachments,\n\
                        request_id: $request_id,\n\
                        correlation_id: $correlation_id,\n\
                        created_at: <datetime>$created_at,\n\
                        edited_at: IF $edited_at IS NONE THEN NONE ELSE <datetime>$edited_at END,\n\
                        deleted_at: IF $deleted_at IS NONE THEN NONE ELSE <datetime>$deleted_at END\n\
                    };\n\
                     SELECT\n\
                        thread_id,\n\
                        message_id,\n\
                        author_id,\n\
                        body,\n\
                        attachments,\n\
                        request_id,\n\
                        correlation_id,\n\
                        type::string(created_at) AS created_at,\n\
                        IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                        IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                     FROM chat_message\n\
                     WHERE thread_id = $thread_id AND message_id = $message_id\n\
                     LIMIT 1;",
                )
                .bind(("thread_id", payload.thread_id))
                .bind(("message_id", payload.message_id))
                .bind(("author_id", payload.author_id))
                .bind(("body", payload.body))
                .bind(("attachments", payload.attachments))
                .bind(("request_id", payload.request_id))
                .bind(("correlation_id", payload.correlation_id))
                .bind(("created_at", payload.created_at))
                .bind(("edited_at", payload.edited_at))
                .bind(("deleted_at", payload.deleted_at))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut messages = Self::decode_message_row(rows)?;
            messages
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
        })
    }

    fn get_message(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
        let thread_id = thread_id.to_string();
        let message_id = message_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        message_id,\n\
                        author_id,\n\
                        body,\n\
                        attachments,\n\
                        request_id,\n\
                        correlation_id,\n\
                        type::string(created_at) AS created_at,\n\
                        IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                        IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                     FROM chat_message\n\
                     WHERE thread_id = $thread_id AND message_id = $message_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .bind(("message_id", message_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_message_row(rows)?.into_iter().next())
        })
    }

    fn get_message_by_request_id(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatMessage>>> {
        let thread_id = thread_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        message_id,\n\
                        author_id,\n\
                        body,\n\
                        attachments,\n\
                        request_id,\n\
                        correlation_id,\n\
                        type::string(created_at) AS created_at,\n\
                        IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                        IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                     FROM chat_message\n\
                     WHERE thread_id = $thread_id AND request_id = $request_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_message_row(rows)?.into_iter().next())
        })
    }

    fn list_messages(
        &self,
        thread_id: &str,
        cursor: &MessageCatchup,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<ChatMessage>>> {
        let thread_id = thread_id.to_string();
        let cursor = cursor.clone();
        let client = self.client.clone();
        if cursor.since_message_id.is_some() && cursor.since_created_at_ms.is_none() {
            return Box::pin(async {
                Err(DomainError::Validation(
                    "since_message_id requires since_created_at_ms".to_string(),
                ))
            });
        }
        Box::pin(async move {
            let mut statement = String::from(
                "SELECT\n\
                    thread_id,\n\
                    message_id,\n\
                    author_id,\n\
                    body,\n\
                    attachments,\n\
                    request_id,\n\
                    correlation_id,\n\
                    type::string(created_at) AS created_at,\n\
                    IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                    IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                 FROM chat_message\n\
                 WHERE thread_id = $thread_id",
            );
            if let Some(since_created_at_ms) = cursor.since_created_at_ms {
                let threshold = Self::to_rfc3339(since_created_at_ms)?;
                if let Some(since_message_id) = cursor.since_message_id {
                    statement.push_str(
                        " AND (created_at > <datetime>$threshold OR (created_at = <datetime>$threshold AND message_id > $since_message_id))",
                    );
                    let mut response = client
                        .query(format!(
                            "{statement} ORDER BY created_at ASC, message_id ASC LIMIT $limit"
                        ))
                        .bind(("thread_id", thread_id))
                        .bind(("threshold", threshold))
                        .bind(("since_message_id", since_message_id))
                        .bind(("limit", cursor.limit as i64))
                        .await
                        .map_err(Self::map_surreal_error)?;
                    let rows: Vec<Value> = response.take(0).map_err(|err| {
                        DomainError::Validation(format!("invalid query result: {err}"))
                    })?;
                    let mut messages = Self::decode_message_row(rows)?;
                    messages.sort_by(|a, b| {
                        a.created_at_ms
                            .cmp(&b.created_at_ms)
                            .then_with(|| a.message_id.cmp(&b.message_id))
                    });
                    messages.truncate(cursor.limit);
                    return Ok(messages);
                }
                statement.push_str(" AND created_at > <datetime>$threshold");
                let mut response = client
                    .query(format!(
                        "{statement} ORDER BY created_at ASC, message_id ASC LIMIT $limit"
                    ))
                    .bind(("thread_id", thread_id))
                    .bind(("threshold", threshold))
                    .bind(("limit", cursor.limit as i64))
                    .await
                    .map_err(Self::map_surreal_error)?;
                let rows: Vec<Value> = response.take(0).map_err(|err| {
                    DomainError::Validation(format!("invalid query result: {err}"))
                })?;
                return Self::decode_message_row(rows);
            }
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        message_id,\n\
                        author_id,\n\
                        body,\n\
                        attachments,\n\
                        request_id,\n\
                        correlation_id,\n\
                        type::string(created_at) AS created_at,\n\
                        IF edited_at IS NONE THEN NONE ELSE type::string(edited_at) END AS edited_at,\n\
                        IF deleted_at IS NONE THEN NONE ELSE type::string(deleted_at) END AS deleted_at\n\
                     FROM chat_message\n\
                     WHERE thread_id = $thread_id\n\
                     ORDER BY created_at DESC, message_id DESC\n\
                     LIMIT $limit",
                )
                .bind(("thread_id", thread_id))
                .bind(("limit", cursor.limit as i64))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut messages = Self::decode_message_row(rows)?;
            messages.sort_by(|a, b| {
                a.created_at_ms
                    .cmp(&b.created_at_ms)
                    .then_with(|| a.message_id.cmp(&b.message_id))
            });
            messages.truncate(cursor.limit);
            Ok(messages)
        })
    }

    fn set_read_cursor(
        &self,
        cursor: &ChatReadCursor,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatReadCursor>> {
        let payload = match Self::to_rfc3339(cursor.last_read_at_ms).map(|last_read_at| {
            SurrealChatReadCursorCreateRow {
                thread_id: cursor.thread_id.clone(),
                user_id: cursor.user_id.clone(),
                last_read_message_id: cursor.last_read_message_id.clone(),
                last_read_at,
            }
        }) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "UPSERT chat_read_cursor CONTENT {\n\
                        thread_id: $thread_id,\n\
                        user_id: $user_id,\n\
                        last_read_message_id: $last_read_message_id,\n\
                        last_read_at: <datetime>$last_read_at\n\
                    };\n\
                     SELECT\n\
                        thread_id,\n\
                        user_id,\n\
                        last_read_message_id,\n\
                        type::string(last_read_at) AS last_read_at\n\
                     FROM chat_read_cursor\n\
                     WHERE thread_id = $thread_id AND user_id = $user_id\n\
                     LIMIT 1;",
                )
                .bind(("thread_id", payload.thread_id))
                .bind(("user_id", payload.user_id))
                .bind(("last_read_message_id", payload.last_read_message_id))
                .bind(("last_read_at", payload.last_read_at))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(1)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut cursors = Self::decode_read_cursor_row(rows)?;
            cursors
                .pop()
                .ok_or_else(|| DomainError::Validation("upsert returned no row".to_string()))
        })
    }

    fn get_read_cursor(
        &self,
        thread_id: &str,
        user_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatReadCursor>>> {
        let thread_id = thread_id.to_string();
        let user_id = user_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        thread_id,\n\
                        user_id,\n\
                        last_read_message_id,\n\
                        type::string(last_read_at) AS last_read_at\n\
                     FROM chat_read_cursor\n\
                     WHERE thread_id = $thread_id AND user_id = $user_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .bind(("user_id", user_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_read_cursor_row(rows)?.into_iter().next())
        })
    }

    fn create_delivery_event(
        &self,
        event: &ChatDeliveryEvent,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<ChatDeliveryEvent>> {
        let payload = match Self::to_rfc3339(event.occurred_at_ms).map(|occurred_at| {
            SurrealChatDeliveryEventCreateRow {
                event_id: event.event_id.clone(),
                thread_id: event.thread_id.clone(),
                message_id: event.message_id.clone(),
                event_type: event.event_type.clone(),
                occurred_at,
                request_id: event.request_id.clone(),
                correlation_id: event.correlation_id.clone(),
            }
        }) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let event_value = event.clone();
        Box::pin(async move {
            let response = client
                .query(
                    "CREATE chat_delivery_event CONTENT {\n\
                        event_id: $event_id,\n\
                        thread_id: $thread_id,\n\
                        message_id: $message_id,\n\
                        event_type: $event_type,\n\
                        occurred_at: <datetime>$occurred_at,\n\
                        request_id: $request_id,\n\
                        correlation_id: $correlation_id\n\
                    };",
                )
                .bind(("event_id", payload.event_id))
                .bind(("thread_id", payload.thread_id))
                .bind(("message_id", payload.message_id))
                .bind(("event_type", payload.event_type))
                .bind(("occurred_at", payload.occurred_at))
                .bind(("request_id", payload.request_id))
                .bind(("correlation_id", payload.correlation_id))
                .await
                .map_err(Self::map_surreal_error)?;
            response.check().map_err(Self::map_surreal_error)?;
            Ok(event_value)
        })
    }

    fn get_delivery_event_by_request(
        &self,
        thread_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<ChatDeliveryEvent>>> {
        let thread_id = thread_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT\n\
                        event_id,\n\
                        thread_id,\n\
                        message_id,\n\
                        event_type,\n\
                        type::string(occurred_at) AS occurred_at,\n\
                        request_id,\n\
                        correlation_id\n\
                     FROM chat_delivery_event\n\
                     WHERE thread_id = $thread_id AND request_id = $request_id\n\
                     LIMIT 1",
                )
                .bind(("thread_id", thread_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Ok(Self::decode_delivery_event_row(rows)?.into_iter().next())
        })
    }
}

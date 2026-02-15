use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::db::DbConfig;
use gotong_domain::DomainResult;
use gotong_domain::chat::{
    ChatDeliveryEvent, ChatMember, ChatMemberRole, ChatMessage, ChatReadCursor, ChatThread,
    ChatThreadQuery, ChatThreadWithMembers, MessageCatchup,
};
use gotong_domain::contributions::Contribution;
use gotong_domain::error::DomainError;
use gotong_domain::evidence::Evidence;
use gotong_domain::moderation::{
    ContentModeration, ModerationAction, ModerationActorSnapshot, ModerationDecision,
    ModerationStatus, ModerationViolation,
};
use gotong_domain::ports::chat::ChatRepository as ChatRepositoryPort;
use gotong_domain::ports::contributions::ContributionRepository;
use gotong_domain::ports::evidence::EvidenceRepository;
use gotong_domain::ports::moderation::ModerationRepository;
use gotong_domain::ports::siaga::SiagaRepository;
use gotong_domain::ports::transitions::TrackTransitionRepository;
use gotong_domain::ports::vault::VaultRepository;
use gotong_domain::ports::vouches::VouchRepository;
use gotong_domain::siaga::{
    SiagaActorSnapshot, SiagaBroadcast, SiagaClosure, SiagaResponder, SiagaState,
    SiagaTimelineEvent, SiagaTimelineEventType,
};
use gotong_domain::transitions::TrackStateTransition;
use gotong_domain::transitions::{
    TransitionActorSnapshot, TransitionGateSnapshot, TransitionMechanism,
};
use gotong_domain::vault::{
    VaultActorSnapshot, VaultEntry, VaultState, VaultTimelineEvent, VaultTimelineEventType,
};
use gotong_domain::vouches::Vouch;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryContributionRepository {
    store: Arc<RwLock<HashMap<String, Contribution>>>,
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
pub struct InMemoryTrackTransitionRepository {
    transitions: Arc<RwLock<HashMap<String, TrackStateTransition>>>,
    by_request: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryTrackTransitionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl InMemoryTrackTransitionRepository {
    fn request_key(entity_id: &str, request_id: &str) -> String {
        format!("{entity_id}:{request_id}")
    }
}

impl TrackTransitionRepository for InMemoryTrackTransitionRepository {
    fn create(
        &self,
        transition: &TrackStateTransition,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<TrackStateTransition>> {
        let transition = transition.clone();
        let transitions = self.transitions.clone();
        let by_request = self.by_request.clone();
        Box::pin(async move {
            let mut transition_map = transitions.write().await;
            if transition_map.contains_key(&transition.transition_id) {
                return Err(DomainError::Conflict);
            }

            let request_key = Self::request_key(&transition.entity_id, &transition.request_id);
            let mut request_map = by_request.write().await;
            if request_map.contains_key(&request_key) {
                return Err(DomainError::Conflict);
            }

            request_map.insert(request_key, transition.transition_id.clone());
            transition_map.insert(transition.transition_id.clone(), transition.clone());
            Ok(transition)
        })
    }

    fn get_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let request_key = Self::request_key(entity_id, request_id);
        let transitions = self.transitions.clone();
        let by_request = self.by_request.clone();
        Box::pin(async move {
            let request_map = by_request.read().await;
            let Some(transition_id) = request_map.get(&request_key) else {
                return Ok(None);
            };
            let transitions = transitions.read().await;
            Ok(transitions.get(transition_id).cloned())
        })
    }

    fn get_by_transition_id(
        &self,
        transition_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let transition_id = transition_id.to_string();
        let transitions = self.transitions.clone();
        Box::pin(async move {
            let transitions = transitions.read().await;
            Ok(transitions.get(&transition_id).cloned())
        })
    }

    fn list_by_entity(
        &self,
        entity_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<TrackStateTransition>>> {
        let entity_id = entity_id.to_string();
        let transitions = self.transitions.clone();
        Box::pin(async move {
            let transitions = transitions.read().await;
            let mut list: Vec<_> = transitions
                .values()
                .filter(|transition| transition.entity_id == entity_id)
                .cloned()
                .collect();
            list.sort_by(|left, right| {
                left.occurred_at_ms
                    .cmp(&right.occurred_at_ms)
                    .then_with(|| left.transition_id.cmp(&right.transition_id))
            });
            Ok(list)
        })
    }
}

#[derive(Clone)]
pub struct SurrealTrackTransitionRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealTrackTransitionRepository {
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

    fn parse_occurred_at_ms(occurred_at: &str) -> DomainResult<i64> {
        let datetime = OffsetDateTime::parse(occurred_at, &Rfc3339)
            .map_err(|err| DomainError::Validation(format!("invalid occurred_at: {err}")))?;
        Ok((datetime.unix_timestamp_nanos() / 1_000_000) as i64)
    }

    fn parse_mechanism(value: &str) -> DomainResult<TransitionMechanism> {
        match value {
            "user_action" => Ok(TransitionMechanism::UserAction),
            "timer" => Ok(TransitionMechanism::Timer),
            "webhook" => Ok(TransitionMechanism::Webhook),
            other => Err(DomainError::Validation(format!(
                "invalid transition mechanism '{other}'"
            ))),
        }
    }

    fn map_row_to_transition(row: SurrealTrackTransitionRow) -> DomainResult<TrackStateTransition> {
        let occurred_at_ms = Self::parse_occurred_at_ms(&row.occurred_at)?;
        Ok(TrackStateTransition {
            track: row.track,
            transition_id: row.transition_id,
            entity_id: row.entity_id,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            from_stage: row.from_stage,
            to_stage: row.to_stage,
            transition_type: Self::parse_mechanism(&row.transition_type)?,
            mechanism: Self::parse_mechanism(&row.mechanism)?,
            actor: row.actor,
            occurred_at_ms,
            gate: row.gate,
        })
    }

    fn to_create_payload(
        transition: &TrackStateTransition,
    ) -> DomainResult<SurrealTrackTransitionCreateRow> {
        let occurred_at_ms = transition.occurred_at_ms;
        let occurred_at =
            OffsetDateTime::from_unix_timestamp_nanos((occurred_at_ms as i128) * 1_000_000)
                .map_err(|err| DomainError::Validation(format!("invalid occurred_at_ms: {err}")))?;
        Ok(SurrealTrackTransitionCreateRow {
            transition_id: transition.transition_id.clone(),
            entity_id: transition.entity_id.clone(),
            request_id: transition.request_id.clone(),
            correlation_id: transition.correlation_id.clone(),
            track: transition.track.clone(),
            from_stage: transition.from_stage.clone(),
            to_stage: transition.to_stage.clone(),
            transition_type: mechanism_to_string(&transition.transition_type),
            mechanism: mechanism_to_string(&transition.mechanism),
            actor: transition.actor.clone(),
            occurred_at: occurred_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            gate: transition.gate.clone(),
        })
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

    fn decode_transition_rows(rows: Vec<Value>) -> DomainResult<Vec<TrackStateTransition>> {
        rows.into_iter()
            .map(|row| {
                serde_json::from_value::<SurrealTrackTransitionRow>(row)
                    .map_err(|err| {
                        DomainError::Validation(format!("invalid transition row: {err}"))
                    })
                    .and_then(Self::map_row_to_transition)
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct SurrealTrackTransitionRow {
    transition_id: String,
    entity_id: String,
    request_id: String,
    correlation_id: String,
    track: String,
    from_stage: String,
    to_stage: String,
    transition_type: String,
    mechanism: String,
    actor: TransitionActorSnapshot,
    occurred_at: String,
    gate: TransitionGateSnapshot,
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealTrackTransitionCreateRow {
    transition_id: String,
    entity_id: String,
    request_id: String,
    correlation_id: String,
    track: String,
    from_stage: String,
    to_stage: String,
    transition_type: String,
    mechanism: String,
    actor: TransitionActorSnapshot,
    #[allow(dead_code)]
    #[serde(rename = "occurred_at")]
    occurred_at: String,
    gate: TransitionGateSnapshot,
}

fn mechanism_to_string(mechanism: &TransitionMechanism) -> String {
    match mechanism {
        TransitionMechanism::UserAction => "user_action".to_string(),
        TransitionMechanism::Timer => "timer".to_string(),
        TransitionMechanism::Webhook => "webhook".to_string(),
    }
}

impl TrackTransitionRepository for SurrealTrackTransitionRepository {
    fn create(
        &self,
        transition: &TrackStateTransition,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<TrackStateTransition>> {
        let payload = match Self::to_create_payload(transition) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE track_state_transition CONTENT $payload")
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
            Self::decode_transition_rows(vec![row]).and_then(|mut transitions| {
                transitions
                    .pop()
                    .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
            })
        })
    }

    fn get_by_request_id(
        &self,
        entity_id: &str,
        request_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let entity_id = entity_id.to_string();
        let request_id = request_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM track_state_transition \
                     WHERE entity_id = $entity_id AND request_id = $request_id",
                )
                .bind(("entity_id", entity_id))
                .bind(("request_id", request_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut transitions = Self::decode_transition_rows(rows)?;
            Ok(transitions.pop())
        })
    }

    fn get_by_transition_id(
        &self,
        transition_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
        let transition_id = transition_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query("SELECT * FROM track_state_transition WHERE transition_id = $transition_id")
                .bind(("transition_id", transition_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut transitions = Self::decode_transition_rows(rows)?;
            Ok(transitions.pop())
        })
    }

    fn list_by_entity(
        &self,
        entity_id: &str,
    ) -> gotong_domain::ports::BoxFuture<'_, DomainResult<Vec<TrackStateTransition>>> {
        let entity_id = entity_id.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let mut response = client
                .query(
                    "SELECT * FROM track_state_transition \
                     WHERE entity_id = $entity_id ORDER BY occurred_at ASC, transition_id ASC",
                )
                .bind(("entity_id", entity_id))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_transition_rows(rows)
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

#[derive(Clone)]
pub struct SurrealVaultRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealVaultRepository {
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
                        let created_at_ms = Self::parse_datetime_ms(&row.created_at)?;
                        let updated_at_ms = Self::parse_datetime_ms(&row.updated_at)?;
                        let sealed_at_ms = match row.sealed_at {
                            Some(value) => Some(Self::parse_datetime_ms(&value)?),
                            None => None,
                        };
                        Ok(VaultEntry {
                            vault_entry_id: row.vault_entry_id,
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
                        Ok(VaultTimelineEvent {
                            event_id: row.event_id,
                            vault_entry_id: row.vault_entry_id,
                            event_type: Self::parse_event_type(&row.event_type)?,
                            actor: row.actor,
                            request_id: row.request_id,
                            correlation_id: row.correlation_id,
                            occurred_at_ms,
                            metadata: row.metadata,
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
                .query("CREATE type::thing('vault_entry', $vault_entry_id) CONTENT $payload")
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
                    "CREATE type::thing('vault_timeline_event', $event_id) CONTENT $event_payload",
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
                .query("UPDATE type::thing('vault_entry', $vault_entry_id) CONTENT $payload")
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
                    "CREATE type::thing('vault_timeline_event', $event_id) CONTENT $event_payload",
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
        Ok(SiagaBroadcast {
            siaga_id: row.siaga_id,
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
        Ok(SiagaTimelineEvent {
            event_id: row.event_id,
            siaga_id: row.siaga_id,
            event_type: Self::parse_event_type(&row.event_type)?,
            actor: row.actor,
            request_id: row.request_id,
            correlation_id: row.correlation_id,
            occurred_at_ms,
            metadata: row.metadata,
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
}

#[derive(Debug, Deserialize)]
struct SurrealSiagaTimelineRequestRow {
    siaga_id: String,
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
                .query("CREATE type::thing('siaga_broadcast', $siaga_id) CONTENT $payload")
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
                    "CREATE type::thing('siaga_timeline_event', $event_id) CONTENT $event_payload",
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

            if let None = Self::get_broadcast_from_store(&client, &siaga_id).await? {
                return Err(DomainError::NotFound);
            }

            let payload = to_value(payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga broadcast payload: {err}"))
            })?;
            let event_payload = to_value(event_payload).map_err(|err| {
                DomainError::Validation(format!("invalid siaga timeline payload: {err}"))
            })?;

            let mut response = client
                .query("UPDATE type::thing('siaga_broadcast', $siaga_id) CONTENT $payload")
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
                    "CREATE type::thing('siaga_timeline_event', $event_id) CONTENT $event_payload",
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
        })
    }

    fn map_content_row(row: SurrealModerationContentRow) -> DomainResult<ContentModeration> {
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
        })
    }

    fn map_decision_row(row: SurrealModerationDecisionRow) -> DomainResult<ModerationDecision> {
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
                .query("UPSERT type::thing('content_moderation', $content_id) CONTENT $payload")
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
                .query("SELECT * FROM type::thing('content_moderation', $content_id)")
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
                .query("CREATE type::thing('moderation_decision', $decision_id) CONTENT $payload")
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
                    .filter_map(|((thread_id, member_user), member)| {
                        (member_user == &actor_id && member.left_at_ms.is_none())
                            .then(|| thread_id.clone())
                    })
                    .collect();
                output = output
                    .into_iter()
                    .filter(|thread| {
                        thread.privacy_level == "public" || allowed.contains(&thread.thread_id)
                    })
                    .collect();
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
            let thread_ids = thread_ids;
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
                .cloned()
                .filter(|message| {
                    cursor.since_created_at_ms.is_none_or(|threshold| {
                        message.created_at_ms > threshold
                            || cursor.since_message_id.as_ref().is_none_or(|cursor_id| {
                                message.created_at_ms != threshold
                                    || message.message_id.as_str() > cursor_id.as_str()
                            })
                    })
                })
                .collect();
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
                    .and_then(|row| Self::map_chat_thread_row(row))
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
                    .and_then(|row| Self::map_chat_member_row(row))
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
                    .and_then(|row| Self::map_chat_message_row(row))
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
                    .and_then(|row| Self::map_chat_read_cursor_row(row))
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
                    .and_then(|row| Self::map_chat_delivery_event_row(row))
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
struct SurrealChatThreadCreateRow {
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
        let payload = match Self::to_rfc3339(thread.created_at_ms).and_then(|created_at| {
            Self::to_rfc3339(thread.updated_at_ms).map(|updated_at| SurrealChatThreadCreateRow {
                thread_id: thread.thread_id.clone(),
                scope_id: thread.scope_id.clone(),
                created_by: thread.created_by.clone(),
                privacy_level: thread.privacy_level.clone(),
                created_at,
                updated_at,
            })
        }) {
            Ok(payload) => payload,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let client = self.client.clone();
        let thread_id = thread.thread_id.clone();
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE chat_thread CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut threads = Self::decode_thread_row(rows)?;
            let thread = threads
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))?;
            if thread.thread_id != thread_id {
                return Ok(thread);
            }
            Ok(thread)
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
                .query("SELECT * FROM chat_thread WHERE thread_id = $thread_id LIMIT 1")
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

            let mut query_sql = String::from("SELECT * FROM chat_thread");
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
                    "SELECT * FROM chat_thread WHERE thread_id IN (SELECT thread_id FROM chat_member WHERE user_id = $user_id AND left_at IS NONE) ORDER BY created_at DESC, thread_id DESC",
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
        let thread_id = member.thread_id.clone();
        let user_id = member.user_id.clone();
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

            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE chat_member CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut members = Self::decode_member_row(rows)?;
            members
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
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
                .query("SELECT * FROM chat_member WHERE thread_id = $thread_id AND left_at IS NONE")
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
                    "SELECT * FROM chat_member \
                     WHERE thread_id = $thread_id AND user_id = $user_id AND left_at IS NONE \
                     ORDER BY joined_at DESC LIMIT 1",
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
                    "SELECT * FROM chat_message WHERE thread_id = $thread_id AND request_id = $request_id LIMIT 1",
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

            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE chat_message CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
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
                    "SELECT * FROM chat_message WHERE thread_id = $thread_id AND message_id = $message_id LIMIT 1",
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
                .query("SELECT * FROM chat_message WHERE thread_id = $thread_id AND request_id = $request_id LIMIT 1")
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
            let mut statement =
                String::from("SELECT * FROM chat_message WHERE thread_id = $thread_id");
            if let Some(since_created_at_ms) = cursor.since_created_at_ms {
                let threshold = Self::to_rfc3339(since_created_at_ms)?;
                if let Some(since_message_id) = cursor.since_message_id {
                    statement.push_str(" AND (created_at > $threshold OR (created_at = $threshold AND message_id > $since_message_id))");
                    let mut response = client
                        .query(&format!(
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
                statement.push_str(" AND created_at > $threshold");
                let mut response = client
                    .query(&format!(
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
                .query("SELECT * FROM chat_message WHERE thread_id = $thread_id ORDER BY created_at ASC, message_id ASC LIMIT $limit")
                .bind(("thread_id", thread_id))
                .bind(("limit", cursor.limit as i64))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            Self::decode_message_row(rows)
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
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("UPSERT chat_read_cursor CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
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
                    "SELECT * FROM chat_read_cursor WHERE thread_id = $thread_id AND user_id = $user_id LIMIT 1",
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
        Box::pin(async move {
            let payload = to_value(payload)
                .map_err(|err| DomainError::Validation(format!("invalid payload: {err}")))?;
            let mut response = client
                .query("CREATE chat_delivery_event CONTENT $payload")
                .bind(("payload", payload))
                .await
                .map_err(Self::map_surreal_error)?;
            let rows: Vec<Value> = response
                .take(0)
                .map_err(|err| DomainError::Validation(format!("invalid query result: {err}")))?;
            let mut events = Self::decode_delivery_event_row(rows)?;
            events
                .pop()
                .ok_or_else(|| DomainError::Validation("create returned no row".to_string()))
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
                    "SELECT * FROM chat_delivery_event WHERE thread_id = $thread_id AND request_id = $request_id LIMIT 1",
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

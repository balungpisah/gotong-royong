use std::collections::HashMap;
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
use gotong_domain::ports::chat::ChatRepository as ChatRepositoryPort;
use gotong_domain::ports::contributions::ContributionRepository;
use gotong_domain::ports::evidence::EvidenceRepository;
use gotong_domain::ports::transitions::TrackTransitionRepository;
use gotong_domain::ports::vouches::VouchRepository;
use gotong_domain::transitions::TrackStateTransition;
use gotong_domain::transitions::{
    TransitionActorSnapshot, TransitionGateSnapshot, TransitionMechanism,
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

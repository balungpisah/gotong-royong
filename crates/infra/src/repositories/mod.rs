use std::collections::HashMap;
use std::sync::Arc;

use crate::db::DbConfig;
use gotong_domain::DomainResult;
use gotong_domain::contributions::Contribution;
use gotong_domain::error::DomainError;
use gotong_domain::evidence::Evidence;
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

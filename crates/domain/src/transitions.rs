use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::auth::{Role, TrackRole};
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::transitions::TrackTransitionRepository;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAction {
    Propose,
    Object,
    Vote,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionMechanism {
    UserAction,
    Timer,
    Webhook,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransitionActorSnapshot {
    pub user_id: String,
    pub username: String,
    pub token_role: String,
    pub track_roles: Vec<TrackRole>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl TransitionActorSnapshot {
    pub fn new(
        actor: ActorIdentity,
        token_role: &Role,
        track_roles: Vec<TrackRole>,
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        request_ts_ms: i64,
    ) -> Self {
        Self {
            user_id: actor.user_id,
            username: actor.username,
            token_role: token_role.as_str().to_string(),
            track_roles,
            request_id: request_id.into(),
            correlation_id: correlation_id.into(),
            request_ts_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransitionGateSnapshot {
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrackStateTransition {
    pub track: String,
    pub transition_id: String,
    pub entity_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub from_stage: String,
    pub to_stage: String,
    pub transition_type: TransitionMechanism,
    pub mechanism: TransitionMechanism,
    pub actor: TransitionActorSnapshot,
    pub occurred_at_ms: i64,
    pub gate: TransitionGateSnapshot,
}

#[derive(Clone)]
pub struct TrackTransitionInput {
    pub track: String,
    pub entity_id: String,
    pub from_stage: String,
    pub to_stage: String,
    pub transition_action: TransitionAction,
    pub transition_type: TransitionMechanism,
    pub mechanism: TransitionMechanism,
    pub request_id: String,
    pub correlation_id: String,
    pub track_roles: Vec<TrackRole>,
    pub gate_status: String,
    pub gate_metadata: Option<serde_json::Value>,
    pub occurred_at_ms: Option<i64>,
    pub request_ts_ms: Option<i64>,
    pub closes_at_ms: Option<i64>,
}

#[derive(Clone)]
pub struct TrackTransitionService {
    repository: Arc<dyn TrackTransitionRepository>,
}

impl TrackTransitionService {
    pub fn new(repository: Arc<dyn TrackTransitionRepository>) -> Self {
        Self { repository }
    }

    pub async fn track_state_transition(
        &self,
        actor: ActorIdentity,
        token_role: Role,
        input: TrackTransitionInput,
    ) -> DomainResult<TrackStateTransition> {
        let input = validate_transition_command(input, &token_role)?;

        let happened_at_ms = input.occurred_at_ms.unwrap_or_else(now_ms);
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let transition_id = crate::util::uuid_v7_without_dashes();
        let actor_snapshot = TransitionActorSnapshot::new(
            actor,
            &token_role,
            input.track_roles,
            input.request_id.clone(),
            input.correlation_id.clone(),
            request_ts_ms,
        );
        let gate = TransitionGateSnapshot {
            status: input.gate_status,
            metadata: input.gate_metadata,
        };
        let transition = TrackStateTransition {
            track: input.track,
            transition_id,
            entity_id: input.entity_id,
            request_id: input.request_id,
            correlation_id: input.correlation_id,
            from_stage: input.from_stage,
            to_stage: input.to_stage,
            transition_type: input.transition_type,
            mechanism: input.mechanism,
            actor: actor_snapshot,
            occurred_at_ms: happened_at_ms,
            gate,
        };

        match self.repository.create(&transition).await {
            Ok(transition) => Ok(transition),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request_id(&transition.entity_id, &transition.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn list_by_entity(&self, entity_id: &str) -> DomainResult<Vec<TrackStateTransition>> {
        self.repository.list_by_entity(entity_id).await
    }

    pub async fn get_by_transition_id(
        &self,
        transition_id: &str,
    ) -> DomainResult<Option<TrackStateTransition>> {
        self.repository.get_by_transition_id(transition_id).await
    }

    pub async fn active_stage(&self, entity_id: &str) -> DomainResult<Option<String>> {
        let mut transitions = self.repository.list_by_entity(entity_id).await?;
        let active_stage = transitions.pop().map(|transition| transition.to_stage);
        Ok(active_stage)
    }
}

fn validate_transition_command(
    input: TrackTransitionInput,
    token_role: &Role,
) -> DomainResult<TrackTransitionInput> {
    let input = sanitize_transition_input(input)?;
    validate_track(&input.track)?;
    validate_stage_change(&input.from_stage, &input.to_stage)?;
    validate_gate_prerequisites(&input.from_stage, &input.to_stage, &input)?;
    validate_gate_and_mechanism(&input)?;
    validate_trigger_mechanism(token_role, &input.transition_type, &input.mechanism)?;
    validate_actor_matrix(token_role, &input.transition_action, &input.track_roles)?;

    Ok(input)
}

fn validate_track(track: &str) -> DomainResult<()> {
    if track.trim().is_empty() {
        return Err(DomainError::Validation("track is required".into()));
    }
    Ok(())
}

fn validate_stage_change(from_stage: &str, to_stage: &str) -> DomainResult<()> {
    let from_stage = from_stage.trim();
    let to_stage = to_stage.trim();
    if from_stage.is_empty() || to_stage.is_empty() {
        return Err(DomainError::Validation(
            "from_stage and to_stage are required".into(),
        ));
    }
    if from_stage == to_stage {
        return Err(DomainError::Validation(
            "from_stage and to_stage must differ".into(),
        ));
    }
    Ok(())
}

fn sanitize_transition_input(
    mut input: TrackTransitionInput,
) -> DomainResult<TrackTransitionInput> {
    input.track = input.track.trim().to_string();
    input.entity_id = input.entity_id.trim().to_string();
    input.from_stage = input.from_stage.trim().to_string();
    input.to_stage = input.to_stage.trim().to_string();
    input.gate_status = input.gate_status.trim().to_string();

    if input.track.is_empty()
        || input.entity_id.is_empty()
        || input.from_stage.is_empty()
        || input.to_stage.is_empty()
        || input.gate_status.is_empty()
    {
        return Err(DomainError::Validation(
            "transition fields cannot be empty".into(),
        ));
    }

    Ok(input)
}

fn validate_gate_and_mechanism(input: &TrackTransitionInput) -> DomainResult<()> {
    if input.transition_type != input.mechanism {
        return Err(DomainError::Validation(
            "transition_type and mechanism must match".into(),
        ));
    }
    if input.gate_status.trim().is_empty() {
        return Err(DomainError::Validation("gate.status is required".into()));
    }

    if input.transition_type == TransitionMechanism::Timer {
        if input.closes_at_ms.is_none() {
            return Err(DomainError::Validation(
                "closes_at_ms is required for timer transitions".into(),
            ));
        }
    } else if input.closes_at_ms.is_some() {
        return Err(DomainError::Validation(
            "closes_at_ms is only valid for timer transitions".into(),
        ));
    }

    Ok(())
}

fn validate_trigger_mechanism(
    token_role: &Role,
    transition_type: &TransitionMechanism,
    mechanism: &TransitionMechanism,
) -> DomainResult<()> {
    if token_role == &Role::System {
        if *transition_type != TransitionMechanism::Timer
            || *mechanism != TransitionMechanism::Timer
        {
            return Err(DomainError::Validation(
                "system role is only allowed for timer transitions".into(),
            ));
        }
        return Ok(());
    }

    if *mechanism != TransitionMechanism::UserAction {
        return Err(DomainError::Validation(
            "non-system role must use user_action mechanism".into(),
        ));
    }

    if *transition_type != TransitionMechanism::UserAction {
        return Err(DomainError::Validation(
            "non-system role must use user_action transition type".into(),
        ));
    }

    Ok(())
}

fn validate_gate_prerequisites(
    from_stage: &str,
    to_stage: &str,
    input: &TrackTransitionInput,
) -> DomainResult<()> {
    let from_stage = from_stage.trim().to_ascii_lowercase();
    let to_stage = to_stage.trim().to_ascii_lowercase();

    match (from_stage.as_str(), to_stage.as_str()) {
        ("garap", "periksa") => {
            let has_por_refs = input
                .gate_metadata
                .as_ref()
                .and_then(|value| value.get("por_refs_ready"))
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if !has_por_refs {
                return Err(DomainError::Validation(
                    "garap -> periksa requires gate_metadata.por_refs_ready=true".into(),
                ));
            }
        }
        ("periksa", "tuntas") => {
            let has_challenge_window = input
                .gate_metadata
                .as_ref()
                .and_then(|value| value.get("challenge_window_configured"))
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if !has_challenge_window {
                return Err(DomainError::Validation(
                    "periksa -> tuntas requires gate_metadata.challenge_window_configured=true"
                        .into(),
                ));
            }
        }
        _ => {}
    }

    Ok(())
}

fn validate_actor_matrix(
    token_role: &Role,
    action: &TransitionAction,
    track_roles: &[TrackRole],
) -> DomainResult<()> {
    if token_role == &Role::System {
        return Ok(());
    }
    if track_roles.is_empty() {
        return Err(DomainError::Validation(
            "actor track role is required for transition action".into(),
        ));
    }

    if !track_roles
        .iter()
        .any(|role| role.supports(action.as_str()))
    {
        return Err(DomainError::Validation(
            "actor role is not permitted for transition action".into(),
        ));
    }

    Ok(())
}

impl TransitionAction {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Propose => "propose",
            Self::Object => "object",
            Self::Vote => "vote",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use crate::ports::transitions::TrackTransitionRepository;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockTransitionRepository {
        items: RwLock<HashMap<String, TrackStateTransition>>,
        by_request: RwLock<HashMap<String, String>>,
    }

    impl MockTransitionRepository {
        fn request_key(entity_id: &str, request_id: &str) -> String {
            format!("{entity_id}:{request_id}")
        }
    }

    impl TrackTransitionRepository for MockTransitionRepository {
        fn create(
            &self,
            transition: &TrackStateTransition,
        ) -> BoxFuture<'_, DomainResult<TrackStateTransition>> {
            let transition = transition.clone();
            Box::pin(async move {
                let mut items = self.items.write().await;
                if items.contains_key(&transition.transition_id) {
                    return Err(DomainError::Conflict);
                }
                let request_key = Self::request_key(&transition.entity_id, &transition.request_id);
                if let Some(existing_id) = self.by_request.read().await.get(&request_key) {
                    let existing = items.get(existing_id).cloned();
                    return existing.ok_or(DomainError::Conflict);
                }
                items.insert(transition.transition_id.clone(), transition.clone());
                self.by_request
                    .write()
                    .await
                    .insert(request_key, transition.transition_id.clone());
                Ok(transition)
            })
        }

        fn get_by_request_id(
            &self,
            entity_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
            let request_key = Self::request_key(entity_id, request_id);
            Box::pin(async move {
                let by_request = self.by_request.read().await;
                let Some(id) = by_request.get(&request_key) else {
                    return Ok(None);
                };
                let items = self.items.read().await;
                Ok(items.get(id).cloned())
            })
        }

        fn get_by_transition_id(
            &self,
            transition_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<TrackStateTransition>>> {
            let transition_id = transition_id.to_string();
            Box::pin(async move {
                let items = self.items.read().await;
                Ok(items.get(&transition_id).cloned())
            })
        }

        fn list_by_entity(
            &self,
            entity_id: &str,
        ) -> BoxFuture<'_, DomainResult<Vec<TrackStateTransition>>> {
            let entity_id = entity_id.to_string();
            Box::pin(async move {
                let items = self.items.read().await;
                let mut transitions: Vec<_> = items
                    .values()
                    .filter(|item| item.entity_id == entity_id)
                    .cloned()
                    .collect();
                transitions.sort_by(|left, right| {
                    left.occurred_at_ms
                        .cmp(&right.occurred_at_ms)
                        .then_with(|| left.transition_id.cmp(&right.transition_id))
                });
                Ok(transitions)
            })
        }
    }

    #[tokio::test]
    async fn transition_replays_by_request_id() {
        let repo = Arc::new(MockTransitionRepository::default());
        let service = TrackTransitionService::new(repo.clone());
        let command = TrackTransitionInput {
            track: "resolve".to_string(),
            entity_id: "track-1".to_string(),
            from_stage: "garap".to_string(),
            to_stage: "periksa".to_string(),
            transition_action: TransitionAction::Object,
            transition_type: TransitionMechanism::UserAction,
            mechanism: TransitionMechanism::UserAction,
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            track_roles: vec![TrackRole::Participant],
            gate_status: "open".to_string(),
            gate_metadata: Some(serde_json::json!({
                "por_refs_ready": true
            })),
            occurred_at_ms: Some(1),
            request_ts_ms: Some(1),
            closes_at_ms: None,
        };

        let actor = ActorIdentity {
            user_id: "u-1".to_string(),
            username: "alice".to_string(),
        };

        let first = service
            .track_state_transition(actor.clone(), Role::User, command.clone())
            .await
            .expect("first transition");
        let second = service
            .track_state_transition(actor, Role::User, command)
            .await
            .expect("second transition");

        assert_eq!(first.transition_id, second.transition_id);
        assert_eq!(first.entity_id, second.entity_id);
    }

    #[tokio::test]
    async fn timeline_ordering_is_deterministic() {
        let repo = Arc::new(MockTransitionRepository::default());
        let service = TrackTransitionService::new(repo);
        let actor = ActorIdentity {
            user_id: "u-1".to_string(),
            username: "alice".to_string(),
        };
        let base = TrackTransitionInput {
            track: "resolve".to_string(),
            entity_id: "entity-1".to_string(),
            from_stage: "garap".to_string(),
            to_stage: "periksa".to_string(),
            transition_action: TransitionAction::Object,
            transition_type: TransitionMechanism::UserAction,
            mechanism: TransitionMechanism::UserAction,
            request_id: "req-newer".to_string(),
            correlation_id: "corr-1".to_string(),
            track_roles: vec![TrackRole::Participant],
            gate_status: "open".to_string(),
            gate_metadata: Some(serde_json::json!({
                "por_refs_ready": true
            })),
            occurred_at_ms: Some(2_000),
            request_ts_ms: Some(2_000),
            closes_at_ms: None,
        };
        let earlier = TrackTransitionInput {
            request_id: "req-earlier".to_string(),
            occurred_at_ms: Some(1_000),
            ..base.clone()
        };
        let later = base.clone();

        service
            .track_state_transition(actor.clone(), Role::User, earlier)
            .await
            .expect("earlier transition");
        service
            .track_state_transition(actor, Role::User, later)
            .await
            .expect("later transition");

        let timeline = service.list_by_entity("entity-1").await.expect("timeline");
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].occurred_at_ms, 1_000);
        assert_eq!(timeline[1].occurred_at_ms, 2_000);
    }

    #[test]
    fn role_matrix_allows_expected_roles() {
        let candidate = [TrackRole::Author, TrackRole::Participant];
        assert!(candidate[0].supports("propose"));
        assert!(!candidate[1].supports("propose"));
        assert!(candidate[1].supports("object"));
    }

    #[test]
    fn role_matrix_blocks_empty_roles() {
        let err = validate_actor_matrix(&Role::User, &TransitionAction::Object, &[]).unwrap_err();
        assert!(matches!(
            err,
            DomainError::Validation(msg) if msg == "actor track role is required for transition action"
        ));
    }

    #[test]
    fn transition_type_must_match_mechanism() {
        let err = validate_gate_and_mechanism(&TrackTransitionInput {
            track: "resolve".to_string(),
            entity_id: "track-1".to_string(),
            from_stage: "garap".to_string(),
            to_stage: "periksa".to_string(),
            transition_action: TransitionAction::Object,
            transition_type: TransitionMechanism::UserAction,
            mechanism: TransitionMechanism::Timer,
            request_id: "req-2".to_string(),
            correlation_id: "corr-2".to_string(),
            track_roles: vec![TrackRole::Author],
            gate_status: "open".to_string(),
            gate_metadata: Some(serde_json::json!({
                "por_refs_ready": true
            })),
            occurred_at_ms: None,
            request_ts_ms: None,
            closes_at_ms: None,
        })
        .unwrap_err();
        assert!(matches!(
            err,
            DomainError::Validation(msg) if msg == "transition_type and mechanism must match"
        ));
    }

    #[test]
    fn non_system_roles_cannot_use_timer_mechanism() {
        let err = validate_trigger_mechanism(
            &Role::User,
            &TransitionMechanism::UserAction,
            &TransitionMechanism::Timer,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            DomainError::Validation(msg) if msg == "non-system role must use user_action mechanism"
        ));
    }
}

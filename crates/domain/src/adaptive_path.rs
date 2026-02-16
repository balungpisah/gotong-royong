use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::auth::Role;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::{DomainResult, jobs::now_ms, ports::adaptive_path::AdaptivePathRepository};

const MAX_TITLE_LEN: usize = 220;
const MAX_SUMMARY_LEN: usize = 2_000;
const MAX_HINT_LEN: usize = 1_000;
const MAX_LABEL_LEN: usize = 180;
const MAX_OBJECTIVE_LEN: usize = 1_000;
const MAX_CHECKPOINT_LEN: usize = 320;
const MAX_BRANCHES: usize = 60;
const MAX_PHASES_PER_BRANCH: usize = 120;
const MAX_CHECKPOINTS_PER_PHASE: usize = 180;
const MAX_ORDER: i64 = 999;
const MAX_LOCKED_FIELDS: usize = 20;
const MAX_TEXT_LINE: usize = 16_384;

type PlanVersion = u64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptivePathSource {
    Ai,
    Human,
    System,
}

impl FromStr for AdaptivePathSource {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ai" => Ok(Self::Ai),
            "human" => Ok(Self::Human),
            "system" => Ok(Self::System),
            other => Err(DomainError::Validation(format!(
                "invalid adaptive path source '{other}'"
            ))),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptivePathStatus {
    Planned,
    Active,
    Open,
    Completed,
    Blocked,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptivePathEventType {
    PlanCreated,
    PlanUpdated,
    SuggestionProposed,
    SuggestionAccepted,
    SuggestionRejected,
}

impl AdaptivePathEventType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::PlanCreated => "plan_created",
            Self::PlanUpdated => "plan_updated",
            Self::SuggestionProposed => "suggestion_proposed",
            Self::SuggestionAccepted => "suggestion_accepted",
            Self::SuggestionRejected => "suggestion_rejected",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionDecisionStatus {
    Pending,
    Accepted,
    Rejected,
}

impl SuggestionDecisionStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

impl FromStr for SuggestionDecisionStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            other => Err(DomainError::Validation(format!(
                "invalid adaptive path suggestion status '{other}'"
            ))),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptivePathEditorRole {
    Author,
    ProjectManager,
    HighestProfileUser,
    Participant,
}

impl AdaptivePathEditorRole {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::ProjectManager => "project_manager",
            Self::HighestProfileUser => "highest_profile_user",
            Self::Participant => "participant",
        }
    }

    fn is_privileged(&self) -> bool {
        matches!(self, Self::ProjectManager | Self::HighestProfileUser)
    }
}

impl FromStr for AdaptivePathEditorRole {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "author" => Ok(Self::Author),
            "project_manager" => Ok(Self::ProjectManager),
            "highest_profile_user" => Ok(Self::HighestProfileUser),
            "participant" => Ok(Self::Participant),
            other => Err(DomainError::Validation(format!(
                "unknown adaptive path role '{other}'"
            ))),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathActorSnapshot {
    pub user_id: String,
    pub username: String,
    pub token_role: String,
    pub editor_roles: Vec<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl AdaptivePathActorSnapshot {
    fn new(
        actor: &ActorIdentity,
        token_role: &Role,
        editor_roles: &[AdaptivePathEditorRole],
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        request_ts_ms: i64,
    ) -> Self {
        Self {
            user_id: actor.user_id.clone(),
            username: actor.username.clone(),
            token_role: token_role.as_str().to_string(),
            editor_roles: editor_roles
                .iter()
                .map(|role| role.as_str().to_string())
                .collect(),
            request_id: request_id.into(),
            correlation_id: correlation_id.into(),
            request_ts_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathCheckpoint {
    pub checkpoint_id: String,
    pub phase_id: String,
    pub title: String,
    pub status: AdaptivePathStatus,
    pub order: i64,
    pub source: AdaptivePathSource,
    pub locked_fields: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathPhase {
    pub phase_id: String,
    pub branch_id: String,
    pub title: String,
    pub objective: String,
    pub status: AdaptivePathStatus,
    pub order: i64,
    pub source: AdaptivePathSource,
    pub locked_fields: Vec<String>,
    pub checkpoints: Vec<AdaptivePathCheckpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathBranch {
    pub branch_id: String,
    pub label: String,
    pub parent_checkpoint_id: Option<String>,
    pub order: i64,
    pub phases: Vec<AdaptivePathPhase>,
    pub locked_fields: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathPlanPayload {
    pub title: String,
    pub summary: Option<String>,
    pub track_hint: Option<String>,
    pub seed_hint: Option<String>,
    pub branches: Vec<AdaptivePathBranch>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathPlan {
    pub plan_id: String,
    pub entity_id: String,
    pub version: PlanVersion,
    pub title: String,
    pub summary: Option<String>,
    pub track_hint: Option<String>,
    pub seed_hint: Option<String>,
    pub author_id: String,
    pub author_username: String,
    pub branches: Vec<AdaptivePathBranch>,
    pub request_id: String,
    pub correlation_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub event_hash: String,
    pub retention_tag: String,
}

impl AdaptivePathPlan {
    pub fn payload(&self) -> AdaptivePathPlanPayload {
        AdaptivePathPlanPayload {
            title: self.title.clone(),
            summary: self.summary.clone(),
            track_hint: self.track_hint.clone(),
            seed_hint: self.seed_hint.clone(),
            branches: self.branches.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathEvent {
    pub event_id: String,
    pub plan_id: String,
    pub event_type: AdaptivePathEventType,
    pub actor: AdaptivePathActorSnapshot,
    pub request_id: String,
    pub correlation_id: String,
    pub base_version: PlanVersion,
    pub next_version: PlanVersion,
    pub occurred_at_ms: i64,
    pub metadata: Option<serde_json::Value>,
    pub event_hash: String,
    pub retention_tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdaptivePathSuggestion {
    pub suggestion_id: String,
    pub plan_id: String,
    pub base_version: PlanVersion,
    pub proposal: AdaptivePathPlanPayload,
    pub status: SuggestionDecisionStatus,
    pub created_by: String,
    pub created_by_role: String,
    pub rationale: Option<String>,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub event_hash: String,
    pub retention_tag: String,
}

#[derive(Clone, Debug)]
pub struct AdaptivePathCheckpointDraftInput {
    pub checkpoint_id: Option<String>,
    pub title: String,
    pub status: AdaptivePathStatus,
    pub order: i64,
    pub source: AdaptivePathSource,
}

#[derive(Clone, Debug)]
pub struct AdaptivePathPhaseDraftInput {
    pub phase_id: Option<String>,
    pub title: String,
    pub objective: String,
    pub status: AdaptivePathStatus,
    pub order: i64,
    pub source: AdaptivePathSource,
    pub checkpoints: Vec<AdaptivePathCheckpointDraftInput>,
}

#[derive(Clone, Debug)]
pub struct AdaptivePathBranchDraftInput {
    pub branch_id: Option<String>,
    pub label: String,
    pub parent_checkpoint_id: Option<String>,
    pub order: i64,
    pub phases: Vec<AdaptivePathPhaseDraftInput>,
}

#[derive(Clone, Debug)]
pub struct AdaptivePathPlanPayloadDraft {
    pub title: String,
    pub summary: Option<String>,
    pub track_hint: Option<String>,
    pub seed_hint: Option<String>,
    pub branches: Vec<AdaptivePathBranchDraftInput>,
}

#[derive(Clone)]
pub struct CreateAdaptivePathInput {
    pub entity_id: String,
    pub payload: AdaptivePathPlanPayloadDraft,
    pub editor_roles: Vec<AdaptivePathEditorRole>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct UpdateAdaptivePathInput {
    pub plan_id: String,
    pub expected_version: PlanVersion,
    pub payload: AdaptivePathPlanPayloadDraft,
    pub editor_roles: Vec<AdaptivePathEditorRole>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct SuggestAdaptivePathInput {
    pub plan_id: String,
    pub base_version: PlanVersion,
    pub payload: AdaptivePathPlanPayloadDraft,
    pub rationale: Option<String>,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    pub editor_roles: Vec<AdaptivePathEditorRole>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct SuggestionReviewInput {
    pub suggestion_id: String,
    pub editor_roles: Vec<AdaptivePathEditorRole>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct AdaptivePathService {
    repository: Arc<dyn AdaptivePathRepository>,
}

impl AdaptivePathService {
    pub fn new(repository: Arc<dyn AdaptivePathRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_plan(
        &self,
        actor: &ActorIdentity,
        token_role: &Role,
        input: CreateAdaptivePathInput,
    ) -> DomainResult<AdaptivePathPlan> {
        ensure_actor_can_initiate(token_role, actor)?;
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let editor_roles = dedupe_editor_roles(&input.editor_roles);
        let payload = validate_and_normalize_payload(&input.payload)?;
        let plan = build_plan(
            &input.entity_id,
            crate::util::uuid_v7_without_dashes(),
            None,
            actor,
            &editor_roles,
            1,
            payload,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            request_ts_ms,
        )?;
        let event = build_plan_event(
            &plan,
            actor,
            token_role,
            &editor_roles,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            0,
            1,
            AdaptivePathEventType::PlanCreated,
        )?;

        match self.repository.create_plan(&plan).await {
            Ok(plan) => {
                let _ = self.repository.create_event(&event).await;
                Ok(plan)
            }
            Err(DomainError::Conflict) => self
                .repository
                .get_plan_by_request_id(&plan.entity_id, &plan.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn get_plan(&self, plan_id: &str) -> DomainResult<Option<AdaptivePathPlan>> {
        self.repository.get_plan(plan_id).await
    }

    pub async fn get_plan_by_entity(
        &self,
        entity_id: &str,
    ) -> DomainResult<Option<AdaptivePathPlan>> {
        self.repository.get_plan_by_entity(entity_id).await
    }

    pub async fn list_events(&self, plan_id: &str) -> DomainResult<Vec<AdaptivePathEvent>> {
        let mut events = self.repository.list_events(plan_id).await?;
        events.sort_by(|left, right| {
            left.occurred_at_ms
                .cmp(&right.occurred_at_ms)
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        Ok(events)
    }

    pub async fn list_suggestions(
        &self,
        plan_id: &str,
    ) -> DomainResult<Vec<AdaptivePathSuggestion>> {
        let mut suggestions = self.repository.list_suggestions(plan_id).await?;
        suggestions.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.suggestion_id.cmp(&left.suggestion_id))
        });
        Ok(suggestions)
    }

    pub async fn update_plan(
        &self,
        actor: &ActorIdentity,
        token_role: &Role,
        input: UpdateAdaptivePathInput,
    ) -> DomainResult<AdaptivePathPlan> {
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let editor_roles = dedupe_editor_roles(&input.editor_roles);

        let plan = self
            .repository
            .get_plan(&input.plan_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if plan.version != input.expected_version {
            return Err(DomainError::Conflict);
        }

        ensure_editor_can_modify(token_role, &editor_roles)?;

        let normalized_payload = validate_and_normalize_payload(&input.payload)?;
        let payload = apply_editorial_locks(&plan.payload(), normalized_payload)?;
        let updated_plan = build_plan(
            &plan.entity_id,
            plan.plan_id.clone(),
            Some(&plan),
            actor,
            &editor_roles,
            plan.version + 1,
            payload,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            request_ts_ms,
        )?;
        let event = build_plan_event(
            &updated_plan,
            actor,
            token_role,
            &editor_roles,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            plan.version,
            updated_plan.version,
            AdaptivePathEventType::PlanUpdated,
        )?;

        match self.repository.update_plan(&updated_plan).await {
            Ok(plan) => {
                let _ = self.repository.create_event(&event).await;
                Ok(plan)
            }
            Err(DomainError::Conflict) => self
                .repository
                .get_plan_by_request_id(&updated_plan.entity_id, &input.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn suggest_plan(
        &self,
        actor: &ActorIdentity,
        token_role: &Role,
        input: SuggestAdaptivePathInput,
    ) -> DomainResult<AdaptivePathSuggestion> {
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let editor_roles = dedupe_editor_roles(&input.editor_roles);
        ensure_editor_can_modify(token_role, &editor_roles)?;

        let plan = self
            .repository
            .get_plan(&input.plan_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if plan.version != input.base_version {
            return Err(DomainError::Conflict);
        }

        let normalized = validate_and_normalize_payload(&input.payload)?;
        let proposal = enforce_locked_fields(&plan, normalized)?;
        let suggestion = AdaptivePathSuggestion {
            suggestion_id: crate::util::uuid_v7_without_dashes(),
            plan_id: plan.plan_id.clone(),
            base_version: plan.version,
            proposal,
            status: SuggestionDecisionStatus::Pending,
            created_by: actor.user_id.clone(),
            created_by_role: token_role.as_str().to_string(),
            rationale: input.rationale,
            model_id: input.model_id,
            prompt_version: input.prompt_version,
            request_id: input.request_id.clone(),
            correlation_id: input.correlation_id.clone(),
            created_at_ms: request_ts_ms,
            updated_at_ms: request_ts_ms,
            event_hash: String::new(),
            retention_tag: adaptive_path_suggestion_retention_tag(&plan.plan_id),
        };
        let suggestion = apply_suggestion_hash(suggestion)?;
        let event = build_suggestion_event(
            &plan,
            actor,
            token_role,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            plan.version,
            plan.version,
            AdaptivePathEventType::SuggestionProposed,
        )?;

        match self.repository.create_suggestion(&suggestion).await {
            Ok(suggestion) => {
                let _ = self.repository.create_event(&event).await;
                Ok(suggestion)
            }
            Err(DomainError::Conflict) => self
                .repository
                .get_suggestion_by_request_id(&plan.plan_id, &input.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn accept_suggestion(
        &self,
        actor: &ActorIdentity,
        token_role: &Role,
        input: SuggestionReviewInput,
    ) -> DomainResult<AdaptivePathPlan> {
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let editor_roles = dedupe_editor_roles(&input.editor_roles);

        let suggestion = self
            .repository
            .get_suggestion(&input.suggestion_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if suggestion.status != SuggestionDecisionStatus::Pending {
            return Err(DomainError::Conflict);
        }

        let plan = self
            .repository
            .get_plan(&suggestion.plan_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        ensure_editor_can_modify(token_role, &editor_roles)?;
        let updated_payload = enforce_locked_fields(&plan, suggestion.proposal.clone())?;
        let updated_plan = build_plan(
            &plan.entity_id,
            plan.plan_id.clone(),
            Some(&plan),
            actor,
            &editor_roles,
            plan.version + 1,
            updated_payload,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            request_ts_ms,
        )?;
        let event = build_plan_event(
            &updated_plan,
            actor,
            token_role,
            &editor_roles,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            plan.version,
            updated_plan.version,
            AdaptivePathEventType::SuggestionAccepted,
        )?;

        match self.repository.update_plan(&updated_plan).await {
            Ok(updated_plan) => {
                let _ = self
                    .repository
                    .update_suggestion_status(
                        &suggestion.suggestion_id,
                        SuggestionDecisionStatus::Accepted,
                    )
                    .await?;
                let _ = self.repository.create_event(&event).await;
                Ok(updated_plan)
            }
            Err(DomainError::Conflict) => {
                let updated = self
                    .repository
                    .get_plan_by_request_id(&updated_plan.entity_id, &input.request_id)
                    .await?;
                updated.ok_or(DomainError::Conflict)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn reject_suggestion(
        &self,
        actor: &ActorIdentity,
        token_role: &Role,
        input: SuggestionReviewInput,
    ) -> DomainResult<AdaptivePathSuggestion> {
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let editor_roles = dedupe_editor_roles(&input.editor_roles);

        let suggestion = self
            .repository
            .get_suggestion(&input.suggestion_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if suggestion.status != SuggestionDecisionStatus::Pending {
            return Err(DomainError::Conflict);
        }

        let plan = self
            .repository
            .get_plan(&suggestion.plan_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_editor_can_modify(token_role, &editor_roles)?;

        let event = build_plan_event(
            &plan,
            actor,
            token_role,
            &editor_roles,
            &input.request_id,
            &input.correlation_id,
            request_ts_ms,
            plan.version,
            plan.version,
            AdaptivePathEventType::SuggestionRejected,
        )?;
        let suggestion = self
            .repository
            .update_suggestion_status(&input.suggestion_id, SuggestionDecisionStatus::Rejected)
            .await?;
        let _ = self.repository.create_event(&event).await;
        Ok(suggestion)
    }
}

fn ensure_actor_can_initiate(token_role: &Role, actor: &ActorIdentity) -> DomainResult<()> {
    if actor.user_id.trim().is_empty() {
        return Err(DomainError::Forbidden("actor identity is required".into()));
    }
    match token_role {
        Role::Anonymous => Err(DomainError::Forbidden(
            "anonymous token cannot modify plans".into(),
        )),
        Role::User | Role::Moderator | Role::Admin | Role::System => Ok(()),
    }
}

fn ensure_editor_can_modify(
    token_role: &Role,
    editor_roles: &[AdaptivePathEditorRole],
) -> DomainResult<()> {
    if matches!(token_role, Role::Admin | Role::Moderator | Role::System) {
        return Ok(());
    }

    if editor_roles
        .iter()
        .any(AdaptivePathEditorRole::is_privileged)
    {
        return Ok(());
    }

    Err(DomainError::Forbidden(
        "actor is not allowed to modify adaptive path".into(),
    ))
}

fn dedupe_editor_roles(roles: &[AdaptivePathEditorRole]) -> Vec<AdaptivePathEditorRole> {
    let mut roles: Vec<_> = roles.to_vec();
    roles.sort_by_key(AdaptivePathEditorRole::as_str);
    roles.dedup_by_key(|role| role.as_str());
    roles
}

#[allow(clippy::too_many_arguments)]
fn build_plan(
    entity_id: &str,
    plan_id: String,
    existing: Option<&AdaptivePathPlan>,
    actor: &ActorIdentity,
    editor_roles: &[AdaptivePathEditorRole],
    version: PlanVersion,
    payload: AdaptivePathPlanPayload,
    request_id: &str,
    correlation_id: &str,
    request_ts_ms: i64,
    now_ms: i64,
) -> DomainResult<AdaptivePathPlan> {
    validate_plan_payload(&payload)?;
    let normalized_entity_id = normalize_text(entity_id, 240, "entity_id")?;
    let (author_id, author_username, created_at_ms) = if let Some(existing) = existing {
        if existing.version == 0 {
            return Err(DomainError::Validation("invalid existing version".into()));
        }
        (
            existing.author_id.clone(),
            existing.author_username.clone(),
            existing.created_at_ms,
        )
    } else {
        (actor.user_id.clone(), actor.username.clone(), now_ms)
    };

    let mut plan = AdaptivePathPlan {
        plan_id,
        entity_id: normalized_entity_id.clone(),
        version,
        title: payload.title,
        summary: payload.summary,
        track_hint: payload.track_hint,
        seed_hint: payload.seed_hint,
        author_id,
        author_username,
        branches: payload.branches,
        request_id: request_id.to_string(),
        correlation_id: correlation_id.to_string(),
        created_at_ms,
        updated_at_ms: now_ms,
        event_hash: String::new(),
        retention_tag: adaptive_path_retention_tag(&normalized_entity_id),
    };

    for branch in &mut plan.branches {
        branch.locked_fields = normalize_locked_fields(branch.locked_fields.clone());
        for phase in &mut branch.phases {
            phase.locked_fields = normalize_locked_fields(phase.locked_fields.clone());
            for checkpoint in &mut phase.checkpoints {
                checkpoint.locked_fields =
                    normalize_locked_fields(checkpoint.locked_fields.clone());
            }
        }
    }

    plan.branches.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.branch_id.cmp(&right.branch_id))
    });

    let _ = editor_roles;
    plan.event_hash = adaptive_path_plan_audit_hash(&AdaptivePathPlanAuditPayload {
        plan_id: plan.plan_id.clone(),
        entity_id: plan.entity_id.clone(),
        version: plan.version,
        title: plan.title.clone(),
        summary: plan.summary.clone(),
        track_hint: plan.track_hint.clone(),
        seed_hint: plan.seed_hint.clone(),
        author_id: plan.author_id.clone(),
        branches: plan.branches.clone(),
        request_id: plan.request_id.clone(),
        correlation_id: plan.correlation_id.clone(),
        request_ts_ms,
        retention_tag: plan.retention_tag.clone(),
    })?;
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
fn build_plan_event(
    plan: &AdaptivePathPlan,
    actor: &ActorIdentity,
    token_role: &Role,
    editor_roles: &[AdaptivePathEditorRole],
    request_id: &str,
    correlation_id: &str,
    occurred_at_ms: i64,
    base_version: PlanVersion,
    next_version: PlanVersion,
    event_type: AdaptivePathEventType,
) -> DomainResult<AdaptivePathEvent> {
    let actor = AdaptivePathActorSnapshot::new(
        actor,
        token_role,
        editor_roles,
        request_id,
        correlation_id,
        occurred_at_ms,
    );
    let retention_tag = adaptive_path_event_retention_tag(&plan.plan_id, &event_type);
    let mut event = AdaptivePathEvent {
        event_id: crate::util::uuid_v7_without_dashes(),
        plan_id: plan.plan_id.clone(),
        event_type,
        actor,
        request_id: request_id.to_string(),
        correlation_id: correlation_id.to_string(),
        base_version,
        next_version,
        occurred_at_ms,
        metadata: Some(serde_json::json!({
            "title": plan.title,
            "version": next_version,
            "version_transition": format!("{base_version}->{next_version}"),
        })),
        event_hash: String::new(),
        retention_tag,
    };
    event.event_hash = adaptive_path_event_audit_hash(&AdaptivePathEventAuditPayload {
        event_id: event.event_id.clone(),
        plan_id: event.plan_id.clone(),
        event_type: event.event_type.as_str().to_string(),
        actor: event.actor.clone(),
        request_id: event.request_id.clone(),
        correlation_id: event.correlation_id.clone(),
        base_version: event.base_version,
        next_version: event.next_version,
        occurred_at_ms: event.occurred_at_ms,
        retention_tag: event.retention_tag.clone(),
    })?;
    Ok(event)
}

#[allow(clippy::too_many_arguments)]
fn build_suggestion_event(
    plan: &AdaptivePathPlan,
    actor: &ActorIdentity,
    token_role: &Role,
    request_id: &str,
    correlation_id: &str,
    occurred_at_ms: i64,
    base_version: PlanVersion,
    next_version: PlanVersion,
    event_type: AdaptivePathEventType,
) -> DomainResult<AdaptivePathEvent> {
    let actor = AdaptivePathActorSnapshot::new(
        actor,
        token_role,
        &[AdaptivePathEditorRole::HighestProfileUser],
        request_id,
        correlation_id,
        occurred_at_ms,
    );
    let retention_tag = adaptive_path_event_retention_tag(&plan.plan_id, &event_type);
    let mut event = AdaptivePathEvent {
        event_id: crate::util::uuid_v7_without_dashes(),
        plan_id: plan.plan_id.clone(),
        event_type,
        actor,
        request_id: request_id.to_string(),
        correlation_id: correlation_id.to_string(),
        base_version,
        next_version,
        occurred_at_ms,
        metadata: Some(serde_json::json!({
            "plan_id": &plan.plan_id,
            "version": base_version,
            "entity_id": &plan.entity_id,
        })),
        event_hash: String::new(),
        retention_tag,
    };
    event.event_hash = adaptive_path_event_audit_hash(&AdaptivePathEventAuditPayload {
        event_id: event.event_id.clone(),
        plan_id: event.plan_id.clone(),
        event_type: event.event_type.as_str().to_string(),
        actor: event.actor.clone(),
        request_id: event.request_id.clone(),
        correlation_id: event.correlation_id.clone(),
        base_version: event.base_version,
        next_version: event.next_version,
        occurred_at_ms: event.occurred_at_ms,
        retention_tag: event.retention_tag.clone(),
    })?;
    Ok(event)
}

fn apply_suggestion_hash(
    mut suggestion: AdaptivePathSuggestion,
) -> DomainResult<AdaptivePathSuggestion> {
    suggestion.event_hash =
        adaptive_path_suggestion_audit_hash(&AdaptivePathSuggestionAuditPayload {
            suggestion_id: suggestion.suggestion_id.clone(),
            plan_id: suggestion.plan_id.clone(),
            base_version: suggestion.base_version,
            status: suggestion.status.as_str().to_string(),
            created_by: suggestion.created_by.clone(),
            created_by_role: suggestion.created_by_role.clone(),
            rationale: suggestion.rationale.clone(),
            model_id: suggestion.model_id.clone(),
            prompt_version: suggestion.prompt_version.clone(),
            request_id: suggestion.request_id.clone(),
            correlation_id: suggestion.correlation_id.clone(),
            created_at_ms: suggestion.created_at_ms,
            updated_at_ms: suggestion.updated_at_ms,
            retention_tag: suggestion.retention_tag.clone(),
        })?;
    Ok(suggestion)
}

fn validate_plan_payload(payload: &AdaptivePathPlanPayload) -> DomainResult<()> {
    validate_non_empty_text(&payload.title, MAX_TITLE_LEN, "title")?;
    if let Some(value) = payload.summary.as_ref() {
        validate_text_len(value, MAX_SUMMARY_LEN, "summary")?;
    }
    if let Some(value) = payload.track_hint.as_ref() {
        validate_text_len(value, MAX_HINT_LEN, "track_hint")?;
    }
    if let Some(value) = payload.seed_hint.as_ref() {
        validate_text_len(value, MAX_HINT_LEN, "seed_hint")?;
    }
    if payload.branches.is_empty() {
        return Err(DomainError::Validation(
            "plan must contain at least one branch".into(),
        ));
    }
    if payload.branches.len() > MAX_BRANCHES {
        return Err(DomainError::Validation(format!(
            "too many branches (max {MAX_BRANCHES})"
        )));
    }
    Ok(())
}

fn validate_and_normalize_payload(
    draft: &AdaptivePathPlanPayloadDraft,
) -> DomainResult<AdaptivePathPlanPayload> {
    let title = normalize_text(&draft.title, MAX_TITLE_LEN, "title")?;
    let summary = normalize_optional_text(draft.summary.as_deref(), MAX_SUMMARY_LEN, "summary")?;
    let track_hint =
        normalize_optional_text(draft.track_hint.as_deref(), MAX_HINT_LEN, "track_hint")?;
    let seed_hint = normalize_optional_text(draft.seed_hint.as_deref(), MAX_HINT_LEN, "seed_hint")?;

    if draft.branches.is_empty() {
        return Err(DomainError::Validation(
            "at least one branch is required".into(),
        ));
    }
    if draft.branches.len() > MAX_BRANCHES {
        return Err(DomainError::Validation(format!(
            "too many branches (max {MAX_BRANCHES})"
        )));
    }

    let mut branches: Vec<AdaptivePathBranch> = Vec::with_capacity(draft.branches.len());
    let mut branch_ids = HashSet::new();
    let mut branch_orders = HashSet::new();
    let mut phase_ids = HashSet::new();
    let mut checkpoint_ids = HashSet::new();

    for branch_input in &draft.branches {
        let branch_id = normalize_or_generate_id(branch_input.branch_id.as_deref());
        if !branch_ids.insert(branch_id.clone()) {
            return Err(DomainError::Validation(format!(
                "duplicate branch_id '{branch_id}'"
            )));
        }
        if !branch_orders.insert(branch_input.order) {
            return Err(DomainError::Validation("duplicate branch order".into()));
        }
        if branch_input.order < 0 || branch_input.order > MAX_ORDER {
            return Err(DomainError::Validation(
                "branch order must be between 0 and 999".into(),
            ));
        }

        let label = normalize_text(&branch_input.label, MAX_LABEL_LEN, "branch label")?;
        let parent_checkpoint_id = branch_input
            .parent_checkpoint_id
            .as_ref()
            .and_then(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then_some(trimmed.to_string())
            });

        if branch_input.phases.is_empty() {
            return Err(DomainError::Validation(format!(
                "branch '{branch_id}' requires at least one phase"
            )));
        }
        if branch_input.phases.len() > MAX_PHASES_PER_BRANCH {
            return Err(DomainError::Validation(format!(
                "too many phases in branch '{branch_id}' (max {MAX_PHASES_PER_BRANCH})"
            )));
        }

        let mut phases: Vec<AdaptivePathPhase> = Vec::with_capacity(branch_input.phases.len());
        let mut local_phase_orders = HashSet::new();
        for phase_input in &branch_input.phases {
            let phase_id = normalize_or_generate_id(phase_input.phase_id.as_deref());
            if !phase_ids.insert(phase_id.clone()) {
                return Err(DomainError::Validation(format!(
                    "duplicate phase_id '{phase_id}'"
                )));
            }
            if !local_phase_orders.insert(phase_input.order) {
                return Err(DomainError::Validation(format!(
                    "duplicate phase order in branch '{branch_id}'"
                )));
            }
            if phase_input.order < 0 || phase_input.order > MAX_ORDER {
                return Err(DomainError::Validation(
                    "phase order must be between 0 and 999".into(),
                ));
            }
            if phase_input.checkpoints.is_empty() {
                return Err(DomainError::Validation(format!(
                    "phase '{phase_id}' must include at least one checkpoint"
                )));
            }
            if phase_input.checkpoints.len() > MAX_CHECKPOINTS_PER_PHASE {
                return Err(DomainError::Validation(format!(
                    "too many checkpoints in phase '{phase_id}' (max {MAX_CHECKPOINTS_PER_PHASE})"
                )));
            }

            let phase_title = normalize_text(&phase_input.title, MAX_TITLE_LEN, "phase title")?;
            let objective =
                normalize_text(&phase_input.objective, MAX_OBJECTIVE_LEN, "phase objective")?;
            let phase_checkpoints = create_phase_checkpoints(
                phase_id.as_str(),
                &phase_input.checkpoints,
                &mut checkpoint_ids,
            )?;

            phases.push(AdaptivePathPhase {
                phase_id: phase_id.clone(),
                branch_id: branch_id.clone(),
                title: phase_title,
                objective,
                status: phase_input.status.clone(),
                order: phase_input.order,
                source: phase_input.source.clone(),
                locked_fields: Vec::new(),
                checkpoints: phase_checkpoints,
            });
        }

        phases.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.phase_id.cmp(&right.phase_id))
        });

        branches.push(AdaptivePathBranch {
            branch_id,
            label,
            parent_checkpoint_id,
            order: branch_input.order,
            phases,
            locked_fields: Vec::new(),
        });
    }

    let mut checkpoint_ids_all = HashSet::new();
    for branch in &branches {
        for phase in &branch.phases {
            for checkpoint in &phase.checkpoints {
                checkpoint_ids_all.insert(checkpoint.checkpoint_id.clone());
            }
        }
    }

    for branch in &mut branches {
        if let Some(parent_checkpoint_id) = &branch.parent_checkpoint_id {
            if !checkpoint_ids_all.contains(parent_checkpoint_id) {
                return Err(DomainError::Validation(format!(
                    "parent_checkpoint_id '{parent_checkpoint_id}' not found"
                )));
            }
        }
    }

    branches.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.branch_id.cmp(&right.branch_id))
    });

    Ok(AdaptivePathPlanPayload {
        title,
        summary,
        track_hint,
        seed_hint,
        branches,
    })
}

fn create_phase_checkpoints(
    phase_id: &str,
    checkpoints: &[AdaptivePathCheckpointDraftInput],
    checkpoint_ids: &mut HashSet<String>,
) -> DomainResult<Vec<AdaptivePathCheckpoint>> {
    let mut order_set = HashSet::new();

    let mut rows = Vec::with_capacity(checkpoints.len());
    for checkpoint_input in checkpoints {
        let checkpoint_id = normalize_or_generate_id(checkpoint_input.checkpoint_id.as_deref());
        if !checkpoint_ids.insert(checkpoint_id.clone()) {
            return Err(DomainError::Validation(format!(
                "duplicate checkpoint_id '{checkpoint_id}'"
            )));
        }
        if !order_set.insert(checkpoint_input.order) {
            return Err(DomainError::Validation(format!(
                "duplicate checkpoint order in phase '{phase_id}'"
            )));
        }
        if checkpoint_input.order < 0 || checkpoint_input.order > MAX_ORDER {
            return Err(DomainError::Validation(
                "checkpoint order must be between 0 and 999".into(),
            ));
        }
        let title = normalize_text(
            &checkpoint_input.title,
            MAX_CHECKPOINT_LEN,
            "checkpoint title",
        )?;
        rows.push(AdaptivePathCheckpoint {
            checkpoint_id: checkpoint_id.clone(),
            phase_id: phase_id.to_string(),
            title,
            status: checkpoint_input.status.clone(),
            order: checkpoint_input.order,
            source: checkpoint_input.source.clone(),
            locked_fields: Vec::new(),
        });
    }
    rows.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.checkpoint_id.cmp(&right.checkpoint_id))
    });
    Ok(rows)
}

fn apply_editorial_locks(
    base: &AdaptivePathPlanPayload,
    mut payload: AdaptivePathPlanPayload,
) -> DomainResult<AdaptivePathPlanPayload> {
    let base_branches = map_branches_by_id(base);
    let base_phases = map_phases_by_id(base);
    let base_checkpoints = map_checkpoints_by_id(base);

    for branch in &mut payload.branches {
        if let Some(base_branch) = base_branches.get(&branch.branch_id) {
            let mut locked = base_branch.locked_fields.clone();
            for field in changed_branch_fields(base_branch, branch) {
                if !locked.contains(&field.to_string()) {
                    locked.push(field.to_string());
                }
            }
            branch.locked_fields = normalize_locked_fields(locked);
        }
        for phase in &mut branch.phases {
            if let Some(base_phase) = base_phases.get(&phase.phase_id) {
                let mut locked = base_phase.locked_fields.clone();
                for field in changed_phase_fields(base_phase, phase) {
                    if !locked.contains(&field.to_string()) {
                        locked.push(field.to_string());
                    }
                }
                phase.locked_fields = normalize_locked_fields(locked);
            }
            for checkpoint in &mut phase.checkpoints {
                if let Some(base_checkpoint) = base_checkpoints.get(&checkpoint.checkpoint_id) {
                    let mut locked = base_checkpoint.locked_fields.clone();
                    for field in changed_checkpoint_fields(base_checkpoint, checkpoint) {
                        if !locked.contains(&field.to_string()) {
                            locked.push(field.to_string());
                        }
                    }
                    checkpoint.locked_fields = normalize_locked_fields(locked);
                }
            }
        }
    }
    Ok(payload)
}

fn enforce_locked_fields(
    plan: &AdaptivePathPlan,
    mut payload: AdaptivePathPlanPayload,
) -> DomainResult<AdaptivePathPlanPayload> {
    let plan_branches = map_plan_branches_by_id(plan);
    let plan_phases = map_plan_phases_by_id(plan);
    let plan_checkpoints = map_plan_checkpoints_by_id(plan);

    for branch in &mut payload.branches {
        if let Some(base_branch) = plan_branches.get(&branch.branch_id) {
            for field in &base_branch.locked_fields {
                match field.as_str() {
                    "label" => branch.label = base_branch.label.clone(),
                    "order" => branch.order = base_branch.order,
                    "parent_checkpoint_id" => {
                        branch.parent_checkpoint_id = base_branch.parent_checkpoint_id.clone();
                    }
                    _ => {}
                }
            }
        }
        for phase in &mut branch.phases {
            if let Some(base_phase) = plan_phases.get(&phase.phase_id) {
                for field in &base_phase.locked_fields {
                    match field.as_str() {
                        "title" => phase.title = base_phase.title.clone(),
                        "objective" => phase.objective = base_phase.objective.clone(),
                        "status" => phase.status = base_phase.status.clone(),
                        "source" => phase.source = base_phase.source.clone(),
                        "order" => phase.order = base_phase.order,
                        _ => {}
                    }
                }
            }
            for checkpoint in &mut phase.checkpoints {
                if let Some(base_checkpoint) = plan_checkpoints.get(&checkpoint.checkpoint_id) {
                    for field in &base_checkpoint.locked_fields {
                        match field.as_str() {
                            "title" => checkpoint.title = base_checkpoint.title.clone(),
                            "status" => checkpoint.status = base_checkpoint.status.clone(),
                            "source" => checkpoint.source = base_checkpoint.source.clone(),
                            "order" => checkpoint.order = base_checkpoint.order,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(payload)
}

fn changed_branch_fields(
    before: &AdaptivePathBranch,
    after: &AdaptivePathBranch,
) -> Vec<&'static str> {
    let mut changed = Vec::new();
    if before.label != after.label {
        changed.push("label");
    }
    if before.order != after.order {
        changed.push("order");
    }
    if before.parent_checkpoint_id != after.parent_checkpoint_id {
        changed.push("parent_checkpoint_id");
    }
    changed
}

fn changed_phase_fields(
    before: &AdaptivePathPhase,
    after: &AdaptivePathPhase,
) -> Vec<&'static str> {
    let mut changed = Vec::new();
    if before.title != after.title {
        changed.push("title");
    }
    if before.objective != after.objective {
        changed.push("objective");
    }
    if before.status != after.status {
        changed.push("status");
    }
    if before.source != after.source {
        changed.push("source");
    }
    if before.order != after.order {
        changed.push("order");
    }
    changed
}

fn changed_checkpoint_fields(
    before: &AdaptivePathCheckpoint,
    after: &AdaptivePathCheckpoint,
) -> Vec<&'static str> {
    let mut changed = Vec::new();
    if before.title != after.title {
        changed.push("title");
    }
    if before.status != after.status {
        changed.push("status");
    }
    if before.source != after.source {
        changed.push("source");
    }
    if before.order != after.order {
        changed.push("order");
    }
    changed
}

fn map_plan_branches_by_id(plan: &AdaptivePathPlan) -> HashMap<String, AdaptivePathBranch> {
    plan.branches
        .iter()
        .map(|branch| (branch.branch_id.clone(), branch.clone()))
        .collect()
}

fn map_plan_phases_by_id(plan: &AdaptivePathPlan) -> HashMap<String, AdaptivePathPhase> {
    plan.branches
        .iter()
        .flat_map(|branch| branch.phases.iter())
        .map(|phase| (phase.phase_id.clone(), phase.clone()))
        .collect()
}

fn map_plan_checkpoints_by_id(plan: &AdaptivePathPlan) -> HashMap<String, AdaptivePathCheckpoint> {
    plan.branches
        .iter()
        .flat_map(|branch| branch.phases.iter())
        .flat_map(|phase| phase.checkpoints.iter())
        .map(|checkpoint| (checkpoint.checkpoint_id.clone(), checkpoint.clone()))
        .collect()
}

fn map_branches_by_id(payload: &AdaptivePathPlanPayload) -> HashMap<String, AdaptivePathBranch> {
    payload
        .branches
        .iter()
        .map(|branch| (branch.branch_id.clone(), branch.clone()))
        .collect()
}

fn map_phases_by_id(payload: &AdaptivePathPlanPayload) -> HashMap<String, AdaptivePathPhase> {
    payload
        .branches
        .iter()
        .flat_map(|branch| branch.phases.iter())
        .map(|phase| (phase.phase_id.clone(), phase.clone()))
        .collect()
}

fn map_checkpoints_by_id(
    payload: &AdaptivePathPlanPayload,
) -> HashMap<String, AdaptivePathCheckpoint> {
    payload
        .branches
        .iter()
        .flat_map(|branch| branch.phases.iter())
        .flat_map(|phase| phase.checkpoints.iter())
        .map(|checkpoint| (checkpoint.checkpoint_id.clone(), checkpoint.clone()))
        .collect()
}

fn normalize_non_empty_text(value: &str, field_name: &str) -> DomainResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DomainError::Validation(format!(
            "{field_name} cannot be empty"
        )));
    }
    Ok(trimmed.to_string())
}

fn sanitize_text(value: &str, max_len: usize, field_name: &str) -> DomainResult<String> {
    let normalized = normalize_non_empty_text(value, field_name)?;
    validate_text_len(&normalized, max_len, field_name)?;
    Ok(normalized)
}

fn validate_non_empty_text(value: &str, max_len: usize, field_name: &str) -> DomainResult<()> {
    let value = normalize_non_empty_text(value, field_name)?;
    validate_text_len(&value, max_len, field_name)?;
    Ok(())
}

fn validate_text_len(value: &str, max_len: usize, field_name: &str) -> DomainResult<()> {
    if value.len() > MAX_TEXT_LINE {
        return Err(DomainError::Validation(format!(
            "{field_name} exceeds hard text ceiling of {MAX_TEXT_LINE}"
        )));
    }
    if value.chars().count() > max_len {
        return Err(DomainError::Validation(format!(
            "{field_name} exceeds max length of {max_len}"
        )));
    }
    Ok(())
}

fn normalize_optional_text(
    value: Option<&str>,
    max_len: usize,
    field_name: &str,
) -> DomainResult<Option<String>> {
    match value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            validate_text_len(trimmed, max_len, field_name)?;
            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

fn normalize_text(value: &str, max_len: usize, field_name: &str) -> DomainResult<String> {
    sanitize_text(value, max_len, field_name)
}

fn normalize_or_generate_id(value: Option<&str>) -> String {
    match value {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                crate::util::uuid_v7_without_dashes()
            } else {
                trimmed.to_string()
            }
        }
        None => crate::util::uuid_v7_without_dashes(),
    }
}

fn normalize_locked_fields(mut values: Vec<String>) -> Vec<String> {
    values.sort_unstable();
    values.dedup();
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .take(MAX_LOCKED_FIELDS)
        .collect()
}

fn adaptive_path_retention_tag(entity_id: &str) -> String {
    format!("adaptive_path:{entity_id}")
}

fn adaptive_path_event_retention_tag(plan_id: &str, event_type: &AdaptivePathEventType) -> String {
    format!("adaptive_path_event:{plan_id}:{}", event_type.as_str())
}

fn adaptive_path_suggestion_retention_tag(plan_id: &str) -> String {
    format!("adaptive_path_suggestion:{plan_id}")
}

fn adaptive_path_plan_audit_hash(payload: &AdaptivePathPlanAuditPayload) -> DomainResult<String> {
    crate::util::immutable_event_hash(payload)
}

fn adaptive_path_event_audit_hash(payload: &AdaptivePathEventAuditPayload) -> DomainResult<String> {
    crate::util::immutable_event_hash(payload)
}

fn adaptive_path_suggestion_audit_hash(
    payload: &AdaptivePathSuggestionAuditPayload,
) -> DomainResult<String> {
    crate::util::immutable_event_hash(payload)
}

#[derive(Clone, Serialize)]
struct AdaptivePathPlanAuditPayload {
    plan_id: String,
    entity_id: String,
    version: PlanVersion,
    title: String,
    summary: Option<String>,
    track_hint: Option<String>,
    seed_hint: Option<String>,
    author_id: String,
    branches: Vec<AdaptivePathBranch>,
    request_id: String,
    correlation_id: String,
    request_ts_ms: i64,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct AdaptivePathEventAuditPayload {
    event_id: String,
    plan_id: String,
    event_type: String,
    actor: AdaptivePathActorSnapshot,
    request_id: String,
    correlation_id: String,
    base_version: PlanVersion,
    next_version: PlanVersion,
    occurred_at_ms: i64,
    retention_tag: String,
}

#[derive(Clone, Serialize)]
struct AdaptivePathSuggestionAuditPayload {
    suggestion_id: String,
    plan_id: String,
    base_version: PlanVersion,
    status: String,
    created_by: String,
    created_by_role: String,
    rationale: Option<String>,
    model_id: Option<String>,
    prompt_version: Option<String>,
    request_id: String,
    correlation_id: String,
    created_at_ms: i64,
    updated_at_ms: i64,
    retention_tag: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockAdaptivePathRepository {
        plans: Arc<RwLock<HashMap<String, AdaptivePathPlan>>>,
        by_entity: Arc<RwLock<HashMap<String, String>>>,
        by_request: Arc<RwLock<HashMap<(String, String), String>>>,
        events: Arc<RwLock<HashMap<String, Vec<AdaptivePathEvent>>>>,
        suggestions: Arc<RwLock<HashMap<String, AdaptivePathSuggestion>>>,
        suggestion_by_request: Arc<RwLock<HashMap<(String, String), String>>>,
    }

    impl MockAdaptivePathRepository {
        fn plan_request_key(entity_id: &str, request_id: &str) -> (String, String) {
            (entity_id.to_string(), request_id.to_string())
        }

        fn suggestion_request_key(plan_id: &str, request_id: &str) -> (String, String) {
            (plan_id.to_string(), request_id.to_string())
        }
    }

    impl ports::adaptive_path::AdaptivePathRepository for MockAdaptivePathRepository {
        fn create_plan(
            &self,
            plan: &AdaptivePathPlan,
        ) -> ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
            let plan = plan.clone();
            let plans = self.plans.clone();
            let by_entity = self.by_entity.clone();
            let by_request = self.by_request.clone();
            Box::pin(async move {
                let request_key = Self::plan_request_key(&plan.entity_id, &plan.request_id);
                {
                    let by_request_read = by_request.read().await;
                    if by_request_read.contains_key(&request_key) {
                        return Err(DomainError::Conflict);
                    }
                }
                {
                    let by_entity_read = by_entity.read().await;
                    if by_entity_read.contains_key(&plan.entity_id) {
                        return Err(DomainError::Conflict);
                    }
                }

                let mut by_entity = by_entity.write().await;
                let mut by_request = by_request.write().await;
                let mut plans = plans.write().await;

                by_entity.insert(plan.entity_id.clone(), plan.plan_id.clone());
                by_request.insert(request_key, plan.plan_id.clone());
                plans.insert(plan.plan_id.clone(), plan.clone());
                Ok(plan)
            })
        }

        fn get_plan(
            &self,
            plan_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
            let plan_id = plan_id.to_string();
            let plans = self.plans.clone();
            Box::pin(async move {
                let plans = plans.read().await;
                Ok(plans.get(&plan_id).cloned())
            })
        }

        fn get_plan_by_entity(
            &self,
            entity_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
            let entity_id = entity_id.to_string();
            let by_entity = self.by_entity.clone();
            let plans = self.plans.clone();
            Box::pin(async move {
                let binding = by_entity.read().await;
                let Some(plan_id) = binding.get(&entity_id) else {
                    return Ok(None);
                };
                let plan_id = plan_id.to_string();
                let plans = plans.read().await;
                Ok(plans.get(&plan_id).cloned())
            })
        }

        fn get_plan_by_request_id(
            &self,
            entity_id: &str,
            request_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathPlan>>> {
            let request_key = Self::plan_request_key(entity_id, request_id);
            let by_request = self.by_request.clone();
            let plans = self.plans.clone();
            Box::pin(async move {
                let binding = by_request.read().await;
                let Some(plan_id) = binding.get(&request_key) else {
                    return Ok(None);
                };
                let plans = plans.read().await;
                Ok(plans.get(plan_id).cloned())
            })
        }

        fn update_plan(
            &self,
            plan: &AdaptivePathPlan,
        ) -> ports::BoxFuture<'_, DomainResult<AdaptivePathPlan>> {
            let plan = plan.clone();
            let plans = self.plans.clone();
            let by_request = self.by_request.clone();
            Box::pin(async move {
                let request_key = Self::plan_request_key(&plan.entity_id, &plan.request_id);
                let by_request = by_request.read().await;
                if let Some(existing_plan_id) = by_request.get(&request_key) {
                    if existing_plan_id != &plan.plan_id {
                        return Err(DomainError::Conflict);
                    }
                }
                drop(by_request);

                let mut plans = plans.write().await;
                if !plans.contains_key(&plan.plan_id) {
                    return Err(DomainError::NotFound);
                }
                plans.insert(plan.plan_id.clone(), plan.clone());
                Ok(plan)
            })
        }

        fn create_event(
            &self,
            event: &AdaptivePathEvent,
        ) -> ports::BoxFuture<'_, DomainResult<AdaptivePathEvent>> {
            let event = event.clone();
            let events = self.events.clone();
            Box::pin(async move {
                let mut events = events.write().await;
                let rows = events.entry(event.plan_id.clone()).or_default();
                if rows
                    .iter()
                    .any(|existing| existing.event_id == event.event_id)
                {
                    return Err(DomainError::Conflict);
                }
                if rows
                    .iter()
                    .any(|existing| existing.request_id == event.request_id)
                {
                    return Err(DomainError::Conflict);
                }
                rows.push(event.clone());
                Ok(event)
            })
        }

        fn list_events(
            &self,
            plan_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathEvent>>> {
            let plan_id = plan_id.to_string();
            let events = self.events.clone();
            Box::pin(async move {
                let mut events = events
                    .read()
                    .await
                    .get(&plan_id)
                    .cloned()
                    .unwrap_or_default();
                events.sort_by(|a, b| {
                    a.occurred_at_ms
                        .cmp(&b.occurred_at_ms)
                        .then_with(|| a.event_id.cmp(&b.event_id))
                });
                Ok(events)
            })
        }

        fn get_event_by_request_id(
            &self,
            request_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathEvent>>> {
            let request_id = request_id.to_string();
            let events = self.events.clone();
            Box::pin(async move {
                for events_by_plan in events.read().await.values() {
                    for event in events_by_plan {
                        if event.request_id == request_id {
                            return Ok(Some(event.clone()));
                        }
                    }
                }
                Ok(None)
            })
        }

        fn create_suggestion(
            &self,
            suggestion: &AdaptivePathSuggestion,
        ) -> ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
            let suggestion = suggestion.clone();
            let suggestions = self.suggestions.clone();
            let suggestion_by_request = self.suggestion_by_request.clone();
            Box::pin(async move {
                let request_key =
                    Self::suggestion_request_key(&suggestion.plan_id, &suggestion.request_id);
                {
                    let by_request = suggestion_by_request.read().await;
                    if by_request.contains_key(&request_key) {
                        return Err(DomainError::Conflict);
                    }
                }
                let mut suggestions = suggestions.write().await;
                if suggestions.contains_key(&suggestion.suggestion_id) {
                    return Err(DomainError::Conflict);
                }
                suggestions.insert(suggestion.suggestion_id.clone(), suggestion.clone());
                suggestion_by_request
                    .write()
                    .await
                    .insert(request_key, suggestion.suggestion_id.clone());
                Ok(suggestion)
            })
        }

        fn list_suggestions(
            &self,
            plan_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Vec<AdaptivePathSuggestion>>> {
            let plan_id = plan_id.to_string();
            let suggestions = self.suggestions.clone();
            Box::pin(async move {
                let mut values: Vec<_> = suggestions
                    .read()
                    .await
                    .values()
                    .filter(|suggestion| suggestion.plan_id == plan_id)
                    .cloned()
                    .collect();
                values.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
                Ok(values)
            })
        }

        fn get_suggestion(
            &self,
            suggestion_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
            let suggestions = self.suggestions.clone();
            let suggestion_id = suggestion_id.to_string();
            Box::pin(async move {
                let suggestions = suggestions.read().await;
                Ok(suggestions.get(&suggestion_id).cloned())
            })
        }

        fn get_suggestion_by_request_id(
            &self,
            plan_id: &str,
            request_id: &str,
        ) -> ports::BoxFuture<'_, DomainResult<Option<AdaptivePathSuggestion>>> {
            let request_key = Self::suggestion_request_key(plan_id, request_id);
            let suggestion_by_request = self.suggestion_by_request.clone();
            let suggestions = self.suggestions.clone();
            Box::pin(async move {
                let by_request = suggestion_by_request.read().await;
                let Some(suggestion_id) = by_request.get(&request_key) else {
                    return Ok(None);
                };
                let suggestions = suggestions.read().await;
                Ok(suggestions.get(suggestion_id).cloned())
            })
        }

        fn update_suggestion_status(
            &self,
            suggestion_id: &str,
            status: SuggestionDecisionStatus,
        ) -> ports::BoxFuture<'_, DomainResult<AdaptivePathSuggestion>> {
            let suggestion_id = suggestion_id.to_string();
            let suggestions = self.suggestions.clone();
            Box::pin(async move {
                let mut suggestions = suggestions.write().await;
                let suggestion = suggestions
                    .get_mut(&suggestion_id)
                    .ok_or(DomainError::NotFound)?;
                suggestion.status = status;
                suggestion.updated_at_ms = now_ms();
                Ok(suggestion.clone())
            })
        }
    }

    fn actor() -> ActorIdentity {
        ActorIdentity {
            user_id: "user-1".to_string(),
            username: "alice".to_string(),
        }
    }

    fn sample_payload() -> AdaptivePathPlanPayloadDraft {
        AdaptivePathPlanPayloadDraft {
            title: "Plan title".to_string(),
            summary: Some("summary".to_string()),
            track_hint: Some("track".to_string()),
            seed_hint: Some("seed".to_string()),
            branches: vec![AdaptivePathBranchDraftInput {
                branch_id: Some("branch-main".to_string()),
                label: "Main".to_string(),
                parent_checkpoint_id: None,
                order: 0,
                phases: vec![AdaptivePathPhaseDraftInput {
                    phase_id: Some("phase-main".to_string()),
                    title: "Fase awal".to_string(),
                    objective: "Validasi awal".to_string(),
                    status: AdaptivePathStatus::Active,
                    order: 0,
                    source: AdaptivePathSource::Ai,
                    checkpoints: vec![AdaptivePathCheckpointDraftInput {
                        checkpoint_id: Some("checkpoint-main-1".to_string()),
                        title: "Laporkan lokasi".to_string(),
                        status: AdaptivePathStatus::Open,
                        order: 0,
                        source: AdaptivePathSource::Ai,
                    }],
                }],
            }],
        }
    }

    #[tokio::test]
    async fn create_plan_is_idempotent() {
        let repo = Arc::new(MockAdaptivePathRepository::default());
        let service = AdaptivePathService::new(repo);
        let command = CreateAdaptivePathInput {
            entity_id: "entity-1".to_string(),
            payload: sample_payload(),
            editor_roles: vec![AdaptivePathEditorRole::ProjectManager],
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            request_ts_ms: Some(1),
        };

        let first = service
            .create_plan(&actor(), &Role::User, command.clone())
            .await
            .expect("create");
        let second = service
            .create_plan(&actor(), &Role::User, command)
            .await
            .expect("replay");
        assert_eq!(first.plan_id, second.plan_id);
    }

    #[tokio::test]
    async fn update_plan_applies_editor_locks() {
        let repo = Arc::new(MockAdaptivePathRepository::default());
        let service = AdaptivePathService::new(repo);
        let create = service
            .create_plan(
                &actor(),
                &Role::User,
                CreateAdaptivePathInput {
                    entity_id: "entity-1".to_string(),
                    payload: sample_payload(),
                    editor_roles: vec![AdaptivePathEditorRole::ProjectManager],
                    request_id: "req-create".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("create");

        let mut updated_payload = sample_payload();
        if let Some(phase) = updated_payload
            .branches
            .get_mut(0)
            .and_then(|branch| branch.phases.get_mut(0))
        {
            phase.title = "Updated title".to_string();
        }

        let updated = service
            .update_plan(
                &actor(),
                &Role::User,
                UpdateAdaptivePathInput {
                    plan_id: create.plan_id,
                    expected_version: 1,
                    payload: updated_payload,
                    editor_roles: vec![AdaptivePathEditorRole::ProjectManager],
                    request_id: "req-update".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("update");

        assert_eq!(updated.version, 2);
        assert!(
            updated
                .payload()
                .branches
                .first()
                .expect("branch")
                .phases
                .first()
                .expect("phase")
                .locked_fields
                .contains(&"title".to_string())
        );
    }

    #[tokio::test]
    async fn suggestion_respects_locked_fields() {
        let repo = Arc::new(MockAdaptivePathRepository::default());
        let service = AdaptivePathService::new(repo.clone());
        let plan = service
            .create_plan(
                &actor(),
                &Role::User,
                CreateAdaptivePathInput {
                    entity_id: "entity-2".to_string(),
                    payload: sample_payload(),
                    editor_roles: vec![AdaptivePathEditorRole::ProjectManager],
                    request_id: "req-create-2".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("create");

        let mut update_payload = sample_payload();
        if let Some(phase) = update_payload
            .branches
            .get_mut(0)
            .and_then(|branch| branch.phases.get_mut(0))
        {
            phase.objective = "changed objective".to_string();
        }

        let updated = service
            .update_plan(
                &actor(),
                &Role::User,
                UpdateAdaptivePathInput {
                    plan_id: plan.plan_id.clone(),
                    expected_version: 1,
                    payload: update_payload,
                    editor_roles: vec![AdaptivePathEditorRole::ProjectManager],
                    request_id: "req-update-2".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("update");

        let mut suggest_payload = sample_payload();
        if let Some(phase) = suggest_payload
            .branches
            .get_mut(0)
            .and_then(|branch| branch.phases.get_mut(0))
        {
            phase.title = "changed title".to_string();
        }

        let suggestion = service
            .suggest_plan(
                &actor(),
                &Role::User,
                SuggestAdaptivePathInput {
                    plan_id: updated.plan_id.clone(),
                    base_version: updated.version,
                    payload: suggest_payload,
                    rationale: Some("no".to_string()),
                    model_id: Some("model-x".to_string()),
                    prompt_version: Some("1.0".to_string()),
                    editor_roles: vec![AdaptivePathEditorRole::HighestProfileUser],
                    request_id: "req-suggest".to_string(),
                    correlation_id: "corr-3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await
            .expect("suggestion");
        assert_eq!(suggestion.status, SuggestionDecisionStatus::Pending);
    }
}

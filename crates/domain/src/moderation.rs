use std::sync::Arc;
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::auth::Role;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::moderation::ModerationRepository;
use crate::util::uuid_v7_without_dashes;

const MODERATION_REASONS_MAX_LEN: usize = 512;
const MODERATION_CONFIDENCE_MAX: f64 = 1.0;
const MODERATION_CONFIDENCE_MIN: f64 = 0.0;
const MODERATION_LIMIT_APPEAL_DAYS: i64 = 7;
const MODERATION_HOLD_MINUTES_DEFAULT: i64 = 15;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModerationStatus {
    Processing,
    UnderReview,
    Published,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAction {
    PublishNow,
    PublishWithWarning,
    HoldForReview,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModerationStatusParseError {
    Unknown,
}

impl fmt::Display for ModerationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Processing => write!(f, "processing"),
            Self::UnderReview => write!(f, "under_review"),
            Self::Published => write!(f, "published"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

impl FromStr for ModerationStatus {
    type Err = ModerationStatusParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "processing" => Ok(Self::Processing),
            "under_review" => Ok(Self::UnderReview),
            "published" => Ok(Self::Published),
            "rejected" => Ok(Self::Rejected),
            _ => Err(ModerationStatusParseError::Unknown),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModerationViolation {
    pub category: String,
    pub severity: Option<String>,
    pub snippet: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationActorSnapshot {
    pub user_id: String,
    pub username: String,
    pub token_role: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl ModerationActorSnapshot {
    pub fn new(
        actor: ActorIdentity,
        token_role: &Role,
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        request_ts_ms: i64,
    ) -> Self {
        Self {
            user_id: actor.user_id,
            username: actor.username,
            token_role: token_role.as_str().to_string(),
            request_id: request_id.into(),
            correlation_id: correlation_id.into(),
            request_ts_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationDecision {
    pub decision_id: String,
    pub content_id: String,
    pub content_type: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub reason_code: Option<String>,
    pub confidence: f64,
    pub decided_at_ms: i64,
    pub actor: ModerationActorSnapshot,
    pub hold_expires_at_ms: Option<i64>,
    pub auto_release_if_no_action: bool,
    pub appeal_window_until_ms: Option<i64>,
    pub reasoning: Option<String>,
    pub violations: Vec<ModerationViolation>,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContentModeration {
    pub content_id: String,
    pub content_type: Option<String>,
    pub author_id: String,
    pub author_username: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub reason_code: Option<String>,
    pub confidence: f64,
    pub decided_at_ms: i64,
    pub decided_by: String,
    pub hold_expires_at_ms: Option<i64>,
    pub auto_release_if_no_action: bool,
    pub violations: Vec<ModerationViolation>,
    pub appeal_window_until_ms: Option<i64>,
    pub reasoning: Option<String>,
    pub last_decision_id: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl ContentModeration {
    pub fn new(content_id: String, author_id: String, author_username: Option<String>) -> Self {
        let now = now_ms();
        Self {
            content_id,
            content_type: None,
            author_id,
            author_username,
            moderation_status: ModerationStatus::Processing,
            moderation_action: ModerationAction::HoldForReview,
            reason_code: None,
            confidence: 1.0,
            decided_at_ms: now,
            decided_by: "system".to_string(),
            hold_expires_at_ms: None,
            auto_release_if_no_action: false,
            violations: vec![],
            appeal_window_until_ms: None,
            reasoning: None,
            last_decision_id: None,
            request_id: "boot".to_string(),
            correlation_id: "boot".to_string(),
            request_ts_ms: now,
        }
    }

    fn apply_decision(&mut self, decision: &ModerationDecision) {
        self.moderation_status = decision.moderation_status.clone();
        self.moderation_action = decision.moderation_action.clone();
        self.reason_code = decision.reason_code.clone();
        self.confidence = decision.confidence;
        self.decided_at_ms = decision.decided_at_ms;
        self.decided_by = decision.actor.user_id.clone();
        self.hold_expires_at_ms = decision.hold_expires_at_ms;
        self.auto_release_if_no_action = decision.auto_release_if_no_action;
        self.appeal_window_until_ms = decision.appeal_window_until_ms;
        self.reasoning = decision.reasoning.clone();
        self.violations = decision.violations.clone();
        self.last_decision_id = Some(decision.decision_id.clone());
        self.request_id = decision.request_id.clone();
        self.correlation_id = decision.correlation_id.clone();
        self.request_ts_ms = decision.actor.request_ts_ms;
        if self.content_type.is_none() {
            self.content_type = decision.content_type.clone();
        }
    }

    pub fn is_visible(&self) -> bool {
        matches!(self.moderation_status, ModerationStatus::Published)
    }

    pub fn published_at_ms(&self) -> Option<i64> {
        match self.moderation_status {
            ModerationStatus::Published => Some(self.decided_at_ms),
            _ => None,
        }
    }

    pub fn placeholder(&self) -> Option<&'static str> {
        match self.moderation_status {
            ModerationStatus::UnderReview => Some("Catatan sedang ditinjau"),
            ModerationStatus::Rejected => Some("Catatan ditolak"),
            _ => None,
        }
    }

    pub fn to_public_view(&self) -> ModerationPublicView {
        ModerationPublicView {
            content_id: self.content_id.clone(),
            content_type: self.content_type.clone(),
            moderation_status: self.moderation_status.clone(),
            moderation_action: self.moderation_action.clone(),
            published_at_ms: self.published_at_ms(),
            is_visible: self.is_visible(),
            placeholder: self.placeholder().map(|value| value.to_string()),
            redacted_summary: None,
            warning_label: if matches!(
                self.moderation_status,
                ModerationStatus::UnderReview | ModerationStatus::Rejected
            ) {
                self.reason_code
                    .clone()
                    .or_else(|| Some("policy_gate".to_string()))
            } else {
                None
            },
        }
    }

    pub fn to_author_view(&self) -> ModerationAuthorView {
        ModerationAuthorView {
            content_id: self.content_id.clone(),
            content_type: self.content_type.clone(),
            moderation_status: self.moderation_status.clone(),
            moderation_action: self.moderation_action.clone(),
            decided_at_ms: self.decided_at_ms,
            decided_by: self.decided_by.clone(),
            reason_code: self.reason_code.clone(),
            confidence: self.confidence,
            hold_expires_at_ms: self.hold_expires_at_ms,
            auto_release_if_no_action: self.auto_release_if_no_action,
            appeal_window_until_ms: self.appeal_window_until_ms,
            warning_label: self.reason_code.clone(),
            published_at_ms: self.published_at_ms(),
        }
    }

    pub fn to_moderator_view(&self, decisions: Vec<ModerationDecision>) -> ModerationModeratorView {
        ModerationModeratorView {
            content_id: self.content_id.clone(),
            content_type: self.content_type.clone(),
            author_id: self.author_id.clone(),
            author_username: self.author_username.clone(),
            moderation_status: self.moderation_status.clone(),
            moderation_action: self.moderation_action.clone(),
            decided_at_ms: self.decided_at_ms,
            decided_by: self.decided_by.clone(),
            reason_code: self.reason_code.clone(),
            confidence: self.confidence,
            hold_expires_at_ms: self.hold_expires_at_ms,
            auto_release_if_no_action: self.auto_release_if_no_action,
            appeal_window_until_ms: self.appeal_window_until_ms,
            reasoning: self.reasoning.clone(),
            violations: self.violations.clone(),
            decision_count: decisions.len(),
            decisions,
            warnings: vec![],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationPublicView {
    pub content_id: String,
    pub content_type: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub published_at_ms: Option<i64>,
    pub is_visible: bool,
    pub placeholder: Option<String>,
    pub warning_label: Option<String>,
    pub redacted_summary: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationAuthorView {
    pub content_id: String,
    pub content_type: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub decided_at_ms: i64,
    pub decided_by: String,
    pub reason_code: Option<String>,
    pub confidence: f64,
    pub hold_expires_at_ms: Option<i64>,
    pub auto_release_if_no_action: bool,
    pub appeal_window_until_ms: Option<i64>,
    pub warning_label: Option<String>,
    pub published_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationModeratorView {
    pub content_id: String,
    pub content_type: Option<String>,
    pub author_id: String,
    pub author_username: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub decided_at_ms: i64,
    pub decided_by: String,
    pub reason_code: Option<String>,
    pub confidence: f64,
    pub hold_expires_at_ms: Option<i64>,
    pub auto_release_if_no_action: bool,
    pub appeal_window_until_ms: Option<i64>,
    pub reasoning: Option<String>,
    pub violations: Vec<ModerationViolation>,
    pub decision_count: usize,
    pub decisions: Vec<ModerationDecision>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub enum ModerationView {
    #[allow(clippy::large_enum_variant)]
    Public(ModerationPublicView),
    Author(ModerationAuthorView),
    Moderator(ModerationModeratorView),
}

#[derive(Clone, Debug)]
pub struct ModerationApplyCommand {
    pub content_id: String,
    pub content_type: Option<String>,
    pub author_id: Option<String>,
    pub author_username: Option<String>,
    pub moderation_status: ModerationStatus,
    pub moderation_action: ModerationAction,
    pub reason_code: Option<String>,
    pub confidence: f64,
    pub hold_duration_minutes: Option<i64>,
    pub auto_release_if_no_action: bool,
    pub appeal_window_minutes: Option<i64>,
    pub reasoning: Option<String>,
    pub violations: Vec<ModerationViolation>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ModerationAutoReleaseCommand {
    pub content_id: String,
    pub hold_decision_request_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub scheduled_ms: i64,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ModerationApplyResult {
    pub content: ContentModeration,
    pub decision: ModerationDecision,
    pub schedule_auto_release: bool,
}

#[derive(Clone)]
pub struct ModerationService {
    repository: Arc<dyn ModerationRepository>,
}

impl ModerationService {
    pub fn new(repository: Arc<dyn ModerationRepository>) -> Self {
        Self { repository }
    }

    pub async fn upsert_moderation_decision(
        &self,
        actor: ActorIdentity,
        token_role: Role,
        mut input: ModerationApplyCommand,
    ) -> DomainResult<ModerationApplyResult> {
        ensure_decision_authorized(&token_role)?;
        input = validate_moderation_input(input)?;

        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let actor_snapshot = ModerationActorSnapshot::new(
            actor.clone(),
            &token_role,
            input.request_id.clone(),
            input.correlation_id.clone(),
            request_ts_ms,
        );

        let existing = self
            .repository
            .get_content_moderation(&input.content_id)
            .await?;
        let author_id = existing
            .as_ref()
            .and_then(|current| Some(current.author_id.clone()))
            .or_else(|| input.author_id.clone())
            .ok_or_else(|| {
                DomainError::Validation(
                    "author_id is required for first moderation record".to_string(),
                )
            })?;
        let author_username = existing
            .as_ref()
            .and_then(|current| current.author_username.clone())
            .or(input.author_username.clone());

        let hold_expires_at_ms = if input.moderation_action == ModerationAction::HoldForReview {
            Some(
                request_ts_ms
                    + input
                        .hold_duration_minutes
                        .unwrap_or(MODERATION_HOLD_MINUTES_DEFAULT)
                        * 60_000,
            )
        } else {
            None
        };
        let auto_release = input.moderation_action == ModerationAction::HoldForReview
            && input.auto_release_if_no_action;
        let appeal_window_until_ms = input
            .appeal_window_minutes
            .map(|minutes| request_ts_ms + minutes * 60_000)
            .or_else(|| Some(request_ts_ms + MODERATION_LIMIT_APPEAL_DAYS * 24 * 60 * 60 * 1000));

        let decision = ModerationDecision {
            decision_id: uuid_v7_without_dashes(),
            content_id: input.content_id.clone(),
            content_type: input.content_type.clone(),
            moderation_status: input.moderation_status.clone(),
            moderation_action: input.moderation_action.clone(),
            reason_code: input.reason_code.clone(),
            confidence: input.confidence,
            decided_at_ms: request_ts_ms,
            actor: actor_snapshot,
            hold_expires_at_ms,
            auto_release_if_no_action: auto_release,
            appeal_window_until_ms,
            reasoning: input.reasoning.clone(),
            violations: input.violations,
            request_id: input.request_id.clone(),
            correlation_id: input.correlation_id.clone(),
        };

        let decision = match self.repository.create_decision(&decision).await {
            Ok(decision) => decision,
            Err(DomainError::Conflict) => {
                return Ok(self
                    .repository
                    .get_decision_by_request(&input.content_id, &input.request_id)
                    .await?
                    .map(|decision| {
                        Ok(ModerationApplyResult {
                            content: existing.unwrap_or_else(|| {
                                ContentModeration::new(
                                    input.content_id.clone(),
                                    author_id,
                                    author_username,
                                )
                            }),
                            decision,
                            schedule_auto_release: false,
                        })
                    })
                    .unwrap_or_else(|| Err(DomainError::Conflict))?);
            }
            Err(err) => return Err(err),
        };

        let mut state = existing.unwrap_or_else(|| {
            ContentModeration::new(input.content_id.clone(), author_id, author_username)
        });
        if state.content_type.is_none() {
            state.content_type = input.content_type;
        }
        state.apply_decision(&decision);
        let content = self.repository.upsert_content_moderation(&state).await?;

        let schedule_auto_release =
            matches!(state.moderation_status, ModerationStatus::UnderReview)
                && state.auto_release_if_no_action
                && state.hold_expires_at_ms.is_some();
        Ok(ModerationApplyResult {
            content,
            decision,
            schedule_auto_release,
        })
    }

    pub async fn apply_auto_release(
        &self,
        actor: ActorIdentity,
        token_role: Role,
        input: ModerationAutoReleaseCommand,
    ) -> DomainResult<ModerationApplyResult> {
        ensure_auto_release_role(&token_role)?;
        let request_ts_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        if request_ts_ms < input.scheduled_ms {
            return Err(DomainError::Validation(
                "release job cannot run before hold expiry".to_string(),
            ));
        }

        let current = self
            .repository
            .get_content_moderation(&input.content_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if current.request_id != input.hold_decision_request_id {
            return Ok(ModerationApplyResult {
                content: current.clone(),
                decision: build_auto_release_noop_decision(
                    &current,
                    &actor,
                    &token_role,
                    &input,
                    request_ts_ms,
                    "auto_release_stale_request",
                    "auto release skipped because hold decision id does not match latest decision",
                ),
                schedule_auto_release: false,
            });
        }

        if current.moderation_status != ModerationStatus::UnderReview {
            return Ok(ModerationApplyResult {
                content: current.clone(),
                decision: build_auto_release_noop_decision(
                    &current,
                    &actor,
                    &token_role,
                    &input,
                    request_ts_ms,
                    "auto_release_not_applicable",
                    "auto release skipped because content is not under review",
                ),
                schedule_auto_release: false,
            });
        }

        if !current.auto_release_if_no_action {
            return Ok(ModerationApplyResult {
                content: current.clone(),
                decision: build_auto_release_noop_decision(
                    &current,
                    &actor,
                    &token_role,
                    &input,
                    request_ts_ms,
                    "auto_release_disabled",
                    "auto release disabled for current moderation decision",
                ),
                schedule_auto_release: false,
            });
        }

        let hold_expires = current
            .hold_expires_at_ms
            .ok_or_else(|| DomainError::Validation("content is missing hold expiry".to_string()))?;
        if request_ts_ms < hold_expires {
            return Err(DomainError::Validation(
                "auto release job fired before hold expiry".to_string(),
            ));
        }

        let command = ModerationApplyCommand {
            content_id: current.content_id.clone(),
            content_type: current.content_type.clone(),
            author_id: Some(current.author_id),
            author_username: current.author_username.clone(),
            moderation_status: ModerationStatus::Published,
            moderation_action: ModerationAction::PublishNow,
            reason_code: Some("auto_release".to_string()),
            confidence: current.confidence,
            hold_duration_minutes: None,
            auto_release_if_no_action: false,
            appeal_window_minutes: None,
            reasoning: Some(
                "otomatis dipublikasikan karena tidak ada aksi moderasi dalam jangka waktu yang disyaratkan"
                    .to_string(),
            ),
            violations: current.violations.clone(),
            request_id: input.request_id,
            correlation_id: input.correlation_id,
            request_ts_ms: Some(request_ts_ms),
        };
        let mut released = self
            .upsert_moderation_decision(actor, token_role, command)
            .await?;
        released.schedule_auto_release = false;
        Ok(released)
    }

    pub async fn get_moderation_view(
        &self,
        content_id: &str,
        actor: &ActorIdentity,
        token_role: &Role,
    ) -> DomainResult<ModerationView> {
        let moderation = self
            .repository
            .get_content_moderation(content_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        if token_role.can_moderate() {
            let decisions = self
                .repository
                .list_decisions(&moderation.content_id)
                .await?;
            return Ok(ModerationView::Moderator(
                moderation.to_moderator_view(decisions),
            ));
        }
        if actor.user_id == moderation.author_id {
            return Ok(ModerationView::Author(moderation.to_author_view()));
        }
        Ok(ModerationView::Public(moderation.to_public_view()))
    }

    pub async fn list_review_queue(
        &self,
        token_role: &Role,
        limit: usize,
    ) -> DomainResult<Vec<ContentModeration>> {
        if !token_role.can_moderate() {
            return Err(DomainError::Validation(
                "only moderators can list moderation queue".to_string(),
            ));
        }
        self.repository
            .list_content_by_status("under_review", limit)
            .await
    }
}

fn validate_moderation_input(
    mut input: ModerationApplyCommand,
) -> DomainResult<ModerationApplyCommand> {
    input.content_id = input.content_id.trim().to_string();
    if input.content_id.is_empty() {
        return Err(DomainError::Validation("content_id is required".into()));
    }
    input.author_id = input.author_id.take().filter(|id| !id.trim().is_empty());
    if input.hold_duration_minutes.is_none()
        && input.moderation_action == ModerationAction::HoldForReview
    {
        return Err(DomainError::Validation(
            "hold_duration_minutes is required for hold_for_review".to_string(),
        ));
    }
    if input.hold_duration_minutes.is_some_and(|value| value < 1) {
        return Err(DomainError::Validation(
            "hold_duration_minutes must be at least 1".to_string(),
        ));
    }
    if !(MODERATION_CONFIDENCE_MIN..=MODERATION_CONFIDENCE_MAX).contains(&input.confidence) {
        return Err(DomainError::Validation(
            "confidence must be between 0 and 1".to_string(),
        ));
    }
    if matches!(input.moderation_status, ModerationStatus::UnderReview)
        && input.moderation_action != ModerationAction::HoldForReview
    {
        return Err(DomainError::Validation(
            "under_review status requires hold_for_review action".to_string(),
        ));
    }
    if matches!(input.moderation_action, ModerationAction::HoldForReview)
        && input.moderation_status != ModerationStatus::UnderReview
    {
        return Err(DomainError::Validation(
            "hold_for_review action requires under_review status".to_string(),
        ));
    }
    if input.auto_release_if_no_action && input.moderation_action != ModerationAction::HoldForReview
    {
        return Err(DomainError::Validation(
            "auto_release_if_no_action requires hold_for_review".to_string(),
        ));
    }
    if let Some(reason) = &input.reason_code {
        if reason.len() > MODERATION_REASONS_MAX_LEN {
            return Err(DomainError::Validation(
                "reason_code exceeds max length".to_string(),
            ));
        }
    }
    if input.request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if input.correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }
    Ok(input)
}

fn ensure_decision_authorized(token_role: &Role) -> DomainResult<()> {
    if token_role.can_moderate() {
        return Ok(());
    }
    Err(DomainError::Validation(
        "moderator privilege required for moderation decisions".to_string(),
    ))
}

fn ensure_auto_release_role(token_role: &Role) -> DomainResult<()> {
    if token_role == &Role::System || token_role == &Role::Admin {
        return Ok(());
    }
    Err(DomainError::Forbidden(
        "auto release requires admin/system role".into(),
    ))
}

fn build_auto_release_noop_decision(
    current: &ContentModeration,
    actor: &ActorIdentity,
    token_role: &Role,
    input: &ModerationAutoReleaseCommand,
    decided_at_ms: i64,
    reason_code: &str,
    reasoning: &str,
) -> ModerationDecision {
    ModerationDecision {
        decision_id: format!(
            "noop:{}:{}",
            input.request_id, input.hold_decision_request_id
        ),
        content_id: current.content_id.clone(),
        content_type: current.content_type.clone(),
        moderation_status: current.moderation_status.clone(),
        moderation_action: current.moderation_action.clone(),
        reason_code: Some(reason_code.to_string()),
        confidence: current.confidence,
        decided_at_ms,
        actor: ModerationActorSnapshot::new(
            actor.clone(),
            token_role,
            input.request_id.clone(),
            input.correlation_id.clone(),
            decided_at_ms,
        ),
        hold_expires_at_ms: current.hold_expires_at_ms,
        auto_release_if_no_action: current.auto_release_if_no_action,
        appeal_window_until_ms: current.appeal_window_until_ms,
        reasoning: Some(reasoning.to_string()),
        violations: current.violations.clone(),
        request_id: input.request_id.clone(),
        correlation_id: input.correlation_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockModerationRepository {
        content: Arc<RwLock<HashMap<String, ContentModeration>>>,
        decisions: Arc<RwLock<HashMap<String, ModerationDecision>>>,
        by_content_request: Arc<RwLock<HashMap<(String, String), String>>>,
    }

    impl MockModerationRepository {
        fn new_key(content_id: &str, request_id: &str) -> (String, String) {
            (content_id.to_string(), request_id.to_string())
        }
    }

    impl ModerationRepository for MockModerationRepository {
        fn upsert_content_moderation(
            &self,
            content: &ContentModeration,
        ) -> BoxFuture<'_, DomainResult<ContentModeration>> {
            let content = content.clone();
            let map = self.content.clone();
            Box::pin(async move {
                map.write()
                    .await
                    .insert(content.content_id.clone(), content.clone());
                Ok(content)
            })
        }

        fn get_content_moderation(
            &self,
            content_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ContentModeration>>> {
            let key = content_id.to_string();
            let map = self.content.clone();
            Box::pin(async move { Ok(map.read().await.get(&key).cloned()) })
        }

        fn list_content_by_status(
            &self,
            status: &str,
            limit: usize,
        ) -> BoxFuture<'_, DomainResult<Vec<ContentModeration>>> {
            let status = status.to_string();
            let map = self.content.clone();
            Box::pin(async move {
                let mut rows: Vec<_> = map
                    .read()
                    .await
                    .values()
                    .filter(|content| content.moderation_status.to_string() == status)
                    .cloned()
                    .collect();
                rows.sort_by(|left, right| {
                    left.decided_at_ms
                        .cmp(&right.decided_at_ms)
                        .then_with(|| left.content_id.cmp(&right.content_id))
                });
                rows.truncate(limit);
                Ok(rows)
            })
        }

        fn create_decision(
            &self,
            decision: &ModerationDecision,
        ) -> BoxFuture<'_, DomainResult<ModerationDecision>> {
            let decision = decision.clone();
            let decisions = self.decisions.clone();
            let by_request = self.by_content_request.clone();
            Box::pin(async move {
                let key =
                    MockModerationRepository::new_key(&decision.content_id, &decision.request_id);
                if decisions.read().await.contains_key(&decision.decision_id) {
                    return Err(DomainError::Conflict);
                }
                if by_request.read().await.contains_key(&key) {
                    return Err(DomainError::Conflict);
                }
                by_request
                    .write()
                    .await
                    .insert(key, decision.decision_id.clone());
                decisions
                    .write()
                    .await
                    .insert(decision.decision_id.clone(), decision.clone());
                Ok(decision)
            })
        }

        fn get_decision_by_request(
            &self,
            content_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<ModerationDecision>>> {
            let key = MockModerationRepository::new_key(content_id, request_id);
            let by_request = self.by_content_request.clone();
            let decisions = self.decisions.clone();
            Box::pin(async move {
                let Some(decision_id) = by_request.read().await.get(&key).cloned() else {
                    return Ok(None);
                };
                let decisions = decisions.read().await;
                Ok(decisions.get(&decision_id).cloned())
            })
        }

        fn list_decisions(
            &self,
            content_id: &str,
        ) -> BoxFuture<'_, DomainResult<Vec<ModerationDecision>>> {
            let target = content_id.to_string();
            let decisions = self.decisions.clone();
            Box::pin(async move {
                let mut rows: Vec<_> = decisions
                    .read()
                    .await
                    .values()
                    .filter(|decision| decision.content_id == target)
                    .cloned()
                    .collect();
                rows.sort_by(|left, right| {
                    left.decided_at_ms
                        .cmp(&right.decided_at_ms)
                        .then_with(|| left.decision_id.cmp(&right.decision_id))
                });
                Ok(rows)
            })
        }
    }

    fn actor_identity() -> ActorIdentity {
        ActorIdentity {
            user_id: "mod-1".to_string(),
            username: "mod".to_string(),
        }
    }

    #[tokio::test]
    async fn moderation_idempotent_decisions_replay() {
        let repository = Arc::new(MockModerationRepository::default());
        let service = ModerationService::new(repository);
        let actor = actor_identity();
        let command = ModerationApplyCommand {
            content_id: "content-1".to_string(),
            content_type: Some("test".to_string()),
            author_id: Some("user-1".to_string()),
            author_username: Some("alice".to_string()),
            moderation_status: ModerationStatus::UnderReview,
            moderation_action: ModerationAction::HoldForReview,
            reason_code: Some("policy_hint".to_string()),
            confidence: 0.8,
            hold_duration_minutes: Some(15),
            auto_release_if_no_action: true,
            appeal_window_minutes: Some(30),
            reasoning: Some("policy violation".to_string()),
            violations: vec![],
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
            request_ts_ms: Some(1_000),
        };

        let first = service
            .upsert_moderation_decision(actor.clone(), Role::Moderator, command.clone())
            .await
            .expect("first decision");
        let second = service
            .upsert_moderation_decision(actor, Role::Moderator, command)
            .await
            .expect("second decision");

        assert_eq!(first.decision.decision_id, second.decision.decision_id);
        assert_eq!(first.content.content_id, second.content.content_id);
        assert!(first.schedule_auto_release);
        assert!(!second.schedule_auto_release);
    }

    #[tokio::test]
    async fn moderation_rejects_low_confidence_without_bounds() {
        let repository = Arc::new(MockModerationRepository::default());
        let service = ModerationService::new(repository);
        let actor = actor_identity();
        let command = ModerationApplyCommand {
            content_id: "content-2".to_string(),
            content_type: None,
            author_id: Some("user-1".to_string()),
            author_username: Some("alice".to_string()),
            moderation_status: ModerationStatus::UnderReview,
            moderation_action: ModerationAction::HoldForReview,
            reason_code: Some("policy_hint".to_string()),
            confidence: 1.5,
            hold_duration_minutes: Some(15),
            auto_release_if_no_action: true,
            appeal_window_minutes: None,
            reasoning: None,
            violations: vec![],
            request_id: "req-bad".to_string(),
            correlation_id: "corr-bad".to_string(),
            request_ts_ms: None,
        };

        let err = service
            .upsert_moderation_decision(actor, Role::Moderator, command)
            .await
            .expect_err("validation");
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[tokio::test]
    async fn moderation_auto_release_stale_request_is_noop() {
        let repository = Arc::new(MockModerationRepository::default());
        let service = ModerationService::new(repository.clone());
        let actor = actor_identity();

        let hold_base = ModerationApplyCommand {
            content_id: "content-4".to_string(),
            content_type: Some("text".to_string()),
            author_id: Some("user-4".to_string()),
            author_username: Some("alice".to_string()),
            moderation_status: ModerationStatus::UnderReview,
            moderation_action: ModerationAction::HoldForReview,
            reason_code: Some("policy_hint".to_string()),
            confidence: 0.9,
            hold_duration_minutes: Some(5),
            auto_release_if_no_action: true,
            appeal_window_minutes: Some(7),
            reasoning: Some("initial hold".to_string()),
            violations: vec![],
            request_id: "req-4-v1".to_string(),
            correlation_id: "corr-4-v1".to_string(),
            request_ts_ms: Some(1_000),
        };
        service
            .upsert_moderation_decision(actor.clone(), Role::Moderator, hold_base.clone())
            .await
            .expect("initial hold");

        let second_hold = ModerationApplyCommand {
            request_id: "req-4-v2".to_string(),
            correlation_id: "corr-4-v2".to_string(),
            hold_duration_minutes: Some(120),
            request_ts_ms: Some(2_000),
            ..hold_base
        };
        service
            .upsert_moderation_decision(actor.clone(), Role::Moderator, second_hold)
            .await
            .expect("override hold");

        let stale_job = ModerationAutoReleaseCommand {
            content_id: "content-4".to_string(),
            hold_decision_request_id: "req-4-v1".to_string(),
            request_id: "auto-stale-req".to_string(),
            correlation_id: "auto-stale-corr".to_string(),
            scheduled_ms: 10_000,
            request_ts_ms: Some(80_000),
        };
        let result = service
            .apply_auto_release(actor, Role::System, stale_job)
            .await
            .expect("stale auto release");

        assert_eq!(
            result.decision.reason_code.as_deref(),
            Some("auto_release_stale_request")
        );
        assert_eq!(
            result.decision.reasoning.as_deref(),
            Some("auto release skipped because hold decision id does not match latest decision")
        );
        assert!(matches!(
            result.content.moderation_status,
            ModerationStatus::UnderReview
        ));
        assert!(!result.schedule_auto_release);
        assert_eq!(result.content.request_id, "req-4-v2");
        assert!(
            repository
                .get_decision_by_request("content-4", "auto-stale-req")
                .await
                .expect("decision replay")
                .is_none()
        );
    }

    #[tokio::test]
    async fn moderation_public_author_moderator_projection_are_role_bound() {
        let repository = Arc::new(MockModerationRepository::default());
        let service = ModerationService::new(repository);
        let actor_mod = actor_identity();
        let command = ModerationApplyCommand {
            content_id: "content-3".to_string(),
            content_type: None,
            author_id: Some("user-2".to_string()),
            author_username: Some("bob".to_string()),
            moderation_status: ModerationStatus::UnderReview,
            moderation_action: ModerationAction::HoldForReview,
            reason_code: Some("policy_hint".to_string()),
            confidence: 0.7,
            hold_duration_minutes: Some(15),
            auto_release_if_no_action: false,
            appeal_window_minutes: Some(15),
            reasoning: Some("manual".to_string()),
            violations: vec![ModerationViolation {
                category: "hate".to_string(),
                severity: Some("high".to_string()),
                snippet: Some("text".to_string()),
                reason: Some("offensive".to_string()),
            }],
            request_id: "req-3".to_string(),
            correlation_id: "corr-3".to_string(),
            request_ts_ms: Some(2_000),
        };
        service
            .upsert_moderation_decision(actor_mod.clone(), Role::Moderator, command)
            .await
            .expect("seed decision");

        let author_view = service
            .get_moderation_view(
                "content-3",
                &ActorIdentity {
                    user_id: "user-2".to_string(),
                    username: "bob".to_string(),
                },
                &Role::User,
            )
            .await
            .expect("author view");
        assert!(matches!(author_view, ModerationView::Author(_)));

        let public_view = service
            .get_moderation_view(
                "content-3",
                &ActorIdentity {
                    user_id: "other".to_string(),
                    username: "x".to_string(),
                },
                &Role::User,
            )
            .await
            .expect("public view");
        assert!(matches!(public_view, ModerationView::Public(_)));

        let mod_view = service
            .get_moderation_view("content-3", &actor_mod, &Role::Moderator)
            .await
            .expect("mod view");
        assert!(matches!(mod_view, ModerationView::Moderator(_)));
    }
}

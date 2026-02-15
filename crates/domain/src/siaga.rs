use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::auth::Role;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::siaga::SiagaRepository;

pub const MAX_SEVERITY: u8 = 5;
pub const MIN_SEVERITY: u8 = 1;
const MAX_TEXT_LEN: usize = 2_000;
const MAX_TITLE_LEN: usize = 160;
const MAX_LOCATION_LEN: usize = 256;
const RESPONDER_ANONYMIZE_AFTER_MS: i64 = 7 * 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SiagaState {
    Draft,
    Active,
    Resolved,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SiagaResponderStatus {
    Watching,
    Coming,
    OnSite,
    Completed,
    Unable,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiagaActorSnapshot {
    pub user_id: String,
    pub username: String,
    pub token_role: String,
    pub is_author: bool,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl SiagaActorSnapshot {
    pub fn new(
        actor: &ActorIdentity,
        token_role: &Role,
        is_author: bool,
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        request_ts_ms: i64,
    ) -> Self {
        Self {
            user_id: actor.user_id.clone(),
            username: actor.username.clone(),
            token_role: token_role.as_str().to_string(),
            is_author,
            request_id: request_id.into(),
            correlation_id: correlation_id.into(),
            request_ts_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SiagaResponder {
    pub responder_id: String,
    pub username: String,
    pub status: SiagaResponderStatus,
    pub joined_at_ms: i64,
    pub updated_at_ms: i64,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SiagaClosure {
    pub reason: String,
    pub summary: String,
    pub closed_by: String,
    pub closed_at_ms: i64,
    pub counters: SiagaCounters,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SiagaCounters {
    pub total_responders: usize,
    pub by_status: HashMap<SiagaResponderStatus, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SiagaBroadcast {
    pub siaga_id: String,
    pub scope_id: String,
    pub author_id: String,
    pub author_username: String,
    pub emergency_type: String,
    pub severity: u8,
    pub location: String,
    pub title: String,
    pub text: String,
    pub state: SiagaState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub request_id: String,
    pub correlation_id: String,
    pub responders: Vec<SiagaResponder>,
    pub closure: Option<SiagaClosure>,
    pub event_hash: String,
    pub retention_tag: String,
}

impl SiagaBroadcast {
    pub fn counters(&self) -> SiagaCounters {
        let mut by_status = HashMap::new();
        for responder in &self.responders {
            *by_status.entry(responder.status.clone()).or_insert(0) += 1;
        }

        SiagaCounters {
            total_responders: self.responders.len(),
            by_status,
        }
    }

    pub fn reveal_responder_identity(
        &self,
        actor_id: &str,
        responder: &SiagaResponder,
        now_ms: i64,
    ) -> bool {
        if actor_id == self.author_id {
            return true;
        }
        if actor_id == responder.responder_id {
            return true;
        }
        let age_ms = now_ms.saturating_sub(responder.joined_at_ms);
        age_ms <= RESPONDER_ANONYMIZE_AFTER_MS
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SiagaTimelineEventType {
    SiagaBroadcastCreated,
    SiagaBroadcastActivated,
    SiagaBroadcastUpdated,
    SiagaResponderJoined,
    SiagaResponderUpdated,
    SiagaBroadcastClosed,
    SiagaBroadcastCancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SiagaTimelineEvent {
    pub event_id: String,
    pub siaga_id: String,
    pub event_type: SiagaTimelineEventType,
    pub actor: SiagaActorSnapshot,
    pub request_id: String,
    pub correlation_id: String,
    pub occurred_at_ms: i64,
    pub metadata: Option<serde_json::Value>,
    pub event_hash: String,
    pub retention_tag: String,
}

#[derive(Clone)]
pub struct CreateSiagaBroadcast {
    pub scope_id: String,
    pub emergency_type: String,
    pub severity: u8,
    pub location: String,
    pub title: String,
    pub text: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct UpdateSiagaBroadcast {
    pub scope_id: Option<String>,
    pub emergency_type: Option<String>,
    pub severity: Option<u8>,
    pub location: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct ActivateSiagaBroadcast {
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct JoinSiagaResponder {
    pub status: SiagaResponderStatus,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct UpdateResponderStatus {
    pub status: SiagaResponderStatus,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct CloseSiagaBroadcast {
    pub reason: String,
    pub summary: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct CancelSiagaBroadcast {
    pub reason: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

pub struct SiagaService {
    repository: std::sync::Arc<dyn SiagaRepository>,
}

impl SiagaService {
    pub fn new(repository: std::sync::Arc<dyn SiagaRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_broadcast(
        &self,
        actor: ActorIdentity,
        role: &Role,
        input: CreateSiagaBroadcast,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let input = validate_create_broadcast(input)?;
        ensure_authenticated(role)?;

        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let siaga_id = crate::util::uuid_v7_without_dashes();
        let snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            true,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );

        let mut broadcast = SiagaBroadcast {
            siaga_id: siaga_id.clone(),
            scope_id: input.scope_id,
            author_id: actor.user_id,
            author_username: actor.username,
            emergency_type: input.emergency_type,
            severity: input.severity,
            location: input.location,
            title: input.title,
            text: input.text,
            state: SiagaState::Draft,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
            request_id: input.request_id.clone(),
            correlation_id: input.correlation_id.clone(),
            responders: vec![],
            closure: None,
            event_hash: String::new(),
            retention_tag: String::new(),
        };
        broadcast = apply_siaga_broadcast_audit(broadcast)?;

        let event = make_siaga_event(
            siaga_id,
            SiagaTimelineEventType::SiagaBroadcastCreated,
            snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({
                "emergency_type": broadcast.emergency_type,
                "severity": broadcast.severity,
                "title": broadcast.title,
            })),
        )?;

        match self.repository.create_broadcast(&broadcast, &event).await {
            Ok(broadcast) => Ok(broadcast),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_actor_request(&broadcast.author_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn get(&self, siaga_id: &str) -> crate::DomainResult<SiagaBroadcast> {
        self.repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn list_by_scope(&self, scope_id: &str) -> crate::DomainResult<Vec<SiagaBroadcast>> {
        self.repository.list_by_scope(scope_id).await
    }

    pub async fn timeline(&self, siaga_id: &str) -> crate::DomainResult<Vec<SiagaTimelineEvent>> {
        self.repository.list_timeline(siaga_id).await
    }

    pub async fn update_broadcast(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        input: UpdateSiagaBroadcast,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let input = validate_update_broadcast(input)?;
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        if !is_author_or_admin(actor.user_id.as_str(), &broadcast.author_id, role) {
            return Err(DomainError::Forbidden(
                "only author can update siaga".into(),
            ));
        }

        if matches!(
            broadcast.state,
            SiagaState::Resolved | SiagaState::Cancelled
        ) {
            return Err(DomainError::Validation(
                "cannot update broadcast in resolved or cancelled state".into(),
            ));
        }

        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let mut updated = broadcast;
        if let Some(scope_id) = input.scope_id {
            updated.scope_id = scope_id;
        }
        if let Some(emergency_type) = input.emergency_type {
            updated.emergency_type = emergency_type;
        }
        if let Some(severity) = input.severity {
            updated.severity = severity;
        }
        if let Some(location) = input.location {
            updated.location = location;
        }
        if let Some(title) = input.title {
            updated.title = title;
        }
        if let Some(text) = input.text {
            updated.text = text;
        }

        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            true,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );

        let event = make_siaga_event(
            updated.siaga_id.clone(),
            SiagaTimelineEventType::SiagaBroadcastUpdated,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({ "state": updated.state })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(broadcast) => Ok(broadcast),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn activate(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        input: ActivateSiagaBroadcast,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if !is_author_or_admin(actor.user_id.as_str(), &broadcast.author_id, role) {
            return Err(DomainError::Forbidden(
                "only author can activate siaga".into(),
            ));
        }

        match broadcast.state {
            SiagaState::Draft => {}
            SiagaState::Active => {
                return Ok(broadcast);
            }
            SiagaState::Resolved | SiagaState::Cancelled => {
                return Err(DomainError::Validation(
                    "cannot activate resolved or cancelled broadcast".into(),
                ));
            }
        }

        let mut updated = broadcast;
        updated.state = SiagaState::Active;
        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            true,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );
        let event = make_siaga_event(
            updated.siaga_id.clone(),
            SiagaTimelineEventType::SiagaBroadcastActivated,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({ "state": updated.state })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(updated) => Ok(updated),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn join_responder(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        input: JoinSiagaResponder,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_authenticated(role)?;
        ensure_active_state(&broadcast)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            actor.user_id == broadcast.author_id,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );

        let mut updated = broadcast;
        let responder_index = updated
            .responders
            .iter()
            .position(|responder| responder.responder_id == actor.user_id);

        let mut event_type = SiagaTimelineEventType::SiagaResponderJoined;
        if let Some(index) = responder_index {
            let responder = &mut updated.responders[index];
            if responder.status == input.status {
                // idempotent retry for same status should return existing broadcast
                return Ok(updated);
            }
            responder.status = input.status.clone();
            responder.updated_at_ms = now_ms;
            responder.request_id = input.request_id.clone();
            responder.correlation_id = input.correlation_id.clone();
            event_type = SiagaTimelineEventType::SiagaResponderUpdated;
        } else {
            updated.responders.push(SiagaResponder {
                responder_id: actor.user_id.clone(),
                username: actor.username.clone(),
                status: input.status.clone(),
                joined_at_ms: now_ms,
                updated_at_ms: now_ms,
                request_id: input.request_id.clone(),
                correlation_id: input.correlation_id.clone(),
            });
        }

        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let event = make_siaga_event(
            updated.siaga_id.clone(),
            event_type,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({
                "responder_id": actor.user_id.clone(),
                "status": input.status.clone(),
            })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(updated) => Ok(updated),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn update_responder_status(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        responder_id: &str,
        input: UpdateResponderStatus,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_authenticated(role)?;
        ensure_active_state(&broadcast)?;

        if !is_author_or_admin(actor.user_id.as_str(), &broadcast.author_id, role)
            && actor.user_id != responder_id
        {
            return Err(DomainError::Forbidden(
                "only author or responder can update responder status".into(),
            ));
        }

        let mut updated = broadcast;
        let responder_pos = updated
            .responders
            .iter()
            .position(|responder| responder.responder_id == responder_id)
            .ok_or(DomainError::NotFound)?;

        let target = &mut updated.responders[responder_pos];
        target.status = input.status.clone();
        target.updated_at_ms = now_ms;
        target.request_id = input.request_id.clone();
        target.correlation_id = input.correlation_id.clone();

        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            actor.user_id == updated.author_id,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );
        let event = make_siaga_event(
            updated.siaga_id.clone(),
            SiagaTimelineEventType::SiagaResponderUpdated,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({
                "responder_id": responder_id,
                "status": input.status,
            })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(updated) => Ok(updated),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn close_broadcast(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        input: CloseSiagaBroadcast,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let input = validate_close_input(input)?;
        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if !is_author_or_admin(actor.user_id.as_str(), &broadcast.author_id, role) {
            return Err(DomainError::Forbidden("only author can close siaga".into()));
        }

        match broadcast.state {
            SiagaState::Resolved => {
                return Ok(broadcast);
            }
            SiagaState::Cancelled => {
                return Err(DomainError::Validation(
                    "cannot close a cancelled broadcast".into(),
                ));
            }
            SiagaState::Draft => {
                return Err(DomainError::Validation(
                    "cannot close a draft broadcast".into(),
                ));
            }
            SiagaState::Active => {}
        }

        let mut updated = broadcast;
        let closure = SiagaClosure {
            reason: input.reason,
            summary: input.summary,
            closed_by: actor.user_id.clone(),
            closed_at_ms: now_ms,
            counters: updated.counters(),
        };
        updated.state = SiagaState::Resolved;
        updated.closure = Some(closure);
        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            true,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );
        let event = make_siaga_event(
            updated.siaga_id.clone(),
            SiagaTimelineEventType::SiagaBroadcastClosed,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({
                "summary": updated.closure.as_ref().map(|closure| closure.summary.clone()),
                "responder_count": updated.counters().total_responders,
            })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(updated) => Ok(updated),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn cancel_broadcast(
        &self,
        actor: ActorIdentity,
        role: &Role,
        siaga_id: &str,
        input: CancelSiagaBroadcast,
    ) -> crate::DomainResult<SiagaBroadcast> {
        let input = validate_cancel_input(input)?;
        let now_ms = input.request_ts_ms.unwrap_or_else(now_ms);
        let broadcast = self
            .repository
            .get_broadcast(siaga_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        if !is_author_or_admin(actor.user_id.as_str(), &broadcast.author_id, role) {
            return Err(DomainError::Forbidden(
                "only author can cancel siaga".into(),
            ));
        }

        if matches!(broadcast.state, SiagaState::Cancelled) {
            return Ok(broadcast);
        }

        let mut updated = broadcast;
        let summary = format!("Cancelled: {}", input.reason);
        let closure = SiagaClosure {
            reason: "cancelled".to_string(),
            summary,
            closed_by: actor.user_id.clone(),
            closed_at_ms: now_ms,
            counters: updated.counters(),
        };
        updated.state = SiagaState::Cancelled;
        updated.closure = Some(closure);
        updated.updated_at_ms = now_ms;
        updated.request_id = input.request_id.clone();
        updated.correlation_id = input.correlation_id.clone();
        let updated = apply_siaga_broadcast_audit(updated)?;

        let actor_snapshot = SiagaActorSnapshot::new(
            &actor,
            role,
            true,
            input.request_id.clone(),
            input.correlation_id.clone(),
            now_ms,
        );
        let event = make_siaga_event(
            updated.siaga_id.clone(),
            SiagaTimelineEventType::SiagaBroadcastCancelled,
            actor_snapshot,
            input.request_id,
            input.correlation_id,
            now_ms,
            Some(serde_json::json!({ "reason": input.reason })),
        )?;

        match self.repository.update_broadcast(&updated, &event).await {
            Ok(updated) => Ok(updated),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(siaga_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }
}

fn ensure_authenticated(role: &Role) -> crate::DomainResult<()> {
    if matches!(role, Role::Anonymous) {
        return Err(DomainError::Forbidden(
            "anonymous actor is not allowed".into(),
        ));
    }
    Ok(())
}

fn is_author_or_admin(actor_id: &str, author_id: &str, role: &Role) -> bool {
    role == &Role::Admin || role == &Role::System || actor_id == author_id
}

fn ensure_active_state(broadcast: &SiagaBroadcast) -> crate::DomainResult<()> {
    if matches!(broadcast.state, SiagaState::Active) {
        return Ok(());
    }
    Err(DomainError::Validation(
        "operation allowed only in active state".into(),
    ))
}

fn siaga_broadcast_retention_tag(siaga_id: &str) -> String {
    format!("siaga_broadcast:{siaga_id}")
}

fn siaga_timeline_retention_tag(siaga_id: &str, event_type: &SiagaTimelineEventType) -> String {
    format!(
        "siaga_timeline:{siaga_id}:{}",
        siaga_timeline_event_to_string(event_type)
    )
}

fn siaga_broadcast_event_type_to_string(event_type: &SiagaState) -> &'static str {
    match event_type {
        SiagaState::Draft => "draft",
        SiagaState::Active => "active",
        SiagaState::Resolved => "resolved",
        SiagaState::Cancelled => "cancelled",
    }
}

fn siaga_timeline_event_to_string(event_type: &SiagaTimelineEventType) -> &'static str {
    match event_type {
        SiagaTimelineEventType::SiagaBroadcastCreated => "siaga_broadcast_created",
        SiagaTimelineEventType::SiagaBroadcastActivated => "siaga_broadcast_activated",
        SiagaTimelineEventType::SiagaBroadcastUpdated => "siaga_broadcast_updated",
        SiagaTimelineEventType::SiagaResponderJoined => "siaga_responder_joined",
        SiagaTimelineEventType::SiagaResponderUpdated => "siaga_responder_updated",
        SiagaTimelineEventType::SiagaBroadcastClosed => "siaga_broadcast_closed",
        SiagaTimelineEventType::SiagaBroadcastCancelled => "siaga_broadcast_cancelled",
    }
}

fn apply_siaga_broadcast_audit(mut broadcast: SiagaBroadcast) -> DomainResult<SiagaBroadcast> {
    broadcast.retention_tag = siaga_broadcast_retention_tag(&broadcast.siaga_id);
    let payload = SiagaBroadcastAuditPayload {
        siaga_id: broadcast.siaga_id.clone(),
        scope_id: broadcast.scope_id.clone(),
        author_id: broadcast.author_id.clone(),
        author_username: broadcast.author_username.clone(),
        emergency_type: broadcast.emergency_type.clone(),
        severity: broadcast.severity,
        location: broadcast.location.clone(),
        title: broadcast.title.clone(),
        text: broadcast.text.clone(),
        state: siaga_broadcast_event_type_to_string(&broadcast.state).to_string(),
        created_at_ms: broadcast.created_at_ms,
        updated_at_ms: broadcast.updated_at_ms,
        request_id: broadcast.request_id.clone(),
        correlation_id: broadcast.correlation_id.clone(),
        responders: broadcast.responders.clone(),
        closure: broadcast.closure.clone(),
        retention_tag: broadcast.retention_tag.clone(),
    };
    broadcast.event_hash = crate::util::immutable_event_hash(&payload)?;
    Ok(broadcast)
}

fn apply_siaga_timeline_audit(mut event: SiagaTimelineEvent) -> DomainResult<SiagaTimelineEvent> {
    event.retention_tag = siaga_timeline_retention_tag(&event.siaga_id, &event.event_type);
    let payload = SiagaTimelineAuditPayload {
        event_id: event.event_id.clone(),
        siaga_id: event.siaga_id.clone(),
        event_type: siaga_timeline_event_to_string(&event.event_type).to_string(),
        actor: event.actor.clone(),
        request_id: event.request_id.clone(),
        correlation_id: event.correlation_id.clone(),
        occurred_at_ms: event.occurred_at_ms,
        metadata: event.metadata.clone(),
        retention_tag: event.retention_tag.clone(),
    };
    event.event_hash = crate::util::immutable_event_hash(&payload)?;
    Ok(event)
}

fn make_siaga_event(
    siaga_id: String,
    event_type: SiagaTimelineEventType,
    actor: SiagaActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at_ms: i64,
    metadata: Option<serde_json::Value>,
) -> DomainResult<SiagaTimelineEvent> {
    let event = SiagaTimelineEvent {
        event_id: crate::util::uuid_v7_without_dashes(),
        siaga_id,
        event_type,
        actor,
        request_id,
        correlation_id,
        occurred_at_ms,
        metadata,
        event_hash: String::new(),
        retention_tag: String::new(),
    };
    apply_siaga_timeline_audit(event)
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

fn validate_create_broadcast(
    input: CreateSiagaBroadcast,
) -> crate::DomainResult<CreateSiagaBroadcast> {
    let CreateSiagaBroadcast {
        mut scope_id,
        mut emergency_type,
        severity,
        mut location,
        mut title,
        mut text,
        request_id,
        correlation_id,
        request_ts_ms,
    } = input;

    scope_id = scope_id.trim().to_string();
    emergency_type = emergency_type.trim().to_string();
    location = location.trim().to_string();
    title = title.trim().to_string();
    text = text.trim().to_string();

    if scope_id.is_empty() {
        return Err(DomainError::Validation("scope_id is required".into()));
    }
    if emergency_type.is_empty() {
        return Err(DomainError::Validation("emergency_type is required".into()));
    }
    if !(MIN_SEVERITY..=MAX_SEVERITY).contains(&severity) {
        return Err(DomainError::Validation(
            "severity must be between 1 and 5".into(),
        ));
    }
    if location.is_empty() {
        return Err(DomainError::Validation("location is required".into()));
    }
    if location.len() > MAX_LOCATION_LEN {
        return Err(DomainError::Validation("location is too long".into()));
    }
    if title.is_empty() {
        return Err(DomainError::Validation("title is required".into()));
    }
    if title.len() > MAX_TITLE_LEN {
        return Err(DomainError::Validation("title is too long".into()));
    }
    if text.is_empty() {
        return Err(DomainError::Validation("text is required".into()));
    }
    if text.len() > MAX_TEXT_LEN {
        return Err(DomainError::Validation("text is too long".into()));
    }
    if request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }

    Ok(CreateSiagaBroadcast {
        scope_id,
        emergency_type,
        severity,
        location,
        title,
        text,
        request_id,
        correlation_id,
        request_ts_ms,
    })
}

fn validate_update_broadcast(
    input: UpdateSiagaBroadcast,
) -> crate::DomainResult<UpdateSiagaBroadcast> {
    let UpdateSiagaBroadcast {
        scope_id,
        mut emergency_type,
        severity,
        mut location,
        mut title,
        mut text,
        request_id,
        correlation_id,
        request_ts_ms,
    } = input;

    if request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }

    if let Some(value) = scope_id.as_ref() {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            return Err(DomainError::Validation("scope_id is required".into()));
        }
    }

    if let Some(value) = emergency_type.as_mut() {
        *value = value.trim().to_string();
        if value.is_empty() {
            return Err(DomainError::Validation(
                "emergency_type is required when provided".into(),
            ));
        }
    }

    if let Some(value) = severity {
        if !(MIN_SEVERITY..=MAX_SEVERITY).contains(&value) {
            return Err(DomainError::Validation(
                "severity must be between 1 and 5".into(),
            ));
        }
    }

    if let Some(value) = location.as_mut() {
        *value = value.trim().to_string();
        if value.is_empty() {
            return Err(DomainError::Validation(
                "location is required when provided".into(),
            ));
        }
        if value.len() > MAX_LOCATION_LEN {
            return Err(DomainError::Validation("location is too long".into()));
        }
    }

    if let Some(value) = title.as_mut() {
        *value = value.trim().to_string();
        if value.is_empty() {
            return Err(DomainError::Validation(
                "title is required when provided".into(),
            ));
        }
        if value.len() > MAX_TITLE_LEN {
            return Err(DomainError::Validation("title is too long".into()));
        }
    }

    if let Some(value) = text.as_mut() {
        *value = value.trim().to_string();
        if value.is_empty() {
            return Err(DomainError::Validation(
                "text is required when provided".into(),
            ));
        }
        if value.len() > MAX_TEXT_LEN {
            return Err(DomainError::Validation("text is too long".into()));
        }
    }

    if scope_id.is_none()
        && emergency_type.is_none()
        && severity.is_none()
        && location.is_none()
        && title.is_none()
        && text.is_none()
    {
        return Err(DomainError::Validation("no update fields provided".into()));
    }

    Ok(UpdateSiagaBroadcast {
        scope_id,
        emergency_type,
        severity,
        location,
        title,
        text,
        request_id,
        correlation_id,
        request_ts_ms,
    })
}

fn validate_close_input(input: CloseSiagaBroadcast) -> crate::DomainResult<CloseSiagaBroadcast> {
    let reason = input.reason.trim().to_string();
    let summary = input.summary.trim().to_string();

    if input.request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if input.correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }
    if reason.is_empty() {
        return Err(DomainError::Validation("close reason is required".into()));
    }
    if reason.len() > MAX_TEXT_LEN {
        return Err(DomainError::Validation("close reason is too long".into()));
    }
    if summary.is_empty() {
        return Err(DomainError::Validation("close summary is required".into()));
    }
    if summary.len() > MAX_TEXT_LEN {
        return Err(DomainError::Validation("close summary is too long".into()));
    }

    Ok(CloseSiagaBroadcast {
        reason,
        summary,
        request_id: input.request_id,
        correlation_id: input.correlation_id,
        request_ts_ms: input.request_ts_ms,
    })
}

fn validate_cancel_input(input: CancelSiagaBroadcast) -> crate::DomainResult<CancelSiagaBroadcast> {
    let mut reason = input.reason.trim().to_string();
    if input.request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if input.correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }
    if reason.is_empty() {
        return Err(DomainError::Validation("cancel reason is required".into()));
    }
    if reason.len() > MAX_TEXT_LEN {
        return Err(DomainError::Validation("cancel reason is too long".into()));
    }

    if reason.len() > MAX_TEXT_LEN {
        reason = reason.chars().take(MAX_TEXT_LEN).collect();
    }

    Ok(CancelSiagaBroadcast {
        reason,
        request_id: input.request_id,
        correlation_id: input.correlation_id,
        request_ts_ms: input.request_ts_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use crate::ports::siaga::SiagaRepository;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockSiagaRepository {
        by_id: Arc<RwLock<HashMap<String, SiagaBroadcast>>>,
        by_actor_request: Arc<RwLock<HashMap<(String, String), String>>>,
        by_request: Arc<RwLock<HashMap<(String, String), String>>>,
        timeline: Arc<RwLock<HashMap<String, VecDeque<SiagaTimelineEvent>>>>,
    }

    impl MockSiagaRepository {
        fn actor_request_key(actor_id: &str, request_id: &str) -> (String, String) {
            (actor_id.to_string(), request_id.to_string())
        }

        fn request_key(siaga_id: &str, request_id: &str) -> (String, String) {
            (siaga_id.to_string(), request_id.to_string())
        }
    }

    impl SiagaRepository for MockSiagaRepository {
        fn create_broadcast(
            &self,
            broadcast: &SiagaBroadcast,
            event: &SiagaTimelineEvent,
        ) -> BoxFuture<'_, crate::DomainResult<SiagaBroadcast>> {
            let broadcast = broadcast.clone();
            let event = event.clone();
            let by_id = self.by_id.clone();
            let by_actor_request = self.by_actor_request.clone();
            let by_request = self.by_request.clone();
            let timeline = self.timeline.clone();

            Box::pin(async move {
                if by_id.read().await.contains_key(&broadcast.siaga_id) {
                    return Err(DomainError::Conflict);
                }
                if by_actor_request
                    .read()
                    .await
                    .contains_key(&Self::actor_request_key(
                        &broadcast.author_id,
                        &event.request_id,
                    ))
                {
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
                    Self::request_key(&broadcast.siaga_id, &event.request_id),
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
        ) -> BoxFuture<'_, crate::DomainResult<SiagaBroadcast>> {
            let broadcast = broadcast.clone();
            let event = event.clone();
            let by_id = self.by_id.clone();
            let by_request = self.by_request.clone();
            let timeline = self.timeline.clone();
            Box::pin(async move {
                if by_request
                    .read()
                    .await
                    .contains_key(&Self::request_key(&broadcast.siaga_id, &event.request_id))
                {
                    let stored_id = by_request
                        .read()
                        .await
                        .get(&Self::request_key(&broadcast.siaga_id, &event.request_id))
                        .cloned();
                    if let Some(stored_id) = stored_id {
                        return by_id
                            .read()
                            .await
                            .get(&stored_id)
                            .cloned()
                            .ok_or(DomainError::Conflict);
                    }
                }

                if !by_id.read().await.contains_key(&broadcast.siaga_id) {
                    return Err(DomainError::NotFound);
                }
                by_id
                    .write()
                    .await
                    .insert(broadcast.siaga_id.clone(), broadcast.clone());
                by_request.write().await.insert(
                    Self::request_key(&broadcast.siaga_id, &event.request_id),
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

        fn get_broadcast(
            &self,
            siaga_id: &str,
        ) -> BoxFuture<'_, crate::DomainResult<Option<SiagaBroadcast>>> {
            let siaga_id = siaga_id.to_string();
            let by_id = self.by_id.clone();
            Box::pin(async move { Ok(by_id.read().await.get(&siaga_id).cloned()) })
        }

        fn list_by_scope(
            &self,
            scope_id: &str,
        ) -> BoxFuture<'_, crate::DomainResult<Vec<SiagaBroadcast>>> {
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
        ) -> BoxFuture<'_, crate::DomainResult<Vec<SiagaTimelineEvent>>> {
            let siaga_id = siaga_id.to_string();
            let timeline = self.timeline.clone();
            Box::pin(async move {
                let mut timeline = timeline
                    .read()
                    .await
                    .get(&siaga_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<_>>();
                timeline.sort_by(|left, right| {
                    left.occurred_at_ms
                        .cmp(&right.occurred_at_ms)
                        .then_with(|| left.event_id.cmp(&right.event_id))
                });
                Ok(timeline)
            })
        }

        fn get_by_actor_request(
            &self,
            actor_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, crate::DomainResult<Option<SiagaBroadcast>>> {
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
        ) -> BoxFuture<'_, crate::DomainResult<Option<SiagaBroadcast>>> {
            let siaga_id = siaga_id.to_string();
            let request_id = request_id.to_string();
            let by_request = self.by_request.clone();
            let by_id = self.by_id.clone();
            Box::pin(async move {
                let by_request = by_request.read().await;
                let Some(stored_id) = by_request.get(&Self::request_key(&siaga_id, &request_id))
                else {
                    return Ok(None);
                };
                Ok(by_id.read().await.get(stored_id).cloned())
            })
        }
    }

    fn service() -> SiagaService {
        SiagaService::new(Arc::new(MockSiagaRepository::default()))
    }

    fn actor(id: &str) -> ActorIdentity {
        ActorIdentity {
            user_id: id.to_string(),
            username: id.to_string(),
        }
    }

    #[tokio::test]
    async fn create_and_close_with_summary() {
        let service = service();
        let created = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-1".to_string(),
                    emergency_type: "fire".to_string(),
                    severity: 3,
                    location: "Jl. Merdeka".to_string(),
                    title: "Bantuan darurat".to_string(),
                    text: "api di rumah warga".to_string(),
                    request_id: "req-create-1".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");

        let activated = service
            .activate(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                ActivateSiagaBroadcast {
                    request_id: "req-activate-1".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("activated");

        let joined = service
            .join_responder(
                actor("u-2"),
                &Role::User,
                &created.siaga_id,
                JoinSiagaResponder {
                    status: SiagaResponderStatus::Coming,
                    request_id: "req-join-1".to_string(),
                    correlation_id: "corr-3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await
            .expect("joined");

        let closed = service
            .close_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                CloseSiagaBroadcast {
                    reason: "resolved".to_string(),
                    summary: "Semua aman".to_string(),
                    request_id: "req-close-1".to_string(),
                    correlation_id: "corr-4".to_string(),
                    request_ts_ms: Some(4),
                },
            )
            .await
            .expect("closed");

        assert_eq!(activated.state, SiagaState::Active);
        assert_eq!(joined.responders.len(), 1);
        assert_eq!(closed.state, SiagaState::Resolved);
        assert!(closed.closure.is_some());
        assert_eq!(
            closed.closure.as_ref().expect("closure").summary,
            "Semua aman"
        );
    }

    #[tokio::test]
    async fn close_requires_summary() {
        let service = service();
        let created = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-2".to_string(),
                    emergency_type: "medical".to_string(),
                    severity: 2,
                    location: "Jl. Kenanga".to_string(),
                    title: "Kecelakaan".to_string(),
                    text: "kecelakaan lalu lintas".to_string(),
                    request_id: "req-create-2".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");
        let _ = service
            .activate(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                ActivateSiagaBroadcast {
                    request_id: "req-activate-2".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("activated");

        let result = service
            .close_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                CloseSiagaBroadcast {
                    reason: "resolved".to_string(),
                    summary: "   ".to_string(),
                    request_id: "req-close-2".to_string(),
                    correlation_id: "corr-3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_replay_uses_same_broadcast_id() {
        let service = service();
        let first = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-3".to_string(),
                    emergency_type: "flood".to_string(),
                    severity: 4,
                    location: "RW 01".to_string(),
                    title: "Banjir".to_string(),
                    text: "banjir masuk rumah".to_string(),
                    request_id: "req-create-3".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("first");
        let second = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-3".to_string(),
                    emergency_type: "flood".to_string(),
                    severity: 4,
                    location: "RW 01".to_string(),
                    title: "Banjir".to_string(),
                    text: "banjir masuk rumah".to_string(),
                    request_id: "req-create-3".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(10),
                },
            )
            .await
            .expect("replayed");
        assert_eq!(first.siaga_id, second.siaga_id);
    }

    #[tokio::test]
    async fn create_replay_works_after_update_has_advanced_request_id() {
        let service = service();
        let created = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-4".to_string(),
                    emergency_type: "flood".to_string(),
                    severity: 4,
                    location: "RW 01".to_string(),
                    title: "Banjir".to_string(),
                    text: "banjir masuk rumah".to_string(),
                    request_id: "req-create-4".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("first");

        let _updated = service
            .update_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                UpdateSiagaBroadcast {
                    scope_id: None,
                    emergency_type: None,
                    severity: None,
                    location: None,
                    title: Some("Banjir di sektor 4".to_string()),
                    text: None,
                    request_id: "req-update-1".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("updated");

        let replayed = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-4".to_string(),
                    emergency_type: "flood".to_string(),
                    severity: 4,
                    location: "RW 01".to_string(),
                    title: "Banjir".to_string(),
                    text: "banjir masuk rumah".to_string(),
                    request_id: "req-create-4".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(10),
                },
            )
            .await
            .expect("create replay");

        assert_eq!(created.siaga_id, replayed.siaga_id);
        assert_eq!(_updated.scope_id, replayed.scope_id);
        assert_eq!(replayed.title, "Banjir di sektor 4".to_string());
    }

    #[tokio::test]
    async fn update_replay_still_works_when_request_id_on_broadcast_advanced() {
        let service = service();
        let created = service
            .create_broadcast(
                actor("u-1"),
                &Role::User,
                CreateSiagaBroadcast {
                    scope_id: "scope-5".to_string(),
                    emergency_type: "fire".to_string(),
                    severity: 3,
                    location: "Jl. Aman".to_string(),
                    title: "Kebakaran".to_string(),
                    text: "api di dapur".to_string(),
                    request_id: "req-create-5".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");

        let first_update = service
            .update_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                UpdateSiagaBroadcast {
                    scope_id: None,
                    emergency_type: Some("smoke".to_string()),
                    severity: None,
                    location: None,
                    title: None,
                    text: None,
                    request_id: "req-update-1".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("first update");

        let second_update = service
            .update_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                UpdateSiagaBroadcast {
                    scope_id: None,
                    emergency_type: None,
                    severity: Some(4),
                    location: None,
                    title: None,
                    text: None,
                    request_id: "req-update-2".to_string(),
                    correlation_id: "corr-3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await
            .expect("second update");

        let replay_update = service
            .update_broadcast(
                actor("u-1"),
                &Role::User,
                &created.siaga_id,
                UpdateSiagaBroadcast {
                    scope_id: None,
                    emergency_type: Some("smoke".to_string()),
                    severity: None,
                    location: None,
                    title: None,
                    text: None,
                    request_id: "req-update-1".to_string(),
                    correlation_id: "corr-replay".to_string(),
                    request_ts_ms: Some(4),
                },
            )
            .await
            .expect("replayed update");

        assert_eq!(second_update.siaga_id, replay_update.siaga_id);
        assert_eq!(second_update.request_id, replay_update.request_id);
        assert_eq!(replay_update.request_id, "req-update-2".to_string());
        assert_eq!(replay_update.severity, 4);
        assert_eq!(first_update.request_id, "req-update-1".to_string());
    }
}

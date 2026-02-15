use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Path, Query, State};
use axum::{
    Json, Router,
    extract::ws::close_code,
    http::{HeaderMap, StatusCode},
    middleware,
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use futures_util::{SinkExt, StreamExt};
use gotong_domain::{
    auth::TrackRole,
    chat::{
        ChatMember, ChatMessage, ChatReadCursor, ChatService, ChatThread, ChatThreadCreate,
        MessageCatchup, SendMessageInput, build_message_catchup,
    },
    contributions::{Contribution, ContributionCreate, ContributionService, ContributionType},
    error::DomainError,
    evidence::{Evidence, EvidenceCreate, EvidenceService, EvidenceType},
    idempotency::{BeginOutcome, timer_request_id},
    identity::ActorIdentity,
    jobs::{ModerationAutoReleasePayload, TransitionClosePayload, new_job},
    moderation::{
        ContentModeration, ModerationApplyCommand, ModerationDecision, ModerationService,
    },
    ports::idempotency::{IdempotencyKey, IdempotencyResponse},
    ports::jobs::JobType,
    transitions::{
        TrackStateTransition, TrackTransitionInput, TrackTransitionService, TransitionAction,
        TransitionMechanism,
    },
    vault::{
        AddTrustee, CreateVaultDraft, ExpireVault, PublishVault, RemoveTrustee, RevokeVault,
        SealVault, UpdateVaultDraft, VaultEntry, VaultService, VaultTimelineEvent,
    },
    vouches::{Vouch, VouchCreate, VouchService, VouchWeightHint},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::wrappers::UnboundedReceiverStream;
use validator::Validate;

use crate::middleware::AuthContext;
use crate::{error::ApiError, middleware as app_middleware, state::AppState, validation};

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/idempotent-echo", post(idempotent_echo))
        .route(
            "/v1/contributions",
            post(create_contribution).get(list_contributions),
        )
        .route("/v1/contributions/:contribution_id", get(get_contribution))
        .route(
            "/v1/contributions/:contribution_id/evidence",
            get(list_evidence_by_contribution),
        )
        .route("/v1/evidence", post(submit_evidence))
        .route("/v1/evidence/:evidence_id", get(get_evidence))
        .route("/v1/vouches", post(submit_vouch).get(list_vouches))
        .route("/v1/transitions", post(create_transition))
        .route(
            "/v1/transitions/:entity_id/timeline",
            get(list_transition_timeline),
        )
        .route(
            "/v1/transitions/:entity_id/active",
            get(get_active_transition_stage),
        )
        .route("/v1/transitions/:transition_id", get(get_transition_by_id))
        .route("/v1/vaults", post(create_vault_draft).get(list_vaults))
        .route(
            "/v1/vaults/:vault_entry_id",
            get(get_vault_entry).delete(delete_vault_draft),
        )
        .route(
            "/v1/vaults/:vault_entry_id/update",
            post(update_vault_entry),
        )
        .route("/v1/vaults/:vault_entry_id/seal", post(seal_vault_entry))
        .route(
            "/v1/vaults/:vault_entry_id/publish",
            post(publish_vault_entry),
        )
        .route(
            "/v1/vaults/:vault_entry_id/revoke",
            post(revoke_vault_entry),
        )
        .route(
            "/v1/vaults/:vault_entry_id/expire",
            post(expire_vault_entry),
        )
        .route(
            "/v1/vaults/:vault_entry_id/timeline",
            get(list_vault_timeline),
        )
        .route(
            "/v1/vaults/:vault_entry_id/trustees",
            get(list_vault_trustees).post(add_vault_trustee),
        )
        .route(
            "/v1/vaults/:vault_entry_id/trustees/:wali_id",
            delete(remove_vault_trustee),
        )
        .route("/v1/moderations", post(apply_moderation))
        .route(
            "/v1/moderations/review-queue",
            get(list_moderation_review_queue),
        )
        .route("/v1/moderations/:content_id", get(get_moderation_view))
        .route(
            "/v1/chat/threads",
            post(create_chat_thread).get(list_chat_threads),
        )
        .route(
            "/v1/chat/threads/:thread_id/members",
            get(list_chat_members),
        )
        .route("/v1/chat/threads/:thread_id/join", post(join_chat_thread))
        .route("/v1/chat/threads/:thread_id/leave", post(leave_chat_thread))
        .route(
            "/v1/chat/threads/:thread_id/messages",
            get(list_chat_messages),
        )
        .route(
            "/v1/chat/threads/:thread_id/messages/send",
            post(send_chat_message),
        )
        .route(
            "/v1/chat/threads/:thread_id/messages/poll",
            get(poll_chat_messages),
        )
        .route(
            "/v1/chat/threads/:thread_id/messages/stream",
            get(stream_chat_messages_sse),
        )
        .route(
            "/v1/chat/threads/:thread_id/messages/ws",
            get(stream_chat_messages_ws),
        )
        .route(
            "/v1/chat/threads/:thread_id/read-cursor",
            get(get_chat_read_cursor).post(mark_chat_read_cursor),
        )
        .route_layer(middleware::from_fn(app_middleware::require_auth_middleware));

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/v1/echo", post(echo))
        .merge(protected)
        .layer(app_middleware::timeout_layer())
        .layer(app_middleware::trace_layer())
        .layer(app_middleware::set_request_id_layer())
        .layer(app_middleware::propagate_request_id_layer())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_middleware::auth_middleware,
        ))
        .layer(middleware::from_fn(
            app_middleware::correlation_id_middleware,
        ));

    if !state.config.app_env.eq_ignore_ascii_case("test") {
        app = app.layer(app_middleware::rate_limit_layer());
    }

    app.with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    environment: String,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        environment: state.config.app_env.clone(),
    })
}

#[derive(Debug, Deserialize, Validate)]
struct EchoRequest {
    #[validate(length(min = 1, max = 256))]
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    message: String,
}

async fn echo(Json(payload): Json<EchoRequest>) -> Result<Json<EchoResponse>, ApiError> {
    validation::validate(&payload)?;
    Ok(Json(EchoResponse {
        message: payload.message,
    }))
}

#[derive(Debug, Deserialize, Validate)]
struct IdempotentEchoRequest {
    #[validate(length(min = 1, max = 128))]
    entity_id: String,
    #[validate(length(min = 1, max = 256))]
    message: String,
}

async fn idempotent_echo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<IdempotentEchoRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let request_id = request_id_from_headers(&headers)?;
    let key = IdempotencyKey::new("echo", payload.entity_id.clone(), request_id);

    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "message": payload.message }),
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
struct CreateVaultDraftRequest {
    pub payload: Option<Value>,
    pub attachment_refs: Vec<String>,
    pub wali: Vec<String>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<Value>,
    pub audit: Option<Value>,
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateVaultDraftRequest {
    pub payload: Option<Value>,
    pub attachment_refs: Option<Vec<String>>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<Value>,
    pub audit: Option<Value>,
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct SealVaultRequest {
    pub sealed_hash: String,
    pub encryption_key_id: Option<String>,
    pub sealed_payload: Option<Value>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<Value>,
    pub audit: Option<Value>,
    pub sealed_at_ms: Option<i64>,
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct SimpleVaultIdempotentRequest {
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct AddVaultTrusteeRequest {
    #[validate(length(min = 1, max = 128))]
    wali_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateContributionRequest {
    pub contribution_type: ContributionType,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub description: Option<String>,
    pub evidence_url: Option<String>,
    pub skill_ids: Vec<String>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize)]
struct ContributionListQuery {
    pub author_id: Option<String>,
}

async fn create_contribution(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateContributionRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "contribution_create",
        actor.user_id.clone(),
        request_id.clone(),
    );

    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ContributionService::new(state.contribution_repo.clone());
            let input = ContributionCreate {
                contribution_type: payload.contribution_type,
                title: payload.title,
                description: payload.description,
                evidence_url: payload.evidence_url,
                skill_ids: payload.skill_ids,
                metadata: payload.metadata,
            };

            let contribution = service
                .create(actor, request_id, correlation_id, input)
                .await
                .map_err(map_domain_error)?;

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&contribution).map_err(|_| ApiError::Internal)?,
            };

            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;

            Ok(to_response(response))
        }
    }
}

async fn list_contributions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ContributionListQuery>,
) -> Result<Json<Vec<Contribution>>, ApiError> {
    let author_id = query.author_id.unwrap_or_else(|| {
        actor_identity(&auth)
            .map(|actor| actor.user_id)
            .unwrap_or_default()
    });

    if author_id.is_empty() {
        return Err(ApiError::Validation("author_id is required".into()));
    }

    let service = ContributionService::new(state.contribution_repo.clone());
    let contributions = service
        .list_by_author(&author_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(contributions))
}

async fn get_contribution(
    State(state): State<AppState>,
    Path(contribution_id): Path<String>,
) -> Result<Json<Contribution>, ApiError> {
    let service = ContributionService::new(state.contribution_repo.clone());
    let contribution = service
        .get(&contribution_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(contribution))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateEvidenceRequest {
    #[validate(length(min = 1, max = 128))]
    contribution_id: String,
    pub evidence_type: EvidenceType,
    pub evidence_data: Value,
    pub proof: Value,
}

async fn submit_evidence(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateEvidenceRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    ContributionService::new(state.contribution_repo.clone())
        .get(&payload.contribution_id)
        .await
        .map_err(map_domain_error)?;

    let key = IdempotencyKey::new("evidence_submit", actor.user_id.clone(), request_id.clone());

    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = EvidenceService::new(state.evidence_repo.clone());
            let input = EvidenceCreate {
                contribution_id: payload.contribution_id,
                evidence_type: payload.evidence_type,
                evidence_data: payload.evidence_data,
                proof: payload.proof,
            };

            let evidence = service
                .submit(actor, request_id, correlation_id, input)
                .await
                .map_err(map_domain_error)?;

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&evidence).map_err(|_| ApiError::Internal)?,
            };

            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;

            Ok(to_response(response))
        }
    }
}

async fn get_evidence(
    State(state): State<AppState>,
    Path(evidence_id): Path<String>,
) -> Result<Json<Evidence>, ApiError> {
    let service = EvidenceService::new(state.evidence_repo.clone());
    let evidence = service.get(&evidence_id).await.map_err(map_domain_error)?;
    Ok(Json(evidence))
}

async fn list_evidence_by_contribution(
    State(state): State<AppState>,
    Path(contribution_id): Path<String>,
) -> Result<Json<Vec<Evidence>>, ApiError> {
    let service = EvidenceService::new(state.evidence_repo.clone());
    let evidences = service
        .list_by_contribution(&contribution_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(evidences))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateVouchRequest {
    #[validate(length(min = 1, max = 128))]
    vouchee_id: String,
    pub skill_id: Option<String>,
    pub weight_hint: Option<VouchWeightHint>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateTransitionRequest {
    #[validate(length(min = 1, max = 128))]
    pub track: String,
    #[validate(length(min = 1, max = 128))]
    pub entity_id: String,
    #[validate(length(min = 1, max = 128))]
    pub from_stage: String,
    #[validate(length(min = 1, max = 128))]
    pub to_stage: String,
    pub transition_action: TransitionAction,
    pub transition_type: TransitionMechanism,
    pub mechanism: TransitionMechanism,
    pub track_roles: Vec<TrackRole>,
    #[validate(length(min = 1, max = 128))]
    pub gate_status: String,
    pub gate_metadata: Option<Value>,
    #[serde(default)]
    pub occurred_at_ms: Option<i64>,
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
    #[serde(default)]
    pub closes_at_ms: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
struct ApplyModerationRequest {
    #[validate(length(min = 1, max = 128))]
    pub content_id: String,
    pub content_type: Option<String>,
    pub author_id: Option<String>,
    pub author_username: Option<String>,
    pub moderation_status: gotong_domain::moderation::ModerationStatus,
    pub moderation_action: gotong_domain::moderation::ModerationAction,
    pub reason_code: Option<String>,
    pub confidence: f64,
    #[serde(default)]
    pub hold_duration_minutes: Option<i64>,
    #[serde(default)]
    pub auto_release_if_no_action: bool,
    #[serde(default)]
    pub appeal_window_minutes: Option<i64>,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub violations: Vec<gotong_domain::moderation::ModerationViolation>,
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ModerationReviewQueueQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize)]
struct ModerationApplyResponse {
    pub content: ContentModeration,
    pub decision: ModerationDecision,
    pub schedule_auto_release: bool,
}

#[derive(Debug, Deserialize)]
struct ListVouchesQuery {
    pub vouchee_id: Option<String>,
    pub voucher_id: Option<String>,
}

async fn submit_vouch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateVouchRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new("vouch_submit", actor.user_id.clone(), request_id.clone());

    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VouchService::new(state.vouch_repo.clone());
            let input = VouchCreate {
                vouchee_id: payload.vouchee_id,
                skill_id: payload.skill_id,
                weight_hint: payload.weight_hint,
                message: payload.message,
            };

            let vouch = service
                .submit(actor, request_id, correlation_id, input)
                .await
                .map_err(map_domain_error)?;

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&vouch).map_err(|_| ApiError::Internal)?,
            };

            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;

            Ok(to_response(response))
        }
    }
}

async fn list_vouches(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListVouchesQuery>,
) -> Result<Json<Vec<Vouch>>, ApiError> {
    let service = VouchService::new(state.vouch_repo.clone());

    let response = match (query.vouchee_id, query.voucher_id) {
        (Some(vouchee), None) => service
            .list_by_vouchee(&vouchee)
            .await
            .map_err(map_domain_error)?,
        (None, Some(voucher)) => service
            .list_by_voucher(&voucher)
            .await
            .map_err(map_domain_error)?,
        (None, None) => service
            .list_by_voucher(&actor_identity(&auth)?.user_id)
            .await
            .map_err(map_domain_error)?,
        (Some(_), Some(_)) => {
            return Err(ApiError::Validation(
                "provide only one of vouchee_id or voucher_id".into(),
            ));
        }
    };

    Ok(Json(response))
}

async fn apply_moderation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ApplyModerationRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let token_role = auth.role.clone();

    let key = IdempotencyKey::new(
        "moderation_apply",
        format!("{}:{}", actor.user_id, payload.content_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ModerationService::new(state.moderation_repo.clone());
            let command = ModerationApplyCommand {
                content_id: payload.content_id,
                content_type: payload.content_type,
                author_id: payload.author_id,
                author_username: payload.author_username,
                moderation_status: payload.moderation_status,
                moderation_action: payload.moderation_action,
                reason_code: payload.reason_code,
                confidence: payload.confidence,
                hold_duration_minutes: payload.hold_duration_minutes,
                auto_release_if_no_action: payload.auto_release_if_no_action,
                appeal_window_minutes: payload.appeal_window_minutes,
                reasoning: payload.reasoning,
                violations: payload.violations,
                request_id: request_id.clone(),
                correlation_id: correlation_id.clone(),
                request_ts_ms: payload.request_ts_ms,
            };

            let result = service
                .upsert_moderation_decision(actor, token_role.clone(), command)
                .await
                .map_err(map_domain_error)?;

            if result.schedule_auto_release {
                let hold_expires_at_ms = result
                    .content
                    .hold_expires_at_ms
                    .ok_or(ApiError::Internal)?;
                let auto_payload = ModerationAutoReleasePayload {
                    content_id: result.content.content_id.clone(),
                    hold_decision_request_id: result.decision.request_id.clone(),
                    request_id: format!(
                        "moderation_auto:{}:{}",
                        result.content.content_id, result.decision.request_id
                    ),
                    correlation_id: result.decision.correlation_id.clone(),
                    scheduled_ms: hold_expires_at_ms,
                    request_ts_ms: result.decision.decided_at_ms,
                };
                let job = new_job(
                    result.decision.decision_id.clone(),
                    JobType::ModerationAutoRelease,
                    serde_json::to_value(&auto_payload).map_err(|_| ApiError::Internal)?,
                    auto_payload.request_id.clone(),
                    result.decision.correlation_id.clone(),
                    gotong_domain::jobs::JobDefaults::default(),
                )
                .with_run_at(hold_expires_at_ms);

                if let Some(queue) = state.transition_job_queue.as_ref() {
                    queue.enqueue(&job).await.map_err(|err| {
                        tracing::error!(error = %err, "failed to enqueue moderation auto-release job");
                        ApiError::Internal
                    })?;
                }
            }

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(ModerationApplyResponse {
                    content: result.content,
                    decision: result.decision,
                    schedule_auto_release: result.schedule_auto_release,
                })
                .map_err(|_| ApiError::Internal)?,
            };

            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;

            Ok(to_response(response))
        }
    }
}

async fn get_moderation_view(
    State(state): State<AppState>,
    Path(content_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role;
    let service = ModerationService::new(state.moderation_repo.clone());
    let view = service
        .get_moderation_view(&content_id, &actor, &token_role)
        .await
        .map_err(map_domain_error)?;

    let response = IdempotencyResponse {
        status_code: StatusCode::OK.as_u16(),
        body: serde_json::to_value(view).map_err(|_| ApiError::Internal)?,
    };
    Ok(to_response(response))
}

async fn list_moderation_review_queue(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ModerationReviewQueueQuery>,
) -> Result<Json<Vec<ContentModeration>>, ApiError> {
    let token_role = auth.role;
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    let service = ModerationService::new(state.moderation_repo.clone());
    let queue = service
        .list_review_queue(&token_role, limit)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(queue))
}

async fn create_transition(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateTransitionRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let token_role = auth.role.clone();

    let key = IdempotencyKey::new(
        "transition_create",
        format!("{}:{}", actor.user_id, payload.entity_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = TrackTransitionService::new(state.transition_repo.clone());
            let closes_at_ms = payload.closes_at_ms;
            let input = TrackTransitionInput {
                track: payload.track,
                entity_id: payload.entity_id,
                from_stage: payload.from_stage,
                to_stage: payload.to_stage,
                transition_action: payload.transition_action,
                transition_type: payload.transition_type,
                mechanism: payload.mechanism,
                request_id,
                correlation_id,
                track_roles: payload.track_roles,
                gate_status: payload.gate_status,
                gate_metadata: payload.gate_metadata,
                occurred_at_ms: payload.occurred_at_ms,
                request_ts_ms: payload.request_ts_ms,
                closes_at_ms,
            };
            let transition = service
                .track_state_transition(actor, token_role, input)
                .await
                .map_err(map_domain_error)?;

            if transition.transition_type == TransitionMechanism::Timer {
                let closes_at_ms = closes_at_ms.ok_or(ApiError::Internal)?;
                let close_request_id = timer_request_id(&transition.transition_id, closes_at_ms);
                let close_payload = TransitionClosePayload {
                    transition_id: transition.transition_id.clone(),
                    entity_id: transition.entity_id.clone(),
                    track: transition.track.clone(),
                    from_stage: transition.from_stage.clone(),
                    to_stage: transition.to_stage.clone(),
                    closes_at_ms,
                    request_id: close_request_id.clone(),
                    request_ts_ms: transition.occurred_at_ms,
                    correlation_id: transition.correlation_id.clone(),
                    gate_status: "applied".to_string(),
                    gate_metadata: transition.gate.metadata.clone(),
                };
                let job = new_job(
                    transition.transition_id.clone(),
                    JobType::TransitionClose,
                    serde_json::to_value(&close_payload).map_err(|_| ApiError::Internal)?,
                    close_request_id,
                    transition.correlation_id.clone(),
                    gotong_domain::jobs::JobDefaults::default(),
                )
                .with_run_at(closes_at_ms);

                if let Some(queue) = state.transition_job_queue.as_ref() {
                    queue.enqueue(&job).await.map_err(|err| {
                        tracing::error!(error = %err, "failed to enqueue transition close job");
                        ApiError::Internal
                    })?;
                }
            }

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&transition).map_err(|_| ApiError::Internal)?,
            };

            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;

            Ok(to_response(response))
        }
    }
}

async fn list_transition_timeline(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
) -> Result<Json<Vec<TrackStateTransition>>, ApiError> {
    let service = TrackTransitionService::new(state.transition_repo.clone());
    let transitions = service
        .list_by_entity(&entity_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(transitions))
}

#[derive(Serialize)]
struct ActiveTransitionResponse {
    pub entity_id: String,
    pub active_stage: Option<String>,
}

async fn get_active_transition_stage(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
) -> Result<Json<ActiveTransitionResponse>, ApiError> {
    let service = TrackTransitionService::new(state.transition_repo.clone());
    let active_stage = service
        .active_stage(&entity_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(ActiveTransitionResponse {
        entity_id,
        active_stage,
    }))
}

async fn get_transition_by_id(
    State(state): State<AppState>,
    Path(transition_id): Path<String>,
) -> Result<Json<TrackStateTransition>, ApiError> {
    let service = TrackTransitionService::new(state.transition_repo.clone());
    let transition = service
        .get_by_transition_id(&transition_id)
        .await
        .map_err(map_domain_error)?;
    let transition = transition.ok_or(ApiError::NotFound)?;
    Ok(Json(transition))
}

async fn create_vault_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateVaultDraftRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "vault_draft_create",
        actor.user_id.clone(),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let command = CreateVaultDraft {
                payload: payload.payload,
                attachment_refs: payload.attachment_refs,
                wali: payload.wali,
                publish_target: payload.publish_target,
                retention_policy: payload.retention_policy,
                audit: payload.audit,
                request_id: request_id.clone(),
                correlation_id: correlation_id.clone(),
                request_ts_ms: payload.request_ts_ms,
            };
            let draft = service
                .create_draft(actor, &role, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&draft).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn list_vaults(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<VaultEntry>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = VaultService::new(state.vault_repo.clone());
    let vaults = service
        .list_by_author(actor)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(vaults))
}

async fn get_vault_entry(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
) -> Result<Json<VaultEntry>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = VaultService::new(state.vault_repo.clone());
    let entry = service
        .get(&vault_entry_id)
        .await
        .map_err(map_domain_error)?;
    if !is_vault_visible_to_actor(&actor, &entry) {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(entry))
}

async fn delete_vault_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new("vault_draft_delete", actor.user_id.clone(), request_id);
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let deleted = service
                .delete_draft(actor, &vault_entry_id)
                .await
                .map_err(map_domain_error)?;
            if !deleted {
                return Err(ApiError::NotFound);
            }

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(serde_json::json!({
                    "vault_entry_id": vault_entry_id,
                    "deleted": true,
                }))
                .map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn update_vault_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<UpdateVaultDraftRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "vault_draft_update",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let command = UpdateVaultDraft {
                payload: payload.payload,
                attachment_refs: payload.attachment_refs,
                publish_target: payload.publish_target,
                retention_policy: payload.retention_policy,
                audit: payload.audit,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let entry = service
                .update_draft(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn seal_vault_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<SealVaultRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "vault_entry_seal",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let command = SealVault {
                sealed_hash: payload.sealed_hash,
                encryption_key_id: payload.encryption_key_id,
                sealed_payload: payload.sealed_payload,
                publish_target: payload.publish_target,
                retention_policy: payload.retention_policy,
                audit: payload.audit,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
                sealed_at_ms: payload.sealed_at_ms,
            };
            let entry = service
                .seal(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn publish_vault_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<SimpleVaultIdempotentRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "vault_entry_publish",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            validation::validate(&payload)?;
            let service = VaultService::new(state.vault_repo.clone());
            let command = PublishVault {
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let entry = service
                .publish(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn revoke_vault_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<SimpleVaultIdempotentRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "vault_entry_revoke",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            validation::validate(&payload)?;
            let service = VaultService::new(state.vault_repo.clone());
            let command = RevokeVault {
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let entry = service
                .revoke(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn expire_vault_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<SimpleVaultIdempotentRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "vault_entry_expire",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            validation::validate(&payload)?;
            let service = VaultService::new(state.vault_repo.clone());
            let command = ExpireVault {
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let entry = service
                .expire(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn list_vault_timeline(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
) -> Result<Json<Vec<VaultTimelineEvent>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = VaultService::new(state.vault_repo.clone());
    let timeline = service
        .list_timeline(&vault_entry_id, actor)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(timeline))
}

async fn list_vault_trustees(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
) -> Result<Json<Vec<String>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = VaultService::new(state.vault_repo.clone());
    let entry = service
        .get(&vault_entry_id)
        .await
        .map_err(map_domain_error)?;
    if !is_vault_visible_to_actor(&actor, &entry) {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(entry.wali))
}

async fn add_vault_trustee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path(vault_entry_id): Path<String>,
    Json(payload): Json<AddVaultTrusteeRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "vault_trustee_add",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let command = AddTrustee {
                wali_id: payload.wali_id,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let entry = service
                .add_trustee(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn remove_vault_trustee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Path((vault_entry_id, wali_id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let role = auth.role;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "vault_trustee_remove",
        format!("{}:{vault_entry_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;
    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = VaultService::new(state.vault_repo.clone());
            let command = RemoveTrustee {
                wali_id,
                request_id,
                correlation_id,
                request_ts_ms: None,
            };
            let entry = service
                .remove_trustee(actor, &role, &vault_entry_id, command)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&entry).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

fn is_vault_visible_to_actor(actor: &ActorIdentity, entry: &VaultEntry) -> bool {
    entry.author_id == actor.user_id || entry.wali.iter().any(|wali_id| wali_id == &actor.user_id)
}

#[derive(Debug, Deserialize)]
struct ChatThreadsQuery {
    scope_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ChatMessagesQuery {
    since_created_at_ms: Option<i64>,
    since_message_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct ChatStreamEnvelope {
    event_type: &'static str,
    message: ChatMessage,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateChatThreadRequest {
    #[validate(length(min = 1, max = 128))]
    scope_id: String,
    #[validate(length(min = 1, max = 16))]
    privacy_level: String,
}

#[derive(Debug, Deserialize, Validate)]
struct SendChatMessageRequest {
    #[validate(length(min = 1, max = 2_000))]
    body: String,
    #[serde(default)]
    attachments: Vec<Value>,
}

#[derive(Debug, Deserialize, Validate)]
struct MarkChatReadCursorRequest {
    #[validate(length(min = 1, max = 128))]
    message_id: String,
}

fn build_message_catchup_from_query(query: &ChatMessagesQuery) -> Result<MessageCatchup, ApiError> {
    if query.since_created_at_ms.is_none() && query.since_message_id.is_some() {
        return Err(ApiError::Validation(
            "since_created_at_ms is required when since_message_id is provided".into(),
        ));
    }

    Ok(build_message_catchup(
        query.limit,
        query.since_created_at_ms,
        query.since_message_id.clone(),
    ))
}

fn chat_message_stream_events(message: ChatMessage) -> Event {
    Event::default()
        .event("message")
        .json_data(ChatStreamEnvelope {
            event_type: "message",
            message,
        })
        .unwrap_or_else(|_| {
            Event::default()
                .event("error")
                .data("failed-to-serialize-message")
        })
}

fn websocket_payload(message: &ChatMessage) -> String {
    serde_json::to_string(&ChatStreamEnvelope {
        event_type: "message",
        message: message.clone(),
    })
    .unwrap_or_else(|_| "{\"event_type\":\"error\",\"message\":{}}".to_string())
}

async fn create_chat_thread(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateChatThreadRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "chat_thread_create",
        actor.user_id.clone(),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ChatService::new(state.chat_repo.clone());
            let input = ChatThreadCreate {
                scope_id: payload.scope_id,
                privacy_level: payload.privacy_level,
            };
            let thread = service
                .create_thread(&actor, request_id, correlation_id, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&thread).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn list_chat_threads(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ChatThreadsQuery>,
) -> Result<Json<Vec<ChatThread>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = ChatService::new(state.chat_repo.clone());
    let threads = if let Some(scope_id) = query.scope_id {
        service
            .list_threads_by_scope(&actor, &scope_id)
            .await
            .map_err(map_domain_error)?
    } else {
        service
            .list_threads_by_user(&actor)
            .await
            .map_err(map_domain_error)?
    };
    Ok(Json(threads))
}

async fn join_chat_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let _ = correlation_id;
    let key = IdempotencyKey::new(
        "chat_thread_join",
        format!("{}:{thread_id}", actor.user_id),
        request_id,
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ChatService::new(state.chat_repo.clone());
            let member = service
                .join_thread(&actor, &thread_id)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&member).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn leave_chat_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let _ = correlation_id;
    let key = IdempotencyKey::new(
        "chat_thread_leave",
        format!("{}:{thread_id}", actor.user_id),
        request_id,
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ChatService::new(state.chat_repo.clone());
            let member = service
                .leave_thread(&actor, &thread_id)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&member).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn list_chat_members(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<ChatMember>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = ChatService::new(state.chat_repo.clone());
    service
        .assert_actor_is_member(&actor, &thread_id)
        .await
        .map_err(map_domain_error)?;
    let members = service
        .list_members(&thread_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(members))
}

async fn list_chat_messages(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<ChatMessage>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let messages = list_chat_messages_by_query(&state, &actor, &thread_id, query).await?;
    Ok(Json(messages))
}

async fn poll_chat_messages(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<ChatMessage>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let messages = list_chat_messages_by_query(&state, &actor, &thread_id, query).await?;
    Ok(Json(messages))
}

async fn list_chat_messages_by_query(
    state: &AppState,
    actor: &ActorIdentity,
    thread_id: &str,
    query: ChatMessagesQuery,
) -> Result<Vec<ChatMessage>, ApiError> {
    let service = ChatService::new(state.chat_repo.clone());
    let cursor = build_message_catchup_from_query(&query)?;
    let messages = service
        .list_messages(thread_id, actor, cursor)
        .await
        .map_err(map_domain_error)?;
    Ok(messages)
}

async fn fetch_replay_messages(
    state: &AppState,
    actor: &ActorIdentity,
    thread_id: &str,
    since_created_at_ms: i64,
    since_message_id: String,
) -> Result<Vec<ChatMessage>, ApiError> {
    let replay_query = ChatMessagesQuery {
        since_created_at_ms: Some(since_created_at_ms),
        since_message_id: Some(since_message_id),
        limit: None,
    };
    list_chat_messages_by_query(state, actor, thread_id, replay_query).await
}

async fn send_chat_message(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<SendChatMessageRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "chat_message_send",
        format!("{}:{thread_id}", actor.user_id),
        request_id.clone(),
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ChatService::new(state.chat_repo.clone());
            let thread_id_for_input = thread_id.clone();
            let input = SendMessageInput {
                thread_id: thread_id_for_input,
                body: payload.body,
                attachments: payload.attachments,
                request_id,
                correlation_id,
                occurred_at_ms: None,
            };
            let message = service
                .send_message(&actor, input)
                .await
                .map_err(map_domain_error)?;
            state
                .chat_realtime
                .publish(&thread_id, message.clone())
                .await;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&message).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn mark_chat_read_cursor(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<MarkChatReadCursorRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let _ = correlation_id;
    let key = IdempotencyKey::new(
        "chat_mark_read",
        format!("{}:{thread_id}", actor.user_id),
        request_id,
    );
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let service = ChatService::new(state.chat_repo.clone());
            let cursor = service
                .mark_read(&actor, &thread_id, payload.message_id)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&cursor).map_err(|_| ApiError::Internal)?,
            };
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

async fn stream_chat_messages_ws(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
    Extension(auth): Extension<AuthContext>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    assert_chat_stream_access(&state, &thread_id, &actor).await?;
    let receiver = state.chat_realtime.subscribe(&thread_id).await;
    let backlog = list_chat_messages_by_query(&state, &actor, &thread_id, query).await?;
    let state_clone = state.clone();
    let actor_clone = actor.clone();
    let thread_id_clone = thread_id.clone();
    Ok(ws.on_upgrade(move |socket| async move {
        handle_chat_websocket(
            socket,
            state_clone,
            thread_id_clone,
            actor_clone,
            backlog,
            receiver,
        )
        .await;
    }))
}

async fn stream_chat_messages_sse(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    assert_chat_stream_access(&state, &thread_id, &actor).await?;
    let (tx, rx) = mpsc::unbounded_channel::<Result<Event, Infallible>>();
    let mut receiver = state.chat_realtime.subscribe(&thread_id).await;
    let mut messages = list_chat_messages_by_query(&state, &actor, &thread_id, query).await?;
    let mut seen = HashSet::new();
    let mut replay_cursor = None::<(i64, String)>;

    for message in messages.drain(..) {
        seen.insert(message.message_id.clone());
        replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
        let _ = tx.send(Ok(chat_message_stream_events(message)));
    }

    let sender = tx.clone();
    let state_clone = state.clone();
    let thread_id = thread_id.clone();
    let actor_id = actor.user_id.clone();
    let actor_name = actor.username.clone();
    let actor_identity = ActorIdentity {
        user_id: actor_id,
        username: actor_name,
    };
    tokio::spawn(async move {
        let mut heartbeat = interval(Duration::from_secs(15));
        let mut seen_messages = seen;
        let mut replay_cursor = replay_cursor;
        loop {
            tokio::select! {
                event = receiver.recv() => {
                    match event {
                        Ok(message) => {
                            if !seen_messages.insert(message.message_id.clone()) {
                                continue;
                            }
                            if assert_chat_stream_access(&state_clone, &thread_id, &actor_identity)
                                .await
                                .is_err()
                            {
                                let _ = sender.send(Ok(Event::default().event("closed").data("permission_lost")));
                                break;
                            }
                            replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
                            let _ = sender.send(Ok(chat_message_stream_events(message)));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            let Some((since_created_at_ms, since_message_id)) = replay_cursor.clone()
                            else {
                                let _ = sender.send(Ok(
                                    Event::default().event("replay").data("missed_messages")
                                ));
                                continue;
                            };

                            if assert_chat_stream_access(&state_clone, &thread_id, &actor_identity)
                                .await
                                .is_err()
                            {
                                let _ = sender.send(Ok(
                                    Event::default().event("closed").data("permission_lost"),
                                ));
                                break;
                            }

                            let replay_messages =
                                match fetch_replay_messages(
                                    &state_clone,
                                    &actor_identity,
                                    &thread_id,
                                    since_created_at_ms,
                                    since_message_id,
                                )
                                .await
                            {
                                Ok(messages) => messages,
                                Err(_) => {
                                    let _ = sender.send(Ok(
                                        Event::default().event("error").data("replay_failed")
                                    ));
                                    continue;
                                }
                            };

                            let mut replayed = false;
                            for message in replay_messages {
                                if !seen_messages.insert(message.message_id.clone()) {
                                    continue;
                                }
                                replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
                                replayed = true;
                                let _ = sender.send(Ok(chat_message_stream_events(message)));
                            }

                            if !replayed {
                                let _ = sender.send(Ok(
                                    Event::default()
                                        .event("replay")
                                        .data("missed_messages"),
                                ));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = heartbeat.tick() => {
                    if sender.send(Ok(Event::default().event("ping").data("keep-alive"))).is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(Sse::new(UnboundedReceiverStream::new(rx))
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response())
}

async fn handle_chat_websocket(
    socket: WebSocket,
    state: AppState,
    thread_id: String,
    actor: ActorIdentity,
    mut backlog: Vec<ChatMessage>,
    mut receiver: tokio::sync::broadcast::Receiver<ChatMessage>,
) {
    let (mut sender, mut incoming) = socket.split();
    let mut seen = HashSet::new();
    let mut replay_cursor = None::<(i64, String)>;

    for message in backlog.drain(..) {
        seen.insert(message.message_id.clone());
        replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
        if sender
            .send(Message::Text(websocket_payload(&message)))
            .await
            .is_err()
        {
            return;
        }
    }

    let mut heartbeat = interval(Duration::from_secs(15));
    loop {
        tokio::select! {
            event = receiver.recv() => {
                match event {
                    Ok(message) => {
                        if assert_chat_stream_access(&state, &thread_id, &actor).await.is_err() {
                            let _ = sender
                                .send(Message::Close(Some(CloseFrame {
                                    code: close_code::POLICY,
                                    reason: "permission lost".into(),
                                })))
                                .await;
                            return;
                        }
                        if !seen.insert(message.message_id.clone()) {
                            continue;
                        }
                        replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
                        if sender.send(Message::Text(websocket_payload(&message))).await.is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        let _ = sender
                            .send(Message::Close(Some(CloseFrame {
                                code: close_code::AWAY,
                                reason: "stream closed".into(),
                            })))
                            .await;
                        return;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        let Some((since_created_at_ms, since_message_id)) = replay_cursor.clone() else {
                            if sender
                                .send(Message::Text(
                                    "{\"event_type\":\"error\",\"message\":\"missed_messages_reconnect\"}"
                                        .to_string(),
                                ))
                                .await
                                .is_err()
                            {
                                return;
                            }
                            continue;
                        };

                        if assert_chat_stream_access(&state, &thread_id, &actor).await.is_err() {
                            let _ = sender
                                .send(Message::Close(Some(CloseFrame {
                                    code: close_code::POLICY,
                                    reason: "permission lost".into(),
                                })))
                                .await;
                            return;
                        }

                        let replay_messages = match fetch_replay_messages(
                            &state,
                            &actor,
                            &thread_id,
                            since_created_at_ms,
                            since_message_id,
                        )
                        .await
                        {
                            Ok(messages) => messages,
                            Err(_) => {
                                if sender
                                    .send(Message::Text(
                                        "{\"event_type\":\"error\",\"message\":\"replay_failed\"}"
                                            .to_string(),
                                    ))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                continue;
                            }
                        };

                        let mut replayed = false;
                        for message in replay_messages {
                            if !seen.insert(message.message_id.clone()) {
                                continue;
                            }
                            replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
                            if sender.send(Message::Text(websocket_payload(&message))).await.is_err() {
                                return;
                            }
                            replayed = true;
                        }

                        if !replayed {
                            if sender
                                .send(Message::Text(
                                    "{\"event_type\":\"error\",\"message\":\"missed_messages_reconnect\"}"
                                        .to_string(),
                                ))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                }
            }
            incoming = incoming.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Close(_) => return,
                            _ => {}
                        }
                    }
                    Some(Err(_)) | None => return,
                }
            }
            _ = heartbeat.tick() => {
                if sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn assert_chat_stream_access(
    state: &AppState,
    thread_id: &str,
    actor: &ActorIdentity,
) -> Result<(), ApiError> {
    let service = ChatService::new(state.chat_repo.clone());
    service
        .assert_actor_is_member(actor, thread_id)
        .await
        .map_err(map_domain_error)
}

async fn get_chat_read_cursor(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<ChatReadCursor>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = ChatService::new(state.chat_repo.clone());
    service
        .assert_actor_is_member(&actor, &thread_id)
        .await
        .map_err(map_domain_error)?;
    let cursor = service
        .get_read_cursor(&actor, &thread_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(cursor))
}

fn actor_identity(auth: &AuthContext) -> Result<ActorIdentity, ApiError> {
    let user_id = auth
        .user_id
        .as_ref()
        .filter(|user_id| !user_id.trim().is_empty())
        .ok_or(ApiError::Unauthorized)?;
    Ok(ActorIdentity {
        user_id: user_id.to_string(),
        username: user_id.to_string(),
    })
}

fn request_id_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| ApiError::Validation("missing request id".into()))
}

fn correlation_id_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
    headers
        .get(app_middleware::CORRELATION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| ApiError::Validation("missing correlation id".into()))
}

fn map_domain_error(err: DomainError) -> ApiError {
    match err {
        DomainError::Validation(message) => ApiError::Validation(message),
        DomainError::NotFound => ApiError::NotFound,
        DomainError::Conflict => ApiError::Conflict,
        DomainError::Forbidden(_) => ApiError::Forbidden,
    }
}

fn to_response(response: IdempotencyResponse) -> Response {
    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
    (status, Json(response.body)).into_response()
}

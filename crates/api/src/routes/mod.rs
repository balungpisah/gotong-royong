use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Path, Query, State};
use axum::{
    Json, Router,
    extract::ws::close_code,
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_TYPE, HeaderValue},
    },
    middleware,
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use futures_util::{SinkExt, StreamExt};
use gotong_domain::{
    adaptive_path::{
        AdaptivePathBranchDraftInput, AdaptivePathCheckpointDraftInput, AdaptivePathEditorRole,
        AdaptivePathEvent, AdaptivePathPhaseDraftInput, AdaptivePathPlan,
        AdaptivePathPlanPayloadDraft, AdaptivePathService, AdaptivePathSuggestion,
        CreateAdaptivePathInput, SuggestAdaptivePathInput, SuggestionReviewInput,
        UpdateAdaptivePathInput,
    },
    chat::{
        ChatMember, ChatMessage, ChatReadCursor, ChatService, ChatThread, ChatThreadCreate,
        MessageCatchup, SendMessageInput, build_message_catchup,
    },
    contributions::{Contribution, ContributionCreate, ContributionService, ContributionType},
    discovery::{
        DiscoveryService, FEED_SOURCE_CONTRIBUTION, FEED_SOURCE_ONTOLOGY_NOTE, FEED_SOURCE_VOUCH,
        FeedIngestInput, FeedListQuery, InAppNotification, NotificationListQuery, PagedFeed,
        PagedNotifications, SearchListQuery, SearchPage, WeeklyDigest,
    },
    error::DomainError,
    evidence::{Evidence, EvidenceCreate, EvidenceService, EvidenceType},
    idempotency::BeginOutcome,
    identity::ActorIdentity,
    jobs::{
        JobDefaults, ModerationAutoReleasePayload, OntologyNoteEnrichPayload, WebhookRetryPayload,
        new_job,
    },
    mode::Mode,
    moderation::{
        ContentModeration, ModerationApplyCommand, ModerationDecision, ModerationService,
    },
    ontology::{
        ActionType, NoteFeedbackCounts, OntologyConcept, OntologyEdgeKind, OntologyNoteCreate,
        OntologyTripleCreate,
    },
    ports::idempotency::{IdempotencyKey, IdempotencyResponse},
    ports::jobs::JobType,
    ranking::wilson_score,
    vault::{
        AddTrustee, CreateVaultDraft, ExpireVault, PublishVault, RemoveTrustee, RevokeVault,
        SealVault, UpdateVaultDraft, VaultEntry, VaultService, VaultTimelineEvent,
    },
    vouches::{Vouch, VouchCreate, VouchService, VouchWeightHint},
    webhook::{
        WebhookDeliveryLog, WebhookOutboxEvent, WebhookOutboxListQuery, WebhookOutboxStatus,
    },
};
use gotong_infra::auth::{SigninParams, SignupParams};
use gotong_infra::markov_client::{CacheMetadata, CachedJson, MarkovClientError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::wrappers::UnboundedReceiverStream;
use validator::Validate;

mod edgepod;

use crate::middleware::AuthContext;
use crate::request_repos;
use crate::{
    error::ApiError, middleware as app_middleware, observability, state::AppState, validation,
};

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
        .route("/v1/adaptive-path/plans", post(create_adaptive_path_plan))
        .route(
            "/v1/adaptive-path/plans/:plan_id",
            get(get_adaptive_path_plan),
        )
        .route(
            "/v1/adaptive-path/entities/:entity_id/plan",
            get(get_adaptive_path_plan_by_entity),
        )
        .route(
            "/v1/adaptive-path/plans/:plan_id/update",
            post(update_adaptive_path_plan),
        )
        .route(
            "/v1/adaptive-path/plans/:plan_id/events",
            get(list_adaptive_path_events),
        )
        .route(
            "/v1/adaptive-path/plans/:plan_id/suggestions",
            post(propose_adaptive_path_suggestion).get(list_adaptive_path_suggestions),
        )
        .route(
            "/v1/adaptive-path/suggestions/:suggestion_id/accept",
            post(accept_adaptive_path_suggestion),
        )
        .route(
            "/v1/adaptive-path/suggestions/:suggestion_id/reject",
            post(reject_adaptive_path_suggestion),
        )
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
        .route("/v1/feed", get(list_discovery_feed))
        .route("/v1/search", get(list_discovery_search))
        .route("/v1/ontology/concepts", post(upsert_ontology_concept))
        .route(
            "/v1/ontology/concepts/:qid",
            get(get_ontology_concept_by_qid),
        )
        .route(
            "/v1/ontology/concepts/:concept_id/broader/:broader_id",
            post(add_ontology_broader_edge),
        )
        .route(
            "/v1/ontology/concepts/:concept_id/hierarchy",
            get(list_ontology_hierarchy),
        )
        .route("/v1/ontology/feed", post(create_ontology_feed))
        .route(
            "/v1/ontology/notes/:note_id/vouches",
            post(vouch_ontology_note),
        )
        .route(
            "/v1/ontology/notes/:note_id/challenges",
            post(challenge_ontology_note),
        )
        .route(
            "/v1/ontology/notes/:note_id/feedback",
            get(get_ontology_note_feedback),
        )
        .route(
            "/v1/ontology/notes/:note_id/ranked",
            get(get_ontology_note_ranking),
        )
        .route(
            "/v1/notifications/weekly-digest",
            get(discovery_weekly_digest),
        )
        .route(
            "/v1/notifications/unread-count",
            get(discovery_unread_count),
        )
        .route(
            "/v1/notifications/:notification_id/read",
            post(mark_notification_read),
        )
        .route("/v1/notifications", get(list_discovery_notifications))
        .route("/v1/tandang/me/profile", get(get_tandang_profile_snapshot))
        .route("/v1/tandang/cv-hidup/qr", get(get_tandang_cv_hidup_qr))
        .route(
            "/v1/tandang/cv-hidup/export",
            post(post_tandang_cv_hidup_export),
        )
        .route(
            "/v1/tandang/cv-hidup/verify/:export_id",
            get(get_tandang_cv_hidup_verify),
        )
        .route("/v1/tandang/skills/search", get(search_tandang_skills))
        .route(
            "/v1/tandang/skills/nodes/:node_id",
            get(get_tandang_skill_node),
        )
        .route(
            "/v1/tandang/skills/nodes/:node_id/labels",
            get(get_tandang_skill_node_labels),
        )
        .route(
            "/v1/tandang/skills/nodes/:node_id/relations",
            get(get_tandang_skill_node_relations),
        )
        .route(
            "/v1/tandang/skills/:skill_id/parent",
            get(get_tandang_skill_parent),
        )
        .route(
            "/v1/tandang/por/requirements/:task_type",
            get(get_tandang_por_requirements),
        )
        .route(
            "/v1/tandang/por/status/:evidence_id",
            get(get_tandang_por_status),
        )
        .route(
            "/v1/tandang/por/triad-requirements/:track/:transition",
            get(get_tandang_por_triad_requirements),
        )
        .route(
            "/v1/tandang/reputation/leaderboard",
            get(get_tandang_reputation_leaderboard),
        )
        .route(
            "/v1/tandang/reputation/distribution",
            get(get_tandang_reputation_distribution),
        )
        .route("/v1/tandang/slash/gdf", get(get_tandang_gdf_weather))
        .route(
            "/v1/tandang/users/:user_id/vouch-budget",
            get(get_tandang_vouch_budget),
        )
        .route(
            "/v1/tandang/decay/warnings/:user_id",
            get(get_tandang_decay_warnings),
        )
        .route(
            "/v1/tandang/community/pulse/overview",
            get(get_tandang_community_pulse_overview),
        )
        .route(
            "/v1/tandang/community/pulse/insights",
            get(get_tandang_community_pulse_insights),
        )
        .route(
            "/v1/tandang/community/pulse/trends",
            get(get_tandang_community_pulse_trends),
        )
        .route(
            "/v1/tandang/hero/leaderboard",
            get(get_tandang_hero_leaderboard),
        )
        .route("/v1/tandang/hero/:user_id", get(get_tandang_hero_status))
        .route("/v1/admin/webhooks/outbox", get(list_webhook_outbox))
        .route(
            "/v1/admin/webhooks/outbox/:event_id",
            get(get_webhook_outbox_event),
        )
        .route(
            "/v1/admin/webhooks/outbox/:event_id/logs",
            get(list_webhook_outbox_logs),
        )
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
        .route(
            "/v1/edge-pod/ai/03/duplicate-detection",
            post(edgepod::edgepod_duplicate_detection),
        )
        .route(
            "/v1/edge-pod/ai/05/gaming-risk",
            post(edgepod::edgepod_gaming_risk),
        )
        .route(
            "/v1/edge-pod/ai/08/sensitive-media",
            post(edgepod::edgepod_sensitive_media),
        )
        .route(
            "/v1/edge-pod/ai/09/credit-recommendation",
            post(edgepod::edgepod_credit_recommendation),
        )
        .route(
            "/v1/edge-pod/ai/siaga/evaluate",
            post(edgepod::edgepod_siaga_evaluate),
        )
        .route_layer(middleware::from_fn(app_middleware::require_auth_middleware));

    let api_edgepod_routes = Router::new()
        .route(
            "/api/v1/edge-pod/ai/03/duplicate-detection",
            post(edgepod::edgepod_duplicate_detection),
        )
        .route(
            "/api/v1/edge-pod/ai/05/gaming-risk",
            post(edgepod::edgepod_gaming_risk),
        )
        .route(
            "/api/v1/edge-pod/ai/08/sensitive-media",
            post(edgepod::edgepod_sensitive_media),
        )
        .route(
            "/api/v1/edge-pod/ai/09/credit-recommendation",
            post(edgepod::edgepod_credit_recommendation),
        )
        .route(
            "/api/v1/edge-pod/ai/siaga/evaluate",
            post(edgepod::edgepod_siaga_evaluate),
        )
        .route_layer(middleware::from_fn(app_middleware::require_auth_middleware));

    let mut app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/v1/echo", post(echo))
        .route("/v1/auth/signup", post(auth_signup))
        .route("/v1/auth/signin", post(auth_signin))
        .route("/v1/auth/refresh", post(auth_refresh))
        .route("/v1/auth/logout", post(auth_logout))
        .route("/v1/auth/me", get(auth_me))
        .merge(protected)
        .merge(api_edgepod_routes)
        .layer(middleware::from_fn(app_middleware::metrics_layer))
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

async fn metrics() -> Response {
    let Some(body) = observability::render_metrics() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

#[derive(Debug, Deserialize, Validate)]
struct AuthSignupRequest {
    #[validate(length(min = 3, max = 256))]
    email: String,
    #[validate(length(min = 8, max = 256))]
    pass: String,
    #[validate(length(min = 3, max = 64))]
    username: String,
    #[validate(length(min = 1, max = 64))]
    community_id: String,
}

#[derive(Debug, Deserialize, Validate)]
struct AuthSigninRequest {
    #[validate(length(min = 3, max = 256))]
    email: String,
    #[validate(length(min = 1, max = 256))]
    pass: String,
}

#[derive(Debug, Deserialize, Validate)]
struct AuthRefreshRequest {
    #[validate(length(min = 8, max = 512))]
    refresh: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: Option<String>,
    user_id: String,
    username: String,
    role: String,
}

async fn auth_signup(
    State(state): State<AppState>,
    Json(payload): Json<AuthSignupRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    validation::validate(&payload)?;
    let auth = state.auth_service.as_ref().ok_or(ApiError::Internal)?;
    let (tokens, identity) = auth
        .signup(SignupParams {
            email: payload.email,
            pass: payload.pass,
            username: payload.username,
            community_id: payload.community_id,
        })
        .await
        .map_err(|err| {
            tracing::warn!(error = ?err, "auth signup failed");
            map_auth_error(err)
        })?;
    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        user_id: identity.user_id,
        username: identity.username,
        role: identity.platform_role,
    }))
}

async fn auth_signin(
    State(state): State<AppState>,
    Json(payload): Json<AuthSigninRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    validation::validate(&payload)?;
    let auth = state.auth_service.as_ref().ok_or(ApiError::Internal)?;
    let (tokens, identity) = auth
        .signin_password(SigninParams {
            email: payload.email,
            pass: payload.pass,
        })
        .await
        .map_err(|err| {
            tracing::warn!(error = ?err, "auth signin failed");
            map_auth_error(err)
        })?;
    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        user_id: identity.user_id,
        username: identity.username,
        role: identity.platform_role,
    }))
}

async fn auth_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuthRefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    validation::validate(&payload)?;
    let auth = state.auth_service.as_ref().ok_or(ApiError::Internal)?;
    let access_token = crate::middleware::auth_token(&headers).ok_or(ApiError::Unauthorized)?;
    let (tokens, identity) = auth
        .refresh(access_token, &payload.refresh)
        .await
        .map_err(|err| {
            tracing::warn!(error = ?err, "auth refresh failed");
            map_auth_error(err)
        })?;
    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        user_id: identity.user_id,
        username: identity.username,
        role: identity.platform_role,
    }))
}

async fn auth_me(Extension(auth): Extension<AuthContext>) -> Result<Json<AuthResponse>, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Unauthorized);
    }
    Ok(Json(AuthResponse {
        access_token: auth.access_token.unwrap_or_default(),
        refresh_token: None,
        user_id: auth.user_id.unwrap_or_default(),
        username: auth.username.unwrap_or_default(),
        role: auth.role.as_str().to_string(),
    }))
}

async fn auth_logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<StatusCode, ApiError> {
    if !auth.is_authenticated {
        return Err(ApiError::Unauthorized);
    }
    let Some(access_token) = auth.access_token.as_deref() else {
        return Err(ApiError::Unauthorized);
    };
    let Some(service) = &state.auth_service else {
        return Err(ApiError::Internal);
    };
    service
        .revoke_access_token(access_token)
        .await
        .map_err(|err| {
            tracing::warn!(error = ?err, "auth logout failed");
            map_auth_error(err)
        })?;
    Ok(StatusCode::NO_CONTENT)
}

fn map_auth_error(err: anyhow::Error) -> ApiError {
    let msg = err.to_string().to_ascii_lowercase();
    if msg.contains("already exists")
        || msg.contains("duplicate")
        || msg.contains("unique")
        || msg.contains("conflict")
    {
        return ApiError::Conflict;
    }
    if msg.contains("authentication") || msg.contains("unauthorized") || msg.contains("forbidden") {
        return ApiError::Unauthorized;
    }
    ApiError::Internal
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
struct UpsertOntologyConceptRequest {
    pub concept_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub qid: String,
    pub label_id: Option<String>,
    pub label_en: Option<String>,
    pub verified: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateOntologyFeedRequest {
    pub note_id: Option<String>,
    #[validate(length(min = 1, max = 2_000))]
    pub content: String,
    #[validate(length(min = 1, max = 128))]
    pub community_id: String,
    #[validate(length(min = 1, max = 32))]
    pub temporal_class: String,
    pub ttl_expires_ms: Option<i64>,
    pub ai_readable: Option<bool>,
    pub rahasia_level: Option<i64>,
    pub confidence: Option<f64>,
    pub triples: Option<Vec<CreateOntologyFeedTripleRequest>>,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateOntologyFeedTripleRequest {
    pub edge: OntologyEdgeKind,
    pub from_id: Option<String>,
    #[validate(length(min = 1, max = 256))]
    pub to_id: String,
    pub predicate: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct OntologyFeedbackRequest {
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
struct OntologyFeedbackResponse {
    note_id: String,
    vouch_count: usize,
    challenge_count: usize,
}

#[derive(Debug, Serialize)]
struct OntologyRankingResponse {
    note_id: String,
    vouch_count: usize,
    challenge_count: usize,
    score: f64,
}

fn normalize_ontology_temporal_class(value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "ephemeral" | "periodic" | "persistent") {
        Ok(normalized)
    } else {
        Err(ApiError::Validation(
            "temporal_class must be one of: ephemeral, periodic, persistent".to_string(),
        ))
    }
}

fn validate_ontology_action_predicate(
    edge: &OntologyEdgeKind,
    predicate: Option<&str>,
) -> Result<(), ApiError> {
    if *edge != OntologyEdgeKind::HasAction {
        return Ok(());
    }
    let Some(predicate) = predicate.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(ApiError::Validation(
            "predicate is required when edge is HasAction".to_string(),
        ));
    };
    if !predicate.starts_with("schema:") {
        return Err(ApiError::Validation(
            "HasAction predicate must start with 'schema:'".to_string(),
        ));
    }
    Ok(())
}

fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let trimmed = input.trim();
    let mut out = trimmed.chars().take(max_chars).collect::<String>();
    if trimmed.chars().count() > max_chars {
        out.push('…');
    }
    out
}

fn ontology_note_title(content: &str) -> String {
    truncate_with_ellipsis(content, 96)
}

async fn build_ontology_note_enrichment(
    state: &AppState,
    auth: &AuthContext,
    triples: &[OntologyTripleCreate],
    feedback: &NoteFeedbackCounts,
) -> Result<Value, ApiError> {
    let mut concept_qids = HashSet::new();
    let mut action_types = HashSet::new();
    let mut place_ids = HashSet::new();

    for triple in triples {
        if triple.to_id.starts_with("concept:") {
            if let Some(qid) = triple.to_id.strip_prefix("concept:") {
                let qid = qid.trim();
                if !qid.is_empty() {
                    concept_qids.insert(qid.to_string());
                }
            }
        }
        if triple.edge == OntologyEdgeKind::HasAction {
            if let Some(predicate) = triple.predicate.as_deref().map(str::trim) {
                if !predicate.is_empty() {
                    action_types.insert(predicate.to_string());
                }
            }
        }
        if triple.edge == OntologyEdgeKind::LocatedAt {
            let place_id = triple.to_id.trim();
            if place_id.starts_with("place:") {
                place_ids.insert(place_id.to_string());
            }
        }
    }

    let repo = request_repos::ontology_repo(state, auth);
    let mut concept_qids = concept_qids.into_iter().collect::<Vec<_>>();
    concept_qids.sort();
    let mut action_types = action_types.into_iter().collect::<Vec<_>>();
    action_types.sort();
    let mut place_ids = place_ids.into_iter().collect::<Vec<_>>();
    place_ids.sort();

    let concepts = repo
        .get_concepts_by_qids(&concept_qids)
        .await
        .map_err(map_domain_error)?;
    let actions = repo
        .get_actions_by_types(&action_types)
        .await
        .map_err(map_domain_error)?;
    let places = repo
        .get_places_by_ids(&place_ids)
        .await
        .map_err(map_domain_error)?;

    let total_feedback = feedback.vouch_count + feedback.challenge_count;
    let score = wilson_score(feedback.vouch_count as u64, total_feedback as u64);

    let enriched_at_ms = gotong_domain::jobs::now_ms();
    Ok(json!({
        "status": "computed",
        "enriched_at_ms": enriched_at_ms,
        "tags_enriched_at_ms": enriched_at_ms,
        "feedback_enriched_at_ms": enriched_at_ms,
        "tags": {
            "concept_qids": concept_qids,
            "action_types": action_types,
            "place_ids": place_ids,
        },
        "labels": {
            "concepts": concepts,
            "actions": actions,
            "places": places,
        },
        "feedback": {
            "vouch_count": feedback.vouch_count,
            "challenge_count": feedback.challenge_count,
            "score": score,
        }
    }))
}

fn build_pending_ontology_note_enrichment(feedback: &NoteFeedbackCounts) -> Value {
    let feedback_enriched_at_ms = gotong_domain::jobs::now_ms();
    let total_feedback = feedback.vouch_count + feedback.challenge_count;
    let score = wilson_score(feedback.vouch_count as u64, total_feedback as u64);
    json!({
        "status": "pending",
        "enriched_at_ms": feedback_enriched_at_ms,
        "feedback_enriched_at_ms": feedback_enriched_at_ms,
        "tags": {
            "concept_qids": [],
            "action_types": [],
            "place_ids": [],
        },
        "labels": {
            "concepts": [],
            "actions": [],
            "places": [],
        },
        "feedback": {
            "vouch_count": feedback.vouch_count,
            "challenge_count": feedback.challenge_count,
            "score": score,
        }
    })
}

fn build_feedback_patch(feedback: &NoteFeedbackCounts) -> Value {
    let total_feedback = feedback.vouch_count + feedback.challenge_count;
    let score = wilson_score(feedback.vouch_count as u64, total_feedback as u64);
    json!({
        "enrichment": {
            "status": "computed",
            "feedback_enriched_at_ms": gotong_domain::jobs::now_ms(),
            "feedback": {
                "vouch_count": feedback.vouch_count,
                "challenge_count": feedback.challenge_count,
                "score": score,
            }
        }
    })
}

async fn patch_ontology_note_feedback_in_feed(
    state: &AppState,
    note_id: &str,
    feedback: &NoteFeedbackCounts,
    action: &str,
) {
    match state
        .feed_repo
        .get_latest_by_source(FEED_SOURCE_ONTOLOGY_NOTE, note_id)
        .await
    {
        Ok(Some(item)) => {
            let payload_patch = build_feedback_patch(feedback);
            if let Err(err) = state
                .feed_repo
                .merge_payload(&item.feed_id, payload_patch)
                .await
            {
                tracing::warn!(error = %err, feed_id = %item.feed_id, note_id = %note_id, action = action, "failed to patch discovery feed payload after ontology feedback write");
            }
        }
        Ok(None) => {}
        Err(err) => {
            tracing::warn!(error = %err, note_id = %note_id, action = action, "failed to fetch latest ontology feed item for feedback patch");
        }
    }
}

fn validate_ontology_feed(payload: &CreateOntologyFeedRequest) -> Result<String, ApiError> {
    let temporal_class = normalize_ontology_temporal_class(&payload.temporal_class)?;
    if temporal_class == "ephemeral" && payload.ttl_expires_ms.is_none() {
        return Err(ApiError::Validation(
            "ttl_expires_ms is required for temporal_class=ephemeral".to_string(),
        ));
    }
    if payload.ttl_expires_ms.is_some_and(|value| value <= 0) {
        return Err(ApiError::Validation(
            "ttl_expires_ms must be a positive epoch milliseconds value".to_string(),
        ));
    }
    if !(0..=3).contains(&payload.rahasia_level.unwrap_or(0)) {
        return Err(ApiError::Validation(
            "rahasia_level must be between 0 and 3".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&payload.confidence.unwrap_or(0.5)) {
        return Err(ApiError::Validation(
            "confidence must be between 0.0 and 1.0".to_string(),
        ));
    }
    if let Some(triples) = &payload.triples {
        for triple in triples {
            validate_ontology_action_predicate(&triple.edge, triple.predicate.as_deref())?;
        }
    }
    Ok(temporal_class)
}

async fn upsert_ontology_concept(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<UpsertOntologyConceptRequest>,
) -> Result<(StatusCode, Json<OntologyConcept>), ApiError> {
    validation::validate(&payload)?;
    let concept = OntologyConcept {
        concept_id: payload.concept_id.unwrap_or_else(|| payload.qid.clone()),
        qid: payload.qid,
        label_id: payload.label_id,
        label_en: payload.label_en,
        verified: payload.verified.unwrap_or(false),
    };
    let concept = request_repos::ontology_repo(&state, &auth)
        .upsert_concept(&concept)
        .await
        .map_err(map_domain_error)?;
    Ok((StatusCode::CREATED, Json(concept)))
}

async fn get_ontology_concept_by_qid(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(qid): Path<String>,
) -> Result<Json<OntologyConcept>, ApiError> {
    let concept = request_repos::ontology_repo(&state, &auth)
        .get_concept_by_qid(&qid)
        .await
        .map_err(map_domain_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(concept))
}

async fn add_ontology_broader_edge(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path((concept_id, broader_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    request_repos::ontology_repo(&state, &auth)
        .add_broader_edge(&concept_id, &broader_id)
        .await
        .map_err(map_domain_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_ontology_hierarchy(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(concept_id): Path<String>,
) -> Result<Json<Vec<OntologyConcept>>, ApiError> {
    let concepts = request_repos::ontology_repo(&state, &auth)
        .list_broader_concepts(&concept_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(concepts))
}

async fn create_ontology_feed(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateOntologyFeedRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let temporal_class = validate_ontology_feed(&payload)?;

    let key = IdempotencyKey::new(
        "ontology_note_create",
        format!("{}:{}", actor.user_id, payload.community_id),
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
            let created_note = request_repos::ontology_repo(&state, &auth)
                .create_note(&OntologyNoteCreate {
                    note_id: payload.note_id,
                    content: payload.content,
                    author_id: actor.user_id.clone(),
                    community_id: payload.community_id,
                    temporal_class,
                    ttl_expires_ms: payload.ttl_expires_ms,
                    ai_readable: payload.ai_readable.unwrap_or(true),
                    rahasia_level: payload.rahasia_level.unwrap_or(0),
                    confidence: payload.confidence.unwrap_or(0.5),
                })
                .await
                .map_err(map_domain_error)?;

            let triples = payload
                .triples
                .unwrap_or_default()
                .into_iter()
                .map(|triple| {
                    validate_ontology_action_predicate(&triple.edge, triple.predicate.as_deref())?;
                    let predicate = triple.predicate;
                    let to_id = if triple.edge == OntologyEdgeKind::HasAction {
                        let predicate = predicate
                            .as_deref()
                            .unwrap_or_default()
                            .trim()
                            .strip_prefix("schema:")
                            .unwrap_or_default()
                            .to_string();
                        format!("action:{predicate}")
                    } else {
                        triple.to_id
                    };
                    Ok(OntologyTripleCreate {
                        edge: triple.edge,
                        from_id: triple
                            .from_id
                            .unwrap_or_else(|| format!("note:{}", created_note.note_id)),
                        to_id,
                        predicate,
                        metadata: triple.metadata,
                    })
                })
                .collect::<Result<Vec<_>, ApiError>>()?;

            if !triples.is_empty() {
                request_repos::ontology_repo(&state, &auth)
                    .write_triples(&triples)
                    .await
                    .map_err(map_domain_error)?;
            }

            let feedback = request_repos::ontology_repo(&state, &auth)
                .note_feedback_counts(&created_note.note_id)
                .await
                .map_err(map_domain_error)?;

            let mut response_body = json!({
                "note": created_note,
                "triple_count": triples.len(),
                "feedback": feedback,
            });

            if response_body
                .get("note")
                .and_then(|note| note.get("rahasia_level"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0)
                == 0
            {
                let note_id = response_body
                    .get("note")
                    .and_then(|note| note.get("note_id"))
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let community_id = response_body
                    .get("note")
                    .and_then(|note| note.get("community_id"))
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let content = response_body
                    .get("note")
                    .and_then(|note| note.get("content"))
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                let created_at_ms = response_body
                    .get("note")
                    .and_then(|note| note.get("created_at_ms"))
                    .and_then(|value| value.as_i64())
                    .unwrap_or_else(gotong_domain::jobs::now_ms);

                let (enrichment, enqueue_async_enrichment) = match build_ontology_note_enrichment(
                    &state, &auth, &triples, &feedback,
                )
                .await
                {
                    Ok(enrichment) => (enrichment, false),
                    Err(err) => {
                        tracing::warn!(error = %err, note_id = %note_id, "failed to build ontology note enrichment");
                        (build_pending_ontology_note_enrichment(&feedback), true)
                    }
                };

                let service =
                    DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
                let input = FeedIngestInput {
                    source_type: FEED_SOURCE_ONTOLOGY_NOTE.to_string(),
                    source_id: note_id.clone(),
                    actor: actor.clone(),
                    title: ontology_note_title(&content),
                    summary: None,
                    scope_id: Some(community_id),
                    privacy_level: Some("public".to_string()),
                    occurred_at_ms: Some(created_at_ms),
                    request_id: request_id.clone(),
                    correlation_id: correlation_id.clone(),
                    request_ts_ms: Some(created_at_ms),
                    participant_ids: vec![],
                    payload: Some(json!({
                        "note": response_body.get("note").cloned().unwrap_or(Value::Null),
                        "enrichment": enrichment,
                    })),
                };

                match service.ingest_feed(input).await {
                    Ok(item) => {
                        response_body["feed_id"] = json!(item.feed_id);
                        if enqueue_async_enrichment {
                            if let Some(queue) = state.job_queue.as_ref() {
                                let payload = serde_json::to_value(OntologyNoteEnrichPayload {
                                    note_id: note_id.clone(),
                                    feed_id: Some(item.feed_id.clone()),
                                    requested_ms: gotong_domain::jobs::now_ms(),
                                })
                                .map_err(|_| ApiError::Internal)?;
                                let defaults = JobDefaults { max_attempts: 3 };
                                let job = new_job(
                                    format!("ontology_note_enrich:{note_id}:{}", item.feed_id),
                                    JobType::OntologyNoteEnrich,
                                    payload,
                                    request_id.clone(),
                                    correlation_id.clone(),
                                    defaults,
                                );
                                if let Err(err) = queue.enqueue(&job).await {
                                    tracing::warn!(error = %err, feed_id = %item.feed_id, note_id = %note_id, "failed to enqueue ontology note enrich job");
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, note_id = %note_id, "failed to ingest ontology note into discovery feed");
                    }
                }
            }

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: response_body,
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

async fn vouch_ontology_note(
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<OntologyFeedbackRequest>,
) -> Result<(StatusCode, Json<OntologyFeedbackResponse>), ApiError> {
    let actor = actor_identity(&auth)?;
    let write_result = request_repos::ontology_repo(&state, &auth)
        .write_triples(&[OntologyTripleCreate {
            edge: OntologyEdgeKind::Vouches,
            from_id: format!("warga:{}", actor.user_id),
            to_id: format!("note:{note_id}"),
            predicate: None,
            metadata: payload.metadata,
        }])
        .await;
    match write_result {
        Ok(()) => {}
        Err(DomainError::Conflict) => {}
        Err(err) => return Err(map_domain_error(err)),
    }

    let counts = request_repos::ontology_repo(&state, &auth)
        .note_feedback_counts(&note_id)
        .await
        .map_err(map_domain_error)?;
    patch_ontology_note_feedback_in_feed(&state, &note_id, &counts, "vouch").await;
    Ok((
        StatusCode::CREATED,
        Json(OntologyFeedbackResponse {
            note_id,
            vouch_count: counts.vouch_count,
            challenge_count: counts.challenge_count,
        }),
    ))
}

async fn challenge_ontology_note(
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<OntologyFeedbackRequest>,
) -> Result<(StatusCode, Json<OntologyFeedbackResponse>), ApiError> {
    let actor = actor_identity(&auth)?;
    let write_result = request_repos::ontology_repo(&state, &auth)
        .write_triples(&[OntologyTripleCreate {
            edge: OntologyEdgeKind::Challenges,
            from_id: format!("warga:{}", actor.user_id),
            to_id: format!("note:{note_id}"),
            predicate: None,
            metadata: payload.metadata,
        }])
        .await;
    match write_result {
        Ok(()) => {}
        Err(DomainError::Conflict) => {}
        Err(err) => return Err(map_domain_error(err)),
    }

    let counts = request_repos::ontology_repo(&state, &auth)
        .note_feedback_counts(&note_id)
        .await
        .map_err(map_domain_error)?;
    patch_ontology_note_feedback_in_feed(&state, &note_id, &counts, "challenge").await;
    Ok((
        StatusCode::CREATED,
        Json(OntologyFeedbackResponse {
            note_id,
            vouch_count: counts.vouch_count,
            challenge_count: counts.challenge_count,
        }),
    ))
}

async fn get_ontology_note_feedback(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(note_id): Path<String>,
) -> Result<Json<OntologyFeedbackResponse>, ApiError> {
    let counts = request_repos::ontology_repo(&state, &auth)
        .note_feedback_counts(&note_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(OntologyFeedbackResponse {
        note_id,
        vouch_count: counts.vouch_count,
        challenge_count: counts.challenge_count,
    }))
}

async fn get_ontology_note_ranking(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(note_id): Path<String>,
) -> Result<Json<OntologyRankingResponse>, ApiError> {
    let counts = request_repos::ontology_repo(&state, &auth)
        .note_feedback_counts(&note_id)
        .await
        .map_err(map_domain_error)?;
    let total_feedback = counts.vouch_count + counts.challenge_count;
    let score = wilson_score(counts.vouch_count as u64, total_feedback as u64);
    Ok(Json(OntologyRankingResponse {
        note_id,
        vouch_count: counts.vouch_count,
        challenge_count: counts.challenge_count,
        score,
    }))
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
    pub mode: Mode,
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
            let service = ContributionService::new(request_repos::contribution_repo(&state, &auth));
            let input = ContributionCreate {
                mode: payload.mode,
                contribution_type: payload.contribution_type,
                title: payload.title,
                description: payload.description,
                evidence_url: payload.evidence_url,
                skill_ids: payload.skill_ids,
                metadata: payload.metadata,
            };

            let contribution = service
                .create(
                    actor.clone(),
                    request_id.clone(),
                    correlation_id.clone(),
                    input,
                )
                .await
                .map_err(map_domain_error)?;
            if state.config.webhook_enabled {
                if let Err(err) = enqueue_webhook_outbox_event(
                    &state,
                    &request_id,
                    &correlation_id,
                    ContributionService::into_tandang_event_payload(&contribution),
                )
                .await
                {
                    tracing::warn!(
                        error = %err,
                        event_type = "contribution_created",
                        "failed to enqueue contribution webhook outbox event"
                    );
                }
            }
            ingest_discovery_contribution_feed(
                &state,
                &actor,
                request_id.to_string(),
                correlation_id.to_string(),
                &contribution,
            )
            .await?;

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

    let service = ContributionService::new(request_repos::contribution_repo(&state, &auth));
    let contributions = service
        .list_by_author(&author_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(contributions))
}

#[derive(Debug, Deserialize)]
struct FeedListQueryParams {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SearchListQueryParams {
    #[serde(alias = "query")]
    pub query_text: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub involvement_only: Option<bool>,
    pub exclude_vault: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct NotificationsListQueryParams {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub include_read: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct WeeklyDigestQuery {
    pub window_start_ms: Option<i64>,
    pub window_end_ms: Option<i64>,
}

#[derive(Serialize)]
struct DiscoveryUnreadCountResponse {
    unread_count: usize,
}

async fn list_discovery_feed(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<FeedListQueryParams>,
) -> Result<Json<PagedFeed>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let request = FeedListQuery {
        actor_id: actor.user_id,
        cursor: query.cursor,
        limit: query.limit,
        scope_id: query.scope_id,
        privacy_level: query.privacy_level,
        from_ms: query.from_ms,
        to_ms: query.to_ms,
        involvement_only: query.involvement_only.unwrap_or(false),
    };
    let response = service.list_feed(request).await.map_err(map_domain_error)?;
    Ok(Json(response))
}

async fn list_discovery_search(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<SearchListQueryParams>,
) -> Result<Json<SearchPage>, ApiError> {
    let actor = actor_identity(&auth)?;
    let query_text = query
        .query_text
        .as_deref()
        .map(str::trim)
        .filter(|query_text| !query_text.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ApiError::Validation("query_text is required".into()))?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let request = SearchListQuery {
        actor_id: actor.user_id,
        query_text,
        cursor: query.cursor,
        limit: query.limit,
        scope_id: query.scope_id,
        privacy_level: query.privacy_level,
        from_ms: query.from_ms,
        to_ms: query.to_ms,
        involvement_only: query.involvement_only.unwrap_or(false),
        exclude_vault: query.exclude_vault.unwrap_or(false),
    };
    let response = service.search(request).await.map_err(map_domain_error)?;
    Ok(Json(response))
}

async fn list_discovery_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<NotificationsListQueryParams>,
) -> Result<Json<PagedNotifications>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let request = NotificationListQuery {
        actor_id: actor.user_id,
        cursor: query.cursor,
        limit: query.limit,
        include_read: query.include_read,
    };
    let response = service
        .list_notifications(request)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(response))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(notification_id): Path<String>,
) -> Result<Json<InAppNotification>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let response = service
        .mark_notification_read(&actor.user_id, &notification_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(response))
}

async fn discovery_unread_count(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<DiscoveryUnreadCountResponse>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let response = service
        .unread_notification_count(&actor.user_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(DiscoveryUnreadCountResponse {
        unread_count: response,
    }))
}

async fn discovery_weekly_digest(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<WeeklyDigestQuery>,
) -> Result<Json<WeeklyDigest>, ApiError> {
    let actor = actor_identity(&auth)?;
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let response = service
        .weekly_digest(&actor.user_id, query.window_start_ms, query.window_end_ms)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(response))
}

async fn get_contribution(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(contribution_id): Path<String>,
) -> Result<Json<Contribution>, ApiError> {
    let service = ContributionService::new(request_repos::contribution_repo(&state, &auth));
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

    ContributionService::new(request_repos::contribution_repo(&state, &auth))
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
            let service = EvidenceService::new(request_repos::evidence_repo(&state, &auth));
            let input = EvidenceCreate {
                contribution_id: payload.contribution_id,
                evidence_type: payload.evidence_type,
                evidence_data: payload.evidence_data,
                proof: payload.proof,
            };

            let evidence = service
                .submit(actor, request_id.clone(), correlation_id.clone(), input)
                .await
                .map_err(map_domain_error)?;
            if state.config.webhook_enabled {
                if let Err(err) = enqueue_webhook_outbox_event(
                    &state,
                    &request_id,
                    &correlation_id,
                    EvidenceService::into_tandang_event_payload(&evidence),
                )
                .await
                {
                    tracing::warn!(
                        error = %err,
                        event_type = "por_evidence",
                        "failed to enqueue evidence webhook outbox event"
                    );
                }

                if matches!(evidence.evidence_type, EvidenceType::WitnessAttestation) {
                    let witness_count = evidence
                        .proof
                        .get("witnesses")
                        .and_then(|value| value.as_array())
                        .map(|items| items.len() as u32)
                        .unwrap_or(0);
                    if let Err(err) = enqueue_webhook_outbox_event(
                        &state,
                        &request_id,
                        &correlation_id,
                        EvidenceService::into_co_witness_attested_payload(&evidence, witness_count),
                    )
                    .await
                    {
                        tracing::warn!(
                            error = %err,
                            event_type = "co_witness_attested",
                            "failed to enqueue co-witness webhook outbox event"
                        );
                    }
                }
            }

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
    Extension(auth): Extension<AuthContext>,
    Path(evidence_id): Path<String>,
) -> Result<Json<Evidence>, ApiError> {
    let service = EvidenceService::new(request_repos::evidence_repo(&state, &auth));
    let evidence = service.get(&evidence_id).await.map_err(map_domain_error)?;
    Ok(Json(evidence))
}

async fn list_evidence_by_contribution(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(contribution_id): Path<String>,
) -> Result<Json<Vec<Evidence>>, ApiError> {
    let service = EvidenceService::new(request_repos::evidence_repo(&state, &auth));
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

#[derive(Debug, Deserialize)]
struct AdaptivePathCheckpointDraftRequest {
    pub checkpoint_id: Option<String>,
    pub title: String,
    pub status: gotong_domain::adaptive_path::AdaptivePathStatus,
    pub order: i64,
    pub source: gotong_domain::adaptive_path::AdaptivePathSource,
}

#[derive(Debug, Deserialize)]
struct AdaptivePathPhaseDraftRequest {
    pub phase_id: Option<String>,
    pub title: String,
    pub objective: String,
    pub status: gotong_domain::adaptive_path::AdaptivePathStatus,
    pub order: i64,
    pub source: gotong_domain::adaptive_path::AdaptivePathSource,
    pub checkpoints: Vec<AdaptivePathCheckpointDraftRequest>,
}

#[derive(Debug, Deserialize)]
struct AdaptivePathBranchDraftRequest {
    pub branch_id: Option<String>,
    pub label: String,
    pub parent_checkpoint_id: Option<String>,
    pub order: i64,
    pub phases: Vec<AdaptivePathPhaseDraftRequest>,
}

#[derive(Debug, Deserialize)]
struct AdaptivePathPayloadDraftRequest {
    pub title: String,
    pub summary: Option<String>,
    pub action_type: ActionType,
    pub branches: Vec<AdaptivePathBranchDraftRequest>,
}

#[derive(Debug, Deserialize)]
struct CreateAdaptivePathPlanRequest {
    pub entity_id: String,
    pub payload: AdaptivePathPayloadDraftRequest,
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UpdateAdaptivePathPlanRequest {
    pub expected_version: u64,
    pub payload: AdaptivePathPayloadDraftRequest,
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SuggestAdaptivePathPlanRequest {
    pub base_version: u64,
    pub payload: AdaptivePathPayloadDraftRequest,
    pub rationale: Option<String>,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ReviewAdaptivePathSuggestionRequest {
    #[serde(default)]
    pub request_ts_ms: Option<i64>,
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

#[derive(Debug, Deserialize)]
struct ListWebhookOutboxQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

async fn list_webhook_outbox(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListWebhookOutboxQuery>,
) -> Result<Json<Vec<WebhookOutboxEvent>>, ApiError> {
    require_admin_role(&auth.role)?;

    let query = WebhookOutboxListQuery {
        status: match query.status.as_deref() {
            Some(status) => Some(
            WebhookOutboxStatus::parse(status).ok_or_else(|| {
                ApiError::Validation(
                    "invalid webhook status filter; use pending|in_flight|delivered|retrying|dead_letter".into(),
                )
                })?,
            ),
            None => None,
        },
        limit: query.limit.unwrap_or(100).clamp(1, 500),
    };

    let events = state
        .webhook_outbox_repo
        .list(&query)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "failed to list webhook outbox");
            ApiError::Internal
        })?;
    Ok(Json(events))
}

async fn get_webhook_outbox_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<WebhookOutboxEvent>, ApiError> {
    require_admin_role(&auth.role)?;
    let event = state
        .webhook_outbox_repo
        .get(&event_id)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "failed to get webhook outbox event");
            ApiError::Internal
        })?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(event))
}

async fn list_webhook_outbox_logs(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<WebhookDeliveryLog>>, ApiError> {
    require_admin_role(&auth.role)?;
    let event = state
        .webhook_outbox_repo
        .get(&event_id)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "failed to validate webhook outbox event existence");
            ApiError::Internal
        })?
        .ok_or(ApiError::NotFound)?;
    let _ = event;
    let logs = state
        .webhook_outbox_repo
        .list_logs(&event_id)
        .await
        .map_err(|err| {
            tracing::warn!(error = %err, "failed to list webhook outbox logs");
            ApiError::Internal
        })?;
    Ok(Json(logs))
}

fn require_admin_role(role: &gotong_domain::auth::Role) -> Result<(), ApiError> {
    if role.is_admin() {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn enqueue_webhook_outbox_event(
    state: &AppState,
    request_id: &str,
    correlation_id: &str,
    payload: Value,
) -> Result<(), ApiError> {
    let candidate = WebhookOutboxEvent::new(
        payload,
        request_id.to_string(),
        correlation_id.to_string(),
        state.config.webhook_max_attempts,
    )
    .map_err(|_| ApiError::Validation("invalid webhook payload".into()))?;
    let event = match state.webhook_outbox_repo.create(&candidate).await {
        Ok(event) => event,
        Err(gotong_domain::error::DomainError::Conflict) => state
            .webhook_outbox_repo
            .get(&candidate.event_id)
            .await
            .map_err(|err| {
                tracing::warn!(
                    error = %err,
                    "failed to fetch existing webhook outbox event after conflict"
                );
                ApiError::Internal
            })?
            .ok_or(ApiError::Conflict)?,
        Err(err) => {
            tracing::warn!(error = %err, "failed to persist webhook outbox event");
            return Err(ApiError::Internal);
        }
    };
    let queue = state.job_queue.as_ref().ok_or(ApiError::Internal)?;
    let job_payload = serde_json::to_value(WebhookRetryPayload {
        event_id: event.event_id.clone(),
    })
    .map_err(|_| ApiError::Internal)?;
    let defaults = JobDefaults {
        max_attempts: state.config.webhook_max_attempts.max(1),
    };
    let job = new_job(
        event.event_id,
        JobType::WebhookRetry,
        job_payload,
        request_id.to_string(),
        correlation_id.to_string(),
        defaults,
    );
    queue.enqueue(&job).await.map_err(|err| {
        tracing::warn!(error = %err, "failed to enqueue webhook retry job");
        ApiError::Internal
    })?;
    Ok(())
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
            let service = VouchService::new(request_repos::vouch_repo(&state, &auth));
            let input = VouchCreate {
                vouchee_id: payload.vouchee_id,
                skill_id: payload.skill_id,
                weight_hint: payload.weight_hint,
                message: payload.message,
            };

            let vouch = service
                .submit(
                    actor.clone(),
                    request_id.clone(),
                    correlation_id.clone(),
                    input,
                )
                .await
                .map_err(map_domain_error)?;
            if state.config.webhook_enabled {
                if let Err(err) = enqueue_webhook_outbox_event(
                    &state,
                    &request_id,
                    &correlation_id,
                    VouchService::into_tandang_event_payload(&vouch),
                )
                .await
                {
                    tracing::warn!(
                        error = %err,
                        event_type = "vouch_submitted",
                        "failed to enqueue vouch webhook outbox event"
                    );
                }
            }
            ingest_discovery_vouch_feed(
                &state,
                &actor,
                request_id.clone(),
                correlation_id.clone(),
                &vouch,
            )
            .await?;

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
    let service = VouchService::new(request_repos::vouch_repo(&state, &auth));

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
            let service = ModerationService::new(request_repos::moderation_repo(&state, &auth));
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

                if let Some(queue) = state.job_queue.as_ref() {
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
    let token_role = auth.role.clone();
    let service = ModerationService::new(request_repos::moderation_repo(&state, &auth));
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
    let token_role = auth.role.clone();
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let service = ModerationService::new(request_repos::moderation_repo(&state, &auth));
    let queue = service
        .list_review_queue(&token_role, limit)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(queue))
}

async fn create_adaptive_path_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateAdaptivePathPlanRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role.clone();
    let editor_roles = trusted_editor_roles(&token_role);
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let entity_id = payload.entity_id.trim().to_string();
    if entity_id.is_empty() {
        return Err(ApiError::Validation("entity_id is required".to_string()));
    }

    let key = IdempotencyKey::new(
        "adaptive_path_create",
        format!("{}:{entity_id}", actor.user_id),
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
            let service =
                AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
            let input = CreateAdaptivePathInput {
                entity_id,
                payload: into_adaptive_path_payload_draft(payload.payload),
                editor_roles,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let plan = service
                .create_plan(&actor, &token_role, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&plan).map_err(|_| ApiError::Internal)?,
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

async fn get_adaptive_path_plan(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(plan_id): Path<String>,
) -> Result<Json<AdaptivePathPlan>, ApiError> {
    let service = AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
    let plan = service.get_plan(&plan_id).await.map_err(map_domain_error)?;
    let plan = plan.ok_or(ApiError::NotFound)?;
    Ok(Json(plan))
}

async fn get_adaptive_path_plan_by_entity(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(entity_id): Path<String>,
) -> Result<Json<AdaptivePathPlan>, ApiError> {
    let service = AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
    let plan = service
        .get_plan_by_entity(&entity_id)
        .await
        .map_err(map_domain_error)?;
    let plan = plan.ok_or(ApiError::NotFound)?;
    Ok(Json(plan))
}

async fn update_adaptive_path_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<UpdateAdaptivePathPlanRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role.clone();
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let editor_roles = trusted_editor_roles(&token_role);

    let key = IdempotencyKey::new(
        "adaptive_path_update",
        format!("{}:{plan_id}", actor.user_id),
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
            let service =
                AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
            let input = UpdateAdaptivePathInput {
                plan_id,
                expected_version: payload.expected_version,
                payload: into_adaptive_path_payload_draft(payload.payload),
                editor_roles,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let plan = service
                .update_plan(&actor, &token_role, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&plan).map_err(|_| ApiError::Internal)?,
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

async fn propose_adaptive_path_suggestion(
    State(state): State<AppState>,
    Path(plan_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<SuggestAdaptivePathPlanRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role.clone();
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let editor_roles = trusted_editor_roles(&token_role);

    let key = IdempotencyKey::new(
        "adaptive_path_suggest",
        format!("{}:{plan_id}", actor.user_id),
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
            let service =
                AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
            let input = SuggestAdaptivePathInput {
                plan_id,
                base_version: payload.base_version,
                payload: into_adaptive_path_payload_draft(payload.payload),
                rationale: payload.rationale,
                model_id: payload.model_id,
                prompt_version: payload.prompt_version,
                editor_roles,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let suggestion = service
                .suggest_plan(&actor, &token_role, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&suggestion).map_err(|_| ApiError::Internal)?,
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

async fn accept_adaptive_path_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ReviewAdaptivePathSuggestionRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role.clone();
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let editor_roles = trusted_editor_roles(&token_role);

    let key = IdempotencyKey::new(
        "adaptive_path_accept",
        format!("{}:{suggestion_id}", actor.user_id),
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
            let service =
                AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
            let input = SuggestionReviewInput {
                suggestion_id,
                editor_roles,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let plan = service
                .accept_suggestion(&actor, &token_role, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&plan).map_err(|_| ApiError::Internal)?,
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

async fn reject_adaptive_path_suggestion(
    State(state): State<AppState>,
    Path(suggestion_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ReviewAdaptivePathSuggestionRequest>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let token_role = auth.role.clone();
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;
    let editor_roles = trusted_editor_roles(&token_role);

    let key = IdempotencyKey::new(
        "adaptive_path_reject",
        format!("{}:{suggestion_id}", actor.user_id),
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
            let service =
                AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
            let input = SuggestionReviewInput {
                suggestion_id,
                editor_roles,
                request_id,
                correlation_id,
                request_ts_ms: payload.request_ts_ms,
            };
            let suggestion = service
                .reject_suggestion(&actor, &token_role, input)
                .await
                .map_err(map_domain_error)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(&suggestion).map_err(|_| ApiError::Internal)?,
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

async fn list_adaptive_path_events(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(plan_id): Path<String>,
) -> Result<Json<Vec<AdaptivePathEvent>>, ApiError> {
    let service = AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
    let events = service
        .list_events(&plan_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(events))
}

async fn list_adaptive_path_suggestions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(plan_id): Path<String>,
) -> Result<Json<Vec<AdaptivePathSuggestion>>, ApiError> {
    let service = AdaptivePathService::new(request_repos::adaptive_path_repo(&state, &auth));
    let suggestions = service
        .list_suggestions(&plan_id)
        .await
        .map_err(map_domain_error)?;
    Ok(Json(suggestions))
}

async fn create_vault_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateVaultDraftRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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
    let role = auth.role.clone();
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
            let service = VaultService::new(request_repos::vault_repo(&state, &auth));
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

async fn ingest_discovery_contribution_feed(
    state: &AppState,
    actor: &ActorIdentity,
    request_id: String,
    correlation_id: String,
    contribution: &Contribution,
) -> Result<(), ApiError> {
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
    let input = FeedIngestInput {
        source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
        source_id: contribution.contribution_id.clone(),
        actor: actor.clone(),
        title: contribution.title.clone(),
        summary: contribution.description.clone(),
        scope_id: None,
        privacy_level: Some("public".to_string()),
        occurred_at_ms: Some(contribution.created_at_ms),
        request_id,
        correlation_id,
        request_ts_ms: Some(contribution.created_at_ms),
        participant_ids: vec![],
        payload: None,
    };
    service.ingest_feed(input).await.map_err(map_domain_error)?;
    Ok(())
}

async fn ingest_discovery_vouch_feed(
    state: &AppState,
    actor: &ActorIdentity,
    request_id: String,
    correlation_id: String,
    vouch: &Vouch,
) -> Result<(), ApiError> {
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
    let summary = vouch
        .message
        .clone()
        .or_else(|| Some(format!("Vouch from {}", vouch.voucher_username)));
    let input = FeedIngestInput {
        source_type: FEED_SOURCE_VOUCH.to_string(),
        source_id: vouch.vouch_id.clone(),
        actor: actor.clone(),
        title: format!("Vouch for {}", vouch.vouchee_id),
        summary,
        scope_id: None,
        privacy_level: Some("public".to_string()),
        occurred_at_ms: Some(vouch.created_at_ms),
        request_id,
        correlation_id,
        request_ts_ms: Some(vouch.created_at_ms),
        participant_ids: vec![vouch.vouchee_id.clone()],
        payload: Some(serde_json::json!({
            "vouchee_id": vouch.vouchee_id,
            "skill_id": vouch.skill_id,
            "weight_hint": vouch.weight_hint,
            "message": vouch.message,
        })),
    };
    service.ingest_feed(input).await.map_err(map_domain_error)?;
    Ok(())
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
            let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
    let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
            let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
            let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
    let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
    let chat_repo = request_repos::chat_repo(&state, &auth);
    let messages = list_chat_messages_by_query(chat_repo, &actor, &thread_id, query).await?;
    Ok(Json(messages))
}

async fn poll_chat_messages(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<ChatMessage>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let chat_repo = request_repos::chat_repo(&state, &auth);
    let messages = list_chat_messages_by_query(chat_repo, &actor, &thread_id, query).await?;
    Ok(Json(messages))
}

async fn list_chat_messages_by_query(
    chat_repo: Arc<dyn gotong_domain::ports::chat::ChatRepository>,
    actor: &ActorIdentity,
    thread_id: &str,
    query: ChatMessagesQuery,
) -> Result<Vec<ChatMessage>, ApiError> {
    let service = ChatService::new(chat_repo);
    let cursor = build_message_catchup_from_query(&query)?;
    let messages = service
        .list_messages(thread_id, actor, cursor)
        .await
        .map_err(map_domain_error)?;
    Ok(messages)
}

async fn fetch_replay_messages(
    chat_repo: Arc<dyn gotong_domain::ports::chat::ChatRepository>,
    actor: &ActorIdentity,
    thread_id: &str,
    since_created_at_ms: i64,
    since_message_id: String,
) -> Result<Vec<ChatMessage>, ApiError> {
    let replay_query = ChatMessagesQuery {
        since_created_at_ms: Some(since_created_at_ms),
        since_message_id: Some(since_message_id),
        // When the realtime receiver lags, we should replay as much as possible in one shot.
        // `build_message_catchup` clamps this to the domain max (currently 200).
        limit: Some(200),
    };
    list_chat_messages_by_query(chat_repo, actor, thread_id, replay_query).await
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
            let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
            let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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
    let chat_repo = request_repos::chat_repo(&state, &auth);
    assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor).await?;
    let receiver = state.chat_realtime.subscribe(&thread_id).await;
    let backlog = list_chat_messages_by_query(chat_repo.clone(), &actor, &thread_id, query).await?;
    let actor_clone = actor.clone();
    let thread_id_clone = thread_id.clone();
    Ok(ws.on_upgrade(move |socket| async move {
        handle_chat_websocket(
            socket,
            chat_repo,
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
    let chat_repo = request_repos::chat_repo(&state, &auth);
    assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor).await?;
    let (tx, rx) = mpsc::unbounded_channel::<Result<Event, Infallible>>();
    let mut receiver = state.chat_realtime.subscribe(&thread_id).await;
    let mut messages =
        list_chat_messages_by_query(chat_repo.clone(), &actor, &thread_id, query).await?;
    let mut seen = HashSet::new();
    let mut replay_cursor = None::<(i64, String)>;

    for message in messages.drain(..) {
        seen.insert(message.message_id.clone());
        replay_cursor = Some((message.created_at_ms, message.message_id.clone()));
        let _ = tx.send(Ok(chat_message_stream_events(message)));
    }

    let sender = tx.clone();
    let thread_id = thread_id.clone();
    let chat_repo = chat_repo.clone();
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
                            if assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor_identity)
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

                            if assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor_identity)
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
                                    chat_repo.clone(),
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
    chat_repo: Arc<dyn gotong_domain::ports::chat::ChatRepository>,
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
                        if assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor)
                            .await
                            .is_err()
                        {
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

                        if assert_chat_stream_access(chat_repo.clone(), &thread_id, &actor)
                            .await
                            .is_err()
                        {
                            let _ = sender
                                .send(Message::Close(Some(CloseFrame {
                                    code: close_code::POLICY,
                                    reason: "permission lost".into(),
                                })))
                                .await;
                            return;
                        }

                        let replay_messages = match fetch_replay_messages(
                            chat_repo.clone(),
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

                        if !replayed
                            && sender
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
            incoming = incoming.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) => return,
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return,
                }
            }
            _ = heartbeat.tick() => {
                if sender.send(Message::Ping(Vec::new())).await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn assert_chat_stream_access(
    chat_repo: Arc<dyn gotong_domain::ports::chat::ChatRepository>,
    thread_id: &str,
    actor: &ActorIdentity,
) -> Result<(), ApiError> {
    let service = ChatService::new(chat_repo);
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
    let service = ChatService::new(request_repos::chat_repo(&state, &auth));
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

fn into_adaptive_path_payload_draft(
    payload: AdaptivePathPayloadDraftRequest,
) -> AdaptivePathPlanPayloadDraft {
    AdaptivePathPlanPayloadDraft {
        title: payload.title,
        summary: payload.summary,
        action_type: payload.action_type,
        branches: payload
            .branches
            .into_iter()
            .map(|branch| AdaptivePathBranchDraftInput {
                branch_id: branch.branch_id,
                label: branch.label,
                parent_checkpoint_id: branch.parent_checkpoint_id,
                order: branch.order,
                phases: branch
                    .phases
                    .into_iter()
                    .map(|phase| AdaptivePathPhaseDraftInput {
                        phase_id: phase.phase_id,
                        title: phase.title,
                        objective: phase.objective,
                        status: phase.status,
                        order: phase.order,
                        source: phase.source,
                        checkpoints: phase
                            .checkpoints
                            .into_iter()
                            .map(|checkpoint| AdaptivePathCheckpointDraftInput {
                                checkpoint_id: checkpoint.checkpoint_id,
                                title: checkpoint.title,
                                status: checkpoint.status,
                                order: checkpoint.order,
                                source: checkpoint.source,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn trusted_editor_roles(token_role: &gotong_domain::auth::Role) -> Vec<AdaptivePathEditorRole> {
    match token_role {
        gotong_domain::auth::Role::Admin
        | gotong_domain::auth::Role::Moderator
        | gotong_domain::auth::Role::System => vec![AdaptivePathEditorRole::ProjectManager],
        gotong_domain::auth::Role::User => vec![AdaptivePathEditorRole::Author],
        gotong_domain::auth::Role::Anonymous => Vec::new(),
    }
}

#[derive(Debug, Deserialize)]
struct TandangSkillSearchQuery {
    q: String,
    lang: Option<String>,
    fuzzy: Option<bool>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TandangLeaderboardQuery {
    limit: Option<u32>,
    tier: Option<String>,
    rank_by: Option<String>,
}

async fn get_tandang_profile_snapshot(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Value>, ApiError> {
    let actor = actor_identity(&auth)?;
    let snapshot = state
        .markov_client
        .user_profile_snapshot(&actor.user_id)
        .await
        .map_err(map_markov_error)?;

    let reputation = snapshot.reputation;
    let tier = snapshot.tier;
    let activity = snapshot.activity;
    let cv_hidup = snapshot.cv_hidup;
    let top_level_cache = cache_metadata_value(&reputation.meta);

    Ok(Json(json!({
        "cache": top_level_cache,
        "data": {
            "source": "tandang",
            "identity": snapshot.identity,
            "markov_user_id": snapshot.markov_user_id,
            "reputation": reputation.value,
            "tier": tier.as_ref().map(|item| item.value.clone()),
            "activity": activity.as_ref().map(|item| item.value.clone()),
            "cv_hidup": cv_hidup.as_ref().map(|item| item.value.clone()),
            "component_cache": {
                "reputation": cache_metadata_value(&reputation.meta),
                "tier": tier.as_ref().map(|item| cache_metadata_value(&item.meta)),
                "activity": activity.as_ref().map(|item| cache_metadata_value(&item.meta)),
                "cv_hidup": cv_hidup.as_ref().map(|item| cache_metadata_value(&item.meta)),
            }
        }
    })))
}

async fn get_tandang_cv_hidup_qr(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
    let result = state
        .markov_client
        .get_cv_hidup_qr(user_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn post_tandang_cv_hidup_export(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
    let result = state
        .markov_client
        .post_cv_hidup_export(user_id, payload)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_cv_hidup_verify(
    State(state): State<AppState>,
    Path(export_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_cv_hidup_verify(&export_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn search_tandang_skills(
    State(state): State<AppState>,
    Query(query): Query<TandangSkillSearchQuery>,
) -> Result<Json<Value>, ApiError> {
    if query.q.trim().is_empty() {
        return Err(ApiError::Validation("q is required".into()));
    }
    let result = state
        .markov_client
        .search_skills(
            query.q.trim(),
            query.lang.as_deref(),
            query.fuzzy,
            query.limit,
        )
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_skill_node(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_skill_node(&node_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_skill_node_labels(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_skill_node_labels(&node_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_skill_node_relations(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_skill_node_relations(&node_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_skill_parent(
    State(state): State<AppState>,
    Path(skill_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_skill_parent(&skill_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_por_requirements(
    State(state): State<AppState>,
    Path(task_type): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if task_type.trim().is_empty() {
        return Err(ApiError::Validation("task_type is required".into()));
    }
    let result = state
        .markov_client
        .get_por_requirements(&task_type)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_por_status(
    State(state): State<AppState>,
    Path(evidence_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if evidence_id.trim().is_empty() {
        return Err(ApiError::Validation("evidence_id is required".into()));
    }
    let result = state
        .markov_client
        .get_por_status(&evidence_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_por_triad_requirements(
    State(state): State<AppState>,
    Path((track, transition)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    if track.trim().is_empty() || transition.trim().is_empty() {
        return Err(ApiError::Validation(
            "track and transition are required".into(),
        ));
    }
    let result = state
        .markov_client
        .get_por_triad_requirements(&track, &transition)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_reputation_leaderboard(
    State(state): State<AppState>,
    Query(query): Query<TandangLeaderboardQuery>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_reputation_leaderboard(query.limit, query.tier.as_deref(), query.rank_by.as_deref())
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_reputation_distribution(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_reputation_distribution()
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

fn require_self_or_admin(auth: &AuthContext, user_id: &str) -> Result<(), ApiError> {
    let requested = user_id.trim();
    if requested.is_empty() {
        return Err(ApiError::Validation("user_id is required".into()));
    }
    let actor_id = auth.user_id.as_deref().map(str::trim).unwrap_or_default();
    if actor_id == requested {
        return Ok(());
    }
    require_admin_role(&auth.role)
}

async fn get_tandang_gdf_weather(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_gdf_weather()
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_vouch_budget(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_self_or_admin(&auth, &user_id)?;
    let result = state
        .markov_client
        .get_vouch_budget(&user_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_decay_warnings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_self_or_admin(&auth, &user_id)?;
    let result = state
        .markov_client
        .get_decay_warnings(&user_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_community_pulse_overview(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_community_pulse()
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_community_pulse_insights(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_community_pulse_insights()
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

#[derive(Debug, Deserialize)]
struct TandangPulseTrendsQuery {
    period: Option<String>,
}

async fn get_tandang_community_pulse_trends(
    State(state): State<AppState>,
    Query(query): Query<TandangPulseTrendsQuery>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_community_pulse_trends(query.period.as_deref())
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

async fn get_tandang_hero_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if user_id.trim().is_empty() {
        return Err(ApiError::Validation("user_id is required".into()));
    }
    let result = state
        .markov_client
        .get_hero_status(&user_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

#[derive(Debug, Deserialize)]
struct TandangHeroLeaderboardQuery {
    limit: Option<u32>,
}

async fn get_tandang_hero_leaderboard(
    State(state): State<AppState>,
    Query(query): Query<TandangHeroLeaderboardQuery>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .markov_client
        .get_hero_leaderboard(query.limit)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(cached_json_value(result)))
}

fn cached_json_value(payload: CachedJson) -> Value {
    json!({
        "cache": cache_metadata_value(&payload.meta),
        "data": payload.value,
    })
}

fn cache_metadata_value(meta: &CacheMetadata) -> Value {
    json!({
        "status": meta.status.as_str(),
        "stale": meta.stale,
        "age_ms": meta.age_ms,
        "cached_at_epoch_ms": meta.cached_at_epoch_ms,
    })
}

fn actor_identity(auth: &AuthContext) -> Result<ActorIdentity, ApiError> {
    let user_id = auth
        .user_id
        .as_ref()
        .filter(|user_id| !user_id.trim().is_empty())
        .ok_or(ApiError::Unauthorized)?;
    let username = auth
        .username
        .as_ref()
        .filter(|username| !username.trim().is_empty())
        .unwrap_or(user_id);
    Ok(ActorIdentity {
        user_id: user_id.to_string(),
        username: username.to_string(),
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

fn map_markov_error(err: MarkovClientError) -> ApiError {
    let reason = match &err {
        MarkovClientError::BadRequest(_) => "bad_request",
        MarkovClientError::Unauthorized(_) => "unauthorized",
        MarkovClientError::Forbidden(_) => "forbidden",
        MarkovClientError::NotFound(_) => "not_found",
        MarkovClientError::CircuitOpen => "circuit_open",
        MarkovClientError::Configuration(_) => "configuration",
        MarkovClientError::Upstream(_) => "upstream",
        MarkovClientError::Transport(_) => "transport",
        MarkovClientError::InvalidResponse(_) => "invalid_response",
    };
    observability::register_markov_integration_error(reason);

    match err {
        MarkovClientError::BadRequest(message) => ApiError::Validation(message),
        MarkovClientError::Unauthorized(_) => ApiError::Unauthorized,
        MarkovClientError::Forbidden(_) => ApiError::Forbidden,
        MarkovClientError::NotFound(_) => ApiError::NotFound,
        MarkovClientError::CircuitOpen => {
            tracing::warn!("markov read circuit is open");
            ApiError::Internal
        }
        MarkovClientError::Configuration(message)
        | MarkovClientError::Upstream(message)
        | MarkovClientError::Transport(message)
        | MarkovClientError::InvalidResponse(message) => {
            tracing::warn!(error = %message, "markov upstream integration failed");
            ApiError::Internal
        }
    }
}

fn to_response(response: IdempotencyResponse) -> Response {
    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
    (status, Json(response.body)).into_response()
}

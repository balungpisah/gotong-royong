use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Multipart, Path, Query, State};
use axum::{
    Json, Router,
    extract::ws::close_code,
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, HeaderValue, LOCATION},
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
        FeedIngestInput, FeedListQuery, FeedSuggestion, FeedSuggestionsQuery, InAppNotification,
        NotificationListQuery, PagedNotifications, SearchListQuery, SearchPage, WeeklyDigest,
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
    ports::group::{GroupJoinRequestRecord, GroupMemberRecord, GroupRecord},
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
use gotong_infra::markov_client::{
    CacheMetadata, CachedJson, MarkovClientError, MarkovProfileSnapshot,
};
use hmac::{Hmac, Mac};
use rusty_s3::S3Action;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::wrappers::UnboundedReceiverStream;
use validator::Validate;

mod edgepod;

use crate::middleware::AuthContext;
use crate::request_repos;
use crate::{
    error::ApiError,
    middleware as app_middleware, observability,
    state::{
        AppState, ChatAttachmentStorage, TriageSessionMessageState, TriageSessionState,
        WitnessImpactVerificationState, WitnessSignalEntry, WitnessSignalState,
        WitnessStempelObjection, WitnessStempelState,
    },
    validation,
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
        .route("/v1/feed/suggestions", get(list_discovery_feed_suggestions))
        .route(
            "/v1/feed/preferences/monitor/:witness_id",
            post(set_feed_monitor_preference),
        )
        .route(
            "/v1/feed/preferences/follow/:entity_id",
            post(set_feed_follow_preference),
        )
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
        .route("/v1/triage/sessions", post(start_triage_session))
        .route(
            "/v1/triage/sessions/:session_id/messages",
            post(continue_triage_session),
        )
        .route(
            "/v1/witnesses/:witness_id/signals",
            post(create_witness_signal),
        )
        .route(
            "/v1/witnesses/:witness_id/signals/:signal_type",
            delete(remove_witness_signal),
        )
        .route(
            "/v1/witnesses/:witness_id/signals/my-relation",
            get(get_witness_signal_my_relation),
        )
        .route(
            "/v1/witnesses/:witness_id/signals/counts",
            get(get_witness_signal_counts),
        )
        .route(
            "/v1/witnesses/:witness_id/signals/resolutions",
            get(list_witness_signal_resolutions),
        )
        .route(
            "/v1/witnesses/:witness_id/stempel/propose",
            post(propose_witness_stempel),
        )
        .route(
            "/v1/witnesses/:witness_id/stempel/objections",
            post(submit_witness_stempel_objection),
        )
        .route(
            "/v1/witnesses/:witness_id/stempel/finalize",
            post(finalize_witness_stempel),
        )
        .route("/v1/witnesses", post(create_witness))
        .route("/v1/groups", post(create_group).get(list_groups))
        .route("/v1/groups/me", get(list_my_groups))
        .route("/v1/groups/:group_id", get(get_group).patch(update_group))
        .route("/v1/groups/:group_id/join", post(join_group))
        .route("/v1/groups/:group_id/requests", post(request_group_join))
        .route(
            "/v1/groups/:group_id/requests/:request_id/approve",
            post(approve_group_request),
        )
        .route(
            "/v1/groups/:group_id/requests/:request_id/reject",
            post(reject_group_request),
        )
        .route("/v1/groups/:group_id/invite", post(invite_group_member))
        .route("/v1/groups/:group_id/leave", post(leave_group))
        .route(
            "/v1/groups/:group_id/members/:user_id/remove",
            post(remove_group_member),
        )
        .route(
            "/v1/groups/:group_id/members/:user_id/role",
            post(update_group_member_role),
        )
        .route("/v1/tandang/me/profile", get(get_tandang_profile_snapshot))
        .route(
            "/v1/tandang/users/:user_id/profile",
            get(get_tandang_user_profile_snapshot),
        )
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
        .route("/v1/chat/attachments/upload", post(upload_chat_attachment))
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
        .route(
            "/v1/chat/attachments/:attachment_id/download",
            get(download_chat_attachment),
        )
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

const TRIAGE_SESSION_TTL_MS: i64 = 4 * 60 * 60 * 1_000;
const TRIAGE_SESSION_MAX_ITEMS: usize = 10_000;

#[derive(Debug, Deserialize, Serialize, Validate)]
struct TriageAttachmentInput {
    #[validate(length(min = 1, max = 256))]
    name: String,
    #[validate(length(min = 1, max = 128))]
    mime_type: String,
    size_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
struct StartTriageSessionRequest {
    #[validate(length(min = 1, max = 4_000))]
    content: String,
    #[validate(length(max = 10), nested)]
    attachments: Option<Vec<TriageAttachmentInput>>,
    operator_output: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
struct ContinueTriageSessionRequest {
    #[validate(length(min = 1, max = 4_000))]
    answer: String,
    #[validate(length(max = 10), nested)]
    attachments: Option<Vec<TriageAttachmentInput>>,
    operator_output: Option<Value>,
}

#[derive(Clone, Copy)]
struct DetectedTriageRoute {
    route: &'static str,
    trajectory_type: Option<&'static str>,
}

fn detect_triage_route(text: &str) -> DetectedTriageRoute {
    let normalized = text.to_ascii_lowercase();

    if normalized.contains("kelola")
        || normalized.contains("kelompok")
        || normalized.contains("mengatur kelompok")
        || normalized.contains("manage a group")
    {
        return DetectedTriageRoute {
            route: "kelola",
            trajectory_type: None,
        };
    }

    if normalized.contains("vault")
        || normalized.contains("catatan saksi")
        || normalized.contains("rahasia")
    {
        return DetectedTriageRoute {
            route: "vault",
            trajectory_type: Some("vault"),
        };
    }

    if normalized.contains("siaga")
        || normalized.contains("darurat")
        || normalized.contains("bahaya")
        || normalized.contains("peringatan")
        || normalized.contains("emergency")
    {
        return DetectedTriageRoute {
            route: "siaga",
            trajectory_type: Some("siaga"),
        };
    }

    if normalized.contains("catat")
        || normalized.contains("data")
        || normalized.contains("dokumentasi")
        || normalized.contains("fakta")
        || normalized.contains("bukti")
        || normalized.contains("document")
    {
        return DetectedTriageRoute {
            route: "catatan_komunitas",
            trajectory_type: Some("data"),
        };
    }

    if normalized.contains("musyawarah")
        || normalized.contains("diskusi")
        || normalized.contains("keputusan")
        || normalized.contains("usul")
        || normalized.contains("discussion")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("mufakat"),
        };
    }

    if normalized.contains("pantau")
        || normalized.contains("awasi")
        || normalized.contains("mengawasi")
        || normalized.contains("monitor")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("pantau"),
        };
    }

    if normalized.contains("bantuan")
        || normalized.contains("butuh")
        || normalized.contains("pertolongan")
        || normalized.contains("need help")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("bantuan"),
        };
    }

    if normalized.contains("rayakan")
        || normalized.contains("kabar baik")
        || normalized.contains("capai")
        || normalized.contains("celebrate")
        || normalized.contains("good news")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("pencapaian"),
        };
    }

    if normalized.contains("program")
        || normalized.contains("jadwal")
        || normalized.contains("kegiatan rutin")
        || normalized.contains("organize")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("program"),
        };
    }

    if normalized.contains("masalah")
        || normalized.contains("rusak")
        || normalized.contains("keluhan")
        || normalized.contains("problem")
    {
        return DetectedTriageRoute {
            route: "komunitas",
            trajectory_type: Some("aksi"),
        };
    }

    DetectedTriageRoute {
        route: "komunitas",
        trajectory_type: Some("aksi"),
    }
}

fn triage_label(route: &str) -> &'static str {
    match route {
        "kelola" => "Kelola Kelompok",
        "siaga" => "Siaga Darurat",
        "vault" => "Catatan Saksi",
        "catatan_komunitas" => "Catatan Komunitas",
        _ => "Komunitas",
    }
}

const TRIAGE_SCHEMA_VERSION: &str = "triage.v1";
const OPERATOR_SCHEMA_VERSION: &str = "operator.v1";
const STEMPEL_MIN_PARTICIPANTS: usize = 3;
const STEMPEL_DEFAULT_WINDOW_MS: i64 = 24 * 60 * 60 * 1_000;
const STEMPEL_MAX_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const IMPACT_VOUCH_DEFAULT_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const IMPACT_VOUCH_MIN_VOUCHES: usize = 3;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorChecklistItem {
    field: String,
    filled: bool,
    value: Option<String>,
    required_for_final: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorTaxonomy {
    category_code: String,
    category_label: String,
    custom_label: Option<String>,
    quality: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorProgramRef {
    program_id: String,
    label: String,
    source: String,
    confidence: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorStempelState {
    state: String,
    proposed_at_ms: Option<i64>,
    objection_deadline_ms: Option<i64>,
    locked_at_ms: Option<i64>,
    min_participants: usize,
    participant_count: usize,
    objection_count: usize,
    latest_objection_at_ms: Option<i64>,
    latest_objection_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorRouting {
    route: String,
    trajectory_type: Option<String>,
    track_hint: Option<String>,
    seed_hint: Option<String>,
    taxonomy: Option<TriageOperatorTaxonomy>,
    program_refs: Option<Vec<TriageOperatorProgramRef>>,
    stempel_state: Option<TriageOperatorStempelState>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorBlocks {
    conversation: Vec<String>,
    structured: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriageOperatorOutput {
    schema_version: String,
    operator: String,
    triage_stage: String,
    output_kind: String,
    confidence: Option<f64>,
    checklist: Vec<TriageOperatorChecklistItem>,
    questions: Option<Vec<String>>,
    missing_fields: Option<Vec<String>>,
    routing: TriageOperatorRouting,
    blocks: Option<TriageOperatorBlocks>,
    payload: Value,
}

fn triage_taxonomy_payload(route: &str, trajectory_type: Option<&str>, kind: &str) -> Value {
    let trajectory = trajectory_type.unwrap_or(if kind == "witness" { "aksi" } else { "data" });
    let (category_code, category_label, quality) = match trajectory {
        "siaga" => (
            "safety_alert",
            "Peringatan Keamanan",
            "community_observation",
        ),
        "bantuan" => ("public_service", "Layanan Publik", "community_observation"),
        "program" => ("community_event", "Program Komunitas", "official_source"),
        "pencapaian" => (
            "community_event",
            "Pencapaian Komunitas",
            "community_observation",
        ),
        "vault" => ("other_custom", "Catatan Privat", "unverified_claim"),
        "data" if route == "catatan_komunitas" => {
            ("public_service", "Data Komunitas", "community_observation")
        }
        _ if kind == "data" => ("public_service", "Data Komunitas", "community_observation"),
        _ => ("infrastructure", "Laporan Warga", "community_observation"),
    };

    json!({
        "category_code": category_code,
        "category_label": category_label,
        "quality": quality
    })
}

fn triage_program_refs_payload(trajectory_type: Option<&str>) -> Value {
    if trajectory_type == Some("program") {
        return json!([{
            "program_id": "program:community-initiative",
            "label": "Program Komunitas",
            "source": "llm_inferred",
            "confidence": 0.55
        }]);
    }
    json!([])
}

fn triage_initial_stempel_payload(trajectory_type: Option<&str>) -> Option<Value> {
    if trajectory_type != Some("mufakat") {
        return None;
    }
    Some(json!({
        "state": "draft",
        "min_participants": STEMPEL_MIN_PARTICIPANTS,
        "participant_count": 0,
        "objection_count": 0
    }))
}

fn triage_terminal_bar_state(route: &str) -> &'static str {
    match route {
        "siaga" => "siaga-ready",
        "vault" => "vault-ready",
        _ => "ready",
    }
}

fn triage_budget(turn: u8) -> Value {
    let total_tokens = 6_000_i64;
    let used_tokens = (i64::from(turn) * 900).min(total_tokens);
    let remaining_tokens = total_tokens.saturating_sub(used_tokens);
    json!({
        "total_tokens": total_tokens,
        "used_tokens": used_tokens,
        "remaining_tokens": remaining_tokens,
        "budget_pct": (used_tokens as f64) / (total_tokens as f64),
        "can_continue": remaining_tokens > 0 && turn < 8,
        "turn_count": turn,
        "max_turns": 8
    })
}

fn triage_kind(route: &str, trajectory_type: Option<&str>) -> &'static str {
    if route == "catatan_komunitas" || route == "kelola" {
        return "data";
    }

    match trajectory_type {
        Some("data" | "vault" | "bantuan" | "pencapaian" | "siaga") => "data",
        _ => "witness",
    }
}

fn triage_missing_fields(kind: &str, step: u8) -> Vec<&'static str> {
    if step >= 3 {
        return Vec::new();
    }

    if kind == "witness" {
        if step == 1 {
            return vec!["location_hint", "impact_summary", "desired_outcome"];
        }
        return vec!["impact_summary"];
    }

    if step == 1 {
        return vec!["record_category", "time_reference"];
    }

    vec!["claim_statement"]
}

fn triage_missing_field_prompt(field: &str) -> &'static str {
    match field {
        "location_hint" => "Mohon sebutkan lokasi singkat supaya konteksnya jelas.",
        "impact_summary" => "Siapa terdampak dan dampaknya apa?",
        "desired_outcome" => "Hasil apa yang paling diharapkan dari laporan ini?",
        "record_category" => "Ini termasuk catatan jenis apa menurut Anda?",
        "time_reference" => "Kapan peristiwa ini terjadi atau diamati?",
        "claim_statement" => "Tolong ringkas klaim utama yang ingin dicatat.",
        _ => "Mohon tambah detail singkat agar saya bisa melanjutkan.",
    }
}

fn triage_confidence_score(step: u8) -> f64 {
    if step >= 3 {
        0.95
    } else if step == 2 {
        0.70
    } else {
        0.40
    }
}

fn triage_bar_state(route: &str, step: u8) -> &'static str {
    if step >= 3 {
        triage_terminal_bar_state(route)
    } else if step == 2 {
        "leaning"
    } else {
        "probing"
    }
}

fn triage_operator_name(route: &str, trajectory_type: Option<&str>) -> &'static str {
    if route == "kelola" {
        return "kelola";
    }

    match trajectory_type {
        Some("mufakat" | "mediasi") => "musyawarah",
        Some("pantau") => "pantau",
        Some("data" | "vault") => "catat",
        Some("bantuan") => "bantuan",
        Some("pencapaian") => "rayakan",
        Some("siaga") => "siaga",
        Some("program") => "program",
        _ => "masalah",
    }
}

fn triage_operator_output_kind(route: &str, trajectory_type: Option<&str>) -> &'static str {
    if route == "kelola" {
        return "kelola";
    }
    triage_kind(route, trajectory_type)
}

fn triage_operator_stage(step: u8) -> &'static str {
    if step >= 3 {
        "triage_final"
    } else {
        "triage_draft"
    }
}

fn triage_operator_default_path_plan(route: &str, trajectory_type: Option<&str>) -> Value {
    let title = match trajectory_type {
        Some("advokasi") => "Rencana Advokasi Komunitas",
        Some("pantau") => "Rencana Pantau Komunitas",
        Some("program") => "Rencana Program Komunitas",
        _ if route == "siaga" => "Rencana Tanggap Siaga",
        _ => "Rencana Aksi Warga",
    };

    json!({
        "plan_id": "plan-auto-triage",
        "version": 1,
        "title": title,
        "summary": "Rencana awal yang disusun dari konteks triase.",
        "branches": [
            {
                "branch_id": "main",
                "label": "Utama",
                "parent_checkpoint_id": Value::Null,
                "phases": [
                    {
                        "phase_id": "phase-1",
                        "title": "Validasi Konteks",
                        "objective": "Pastikan lingkup dan prioritas masalah disepakati.",
                        "status": "planned",
                        "source": "ai",
                        "locked_fields": [],
                        "checkpoints": [
                            {
                                "checkpoint_id": "cp-1",
                                "title": "Konteks inti terkonfirmasi",
                                "status": "open",
                                "source": "ai",
                                "locked_fields": []
                            }
                        ]
                    }
                ]
            }
        ]
    })
}

fn triage_operator_payload(
    operator: &str,
    route: &str,
    trajectory_type: Option<&str>,
    step: u8,
) -> Value {
    let is_final = step >= 3;
    match operator {
        "masalah" => {
            let trajectory = if trajectory_type == Some("advokasi") {
                "B"
            } else {
                "A"
            };
            if is_final {
                json!({
                    "trajectory": trajectory,
                    "path_plan": triage_operator_default_path_plan(route, trajectory_type)
                })
            } else {
                json!({
                    "trajectory": trajectory
                })
            }
        }
        "musyawarah" => {
            let context = if trajectory_type == Some("mediasi") {
                "dispute"
            } else {
                "proposal"
            };
            if is_final {
                json!({
                    "context": context,
                    "decision_steps": [
                        {
                            "question": "Apakah usulan ini disepakati untuk dijalankan?",
                            "options": ["Setuju", "Tidak Setuju", "Perlu Revisi"],
                            "rationale": "Perlu keputusan awal sebelum lanjut ke eksekusi.",
                            "order": 1
                        }
                    ],
                    "stempel_candidate": {
                        "summary": "Kesepakatan awal telah dirumuskan.",
                        "rationale": "Poin utama sudah dibahas dengan pihak terkait.",
                        "objection_window_seconds": 86400
                    }
                })
            } else {
                json!({
                    "context": context
                })
            }
        }
        "pantau" => {
            if is_final {
                json!({
                    "case_type": "komunitas",
                    "timeline_seed": [
                        {
                            "event": "Laporan awal diterima",
                            "date": "2026-02-26T00:00:00Z",
                            "source": "user"
                        }
                    ],
                    "tracking_points": [
                        "Perkembangan tindak lanjut warga",
                        "Respons pihak terkait"
                    ]
                })
            } else {
                json!({
                    "case_type": "komunitas"
                })
            }
        }
        "catat" => {
            let record_type = if trajectory_type == Some("vault") {
                "vault"
            } else {
                "data"
            };
            if is_final {
                json!({
                    "record_type": record_type,
                    "claim": "Catatan komunitas terstruktur dari hasil triase.",
                    "location": "Lingkungan setempat",
                    "observed_at": "2026-02-26T00:00:00Z",
                    "category": "catatan_komunitas"
                })
            } else {
                json!({
                    "record_type": record_type,
                    "claim": "Catatan awal"
                })
            }
        }
        "bantuan" => {
            if is_final {
                json!({
                    "help_type": "dukungan_komunitas",
                    "description": "Warga memerlukan bantuan tindak lanjut.",
                    "urgency": "sedang",
                    "matched_resources": [
                        {
                            "resource_id": "resource-1",
                            "name": "Relawan Lingkungan",
                            "relevance": 0.74,
                            "match_reason": "Dekat lokasi dan relevan dengan kebutuhan."
                        }
                    ]
                })
            } else {
                json!({
                    "help_type": "dukungan_komunitas"
                })
            }
        }
        "rayakan" => {
            if is_final {
                json!({
                    "achievement": "Pencapaian komunitas teridentifikasi.",
                    "contributors": ["warga-utama"],
                    "impact_summary": "Pencapaian ini berdampak positif untuk lingkungan."
                })
            } else {
                json!({
                    "achievement": "Pencapaian awal"
                })
            }
        }
        "siaga" => {
            if is_final {
                json!({
                    "threat_type": "peringatan_warga",
                    "severity": "siaga",
                    "location": "Area komunitas",
                    "description": "Potensi risiko perlu perhatian cepat.",
                    "source": "laporan_warga",
                    "expires_at": "2026-02-27T00:00:00Z"
                })
            } else {
                json!({
                    "threat_type": "peringatan_warga",
                    "severity": "waspada"
                })
            }
        }
        "program" => {
            if is_final {
                json!({
                    "activity_name": "Program Rutin Komunitas",
                    "frequency": "mingguan",
                    "rotation": [
                        {
                            "participant": "warga-utama",
                            "slot": "Minggu ke-1"
                        }
                    ],
                    "next_occurrence": "2026-03-01T00:00:00Z"
                })
            } else {
                json!({
                    "activity_name": "Program Rutin Komunitas",
                    "frequency": "mingguan"
                })
            }
        }
        "kelola" => {
            if is_final {
                json!({
                    "action": "create",
                    "group_detail": {
                        "name": "Karang Taruna RT 05",
                        "description": "Kelompok pemuda untuk kegiatan sosial dan pembangunan di lingkungan RT 05.",
                        "join_policy": "terbuka",
                        "entity_type": "kelompok"
                    }
                })
            } else {
                json!({
                    "action": "create"
                })
            }
        }
        _ => json!({}),
    }
}

fn triage_operator_blocks_payload(operator: &str) -> Value {
    match operator {
        "musyawarah" => json!({
            "conversation": ["ai_inline_card", "diff_card", "vote_card"],
            "structured": ["document", "list", "vote", "computed"]
        }),
        "siaga" => json!({
            "conversation": ["ai_inline_card", "diff_card", "vote_card"],
            "structured": ["form", "list", "computed"]
        }),
        "pantau" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["list", "document", "computed"]
        }),
        "program" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["list", "form", "computed"]
        }),
        "catat" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["form", "document", "reference"]
        }),
        "bantuan" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["form", "list", "computed"]
        }),
        "rayakan" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["display", "document", "reference"]
        }),
        "kelola" => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["form", "list", "reference"]
        }),
        _ => json!({
            "conversation": ["ai_inline_card", "diff_card"],
            "structured": ["document", "list", "computed"]
        }),
    }
}

fn triage_operator_output_payload(route: &str, trajectory_type: Option<&str>, step: u8) -> Value {
    let operator = triage_operator_name(route, trajectory_type);
    let output_kind = triage_operator_output_kind(route, trajectory_type);
    let stage = triage_operator_stage(step);
    let kind_for_missing = if output_kind == "witness" {
        "witness"
    } else {
        "data"
    };
    let missing_fields = triage_missing_fields(kind_for_missing, step);

    let checklist = if stage == "triage_final" {
        vec![json!({
            "field": "core_context",
            "filled": true,
            "required_for_final": true
        })]
    } else if missing_fields.is_empty() {
        vec![json!({
            "field": "core_context",
            "filled": false,
            "required_for_final": true
        })]
    } else {
        missing_fields
            .iter()
            .map(|field| {
                json!({
                    "field": field,
                    "filled": false,
                    "required_for_final": true
                })
            })
            .collect()
    };

    let questions = if stage == "triage_draft" {
        missing_fields
            .iter()
            .map(|field| triage_missing_field_prompt(field).to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut routing = json!({
        "route": route,
        "track_hint": "tuntaskan",
        "seed_hint": if output_kind == "witness" { "Keresahan" } else { "Kejadian" }
    });
    if let Some(trajectory_type) = trajectory_type {
        routing["trajectory_type"] = json!(trajectory_type);
    }
    if output_kind == "data" {
        routing["taxonomy"] = triage_taxonomy_payload(route, trajectory_type, "data");
    }
    if output_kind == "witness" {
        routing["program_refs"] = triage_program_refs_payload(trajectory_type);
    }
    if let Some(stempel_state) = triage_initial_stempel_payload(trajectory_type) {
        routing["stempel_state"] = stempel_state;
    }

    json!({
        "schema_version": OPERATOR_SCHEMA_VERSION,
        "operator": operator,
        "triage_stage": stage,
        "output_kind": output_kind,
        "confidence": triage_confidence_score(step),
        "checklist": checklist,
        "questions": questions,
        "missing_fields": if stage == "triage_draft" {
            json!(missing_fields)
        } else {
            json!([])
        },
        "routing": routing,
        "blocks": triage_operator_blocks_payload(operator),
        "payload": triage_operator_payload(operator, route, trajectory_type, step)
    })
}

fn triage_non_empty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn triage_payload_has_non_empty_string(payload: &Value, key: &str) -> bool {
    payload
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn triage_payload_has_array_with_min(payload: &Value, key: &str, min: usize) -> bool {
    payload
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_array)
        .is_some_and(|items| items.len() >= min)
}

fn triage_validate_operator_final_payload(contract: &TriageOperatorOutput) -> Result<(), String> {
    let payload = &contract.payload;
    if !payload.is_object() {
        return Err("payload must be an object".to_string());
    }

    match contract.operator.as_str() {
        "masalah" => {
            let trajectory = payload
                .get("trajectory")
                .and_then(Value::as_str)
                .ok_or_else(|| "masalah payload requires trajectory".to_string())?;
            if !matches!(trajectory, "A" | "B") {
                return Err("masalah payload trajectory must be A|B".to_string());
            }
            if !payload
                .as_object()
                .and_then(|object| object.get("path_plan"))
                .is_some_and(Value::is_object)
            {
                return Err("masalah payload requires path_plan object".to_string());
            }
        }
        "musyawarah" => {
            let context = payload
                .get("context")
                .and_then(Value::as_str)
                .ok_or_else(|| "musyawarah payload requires context".to_string())?;
            if !matches!(context, "proposal" | "dispute") {
                return Err("musyawarah payload context must be proposal|dispute".to_string());
            }
            if !triage_payload_has_array_with_min(payload, "decision_steps", 1) {
                return Err("musyawarah payload requires decision_steps[]".to_string());
            }
        }
        "pantau" => {
            if !triage_payload_has_non_empty_string(payload, "case_type")
                || !triage_payload_has_array_with_min(payload, "timeline_seed", 1)
                || !triage_payload_has_array_with_min(payload, "tracking_points", 1)
            {
                return Err("pantau payload missing required fields".to_string());
            }
        }
        "catat" => {
            let record_type = payload
                .get("record_type")
                .and_then(Value::as_str)
                .ok_or_else(|| "catat payload requires record_type".to_string())?;
            if !matches!(record_type, "data" | "vault") {
                return Err("catat payload record_type must be data|vault".to_string());
            }
            if !triage_payload_has_non_empty_string(payload, "claim")
                || !triage_payload_has_non_empty_string(payload, "observed_at")
                || !triage_payload_has_non_empty_string(payload, "category")
            {
                return Err("catat payload missing required fields".to_string());
            }
        }
        "bantuan" => {
            let urgency = payload
                .get("urgency")
                .and_then(Value::as_str)
                .ok_or_else(|| "bantuan payload requires urgency".to_string())?;
            if !matches!(urgency, "rendah" | "sedang" | "tinggi") {
                return Err("bantuan payload urgency must be rendah|sedang|tinggi".to_string());
            }
            if !triage_payload_has_non_empty_string(payload, "help_type")
                || !triage_payload_has_non_empty_string(payload, "description")
                || payload
                    .as_object()
                    .and_then(|object| object.get("matched_resources"))
                    .and_then(Value::as_array)
                    .is_none()
            {
                return Err("bantuan payload missing required fields".to_string());
            }
        }
        "rayakan" => {
            if !triage_payload_has_non_empty_string(payload, "achievement")
                || !triage_payload_has_array_with_min(payload, "contributors", 1)
                || !triage_payload_has_non_empty_string(payload, "impact_summary")
            {
                return Err("rayakan payload missing required fields".to_string());
            }
        }
        "siaga" => {
            let severity = payload
                .get("severity")
                .and_then(Value::as_str)
                .ok_or_else(|| "siaga payload requires severity".to_string())?;
            if !matches!(severity, "waspada" | "siaga" | "darurat") {
                return Err("siaga payload severity must be waspada|siaga|darurat".to_string());
            }
            if !triage_payload_has_non_empty_string(payload, "threat_type")
                || !triage_payload_has_non_empty_string(payload, "location")
                || !triage_payload_has_non_empty_string(payload, "description")
                || !triage_payload_has_non_empty_string(payload, "source")
                || !triage_payload_has_non_empty_string(payload, "expires_at")
            {
                return Err("siaga payload missing required fields".to_string());
            }
        }
        "program" => {
            let frequency = payload
                .get("frequency")
                .and_then(Value::as_str)
                .ok_or_else(|| "program payload requires frequency".to_string())?;
            if !matches!(frequency, "harian" | "mingguan" | "bulanan" | "custom") {
                return Err(
                    "program payload frequency must be harian|mingguan|bulanan|custom".to_string(),
                );
            }
            if !triage_payload_has_non_empty_string(payload, "activity_name")
                || !triage_payload_has_array_with_min(payload, "rotation", 1)
            {
                return Err("program payload missing required fields".to_string());
            }
        }
        "kelola" => {
            let action = payload
                .get("action")
                .and_then(Value::as_str)
                .ok_or_else(|| "kelola payload requires action".to_string())?;
            if !matches!(action, "create" | "edit" | "invite" | "join" | "leave") {
                return Err(
                    "kelola payload action must be create|edit|invite|join|leave".to_string(),
                );
            }
            if matches!(action, "create" | "edit")
                && !payload
                    .as_object()
                    .and_then(|object| object.get("group_detail"))
                    .is_some_and(Value::is_object)
            {
                return Err("kelola payload requires group_detail for create|edit".to_string());
            }
            if matches!(action, "edit" | "invite" | "join" | "leave")
                && !triage_payload_has_non_empty_string(payload, "group_id")
            {
                return Err(
                    "kelola payload requires group_id for edit|invite|join|leave".to_string(),
                );
            }
            if action == "invite"
                && !triage_payload_has_array_with_min(payload, "invited_user_ids", 1)
            {
                return Err("kelola payload requires invited_user_ids for invite".to_string());
            }
        }
        _ => {
            return Err("unsupported operator".to_string());
        }
    }

    Ok(())
}

fn triage_validate_operator_blocks(contract: &TriageOperatorOutput) -> Result<(), String> {
    let blocks = contract
        .blocks
        .as_ref()
        .ok_or_else(|| "blocks is required".to_string())?;

    if blocks.conversation.is_empty() {
        return Err("blocks.conversation must contain at least one item".to_string());
    }
    if blocks.structured.is_empty() {
        return Err("blocks.structured must contain at least one item".to_string());
    }

    let allowed_conversation = HashSet::from([
        "chat_message",
        "ai_inline_card",
        "diff_card",
        "vote_card",
        "moderation_hold_card",
        "duplicate_detection_card",
        "credit_nudge_card",
    ]);
    let allowed_structured = HashSet::from([
        "list",
        "document",
        "form",
        "computed",
        "display",
        "vote",
        "reference",
    ]);

    let mut seen_conversation = HashSet::new();
    for item in &blocks.conversation {
        if !allowed_conversation.contains(item.as_str()) {
            return Err("blocks.conversation contains invalid block id".to_string());
        }
        if !seen_conversation.insert(item) {
            return Err("blocks.conversation contains duplicate block id".to_string());
        }
    }

    let mut seen_structured = HashSet::new();
    for item in &blocks.structured {
        if !allowed_structured.contains(item.as_str()) {
            return Err("blocks.structured contains invalid block id".to_string());
        }
        if !seen_structured.insert(item) {
            return Err("blocks.structured contains duplicate block id".to_string());
        }
    }

    if blocks.conversation.iter().any(|item| item == "vote_card")
        && !matches!(contract.operator.as_str(), "musyawarah" | "siaga")
    {
        return Err("blocks.vote_card is only allowed for musyawarah|siaga".to_string());
    }

    Ok(())
}

fn triage_validate_operator_output(contract: &TriageOperatorOutput) -> Result<(), String> {
    if contract.schema_version != OPERATOR_SCHEMA_VERSION {
        return Err(format!(
            "operator schema_version must be '{OPERATOR_SCHEMA_VERSION}'"
        ));
    }

    if !matches!(
        contract.triage_stage.as_str(),
        "triage_draft" | "triage_final"
    ) {
        return Err("triage_stage must be triage_draft|triage_final".to_string());
    }

    if !matches!(contract.output_kind.as_str(), "witness" | "data" | "kelola") {
        return Err("output_kind must be witness|data|kelola".to_string());
    }

    if let Some(confidence) = contract.confidence
        && !(0.0..=1.0).contains(&confidence)
    {
        return Err("confidence must be between 0 and 1".to_string());
    }

    if contract.checklist.is_empty() {
        return Err("checklist must contain at least one item".to_string());
    }
    for item in &contract.checklist {
        if item.field.trim().is_empty() {
            return Err("checklist field cannot be empty".to_string());
        }
    }

    let expected_kind = match contract.operator.as_str() {
        "masalah" | "musyawarah" | "pantau" | "program" => "witness",
        "catat" | "bantuan" | "rayakan" | "siaga" => "data",
        "kelola" => "kelola",
        _ => return Err("operator is not recognized".to_string()),
    };
    if contract.output_kind != expected_kind {
        return Err("operator/output_kind combination is invalid".to_string());
    }

    if !matches!(
        contract.routing.route.as_str(),
        "komunitas" | "vault" | "siaga" | "catatan_komunitas" | "kelola"
    ) {
        return Err("routing.route is invalid".to_string());
    }

    if let Some(seed_hint) = contract.routing.seed_hint.as_deref()
        && !matches!(
            seed_hint,
            "Keresahan" | "Aspirasi" | "Kejadian" | "Rencana" | "Pertanyaan"
        )
    {
        return Err("routing.seed_hint is invalid".to_string());
    }

    if let Some(taxonomy) = &contract.routing.taxonomy {
        if taxonomy.category_label.trim().is_empty() {
            return Err("routing.taxonomy.category_label cannot be empty".to_string());
        }
        if !matches!(
            taxonomy.category_code.as_str(),
            "commodity_price"
                | "public_service"
                | "training"
                | "employment"
                | "health"
                | "education"
                | "infrastructure"
                | "safety_alert"
                | "environment"
                | "community_event"
                | "other_custom"
        ) {
            return Err("routing.taxonomy.category_code is invalid".to_string());
        }
        if !matches!(
            taxonomy.quality.as_str(),
            "official_source" | "community_observation" | "unverified_claim"
        ) {
            return Err("routing.taxonomy.quality is invalid".to_string());
        }
    }

    if let Some(program_refs) = &contract.routing.program_refs {
        for item in program_refs {
            if item.program_id.trim().is_empty()
                || item.label.trim().is_empty()
                || item.source.trim().is_empty()
            {
                return Err("routing.program_refs contains empty required fields".to_string());
            }
            if !(0.0..=1.0).contains(&item.confidence) {
                return Err("routing.program_refs confidence must be between 0 and 1".to_string());
            }
        }
    }

    if let Some(stempel_state) = &contract.routing.stempel_state
        && !matches!(
            stempel_state.state.as_str(),
            "draft" | "proposed" | "objection_window" | "locked"
        )
    {
        return Err("routing.stempel_state.state is invalid".to_string());
    }

    if contract.triage_stage == "triage_draft" {
        if contract.blocks.is_some() {
            triage_validate_operator_blocks(contract)?;
        }
        if contract
            .questions
            .as_ref()
            .is_none_or(|items| items.is_empty())
        {
            return Err("triage_draft requires questions[]".to_string());
        }
        if contract
            .missing_fields
            .as_ref()
            .is_none_or(|items| items.is_empty())
        {
            return Err("triage_draft requires missing_fields[]".to_string());
        }
    } else {
        triage_validate_operator_blocks(contract)?;
        if contract
            .questions
            .as_ref()
            .is_some_and(|items| !items.is_empty())
        {
            return Err("triage_final requires empty questions[]".to_string());
        }
        if contract
            .missing_fields
            .as_ref()
            .is_some_and(|items| !items.is_empty())
        {
            return Err("triage_final requires empty missing_fields[]".to_string());
        }
        if contract
            .checklist
            .iter()
            .any(|item| item.required_for_final && !item.filled)
        {
            return Err("triage_final checklist has unfilled required fields".to_string());
        }
        triage_validate_operator_final_payload(contract)?;
    }

    match contract.output_kind.as_str() {
        "witness" => {
            let trajectory_type = contract
                .routing
                .trajectory_type
                .as_deref()
                .ok_or_else(|| "witness output requires routing.trajectory_type".to_string())?;
            if !matches!(
                trajectory_type,
                "aksi" | "advokasi" | "pantau" | "mufakat" | "mediasi" | "program"
            ) {
                return Err("witness output uses invalid trajectory_type".to_string());
            }
        }
        "data" => {
            let trajectory_type = contract
                .routing
                .trajectory_type
                .as_deref()
                .ok_or_else(|| "data output requires routing.trajectory_type".to_string())?;
            if !matches!(
                trajectory_type,
                "data" | "vault" | "bantuan" | "pencapaian" | "siaga"
            ) {
                return Err("data output uses invalid trajectory_type".to_string());
            }
            if contract.routing.taxonomy.is_none() {
                return Err("data output requires routing.taxonomy".to_string());
            }
        }
        "kelola" => {
            if contract.routing.route != "kelola" {
                return Err("kelola output requires routing.route=kelola".to_string());
            }
        }
        _ => {}
    }

    Ok(())
}

fn triage_result_from_operator_contract(contract: &TriageOperatorOutput, step: u8) -> Value {
    let route = contract.routing.route.as_str();
    let trajectory_type = contract.routing.trajectory_type.as_deref();
    let kind = match contract.output_kind.as_str() {
        "witness" => "witness",
        _ => "data",
    };
    let status = if contract.triage_stage == "triage_final" {
        "final"
    } else {
        "draft"
    };
    let bar_state = if status == "final" {
        triage_terminal_bar_state(route)
    } else {
        triage_bar_state(route, step)
    };
    let score = contract
        .confidence
        .unwrap_or_else(|| triage_confidence_score(step));
    let taxonomy = contract
        .routing
        .taxonomy
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok())
        .unwrap_or_else(|| triage_taxonomy_payload(route, trajectory_type, kind));
    let program_refs = contract
        .routing
        .program_refs
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok())
        .unwrap_or_else(|| triage_program_refs_payload(trajectory_type));

    let mut payload = json!({
        "schema_version": TRIAGE_SCHEMA_VERSION,
        "status": status,
        "kind": kind,
        "missing_fields": contract.missing_fields.clone().unwrap_or_default(),
        "taxonomy": taxonomy,
        "program_refs": program_refs,
        "bar_state": bar_state,
        "route": route,
        "track_hint": contract.routing.track_hint.clone().unwrap_or_else(|| "tuntaskan".to_string()),
        "seed_hint": contract.routing.seed_hint.clone().unwrap_or_else(|| {
            if kind == "witness" {
                "Keresahan".to_string()
            } else {
                "Kejadian".to_string()
            }
        }),
        "summary_text": triage_summary_text(route, kind),
        "card": triage_card_payload(route, trajectory_type, kind),
        "confidence": {
            "score": score,
            "label": format!("{} · {}%", triage_label(route), (score * 100.0).round() as i64)
        },
        "budget": triage_budget(step),
    });
    if let Some(trajectory_type) = trajectory_type {
        payload["trajectory_type"] = json!(trajectory_type);
    }

    let stempel_state = contract
        .routing
        .stempel_state
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok())
        .or_else(|| triage_initial_stempel_payload(trajectory_type));
    if let Some(stempel_state) = stempel_state {
        payload["stempel_state"] = stempel_state;
    }

    if route == "kelola" && status == "final" {
        payload["kelola_result"] = contract.payload.clone();
    }
    if status == "final"
        && contract.operator == "masalah"
        && triage_non_empty_string(contract.payload.get("trajectory"))
        && contract
            .payload
            .get("path_plan")
            .is_some_and(Value::is_object)
    {
        payload["proposed_plan"] = contract.payload["path_plan"].clone();
    }

    payload
}

fn triage_result_from_operator_output(operator_output: Value, step: u8) -> Result<Value, ApiError> {
    let contract: TriageOperatorOutput =
        serde_json::from_value(operator_output).map_err(|error| {
            tracing::error!(error = %error, "failed to parse operator.v1 output");
            ApiError::Internal
        })?;
    triage_validate_operator_output(&contract).map_err(|error| {
        tracing::error!(error = %error, "operator.v1 output failed validation");
        ApiError::Internal
    })?;
    Ok(triage_result_from_operator_contract(&contract, step))
}

fn triage_route_from_result(result: &Value) -> String {
    result
        .get("route")
        .and_then(Value::as_str)
        .unwrap_or("komunitas")
        .to_string()
}

fn triage_trajectory_type_from_result(result: &Value) -> Option<String> {
    result
        .get("trajectory_type")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn triage_card_payload(route: &str, trajectory_type: Option<&str>, kind: &str) -> Value {
    let trajectory = trajectory_type.unwrap_or(if kind == "witness" { "aksi" } else { "data" });
    let icon = match trajectory {
        "aksi" => "construction",
        "pantau" => "eye",
        "mufakat" => "users",
        "program" => "calendar",
        "siaga" => "siren",
        "vault" => "lock",
        "pencapaian" => "trophy",
        "bantuan" => "heart",
        "data" => "file-text",
        _ => "sparkles",
    };
    let title = match route {
        "vault" => "Catatan Rahasia Warga",
        "siaga" => "Peringatan Siaga Komunitas",
        "catatan_komunitas" => "Catatan Fakta Komunitas",
        _ if kind == "witness" => "Laporan Saksi Komunitas",
        _ => "Data Komunitas",
    };
    let hook_line = if kind == "witness" {
        "Konteks awal terkumpul, siap diproses jadi saksi komunitas."
    } else {
        "Catatan siap dipublikasikan sebagai data satu kali."
    };

    json!({
        "icon": icon,
        "trajectory_type": trajectory,
        "title": title,
        "hook_line": hook_line,
        "body": format!("{title}. {hook_line}"),
        "sentiment": if route == "siaga" { "urgent" } else { "curious" },
        "intensity": if route == "siaga" { 5 } else { 2 },
    })
}

fn triage_summary_text(route: &str, kind: &str) -> String {
    if kind == "witness" {
        return format!("{} siap dibuat sebagai saksi.", triage_label(route));
    }
    format!("{} siap dicatat sebagai data.", triage_label(route))
}

fn triage_result_payload(
    route: &str,
    trajectory_type: Option<&str>,
    step: u8,
    operator_output: Option<Value>,
) -> Result<Value, ApiError> {
    let operator_output = operator_output
        .unwrap_or_else(|| triage_operator_output_payload(route, trajectory_type, step));
    triage_result_from_operator_output(operator_output, step)
}

fn triage_ai_message(result: &Value) -> String {
    let status = result
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft");
    if status == "final" {
        return "Siap, detail sudah cukup untuk diproses.".to_string();
    }

    let first_missing = result
        .get("missing_fields")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_str);
    match first_missing {
        Some(field) => triage_missing_field_prompt(field).to_string(),
        None => "Boleh tambah satu detail penting lagi agar saya bisa menutup triase.".to_string(),
    }
}

#[cfg(test)]
mod triage_operator_contract_tests {
    use super::*;

    #[test]
    fn operator_output_maps_to_runtime_triage_payload() {
        let operator_output = triage_operator_output_payload("komunitas", Some("aksi"), 3);
        let result = triage_result_from_operator_output(operator_output, 3).expect("triage result");

        assert_eq!(result.get("schema_version"), Some(&json!("triage.v1")));
        assert_eq!(result.get("status"), Some(&json!("final")));
        assert_eq!(result.get("kind"), Some(&json!("witness")));
        assert_eq!(result.get("bar_state"), Some(&json!("ready")));
        assert_eq!(result.get("route"), Some(&json!("komunitas")));
        assert!(result.get("proposed_plan").is_some_and(Value::is_object));
    }

    #[test]
    fn operator_contract_rejects_invalid_final_payload_shape() {
        let invalid = json!({
            "schema_version": "operator.v1",
            "operator": "musyawarah",
            "triage_stage": "triage_final",
            "output_kind": "witness",
            "confidence": 0.92,
            "checklist": [
                { "field": "stakeholders", "filled": true, "required_for_final": true }
            ],
            "questions": [],
            "missing_fields": [],
            "routing": {
                "route": "komunitas",
                "trajectory_type": "mufakat"
            },
            "blocks": {
                "conversation": ["ai_inline_card", "diff_card", "vote_card"],
                "structured": ["document", "list", "vote", "computed"]
            },
            "payload": {
                "context": "proposal",
                "decision_steps": []
            }
        });

        let result = triage_result_from_operator_output(invalid, 3);
        assert!(matches!(result, Err(ApiError::Internal)));
    }

    #[test]
    fn operator_contract_rejects_data_output_without_taxonomy() {
        let invalid = json!({
            "schema_version": "operator.v1",
            "operator": "catat",
            "triage_stage": "triage_final",
            "output_kind": "data",
            "confidence": 0.85,
            "checklist": [
                { "field": "claim", "filled": true, "required_for_final": true }
            ],
            "questions": [],
            "missing_fields": [],
            "routing": {
                "route": "catatan_komunitas",
                "trajectory_type": "data"
            },
            "blocks": {
                "conversation": ["ai_inline_card", "diff_card"],
                "structured": ["form", "document", "reference"]
            },
            "payload": {
                "record_type": "data",
                "claim": "Harga komoditas naik",
                "observed_at": "2026-02-26T00:00:00Z",
                "category": "harga_pangan"
            }
        });

        let result = triage_result_from_operator_output(invalid, 3);
        assert!(matches!(result, Err(ApiError::Internal)));
    }

    #[test]
    fn operator_contract_rejects_final_output_without_blocks() {
        let invalid = json!({
            "schema_version": "operator.v1",
            "operator": "masalah",
            "triage_stage": "triage_final",
            "output_kind": "witness",
            "confidence": 0.89,
            "checklist": [
                { "field": "problem_scope", "filled": true, "required_for_final": true }
            ],
            "questions": [],
            "missing_fields": [],
            "routing": {
                "route": "komunitas",
                "trajectory_type": "aksi"
            },
            "payload": {
                "trajectory": "A",
                "path_plan": {
                    "plan_id": "plan-test",
                    "version": 1,
                    "title": "Rencana",
                    "summary": "Ringkas",
                    "branches": [
                        {
                            "branch_id": "main",
                            "label": "Utama",
                            "parent_checkpoint_id": null,
                            "phases": [
                                {
                                    "phase_id": "p1",
                                    "title": "Validasi",
                                    "objective": "Sepakati lingkup",
                                    "status": "planned",
                                    "source": "ai",
                                    "locked_fields": [],
                                    "checkpoints": [
                                        {
                                            "checkpoint_id": "c1",
                                            "title": "Konteks dikunci",
                                            "status": "open",
                                            "source": "ai",
                                            "locked_fields": []
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        });

        let result = triage_result_from_operator_output(invalid, 3);
        assert!(matches!(result, Err(ApiError::Internal)));
    }
}

fn compact_triage_sessions(sessions: &mut HashMap<String, TriageSessionState>, now_ms: i64) {
    sessions
        .retain(|_, session| now_ms.saturating_sub(session.updated_at_ms) <= TRIAGE_SESSION_TTL_MS);

    if sessions.len() <= TRIAGE_SESSION_MAX_ITEMS {
        return;
    }

    let remove_count = sessions.len() - TRIAGE_SESSION_MAX_ITEMS;
    let mut ordered = sessions
        .iter()
        .map(|(session_id, session)| (session_id.clone(), session.updated_at_ms))
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(_, updated_at_ms)| *updated_at_ms);
    for (session_id, _) in ordered.into_iter().take(remove_count) {
        sessions.remove(&session_id);
    }
}

async fn start_triage_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<StartTriageSessionRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "triage_session_start",
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
            let detected = detect_triage_route(&payload.content);
            let now_ms = gotong_domain::jobs::now_ms();
            let session_id = format!("triage-sess-{request_id}");
            let result = triage_result_payload(
                detected.route,
                detected.trajectory_type,
                1,
                payload.operator_output.clone(),
            )?;
            let result_route = triage_route_from_result(&result);
            let result_trajectory_type = triage_trajectory_type_from_result(&result);
            let ai_message = triage_ai_message(&result);

            {
                let mut sessions = state.triage_sessions.write().await;
                compact_triage_sessions(&mut sessions, now_ms);
                sessions.insert(
                    session_id.clone(),
                    TriageSessionState {
                        owner_user_id: actor.user_id.clone(),
                        route: result_route,
                        trajectory_type: result_trajectory_type,
                        step: 1,
                        latest_result: result.clone(),
                        messages: vec![
                            TriageSessionMessageState {
                                role: "user".to_string(),
                                text: payload.content.clone(),
                                created_at_ms: now_ms,
                            },
                            TriageSessionMessageState {
                                role: "ai".to_string(),
                                text: ai_message.clone(),
                                created_at_ms: now_ms,
                            },
                        ],
                        created_at_ms: now_ms,
                        updated_at_ms: now_ms,
                    },
                );
            }

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({
                    "session_id": session_id,
                    "result": result,
                    "ai_message": ai_message
                }),
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

async fn continue_triage_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ContinueTriageSessionRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "triage_session_continue",
        format!("{}:{session_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let (result, ai_message) = {
                let mut sessions = state.triage_sessions.write().await;
                compact_triage_sessions(&mut sessions, now_ms);
                let session = sessions.get_mut(&session_id).ok_or(ApiError::NotFound)?;
                if session.owner_user_id != actor.user_id {
                    return Err(ApiError::Forbidden);
                }
                let _session_age_ms = now_ms.saturating_sub(session.created_at_ms);
                let _answer = payload.answer.trim();
                let _attachment_count = payload.attachments.as_ref().map_or(0, |items| items.len());
                let _attachment_name_total =
                    payload.attachments.as_ref().map_or(0_usize, |items| {
                        items
                            .iter()
                            .map(|item| item.name.len() + item.mime_type.len())
                            .sum()
                    });
                let _total_attachment_size_bytes =
                    payload.attachments.as_ref().map_or(0_u64, |items| {
                        items.iter().filter_map(|item| item.size_bytes).sum()
                    });
                session.step = session.step.saturating_add(1).min(3);
                session.updated_at_ms = now_ms;
                session.messages.push(TriageSessionMessageState {
                    role: "user".to_string(),
                    text: payload.answer.clone(),
                    created_at_ms: now_ms,
                });
                let result = triage_result_payload(
                    &session.route,
                    session.trajectory_type.as_deref(),
                    session.step,
                    payload.operator_output.clone(),
                )?;
                session.route = triage_route_from_result(&result);
                session.trajectory_type = triage_trajectory_type_from_result(&result);
                session.latest_result = result.clone();
                let ai_message = triage_ai_message(&result);
                session.messages.push(TriageSessionMessageState {
                    role: "ai".to_string(),
                    text: ai_message.clone(),
                    created_at_ms: now_ms,
                });
                (result, ai_message)
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({
                    "result": result,
                    "ai_message": ai_message
                }),
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

const SIGNAL_REGISTRY_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const SIGNAL_REGISTRY_MAX_ITEMS: usize = 50_000;

#[derive(Debug, Deserialize, Validate)]
struct CreateWitnessSignalRequest {
    #[validate(length(min = 1, max = 32))]
    signal_type: String,
}

#[derive(Debug, Deserialize, Validate)]
struct ProposeWitnessStempelRequest {
    #[validate(length(max = 512))]
    summary: Option<String>,
    #[validate(length(max = 2_000))]
    rationale: Option<String>,
    objection_window_seconds: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
struct SubmitWitnessStempelObjectionRequest {
    #[validate(length(min = 1, max = 1_000))]
    reason: String,
}

#[derive(Debug, Serialize, Clone)]
struct WitnessSignalDto {
    signal_id: String,
    witness_id: String,
    user_id: String,
    signal_type: String,
    outcome: String,
    created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credit_delta: Option<f64>,
}

#[derive(Debug, Serialize)]
struct WitnessSignalRelationDto {
    vouched: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    vouch_type: Option<String>,
    witnessed: bool,
    flagged: bool,
    supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    vote_cast: Option<String>,
}

#[derive(Debug, Serialize)]
struct WitnessSignalCountsDto {
    vouch_positive: usize,
    vouch_skeptical: usize,
    witness_count: usize,
    dukung_count: usize,
    flags: usize,
}

#[derive(Debug, Serialize)]
struct WitnessStempelStateDto {
    state: String,
    proposed_at_ms: Option<i64>,
    objection_deadline_ms: Option<i64>,
    locked_at_ms: Option<i64>,
    min_participants: usize,
    participant_count: usize,
    objection_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_objection_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_objection_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct WitnessImpactVerificationDto {
    status: String,
    opened_at_ms: Option<i64>,
    closes_at_ms: Option<i64>,
    yes_count: usize,
    no_count: usize,
    min_vouches: usize,
}

#[derive(Debug, Serialize)]
struct WitnessStempelEnvelopeDto {
    witness_id: String,
    stempel_state: WitnessStempelStateDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    impact_verification: Option<WitnessImpactVerificationDto>,
}

fn normalize_content_signal_type(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "saksi" => Some("saksi"),
        "perlu_dicek" => Some("perlu_dicek"),
        _ => None,
    }
}

fn witness_signal_active_key(user_id: &str, signal_type: &str) -> String {
    format!("{user_id}:{signal_type}")
}

fn compact_witness_signal_registry(
    registry: &mut HashMap<String, WitnessSignalState>,
    now_ms: i64,
) {
    registry
        .retain(|_, state| now_ms.saturating_sub(state.updated_at_ms) <= SIGNAL_REGISTRY_TTL_MS);

    if registry.len() <= SIGNAL_REGISTRY_MAX_ITEMS {
        return;
    }

    let remove_count = registry.len() - SIGNAL_REGISTRY_MAX_ITEMS;
    let mut ordered = registry
        .iter()
        .map(|(witness_id, state)| (witness_id.clone(), state.updated_at_ms))
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(_, updated_at_ms)| *updated_at_ms);
    for (witness_id, _) in ordered.into_iter().take(remove_count) {
        registry.remove(&witness_id);
    }
}

fn to_signal_dto(signal: &WitnessSignalEntry) -> WitnessSignalDto {
    WitnessSignalDto {
        signal_id: signal.signal_id.clone(),
        witness_id: signal.witness_id.clone(),
        user_id: signal.user_id.clone(),
        signal_type: signal.signal_type.clone(),
        outcome: signal.outcome.clone(),
        created_at: signal.created_at_ms,
        resolved_at: signal.resolved_at_ms,
        credit_delta: signal.credit_delta,
    }
}

fn to_stempel_state_dto(state: &WitnessStempelState) -> WitnessStempelStateDto {
    let latest_objection = state.objections.last();
    WitnessStempelStateDto {
        state: state.state.clone(),
        proposed_at_ms: state.proposed_at_ms,
        objection_deadline_ms: state.objection_deadline_ms,
        locked_at_ms: state.locked_at_ms,
        min_participants: state.min_participants,
        participant_count: state.participant_count,
        objection_count: state.objections.len(),
        latest_objection_at_ms: latest_objection.map(|item| item.created_at_ms),
        latest_objection_reason: latest_objection.map(|item| item.reason.clone()),
    }
}

fn to_impact_verification_dto(
    state: &WitnessImpactVerificationState,
) -> WitnessImpactVerificationDto {
    WitnessImpactVerificationDto {
        status: state.status.clone(),
        opened_at_ms: state.opened_at_ms,
        closes_at_ms: state.closes_at_ms,
        yes_count: state.yes_count,
        no_count: state.no_count,
        min_vouches: state.min_vouches,
    }
}

fn impact_verification_not_open_json() -> Value {
    json!({
        "status": "not_open",
        "opened_at_ms": Value::Null,
        "closes_at_ms": Value::Null,
        "yes_count": 0,
        "no_count": 0,
        "min_vouches": IMPACT_VOUCH_MIN_VOUCHES
    })
}

fn stempel_state_from_value(raw: &Value) -> Option<WitnessStempelState> {
    let obj = raw.as_object()?;
    let state = obj
        .get("state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("draft");
    let min_participants = obj
        .get("min_participants")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(STEMPEL_MIN_PARTICIPANTS);
    let participant_count = obj
        .get("participant_count")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    Some(WitnessStempelState {
        state: state.to_string(),
        proposed_at_ms: obj.get("proposed_at_ms").and_then(Value::as_i64),
        objection_deadline_ms: obj.get("objection_deadline_ms").and_then(Value::as_i64),
        locked_at_ms: obj.get("locked_at_ms").and_then(Value::as_i64),
        min_participants,
        participant_count,
        objections: Vec::new(),
    })
}

async fn witness_participant_count(state: &AppState, witness_id: &str) -> usize {
    let registry = state.witness_signals.read().await;
    let Some(signal_state) = registry.get(witness_id) else {
        return 1;
    };
    let mut participants = HashSet::new();
    for signal in signal_state.active_signals.values() {
        participants.insert(signal.user_id.clone());
    }
    for signal in &signal_state.resolved_signals {
        participants.insert(signal.user_id.clone());
    }
    participants.len().max(1)
}

async fn create_witness_signal(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateWitnessSignalRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let signal_type = normalize_content_signal_type(&payload.signal_type).ok_or_else(|| {
        ApiError::Validation("signal_type must be 'saksi' or 'perlu_dicek'".into())
    })?;

    let key = IdempotencyKey::new(
        "witness_signal_create",
        format!("{}:{witness_id}:{signal_type}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let signal = {
                let mut registry = state.witness_signals.write().await;
                compact_witness_signal_registry(&mut registry, now_ms);
                let witness_state = registry.entry(witness_id.clone()).or_default();
                witness_state.updated_at_ms = now_ms;
                let active_key = witness_signal_active_key(&actor.user_id, signal_type);
                if let Some(existing) = witness_state.active_signals.get(&active_key) {
                    existing.clone()
                } else {
                    let new_signal = WitnessSignalEntry {
                        signal_id: format!("sig-{}", gotong_domain::util::uuid_v7_without_dashes()),
                        witness_id: witness_id.clone(),
                        user_id: actor.user_id.clone(),
                        signal_type: signal_type.to_string(),
                        outcome: "pending".to_string(),
                        created_at_ms: now_ms,
                        resolved_at_ms: None,
                        credit_delta: None,
                    };
                    witness_state
                        .active_signals
                        .insert(active_key, new_signal.clone());
                    new_signal
                }
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(to_signal_dto(&signal))
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

async fn remove_witness_signal(
    State(state): State<AppState>,
    Path((witness_id, signal_type)): Path<(String, String)>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let signal_type = normalize_content_signal_type(&signal_type).ok_or_else(|| {
        ApiError::Validation("signal_type must be 'saksi' or 'perlu_dicek'".into())
    })?;

    let key = IdempotencyKey::new(
        "witness_signal_remove",
        format!("{}:{witness_id}:{signal_type}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let removed = {
                let mut registry = state.witness_signals.write().await;
                compact_witness_signal_registry(&mut registry, now_ms);
                if let Some(witness_state) = registry.get_mut(&witness_id) {
                    witness_state.updated_at_ms = now_ms;
                    let active_key = witness_signal_active_key(&actor.user_id, signal_type);
                    witness_state.active_signals.remove(&active_key).is_some()
                } else {
                    false
                }
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "removed": removed }),
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

async fn get_witness_signal_my_relation(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<WitnessSignalRelationDto>, ApiError> {
    let actor = actor_identity(&auth)?;
    let registry = state.witness_signals.read().await;
    let witness_state = registry.get(&witness_id);
    let witnessed = witness_state.is_some_and(|state| {
        state
            .active_signals
            .contains_key(&witness_signal_active_key(&actor.user_id, "saksi"))
    });
    let flagged = witness_state.is_some_and(|state| {
        state
            .active_signals
            .contains_key(&witness_signal_active_key(&actor.user_id, "perlu_dicek"))
    });
    Ok(Json(WitnessSignalRelationDto {
        vouched: false,
        vouch_type: None,
        witnessed,
        flagged,
        supported: false,
        vote_cast: None,
    }))
}

async fn get_witness_signal_counts(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
) -> Result<Json<WitnessSignalCountsDto>, ApiError> {
    let registry = state.witness_signals.read().await;
    let witness_state = registry.get(&witness_id);
    let (witness_count, flags) = if let Some(state) = witness_state {
        let witness_count = state
            .active_signals
            .values()
            .filter(|signal| signal.signal_type == "saksi")
            .count();
        let flags = state
            .active_signals
            .values()
            .filter(|signal| signal.signal_type == "perlu_dicek")
            .count();
        (witness_count, flags)
    } else {
        (0, 0)
    };
    Ok(Json(WitnessSignalCountsDto {
        vouch_positive: 0,
        vouch_skeptical: 0,
        witness_count,
        dukung_count: 0,
        flags,
    }))
}

async fn list_witness_signal_resolutions(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
) -> Result<Json<Vec<WitnessSignalDto>>, ApiError> {
    let registry = state.witness_signals.read().await;
    let witness_state = registry.get(&witness_id);
    let items = witness_state
        .map(|state| {
            state
                .resolved_signals
                .iter()
                .map(to_signal_dto)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(Json(items))
}

async fn propose_witness_stempel(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<ProposeWitnessStempelRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let _summary = payload.summary.as_deref().unwrap_or_default();
    let _rationale = payload.rationale.as_deref().unwrap_or_default();

    let key = IdempotencyKey::new(
        "witness_stempel_propose",
        format!("{}:{witness_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let window_seconds = payload
                .objection_window_seconds
                .unwrap_or((STEMPEL_DEFAULT_WINDOW_MS / 1_000) as u64);
            let window_ms = i64::try_from(window_seconds)
                .unwrap_or(STEMPEL_DEFAULT_WINDOW_MS / 1_000)
                .saturating_mul(1_000)
                .clamp(0, STEMPEL_MAX_WINDOW_MS);
            let participant_count = witness_participant_count(&state, &witness_id).await;

            let stempel_state = {
                let mut registry = state.witness_stempel.write().await;
                let state_entry = registry.entry(witness_id.clone()).or_default();
                if state_entry.state == "locked" {
                    None
                } else {
                    state_entry.state = "objection_window".to_string();
                    state_entry.proposed_at_ms = Some(now_ms);
                    state_entry.objection_deadline_ms = Some(now_ms.saturating_add(window_ms));
                    state_entry.locked_at_ms = None;
                    state_entry.min_participants = STEMPEL_MIN_PARTICIPANTS;
                    state_entry.participant_count = participant_count;
                    state_entry.objections.clear();
                    Some(state_entry.clone())
                }
            };

            let response = if let Some(stempel_state) = stempel_state {
                IdempotencyResponse {
                    status_code: StatusCode::OK.as_u16(),
                    body: serde_json::to_value(WitnessStempelEnvelopeDto {
                        witness_id,
                        stempel_state: to_stempel_state_dto(&stempel_state),
                        impact_verification: None,
                    })
                    .map_err(|_| ApiError::Internal)?,
                }
            } else {
                IdempotencyResponse {
                    status_code: StatusCode::CONFLICT.as_u16(),
                    body: json!({
                        "error": {
                            "code": "stempel_already_locked",
                            "message": "stempel is already locked for this witness",
                        }
                    }),
                }
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

async fn submit_witness_stempel_objection(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<SubmitWitnessStempelObjectionRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "witness_stempel_objection",
        format!("{}:{witness_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let participant_count = witness_participant_count(&state, &witness_id).await;
            let response = {
                let mut registry = state.witness_stempel.write().await;
                let state_entry = registry.entry(witness_id.clone()).or_default();
                state_entry.participant_count = participant_count;

                if state_entry.state != "objection_window" {
                    IdempotencyResponse {
                        status_code: StatusCode::CONFLICT.as_u16(),
                        body: json!({
                            "error": {
                                "code": "stempel_not_open",
                                "message": "stempel objection window is not open",
                            }
                        }),
                    }
                } else if state_entry
                    .objection_deadline_ms
                    .is_some_and(|deadline| now_ms > deadline)
                {
                    IdempotencyResponse {
                        status_code: StatusCode::CONFLICT.as_u16(),
                        body: json!({
                            "error": {
                                "code": "stempel_window_closed",
                                "message": "objection window has already closed",
                            }
                        }),
                    }
                } else {
                    if !state_entry
                        .objections
                        .iter()
                        .any(|item| item.user_id == actor.user_id)
                    {
                        state_entry.objections.push(WitnessStempelObjection {
                            user_id: actor.user_id.clone(),
                            reason: payload.reason.clone(),
                            created_at_ms: now_ms,
                        });
                    }
                    IdempotencyResponse {
                        status_code: StatusCode::CREATED.as_u16(),
                        body: serde_json::to_value(WitnessStempelEnvelopeDto {
                            witness_id,
                            stempel_state: to_stempel_state_dto(state_entry),
                            impact_verification: None,
                        })
                        .map_err(|_| ApiError::Internal)?,
                    }
                }
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

async fn finalize_witness_stempel(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let key = IdempotencyKey::new(
        "witness_stempel_finalize",
        format!("{}:{witness_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let participant_count = witness_participant_count(&state, &witness_id).await;

            let maybe_stempel_state = {
                let mut registry = state.witness_stempel.write().await;
                let state_entry = registry.entry(witness_id.clone()).or_default();
                state_entry.participant_count = participant_count;

                if state_entry.state == "locked" {
                    Some(state_entry.clone())
                } else if state_entry.state != "objection_window" {
                    None
                } else if state_entry
                    .objection_deadline_ms
                    .is_some_and(|deadline| now_ms < deadline)
                {
                    None
                } else if !state_entry.objections.is_empty() {
                    state_entry.state = "proposed".to_string();
                    None
                } else if state_entry.participant_count < state_entry.min_participants {
                    None
                } else {
                    state_entry.state = "locked".to_string();
                    state_entry.locked_at_ms = Some(now_ms);
                    Some(state_entry.clone())
                }
            };

            let response = if let Some(stempel_state) = maybe_stempel_state {
                let impact_state = {
                    let mut impact_registry = state.witness_impact_verifications.write().await;
                    let entry = impact_registry.entry(witness_id.clone()).or_default();
                    if entry.status != "open" {
                        entry.status = "open".to_string();
                        entry.opened_at_ms = Some(now_ms);
                        entry.closes_at_ms =
                            Some(now_ms.saturating_add(IMPACT_VOUCH_DEFAULT_WINDOW_MS));
                        entry.yes_count = 0;
                        entry.no_count = 0;
                        entry.min_vouches = IMPACT_VOUCH_MIN_VOUCHES;
                    }
                    entry.clone()
                };

                IdempotencyResponse {
                    status_code: StatusCode::OK.as_u16(),
                    body: serde_json::to_value(WitnessStempelEnvelopeDto {
                        witness_id,
                        stempel_state: to_stempel_state_dto(&stempel_state),
                        impact_verification: Some(to_impact_verification_dto(&impact_state)),
                    })
                    .map_err(|_| ApiError::Internal)?,
                }
            } else {
                let current_stempel = {
                    let registry = state.witness_stempel.read().await;
                    registry.get(&witness_id).cloned().unwrap_or_default()
                };
                let deadline_open = current_stempel
                    .objection_deadline_ms
                    .is_some_and(|deadline| now_ms < deadline);
                let has_objection = !current_stempel.objections.is_empty();
                let participant_shortfall =
                    current_stempel.participant_count < current_stempel.min_participants;
                let (code, message) = if current_stempel.state != "objection_window" {
                    (
                        "stempel_not_open",
                        "stempel finalize requires objection_window state",
                    )
                } else if deadline_open {
                    (
                        "stempel_objection_window_open",
                        "objection window is still active",
                    )
                } else if has_objection {
                    (
                        "stempel_has_objection",
                        "cannot lock stempel while objections exist",
                    )
                } else if participant_shortfall {
                    (
                        "stempel_min_participants_not_met",
                        "minimum participant threshold not met",
                    )
                } else {
                    ("stempel_finalize_failed", "unable to finalize stempel")
                };

                IdempotencyResponse {
                    status_code: StatusCode::CONFLICT.as_u16(),
                    body: json!({
                        "error": {
                            "code": code,
                            "message": message,
                        },
                        "stempel_state": to_stempel_state_dto(&current_stempel)
                    }),
                }
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

#[derive(Debug, Deserialize)]
struct GroupListQuery {
    cursor: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateGroupRequest {
    #[validate(length(min = 1, max = 128))]
    name: String,
    #[validate(length(min = 1, max = 2_000))]
    description: String,
    #[validate(length(min = 1, max = 32))]
    entity_type: String,
    #[validate(length(min = 1, max = 32))]
    join_policy: String,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateGroupRequest {
    #[validate(length(min = 1, max = 128))]
    name: Option<String>,
    #[validate(length(min = 1, max = 2_000))]
    description: Option<String>,
    #[validate(length(min = 1, max = 32))]
    join_policy: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct RequestGroupJoinRequest {
    #[validate(length(max = 512))]
    message: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct InviteGroupMemberRequest {
    #[validate(length(min = 1, max = 128))]
    user_id: String,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateGroupMemberRoleRequest {
    #[validate(length(min = 1, max = 32))]
    role: String,
}

#[derive(Debug, Serialize)]
struct GroupEntityTagDto {
    entity_id: String,
    entity_type: String,
    label: String,
    followed: bool,
}

#[derive(Debug, Serialize)]
struct GroupSummaryDto {
    group_id: String,
    name: String,
    description: String,
    entity_type: String,
    join_policy: String,
    member_count: usize,
    witness_count: usize,
    entity_tag: GroupEntityTagDto,
}

#[derive(Debug, Serialize)]
struct GroupMemberDto {
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    role: String,
    joined_at: String,
}

#[derive(Debug, Serialize)]
struct GroupJoinRequestDto {
    request_id: String,
    user_id: String,
    name: String,
    avatar_url: Option<String>,
    message: Option<String>,
    status: String,
    requested_at: String,
}

#[derive(Debug, Serialize)]
struct GroupDetailDto {
    #[serde(flatten)]
    summary: GroupSummaryDto,
    members: Vec<GroupMemberDto>,
    pending_requests: Vec<GroupJoinRequestDto>,
    my_role: Option<String>,
    my_membership_status: String,
}

#[derive(Debug, Serialize)]
struct GroupListResponseDto {
    items: Vec<GroupSummaryDto>,
    total: usize,
    cursor: Option<String>,
}

fn normalize_group_entity_type(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "kelompok" => Some("kelompok"),
        "lembaga" => Some("lembaga"),
        _ => None,
    }
}

fn normalize_group_join_policy(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "terbuka" => Some("terbuka"),
        "persetujuan" => Some("persetujuan"),
        "undangan" => Some("undangan"),
        _ => None,
    }
}

fn normalize_group_member_role(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "admin" => Some("admin"),
        "moderator" => Some("moderator"),
        "anggota" => Some("anggota"),
        _ => None,
    }
}

fn normalize_non_empty(value: &str, field: &str) -> Result<String, ApiError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ApiError::Validation(format!("{field} cannot be empty")));
    }
    Ok(normalized.to_string())
}

fn group_member_role<'a>(group: &'a GroupRecord, user_id: &str) -> Option<&'a str> {
    group
        .members
        .iter()
        .find(|member| member.user_id == user_id)
        .map(|member| member.role.as_str())
}

fn group_admin_count(group: &GroupRecord) -> usize {
    group
        .members
        .iter()
        .filter(|member| member.role == "admin")
        .count()
}

fn group_can_manage(role: Option<&str>) -> bool {
    matches!(role, Some("admin" | "moderator"))
}

fn group_is_admin(role: Option<&str>) -> bool {
    matches!(role, Some("admin"))
}

fn to_group_member_dto(member: &GroupMemberRecord) -> GroupMemberDto {
    GroupMemberDto {
        user_id: member.user_id.clone(),
        name: member.name.clone(),
        avatar_url: member.avatar_url.clone(),
        role: member.role.clone(),
        joined_at: gotong_domain::util::format_ms_rfc3339(member.joined_at_ms),
    }
}

fn to_group_join_request_dto(request: &GroupJoinRequestRecord) -> GroupJoinRequestDto {
    GroupJoinRequestDto {
        request_id: request.request_id.clone(),
        user_id: request.user_id.clone(),
        name: request.name.clone(),
        avatar_url: request.avatar_url.clone(),
        message: request.message.clone(),
        status: request.status.clone(),
        requested_at: gotong_domain::util::format_ms_rfc3339(request.requested_at_ms),
    }
}

fn to_group_summary_dto(group: &GroupRecord) -> GroupSummaryDto {
    GroupSummaryDto {
        group_id: group.group_id.clone(),
        name: group.name.clone(),
        description: group.description.clone(),
        entity_type: group.entity_type.clone(),
        join_policy: group.join_policy.clone(),
        member_count: group.member_count,
        witness_count: group.witness_count,
        entity_tag: GroupEntityTagDto {
            entity_id: group.group_id.clone(),
            entity_type: group.entity_type.clone(),
            label: group.name.clone(),
            followed: false,
        },
    }
}

fn to_group_detail_dto(group: &GroupRecord, user_id: &str) -> GroupDetailDto {
    let my_member = group
        .members
        .iter()
        .find(|member| member.user_id == user_id);
    let pending = group
        .pending_requests
        .iter()
        .any(|request| request.user_id == user_id && request.status == "pending");
    GroupDetailDto {
        summary: to_group_summary_dto(group),
        members: group
            .members
            .iter()
            .map(to_group_member_dto)
            .collect::<Vec<_>>(),
        pending_requests: group
            .pending_requests
            .iter()
            .filter(|request| request.status == "pending")
            .map(to_group_join_request_dto)
            .collect::<Vec<_>>(),
        my_role: my_member.map(|member| member.role.clone()),
        my_membership_status: if my_member.is_some() {
            "approved".to_string()
        } else if pending {
            "pending".to_string()
        } else {
            "none".to_string()
        },
    }
}

fn sort_groups_desc(groups: &mut [GroupRecord]) {
    groups.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| left.group_id.cmp(&right.group_id))
    });
}

fn ordered_groups(groups: Vec<GroupRecord>) -> Vec<GroupRecord> {
    let mut items = groups;
    sort_groups_desc(&mut items);
    items
}

async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let entity_type = normalize_group_entity_type(&payload.entity_type).ok_or_else(|| {
        ApiError::Validation("entity_type must be 'kelompok' or 'lembaga'".into())
    })?;
    let join_policy = normalize_group_join_policy(&payload.join_policy).ok_or_else(|| {
        ApiError::Validation("join_policy must be 'terbuka', 'persetujuan', or 'undangan'".into())
    })?;
    let name = normalize_non_empty(&payload.name, "name")?;
    let description = normalize_non_empty(&payload.description, "description")?;

    let key = IdempotencyKey::new("group_create", actor.user_id.clone(), request_id);
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let now_ms = gotong_domain::jobs::now_ms();
            let group = GroupRecord {
                group_id: format!("ent-{}", gotong_domain::util::uuid_v7_without_dashes()),
                name: name.clone(),
                description: description.clone(),
                entity_type: entity_type.to_string(),
                join_policy: join_policy.to_string(),
                member_count: 1,
                witness_count: 0,
                members: vec![GroupMemberRecord {
                    user_id: actor.user_id.clone(),
                    name: actor.username.clone(),
                    avatar_url: None,
                    role: "admin".to_string(),
                    joined_at_ms: now_ms,
                }],
                pending_requests: Vec::new(),
                updated_at_ms: now_ms,
            };
            let persisted = state
                .group_repo
                .create_group(&group)
                .await
                .map_err(map_domain_error)?;
            let detail = to_group_detail_dto(&persisted, &actor.user_id);

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(detail).map_err(|_| ApiError::Internal)?,
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

async fn list_groups(
    State(state): State<AppState>,
    Query(query): Query<GroupListQuery>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<GroupListResponseDto>, ApiError> {
    let _actor = actor_identity(&auth)?;
    let offset = query
        .cursor
        .as_deref()
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let groups = state
        .group_repo
        .list_groups()
        .await
        .map_err(map_domain_error)?;
    let ordered = ordered_groups(groups)
        .into_iter()
        .filter(|group| group.join_policy != "undangan")
        .collect::<Vec<_>>();
    let total = ordered.len();
    let items = ordered
        .iter()
        .skip(offset)
        .take(limit)
        .map(|group| to_group_summary_dto(group))
        .collect::<Vec<_>>();
    let next_offset = offset.saturating_add(items.len());
    let cursor = if next_offset < total {
        Some(next_offset.to_string())
    } else {
        None
    };
    Ok(Json(GroupListResponseDto {
        items,
        total,
        cursor,
    }))
}

async fn list_my_groups(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<GroupSummaryDto>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let groups = state
        .group_repo
        .list_groups()
        .await
        .map_err(map_domain_error)?;
    let items = ordered_groups(groups)
        .into_iter()
        .filter(|group| {
            group
                .members
                .iter()
                .any(|member| member.user_id == actor.user_id)
        })
        .map(|group| to_group_summary_dto(&group))
        .collect::<Vec<_>>();
    Ok(Json(items))
}

async fn get_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<GroupDetailDto>, ApiError> {
    let actor = actor_identity(&auth)?;
    let group = state
        .group_repo
        .get_group(&group_id)
        .await
        .map_err(map_domain_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(to_group_detail_dto(&group, &actor.user_id)))
}

async fn update_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_update",
        format!("{}:{group_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if !group_is_admin(group_member_role(&group, &actor.user_id)) {
                return Err(ApiError::Forbidden);
            }
            if let Some(name) = payload.name.as_deref() {
                group.name = normalize_non_empty(name, "name")?;
            }
            if let Some(description) = payload.description.as_deref() {
                group.description = normalize_non_empty(description, "description")?;
            }
            if let Some(join_policy) = payload.join_policy.as_deref() {
                group.join_policy = normalize_group_join_policy(join_policy)
                    .ok_or_else(|| {
                        ApiError::Validation(
                            "join_policy must be 'terbuka', 'persetujuan', or 'undangan'".into(),
                        )
                    })?
                    .to_string();
            }
            group.member_count = group.members.len();
            group.updated_at_ms = now_ms;
            let persisted = state
                .group_repo
                .update_group(&group)
                .await
                .map_err(map_domain_error)?;
            let detail = to_group_detail_dto(&persisted, &actor.user_id);

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(detail).map_err(|_| ApiError::Internal)?,
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

async fn join_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_join",
        format!("{}:{group_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if group.join_policy != "terbuka" {
                return Err(ApiError::Validation(
                    "group cannot be joined directly; use request flow".into(),
                ));
            }
            let member = if let Some(existing) = group
                .members
                .iter()
                .find(|member| member.user_id == actor.user_id)
            {
                to_group_member_dto(existing)
            } else {
                group.pending_requests.retain(|request| {
                    !(request.user_id == actor.user_id && request.status == "pending")
                });
                let member = GroupMemberRecord {
                    user_id: actor.user_id.clone(),
                    name: actor.username.clone(),
                    avatar_url: None,
                    role: "anggota".to_string(),
                    joined_at_ms: now_ms,
                };
                group.members.push(member.clone());
                group.member_count = group.members.len();
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                to_group_member_dto(&member)
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(member).map_err(|_| ApiError::Internal)?,
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

async fn request_group_join(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<RequestGroupJoinRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_request_join",
        format!("{}:{group_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if group.join_policy != "persetujuan" {
                return Err(ApiError::Validation(
                    "group does not accept join requests".into(),
                ));
            }
            if group
                .members
                .iter()
                .any(|member| member.user_id == actor.user_id)
            {
                return Err(ApiError::Validation("already a member".into()));
            }
            let request = if let Some(existing) = group
                .pending_requests
                .iter()
                .find(|request| request.user_id == actor.user_id && request.status == "pending")
            {
                to_group_join_request_dto(existing)
            } else {
                let request = GroupJoinRequestRecord {
                    request_id: format!("req-{}", gotong_domain::util::uuid_v7_without_dashes()),
                    user_id: actor.user_id.clone(),
                    name: actor.username.clone(),
                    avatar_url: None,
                    message: payload
                        .message
                        .as_ref()
                        .map(|value| value.trim().to_string()),
                    status: "pending".to_string(),
                    requested_at_ms: now_ms,
                };
                group.pending_requests.insert(0, request.clone());
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                to_group_join_request_dto(&request)
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(request).map_err(|_| ApiError::Internal)?,
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

async fn approve_group_request(
    State(state): State<AppState>,
    Path((group_id, request_id_param)): Path<(String, String)>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_request_approve",
        format!("{}:{group_id}:{request_id_param}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if !group_can_manage(group_member_role(&group, &actor.user_id)) {
                return Err(ApiError::Forbidden);
            }
            let request = group
                .pending_requests
                .iter()
                .find(|request| {
                    request.request_id == request_id_param && request.status == "pending"
                })
                .cloned()
                .ok_or(ApiError::NotFound)?;
            let member = if let Some(existing) = group
                .members
                .iter()
                .find(|member| member.user_id == request.user_id)
            {
                group
                    .pending_requests
                    .retain(|item| item.request_id != request_id_param);
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                to_group_member_dto(existing)
            } else {
                let member = GroupMemberRecord {
                    user_id: request.user_id.clone(),
                    name: request.name.clone(),
                    avatar_url: request.avatar_url.clone(),
                    role: "anggota".to_string(),
                    joined_at_ms: now_ms,
                };
                group.members.push(member.clone());
                group.member_count = group.members.len();
                group
                    .pending_requests
                    .retain(|item| item.request_id != request_id_param);
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                to_group_member_dto(&member)
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: serde_json::to_value(member).map_err(|_| ApiError::Internal)?,
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

async fn reject_group_request(
    State(state): State<AppState>,
    Path((group_id, request_id_param)): Path<(String, String)>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_request_reject",
        format!("{}:{group_id}:{request_id_param}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if !group_can_manage(group_member_role(&group, &actor.user_id)) {
                return Err(ApiError::Forbidden);
            }
            let mut rejected = false;
            for request in &mut group.pending_requests {
                if request.request_id == request_id_param && request.status == "pending" {
                    request.status = "rejected".to_string();
                    rejected = true;
                    break;
                }
            }
            if !rejected {
                return Err(ApiError::NotFound);
            }
            group.updated_at_ms = now_ms;
            state
                .group_repo
                .update_group(&group)
                .await
                .map_err(map_domain_error)?;

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "rejected": rejected }),
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

async fn invite_group_member(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<InviteGroupMemberRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let invited_user_id = normalize_non_empty(&payload.user_id, "user_id")?;

    let key = IdempotencyKey::new(
        "group_invite",
        format!("{}:{group_id}:{invited_user_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if !group_can_manage(group_member_role(&group, &actor.user_id)) {
                return Err(ApiError::Forbidden);
            }
            let added = if group
                .members
                .iter()
                .any(|member| member.user_id == invited_user_id)
            {
                false
            } else {
                let member = GroupMemberRecord {
                    user_id: invited_user_id.clone(),
                    name: invited_user_id.clone(),
                    avatar_url: None,
                    role: "anggota".to_string(),
                    joined_at_ms: now_ms,
                };
                group.members.push(member);
                group.member_count = group.members.len();
                group.pending_requests.retain(|request| {
                    !(request.user_id == invited_user_id && request.status == "pending")
                });
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                true
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "added": added }),
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

async fn leave_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_leave",
        format!("{}:{group_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            let is_last_admin_leaving =
                matches!(group_member_role(&group, &actor.user_id), Some("admin"))
                    && group_admin_count(&group) <= 1
                    && group.members.len() > 1;
            if is_last_admin_leaving {
                return Err(ApiError::Conflict);
            }
            let previous_len = group.members.len();
            group
                .members
                .retain(|member| member.user_id != actor.user_id);
            let left = previous_len != group.members.len();
            group.member_count = group.members.len();
            group.pending_requests.retain(|request| {
                !(request.user_id == actor.user_id && request.status == "pending")
            });
            group.updated_at_ms = now_ms;
            state
                .group_repo
                .update_group(&group)
                .await
                .map_err(map_domain_error)?;

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "left": left }),
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

async fn remove_group_member(
    State(state): State<AppState>,
    Path((group_id, user_id)): Path<(String, String)>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
) -> Result<Response, ApiError> {
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new(
        "group_member_remove",
        format!("{}:{group_id}:{user_id}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            let actor_role =
                group_member_role(&group, &actor.user_id).map(std::string::ToString::to_string);
            let actor_can_manage = group_can_manage(actor_role.as_deref());
            let actor_is_admin = group_is_admin(actor_role.as_deref());
            if !actor_can_manage {
                return Err(ApiError::Forbidden);
            }
            let target_role =
                group_member_role(&group, &user_id).map(std::string::ToString::to_string);
            let removed = if target_role.is_none() {
                false
            } else {
                if matches!(target_role.as_deref(), Some("admin")) {
                    if !actor_is_admin {
                        return Err(ApiError::Forbidden);
                    }
                    if group_admin_count(&group) <= 1 {
                        return Err(ApiError::Conflict);
                    }
                }
                let previous_len = group.members.len();
                group.members.retain(|member| member.user_id != user_id);
                group.member_count = group.members.len();
                group
                    .pending_requests
                    .retain(|request| !(request.user_id == user_id && request.status == "pending"));
                group.updated_at_ms = now_ms;
                state
                    .group_repo
                    .update_group(&group)
                    .await
                    .map_err(map_domain_error)?;
                previous_len != group.members.len()
            };

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "removed": removed }),
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

async fn update_group_member_role(
    State(state): State<AppState>,
    Path((group_id, user_id)): Path<(String, String)>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<UpdateGroupMemberRoleRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    let role = normalize_group_member_role(&payload.role).ok_or_else(|| {
        ApiError::Validation("role must be 'admin', 'moderator', or 'anggota'".into())
    })?;

    let key = IdempotencyKey::new(
        "group_member_role_update",
        format!("{}:{group_id}:{user_id}:{role}", actor.user_id),
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
            let now_ms = gotong_domain::jobs::now_ms();
            let mut group = state
                .group_repo
                .get_group(&group_id)
                .await
                .map_err(map_domain_error)?
                .ok_or(ApiError::NotFound)?;
            if !group_is_admin(group_member_role(&group, &actor.user_id)) {
                return Err(ApiError::Forbidden);
            }
            let target_idx = group
                .members
                .iter()
                .position(|member| member.user_id == user_id)
                .ok_or(ApiError::NotFound)?;
            let current_role = group.members[target_idx].role.clone();
            if current_role == "admin" && role != "admin" && group_admin_count(&group) <= 1 {
                return Err(ApiError::Conflict);
            }
            group.members[target_idx].role = role.to_string();
            group.updated_at_ms = now_ms;
            state
                .group_repo
                .update_group(&group)
                .await
                .map_err(map_domain_error)?;
            let updated = true;

            let response = IdempotencyResponse {
                status_code: StatusCode::OK.as_u16(),
                body: json!({ "updated": updated }),
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

#[derive(Debug, Deserialize, Validate)]
struct CreateWitnessRequest {
    #[validate(length(min = 1, max = 32))]
    pub schema_version: String,
    #[validate(length(min = 1, max = 128))]
    pub triage_session_id: String,
}

fn mode_from_triage_route(route: &str) -> Mode {
    match route {
        "vault" => Mode::CatatanSaksi,
        "siaga" => Mode::Siaga,
        "catatan_komunitas" => Mode::CatatanKomunitas,
        _ => Mode::Komunitas,
    }
}

fn rahasia_level_from_triage_route(route: &str) -> String {
    if route == "vault" {
        return "L2".to_string();
    }
    "L0".to_string()
}

fn privacy_level_from_rahasia_level(value: &str) -> String {
    match value {
        "L1" => "l1".to_string(),
        "L2" => "l2".to_string(),
        "L3" => "l3".to_string(),
        _ => "public".to_string(),
    }
}

fn triage_missing_fields_from_result(result: &Value) -> Vec<String> {
    result
        .get("missing_fields")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn triage_result_string(result: &Value, key: &str) -> Option<String> {
    result
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
            let _feed_item = ingest_discovery_contribution_feed(
                &state,
                &actor,
                request_id.to_string(),
                correlation_id.to_string(),
                &contribution,
                None,
                None,
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

async fn create_witness(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<CreateWitnessRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    if payload.schema_version.trim() != TRIAGE_SCHEMA_VERSION {
        return Err(ApiError::Validation(format!(
            "schema_version must be '{TRIAGE_SCHEMA_VERSION}'"
        )));
    }
    let actor = actor_identity(&auth)?;
    let request_id = request_id_from_headers(&headers)?;
    let correlation_id = correlation_id_from_headers(&headers)?;

    let key = IdempotencyKey::new("witness_create", actor.user_id.clone(), request_id.clone());
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let now_ms = gotong_domain::jobs::now_ms();
            let triage_session_id = payload.triage_session_id.trim().to_string();
            let (route, triage_result, triage_messages) = {
                let mut sessions = state.triage_sessions.write().await;
                compact_triage_sessions(&mut sessions, now_ms);
                let session = sessions
                    .get(&triage_session_id)
                    .ok_or(ApiError::Validation("triage_session_id not found".into()))?;
                if session.owner_user_id != actor.user_id {
                    return Err(ApiError::Forbidden);
                }
                let messages = session
                    .messages
                    .iter()
                    .map(|message| {
                        json!({
                            "role": message.role,
                            "text": message.text,
                            "created_at_ms": message.created_at_ms,
                        })
                    })
                    .collect::<Vec<_>>();
                (
                    session.route.clone(),
                    session.latest_result.clone(),
                    messages,
                )
            };

            let result_schema_version = triage_result_string(&triage_result, "schema_version")
                .unwrap_or_else(|| "unknown".to_string());
            if result_schema_version != TRIAGE_SCHEMA_VERSION {
                let response = IdempotencyResponse {
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                    body: json!({
                        "error": {
                            "code": "validation_error",
                            "message": format!("triage result schema_version must be '{TRIAGE_SCHEMA_VERSION}'"),
                            "details": {
                                "triage_session_id": triage_session_id,
                                "schema_version": result_schema_version,
                            }
                        }
                    }),
                };
                state
                    .idempotency
                    .complete(&key, response.clone())
                    .await
                    .map_err(|err| {
                        tracing::error!(error = %err, "idempotency complete failed");
                        ApiError::Internal
                    })?;
                return Ok(to_response(response));
            }

            let triage_status =
                triage_result_string(&triage_result, "status").unwrap_or_else(|| "draft".into());
            if triage_status != "final" {
                let missing_fields = triage_missing_fields_from_result(&triage_result);
                let response = IdempotencyResponse {
                    status_code: StatusCode::CONFLICT.as_u16(),
                    body: json!({
                        "error": {
                            "code": "triage_incomplete",
                            "message": "triage session is still draft",
                            "details": {
                                "triage_session_id": triage_session_id,
                                "status": triage_status,
                            }
                        },
                        "missing_fields": missing_fields
                    }),
                };
                state
                    .idempotency
                    .complete(&key, response.clone())
                    .await
                    .map_err(|err| {
                        tracing::error!(error = %err, "idempotency complete failed");
                        ApiError::Internal
                    })?;
                return Ok(to_response(response));
            }

            let triage_kind =
                triage_result_string(&triage_result, "kind").unwrap_or_else(|| "witness".into());
            if triage_kind != "witness" {
                let response = IdempotencyResponse {
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                    body: json!({
                        "error": {
                            "code": "validation_error",
                            "message": "triage final kind must be 'witness' for /v1/witnesses",
                            "details": {
                                "triage_session_id": triage_session_id,
                                "kind": triage_kind,
                            }
                        }
                    }),
                };
                state
                    .idempotency
                    .complete(&key, response.clone())
                    .await
                    .map_err(|err| {
                        tracing::error!(error = %err, "idempotency complete failed");
                        ApiError::Internal
                    })?;
                return Ok(to_response(response));
            }

            let track_hint = triage_result_string(&triage_result, "track_hint");
            let seed_hint = triage_result_string(&triage_result, "seed_hint");
            let card = triage_result
                .get("card")
                .and_then(Value::as_object)
                .cloned();
            let title = card
                .as_ref()
                .and_then(|item| item.get("title"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("{} Baru", triage_label(&route)));
            let summary = card
                .as_ref()
                .and_then(|item| item.get("body"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .or_else(|| triage_result_string(&triage_result, "summary_text"));
            let rahasia_level = rahasia_level_from_triage_route(&route);
            let trajectory_type = triage_result_string(&triage_result, "trajectory_type");
            let taxonomy = triage_result.get("taxonomy").cloned().unwrap_or_else(|| {
                triage_taxonomy_payload(&route, trajectory_type.as_deref(), "witness")
            });
            let program_refs = triage_result
                .get("program_refs")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let stempel_state_value = triage_result.get("stempel_state").cloned();
            let impact_verification_value = impact_verification_not_open_json();

            let mut metadata = HashMap::new();
            metadata.insert("route".to_string(), Value::String(route.clone()));
            metadata.insert(
                "rahasia_level".to_string(),
                Value::String(rahasia_level.clone()),
            );
            metadata.insert(
                "schema_version".to_string(),
                Value::String(TRIAGE_SCHEMA_VERSION.to_string()),
            );
            metadata.insert(
                "triage_session_id".to_string(),
                Value::String(triage_session_id.clone()),
            );
            metadata.insert("kind".to_string(), Value::String(triage_kind));
            metadata.insert("status".to_string(), Value::String("open".to_string()));
            metadata.insert("message_count".to_string(), Value::from(0));
            metadata.insert("unread_count".to_string(), Value::from(0));
            if let Some(track_hint) = track_hint.clone() {
                metadata.insert("track_hint".to_string(), Value::String(track_hint));
            }
            if let Some(seed_hint) = seed_hint.clone() {
                metadata.insert("seed_hint".to_string(), Value::String(seed_hint));
            }
            if let Some(trajectory_type) = trajectory_type.clone() {
                metadata.insert(
                    "trajectory_type".to_string(),
                    Value::String(trajectory_type),
                );
            }
            metadata.insert("taxonomy".to_string(), taxonomy.clone());
            metadata.insert("program_refs".to_string(), program_refs.clone());
            metadata.insert("triage_result".to_string(), triage_result.clone());
            metadata.insert("triage_messages".to_string(), Value::Array(triage_messages));
            if let Some(stempel_state) = stempel_state_value.clone() {
                metadata.insert("stempel_state".to_string(), stempel_state);
            }
            metadata.insert(
                "impact_verification".to_string(),
                impact_verification_value.clone(),
            );
            if let Some(card) = card {
                metadata.insert("enrichment".to_string(), Value::Object(card));
            }

            let service = ContributionService::new(request_repos::contribution_repo(&state, &auth));
            let contribution = service
                .create(
                    actor.clone(),
                    request_id.clone(),
                    correlation_id.clone(),
                    ContributionCreate {
                        mode: mode_from_triage_route(&route),
                        contribution_type: ContributionType::Custom,
                        title: title.clone(),
                        description: summary.clone(),
                        evidence_url: None,
                        skill_ids: vec![],
                        metadata: Some(metadata.clone()),
                    },
                )
                .await
                .map_err(map_domain_error)?;

            if let Some(stempel_state) = stempel_state_value
                .as_ref()
                .and_then(stempel_state_from_value)
            {
                let mut registry = state.witness_stempel.write().await;
                registry.insert(contribution.contribution_id.clone(), stempel_state);
            }
            {
                let mut impact_registry = state.witness_impact_verifications.write().await;
                impact_registry.insert(
                    contribution.contribution_id.clone(),
                    WitnessImpactVerificationState::default(),
                );
            }

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
                        "failed to enqueue witness webhook outbox event"
                    );
                }
            }

            let mut feed_payload = serde_json::Map::new();
            for (key, value) in metadata {
                feed_payload.insert(key, value);
            }
            feed_payload.insert(
                "witness_id".to_string(),
                Value::String(contribution.contribution_id.clone()),
            );

            let feed_item = ingest_discovery_contribution_feed(
                &state,
                &actor,
                request_id.to_string(),
                correlation_id.to_string(),
                &contribution,
                Some(privacy_level_from_rahasia_level(&rahasia_level)),
                Some(Value::Object(feed_payload)),
            )
            .await?;
            let stream_item = to_feed_witness_stream_item(feed_item.clone());

            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::json!({
                    "witness_id": contribution.contribution_id,
                    "title": contribution.title,
                    "summary": contribution.description,
                    "track_hint": track_hint,
                    "seed_hint": seed_hint,
                    "rahasia_level": rahasia_level,
                    "author_id": actor.user_id,
                    "created_at_ms": contribution.created_at_ms,
                    "taxonomy": taxonomy,
                    "program_refs": program_refs,
                    "stempel_state": stempel_state_value,
                    "impact_verification": impact_verification_value,
                    "stream_item": stream_item,
                }),
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
struct FeedSuggestionsQueryParams {
    pub limit: Option<usize>,
    pub scope_id: Option<String>,
    pub privacy_level: Option<String>,
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct FeedMonitorPreferenceRequest {
    monitored: bool,
}

#[derive(Debug, Serialize)]
struct FeedMonitorPreferenceResponse {
    witness_id: String,
    monitored: bool,
}

#[derive(Debug, Deserialize)]
struct FeedFollowPreferenceRequest {
    followed: bool,
}

#[derive(Debug, Serialize)]
struct FeedFollowPreferenceResponse {
    entity_id: String,
    followed: bool,
}

#[derive(Debug, Serialize)]
struct FeedStreamResponse {
    items: Vec<gotong_domain::discovery::FeedItem>,
    stream: Vec<FeedStreamItemDto>,
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FeedStreamItemDto {
    Witness {
        stream_id: String,
        sort_timestamp: String,
        data: gotong_domain::discovery::FeedItem,
    },
    System {
        stream_id: String,
        sort_timestamp: String,
        data: FeedSystemCardDto,
    },
}

#[derive(Debug, Serialize)]
struct FeedSystemCardDto {
    variant: String,
    icon: String,
    title: String,
    description: Option<String>,
    dismissible: bool,
    payload: FeedSystemCardPayloadDto,
}

#[derive(Debug, Serialize)]
#[serde(tag = "variant", rename_all = "snake_case")]
enum FeedSystemCardPayloadDto {
    Suggestion {
        entities: Vec<FeedSuggestionEntityDto>,
    },
    Tip {
        tip_id: String,
    },
    Milestone {
        metric_label: String,
        metric_value: String,
    },
    Prompt {
        cta_label: String,
        cta_action: String,
    },
}

#[derive(Debug, Serialize)]
struct FeedSuggestionEntityDto {
    entity_id: String,
    entity_type: String,
    label: String,
    followed: bool,
    description: Option<String>,
    witness_count: usize,
    follower_count: usize,
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

const FEED_MONITOR_PREFERENCE_TABLE: &str = "feed_monitor_preference";
const FEED_FOLLOW_PREFERENCE_TABLE: &str = "feed_follow_preference";
const FEED_DB_SEED_SOURCE_PREFIX: &str = "seed-";

fn feed_preference_key(actor_id: &str, target_id: &str) -> String {
    format!("{actor_id}:{target_id}")
}

fn extract_witness_id(item: &gotong_domain::discovery::FeedItem) -> String {
    item.payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|payload| payload.get("witness_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| item.source_id.clone())
}

fn extract_entity_ids(item: &gotong_domain::discovery::FeedItem) -> Vec<String> {
    item.payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|payload| payload.get("enrichment"))
        .and_then(Value::as_object)
        .and_then(|enrichment| enrichment.get("entity_tags"))
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter()
                .filter_map(|tag| {
                    tag.as_object()
                        .and_then(|obj| obj.get("entity_id"))
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn ensure_payload_object(payload: Option<Value>) -> serde_json::Map<String, Value> {
    match payload {
        Some(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    }
}

fn is_db_seed_source_id(source_id: &str) -> bool {
    source_id
        .trim()
        .to_ascii_lowercase()
        .starts_with(FEED_DB_SEED_SOURCE_PREFIX)
}

fn apply_db_seed_dev_meta(payload: &mut serde_json::Map<String, Value>, source_id: &str) {
    if !is_db_seed_source_id(source_id) {
        return;
    }

    let mut dev_meta = match payload.remove("dev_meta") {
        Some(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    dev_meta.insert("is_seed".to_string(), Value::Bool(true));
    dev_meta.insert("seed_origin".to_string(), Value::String("db".to_string()));
    payload.insert("dev_meta".to_string(), Value::Object(dev_meta));
}

fn apply_follow_preferences_to_payload(
    payload: &mut serde_json::Map<String, Value>,
    follow_map: &HashMap<String, bool>,
) {
    let Some(Value::Object(enrichment)) = payload.get_mut("enrichment") else {
        return;
    };
    let Some(Value::Array(entity_tags)) = enrichment.get_mut("entity_tags") else {
        return;
    };
    for tag in entity_tags {
        let Some(tag_obj) = tag.as_object_mut() else {
            continue;
        };
        let entity_id = tag_obj
            .get("entity_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let Some(entity_id) = entity_id else {
            continue;
        };
        if let Some(followed) = follow_map.get(entity_id) {
            tag_obj.insert("followed".to_string(), Value::Bool(*followed));
        }
    }
}

fn sort_timestamp_ms(item: &gotong_domain::discovery::FeedItem) -> i64 {
    if item.occurred_at_ms > 0 {
        item.occurred_at_ms
    } else {
        item.created_at_ms
    }
}

fn to_sort_timestamp_iso(epoch_ms: i64) -> String {
    gotong_domain::util::format_ms_rfc3339(epoch_ms)
}

fn to_feed_witness_stream_item(item: gotong_domain::discovery::FeedItem) -> FeedStreamItemDto {
    FeedStreamItemDto::Witness {
        stream_id: format!("w-{}", item.feed_id),
        sort_timestamp: to_sort_timestamp_iso(sort_timestamp_ms(&item)),
        data: item,
    }
}

fn to_feed_suggestion_entity_dto(item: &FeedSuggestion) -> FeedSuggestionEntityDto {
    FeedSuggestionEntityDto {
        entity_id: item.entity_id.clone(),
        entity_type: item.entity_type.clone(),
        label: item.label.clone(),
        followed: item.followed,
        description: item.description.clone(),
        witness_count: item.witness_count,
        follower_count: item.follower_count,
    }
}

fn build_feed_system_cards(
    suggestions: &[FeedSuggestion],
    witness_count: usize,
) -> Vec<FeedSystemCardDto> {
    let mut cards = Vec::new();

    let suggestion_entities = suggestions
        .iter()
        .filter(|item| !item.followed)
        .take(3)
        .map(to_feed_suggestion_entity_dto)
        .collect::<Vec<_>>();
    if !suggestion_entities.is_empty() {
        cards.push(FeedSystemCardDto {
            variant: "suggestion".to_string(),
            icon: "💡".to_string(),
            title: "Ikuti topik yang relevan".to_string(),
            description: Some("Dapatkan update tentang isu yang Anda pedulikan.".to_string()),
            dismissible: true,
            payload: FeedSystemCardPayloadDto::Suggestion {
                entities: suggestion_entities,
            },
        });
    }

    cards.push(FeedSystemCardDto {
        variant: "tip".to_string(),
        icon: "📸".to_string(),
        title: "Tahukah Anda?".to_string(),
        description: Some(
            "Anda bisa melampirkan foto dan video sebagai bukti untuk memperkuat laporan."
                .to_string(),
        ),
        dismissible: true,
        payload: FeedSystemCardPayloadDto::Tip {
            tip_id: "tip-evidence-upload".to_string(),
        },
    });

    if witness_count > 0 {
        cards.push(FeedSystemCardDto {
            variant: "milestone".to_string(),
            icon: "🎉".to_string(),
            title: "Komunitas makin aktif!".to_string(),
            description: Some(
                "Sinyal partisipasi warga meningkat dari aktivitas feed terbaru.".to_string(),
            ),
            dismissible: true,
            payload: FeedSystemCardPayloadDto::Milestone {
                metric_label: "Aktivitas saksi terlihat".to_string(),
                metric_value: witness_count.to_string(),
            },
        });
    } else {
        cards.push(FeedSystemCardDto {
            variant: "prompt".to_string(),
            icon: "🧭".to_string(),
            title: "Belum ada update baru".to_string(),
            description: Some("Mulai saksi baru untuk memantik gotong royong.".to_string()),
            dismissible: false,
            payload: FeedSystemCardPayloadDto::Prompt {
                cta_label: "Buat Saksi".to_string(),
                cta_action: "create_witness".to_string(),
            },
        });
    }

    cards
}

fn build_feed_stream(
    witness_items: Vec<gotong_domain::discovery::FeedItem>,
    system_cards: Vec<FeedSystemCardDto>,
) -> Vec<FeedStreamItemDto> {
    const SYSTEM_CARD_INTERVAL: usize = 3;
    let mut stream = Vec::with_capacity(witness_items.len() + system_cards.len());
    let mut last_sort_timestamp = to_sort_timestamp_iso(gotong_domain::jobs::now_ms());
    let mut system_index = 0usize;
    let mut system_cards_iter = system_cards.into_iter();

    for (idx, item) in witness_items.into_iter().enumerate() {
        let sort_timestamp = to_sort_timestamp_iso(sort_timestamp_ms(&item));
        last_sort_timestamp = sort_timestamp.clone();
        stream.push(FeedStreamItemDto::Witness {
            stream_id: format!("w-{}", item.feed_id),
            sort_timestamp: sort_timestamp.clone(),
            data: item,
        });

        if (idx + 1) % SYSTEM_CARD_INTERVAL == 0 {
            if let Some(card) = system_cards_iter.next() {
                stream.push(FeedStreamItemDto::System {
                    stream_id: format!("sys-{system_index}"),
                    sort_timestamp: sort_timestamp.clone(),
                    data: card,
                });
                system_index += 1;
            }
        }
    }

    for card in system_cards_iter {
        stream.push(FeedStreamItemDto::System {
            stream_id: format!("sys-{system_index}"),
            sort_timestamp: last_sort_timestamp.clone(),
            data: card,
        });
        system_index += 1;
    }

    stream
}

async fn persist_feed_monitor_preference(
    state: &AppState,
    auth: &AuthContext,
    actor_id: &str,
    witness_id: &str,
    monitored: bool,
) -> Result<(), ApiError> {
    if let Some(session) = auth.surreal_db_session.as_ref() {
        let record_id = feed_preference_key(actor_id, witness_id);
        if monitored {
            session
                .client()
                .query(format!(
                    "UPSERT type::thing('{FEED_MONITOR_PREFERENCE_TABLE}', $record_id) CONTENT {{ user_id: $user_id, witness_id: $witness_id, monitored: $monitored, updated_at_ms: $updated_at_ms }};"
                ))
                .bind(("record_id", record_id))
                .bind(("user_id", actor_id.to_string()))
                .bind(("witness_id", witness_id.to_string()))
                .bind(("monitored", monitored))
                .bind(("updated_at_ms", gotong_domain::jobs::now_ms()))
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, actor_id = %actor_id, witness_id = %witness_id, "failed to persist monitor preference");
                    ApiError::Internal
                })?;
        } else {
            session
                .client()
                .query(format!(
                    "DELETE type::thing('{FEED_MONITOR_PREFERENCE_TABLE}', $record_id);"
                ))
                .bind(("record_id", record_id))
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, actor_id = %actor_id, witness_id = %witness_id, "failed to delete monitor preference");
                    ApiError::Internal
                })?;
        }
        return Ok(());
    }

    let key = feed_preference_key(actor_id, witness_id);
    let mut prefs = state.feed_monitor_preferences.write().await;
    if monitored {
        prefs.insert(key, true);
    } else {
        prefs.remove(&key);
    }
    Ok(())
}

async fn persist_feed_follow_preference(
    state: &AppState,
    auth: &AuthContext,
    actor_id: &str,
    entity_id: &str,
    followed: bool,
) -> Result<(), ApiError> {
    if let Some(session) = auth.surreal_db_session.as_ref() {
        let record_id = feed_preference_key(actor_id, entity_id);
        if followed {
            session
                .client()
                .query(format!(
                    "UPSERT type::thing('{FEED_FOLLOW_PREFERENCE_TABLE}', $record_id) CONTENT {{ user_id: $user_id, entity_id: $entity_id, followed: $followed, updated_at_ms: $updated_at_ms }};"
                ))
                .bind(("record_id", record_id))
                .bind(("user_id", actor_id.to_string()))
                .bind(("entity_id", entity_id.to_string()))
                .bind(("followed", followed))
                .bind(("updated_at_ms", gotong_domain::jobs::now_ms()))
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, actor_id = %actor_id, entity_id = %entity_id, "failed to persist follow preference");
                    ApiError::Internal
                })?;
        } else {
            session
                .client()
                .query(format!(
                    "DELETE type::thing('{FEED_FOLLOW_PREFERENCE_TABLE}', $record_id);"
                ))
                .bind(("record_id", record_id))
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, actor_id = %actor_id, entity_id = %entity_id, "failed to delete follow preference");
                    ApiError::Internal
                })?;
        }
        return Ok(());
    }

    let key = feed_preference_key(actor_id, entity_id);
    let mut prefs = state.feed_follow_preferences.write().await;
    if followed {
        prefs.insert(key, true);
    } else {
        prefs.remove(&key);
    }
    Ok(())
}

async fn load_monitor_preferences(
    state: &AppState,
    auth: &AuthContext,
    actor_id: &str,
    witness_ids: &[String],
) -> Result<HashMap<String, bool>, ApiError> {
    if witness_ids.is_empty() {
        return Ok(HashMap::new());
    }

    if let Some(session) = auth.surreal_db_session.as_ref() {
        let mut response = session
            .client()
            .query(format!(
                "SELECT witness_id, monitored FROM {FEED_MONITOR_PREFERENCE_TABLE} WHERE user_id = $user_id AND witness_id IN $witness_ids;"
            ))
            .bind(("user_id", actor_id.to_string()))
            .bind(("witness_ids", witness_ids.to_vec()))
            .await
            .map_err(|err| {
                tracing::error!(error = %err, actor_id = %actor_id, "failed to load monitor preferences");
                ApiError::Internal
            })?;
        let rows: Vec<Value> = response.take(0).map_err(|err| {
            tracing::error!(error = %err, actor_id = %actor_id, "failed to decode monitor preferences");
            ApiError::Internal
        })?;
        let mapped = rows
            .into_iter()
            .filter_map(|row| {
                let obj = row.as_object()?;
                let witness_id = obj
                    .get("witness_id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())?
                    .to_string();
                let monitored = obj
                    .get("monitored")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Some((witness_id, monitored))
            })
            .collect();
        return Ok(mapped);
    }

    let prefs = state.feed_monitor_preferences.read().await;
    let result = witness_ids
        .iter()
        .filter_map(|witness_id| {
            let key = feed_preference_key(actor_id, witness_id);
            prefs.get(&key).map(|value| (witness_id.clone(), *value))
        })
        .collect();
    Ok(result)
}

async fn load_follow_preferences(
    state: &AppState,
    auth: &AuthContext,
    actor_id: &str,
    entity_ids: &[String],
) -> Result<HashMap<String, bool>, ApiError> {
    if entity_ids.is_empty() {
        return Ok(HashMap::new());
    }

    if let Some(session) = auth.surreal_db_session.as_ref() {
        let mut response = session
            .client()
            .query(format!(
                "SELECT entity_id, followed FROM {FEED_FOLLOW_PREFERENCE_TABLE} WHERE user_id = $user_id AND entity_id IN $entity_ids;"
            ))
            .bind(("user_id", actor_id.to_string()))
            .bind(("entity_ids", entity_ids.to_vec()))
            .await
            .map_err(|err| {
                tracing::error!(error = %err, actor_id = %actor_id, "failed to load follow preferences");
                ApiError::Internal
            })?;
        let rows: Vec<Value> = response.take(0).map_err(|err| {
            tracing::error!(error = %err, actor_id = %actor_id, "failed to decode follow preferences");
            ApiError::Internal
        })?;
        let mapped = rows
            .into_iter()
            .filter_map(|row| {
                let obj = row.as_object()?;
                let entity_id = obj
                    .get("entity_id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())?
                    .to_string();
                let followed = obj
                    .get("followed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                Some((entity_id, followed))
            })
            .collect();
        return Ok(mapped);
    }

    let prefs = state.feed_follow_preferences.read().await;
    let result = entity_ids
        .iter()
        .filter_map(|entity_id| {
            let key = feed_preference_key(actor_id, entity_id);
            prefs.get(&key).map(|value| (entity_id.clone(), *value))
        })
        .collect();
    Ok(result)
}

async fn set_feed_monitor_preference(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(witness_id): Path<String>,
    Json(payload): Json<FeedMonitorPreferenceRequest>,
) -> Result<Json<FeedMonitorPreferenceResponse>, ApiError> {
    let actor = actor_identity(&auth)?;
    let witness_id = witness_id.trim().to_string();
    if witness_id.is_empty() {
        return Err(ApiError::Validation("witness_id is required".into()));
    }
    persist_feed_monitor_preference(
        &state,
        &auth,
        &actor.user_id,
        &witness_id,
        payload.monitored,
    )
    .await?;
    Ok(Json(FeedMonitorPreferenceResponse {
        witness_id,
        monitored: payload.monitored,
    }))
}

async fn set_feed_follow_preference(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(entity_id): Path<String>,
    Json(payload): Json<FeedFollowPreferenceRequest>,
) -> Result<Json<FeedFollowPreferenceResponse>, ApiError> {
    let actor = actor_identity(&auth)?;
    let entity_id = entity_id.trim().to_string();
    if entity_id.is_empty() {
        return Err(ApiError::Validation("entity_id is required".into()));
    }
    persist_feed_follow_preference(&state, &auth, &actor.user_id, &entity_id, payload.followed)
        .await?;
    Ok(Json(FeedFollowPreferenceResponse {
        entity_id,
        followed: payload.followed,
    }))
}

async fn list_discovery_feed(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<FeedListQueryParams>,
) -> Result<Json<FeedStreamResponse>, ApiError> {
    let actor = actor_identity(&auth)?;
    let actor_id = actor.user_id.clone();
    let feed_scope_id = query.scope_id.clone();
    let feed_privacy_level = query.privacy_level.clone();
    let feed_from_ms = query.from_ms;
    let feed_to_ms = query.to_ms;
    let feed_limit = query.limit;
    let feed_cursor = query.cursor.clone();
    let feed_involvement_only = query.involvement_only.unwrap_or(false);
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let request = FeedListQuery {
        actor_id: actor_id.clone(),
        cursor: feed_cursor,
        limit: feed_limit,
        scope_id: feed_scope_id.clone(),
        privacy_level: feed_privacy_level.clone(),
        from_ms: feed_from_ms,
        to_ms: feed_to_ms,
        involvement_only: feed_involvement_only,
    };
    let mut response = service.list_feed(request).await.map_err(map_domain_error)?;

    let witness_ids: Vec<String> = response.items.iter().map(extract_witness_id).collect();
    let entity_ids: Vec<String> = response
        .items
        .iter()
        .flat_map(extract_entity_ids)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let monitor_map = load_monitor_preferences(&state, &auth, &actor_id, &witness_ids).await?;
    let follow_map = load_follow_preferences(&state, &auth, &actor_id, &entity_ids).await?;

    for item in &mut response.items {
        let witness_id = extract_witness_id(item);
        let monitored = monitor_map.get(&witness_id).copied().unwrap_or(false);
        let mut payload = ensure_payload_object(item.payload.take());
        payload.insert("witness_id".to_string(), Value::String(witness_id));
        payload.insert("monitored".to_string(), Value::Bool(monitored));
        apply_db_seed_dev_meta(&mut payload, &item.source_id);
        apply_follow_preferences_to_payload(&mut payload, &follow_map);
        item.payload = Some(Value::Object(payload));
    }

    let mut suggestions = service
        .list_feed_suggestions(FeedSuggestionsQuery {
            actor_id: actor_id.clone(),
            limit: Some(6),
            scope_id: feed_scope_id,
            privacy_level: feed_privacy_level,
            from_ms: feed_from_ms,
            to_ms: feed_to_ms,
        })
        .await
        .map_err(map_domain_error)?;
    for suggestion in &mut suggestions {
        if let Some(followed) = follow_map.get(&suggestion.entity_id) {
            suggestion.followed = *followed;
        }
    }

    let next_cursor = response.next_cursor;
    let witness_items = response.items;
    let system_cards = build_feed_system_cards(&suggestions, witness_items.len());
    let stream_items = build_feed_stream(witness_items.clone(), system_cards);
    let has_more = next_cursor.is_some();

    Ok(Json(FeedStreamResponse {
        items: witness_items,
        stream: stream_items,
        next_cursor,
        has_more,
    }))
}

async fn list_discovery_feed_suggestions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<FeedSuggestionsQueryParams>,
) -> Result<Json<Vec<FeedSuggestion>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let actor_id = actor.user_id.clone();
    let service = DiscoveryService::new(
        request_repos::feed_repo(&state, &auth),
        request_repos::notification_repo(&state, &auth),
    );
    let request = FeedSuggestionsQuery {
        actor_id: actor_id.clone(),
        limit: query.limit,
        scope_id: query.scope_id,
        privacy_level: query.privacy_level,
        from_ms: query.from_ms,
        to_ms: query.to_ms,
    };
    let mut response = service
        .list_feed_suggestions(request)
        .await
        .map_err(map_domain_error)?;
    let entity_ids: Vec<String> = response.iter().map(|item| item.entity_id.clone()).collect();
    let follow_map = load_follow_preferences(&state, &auth, &actor_id, &entity_ids).await?;
    for suggestion in &mut response {
        if let Some(followed) = follow_map.get(&suggestion.entity_id) {
            suggestion.followed = *followed;
        }
    }
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
    let payload = attach_source_platform_id(payload, &state.config.webhook_source_platform_id);
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

fn attach_source_platform_id(mut payload: Value, platform_id: &str) -> Value {
    let platform_id = platform_id.trim();
    if platform_id.is_empty() {
        return payload;
    }

    let Some(payload_map) = payload.as_object_mut() else {
        return payload;
    };

    let has_existing = payload_map
        .get("source_platform_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if !has_existing {
        payload_map.insert(
            "source_platform_id".to_string(),
            Value::String(platform_id.to_string()),
        );
    }
    payload
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
    privacy_level: Option<String>,
    payload: Option<Value>,
) -> Result<gotong_domain::discovery::FeedItem, ApiError> {
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
    let metadata_payload = payload.or_else(|| {
        contribution.metadata.as_ref().map(|metadata| {
            let mut object = serde_json::Map::new();
            for (key, value) in metadata {
                object.insert(key.clone(), value.clone());
            }
            Value::Object(object)
        })
    });
    let input = FeedIngestInput {
        source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
        source_id: contribution.contribution_id.clone(),
        actor: actor.clone(),
        title: contribution.title.clone(),
        summary: contribution.description.clone(),
        scope_id: None,
        privacy_level: Some(privacy_level.unwrap_or_else(|| "public".to_string())),
        occurred_at_ms: Some(contribution.created_at_ms),
        request_id,
        correlation_id,
        request_ts_ms: Some(contribution.created_at_ms),
        participant_ids: vec![],
        payload: metadata_payload,
    };
    service.ingest_feed(input).await.map_err(map_domain_error)
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

const CHAT_ATTACHMENT_MAX_BYTES: usize = 20 * 1024 * 1024;
const CHAT_ATTACHMENT_URL_TTL_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, Deserialize)]
struct ChatMessagesQuery {
    since_created_at_ms: Option<i64>,
    since_message_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ChatAttachmentDownloadQuery {
    exp: i64,
    sig: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatAttachmentStoredMetadata {
    attachment_id: String,
    file_name: String,
    mime_type: String,
    size_bytes: usize,
    media_type: String,
    uploaded_by: String,
    created_at_ms: i64,
}

#[derive(Debug, Serialize)]
struct ChatAttachmentUploadResponse {
    attachment_id: String,
    file_name: String,
    mime_type: String,
    size_bytes: usize,
    media_type: String,
    url: String,
    expires_at_ms: i64,
}

#[derive(Serialize)]
struct ChatStreamEnvelope {
    event_type: &'static str,
    message: ChatMessage,
}

#[derive(Clone, Debug, Serialize)]
struct ChatAuthorSnapshot {
    user_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ChatMessageView {
    #[serde(flatten)]
    message: ChatMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<ChatAuthorSnapshot>,
}

#[derive(Debug, Deserialize)]
struct ChatAuthorRow {
    id: String,
    username: Option<String>,
    platform_role: Option<String>,
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

fn chat_attachment_file_path(root: &FsPath, attachment_id: &str) -> PathBuf {
    root.join(format!("{attachment_id}.bin"))
}

fn chat_attachment_meta_path(root: &FsPath, attachment_id: &str) -> PathBuf {
    root.join(format!("{attachment_id}.json"))
}

fn chat_attachment_s3_file_key(key_prefix: &str, attachment_id: &str) -> String {
    format!("{key_prefix}/{attachment_id}.bin")
}

fn chat_attachment_s3_meta_key(key_prefix: &str, attachment_id: &str) -> String {
    format!("{key_prefix}/{attachment_id}.json")
}

async fn save_chat_attachment_artifacts(
    state: &AppState,
    attachment_id: &str,
    _mime_type: &str,
    file_bytes: Vec<u8>,
    metadata_bytes: Vec<u8>,
) -> Result<(), ApiError> {
    match &state.chat_attachment_storage {
        ChatAttachmentStorage::Local { root } => {
            tokio::fs::create_dir_all(root)
                .await
                .map_err(|_| ApiError::Internal)?;
            let file_path = chat_attachment_file_path(root, attachment_id);
            tokio::fs::write(file_path, file_bytes)
                .await
                .map_err(|_| ApiError::Internal)?;
            let metadata_path = chat_attachment_meta_path(root, attachment_id);
            tokio::fs::write(metadata_path, metadata_bytes)
                .await
                .map_err(|_| ApiError::Internal)?;
            Ok(())
        }
        ChatAttachmentStorage::S3 {
            client,
            bucket,
            credentials,
            key_prefix,
        } => {
            let file_key = chat_attachment_s3_file_key(key_prefix, attachment_id);
            let metadata_key = chat_attachment_s3_meta_key(key_prefix, attachment_id);
            let file_put_url = bucket
                .put_object(Some(credentials), &file_key)
                .sign(Duration::from_secs(300));
            let file_put_response = client
                .put(file_put_url)
                .body(file_bytes)
                .send()
                .await
                .map_err(|_| ApiError::Internal)?;
            if !file_put_response.status().is_success() {
                return Err(ApiError::Internal);
            }

            let metadata_put_url = bucket
                .put_object(Some(credentials), &metadata_key)
                .sign(Duration::from_secs(300));
            let metadata_put_response = client
                .put(metadata_put_url)
                .body(metadata_bytes)
                .send()
                .await
                .map_err(|_| ApiError::Internal)?;
            if !metadata_put_response.status().is_success() {
                return Err(ApiError::Internal);
            }
            Ok(())
        }
    }
}

fn infer_chat_media_type(mime_type: &str) -> Option<&'static str> {
    let normalized = mime_type.trim().to_ascii_lowercase();
    if normalized.starts_with("image/") {
        return Some("image");
    }
    if normalized.starts_with("video/") {
        return Some("video");
    }
    if normalized.starts_with("audio/") {
        return Some("audio");
    }
    None
}

fn sanitize_chat_attachment_filename(file_name: &str, media_type: &str) -> String {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return format!("chat-attachment.{media_type}");
    }

    let mut sanitized = trimmed
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        sanitized = format!("chat-attachment.{media_type}");
    }
    sanitized
}

fn chat_attachment_signature(secret: &str, attachment_id: &str, expires_at_ms: i64) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac accepts any key length");
    mac.update(format!("{attachment_id}:{expires_at_ms}").as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn verify_chat_attachment_signature(
    secret: &str,
    attachment_id: &str,
    expires_at_ms: i64,
    signature: &str,
) -> bool {
    let expected = chat_attachment_signature(secret, attachment_id, expires_at_ms);
    if expected.len() != signature.len() {
        return false;
    }
    let expected_bytes = expected.as_bytes();
    let provided_bytes = signature.as_bytes();
    let mut diff: u8 = 0;
    for (left, right) in expected_bytes.iter().zip(provided_bytes.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

fn raw_record_id(value: &str) -> &str {
    value.split_once(':').map(|(_, id)| id).unwrap_or(value)
}

async fn lookup_chat_author_snapshot(
    session: &gotong_infra::auth::SurrealDbSession,
    user_id: &str,
) -> Option<ChatAuthorSnapshot> {
    let mut response = session
        .client()
        .query(
            "SELECT type::string(id) AS id, username, platform_role \
             FROM type::record('warga', $user_id)",
        )
        .bind(("user_id", user_id.to_string()))
        .await
        .ok()?;
    let rows: Vec<Value> = response.take(0).ok()?;
    let row: ChatAuthorRow = serde_json::from_value(rows.into_iter().next()?).ok()?;
    let resolved_user_id = raw_record_id(&row.id).to_string();
    let name = row
        .username
        .clone()
        .unwrap_or_else(|| resolved_user_id.clone());
    Some(ChatAuthorSnapshot {
        user_id: resolved_user_id,
        name,
        avatar_url: None,
        tier: None,
        role: row.platform_role,
    })
}

async fn hydrate_chat_message_views(
    auth: &AuthContext,
    actor: &ActorIdentity,
    messages: Vec<ChatMessage>,
) -> Vec<ChatMessageView> {
    let mut profiles = HashMap::<String, ChatAuthorSnapshot>::new();
    if let Some(actor_user_id) = auth.user_id.as_deref() {
        let actor_name = auth
            .username
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| actor.username.clone());
        profiles.insert(
            actor_user_id.to_string(),
            ChatAuthorSnapshot {
                user_id: actor_user_id.to_string(),
                name: actor_name,
                avatar_url: None,
                tier: None,
                role: Some(auth.role.as_str().to_string()),
            },
        );
    }

    for message in &messages {
        if profiles.contains_key(&message.author_id) {
            continue;
        }

        let profile = if let Some(session) = auth.surreal_db_session.as_ref() {
            lookup_chat_author_snapshot(session, &message.author_id).await
        } else {
            None
        };
        profiles.insert(
            message.author_id.clone(),
            profile.unwrap_or_else(|| ChatAuthorSnapshot {
                user_id: message.author_id.clone(),
                name: message.author_id.clone(),
                avatar_url: None,
                tier: None,
                role: None,
            }),
        );
    }

    messages
        .into_iter()
        .map(|message| ChatMessageView {
            author: profiles.get(&message.author_id).cloned(),
            message,
        })
        .collect()
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
) -> Result<Json<Vec<ChatMessageView>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let chat_repo = request_repos::chat_repo(&state, &auth);
    let messages = list_chat_messages_by_query(chat_repo, &actor, &thread_id, query).await?;
    let views = hydrate_chat_message_views(&auth, &actor, messages).await;
    Ok(Json(views))
}

async fn poll_chat_messages(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(thread_id): Path<String>,
    Query(query): Query<ChatMessagesQuery>,
) -> Result<Json<Vec<ChatMessageView>>, ApiError> {
    let actor = actor_identity(&auth)?;
    let chat_repo = request_repos::chat_repo(&state, &auth);
    let messages = list_chat_messages_by_query(chat_repo, &actor, &thread_id, query).await?;
    let views = hydrate_chat_message_views(&auth, &actor, messages).await;
    Ok(Json(views))
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

async fn upload_chat_attachment(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Result<Json<ChatAttachmentUploadResponse>, ApiError> {
    let actor = actor_identity(&auth)?;
    let mut uploaded_file: Option<(Vec<u8>, String, String)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::Validation("invalid multipart payload".into()))?
    {
        let Some(file_name) = field.file_name().map(str::to_string) else {
            continue;
        };
        let mime_type = field
            .content_type()
            .map(str::to_string)
            .ok_or_else(|| ApiError::Validation("attachment content_type is required".into()))?;
        let bytes = field
            .bytes()
            .await
            .map_err(|_| ApiError::Validation("failed to read attachment bytes".into()))?;
        if bytes.is_empty() {
            continue;
        }
        uploaded_file = Some((bytes.to_vec(), file_name, mime_type));
        break;
    }

    let Some((file_bytes, raw_file_name, mime_type)) = uploaded_file else {
        return Err(ApiError::Validation(
            "multipart form file is required".into(),
        ));
    };

    if file_bytes.len() > CHAT_ATTACHMENT_MAX_BYTES {
        return Err(ApiError::Validation(format!(
            "attachment exceeds max size of {CHAT_ATTACHMENT_MAX_BYTES} bytes"
        )));
    }

    let Some(media_type) = infer_chat_media_type(&mime_type) else {
        return Err(ApiError::Validation(
            "unsupported attachment mime type; only image/video/audio are allowed".into(),
        ));
    };

    let size_bytes = file_bytes.len();
    let attachment_id = gotong_domain::util::uuid_v7_without_dashes();
    let file_name = sanitize_chat_attachment_filename(&raw_file_name, media_type);
    let created_at_ms = gotong_domain::jobs::now_ms();
    let metadata = ChatAttachmentStoredMetadata {
        attachment_id: attachment_id.clone(),
        file_name: file_name.clone(),
        mime_type: mime_type.clone(),
        size_bytes,
        media_type: media_type.to_string(),
        uploaded_by: actor.user_id,
        created_at_ms,
    };
    let metadata_bytes = serde_json::to_vec(&metadata).map_err(|_| ApiError::Internal)?;
    save_chat_attachment_artifacts(
        &state,
        &attachment_id,
        &mime_type,
        file_bytes,
        metadata_bytes,
    )
    .await?;

    let expires_at_ms = created_at_ms + CHAT_ATTACHMENT_URL_TTL_MS;
    let signature =
        chat_attachment_signature(&state.config.jwt_secret, &attachment_id, expires_at_ms);
    let url = format!(
        "/v1/chat/attachments/{attachment_id}/download?exp={expires_at_ms}&sig={signature}"
    );

    Ok(Json(ChatAttachmentUploadResponse {
        attachment_id,
        file_name,
        mime_type,
        size_bytes,
        media_type: media_type.to_string(),
        url,
        expires_at_ms,
    }))
}

async fn download_chat_attachment(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
    Query(query): Query<ChatAttachmentDownloadQuery>,
) -> Result<Response, ApiError> {
    if attachment_id.trim().is_empty()
        || !attachment_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(ApiError::Validation("invalid attachment_id".into()));
    }

    let now_ms = gotong_domain::jobs::now_ms();
    if query.exp < now_ms {
        return Err(ApiError::Forbidden);
    }
    if !verify_chat_attachment_signature(
        &state.config.jwt_secret,
        &attachment_id,
        query.exp,
        query.sig.trim(),
    ) {
        return Err(ApiError::Forbidden);
    }

    let remaining_ms = query.exp.saturating_sub(now_ms).max(0);
    let remaining_secs = remaining_ms / 1000;

    match &state.chat_attachment_storage {
        ChatAttachmentStorage::Local { root } => {
            let metadata_path = chat_attachment_meta_path(root, &attachment_id);
            let file_path = chat_attachment_file_path(root, &attachment_id);
            let metadata_bytes = tokio::fs::read(metadata_path)
                .await
                .map_err(|_| ApiError::NotFound)?;
            let metadata: ChatAttachmentStoredMetadata =
                serde_json::from_slice(&metadata_bytes).map_err(|_| ApiError::Internal)?;
            let file_bytes = tokio::fs::read(file_path)
                .await
                .map_err(|_| ApiError::NotFound)?;

            let mut response = (StatusCode::OK, file_bytes).into_response();
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_str(&metadata.mime_type)
                    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
            );
            if let Ok(cache_control) =
                HeaderValue::from_str(&format!("private, max-age={remaining_secs}"))
            {
                response.headers_mut().insert(CACHE_CONTROL, cache_control);
            }
            if let Ok(content_disposition) =
                HeaderValue::from_str(&format!("inline; filename=\"{}\"", metadata.file_name))
            {
                response
                    .headers_mut()
                    .insert(CONTENT_DISPOSITION, content_disposition);
            }
            Ok(response)
        }
        ChatAttachmentStorage::S3 {
            client,
            bucket,
            credentials,
            key_prefix,
        } => {
            let metadata_key = chat_attachment_s3_meta_key(key_prefix, &attachment_id);
            let metadata_get_url = bucket
                .get_object(Some(credentials), &metadata_key)
                .sign(Duration::from_secs(60));
            let metadata_output = client
                .get(metadata_get_url)
                .send()
                .await
                .map_err(|_| ApiError::Internal)?;
            if metadata_output.status() == StatusCode::NOT_FOUND {
                return Err(ApiError::NotFound);
            }
            if !metadata_output.status().is_success() {
                return Err(ApiError::Internal);
            }
            let metadata_bytes = metadata_output
                .bytes()
                .await
                .map_err(|_| ApiError::Internal)?;
            let metadata: ChatAttachmentStoredMetadata =
                serde_json::from_slice(&metadata_bytes).map_err(|_| ApiError::Internal)?;
            let file_key = chat_attachment_s3_file_key(key_prefix, &attachment_id);
            let file_head_url = bucket
                .head_object(Some(credentials), &file_key)
                .sign(Duration::from_secs(60));
            let file_head_response = client
                .head(file_head_url)
                .send()
                .await
                .map_err(|_| ApiError::Internal)?;
            if file_head_response.status() == StatusCode::NOT_FOUND {
                return Err(ApiError::NotFound);
            }
            if !file_head_response.status().is_success() {
                return Err(ApiError::Internal);
            }
            let presign_secs = u64::try_from(remaining_secs.max(1))
                .ok()
                .map(|seconds| seconds.min(7 * 24 * 60 * 60))
                .unwrap_or(60);
            let mut signed_get = bucket.get_object(Some(credentials), &file_key);
            let response_content_type = metadata.mime_type;
            let response_content_disposition =
                format!("inline; filename=\"{}\"", metadata.file_name);
            signed_get
                .query_mut()
                .insert("response-content-type", response_content_type.as_str());
            signed_get.query_mut().insert(
                "response-content-disposition",
                response_content_disposition.as_str(),
            );
            let presigned = signed_get.sign(Duration::from_secs(presign_secs));
            let location =
                HeaderValue::from_str(presigned.as_str()).map_err(|_| ApiError::Internal)?;
            let mut response = StatusCode::TEMPORARY_REDIRECT.into_response();
            response.headers_mut().insert(LOCATION, location);
            if let Ok(cache_control) =
                HeaderValue::from_str(&format!("private, max-age={remaining_secs}"))
            {
                response.headers_mut().insert(CACHE_CONTROL, cache_control);
            }
            Ok(response)
        }
    }
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
            let mut views = hydrate_chat_message_views(&auth, &actor, vec![message]).await;
            let response_body = views.pop().ok_or(ApiError::Internal)?;
            let response = IdempotencyResponse {
                status_code: StatusCode::CREATED.as_u16(),
                body: serde_json::to_value(&response_body).map_err(|_| ApiError::Internal)?,
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
    Ok(Json(tandang_profile_snapshot_value(
        snapshot,
        &actor.user_id,
    )))
}

async fn get_tandang_user_profile_snapshot(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let _actor = actor_identity(&auth)?;
    let normalized_user_id = user_id
        .trim()
        .strip_prefix("gotong_royong:")
        .unwrap_or(user_id.trim())
        .trim();
    if normalized_user_id.is_empty() {
        return Err(ApiError::Validation("user_id is required".into()));
    }
    let snapshot = state
        .markov_client
        .user_profile_snapshot(normalized_user_id)
        .await
        .map_err(map_markov_error)?;
    Ok(Json(tandang_profile_snapshot_value(
        snapshot,
        normalized_user_id,
    )))
}

fn tandang_profile_snapshot_value(
    snapshot: MarkovProfileSnapshot,
    platform_user_id: &str,
) -> Value {
    let reputation = snapshot.reputation;
    let tier = snapshot.tier;
    let activity = snapshot.activity;
    let cv_hidup = snapshot.cv_hidup;
    let top_level_cache = cache_metadata_value(&reputation.meta);

    json!({
        "cache": top_level_cache,
        "data": {
            "source": "tandang",
            "platform_user_id": platform_user_id,
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
    })
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

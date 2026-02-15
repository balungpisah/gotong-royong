use std::collections::HashMap;

use axum::extract::{Extension, Path, Query, State};
use axum::{
    Json, Router,
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use gotong_domain::{
    auth::TrackRole,
    contributions::{Contribution, ContributionCreate, ContributionService, ContributionType},
    error::DomainError,
    evidence::{Evidence, EvidenceCreate, EvidenceService, EvidenceType},
    idempotency::BeginOutcome,
    identity::ActorIdentity,
    ports::idempotency::{IdempotencyKey, IdempotencyResponse},
    transitions::{
        TrackStateTransition, TrackTransitionInput, TrackTransitionService, TransitionAction,
        TransitionMechanism,
    },
    vouches::{Vouch, VouchCreate, VouchService, VouchWeightHint},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
            };

            let transition = service
                .track_state_transition(actor, token_role, input)
                .await
                .map_err(map_domain_error)?;
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
    }
}

fn to_response(response: IdempotencyResponse) -> Response {
    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
    (status, Json(response.body)).into_response()
}

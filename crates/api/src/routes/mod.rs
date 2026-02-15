use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::request_id::RequestId;
use validator::Validate;

use gotong_domain::idempotency::BeginOutcome;
use gotong_domain::ports::idempotency::{IdempotencyKey, IdempotencyResponse};

use crate::{error::ApiError, middleware as app_middleware, state::AppState, validation};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/echo", post(echo))
        .route("/v1/idempotent-echo", post(idempotent_echo))
        .layer(app_middleware::rate_limit_layer())
        .layer(app_middleware::timeout_layer())
        .layer(app_middleware::trace_layer())
        .layer(app_middleware::set_request_id_layer())
        .layer(app_middleware::propagate_request_id_layer())
        .layer(middleware::from_fn(app_middleware::auth_stub_middleware))
        .with_state(state)
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
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<IdempotentEchoRequest>,
) -> Result<Response, ApiError> {
    validation::validate(&payload)?;
    let request_id = request_id
        .header_value()
        .to_str()
        .unwrap_or("unknown")
        .to_string();
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

fn to_response(response: IdempotencyResponse) -> Response {
    let status = StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK);
    (status, Json(response.body)).into_response()
}

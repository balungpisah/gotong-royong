use axum::{
    Json, Router,
    extract::State,
    middleware,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{error::ApiError, middleware as app_middleware, state::AppState, validation};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/echo", post(echo))
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

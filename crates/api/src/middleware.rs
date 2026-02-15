use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Request, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::middleware::NoOpMiddleware;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::{Span, info_span};
use uuid::Uuid;

use gotong_domain::auth::Role;

use crate::error::ApiError;
use crate::state::AppState;

pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CorrelationId(pub String);

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub role: Role,
    pub is_authenticated: bool,
}

impl AuthContext {
    fn anonymous() -> Self {
        Self {
            user_id: None,
            role: Role::Anonymous,
            is_authenticated: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Claims {
    sub: String,
    role: Option<String>,
    exp: usize,
}

#[derive(Clone)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::now_v7().to_string();
        let value = HeaderValue::from_str(&id).ok()?;
        Some(RequestId::new(value))
    }
}

pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, RequestSpan> {
    TraceLayer::new_for_http().make_span_with(RequestSpan::default())
}

#[derive(Clone, Default)]
pub(crate) struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let request_id_header = HeaderName::from_static("x-request-id");
        let request_id = req
            .headers()
            .get(&request_id_header)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("-");
        let correlation_id = req
            .headers()
            .get(CORRELATION_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("-");
        info_span!(
            "http_request",
            method = %req.method(),
            uri = %req.uri(),
            request_id = %request_id,
            correlation_id = %correlation_id
        )
    }
}

pub fn set_request_id_layer() -> SetRequestIdLayer<UuidRequestId> {
    SetRequestIdLayer::x_request_id(UuidRequestId)
}

pub fn propagate_request_id_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::x_request_id()
}

pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}

pub type RateLimitLayer = GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware>;

pub fn rate_limit_layer() -> RateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .per_second(100)
        .burst_size(200)
        .finish()
        .expect("rate limit config");
    GovernorLayer {
        config: Arc::new(config),
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = match bearer_token(req.headers()) {
        Some(token) => token,
        None => {
            req.extensions_mut().insert(AuthContext::anonymous());
            return next.run(req).await;
        }
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(err) => {
            tracing::warn!(error = %err, "invalid auth token");
            return ApiError::Unauthorized.into_response();
        }
    };

    let role = data
        .claims
        .role
        .as_deref()
        .and_then(Role::from_str)
        .unwrap_or(Role::User);

    req.extensions_mut().insert(AuthContext {
        user_id: Some(data.claims.sub),
        role,
        is_authenticated: true,
    });

    next.run(req).await
}

pub async fn require_auth_middleware(req: Request<Body>, next: Next) -> Response {
    let authenticated = req
        .extensions()
        .get::<AuthContext>()
        .map(|ctx| ctx.is_authenticated)
        .unwrap_or(false);
    if authenticated {
        next.run(req).await
    } else {
        ApiError::Unauthorized.into_response()
    }
}

pub async fn correlation_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    let header_name = HeaderName::from_static(CORRELATION_ID_HEADER);
    let correlation_id = match req.headers().get(&header_name) {
        Some(value) => match value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => {
                return ApiError::Validation("invalid correlation id".into()).into_response();
            }
        },
        None => Uuid::now_v7().to_string(),
    };

    req.extensions_mut()
        .insert(CorrelationId(correlation_id.clone()));

    let mut response = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert(header_name, value);
    }
    response
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?;
    let value = value.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

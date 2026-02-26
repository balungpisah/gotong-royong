use axum::{
    body::Body,
    extract::MatchedPath,
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
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
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
use gotong_infra::auth::SurrealDbSession;

use crate::error::ApiError;
use crate::observability;
use crate::state::AppState;

pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CorrelationId(pub String);

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub access_token: Option<String>,
    pub surreal_db_session: Option<SurrealDbSession>,
    pub role: Role,
    pub is_authenticated: bool,
}

impl AuthContext {
    fn anonymous() -> Self {
        Self {
            user_id: None,
            username: None,
            access_token: None,
            surreal_db_session: None,
            role: Role::Anonymous,
            is_authenticated: false,
        }
    }
}

const DEV_BYPASS_USER_ID: &str = "dev-user";
const DEV_BYPASS_USERNAME: &str = "dev-user";
const DEV_BYPASS_ACCESS_TOKEN: &str = "dev-bypass-token";

fn dev_bypass_enabled(state: &AppState) -> bool {
    state.config.auth_dev_bypass_enabled && state.config.app_env.eq_ignore_ascii_case("development")
}

fn dev_bypass_auth_context() -> AuthContext {
    AuthContext {
        user_id: Some(DEV_BYPASS_USER_ID.to_string()),
        username: Some(DEV_BYPASS_USERNAME.to_string()),
        access_token: Some(DEV_BYPASS_ACCESS_TOKEN.to_string()),
        surreal_db_session: None,
        role: Role::User,
        is_authenticated: true,
    }
}

fn insert_auth_context(req: &mut Request<Body>, state: &AppState, reason: &'static str) {
    if dev_bypass_enabled(state) {
        tracing::debug!(
            reason,
            "auth dev bypass enabled; injecting synthetic auth context"
        );
        req.extensions_mut().insert(dev_bypass_auth_context());
        return;
    }

    req.extensions_mut().insert(AuthContext::anonymous());
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
    TraceLayer::new_for_http().make_span_with(RequestSpan)
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
        .unwrap_or_else(|| {
            tracing::error!(
                "rate limit config builder produced invalid values; using conservative default"
            );
            GovernorConfig::default()
        });
    GovernorLayer {
        config: Arc::new(config),
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = match auth_token(req.headers()) {
        Some(token) => token.to_string(),
        None => {
            insert_auth_context(&mut req, &state, "missing_token");
            return next.run(req).await;
        }
    };

    if let Some(auth_service) = &state.auth_service {
        let session = match auth_service.validate(&token).await {
            Ok(session) => session,
            Err(err) => {
                tracing::warn!(error = %err, "invalid surreal auth token");
                insert_auth_context(&mut req, &state, "invalid_surreal_token");
                return next.run(req).await;
            }
        };
        let role = match Role::parse(&session.identity.platform_role) {
            Some(role) => role,
            None => {
                tracing::warn!(
                    platform_role = %session.identity.platform_role,
                    "token missing or invalid platform_role"
                );
                insert_auth_context(&mut req, &state, "invalid_platform_role");
                return next.run(req).await;
            }
        };

        req.extensions_mut().insert(AuthContext {
            user_id: Some(session.identity.user_id),
            username: Some(session.identity.username),
            access_token: Some(token),
            surreal_db_session: Some(session.db_session),
            role,
            is_authenticated: true,
        });

        return next.run(req).await;
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(err) => {
            tracing::warn!(error = %err, "invalid auth token");
            insert_auth_context(&mut req, &state, "invalid_jwt");
            return next.run(req).await;
        }
    };

    let role = match data.claims.role.as_deref().and_then(Role::parse) {
        Some(role) => role,
        None => {
            tracing::warn!("token missing or invalid role");
            insert_auth_context(&mut req, &state, "invalid_role_claim");
            return next.run(req).await;
        }
    };

    let user_id = data.claims.sub;
    req.extensions_mut().insert(AuthContext {
        user_id: Some(user_id.clone()),
        username: Some(user_id),
        access_token: Some(token),
        surreal_db_session: None,
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

    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        req.headers_mut().insert(header_name.clone(), value);
    }

    req.extensions_mut()
        .insert(CorrelationId(correlation_id.clone()));

    let mut response = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert(header_name, value);
    }
    response
}

pub async fn metrics_layer(req: Request<Body>, next: Next) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().as_str().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched| matched.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let response = next.run(req).await;
    let status = response.status();
    observability::register_http_request(&method, &route, status, start.elapsed());
    response
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?;
    let value = value.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::COOKIE)?;
    let value = value.to_str().ok()?;
    for part in value.split(';') {
        let mut it = part.trim().splitn(2, '=');
        let name = it.next()?.trim();
        let val = it.next()?.trim();
        if name == "gr_session" {
            return Some(val);
        }
    }
    None
}

pub(crate) fn auth_token(headers: &HeaderMap) -> Option<&str> {
    bearer_token(headers).or_else(|| cookie_token(headers))
}

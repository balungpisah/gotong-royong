use axum::{
    body::Body,
    http::{HeaderValue, Request, header},
    middleware::Next,
    response::Response,
};
use governor::middleware::NoOpMiddleware;
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
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub role: Option<String>,
    pub is_authenticated: bool,
}

impl AuthContext {
    fn from_header(value: Option<&HeaderValue>) -> Self {
        let token = value.and_then(|v| v.to_str().ok()).map(|v| v.to_string());
        let is_authenticated = token.is_some();
        Self {
            user_id: token.clone(),
            role: None,
            is_authenticated,
        }
    }
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

pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
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

pub async fn auth_stub_middleware(mut req: Request<Body>, next: Next) -> Response {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    let ctx = AuthContext::from_header(auth_header);
    req.extensions_mut().insert(ctx);
    next.run(req).await
}

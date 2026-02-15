use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use gotong_domain::idempotency::InMemoryIdempotencyStore;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use serde_json::json;
use tower_util::ServiceExt;

use crate::routes;
use crate::state::AppState;
use gotong_infra::config::AppConfig;

#[derive(Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        port: 0,
        log_level: "info".to_string(),
        surreal_endpoint: "ws://127.0.0.1:8000".to_string(),
        data_backend: "memory".to_string(),
        surreal_ns: "gotong".to_string(),
        surreal_db: "chat".to_string(),
        surreal_user: "root".to_string(),
        surreal_pass: "root".to_string(),
        redis_url: "redis://127.0.0.1:6379".to_string(),
        jwt_secret: "test-secret".to_string(),
        s3_endpoint: "http://127.0.0.1:9000".to_string(),
        s3_bucket: "gotong-royong-evidence-test".to_string(),
        s3_region: "us-east-1".to_string(),
        s3_access_key: "test-access-key".to_string(),
        s3_secret_key: "test-secret-key".to_string(),
        worker_queue_prefix: "gotong:jobs".to_string(),
        worker_poll_interval_ms: 1000,
        worker_promote_batch: 10,
        worker_backoff_base_ms: 1000,
        worker_backoff_max_ms: 60000,
    }
}

fn test_token(secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_secs();
    let claims = Claims {
        sub: "user-123".to_string(),
        role: "user".to_string(),
        exp: (now + 3600) as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("token")
}

fn test_app() -> axum::Router {
    let config = test_config();
    let store = InMemoryIdempotencyStore::new("test");
    let state = AppState::with_idempotency_store(config, Arc::new(store));
    routes::router(state)
}

#[tokio::test]
async fn contribution_evidence_vouch_flow() {
    let app = test_app();
    let token = test_token("test-secret");
    let contribution_request = json!({
        "contribution_type": "task_completion",
        "title": "Test task",
        "description": "Completed integration test task",
        "skill_ids": ["skill-1", "skill-2"],
        "metadata": {
            "source": "unit-test"
        }
    });

    let create_contribution_response = Request::builder()
        .method("POST")
        .uri("/v1/contributions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token.clone()))
        .header("x-request-id", "flow-req-1")
        .body(Body::from(contribution_request.to_string()))
        .unwrap();

    let response = app
        .clone()
        .oneshot(create_contribution_response)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let contribution: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let contribution_id = contribution
        .get("contribution_id")
        .and_then(|value| value.as_str())
        .expect("contribution_id")
        .to_string();

    let contribution_list_request = Request::builder()
        .method("GET")
        .uri("/v1/contributions?author_id=user-123")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(contribution_list_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let contributions: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let is_in_list = contributions
        .as_array()
        .expect("array")
        .iter()
        .any(|item| item.get("contribution_id") == Some(&json!(contribution_id)));
    assert!(is_in_list);

    let evidence_request = json!({
        "contribution_id": contribution_id,
        "evidence_type": "photo_with_timestamp",
        "evidence_data": {
            "notes": "worked"
        },
        "proof": {
            "timestamp": "2026-02-14T01:00:00Z",
            "media_hash": "abcd1234abcd1234abcd1234abcd1234"
        }
    });
    let evidence_response = Request::builder()
        .method("POST")
        .uri("/v1/evidence")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token.clone()))
        .header("x-request-id", "flow-req-2")
        .body(Body::from(evidence_request.to_string()))
        .unwrap();
    let response = app
        .clone()
        .oneshot(evidence_response)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let evidence_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let evidence: serde_json::Value = serde_json::from_slice(&evidence_body).expect("json");
    let evidence_id = evidence
        .get("evidence_id")
        .and_then(|value| value.as_str())
        .expect("evidence_id");
    let evidence_get_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/evidence/{}", evidence_id))
        .header("authorization", format!("Bearer {}", token.clone()))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(evidence_get_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let vouch_request = json!({
        "vouchee_id": "user-456",
        "message": "Great contribution",
        "skill_id": "skill-1"
    });
    let response = Request::builder()
        .method("POST")
        .uri("/v1/vouches")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token.clone()))
        .header("x-request-id", "flow-req-3")
        .body(Body::from(vouch_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(response).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let vouch_list_request = Request::builder()
        .method("GET")
        .uri("/v1/vouches?vouchee_id=user-456")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(vouch_list_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn contribution_create_is_idempotent() {
    let app = test_app();
    let token = test_token("test-secret");

    let contribution_request = json!({
        "contribution_type": "task_completion",
        "title": "Idempotent task",
        "description": "This is a repeated create",
        "skill_ids": ["skill-1"]
    });

    let make_request = |request_id: &str| {
        Request::builder()
            .method("POST")
            .uri("/v1/contributions")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .header("x-request-id", request_id)
            .body(Body::from(contribution_request.to_string()))
            .unwrap()
    };

    let response_one = app
        .clone()
        .oneshot(make_request("idem-1"))
        .await
        .expect("response");
    assert_eq!(response_one.status(), StatusCode::CREATED);
    let response_one_body = to_bytes(response_one.into_body(), usize::MAX)
        .await
        .expect("body");
    let contribution_one: serde_json::Value =
        serde_json::from_slice(&response_one_body).expect("json");

    let response_two = app.oneshot(make_request("idem-1")).await.expect("response");
    assert_eq!(response_two.status(), StatusCode::CREATED);
    let response_two_body = to_bytes(response_two.into_body(), usize::MAX)
        .await
        .expect("body");
    let contribution_two: serde_json::Value =
        serde_json::from_slice(&response_two_body).expect("json");

    assert_eq!(contribution_one, contribution_two);
}

#[tokio::test]
async fn evidence_rejects_missing_contribution() {
    let app = test_app();
    let token = test_token("test-secret");

    let evidence_request = json!({
        "contribution_id": "missing-contribution",
        "evidence_type": "photo_with_timestamp",
        "evidence_data": {
            "notes": "orphan"
        },
        "proof": {
            "timestamp": "2026-02-14T01:00:00Z",
            "media_hash": "abcd1234abcd1234abcd1234abcd1234"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/evidence")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(evidence_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn protected_route_rejects_unauthenticated() {
    let app = test_app();
    let request = Request::builder()
        .method("POST")
        .uri("/v1/idempotent-echo")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"entity_id":"t-1","message":"hello"}"#))
        .unwrap();

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_accepts_valid_token() {
    let app = test_app();
    let token = test_token("test-secret");
    let request = Request::builder()
        .method("POST")
        .uri("/v1/idempotent-echo")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(r#"{"entity_id":"t-1","message":"hello"}"#))
        .unwrap();

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-correlation-id"));
}

#[tokio::test]
async fn protected_route_rejects_invalid_token() {
    let app = test_app();
    let request = Request::builder()
        .method("POST")
        .uri("/v1/idempotent-echo")
        .header("content-type", "application/json")
        .header("authorization", "Bearer invalid.token.here")
        .body(Body::from(r#"{"entity_id":"t-1","message":"hello"}"#))
        .unwrap();

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

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
    test_token_with_identity(secret, "user", "user-123")
}

fn test_token_with_identity(secret: &str, role: &str, sub: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_secs();
    let claims = Claims {
        sub: sub.to_string(),
        role: role.to_string(),
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
        .header("authorization", format!("Bearer {token}"))
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
        .header("authorization", format!("Bearer {token}"))
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
        .header("authorization", format!("Bearer {token}"))
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
        .uri(format!("/v1/evidence/{evidence_id}"))
        .header("authorization", format!("Bearer {token}"))
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
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "flow-req-3")
        .body(Body::from(vouch_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(response).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let vouch_list_request = Request::builder()
        .method("GET")
        .uri("/v1/vouches?vouchee_id=user-456")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(vouch_list_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn track_transition_end_to_end() {
    let app = test_app();
    let token = test_token("test-secret");
    let first_request = json!({
        "track": "resolve",
        "entity_id": "entity-100",
        "from_stage": "garap",
        "to_stage": "periksa",
        "transition_action": "object",
        "transition_type": "user_action",
        "mechanism": "user_action",
        "track_roles": ["participant"],
        "gate_status": "open",
        "gate_metadata": {
            "por_refs_ready": true
        },
        "occurred_at_ms": 1000
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "t-req-1")
        .body(Body::from(first_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let first_transition: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let first_transition_id = first_transition
        .get("transition_id")
        .and_then(|value| value.as_str())
        .expect("transition_id")
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "t-req-1")
        .body(Body::from(first_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let second_transition: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        first_transition_id,
        second_transition
            .get("transition_id")
            .and_then(|value| value.as_str())
            .expect("transition_id")
    );

    let timeline_request = Request::builder()
        .method("GET")
        .uri("/v1/transitions/entity-100/timeline")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(timeline_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let timeline: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert_eq!(timeline.len(), 1);
    assert_eq!(
        timeline[0].get("transition_id"),
        Some(&json!(first_transition_id))
    );

    let active_request = Request::builder()
        .method("GET")
        .uri("/v1/transitions/entity-100/active")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(active_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let active: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(active.get("active_stage"), Some(&json!("periksa")));

    let get_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/transitions/{first_transition_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(get_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let transition_by_id: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        transition_by_id.get("transition_id"),
        Some(&json!(first_transition_id))
    );
}

#[tokio::test]
async fn track_transition_request_id_is_entity_scoped() {
    let app = test_app();
    let token = test_token("test-secret");
    let request_payload = |entity_id: &str| {
        json!({
            "track": "resolve",
            "entity_id": entity_id,
            "from_stage": "garap",
            "to_stage": "periksa",
            "transition_action": "object",
            "transition_type": "user_action",
            "mechanism": "user_action",
            "track_roles": ["participant"],
            "gate_status": "open",
            "gate_metadata": {
                "por_refs_ready": true
            },
            "occurred_at_ms": 1000
        })
    };

    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "scoped-req-1")
        .body(Body::from(request_payload("entity-scope-a").to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let first_transition: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let first_transition_id = first_transition
        .get("transition_id")
        .and_then(|value| value.as_str())
        .expect("transition_id")
        .to_string();

    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "scoped-req-1")
        .body(Body::from(request_payload("entity-scope-b").to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let second_transition: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let second_transition_id = second_transition
        .get("transition_id")
        .and_then(|value| value.as_str())
        .expect("transition_id")
        .to_string();

    assert_ne!(first_transition_id, second_transition_id);
}

#[tokio::test]
async fn track_transition_validates_gate_and_role_matrix() {
    let app = test_app();
    let token = test_token("test-secret");
    let bad_gate_request = json!({
        "track": "resolve",
        "entity_id": "entity-bad-1",
        "from_stage": "garap",
        "to_stage": "periksa",
        "transition_action": "object",
        "transition_type": "user_action",
        "mechanism": "user_action",
        "track_roles": ["participant"],
        "gate_status": "open",
        "gate_metadata": {
            "por_refs_ready": false
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "bad-1")
        .body(Body::from(bad_gate_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bad_role_request = json!({
        "track": "resolve",
        "entity_id": "entity-bad-2",
        "from_stage": "garap",
        "to_stage": "periksa",
        "transition_action": "propose",
        "transition_type": "user_action",
        "mechanism": "user_action",
        "track_roles": ["participant"],
        "gate_status": "open",
        "gate_metadata": {
            "por_refs_ready": true
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/transitions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "bad-2")
        .body(Body::from(bad_role_request.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
            .header("authorization", format!("Bearer {token}"))
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
        .header("authorization", format!("Bearer {token}"))
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
        .header("authorization", format!("Bearer {token}"))
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

#[tokio::test]
async fn chat_thread_and_message_flow() {
    let app = test_app();
    let token = test_token("test-secret");

    let thread_request = json!({
        "scope_id": "scope-chat",
        "privacy_level": "public",
    });
    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-req-1")
        .header("x-correlation-id", "corr-chat-1")
        .body(Body::from(thread_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let thread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let thread_id = thread
        .get("thread_id")
        .and_then(|value| value.as_str())
        .expect("thread_id")
        .to_string();
    assert_eq!(thread.get("scope_id"), Some(&json!("scope-chat")));

    let replay_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-req-1")
        .header("x-correlation-id", "corr-chat-1b")
        .body(Body::from(thread_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(replay_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let replay_thread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        thread_id,
        replay_thread
            .get("thread_id")
            .and_then(|value| value.as_str())
            .expect("thread_id")
    );

    let list_request = Request::builder()
        .method("GET")
        .uri("/v1/chat/threads")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(list_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert!(
        list.iter()
            .any(|item| item.get("thread_id") == Some(&json!(thread_id)))
    );

    let message_request = json!({
        "body": "hello world",
        "attachments": [],
    });
    let send_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/chat/threads/{thread_id}/messages/send"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-msg-1")
        .body(Body::from(message_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(send_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let message: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let message_id = message
        .get("message_id")
        .and_then(|value| value.as_str())
        .expect("message_id")
        .to_string();

    let list_messages_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/messages"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(list_messages_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let messages: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].get("message_id"), Some(&json!(message_id)));

    let mark_cursor_request = json!({
        "message_id": message_id,
    });
    let mark_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/chat/threads/{thread_id}/read-cursor"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-cursor-1")
        .body(Body::from(mark_cursor_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(mark_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let read_cursor_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/read-cursor"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(read_cursor_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let cursor: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(cursor.get("last_read_message_id"), Some(&json!(message_id)));

    let members_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/members"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(members_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let members: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert!(
        members
            .iter()
            .any(|member| member.get("user_id") == Some(&json!("user-123")))
    );
}

#[tokio::test]
async fn chat_read_cursor_is_member_only() {
    let app = test_app();
    let owner_token = test_token("test-secret");
    let outsider_token = test_token_with_identity("test-secret", "user", "user-456");

    let thread_request = json!({
        "scope_id": "scope-private-chat",
        "privacy_level": "private",
    });
    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {owner_token}"))
        .header("x-request-id", "chat-priv-1")
        .header("x-correlation-id", "corr-chat-priv-1")
        .body(Body::from(thread_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let thread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let thread_id = thread
        .get("thread_id")
        .and_then(|value| value.as_str())
        .expect("thread_id");

    let read_cursor_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/read-cursor"))
        .header("authorization", format!("Bearer {outsider_token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(read_cursor_request).await.expect("response");
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_poll_messages_endpoint() {
    let app = test_app();
    let token = test_token("test-secret");

    let thread_request = json!({
        "scope_id": "scope-chat-poll",
        "privacy_level": "public",
    });
    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-poll-1")
        .body(Body::from(thread_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let thread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let thread_id = thread
        .get("thread_id")
        .and_then(|value| value.as_str())
        .expect("thread_id")
        .to_string();

    let message_request = json!({
        "body": "poll me",
        "attachments": [],
    });
    let send_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/chat/threads/{thread_id}/messages/send"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-poll-2")
        .body(Body::from(message_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(send_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let poll_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/messages/poll"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(poll_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let messages: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].get("body"), Some(&json!("poll me")));
}

#[tokio::test]
async fn chat_messages_query_rejects_since_message_without_created_at() {
    let app = test_app();
    let token = test_token("test-secret");

    let thread_request = json!({
        "scope_id": "scope-chat-catchup",
        "privacy_level": "public",
    });
    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-catchup-1")
        .body(Body::from(thread_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let thread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let thread_id = thread
        .get("thread_id")
        .and_then(|value| value.as_str())
        .expect("thread_id")
        .to_string();

    let message_request = json!({
        "body": "catchup test",
        "attachments": [],
    });
    let send_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/chat/threads/{thread_id}/messages/send"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-catchup-2")
        .body(Body::from(message_request.to_string()))
        .unwrap();
    let response = app.clone().oneshot(send_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let poll_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/chat/threads/{thread_id}/messages/poll?since_message_id=msg-dummy"
        ))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(poll_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

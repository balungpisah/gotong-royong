use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use gotong_domain::discovery::{
    DiscoveryService, FEED_SOURCE_CONTRIBUTION, FeedIngestInput, FeedListQuery, NOTIF_TYPE_SYSTEM,
    NotificationIngestInput, SearchListQuery,
};
use gotong_domain::idempotency::InMemoryIdempotencyStore;
use gotong_domain::identity::ActorIdentity;
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

fn actor_identity_for_tests(user_id: &str) -> ActorIdentity {
    ActorIdentity {
        user_id: user_id.to_string(),
        username: format!("{user_id}-name"),
    }
}

fn test_app() -> axum::Router {
    test_app_state_router().1
}

fn test_app_state() -> AppState {
    let config = test_config();
    let store = InMemoryIdempotencyStore::new("test");
    AppState::with_idempotency_store(config, Arc::new(store))
}

fn test_app_state_router() -> (AppState, axum::Router) {
    let state = test_app_state();
    let app = routes::router(state.clone());
    (state, app)
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

#[tokio::test]
async fn discovery_feed_and_search_endpoints() {
    let (state, app) = test_app_state_router();
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());

    let actor = actor_identity_for_tests("user-123");
    let second_actor = actor_identity_for_tests("user-456");

    let feed_a = service
        .ingest_feed(FeedIngestInput {
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-a".into(),
            actor: actor.clone(),
            title: "Need help in neighborhood".into(),
            summary: Some("Need help fixing flood barrier".into()),
            track: Some("resolve".into()),
            stage: Some("garap".into()),
            scope_id: Some("scope-rw-01".into()),
            privacy_level: Some("public".into()),
            occurred_at_ms: Some(1_000),
            request_id: "feed-ingest-1".into(),
            correlation_id: "corr-feed-1".into(),
            request_ts_ms: Some(1_000),
            participant_ids: vec!["user-456".into()],
            payload: None,
        })
        .await
        .expect("seed feed-a");

    let feed_b = service
        .ingest_feed(FeedIngestInput {
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-b".into(),
            actor: second_actor.clone(),
            title: "Cleanup support requested".into(),
            summary: Some("Need cleanup support this weekend".into()),
            track: Some("resolve".into()),
            stage: Some("garap".into()),
            scope_id: Some("scope-rw-01".into()),
            privacy_level: Some("public".into()),
            occurred_at_ms: Some(2_000),
            request_id: "feed-ingest-2".into(),
            correlation_id: "corr-feed-2".into(),
            request_ts_ms: Some(2_000),
            participant_ids: vec!["user-789".into()],
            payload: None,
        })
        .await
        .expect("seed feed-b");

    let _feed_c = service
        .ingest_feed(FeedIngestInput {
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-c".into(),
            actor: second_actor,
            title: "Neighborhood event announcement".into(),
            summary: Some("Community cleanup event scheduled".into()),
            track: Some("bantu".into()),
            stage: Some("garap".into()),
            scope_id: Some("scope-rw-01".into()),
            privacy_level: Some("private".into()),
            occurred_at_ms: Some(3_000),
            request_id: "feed-ingest-3".into(),
            correlation_id: "corr-feed-3".into(),
            request_ts_ms: Some(3_000),
            participant_ids: vec!["user-123".into()],
            payload: None,
        })
        .await
        .expect("seed feed-c");

    let feed_list_request = Request::builder()
        .method("GET")
        .uri("/v1/feed?scope_id=scope-rw-01&track=resolve&limit=10")
        .header(
            "authorization",
            format!("Bearer {}", test_token("test-secret")),
        )
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(feed_list_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed_list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let feed_items = feed_list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    let feed_ids: Vec<&str> = feed_items
        .iter()
        .map(|item| {
            item.get("feed_id")
                .and_then(|value| value.as_str())
                .unwrap()
        })
        .collect();
    assert!(feed_ids.iter().any(|id| *id == feed_a.feed_id));
    assert!(feed_ids.iter().any(|id| *id == feed_b.feed_id));
    assert_eq!(feed_ids.len(), 2);

    let search_request = Request::builder()
        .method("GET")
        .uri("/v1/search?query_text=cleanup&scope_id=scope-rw-01&limit=10")
        .header(
            "authorization",
            format!("Bearer {}", test_token("test-secret")),
        )
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(search_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let search_list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let search_items = search_list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    let search_ids: Vec<&str> = search_items
        .iter()
        .map(|item| {
            item.get("item")
                .and_then(|value| value.get("feed_id"))
                .and_then(|value| value.as_str())
                .expect("feed_id")
        })
        .collect();
    assert!(
        search_ids
            .iter()
            .any(|id| *id == feed_a.feed_id || *id == feed_b.feed_id)
    );

    let private_feed_request = Request::builder()
        .method("GET")
        .uri("/v1/feed?scope_id=scope-rw-01&track=resolve&involvement_only=true")
        .header(
            "authorization",
            format!("Bearer {}", test_token("test-secret")),
        )
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(private_feed_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let private_feed: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let private_feed_ids: Vec<&str> = private_feed
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items")
        .iter()
        .map(|item| {
            item.get("feed_id")
                .and_then(|value| value.as_str())
                .unwrap()
        })
        .collect();
    assert_eq!(private_feed_ids.len(), 1);
    assert_eq!(private_feed_ids[0], feed_a.feed_id.as_str());
}

#[tokio::test]
async fn discovery_feed_pagination_skips_hidden_rows_for_actor_visibility() {
    let (state, app) = test_app_state_router();
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
    let reader = actor_identity_for_tests("reader-user");
    let hidden_actor = actor_identity_for_tests("hidden-user");

    for idx in 0..6_usize {
        let is_visible = idx >= 3;
        let occurred_at_ms = 1_000 - (idx as i64 * 100);
        service
            .ingest_feed(FeedIngestInput {
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: format!("seed-hidden-pagination-{idx}"),
                actor: if is_visible {
                    reader.clone()
                } else {
                    hidden_actor.clone()
                },
                title: format!("seed {idx}"),
                summary: Some("pagination test payload".to_string()),
                track: None,
                stage: None,
                scope_id: None,
                privacy_level: Some(if is_visible { "public" } else { "private" }.to_string()),
                occurred_at_ms: Some(occurred_at_ms),
                request_id: format!("pagination-feed-{idx}"),
                correlation_id: format!("pagination-corr-{idx}"),
                request_ts_ms: Some(occurred_at_ms),
                participant_ids: Vec::new(),
                payload: None,
            })
            .await
            .expect("seed feed row");
    }

    let first_page = service
        .list_feed(FeedListQuery {
            actor_id: reader.user_id.clone(),
            cursor: None,
            limit: Some(2),
            scope_id: None,
            track: None,
            stage: None,
            privacy_level: None,
            from_ms: None,
            to_ms: None,
            involvement_only: false,
        })
        .await
        .expect("first page");

    assert_eq!(first_page.items.len(), 2);
    let first_page_ts: Vec<i64> = first_page
        .items
        .iter()
        .map(|item| item.occurred_at_ms)
        .collect();
    assert_eq!(first_page_ts, vec![700, 600]);
    let cursor = first_page.next_cursor.expect("cursor present");

    let second_page = service
        .list_feed(FeedListQuery {
            actor_id: reader.user_id.clone(),
            cursor: Some(cursor),
            limit: Some(2),
            scope_id: None,
            track: None,
            stage: None,
            privacy_level: None,
            from_ms: None,
            to_ms: None,
            involvement_only: false,
        })
        .await
        .expect("second page");

    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].occurred_at_ms, 500);
    assert!(second_page.next_cursor.is_none());

    let response = Request::builder()
        .method("GET")
        .uri("/v1/feed?limit=2")
        .header(
            "authorization",
            format!("Bearer {}", test_token("test-secret")),
        )
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(response).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed_list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let count = feed_list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items")
        .len();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn discovery_search_pagination_skips_hidden_rows_for_actor_visibility() {
    let (state, app) = test_app_state_router();
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());
    let reader = actor_identity_for_tests("reader-user");
    let hidden_actor = actor_identity_for_tests("hidden-user");

    for idx in 0..6_usize {
        let is_visible = idx >= 3;
        let occurred_at_ms = 1_000 - (idx as i64 * 100);
        service
            .ingest_feed(FeedIngestInput {
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: format!("seed-search-hidden-pagination-{idx}"),
                actor: if is_visible {
                    reader.clone()
                } else {
                    hidden_actor.clone()
                },
                title: format!("seed-search {idx}"),
                summary: Some("pagination search payload".to_string()),
                track: None,
                stage: None,
                scope_id: None,
                privacy_level: Some(if is_visible { "public" } else { "private" }.to_string()),
                occurred_at_ms: Some(occurred_at_ms),
                request_id: format!("search-pagination-feed-{idx}"),
                correlation_id: format!("search-pagination-corr-{idx}"),
                request_ts_ms: Some(occurred_at_ms),
                participant_ids: Vec::new(),
                payload: None,
            })
            .await
            .expect("seed feed row");
    }

    let first_page = service
        .search(SearchListQuery {
            actor_id: reader.user_id.clone(),
            query_text: "seed-search".into(),
            cursor: None,
            limit: Some(2),
            scope_id: None,
            track: None,
            stage: None,
            privacy_level: None,
            from_ms: None,
            to_ms: None,
            involvement_only: false,
            exclude_vault: false,
        })
        .await
        .expect("first page");

    assert_eq!(first_page.items.len(), 2);
    let first_page_ts: Vec<i64> = first_page
        .items
        .iter()
        .map(|result| result.item.occurred_at_ms)
        .collect();
    assert_eq!(first_page_ts, vec![700, 600]);
    let cursor = first_page.next_cursor.expect("cursor present");

    let second_page = service
        .search(SearchListQuery {
            actor_id: reader.user_id.clone(),
            query_text: "seed-search".into(),
            cursor: Some(cursor),
            limit: Some(2),
            scope_id: None,
            track: None,
            stage: None,
            privacy_level: None,
            from_ms: None,
            to_ms: None,
            involvement_only: false,
            exclude_vault: false,
        })
        .await
        .expect("second page");

    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].item.occurred_at_ms, 500);
    assert!(second_page.next_cursor.is_none());

    let search_request = Request::builder()
        .method("GET")
        .uri("/v1/search?query_text=seed-search&limit=2")
        .header(
            "authorization",
            format!("Bearer {}", test_token("test-secret")),
        )
        .body(Body::empty())
        .unwrap();
    let search_response = app.clone().oneshot(search_request).await.expect("response");
    assert_eq!(search_response.status(), StatusCode::OK);
    let body = to_bytes(search_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let search_list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let count = search_list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items")
        .len();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn discovery_notifications_endpoints() {
    let (state, app) = test_app_state_router();
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());

    let actor = actor_identity_for_tests("user-321");
    let token = test_token_with_identity("test-secret", "user", &actor.user_id);

    let notification_one = service
        .ingest_notification(NotificationIngestInput {
            recipient_id: actor.user_id.clone(),
            actor: actor.clone(),
            notification_type: NOTIF_TYPE_SYSTEM.to_string(),
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-a".into(),
            title: "Transition updated".into(),
            body: "Seed seed-a moved to periksa".into(),
            payload: None,
            privacy_level: Some("public".into()),
            request_id: "notif-ingest-1".into(),
            correlation_id: "corr-notif-1".into(),
            request_ts_ms: Some(1_234),
            dedupe_key: Some("notif-uniq-1".into()),
        })
        .await
        .expect("seed notification");

    let list_request = Request::builder()
        .method("GET")
        .uri("/v1/notifications?limit=10")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(list_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let unread_count = list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items")
        .len();
    assert_eq!(unread_count, 1);

    let read_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/notifications/{}/read",
            notification_one.notification_id
        ))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(read_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let read_notification: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        read_notification.get("notification_id"),
        Some(&json!(notification_one.notification_id))
    );
    assert!(read_notification.get("read_at_ms").is_some());

    let unread_count_request = Request::builder()
        .method("GET")
        .uri("/v1/notifications/unread-count")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(unread_count_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let unread: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(unread.get("unread_count"), Some(&json!(0)));

    let include_read_request = Request::builder()
        .method("GET")
        .uri("/v1/notifications?include_read=true")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app
        .clone()
        .oneshot(include_read_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let read_list: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let read_items = read_list
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    assert_eq!(read_items.len(), 1);

    let digest_request = Request::builder()
        .method("GET")
        .uri("/v1/notifications/weekly-digest?window_start_ms=0&window_end_ms=9999999999999")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(digest_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let digest: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(digest.get("user_id"), Some(&json!(actor.user_id)));
    assert_eq!(
        digest
            .get("events")
            .and_then(|value| value.as_array())
            .expect("events")
            .len(),
        1
    );
}

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json as AxumJson;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Router, extract::Path};
use gotong_domain::discovery::{
    DiscoveryService, FEED_SOURCE_CONTRIBUTION, FEED_SOURCE_ONTOLOGY_NOTE, FeedIngestInput,
    FeedListQuery, NOTIF_TYPE_SYSTEM, NotificationIngestInput, SearchListQuery,
};
use gotong_domain::idempotency::InMemoryIdempotencyStore;
use gotong_domain::identity::ActorIdentity;
use gotong_domain::ontology::OntologyEdgeKind;
use gotong_domain::ranking::wilson_score;
use gotong_domain::webhook::WebhookOutboxListQuery;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_util::ServiceExt;

use crate::observability;
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
        chat_realtime_transport: "local".to_string(),
        chat_realtime_channel_prefix: "gotong:chat:realtime:test".to_string(),
        worker_queue_prefix: "gotong:jobs".to_string(),
        worker_poll_interval_ms: 1000,
        worker_promote_batch: 10,
        worker_backoff_base_ms: 1000,
        worker_backoff_max_ms: 60000,
        worker_ttl_cleanup_interval_ms: 3_600_000,
        worker_concept_verification_interval_ms: 86_400_000,
        worker_concept_verification_qids: "Q2095".to_string(),
        webhook_enabled: false,
        webhook_markov_url: "http://127.0.0.1:8080/webhook".to_string(),
        webhook_secret: "dev_webhook_secret_32_chars_minimum".to_string(),
        webhook_max_attempts: 5,
        markov_read_base_url: "http://127.0.0.1:3000/api/v1".to_string(),
        markov_read_platform_token: "test-platform-token".to_string(),
        markov_read_timeout_ms: 2_500,
        markov_read_retry_max_attempts: 3,
        markov_read_retry_backoff_base_ms: 200,
        markov_read_retry_backoff_max_ms: 2_000,
        markov_read_circuit_fail_threshold: 5,
        markov_read_circuit_open_ms: 15_000,
        markov_cache_profile_ttl_ms: 300_000,
        markov_cache_profile_stale_while_revalidate_ms: 1_200_000,
        markov_cache_gameplay_ttl_ms: 45_000,
        markov_cache_gameplay_stale_while_revalidate_ms: 180_000,
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

fn test_app_state_router_webhook_enabled() -> (AppState, axum::Router) {
    let mut config = test_config();
    config.webhook_enabled = true;
    let store = InMemoryIdempotencyStore::new("test");
    let state = AppState::with_idempotency_store(config, Arc::new(store));
    let app = routes::router(state.clone());
    (state, app)
}

fn test_app_with_markov_base(markov_base_url: String) -> axum::Router {
    let mut config = test_config();
    config.markov_read_base_url = markov_base_url;
    config.markov_cache_profile_ttl_ms = 60_000;
    config.markov_cache_profile_stale_while_revalidate_ms = 120_000;
    config.markov_cache_gameplay_ttl_ms = 60_000;
    config.markov_cache_gameplay_stale_while_revalidate_ms = 120_000;
    let store = InMemoryIdempotencyStore::new("test");
    let state = AppState::with_idempotency_store(config, Arc::new(store));
    routes::router(state)
}

async fn spawn_markov_stub_base_url() -> String {
    async fn user_reputation(Path(identity): Path<String>) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "user_id": "markov-user-123",
            "identity": identity,
            "tier": "Contributor",
            "total_reputation": "0.77"
        }))
    }

    async fn user_tier(Path(user_id): Path<String>) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "user_id": user_id,
            "tier": "Contributor",
            "tier_symbol": "◆◆◇◇"
        }))
    }

    async fn user_activity(Path(user_id): Path<String>) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "user_id": user_id,
            "solutions_submitted": 4,
            "vouches_given": 2
        }))
    }

    async fn cv_hidup(Path(user_id): Path<String>) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "user_id": user_id,
            "username": "markov-user",
            "tier": "Contributor"
        }))
    }

    async fn skills_search() -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "results": [
                {"skill_id": "skill-1", "label": "cleanup"}
            ]
        }))
    }

    async fn por_requirements(Path(task_type): Path<String>) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "task_type": task_type,
            "min_media_items": 1
        }))
    }

    async fn por_triad(
        Path((track, transition)): Path<(String, String)>,
    ) -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "track": track,
            "stage_transition": transition,
            "min_of_three": 2
        }))
    }

    async fn leaderboard() -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "entries": [],
            "total_users": 42
        }))
    }

    async fn distribution() -> AxumJson<serde_json::Value> {
        AxumJson(json!({
            "keystone": 1,
            "pillar": 3,
            "contributor": 10,
            "novice": 28,
            "shadow": 0,
            "total": 42
        }))
    }

    let app = Router::new()
        .route("/api/v1/users/:id/reputation", get(user_reputation))
        .route("/api/v1/users/:id/tier", get(user_tier))
        .route("/api/v1/users/:id/activity", get(user_activity))
        .route("/api/v1/cv-hidup/:user_id", get(cv_hidup))
        .route("/api/v1/skills/search", get(skills_search))
        .route("/api/v1/por/requirements/:task_type", get(por_requirements))
        .route(
            "/api/v1/por/triad-requirements/:track/:transition",
            get(por_triad),
        )
        .route("/api/v1/reputation/leaderboard", get(leaderboard))
        .route("/api/v1/reputation/distribution", get(distribution));

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind markov stub");
    let addr = listener.local_addr().expect("markov stub addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve markov stub");
    });

    format!("http://{addr}/api/v1")
}

#[tokio::test]
async fn ontology_routes_support_feed_hierarchy_feedback_and_ranking() {
    let app = test_app();
    let token = test_token("test-secret");

    for concept in [
        json!({
            "concept_id": "Q93189",
            "qid": "Q93189",
            "label_id": "Telur",
            "label_en": "Egg",
            "verified": true
        }),
        json!({
            "concept_id": "Q2095",
            "qid": "Q2095",
            "label_id": "Makanan",
            "label_en": "Food",
            "verified": true
        }),
    ] {
        let request = Request::builder()
            .method("POST")
            .uri("/v1/ontology/concepts")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(concept.to_string()))
            .expect("request");
        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let broader_request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/concepts/Q93189/broader/Q2095")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(broader_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let feed_payload = json!({
        "content": "Telur Rp 28k di pasar",
        "community_id": "rt05",
        "temporal_class": "ephemeral",
        "ttl_expires_ms": 1_893_456_000_000i64,
        "confidence": 0.92,
        "triples": [
            {
                "edge": "About",
                "to_id": "concept:Q93189",
                "predicate": "schema:price",
                "metadata": {
                    "object_value": 28000,
                    "object_unit": "IDR"
                }
            }
        ]
    });
    let create_feed_request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(feed_payload.to_string()))
        .expect("request");
    let response = app
        .clone()
        .oneshot(create_feed_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed_response: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let note_id = feed_response
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id")
        .to_string();

    let get_concept_request = Request::builder()
        .method("GET")
        .uri("/v1/ontology/concepts/Q93189")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(get_concept_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let hierarchy_request = Request::builder()
        .method("GET")
        .uri("/v1/ontology/concepts/Q93189/hierarchy")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(hierarchy_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let hierarchy: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let has_parent = hierarchy
        .as_array()
        .expect("array")
        .iter()
        .any(|row| row.get("qid") == Some(&json!("Q2095")));
    assert!(has_parent);

    let vouch_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/ontology/notes/{note_id}/vouches"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"metadata":{"reason":"valid"}}"#))
        .expect("request");
    let response = app.clone().oneshot(vouch_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri(format!("/v1/ontology/notes/{note_id}/challenges"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"metadata":{"reason":"unsure"}}"#))
        .expect("request");
    let response = app
        .clone()
        .oneshot(challenge_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let feedback_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/feedback"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(feedback_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feedback: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(feedback.get("vouch_count"), Some(&json!(1)));
    assert_eq!(feedback.get("challenge_count"), Some(&json!(1)));

    let ranked_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/ranked"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(ranked_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let ranked: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let score = ranked
        .get("score")
        .and_then(|value| value.as_f64())
        .expect("score");
    let ranked_vouch_count = ranked
        .get("vouch_count")
        .and_then(|value| value.as_u64())
        .expect("vouch_count");
    let ranked_challenge_count = ranked
        .get("challenge_count")
        .and_then(|value| value.as_u64())
        .expect("challenge_count");
    let expected_score = wilson_score(
        ranked_vouch_count,
        ranked_vouch_count + ranked_challenge_count,
    );
    assert!((score - expected_score).abs() < 1e-12);
}

#[tokio::test]
async fn ontology_feed_validation_rejects_invalid_temporal_privacy_and_confidence() {
    let app = test_app();
    let token = test_token("test-secret");
    let payload = json!({
        "content": "Invalid ontology feed",
        "community_id": "rt05",
        "temporal_class": "ephemeral",
        "rahasia_level": 5,
        "confidence": 1.5
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ontology_concept_by_qid_returns_not_found_when_unknown() {
    let app = test_app();
    let token = test_token("test-secret");
    let request = Request::builder()
        .method("GET")
        .uri("/v1/ontology/concepts/Q_NOT_FOUND_999")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ontology_feed_rejects_has_action_without_predicate() {
    let app = test_app();
    let token = test_token("test-secret");
    let payload = json!({
        "content": "Action edge missing predicate",
        "community_id": "rt05",
        "temporal_class": "ephemeral",
        "ttl_expires_ms": 1_893_456_000_000i64,
        "triples": [
            {
                "edge": "HasAction",
                "to_id": "concept:Q2095"
            }
        ]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ontology_feed_rejects_has_action_with_invalid_predicate() {
    let app = test_app();
    let token = test_token("test-secret");
    let payload = json!({
        "content": "Action edge with wrong predicate",
        "community_id": "rt05",
        "temporal_class": "ephemeral",
        "ttl_expires_ms": 1_893_456_000_000i64,
        "triples": [
            {
                "edge": "HasAction",
                "to_id": "concept:Q2095",
                "predicate": "invalid:Action"
            }
        ]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ontology_ranking_returns_zero_for_empty_feedback() {
    let app = test_app();
    let token = test_token("test-secret");

    let concept = json!({
        "concept_id": "Q93189",
        "qid": "Q93189",
        "label_id": "Telur",
        "label_en": "Egg",
        "verified": true
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/concepts")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(concept.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let feed_payload = json!({
        "content": "Telur masih murah",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "confidence": 0.9
    });
    let create_feed_request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(feed_payload.to_string()))
        .expect("request");
    let response = app
        .clone()
        .oneshot(create_feed_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed_response: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let note_id = feed_response
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id")
        .to_string();

    let ranked_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/ranked"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(ranked_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let ranked: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(ranked.get("vouch_count"), Some(&json!(0)));
    assert_eq!(ranked.get("challenge_count"), Some(&json!(0)));
    assert_eq!(ranked.get("score"), Some(&json!(0.0)));
}

#[tokio::test]
async fn ontology_note_feedback_for_unknown_note_returns_zero_counts() {
    let app = test_app();
    let token = test_token("test-secret");
    let note_id = "note-unknown-for-feedback";

    let feedback_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/feedback"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(feedback_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feedback: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(feedback.get("note_id"), Some(&json!(note_id)));
    assert_eq!(feedback.get("vouch_count"), Some(&json!(0)));
    assert_eq!(feedback.get("challenge_count"), Some(&json!(0)));
}

#[tokio::test]
async fn ontology_ranking_for_unknown_note_returns_zero_score() {
    let app = test_app();
    let token = test_token("test-secret");
    let note_id = "note-unknown-for-ranking";

    let ranking_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/ranked"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(ranking_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let ranking: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(ranking.get("note_id"), Some(&json!(note_id)));
    assert_eq!(ranking.get("vouch_count"), Some(&json!(0)));
    assert_eq!(ranking.get("challenge_count"), Some(&json!(0)));
    assert_eq!(ranking.get("score"), Some(&json!(0.0)));
}

#[tokio::test]
async fn ontology_feed_rejects_ephemeral_without_ttl() {
    let app = test_app();
    let token = test_token("test-secret");
    let payload = json!({
        "content": "Ephemeral without ttl",
        "community_id": "rt05",
        "temporal_class": "ephemeral",
        "confidence": 0.95
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ontology_feed_rejects_negative_ttl_expires_ms() {
    let app = test_app();
    let token = test_token("test-secret");
    let payload = json!({
        "content": "Negative ttl",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "ttl_expires_ms": -1,
        "confidence": 0.95
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ontology_feed_accepts_explicit_note_id() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "note_id": "note-explicit-007",
        "content": "Explicit note id flow",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "confidence": 0.98
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed_response: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let note_id = feed_response
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id");
    assert_eq!(note_id, "note-explicit-007");

    let feedback_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/feedback"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(feedback_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let feedback: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    assert_eq!(feedback.get("note_id"), Some(&json!(note_id)));
}

#[tokio::test]
async fn ontology_feed_accepts_valid_schema_action_predicate() {
    let app = test_app();
    let token = test_token("test-secret");

    let concept = json!({
        "concept_id": "Q2048",
        "qid": "Q2048",
        "label_id": "Donasi",
        "label_en": "Donation",
        "verified": true
    });
    let upsert_request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/concepts")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(concept.to_string()))
        .expect("request");
    let upsert_response = app.clone().oneshot(upsert_request).await.expect("response");
    assert_eq!(upsert_response.status(), StatusCode::CREATED);

    let payload = json!({
        "content": "Valid schema action predicate",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "triples": [
            {
                "edge": "HasAction",
                "to_id": "concept:Q2048",
                "predicate": "schema:DonateAction"
            }
        ]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn ontology_feed_public_note_is_visible_in_discovery_feed() {
    let (_state, app) = test_app_state_router();
    let token = test_token("test-secret");

    let payload = json!({
        "content": "Public note should appear in discovery feed",
        "community_id": "rt05",
        "temporal_class": "persistent"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "ontology-discovery-1")
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let note_id = created
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id");
    let feed_id = created
        .get("feed_id")
        .and_then(|value| value.as_str())
        .expect("feed_id");

    let feed_request = Request::builder()
        .method("GET")
        .uri("/v1/feed?scope_id=rt05&limit=10")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(feed_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let feed: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let items = feed
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    let found = items.iter().find(|row| {
        row.get("feed_id").and_then(|value| value.as_str()) == Some(feed_id)
            && row.get("source_type").and_then(|value| value.as_str())
                == Some(FEED_SOURCE_ONTOLOGY_NOTE)
            && row.get("source_id").and_then(|value| value.as_str()) == Some(note_id)
    });
    let found = found.expect("ontology note feed item present");
    assert_eq!(
        found
            .get("payload")
            .and_then(|payload| payload.get("note"))
            .and_then(|note| note.get("note_id"))
            .and_then(|value| value.as_str()),
        Some(note_id)
    );
    assert!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .is_some()
    );
}

#[tokio::test]
async fn ontology_feed_normalizes_has_action_to_action_record_id() {
    let (state, app) = test_app_state_router();
    let token = test_token("test-secret");

    let payload = json!({
        "content": "Action normalization check",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "triples": [
            {
                "edge": "HasAction",
                "to_id": "concept:Q2048",
                "predicate": "schema:DonateAction"
            }
        ]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "ontology-action-normalize-1")
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let note_id = created
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id");

    let targets = state
        .ontology_repo
        .list_note_edge_targets(note_id, OntologyEdgeKind::HasAction)
        .await
        .expect("list targets");
    assert!(targets.iter().any(|value| value == "action:DonateAction"));
}

#[tokio::test]
async fn ontology_note_vouch_is_idempotent_and_patches_feed_payload() {
    let (_state, app) = test_app_state_router();
    let token = test_token("test-secret");

    let payload = json!({
        "content": "Idempotent vouch note",
        "community_id": "rt05",
        "temporal_class": "persistent"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "ontology-vouch-idem-1")
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let note_id = created
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id")
        .to_string();

    for _ in 0..2 {
        let vouch_request = Request::builder()
            .method("POST")
            .uri(format!("/v1/ontology/notes/{note_id}/vouches"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"metadata":{"reason":"valid"}}"#))
            .expect("request");
        let response = app.clone().oneshot(vouch_request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let feedback_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/feedback"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(feedback_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let feedback: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    assert_eq!(feedback.get("vouch_count"), Some(&json!(1)));

    let feed_request = Request::builder()
        .method("GET")
        .uri("/v1/feed?scope_id=rt05&limit=10")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(feed_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let feed: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let items = feed
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    let found = items.iter().find(|row| {
        row.get("source_type").and_then(|value| value.as_str()) == Some(FEED_SOURCE_ONTOLOGY_NOTE)
            && row.get("source_id").and_then(|value| value.as_str()) == Some(note_id.as_str())
    });
    let found = found.expect("feed item present");
    assert_eq!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("feedback"))
            .and_then(|feedback| feedback.get("vouch_count"))
            .and_then(|value| value.as_i64()),
        Some(1)
    );
    assert!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("feedback_enriched_at_ms"))
            .and_then(|value| value.as_i64())
            .is_some()
    );
    assert!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("tags_enriched_at_ms"))
            .and_then(|value| value.as_i64())
            .is_some()
    );
}

#[tokio::test]
async fn ontology_note_challenge_is_idempotent_and_patches_feed_payload() {
    let (_state, app) = test_app_state_router();
    let token = test_token("test-secret");

    let payload = json!({
        "content": "Idempotent challenge note",
        "community_id": "rt05",
        "temporal_class": "persistent",
        "triples": [{
            "edge": "HasAction",
            "predicate": "schema:InformAction",
            "to_id": "action:InformAction"
        }]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/ontology/feed")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "ontology-challenge-idem-1")
        .body(Body::from(payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let note_id = created
        .get("note")
        .and_then(|note| note.get("note_id"))
        .and_then(|value| value.as_str())
        .expect("note_id")
        .to_string();

    for _ in 0..2 {
        let challenge_request = Request::builder()
            .method("POST")
            .uri(format!("/v1/ontology/notes/{note_id}/challenges"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(r#"{"metadata":{"reason":"unsure"}}"#))
            .expect("request");
        let response = app
            .clone()
            .oneshot(challenge_request)
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let feedback_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/ontology/notes/{note_id}/feedback"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(feedback_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let feedback: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    assert_eq!(feedback.get("challenge_count"), Some(&json!(1)));

    let feed_request = Request::builder()
        .method("GET")
        .uri("/v1/feed?scope_id=rt05&limit=10")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(feed_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let feed: serde_json::Value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    let items = feed
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items");
    let found = items.iter().find(|row| {
        row.get("source_type").and_then(|value| value.as_str()) == Some(FEED_SOURCE_ONTOLOGY_NOTE)
            && row.get("source_id").and_then(|value| value.as_str()) == Some(note_id.as_str())
    });
    let found = found.expect("feed item present");
    assert_eq!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("feedback"))
            .and_then(|feedback| feedback.get("challenge_count"))
            .and_then(|value| value.as_i64()),
        Some(1)
    );
    assert_eq!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("feedback"))
            .and_then(|feedback| feedback.get("vouch_count"))
            .and_then(|value| value.as_i64()),
        Some(0)
    );
    assert!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("feedback_enriched_at_ms"))
            .and_then(|value| value.as_i64())
            .is_some()
    );
    assert!(
        found
            .get("payload")
            .and_then(|payload| payload.get("enrichment"))
            .and_then(|enrichment| enrichment.get("tags_enriched_at_ms"))
            .and_then(|value| value.as_i64())
            .is_some()
    );
}

#[tokio::test]
async fn contribution_create_rejects_invalid_mode_payload() {
    let app = test_app();
    let token = test_token("test-secret");
    let contribution_request = json!({
        "mode": "unknown_mode",
        "contribution_type": "task_completion",
        "title": "Invalid mode contribution",
        "description": "Must fail",
        "skill_ids": ["skill-1"]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/contributions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(contribution_request.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn contribution_create_rejects_missing_mode() {
    let app = test_app();
    let token = test_token("test-secret");
    let contribution_request = json!({
        "contribution_type": "task_completion",
        "title": "Missing mode contribution",
        "description": "Must fail",
        "skill_ids": ["skill-1"]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/contributions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(contribution_request.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.expect("response");
    assert!(response.status().is_client_error());
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn contribution_create_persists_mode_in_list_response() {
    let app = test_app();
    let token = test_token("test-secret");

    let contribution_request = json!({
        "mode": "siaga",
        "contribution_type": "task_completion",
        "title": "Mode persistence check",
        "description": "Verify mode round-trips",
        "skill_ids": ["skill-1"],
        "metadata": {
            "phase": "siaga_beta"
        }
    });

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/contributions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "mode-list-1")
        .body(Body::from(contribution_request.to_string()))
        .unwrap();
    let create_response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created: serde_json::Value = serde_json::from_slice(&create_body).expect("json");
    let contribution_id = created
        .get("contribution_id")
        .and_then(|value| value.as_str())
        .expect("contribution_id")
        .to_string();
    assert_eq!(created.get("mode"), Some(&json!("siaga")));

    let list_request = Request::builder()
        .method("GET")
        .uri("/v1/contributions?author_id=user-123")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let list_response = app.clone().oneshot(list_request).await.expect("response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let contributions: serde_json::Value = serde_json::from_slice(&list_body).expect("json");
    let contribution = contributions
        .as_array()
        .expect("array")
        .iter()
        .find(|item| item.get("contribution_id") == Some(&json!(contribution_id)))
        .expect("created contribution");
    assert_eq!(contribution.get("mode"), Some(&json!("siaga")));
}

#[tokio::test]
async fn contribution_evidence_vouch_flow() {
    let app = test_app();
    let token = test_token("test-secret");
    let contribution_request = json!({
        "mode": "komunitas",
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
async fn contribution_webhook_outbox_payload_includes_schema_and_request_id() {
    let (state, app) = test_app_state_router_webhook_enabled();
    let token = test_token("test-secret");
    let request_id = "webhook-contribution-req-1";

    let contribution_request = json!({
        "mode": "komunitas",
        "contribution_type": "task_completion",
        "title": "Webhook payload contract test",
        "description": "Ensures required payload fields are present",
        "skill_ids": ["skill-1"]
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/contributions")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", request_id)
        .header("x-correlation-id", "webhook-contribution-corr-1")
        .body(Body::from(contribution_request.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let outbox = state
        .webhook_outbox_repo
        .list(&WebhookOutboxListQuery {
            status: None,
            limit: 10,
        })
        .await
        .expect("outbox list");
    assert_eq!(outbox.len(), 1);
    let event = outbox.first().expect("webhook event");
    assert_eq!(event.request_id, request_id);
    assert_eq!(
        event
            .payload
            .get("schema_version")
            .and_then(|value| value.as_str()),
        Some("1")
    );
    assert_eq!(
        event
            .payload
            .get("request_id")
            .and_then(|value| value.as_str()),
        Some(request_id)
    );
}

#[tokio::test]
async fn vouch_submit_is_idempotent_with_request_id() {
    let app = test_app();
    let token = test_token("test-secret");

    let vouch_request = json!({
        "vouchee_id": "user-456",
        "message": "Great contribution",
        "skill_id": "skill-1"
    });
    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/vouches")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "vouch-idempotency-1")
        .body(Body::from(vouch_request.to_string()))
        .unwrap();
    let first_response = app.clone().oneshot(first_request).await.expect("response");
    assert_eq!(first_response.status(), StatusCode::CREATED);
    let first_body = to_bytes(first_response.into_body(), usize::MAX)
        .await
        .expect("body");

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/vouches")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "vouch-idempotency-1")
        .body(Body::from(vouch_request.to_string()))
        .unwrap();
    let second_response = app.clone().oneshot(second_request).await.expect("response");
    assert_eq!(second_response.status(), StatusCode::CREATED);
    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("body");
    assert_eq!(first_body, second_body);
}

#[tokio::test]
async fn adaptive_path_create_rejects_invalid_action_type() {
    let app = test_app();
    let token = test_token_with_identity("test-secret", "admin", "admin-action-invalid");

    let create_payload = json!({
        "entity_id": "case-adaptive-invalid",
        "payload": {
            "title": "Invalid adaptive plan",
            "summary": "Should fail",
            "action_type": "schema:UnknownAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks",
                            "status": "active",
                            "order": 0,
                            "source": "ai",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/adaptive-path/plans")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-invalid-action-1")
        .header("x-correlation-id", "adaptive-invalid-action-corr-1")
        .body(Body::from(create_payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert!(response.status().is_client_error());
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn adaptive_path_plan_flow_works() {
    let app = test_app();
    let token = test_token_with_identity("test-secret", "admin", "admin-123");

    let create_payload = json!({
        "entity_id": "case-adaptive-1",
        "editor_roles": ["project_manager"],
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks",
                            "status": "active",
                            "order": 0,
                            "source": "ai",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/adaptive-path/plans")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-create-1")
        .header("x-correlation-id", "adaptive-corr-1")
        .body(Body::from(create_payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created_plan: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let plan_id = created_plan
        .get("plan_id")
        .and_then(|value| value.as_str())
        .expect("plan_id")
        .to_string();

    let get_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(get_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let update_payload = json!({
        "expected_version": 1,
        "editor_roles": ["project_manager"],
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks terbaru",
                            "status": "active",
                            "order": 0,
                            "source": "human",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}/update"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-update-1")
        .header("x-correlation-id", "adaptive-corr-2")
        .body(Body::from(update_payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let updated_plan: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(updated_plan.get("version"), Some(&json!(2)));

    let suggest_payload = json!({
        "base_version": 2,
        "editor_roles": ["highest_profile_user"],
        "rationale": "update phase title",
        "model_id": "model-x",
        "prompt_version": "1.0",
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis lanjutan",
                            "objective": "Kumpulkan konteks",
                            "status": "active",
                            "order": 0,
                            "source": "ai",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}/suggestions"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-suggest-1")
        .header("x-correlation-id", "adaptive-corr-3")
        .body(Body::from(suggest_payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let suggestion: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let suggestion_id = suggestion
        .get("suggestion_id")
        .and_then(|value| value.as_str())
        .expect("suggestion_id")
        .to_string();

    let accept_payload = json!({
        "editor_roles": ["project_manager"]
    });
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/adaptive-path/suggestions/{suggestion_id}/accept"
        ))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-accept-1")
        .header("x-correlation-id", "adaptive-corr-4")
        .body(Body::from(accept_payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let accepted_plan: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(accepted_plan.get("version"), Some(&json!(3)));

    let events_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}/events"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.clone().oneshot(events_request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let events: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert!(
        events.len() >= 4,
        "expected at least create/update/suggest/accept events"
    );

    let suggestions_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}/suggestions"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app
        .clone()
        .oneshot(suggestions_request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let suggestions: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("json");
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].get("status"), Some(&json!("accepted")));
}

#[tokio::test]
async fn adaptive_path_plan_by_entity_returns_latest_plan() {
    let app = test_app();
    let token = test_token_with_identity("test-secret", "admin", "admin-entity");

    let create_payload = json!({
        "entity_id": "entity-lookup-1",
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks",
                            "status": "active",
                            "order": 0,
                            "source": "ai",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/adaptive-path/plans")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "adaptive-entity-create-1")
        .body(Body::from(create_payload.to_string()))
        .expect("request");
    let create_response = app.clone().oneshot(create_request).await.expect("response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created_plan: serde_json::Value = serde_json::from_slice(&create_body).expect("json");
    let created_plan_id = created_plan
        .get("plan_id")
        .and_then(|value| value.as_str())
        .expect("plan_id")
        .to_string();

    let by_entity_request = Request::builder()
        .method("GET")
        .uri("/v1/adaptive-path/entities/entity-lookup-1/plan")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let by_entity_response = app
        .clone()
        .oneshot(by_entity_request)
        .await
        .expect("response");
    assert_eq!(by_entity_response.status(), StatusCode::OK);
    let by_entity_body = to_bytes(by_entity_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let by_entity_plan: serde_json::Value = serde_json::from_slice(&by_entity_body).expect("json");
    let by_entity_plan_id = by_entity_plan
        .get("plan_id")
        .and_then(|value| value.as_str())
        .expect("plan_id");
    assert_eq!(by_entity_plan_id, created_plan_id);
    assert_eq!(by_entity_plan.get("version"), Some(&json!(1)));
}

#[tokio::test]
async fn adaptive_path_plan_by_entity_returns_not_found_when_unknown() {
    let app = test_app();
    let token = test_token_with_identity("test-secret", "admin", "admin-lookup-miss");

    let request = Request::builder()
        .method("GET")
        .uri("/v1/adaptive-path/entities/unknown-entity-404/plan")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn adaptive_path_user_cannot_spoof_privileged_editor_roles() {
    let app = test_app();
    let user_token = test_token("test-secret");

    let create_payload = json!({
        "entity_id": "case-adaptive-2",
        "editor_roles": ["project_manager"],
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks",
                            "status": "active",
                            "order": 0,
                            "source": "ai",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri("/v1/adaptive-path/plans")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {user_token}"))
        .header("x-request-id", "adaptive-user-create-1")
        .header("x-correlation-id", "adaptive-user-corr-1")
        .body(Body::from(create_payload.to_string()))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let created_plan: serde_json::Value = serde_json::from_slice(&body).expect("json");
    let plan_id = created_plan
        .get("plan_id")
        .and_then(|value| value.as_str())
        .expect("plan_id")
        .to_string();

    let update_payload = json!({
        "expected_version": 1,
        "editor_roles": ["project_manager"],
        "payload": {
            "title": "Rencana awal",
            "summary": "Ringkas",
            "action_type": "schema:InformAction",
            "branches": [
                {
                    "branch_id": "main",
                    "label": "Utama",
                    "parent_checkpoint_id": null,
                    "order": 0,
                    "phases": [
                        {
                            "phase_id": "phase-1",
                            "title": "Analisis",
                            "objective": "Kumpulkan konteks terbaru",
                            "status": "active",
                            "order": 0,
                            "source": "human",
                            "checkpoints": [
                                {
                                    "checkpoint_id": "checkpoint-1",
                                    "title": "Validasi data",
                                    "status": "open",
                                    "order": 0,
                                    "source": "ai"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });
    let request = Request::builder()
        .method("POST")
        .uri(format!("/v1/adaptive-path/plans/{plan_id}/update"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {user_token}"))
        .header("x-request-id", "adaptive-user-update-1")
        .header("x-correlation-id", "adaptive-user-corr-2")
        .body(Body::from(update_payload.to_string()))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn contribution_create_is_idempotent() {
    let app = test_app();
    let token = test_token("test-secret");

    let contribution_request = json!({
        "mode": "komunitas",
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
async fn chat_list_messages_defaults_to_latest_limit() {
    let app = test_app();
    let token = test_token("test-secret");

    let thread_request = json!({
        "scope_id": "scope-chat-latest",
        "privacy_level": "public",
    });
    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/chat/threads")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "chat-latest-1")
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

    let mut sent_ids = Vec::new();
    for idx in 0..3 {
        let message_request = json!({
            "body": format!("msg-{idx}"),
            "attachments": [],
        });
        let send_request = Request::builder()
            .method("POST")
            .uri(format!("/v1/chat/threads/{thread_id}/messages/send"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .header("x-request-id", format!("chat-latest-msg-{idx}"))
            .body(Body::from(message_request.to_string()))
            .unwrap();
        let response = app.clone().oneshot(send_request).await.expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let message: serde_json::Value = serde_json::from_slice(&body).expect("json");
        sent_ids.push(
            message
                .get("message_id")
                .and_then(|value| value.as_str())
                .expect("message_id")
                .to_string(),
        );
    }

    let list_messages_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/chat/threads/{thread_id}/messages?limit=2"))
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
    assert_eq!(messages.len(), 2);
    let listed_ids: Vec<String> = messages
        .iter()
        .map(|item| {
            item.get("message_id")
                .and_then(|value| value.as_str())
                .expect("message_id")
                .to_string()
        })
        .collect();
    assert_eq!(listed_ids, vec![sent_ids[1].clone(), sent_ids[2].clone()]);
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

    let feed_c = service
        .ingest_feed(FeedIngestInput {
            source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
            source_id: "seed-c".into(),
            actor: second_actor,
            title: "Neighborhood event announcement".into(),
            summary: Some("Community cleanup event scheduled".into()),
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
        .uri("/v1/feed?scope_id=scope-rw-01&limit=10")
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
    assert_eq!(feed_ids.len(), 3);

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
        .uri("/v1/feed?scope_id=scope-rw-01&involvement_only=true")
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
    assert_eq!(private_feed_ids.len(), 2);
    assert!(private_feed_ids.contains(&feed_a.feed_id.as_str()));
    assert!(private_feed_ids.contains(&feed_c.feed_id.as_str()));
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

#[tokio::test]
async fn discovery_notifications_pagination_cursor_does_not_skip_items() {
    let (state, _app) = test_app_state_router();
    let service = DiscoveryService::new(state.feed_repo.clone(), state.notification_repo.clone());

    let actor = actor_identity_for_tests("user-notif-page");

    for (idx, ts) in [(0, 3_000_i64), (1, 2_000_i64), (2, 1_000_i64)] {
        service
            .ingest_notification(NotificationIngestInput {
                recipient_id: actor.user_id.clone(),
                actor: actor.clone(),
                notification_type: NOTIF_TYPE_SYSTEM.to_string(),
                source_type: FEED_SOURCE_CONTRIBUTION.to_string(),
                source_id: format!("seed-notif-page-{idx}"),
                title: format!("Notif {idx}"),
                body: "pagination check".into(),
                payload: None,
                privacy_level: Some("public".into()),
                request_id: format!("notif-page-req-{idx}"),
                correlation_id: format!("notif-page-corr-{idx}"),
                request_ts_ms: Some(ts),
                dedupe_key: Some(format!("notif-page-dedupe-{idx}")),
            })
            .await
            .expect("seed notification");
    }

    let first_page = service
        .list_notifications(gotong_domain::discovery::NotificationListQuery {
            actor_id: actor.user_id.clone(),
            cursor: None,
            limit: Some(2),
            include_read: Some(true),
        })
        .await
        .expect("first page");

    assert_eq!(first_page.items.len(), 2);
    let first_ts: Vec<i64> = first_page.items.iter().map(|n| n.created_at_ms).collect();
    assert_eq!(first_ts, vec![3_000, 2_000]);
    let cursor = first_page.next_cursor.expect("cursor present");

    let second_page = service
        .list_notifications(gotong_domain::discovery::NotificationListQuery {
            actor_id: actor.user_id.clone(),
            cursor: Some(cursor),
            limit: Some(2),
            include_read: Some(true),
        })
        .await
        .expect("second page");

    assert_eq!(second_page.items.len(), 1);
    assert_eq!(second_page.items[0].created_at_ms, 1_000);
    assert!(second_page.next_cursor.is_none());
}

#[tokio::test]
async fn edgepod_ep03_duplicate_detection_success() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "request_id": "req_ep03_success_01",
        "correlation_id": "corr-ep03-success-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "user_action",
        "payload_version": "2026-02-14",
        "seed_text": "Need help with neighborhood cleanup and logistics",
        "media_hashes": ["h1", "h2"],
        "location": { "lat": -6.2, "lng": 106.8 },
        "radius_km": 2.5,
        "scope": "community"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/03/duplicate-detection")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep03_success_01")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let envelope: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(
        envelope.get("request_id"),
        Some(&json!("req_ep03_success_01"))
    );
    assert_eq!(envelope.get("result_version"), Some(&json!("v0.2.0")));
    assert_eq!(envelope.get("reason_code"), Some(&json!("OK")));
    let output = envelope
        .get("output")
        .and_then(|value| value.as_object())
        .expect("output");
    assert!(
        output
            .get("matches")
            .and_then(|value| value.as_array())
            .is_some()
    );
}

#[tokio::test]
async fn edgepod_ep05_gaming_risk_success_and_replay() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "request_id": "req_ep05_success_01",
        "correlation_id": "corr-ep05-success-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "query_users": ["user-a", "user-b", "user-c"],
        "lookback_hours": 24,
        "platform": "web",
        "focus_metric": "posting_rate"
    });

    let make_request = || {
        Request::builder()
            .method("POST")
            .uri("/v1/edge-pod/ai/05/gaming-risk")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .header("x-request-id", "req_ep05_success_01")
            .body(Body::from(payload.to_string()))
            .unwrap()
    };

    let first = app.clone().oneshot(make_request()).await.expect("response");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = to_bytes(first.into_body(), usize::MAX).await.expect("body");
    let first_envelope: serde_json::Value = serde_json::from_slice(&first_body).expect("json");

    let second = app.clone().oneshot(make_request()).await.expect("response");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = to_bytes(second.into_body(), usize::MAX)
        .await
        .expect("body");
    let second_envelope: serde_json::Value = serde_json::from_slice(&second_body).expect("json");

    assert_eq!(first_envelope, second_envelope);
}

#[tokio::test]
async fn edgepod_ep05_gaming_risk_token_role_controls_privilege() {
    let app = test_app();

    let role_sensitive_payload = json!({
        "request_id": "req_ep05_priv_user_01",
        "correlation_id": "corr-ep05-priv-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "moderator"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "query_users": ["user-1", "user-2", "user-8"],
        "lookback_hours": 24,
        "platform": "web"
    });
    let user_request_with_moderator_role = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/05/gaming-risk")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                test_token_with_identity("test-secret", "user", "user-123")
            ),
        )
        .header("x-request-id", "req_ep05_priv_user_01")
        .body(Body::from(role_sensitive_payload.to_string()))
        .unwrap();
    let user_response = app
        .clone()
        .oneshot(user_request_with_moderator_role)
        .await
        .expect("response");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_body = to_bytes(user_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let user_envelope_with_payload_role: serde_json::Value =
        serde_json::from_slice(&user_body).expect("json");
    assert_eq!(
        user_envelope_with_payload_role.get("reason_code"),
        Some(&json!("OK"))
    );
    let user_total_flags = user_envelope_with_payload_role
        .get("output")
        .and_then(|output| output.get("summary"))
        .and_then(|summary| summary.get("total_flags"))
        .and_then(|value| value.as_u64())
        .expect("user total flags");

    let user_payload_member = json!({
        "request_id": "req_ep05_priv_user_02",
        "correlation_id": "corr-ep05-priv-03",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "query_users": ["user-1", "user-2", "user-8"],
        "lookback_hours": 24,
        "platform": "web"
    });
    let user_request_with_member_role = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/05/gaming-risk")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                test_token_with_identity("test-secret", "user", "user-123")
            ),
        )
        .header("x-request-id", "req_ep05_priv_user_02")
        .body(Body::from(user_payload_member.to_string()))
        .unwrap();
    let user_response = app
        .clone()
        .oneshot(user_request_with_member_role)
        .await
        .expect("response");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_body = to_bytes(user_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let user_envelope_with_member_role: serde_json::Value =
        serde_json::from_slice(&user_body).expect("json");
    assert_eq!(
        user_envelope_with_member_role.get("reason_code"),
        Some(&json!("OK"))
    );
    let member_total_flags = user_envelope_with_member_role
        .get("output")
        .and_then(|output| output.get("summary"))
        .and_then(|summary| summary.get("total_flags"))
        .and_then(|value| value.as_u64())
        .expect("member total flags");

    assert_eq!(user_total_flags, member_total_flags);

    let moderator_payload = json!({
        "request_id": "req_ep05_priv_mod_01",
        "correlation_id": "corr-ep05-priv-02",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "query_users": ["user-1", "user-2", "user-8"],
        "lookback_hours": 24,
        "platform": "web"
    });
    let moderator_request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/05/gaming-risk")
        .header("content-type", "application/json")
        .header(
            "authorization",
            format!(
                "Bearer {}",
                test_token_with_identity("test-secret", "moderator", "user-123")
            ),
        )
        .header("x-request-id", "req_ep05_priv_mod_01")
        .body(Body::from(moderator_payload.to_string()))
        .unwrap();
    let moderator_response = app
        .clone()
        .oneshot(moderator_request)
        .await
        .expect("response");
    assert_eq!(moderator_response.status(), StatusCode::OK);
    let moderator_body = to_bytes(moderator_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let moderator_envelope: serde_json::Value =
        serde_json::from_slice(&moderator_body).expect("json");
    assert_eq!(moderator_envelope.get("reason_code"), Some(&json!("OK")));
    let moderator_total_flags = moderator_envelope
        .get("output")
        .and_then(|output| output.get("summary"))
        .and_then(|summary| summary.get("total_flags"))
        .and_then(|value| value.as_u64())
        .expect("moderator total flags");

    assert_eq!(moderator_total_flags, 3);
    assert!(moderator_total_flags > user_total_flags);
}

#[tokio::test]
async fn edgepod_ep08_sensitive_media_success() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "request_id": "req_ep08_success_01",
        "correlation_id": "corr-ep08-success-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "user_action",
        "payload_version": "2026-02-14",
        "media_urls": ["https://cdn.example.com/assets/example.jpg"],
        "media_types": ["image/jpeg"],
        "seed_id": "seed-01",
        "author_id": "author-123",
        "seed_text": "photo evidence for cleanup"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/edge-pod/ai/08/sensitive-media")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep08_success_01")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let envelope: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(envelope.get("reason_code"), Some(&json!("OK")));
    assert!(
        envelope
            .get("output")
            .and_then(|value| value.get("overall_safety"))
            .and_then(|value| value.as_str())
            .is_some()
    );
}

#[tokio::test]
async fn edgepod_ep09_credit_recommendation_success_and_fallback() {
    let app = test_app();
    let token = test_token("test-secret");

    let success_payload = json!({
        "request_id": "req_ep09_success_01",
        "correlation_id": "corr-ep09-success-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "user_id": "user-123",
        "timeline_events": [{"event":"contrib-submitted"}],
        "skill_profile": ["ar_site", "media"],
        "reputation_snapshot": { "forged": true }
    });

    let success_request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/09/credit-recommendation")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep09_success_01")
        .body(Body::from(success_payload.to_string()))
        .unwrap();
    let success_response = app
        .clone()
        .oneshot(success_request)
        .await
        .expect("response");
    assert_eq!(success_response.status(), StatusCode::OK);
    let success_body = to_bytes(success_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let success_envelope: serde_json::Value = serde_json::from_slice(&success_body).expect("json");
    assert_eq!(success_envelope.get("reason_code"), Some(&json!("OK")));
    assert!(
        success_envelope
            .get("output")
            .and_then(|value| value.get("dispute_window"))
            .is_some()
    );
    assert_eq!(
        success_envelope
            .get("output")
            .and_then(|value| value.get("confidence_source")),
        Some(&json!("heuristic"))
    );
    assert_eq!(
        success_envelope
            .get("actor_context")
            .and_then(|context| context.get("client_reputation_snapshot_ignored")),
        Some(&json!(true))
    );

    let fallback_payload = json!({
        "request_id": "req_ep09_fallback_01",
        "correlation_id": "corr-ep09-fallback-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "user_id": "user-123",
        "timeline_events": [{"event":"contrib-submitted"}],
        "skill_profile": ["ar_site", "media"]
    });

    let fallback_request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/09/credit-recommendation")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep09_fallback_01")
        .body(Body::from(fallback_payload.to_string()))
        .unwrap();
    let fallback_response = app
        .clone()
        .oneshot(fallback_request)
        .await
        .expect("response");
    assert_eq!(fallback_response.status(), StatusCode::OK);
    let fallback_body = to_bytes(fallback_response.into_body(), usize::MAX)
        .await
        .expect("body");
    let fallback_envelope: serde_json::Value =
        serde_json::from_slice(&fallback_body).expect("json");
    assert_eq!(
        fallback_envelope.get("reason_code"),
        Some(&json!("MODEL_UNAVAILABLE"))
    );
    assert!(
        fallback_envelope
            .get("actor_context")
            .and_then(|context| context.get("endpoint"))
            .is_some()
    );
}

#[tokio::test]
async fn edgepod_ep09_rejects_cross_user_lookup() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "request_id": "req_ep09_cross_user_01",
        "correlation_id": "corr-ep09-cross-user-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "member"
        },
        "trigger": "timer",
        "payload_version": "2026-02-14",
        "user_id": "user-999",
        "timeline_events": [{"event":"contrib-submitted"}],
        "skill_profile": ["ar_site", "media"]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/09/credit-recommendation")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep09_cross_user_01")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn edgepod_ep11_siaga_evaluate_success() {
    let app = test_app();
    let token = test_token("test-secret");

    let payload = json!({
        "request_id": "req_ep11_success_01",
        "correlation_id": "corr-ep11-success-01",
        "actor": {
            "user_id": "user-123",
            "platform_user_id": "platform-user-123",
            "role": "admin"
        },
        "trigger": "webhook",
        "payload_version": "2026-02-14",
        "text": "Sudden flood near the village road after rain",
        "location": { "lat": -6.2, "lng": 106.8 },
        "reported_urgency": "high",
        "community_scope": "rw-01",
        "current_track": "resolve"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/edge-pod/ai/siaga/evaluate")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("x-request-id", "req_ep11_success_01")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let envelope: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(envelope.get("result_version"), Some(&json!("v0.2.0")));
    let output = envelope
        .get("output")
        .and_then(|value| value.as_object())
        .expect("output");
    assert!(output.get("severity").is_some());
    assert!(output.get("responder_payload").is_some());
}

#[tokio::test]
async fn tandang_routes_surface_cache_metadata_and_data() {
    let markov_base_url = spawn_markov_stub_base_url().await;
    let app = test_app_with_markov_base(markov_base_url);
    let token = test_token("test-secret");

    let profile_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/me/profile")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("profile request");
    let profile_response = app
        .clone()
        .oneshot(profile_request)
        .await
        .expect("profile response");
    assert_eq!(profile_response.status(), StatusCode::OK);
    let profile_body = to_bytes(profile_response.into_body(), usize::MAX)
        .await
        .expect("profile body");
    let profile_json: serde_json::Value =
        serde_json::from_slice(&profile_body).expect("profile json");
    assert!(
        profile_json
            .get("cache")
            .and_then(|cache| cache.get("status"))
            .and_then(|value| value.as_str())
            .is_some()
    );
    assert_eq!(
        profile_json
            .get("data")
            .and_then(|data| data.get("reputation"))
            .and_then(|value| value.get("user_id")),
        Some(&json!("markov-user-123"))
    );
    assert!(
        profile_json
            .get("data")
            .and_then(|data| data.get("component_cache"))
            .and_then(|cache| cache.get("reputation"))
            .and_then(|cache| cache.get("status"))
            .is_some()
    );

    let profile_second_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/me/profile")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("profile second request");
    let profile_second_response = app
        .clone()
        .oneshot(profile_second_request)
        .await
        .expect("profile second response");
    assert_eq!(profile_second_response.status(), StatusCode::OK);
    let profile_second_body = to_bytes(profile_second_response.into_body(), usize::MAX)
        .await
        .expect("profile second body");
    let profile_second_json: serde_json::Value =
        serde_json::from_slice(&profile_second_body).expect("profile second json");
    assert_eq!(
        profile_second_json
            .get("cache")
            .and_then(|cache| cache.get("status")),
        Some(&json!("hit"))
    );

    for endpoint in [
        "/v1/tandang/skills/search?q=cleanup",
        "/v1/tandang/por/requirements/delivery",
        "/v1/tandang/por/triad-requirements/resolve/seed_to_define",
        "/v1/tandang/reputation/leaderboard?limit=5",
        "/v1/tandang/reputation/distribution",
    ] {
        let request = Request::builder()
            .method("GET")
            .uri(endpoint)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request");
        let response = app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::OK, "endpoint {endpoint}");
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let parsed: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert!(
            parsed
                .get("cache")
                .and_then(|cache| cache.get("status"))
                .is_some(),
            "missing cache status for endpoint {endpoint}"
        );
        assert!(
            parsed.get("data").is_some(),
            "missing data for endpoint {endpoint}"
        );
    }
}

#[tokio::test]
async fn tandang_routes_match_read_contract_shapes() {
    let markov_base_url = spawn_markov_stub_base_url().await;
    let app = test_app_with_markov_base(markov_base_url);
    let token = test_token("test-secret");

    let profile_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/me/profile")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("profile request");
    let profile_response = app
        .clone()
        .oneshot(profile_request)
        .await
        .expect("profile response");
    assert_eq!(profile_response.status(), StatusCode::OK);
    let profile_body = to_bytes(profile_response.into_body(), usize::MAX)
        .await
        .expect("profile body");
    let profile_json: serde_json::Value = serde_json::from_slice(&profile_body).expect("profile");
    let profile_data = profile_json.get("data").expect("profile data");
    assert!(
        profile_data
            .get("reputation")
            .and_then(|value| value.get("user_id"))
            .is_some()
    );
    assert!(
        profile_data
            .get("reputation")
            .and_then(|value| value.get("tier"))
            .is_some()
    );
    assert!(
        profile_data
            .get("tier")
            .and_then(|value| value.get("tier_symbol"))
            .is_some()
    );
    assert!(
        profile_data
            .get("cv_hidup")
            .and_then(|value| value.get("user_id"))
            .is_some()
    );

    let skills_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/skills/search?q=cleanup")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("skills request");
    let skills_response = app
        .clone()
        .oneshot(skills_request)
        .await
        .expect("skills response");
    assert_eq!(skills_response.status(), StatusCode::OK);
    let skills_body = to_bytes(skills_response.into_body(), usize::MAX)
        .await
        .expect("skills body");
    let skills_json: serde_json::Value = serde_json::from_slice(&skills_body).expect("skills");
    assert!(
        skills_json
            .get("data")
            .and_then(|value| value.get("results"))
            .and_then(|value| value.as_array())
            .is_some()
    );

    let por_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/por/requirements/delivery")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("por request");
    let por_response = app
        .clone()
        .oneshot(por_request)
        .await
        .expect("por response");
    assert_eq!(por_response.status(), StatusCode::OK);
    let por_body = to_bytes(por_response.into_body(), usize::MAX)
        .await
        .expect("por body");
    let por_json: serde_json::Value = serde_json::from_slice(&por_body).expect("por");
    assert!(
        por_json
            .get("data")
            .and_then(|value| value.get("task_type"))
            .is_some()
    );
    assert!(
        por_json
            .get("data")
            .and_then(|value| value.get("min_media_items"))
            .is_some()
    );

    let triad_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/por/triad-requirements/resolve/seed_to_define")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("triad request");
    let triad_response = app
        .clone()
        .oneshot(triad_request)
        .await
        .expect("triad response");
    assert_eq!(triad_response.status(), StatusCode::OK);
    let triad_body = to_bytes(triad_response.into_body(), usize::MAX)
        .await
        .expect("triad body");
    let triad_json: serde_json::Value = serde_json::from_slice(&triad_body).expect("triad");
    assert!(
        triad_json
            .get("data")
            .and_then(|value| value.get("track"))
            .is_some()
    );
    assert!(
        triad_json
            .get("data")
            .and_then(|value| value.get("stage_transition"))
            .is_some()
    );

    let leaderboard_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/reputation/leaderboard?limit=5")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("leaderboard request");
    let leaderboard_response = app
        .clone()
        .oneshot(leaderboard_request)
        .await
        .expect("leaderboard response");
    assert_eq!(leaderboard_response.status(), StatusCode::OK);
    let leaderboard_body = to_bytes(leaderboard_response.into_body(), usize::MAX)
        .await
        .expect("leaderboard body");
    let leaderboard_json: serde_json::Value =
        serde_json::from_slice(&leaderboard_body).expect("leaderboard");
    assert!(
        leaderboard_json
            .get("data")
            .and_then(|value| value.get("entries"))
            .and_then(|value| value.as_array())
            .is_some()
    );
    assert!(
        leaderboard_json
            .get("data")
            .and_then(|value| value.get("total_users"))
            .is_some()
    );

    let distribution_request = Request::builder()
        .method("GET")
        .uri("/v1/tandang/reputation/distribution")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("distribution request");
    let distribution_response = app
        .clone()
        .oneshot(distribution_request)
        .await
        .expect("distribution response");
    assert_eq!(distribution_response.status(), StatusCode::OK);
    let distribution_body = to_bytes(distribution_response.into_body(), usize::MAX)
        .await
        .expect("distribution body");
    let distribution_json: serde_json::Value =
        serde_json::from_slice(&distribution_body).expect("distribution");
    let distribution_data = distribution_json.get("data").expect("distribution data");
    for field in [
        "keystone",
        "pillar",
        "contributor",
        "novice",
        "shadow",
        "total",
    ] {
        assert!(
            distribution_data.get(field).is_some(),
            "missing field {field} in distribution response"
        );
    }
}

#[tokio::test]
async fn metrics_endpoint_is_exposed() {
    let _ = observability::init_metrics();
    observability::register_markov_integration_error("test_reason");
    let app = test_app();

    let health_request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let health_response = app.clone().oneshot(health_request).await.expect("response");
    assert_eq!(health_response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("GET")
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/plain"))
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body = String::from_utf8(body.to_vec()).expect("metrics body");
    assert!(!body.trim().is_empty());
    assert!(
        body.contains("gotong_api_http_requests_total")
            || body.contains("gotong_api_http_request_duration_seconds")
    );
    assert!(body.contains("gotong_api_markov_integration_errors_total"));
}

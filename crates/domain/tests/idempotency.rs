use std::sync::Arc;
use std::time::Duration;

use gotong_domain::idempotency::{
    BeginOutcome, IdempotencyConfig, IdempotencyService, InMemoryIdempotencyStore,
};
use gotong_domain::ports::idempotency::{IdempotencyKey, IdempotencyResponse};
use serde_json::json;

#[tokio::test]
async fn replay_returns_prior_response() {
    let store = InMemoryIdempotencyStore::new("test");
    let service = IdempotencyService::new(
        Arc::new(store),
        IdempotencyConfig {
            in_progress_ttl: Duration::from_secs(60),
            completed_ttl: Duration::from_secs(60),
        },
    );

    let key = IdempotencyKey::new("echo", "entity-1", "req-1");
    let outcome = service.begin(&key).await.unwrap();
    assert_eq!(outcome, BeginOutcome::Started);

    let response = IdempotencyResponse {
        status_code: 200,
        body: json!({ "message": "hello" }),
    };
    service.complete(&key, response.clone()).await.unwrap();

    let replay = service.begin(&key).await.unwrap();
    assert_eq!(replay, BeginOutcome::Replay(response));
}

#[tokio::test]
async fn in_progress_conflict_is_visible() {
    let store = InMemoryIdempotencyStore::new("test");
    let service = IdempotencyService::new(
        Arc::new(store),
        IdempotencyConfig {
            in_progress_ttl: Duration::from_secs(60),
            completed_ttl: Duration::from_secs(60),
        },
    );

    let key = IdempotencyKey::new("echo", "entity-2", "req-2");
    let outcome = service.begin(&key).await.unwrap();
    assert_eq!(outcome, BeginOutcome::Started);

    let outcome = service.begin(&key).await.unwrap();
    assert_eq!(outcome, BeginOutcome::InProgress);
}

#[tokio::test]
async fn in_progress_expiry_allows_new_start() {
    let store = InMemoryIdempotencyStore::new("test");
    let service = IdempotencyService::new(
        Arc::new(store),
        IdempotencyConfig {
            in_progress_ttl: Duration::from_millis(10),
            completed_ttl: Duration::from_secs(60),
        },
    );

    let key = IdempotencyKey::new("echo", "entity-3", "req-3");
    let outcome = service.begin(&key).await.unwrap();
    assert_eq!(outcome, BeginOutcome::Started);

    tokio::time::sleep(Duration::from_millis(20)).await;

    let outcome = service.begin(&key).await.unwrap();
    assert_eq!(outcome, BeginOutcome::Started);
}

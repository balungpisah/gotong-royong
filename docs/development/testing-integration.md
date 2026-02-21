# Testing Integration

## Stack

All tests use the Rust testing infrastructure:

| Tool | Purpose |
|------|---------|
| `cargo test` | Built-in Rust test runner |
| `tokio::test` | Async test macro (Tokio runtime) |
| `axum-test` or `tower::ServiceExt` | HTTP integration testing |
| `pretty_assertions` | Readable assertion diffs |
| `surrealdb` in-memory | Isolated DB per test suite |
| `mockall` | Mock trait implementations |
| `wiremock` | HTTP mock server (Markov Engine stubs) |

See [ADR-001](../architecture/adr/ADR-001-rust-axum-surrealdb-stack-lock.md) for the full stack decision.

## Test Pyramid

```
         /\
        /  \  E2E Tests (5%)
       /____\
      /      \
     / Integr-\ Integration Tests (25%)
    /  ation  \
   /___________\
  /             \
 /   Unit Tests  \ Unit Tests (70%)
/_________________\
```

- **Unit Tests (70%)**: Fast, isolated, test individual functions and domain logic
- **Integration Tests (25%)**: Test component interactions (API handlers + DB)
- **E2E Tests (5%)**: Test complete user flows against a running stack

## Unit Tests

Unit tests live alongside source code in `#[cfg(test)]` modules.

### Domain Logic

```rust
// crates/domain/src/evidence.rs

pub fn validate_timestamp(timestamp: &DateTime<Utc>) -> Result<(), ValidationError> {
    let age_days = (Utc::now() - *timestamp).num_days();
    if age_days < 0 {
        return Err(ValidationError::FutureTimestamp);
    }
    if age_days > 30 {
        return Err(ValidationError::TimestampTooOld { days: age_days });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_timestamp_older_than_30_days() {
        let old_timestamp = Utc::now() - chrono::Duration::days(45);
        let result = validate_timestamp(&old_timestamp);
        assert!(matches!(result, Err(ValidationError::TimestampTooOld { .. })));
    }

    #[test]
    fn rejects_future_timestamp() {
        let future = Utc::now() + chrono::Duration::hours(1);
        let result = validate_timestamp(&future);
        assert!(matches!(result, Err(ValidationError::FutureTimestamp)));
    }

    #[test]
    fn accepts_recent_timestamp() {
        let recent = Utc::now() - chrono::Duration::days(5);
        assert!(validate_timestamp(&recent).is_ok());
    }
}
```

### HMAC Signature

```rust
// crates/infrastructure/src/webhook/signature.rs

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn compute_signature_is_deterministic() {
        let secret = "test-secret-32-chars-minimum-req";
        let payload = b"test payload";
        let sig1 = compute_hmac_signature(secret, payload);
        let sig2 = compute_hmac_signature(secret, payload);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn verify_accepts_valid_signature() {
        let secret = "test-secret-32-chars-minimum-req";
        let payload = b"test payload";
        let signature = compute_hmac_signature(secret, payload);
        let header = format!("sha256={}", signature);
        assert!(verify_hmac_signature(secret, payload, &header).is_ok());
    }

    #[test]
    fn verify_rejects_tampered_payload() {
        let secret = "test-secret-32-chars-minimum-req";
        let original = b"original payload";
        let signature = compute_hmac_signature(secret, original);
        let header = format!("sha256={}", signature);
        let tampered = b"tampered payload";
        assert!(verify_hmac_signature(secret, tampered, &header).is_err());
    }
}
```

## Integration Tests

Integration tests live in `crates/*/tests/` and use a real SurrealDB in-memory instance.

### Test App Setup

```rust
// tests/common/mod.rs

pub async fn test_app() -> TestApp {
    // Start in-memory SurrealDB
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    crate::infrastructure::db::run_migrations(&db).await.unwrap();

    // Start Redis (test instance via testcontainers or mock)
    let redis = redis::Client::open("redis://127.0.0.1/15").unwrap();

    let state = AppState { db, redis };
    let router = crate::api::router(state);

    TestApp { router }
}

pub struct TestApp {
    router: Router,
}

impl TestApp {
    pub async fn post(&self, path: &str) -> TestRequest {
        // ... axum-test helper
    }
}
```

### API Handler Tests

```rust
// crates/api/tests/tasks_test.rs

#[tokio::test]
async fn create_task_returns_201() {
    let app = test_app().await;
    let token = app.login_test_user().await;

    let response = app
        .post("/api/tasks")
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "title": "Fix pothole on Jl. Merdeka",
            "description": "Large pothole near the market"
        }))
        .await;

    assert_eq!(response.status(), 201);
    let body: Task = response.json().await;
    assert!(!body.id.to_raw().is_empty());
    assert_eq!(body.title, "Fix pothole on Jl. Merdeka");
}

#[tokio::test]
async fn create_task_requires_auth() {
    let app = test_app().await;

    let response = app
        .post("/api/tasks")
        .json(&serde_json::json!({"title": "No auth task"}))
        .await;

    assert_eq!(response.status(), 401);
}
```

### Evidence Validation Tests

```rust
// crates/api/tests/evidence_test.rs

#[tokio::test]
async fn evidence_upload_requires_valid_timestamp() {
    let app = test_app().await;
    let token = app.login_test_user().await;

    let old_timestamp = Utc::now() - chrono::Duration::days(45);

    let response = app
        .post("/api/evidence")
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "task_id": "task:test123",
            "evidence_type": "photo",
            "timestamp": old_timestamp.to_rfc3339()
        }))
        .await;

    assert_eq!(response.status(), 422);
    let body: ErrorResponse = response.json().await;
    assert_eq!(body.code, "TIMESTAMP_TOO_OLD");
}
```

## Webhook Integration Tests

Test the full webhook delivery pipeline against a mock Markov server.

```rust
// crates/infrastructure/tests/webhook_delivery_test.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header_exists};

#[tokio::test]
async fn delivers_contribution_created_event() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/platforms/gotong_royong/webhook"))
        .and(header_exists("X-GR-Signature"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "processed": 1
        })))
        .mount(&mock_server)
        .await;

    let client = WebhookClient::new(
        mock_server.uri(),
        "test-webhook-secret-32-chars-minimum",
    );

    let event = ContributionCreatedEvent {
        event_id: "evt_test123".to_string(),
        actor: Actor { user_id: "user:alice".to_string(), username: "alice".to_string() },
        // ...
    };

    let result = client.deliver(event).await;
    assert!(result.is_ok());

    // Verify the mock received exactly one request
    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);

    // Verify HMAC header was present
    let signature = received[0].headers.get("X-GR-Signature").unwrap();
    assert!(signature.to_str().unwrap().starts_with("sha256="));
}

#[tokio::test]
async fn retries_on_500_with_exponential_backoff() {
    let mock_server = MockServer::start().await;

    // First two attempts fail, third succeeds
    Mock::given(method("POST"))
        .and(path("/api/v1/platforms/gotong_royong/webhook"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/platforms/gotong_royong/webhook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"processed": 1})))
        .mount(&mock_server)
        .await;

    let client = WebhookClient::new_with_config(
        mock_server.uri(),
        "test-webhook-secret-32-chars-minimum",
        RetryConfig { max_attempts: 5, base_delay_ms: 10 }, // fast for tests
    );

    let result = client.deliver(test_event()).await;
    assert!(result.is_ok());

    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 3); // 2 failures + 1 success
}
```

## SurrealDB Integration Tests

```rust
// crates/infrastructure/tests/db_test.rs

#[tokio::test]
async fn stores_and_retrieves_task() {
    let db = test_db().await;
    let repo = TaskRepository::new(db);

    let task = Task {
        id: Thing::from(("task", Id::ulid())),
        title: "Test task".to_string(),
        status: TaskStatus::Open,
        created_at: Utc::now(),
        ..Default::default()
    };

    repo.insert(&task).await.unwrap();

    let retrieved = repo.find_by_id(&task.id).await.unwrap();
    assert_eq!(retrieved.title, "Test task");
    assert_eq!(retrieved.status, TaskStatus::Open);
}

#[tokio::test]
async fn lists_tasks_by_community() {
    let db = test_db().await;
    let repo = TaskRepository::new(db);

    // Insert tasks for two communities
    for i in 0..3 {
        repo.insert(&test_task("community:a", &format!("Task {i}"))).await.unwrap();
    }
    repo.insert(&test_task("community:b", "Other task")).await.unwrap();

    let tasks = repo.list_by_community("community:a", 10, 0).await.unwrap();
    assert_eq!(tasks.len(), 3);
}
```

## Running Tests

### Commands

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p gotong-api

# Run a specific test
cargo test test_create_task -- --nocapture

# Run tests with detailed output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'

# Run via just
just test          # all tests
just test-api      # API crate only
just test-infra    # infrastructure crate only
```

### Test Isolation

Each test that touches SurrealDB should use a fresh in-memory instance:

```rust
async fn test_db() -> Surreal<Mem> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db(ulid::Ulid::new().to_string()).await.unwrap();
    run_migrations(&db).await.unwrap();
    db
}
```

Using a unique DB name (`Ulid::new()`) ensures full isolation between concurrent tests.

### CI Pipeline

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test --workspace --all-features
  env:
    RUST_LOG: error
    DATABASE_URL: "memory"
    REDIS_URL: "redis://localhost:6379/15"
```

## E2E Tests

E2E tests run against a fully assembled Docker Compose stack. They are slower and run only in CI on pull requests to `main`.

```bash
# Start full stack
docker-compose -f docker-compose.e2e.yml up -d

# Run E2E tests
cargo test --test e2e

# Teardown
docker-compose -f docker-compose.e2e.yml down -v
```

Key E2E scenarios:
1. User registers → completes task → submits evidence → webhook delivered to Markov mock
2. Peer vouches for contribution → vouch webhook delivered
3. Evidence rejected (age > 30 days) → no webhook sent

## Test Coverage

```bash
# Install cargo-tarpaulin (coverage tool)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html

# Open report
open tarpaulin-report.html
```

**Targets**:
- Domain logic: ≥ 90% coverage
- Infrastructure: ≥ 80% coverage
- API handlers: ≥ 75% coverage

## References

- [Setup Guide](setup-guide.md) — Initial environment setup
- [Local Development](local-development.md) — Day-to-day workflow
- [Webhook Spec](../api/webhook-spec.md) — Webhook protocol details
- [Validation Rules](../por-evidence/validation-rules.md) — Evidence validation spec

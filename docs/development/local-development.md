# Local Development

## Stack

Canonical implementation stack (locked — see [ADR-001](../architecture/adr/ADR-001-rust-axum-surrealdb-stack-lock.md)):

| Component | Technology |
|-----------|-----------|
| Backend | Rust 2024 edition, MSRV 1.88.0 |
| HTTP | Axum 0.7 + Tokio |
| Database | SurrealDB `=3.0.0` (in-memory in dev) |
| Cache | Redis 7+ |
| Object Storage | MinIO (local S3-compatible) |
| Frontend | SvelteKit 2 + Svelte 5 runes + Tailwind + Bun |
| Task Runner | `just` (Justfile at repo root) |

## Development Workflow

### Daily Workflow

```
1. Pull latest changes
   ↓
2. Create feature branch
   ↓
3. Start services (docker-compose up -d)
   ↓
4. Make changes (cargo watch for API, bun dev for frontend)
   ↓
5. Run tests (just test)
   ↓
6. Commit changes
   ↓
7. Push → Pull request
   ↓
8. Code review + merge
```

### Starting Your Day

```bash
# 1. Pull latest changes
git checkout main
git pull origin main

# 2. Create feature branch
git checkout -b feature/add-evidence-validation

# 3. Start backing services
docker-compose up -d  # SurrealDB, Redis, MinIO

# 4a. Start API (hot-reload)
cargo watch -x run

# 4b. Start frontend (separate terminal)
cd apps/web && bun dev
```

## Project Structure

```
gotong-royong/
├── crates/
│   ├── api/               # Axum route handlers
│   │   ├── src/
│   │   │   ├── routes/    # REST endpoints (tasks, evidence, users)
│   │   │   ├── middleware/ # JWT auth, rate limiting, tracing
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── domain/            # Business logic (no I/O dependencies)
│   │   └── src/
│   │       ├── task.rs
│   │       ├── evidence.rs
│   │       └── user.rs
│   ├── infrastructure/    # SurrealDB, Redis, S3, webhook client
│   │   └── src/
│   │       ├── db/
│   │       ├── cache/
│   │       ├── storage/
│   │       └── webhook/
│   └── ai/               # AI touch point integrations
├── apps/
│   └── web/              # SvelteKit frontend
│       ├── src/
│       ├── package.json
│       └── bun.lockb
├── tests/                # Integration tests
├── docs/                 # Documentation
├── Justfile              # Task runner
├── Cargo.toml            # Workspace manifest
├── docker-compose.yml
└── .env.example
```

## Naming Conventions

**Rust**:
- `snake_case` for modules, functions, variables: `user_service`, `create_task`, `webhook_secret`
- `PascalCase` for types and traits: `UserService`, `TaskRepository`, `WebhookEvent`
- `SCREAMING_SNAKE_CASE` for constants: `MAX_FILE_SIZE`, `JWT_EXPIRY_SECS`

**SurrealDB**:
- `snake_case` for table names: `users`, `task_assignments`, `evidence`
- `snake_case` for fields: `user_id`, `created_at`, `media_hash`
- Record IDs: `user:ulid_here`, `task:ulid_here`

**Frontend (Svelte/TypeScript)**:
- `camelCase` for variables: `userId`, `webhookSecret`
- `PascalCase` for Svelte components: `EvidenceCard.svelte`, `TaskFeed.svelte`
- `kebab-case` for files: `evidence-card.svelte`, `task-feed.svelte`

## Development Practices

### Code Style

**Backend** — use `rustfmt` + `clippy`:
```bash
cargo fmt
cargo clippy -- -D warnings
```

**Frontend** — use `prettier` + `eslint` via bun:
```bash
cd apps/web
bun run lint
bun run format
```

### Git Commit Messages

**Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples**:
```
feat(evidence): add presigned S3 upload endpoint

Implements presigned URL generation for direct S3 uploads.
Includes validation for file type and size limits.

Closes #123
```

### Branch Strategy

- `main`: Production-ready code (protected)
- `feature/<description>`: New features (e.g., `feature/add-gps-validation`)
- `fix/<description>`: Bug fixes (e.g., `fix/webhook-retry-logic`)
- `hotfix/<description>`: Critical production patches

## Debugging

### Rust / Axum Debugging

**Enable detailed tracing**:
```bash
RUST_LOG=debug cargo run
# Or for specific modules:
RUST_LOG=gotong_api=debug,gotong_infrastructure=trace cargo run
```

**VS Code launch configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug API",
      "cargo": {
        "args": ["build", "--bin=gotong-api"],
        "filter": { "name": "gotong-api", "kind": "bin" }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "envFile": "${workspaceFolder}/.env.local"
    }
  ]
}
```

**LLDB in terminal**:
```bash
rust-lldb target/debug/gotong-api
```

### HTTP Debugging

**Using curl**:
```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"SecurePassword123!"}' \
  | jq -r '.access_token')

# Get tasks (authenticated)
curl http://localhost:8080/api/tasks \
  -H "Authorization: Bearer $TOKEN"

# Submit evidence
curl -X POST http://localhost:8080/api/evidence \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task:abc123","evidence_type":"photo"}'
```

### SurrealDB Debugging

**Open SurrealDB console**:
```bash
docker-compose exec surrealdb surreal sql \
  --conn http://localhost:8000 \
  --user root --pass root \
  --ns gotong --db dev
```

**Useful SurrealQL queries**:
```sql
-- List all tasks
SELECT * FROM task ORDER BY created_at DESC LIMIT 10;

-- Count by status
SELECT status, count() FROM task GROUP BY status;

-- Recent evidence
SELECT * FROM evidence WHERE created_at > time::now() - 1d;
```

**Enable SurrealDB query logging** (dev only):
```bash
# In docker-compose.yml, add to surreal service:
command: start --log trace --user root --pass root memory
```

### Redis Debugging

```bash
# Connect to Redis CLI
docker-compose exec redis redis-cli

# Inspect webhook DLQ
KEYS gotong:webhook:dlq:*
LRANGE gotong:webhook:dlq:contribution_created 0 -1

# Clear all (dev only)
FLUSHALL
```

## Common Tasks

### Adding a New API Endpoint

**1. Define the route handler** (`crates/api/src/routes/tasks.rs`):
```rust
use axum::{extract::State, Json};
use crate::AppState;

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let task = state.task_service.create(payload).await?;
    Ok(Json(task))
}
```

**2. Register the route** (`crates/api/src/main.rs`):
```rust
Router::new()
    .route("/api/tasks", post(routes::tasks::create_task))
    .route("/api/tasks/:id", get(routes::tasks::get_task))
    .with_state(app_state)
```

**3. Implement the domain logic** (`crates/domain/src/task.rs`):
```rust
pub async fn create(
    &self,
    payload: CreateTaskRequest,
) -> Result<Task, DomainError> {
    self.validate(&payload)?;
    self.repo.insert(Task::from(payload)).await
}
```

**4. Write the test** (`crates/api/tests/tasks_test.rs`):
```rust
#[tokio::test]
async fn test_create_task() {
    let app = test_app().await;
    let response = app
        .post("/api/tasks")
        .json(&json!({"title": "Fix road", "description": "Pothole on Jl. Merdeka"}))
        .await;
    assert_eq!(response.status(), 201);
}
```

### Adding a SurrealDB Migration

SurrealDB uses schema-on-write; migrations are SurrealQL scripts:

**1. Create migration file** (`crates/infrastructure/migrations/0007_add_location_to_tasks.surql`):
```sql
DEFINE FIELD location ON TABLE task TYPE option<geometry<point>>;
DEFINE INDEX task_location ON TABLE task COLUMNS location MTREE DIMENSION 2;
```

**2. Run migration**:
```bash
just migrate
# or
cargo run --bin migrate
```

### Adding Environment Variables

**1. Add to `.env.example`**:
```bash
# Feature flags
ENABLE_GPS_VERIFICATION=true
```

**2. Add to `Config` struct** (`crates/api/src/config.rs`):
```rust
pub struct Config {
    #[serde(default = "defaults::enable_gps_verification")]
    pub enable_gps_verification: bool,
}
```

**3. Load via `envy`** (already wired in startup):
```rust
let config = envy::from_env::<Config>()?;
```

### Running Background Tasks Locally

Background tasks run as Tokio spawned tasks, no separate process needed:

```rust
// In startup (crates/api/src/main.rs)
tokio::spawn(workers::webhook_delivery::run(state.clone()));
tokio::spawn(workers::evidence_hash::run(state.clone()));
```

**Monitor worker logs**:
```bash
RUST_LOG=gotong_infrastructure::workers=debug cargo run
```

## Hot Reloading

**API (cargo-watch)**:
```bash
cargo install cargo-watch
cargo watch -x run
# With specific log level:
RUST_LOG=debug cargo watch -x run
```

**Frontend**:
```bash
cd apps/web && bun dev
# HMR is enabled by default in SvelteKit dev server
```

## Performance Profiling

**CPU profiling with `flamegraph`**:
```bash
cargo install flamegraph
cargo flamegraph --bin gotong-api
# Opens flamegraph.svg
```

**Heap allocation profiling with `dhat`**:
```bash
# Add to Cargo.toml: dhat = { version = "0.3", features = ["ad-hoc"] }
cargo run --features dhat-heap --bin gotong-api
```

**Benchmark with `criterion`**:
```bash
cargo bench
```

## Troubleshooting

### Issue: Port already in use

```bash
# Find process on port 8080
lsof -i :8080
kill -9 <PID>
```

### Issue: SurrealDB connection error

```bash
# Check service is running
docker-compose ps

# Restart SurrealDB
docker-compose restart surrealdb

# Check connectivity
curl -s http://localhost:8000/health | jq
```

### Issue: Tests failing

```bash
# Reset test database (in-memory, just restart)
docker-compose restart surrealdb

# Clear build cache
cargo clean
cargo test

# Run specific test with output
cargo test test_create_task -- --nocapture
```

### Issue: Bun frontend dependency error

```bash
cd apps/web
rm -rf node_modules bun.lockb
bun install
```

## Code Review Checklist

**Before creating PR**:
- [ ] `cargo fmt` passes (no formatting changes)
- [ ] `cargo clippy -- -D warnings` passes (no warnings)
- [ ] `cargo test` passes
- [ ] `bun run lint` passes (frontend)
- [ ] No `unwrap()` / `expect()` in production paths (use `?` operator)
- [ ] Error types are descriptive (use `thiserror` or `error-stack`)
- [ ] Environment variables documented in `.env.example`
- [ ] SurrealDB migrations are in `crates/infrastructure/migrations/`

**Reviewing PR**:
- [ ] Rust idioms respected (ownership, lifetimes, no unnecessary clones)
- [ ] Tests cover new functionality
- [ ] No security vulnerabilities (injection, auth bypass)
- [ ] SurrealDB queries use prepared statements / parameterized bindings
- [ ] API changes reflected in api/ docs

## References

- [Setup Guide](setup-guide.md) — Initial setup
- [Testing Integration](testing-integration.md) — Testing practices
- [System Overview](../architecture/system-overview.md) — System design
- [Database Schema](../database/schema-requirements.md) — Database design
- [Webhook Spec](../api/webhook-spec.md) — Webhook protocol
- [ADR-001](../architecture/adr/ADR-001-rust-axum-surrealdb-stack-lock.md) — Stack decision record

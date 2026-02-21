# Backend Research Notes (Rust 2024)

Date: 2026-02-15

## Scope
General research for building a modern Rust backend using the Rust 2024 edition. This focuses on mature, widely used crates, common architectural patterns, and the knowledge to prepare before implementation.

## Final Stack Lock (Plan Basis)
- **Language/Edition**: Rust `2024` (MSRV `1.88.0`).
- **Runtime + HTTP**: Tokio + Axum + Tower + tower-http.
- **Primary database**: SurrealDB server `v3.0.0-beta-4` with Rust SDK 3 beta channel.
- **Realtime delivery**: WebSocket primary, SSE fallback, polling fallback for degraded clients.
- **Storage**: S3-compatible object storage for evidence payloads.
- **Cache/ephemeral state**: Redis for idempotency cache, rate controls, and short-lived fanout metadata.
- **Observability**: tracing + OpenTelemetry.
- **Auth**: JWT for API auth + HMAC-SHA256 for webhook signatures.

This lock is the canonical basis for implementation planning. SQLx/Diesel/SeaORM remain reference alternatives only and are deferred.

## Rust 2024 Status
- Rust 1.88.0 stabilized the Rust 2024 edition baseline for this codebase (toolchain pinned).
- New Cargo projects default to `edition = "2024"` in `Cargo.toml`.
- Cargo 1.88+ supports this pinned stack.

Sources:
- https://doc.rust-lang.org/edition-guide/editions/creating-a-new-project.html
- https://doc.rust-lang.org/cargo/CHANGELOG.html

## Core Ecosystem Building Blocks

### Async Runtime
- **Tokio**: Async runtime with scheduler, I/O driver, and timers. `#[tokio::main]` starts a runtime automatically.

Sources:
- https://docs.rs/tokio/latest/tokio/runtime/
- https://tokio.rs/tokio/tutorial/hello-tokio

### HTTP Foundation
- **Hyper**: Low-level, efficient HTTP library (client + server). Frameworks like Axum build on it.

Source:
- https://docs.rs/crate/hyper/latest

### Web Frameworks
- **Axum**: Ergonomic and modular web framework, uses Tower middleware, designed to work with Tokio + Hyper.
- **Actix Web**: Pragmatic, high-performance framework with routing macros and rich features.

Sources:
- https://docs.rs/axum/latest
- https://docs.rs/actix-web/

### Middleware & Cross-Cutting Concerns
- **Tower**: Provides `Service` and `Layer` abstractions for composable middleware.
- **tower-http**: HTTP-specific middleware built on Tower and compatible with Hyper/Tonic/Warp.

Sources:
- https://docs.rs/tower
- https://docs.rs/tower-http/latest/tower_http/

### Serialization
- **Serde**: Standard serialization/deserialization framework using `Serialize` / `Deserialize` traits.

Source:
- https://docs.rs/serde

### Database Access Options
- **SQLx**: Async SQL toolkit with optional compile-time checked queries and support for Postgres/MySQL/SQLite.
- **Diesel**: ORM + query builder with compile-time query validation using schema definitions.
- **SeaORM**: Async ORM designed for web services.
- **SurrealDB**: Database with live queries for real-time updates and a Rust SDK; supports embedded, self-hosted, and cloud deployments.

Sources:
- https://docs.rs/sqlx/latest/sqlx/
- https://github.com/launchbadge/sqlx
- https://docs.diesel.rs/main/diesel/index.html
- https://docs.rs/sea-orm/latest/sea_orm/index.html
- https://surrealdb.com/docs/sdk/rust
- https://surrealdb.com/docs/surrealdb/introduction/architecture

### SurrealDB Version Snapshot (as of 2026-02-15)
- **Latest beta listed**: v3.0.0-beta-4 (released Feb 10, 2026).
- **Latest stable listed**: v2.6.1.
- **No v3.0.0 stable listing yet** on the official releases page.
- **Team expectation**: v3.0.0 stable may land around **Feb 17, 2026** (unverified).

### SurrealDB Adoption Stance (Team)
- **Decision**: Proceed with **v3 beta now** and treat it as beta in production design.
- **Risk posture**: Assume breaking changes before stable; isolate SurrealDB behind an adapter/repository layer.
- **Versioning**: Pin exact server + SDK versions; avoid floating tags.
- **Verification plan**: Re-check on **Feb 17, 2026** for stable release and upgrade if available.

### SurrealDB Beta Hardening Checklist
- **Pin versions**: Lock server and SDK versions (no floating tags) in deploy manifests and `Cargo.toml`.
- **Isolation layer**: Keep SurrealDB behind a repository/adapter boundary to contain API and type changes.
- **Contract tests**: Add integration tests for `LIVE SELECT` payload shapes (`DEFAULT`, `DIFF`, `VALUE`) and ordering expectations.
- **Permission tests**: Verify live query filtering by table/row/field permissions for each auth role.
- **Backfill strategy**: Live queries have no historical backfill; build a catch-up query path on connect/reconnect.
- **Rollback plan**: Maintain a tested downgrade path to the latest stable (v2.6.1) and document data/schema impact.
- **Monitoring**: Alert on live-query disconnects, unexpected payload shape, and latency spikes.
- **Realtime-related fixes in v3 betas**: v3.0.0-beta-2 fixed `LIVE SELECT` not returning UUID and fixed `LIVE DIFF` response structure.

Sources:
- https://surrealdb.com/releases

### SurrealDB Notes (Realtime / Chat)
- **Live queries**: Real-time notifications for creates, updates, and deletes; no historical backfill.
- **Payload shape**: By default, full record on create/update and only record ID on delete; `DIFF` mode uses JSON Patch; `VALUE` can project fields.
- **Session-scoped & permission-filtered**: Notifications are filtered by table/row/field permissions for the authenticated session.
- **Single-node only (for now)**: `LIVE SELECT` is currently supported only in single-node deployments; multi-node support is in progress.
- **Ordering & consistency**: Notifications are published only for committed transactions; ordering is best-effort and can be out of commit order, but commits from the same client remain ordered. Authorization is evaluated per notification.
- **Parameters in live queries**: Supported since v3.0.0-beta.
- **Rust SDK**: Async API, supports live queries and embedded usage; requires Rust >= 1.80.1 and is compatible with SurrealDB `v2.0.0` to `v2.6.1`.
- **Storage engines**: SurrealKV single-node is beta; distributed SurrealKV is in development; RocksDB single-node and TiKV distributed are listed as complete.
- **Engine profile decision**: Local dev uses in-memory engine; staging/production use TiKV. SurrealKV is deferred until stable.
- **License**: Core database code is under BSL 1.1 with a restriction on offering SurrealDB as a commercial DBaaS without an agreement.

Sources:
- https://surrealdb.com/docs/surrealql/statements/live
- https://surrealdb.com/features/realtime-data-sync
- https://surrealdb.com/docs/sdk/rust
- https://surrealdb.com/docs/surrealkv
- https://surrealdb.com/features
- https://surrealdb.com/license

### Rust SDK 3 (beta) highlights
- **`surrealdb-types`**: New shared public value type system decoupled from the core database, so other crates can use types without pulling in the full DB.
- **`SurrealValue` trait**: The main Rust-side change in 3.0; can be derived for serialization/deserialization or implemented manually.
- **`kind!` macro**: Helps map SurrealQL type definitions to Rust implementations for `SurrealValue`.

Sources:
- https://surrealdb.com/docs/sdk/rust/concepts/rust-after-30
- https://surrealdb.com/releases

### Release Channels (Server + Rust SDK)
- **Channels**: Stable, beta, and nightly releases for the server and the Rust SDK (`surrealdb`, `surrealdb-beta`, `surrealdb-nightly` crates).
- **Cadence**: Monthly beta on the second Tuesday; the following month’s second Tuesday promotes beta to stable. Beta features may be reverted before stable.

Sources:
- https://surrealdb.com/blog/introducing-our-new-monthly-release-schedule

### Locked Version Matrix (V0)
| Layer | Locked choice | Policy |
|---|---|---|
| Rust toolchain | `1.88.0` | Pin in `rust-toolchain.toml`; no floating toolchain in CI |
| Edition | `2024` | Mandatory for all new crates |
| Runtime | `tokio` | Pin exact crate version in `Cargo.lock` |
| API framework | `axum` | Pin exact crate version in `Cargo.lock` |
| Middleware | `tower` + `tower-http` | Pin exact crate versions in `Cargo.lock` |
| Database server | `surrealdb:v3.0.0-beta-4` | Pin container image tag; no `latest` |
| Rust DB SDK | `surrealdb-beta` channel (SDK 3 beta) | Pin exact crate release matching server lock |
| Cache | `redis` crate + Redis server | Pin exact crate + image tags |
| Telemetry | `tracing` + `opentelemetry` | Pin exact crate versions in `Cargo.lock` |

### Chat Data Model (SurrealDB)
- **`chat_thread`**: thread metadata (`thread_id`, `scope_id`, `created_by`, `created_at`, `updated_at`, `privacy_level`).
- **`chat_member`**: membership and role (`thread_id`, `user_id`, `role`, `joined_at`, `left_at`, `mute_until`).
- **`chat_message`**: immutable message records (`thread_id`, `message_id`, `author_id`, `body`, `attachments`, `created_at`, `edited_at`, `deleted_at`).
- **`chat_read_cursor`**: per-user read state (`thread_id`, `user_id`, `last_read_message_id`, `last_read_at`).
- **`chat_delivery_event`**: append-only delivery/audit events (`event_id`, `thread_id`, `message_id`, `event_type`, `occurred_at`, `request_id`, `correlation_id`).

Indexes and ordering rules:
- Index by `thread_id + created_at + message_id` for deterministic pagination.
- Index by `user_id + thread_id` for membership/read lookups.
- Use monotonic message IDs (ULID/UUIDv7) to stabilize tie-break ordering.
- Never mutate message order fields after insert; edits update content metadata only.

Live query subscription keys:
- Thread stream: `chat_message` filtered by `thread_id`.
- Presence/member stream: `chat_member` filtered by `thread_id`.
- Read receipt stream: `chat_read_cursor` filtered by `thread_id`.

### Realtime Transport Decision
- **Primary**: WebSocket channel per authenticated user session.
- **Fallback A**: SSE for environments where WS is blocked.
- **Fallback B**: short-interval polling for degraded clients or incident mode.
- **Reconnect behavior**: on reconnect, run catch-up query (`created_at > last_seen_at`) before resubscribing.
- **Delivery contract**: at-least-once delivery to clients; idempotent dedupe by `message_id`/`event_id` on consumer.

### SurrealDB Beta Go/No-Go Gates
- **Gate 1 (contract correctness)**: 100% pass for live query payload-shape tests (`DEFAULT`, `DIFF`, `VALUE`).
- **Gate 2 (ordering safety)**: 0 unresolved ordering regressions in chat timeline tests under concurrent writes.
- **Gate 3 (permission safety)**: 100% pass for row/field permission leakage tests by role.
- **Gate 4 (resilience)**: reconnect catch-up tests pass with no message loss in chaos/network interruption scenarios.
- **Gate 5 (operations)**: dashboards/alerts active for disconnect rate, stream lag, payload decode failures, and fallback activation rate.
- **Rollback trigger**: two consecutive release-candidate runs failing any `P0` gate above or production incident severity `SEV-1` tied to beta behavior.
- **Rollback target**: pinned stable stack (latest stable SurrealDB line) through repository adapter compatibility layer.

### Configuration
- **config**: Layered configuration from env, files, defaults, and overrides with serde deserialization.

Source:
- https://docs.rs/config/latest/config/

### Error Handling
- **anyhow**: Application-focused error type with context and easy propagation.
- **thiserror**: Derive macro for structured error enums/structs.

Sources:
- https://docs.rs/anyhow/
- https://docs.rs/thiserror

### Observability
- **tracing**: Structured, async-aware logging with spans and events.

Source:
- https://docs.rs/tracing/

### Validation
- **validator**: Derive-based validation for request/input structs (email, url, length, range, custom, etc).

Source:
- https://docs.rs/validator

### Auth & Security
- **jsonwebtoken**: Encode/decode JWTs for auth.
- **argon2**: Password hashing using Argon2 (PHC winner, Argon2id default).

Sources:
- https://docs.rs/jsonwebtoken
- https://docs.rs/argon2

### HTTP Client
- **reqwest**: High-level HTTP client, async and blocking APIs; async client requires Tokio.

Source:
- https://docs.rs/reqwest/

### IDs
- **uuid**: Generate/parse UUIDs (v4 for randomness, v7 for time-ordered IDs).

Source:
- https://docs.rs/uuid

### Optional: API Docs & Telemetry
- **utoipa**: OpenAPI generation via macros for Rust REST APIs.
- **opentelemetry** + **tracing-opentelemetry**: Export traces to OTel systems.
- **metrics-exporter-prometheus** or **prometheus**: Metrics export for Prometheus.

Sources:
- https://docs.rs/utoipa
- https://docs.rs/opentelemetry
- https://docs.rs/tracing-opentelemetry
- https://docs.rs/metrics-exporter-prometheus/
- https://docs.rs/prometheus

## Architecture Patterns (Inferred from Ecosystem Design)

Note: These patterns are inferred from how the core crates are designed and composed.

- **Middleware pipeline**: Compose cross-cutting concerns (timeouts, tracing, auth, compression) as Tower `Layer`s around application handlers. This matches Axum’s Tower-based design and tower-http’s middleware library.
- **Clear boundary types**: Use Serde DTOs at the HTTP boundary and validate with `validator` before mapping into domain types.
- **Error taxonomy**: Use `thiserror` for domain/library error enums and `anyhow` at the application boundary with context.
- **Observability-first**: Model request spans and events with `tracing` to keep async flow debuggable.
- **Database contract upfront**: Standardize SurrealDB access through a repository adapter boundary, with stable query patterns, event writes, and rollback compatibility rules.

Relevant sources:
- https://docs.rs/tower
- https://docs.rs/tower-http/latest/tower_http/
- https://docs.rs/axum/latest
- https://docs.rs/serde
- https://docs.rs/validator
- https://docs.rs/thiserror
- https://docs.rs/anyhow/
- https://docs.rs/tracing/
- https://docs.rs/sqlx/latest/sqlx/
- https://github.com/launchbadge/sqlx
- https://docs.diesel.rs/main/diesel/index.html
- https://docs.rs/sea-orm/latest/sea_orm/index.html

## Knowledge To Prepare Before Implementation

- **Async fundamentals**: Tokio runtime, tasks, and async I/O behavior.
- **HTTP stack**: Hyper basics plus Axum routing and Tower middleware composition.
- **Middleware composition**: Tower `Service`/`Layer` model and tower-http utilities.
- **Serialization and validation**: Serde for DTOs and validator for inputs.
- **Database access**: SurrealDB SDK patterns, record modeling, live query semantics, and adapter-based isolation for beta/stable transitions.
- **SurrealDB (if chosen)**: Live query semantics (payload, diff, ordering), single-node limitation, permission filtering, SDK 3 type system changes, release channel strategy, embedded vs remote deployment, storage engine selection, and BSL licensing constraints.
- **Error handling**: thiserror and anyhow patterns for error propagation and context.
- **Observability**: tracing spans/events and optional OpenTelemetry export.
- **Security**: JWT handling and Argon2 password hashing best practices.
- **Configuration**: Layered config with env + file overrides.
- **External calls**: reqwest for HTTP clients.

Sources:
- https://docs.rs/tokio/latest/tokio/runtime/
- https://tokio.rs/tokio/tutorial/hello-tokio
- https://docs.rs/crate/hyper/latest
- https://docs.rs/axum/latest
- https://docs.rs/actix-web/
- https://docs.rs/tower
- https://docs.rs/tower-http/latest/tower_http/
- https://docs.rs/serde
- https://docs.rs/validator
- https://docs.rs/sqlx/latest/sqlx/
- https://github.com/launchbadge/sqlx
- https://docs.diesel.rs/main/diesel/index.html
- https://docs.rs/sea-orm/latest/sea_orm/index.html
- https://surrealdb.com/docs/sdk/rust
- https://surrealdb.com/docs/surrealkv
- https://surrealdb.com/docs/surrealql/statements/live
- https://surrealdb.com/features
- https://surrealdb.com/license
- https://surrealdb.com/docs/surrealdb/introduction/architecture
- https://surrealdb.com/releases
- https://surrealdb.com/features/realtime-data-sync
- https://surrealdb.com/docs/sdk/rust/concepts/rust-after-30
- https://surrealdb.com/blog/introducing-our-new-monthly-release-schedule
- https://docs.rs/thiserror
- https://docs.rs/anyhow/
- https://docs.rs/tracing/
- https://docs.rs/opentelemetry
- https://docs.rs/tracing-opentelemetry
- https://docs.rs/jsonwebtoken
- https://docs.rs/argon2
- https://docs.rs/config/latest/config/
- https://docs.rs/reqwest/

## Locked Baseline (Final)
- Runtime: Tokio
- HTTP: Hyper
- Framework: Axum
- Middleware: Tower + tower-http
- Database: SurrealDB `v3.0.0-beta-4` with Rust SDK 3 beta channel
- Realtime transport: WebSocket primary, SSE fallback, polling fallback
- Cache/ephemeral: Redis
- Object storage: S3-compatible
- Observability: tracing + OpenTelemetry

Deferred alternatives:
- Actix, SQLx, Diesel, and SeaORM remain documented as alternatives but are not part of the implementation baseline.

## Implementation Planning Readiness Audit (2026-02-15, Closed)

### Summary
- **Overall assessment**: `complete`
- **Readiness verdict**: stack research is now complete enough to begin implementation planning.

### Gap Closure Record
- `SR-G01` Stack inconsistency: **closed**
  - Alignment completed in `README.md`, `docs/research/rust-backend-stack-research.md`, and `docs/research/backend-implementation-tickets.md`.
- `SR-G02` Framework/data-layer not finalized: **closed**
  - Canonical execution path locked to Rust 2024 + Axum + SurrealDB beta.
- `SR-G03` Missing pinned version manifest: **closed**
  - Locked version matrix added in this document.
- `SR-G04` Missing chat data model: **closed**
  - Chat schema, index strategy, and subscription keys added in this document.
- `SR-G05` Missing realtime transport decision: **closed**
  - Transport strategy (WS/SSE/polling + reconnect catch-up) added in this document.
- `SR-G06` Missing beta go/no-go gates: **closed**
  - Measurable release gates and rollback triggers added in this document.

### Final Decision
Planning status is **ready**. The next artifact should be the implementation plan derived from this locked baseline.

## Empirical Sampling Proof (SurrealDB)
- **Sampling report (local baseline)**: `docs/research/surrealdb-pattern-sampling.md`
- **Sampling report (locked target runtime)**: `docs/research/surrealdb-pattern-sampling-v3-beta4.md`
- **Probe script**: `docs/research/samples/surrealdb/pattern_probe.sh`
- **Patterns verified in runnable probe**:
  - idempotent write guard via unique `(entity_id, request_id)` index,
  - deterministic timeline ordering (`ORDER BY created_at, message_id`),
  - reconnect catch-up query using `(created_at, message_id)` cursor,
  - live query behavior (WS streaming event observed; HTTP live query not supported in this local setup),
  - `LIVE SELECT DIFF` payload contract on update events,
  - permission-filtered live subscriptions using record-auth tokens.
- **Result**: all sampled checks passed on both local baseline (`2.3.10`) and locked target runtime (`3.0.0-beta.4`).
- **Protocol note**: live query streaming is observed on `ws://` and not on `http://` in both runs.

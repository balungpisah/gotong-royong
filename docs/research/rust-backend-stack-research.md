# Rust Backend Stack Research Report

Date: 2026-02-15
Scope: finalized crates, architecture patterns, and implementation readiness baseline for Gotong backend.
Status: Stack finalized for implementation planning.

**Executive Summary**
The backend stack is locked to Rust 2024 with `tokio` + `axum` + `tower`/`tower-http` and SurrealDB SDK 3 beta against SurrealDB server `v3.0.0-beta-4`. Realtime chat/event delivery uses WebSocket as primary transport with SSE and polling fallback. The architecture keeps a repository/adapter boundary around SurrealDB to manage beta-to-stable migration risk.

**Finalized Baseline**
- Runtime: `tokio`
- API framework: `axum`
- Middleware: `tower`, `tower-http`
- Serialization: `serde`, `serde_json`
- Validation: `validator`, `jsonschema`
- Database: `surrealdb-beta` (SDK 3 beta channel) + SurrealDB server `v3.0.0-beta-4`
- API docs: `utoipa`
- Observability: `tracing`, `opentelemetry`
- IDs: `uuid` (v7 for ordered IDs where needed)
- Cache/ephemeral state: `redis`

**Why this path**
- Aligns with Rust 2024 readiness and async-first backend architecture.
- Matches chat-heavy realtime requirements through live queries plus application-level streaming.
- Preserves migration flexibility by isolating DB access behind repository interfaces.

**Implementation Patterns (Locked)**
- Adapter/repository boundary for all SurrealDB access.
- Idempotency store keyed by `(entity_id, request_id)`.
- Append-only event log for transitions and audits.
- Outbox/event-forwarding pattern for external delivery.
- Reconnect catch-up query before resubscribing realtime streams.
- Strict schema validation for Edge-Pod boundary contracts.

**Known Risks and Controls**
- Risk: SurrealDB/SDK 3 beta behavior changes.
  Control: version pinning, compatibility adapter, release gates, rollback target.
- Risk: realtime ordering drift under concurrency.
  Control: deterministic message ordering (`created_at` + monotonic ID) and contract tests.
- Risk: permission leakage on live streams.
  Control: role-based integration tests on every release candidate.

**Readiness Result**
This research is complete enough to start implementation planning and ticket sequencing.

**Sources**
- Rust edition tooling and stack lock references as defined by project ADR and CI config (`rustc 1.88.0`, exact tags).
- Tokio docs: https://docs.rs/tokio
- Axum docs: https://docs.rs/axum/latest
- Tower docs: https://docs.rs/tower
- Tower HTTP docs: https://docs.rs/tower-http
- SurrealDB releases: https://surrealdb.com/releases
- SurrealDB live queries: https://surrealdb.com/docs/surrealql/statements/live
- SurrealDB Rust SDK docs: https://surrealdb.com/docs/sdk/rust
- SurrealDB Rust SDK 3 changes: https://surrealdb.com/docs/sdk/rust/concepts/rust-after-30
- SurrealDB release schedule: https://surrealdb.com/blog/introducing-our-new-monthly-release-schedule
- JSON Schema validation crate: https://docs.rs/jsonschema
- Utoipa OpenAPI generator: https://docs.rs/utoipa
- Tracing: https://docs.rs/tracing
- OpenTelemetry Rust: https://docs.rs/opentelemetry
- UUID crate: https://docs.rs/uuid
- Redis client: https://docs.rs/redis

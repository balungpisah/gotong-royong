# Backend Implementation Plan — PR Chunks (Greenfield)

Last updated: `2026-02-15`  
Status: `READY FOR EXECUTION`

## Purpose

Define a mergeable, end-to-end backend implementation sequence for a new codebase, aligned to the locked architecture and existing ticket packet.

Canonical references:
- `docs/research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`
- `docs/backend-research.md`
- `docs/research/backend-implementation-tickets.md`
- `docs/research/surrealdb-pattern-sampling-v3-beta4.md`
- `docs/design/backend-design-contract-gotong-tandang.md`

## Scope

In scope:
- Rust backend bootstrap and architecture boundaries
- SurrealDB schema/migration baseline and repository adapters
- BE tickets `BE-001` through `BE-013`
- Edge-Pod contracts `EP-03`, `EP-05`, `EP-08`, `EP-09`, `EP-11`
- Markov/Tandang webhook integration contract and outbox delivery
- Realtime transport, testing, observability, and beta rollback guardrails

Out of scope:
- Frontend implementation
- Switching off locked stack profile
- SurrealDB multi-node redesign

## Non-Negotiable Constraints

- Stack lock: Rust 2024 + Axum + Tokio + Tower/tower-http + SurrealDB `v3.0.0-beta-4`.
- Every write path carries `request_id` and `correlation_id`.
- Event and transition records are append-only and replay-safe.
- No floating versions in toolchain, crate manifests, or DB image tags.
- Webhook signing and replay safety are required for Markov integration.
- Engine profile: local dev uses in-memory SurrealDB; staging/prod uses TiKV.
- Realtime correctness tests are required: ordering, permission filtering, reconnect catch-up.

## PR Sequence Overview

| PR | Title | Primary tickets | Dependency |
|---|---|---|---|
| PR-01 | Foundation Bootstrap | - | none |
| PR-02 | Surreal Schema + Migration Baseline | - | PR-01 |
| PR-03 | Idempotency Core | `BE-011` | PR-02 |
| PR-04 | Auth + Permissions Baseline | - | PR-03 |
| PR-05 | Worker Runtime + Scheduling Baseline | - | PR-04 |
| PR-06 | Core Contributions Domain | - | PR-05 |
| PR-07 | Track Transition Engine | `BE-001`, `BE-002` | PR-06 |
| PR-08 | Realtime Chat Core | chat model from stack lock | PR-02, PR-03, PR-04 |
| PR-09 | Moderation Services | `BE-003`, `BE-004` | PR-07 |
| PR-10 | Vault Lifecycle | `BE-006`, `BE-007` | PR-05 |
| PR-11 | Siaga Lifecycle | `BE-005` | PR-05 |
| PR-12 | Feed/Search/Notifications | `BE-008`, `BE-009`, `BE-010` | PR-07, PR-09, PR-10, PR-11 |
| PR-13 | Markov Webhook Integration + Outbox | - | PR-06, PR-05 |
| PR-14 | Edge-Pod Integrations | `EP-03`, `EP-05`, `EP-08`, `EP-09`, `EP-11` | PR-03, PR-07, PR-09, PR-12 |
| PR-15 | Audit + Observability + Release Gates | `BE-012`, `BE-013` | PR-14 |

## PR Lock State

- `PR-01` to `PR-09`: DONE
- `PR-10`: DONE
- `PR-11` to `PR-15`: NOT STARTED

## Detailed PR Plan

## PR-01 — Foundation Bootstrap

Status:
- DONE (2026-02-15)

Goal:
- Establish project skeleton and engineering guardrails.

Deliverables:
- Rust workspace layout (`api`, `domain`, `infra`, optional `worker`).
- `rust-toolchain.toml` pinned to `1.88.0`.
- Base config loader, startup wiring, and structured error envelope.
- Baseline middleware stack: tracing, request IDs, timeout, auth stub.
- Request validation layer and rate limiting baseline.
- CI pipeline for `fmt`, `clippy`, `test`.

Validation:
- CI green on empty baseline.
- Health endpoint and config bootstrap tests pass.

Exit criteria:
- New contributors can run `cargo build`, `cargo test`, and local startup.

## PR-02 — Surreal Schema + Migration Baseline

Status:
- DONE (2026-02-15)

Goal:
- Establish reproducible database schema/migration flow and adapter isolation.

Deliverables:
- `database/migrations/*.surql` and `database/checks/*.surql`.
- Core tables/indexes from schema requirements.
- Migration runner command path documented and automated in CI.
- Repository/adapter boundary interfaces in Rust.
- Engine profile configuration documented (memory local, TiKV staging/prod).

Validation:
- Migrations apply on ephemeral DB in CI.
- Verification queries pass.

Exit criteria:
- Schema state can be rebuilt from zero via migration scripts only.

## PR-03 — Idempotency Core (`BE-011`)

Status:
- DONE (2026-02-15)

Goal:
- Make write paths replay-safe and deterministic before feature services.

Deliverables:
- Idempotency service keyed by `(entity_id, request_id)`.
- Replay response cache contract.
- Timer/job deterministic `request_id` conventions.
- Redis integration for short-lived de-dup and lock semantics.

Validation:
- Repeated request returns canonical prior result.
- Collision and timeout behavior covered by integration tests.

Exit criteria:
- All command handlers can depend on shared idempotency middleware/service.

## PR-04 — Auth + Permissions Baseline

Status:
- DONE (2026-02-15)

Goal:
- Establish authentication, RBAC, and Surreal permission scaffolding.

Deliverables:
- JWT auth middleware and request context extraction.
- Role model and RBAC policy helpers.
- Surreal table/field/row permission rules for private data.
- Baseline auth test harness for permission-filtered queries.

Validation:
- Protected routes reject unauthenticated requests.
- Permission tests confirm no cross-user leakage on core tables.

Exit criteria:
- All feature services can rely on validated auth context and permission rules.

## PR-05 — Worker Runtime + Scheduling Baseline

Status:
- DONE (2026-02-15)

Goal:
- Provide background job runtime for timers, retries, and digests.

Deliverables:
- Worker binary/crate with shared config and tracing.
- Job queue abstraction and Redis-backed implementation.
- Common job envelope with idempotency and retry semantics.
- Stub jobs for transition close, moderation auto-release, webhook retry, and digests.

Validation:
- Jobs can be enqueued and processed locally.
- Retries follow deterministic backoff and idempotency rules.

Exit criteria:
- Feature services can schedule timers and retries via shared worker runtime.

## PR-06 — Core Contributions Domain

Status:
- DONE (2026-02-15)

Goal:
- Implement core task/evidence/vouch services and PoR storage baseline.

Deliverables:
- Schema and services for contributions, evidence, and vouches.
- PoR evidence storage adapter boundary (in-memory baseline now; S3-compatible implementation to follow in PR-07+).
- Mapping from internal records to Tandang event shapes.
- Basic CRUD endpoints for contribution and evidence flows.

Validation:
- Evidence upload and retrieval pass integrity checks.
- Idempotency on contribution and evidence writes is enforced.

Exit criteria:
- Core domain objects are available for track transitions and webhook publishing.

## PR-07 — Track Transition Engine (`BE-001`, `BE-002`)

Status:
- DONE (2026-02-15)

Implementation progress note:
- PR-07 domain, ports, repository contract, persistence adapter, API endpoints, and timer-close job path are implemented.
- AppState transition repository wiring is available in `crates/api/src/state.rs`.
- `crates/worker/src/main.rs` now executes `JobType::TransitionClose` through `TrackTransitionService`.

Goal:
- Implement canonical governance transition write/read model.

Deliverables:
- `track_state_transition` command path.
- Role matrix + gate prerequisite validation.
- Append-only transition event ledger.
- Transition-actor snapshot captured at request time (immutable actor identity, role, and membership context for deterministic auditability).
- Single atomic command path for gate validation + transition persistence (no cross-service eventual consistency split for command correctness).
- Minimal, append-only transition event ledger with de-duplication keyed by `(entity_id, request_id)`.
- Read-model projections for PR-07 are intentionally minimal: active-stage and timeline queries from the same event stream.
  - Full projection optimization/features can be expanded in PR-12 without breaking compatibility.

Validation:
- Deterministic ordering and immutable history tests pass.
- Replay returns same transition record.
- Snapshot integrity is stable under later role/membership changes.
- `cargo +nightly test --workspace --all-features --quiet` passes.
- Transition close worker path compiles and routes timer jobs through `TrackTransitionService`.

Exit criteria:
- `UI-03` backend contract is executable end-to-end.
- `track_state_transition` writes are safe under retries and idempotent replays.

## PR-08 — Realtime Chat Core

Status:
- DONE (2026-02-15)

Recent note:
- WS primary stream, SSE fallback, and polling endpoints were added on top of existing chat API (`/v1/chat/threads/*/messages/{stream,ws,poll}`), including catch-up from query snapshots.

Goal:
- Deliver chat baseline with stable ordering and reconnect semantics.

Deliverables:
- Thread/member/message/read-cursor repositories and services.
- WS primary stream, SSE fallback, polling fallback.
- Catch-up query flow on reconnect.
- Dedupe and ordering strategy based on `created_at` + `message_id`.
- Idempotent message write guard aligned to `(thread_id, request_id)`.

Validation:
- Live stream tests (`DEFAULT`, `DIFF`, and permission-scoped behavior).
- Reconnect tests show no user-visible message loss.

Exit criteria:
- Chat workload baseline is operational on locked Surreal beta runtime.

## PR-09 — Moderation Services (`BE-003`, `BE-004`)

Status:
- DONE (2026-02-15)

Goal:
- Implement moderation source-of-truth and enforceable policy runtime.

Deliverables:
- Moderation decision store + append-only decision trail.
- Hold/review/publish runtime with auto-release path.
- Role-specific projection logic for public/author/moderator views.

Validation:
- Timeout fallback and manual-review queue behavior covered.
- Data exposure tests prevent private moderation leakage.

Exit criteria:
- `UI-07` moderation behavior is complete and auditable.

## PR-10 — Vault Lifecycle (`BE-006`, `BE-007`)

Status:
- DONE (2026-02-15)

Goal:
- Implement witness vault persistence and lifecycle events.

Deliverables:
- State model: `draft`, `sealed`, `published`, `revoked`, `expired`.
- Trustee (`wali`) management operations.
- Event timeline for witness lifecycle.
- Revoke behavior with payload deletion + metadata retention.

Validation:
- State transition invariants and irreversible states tested.
- Retention and revoke behavior verified.

Exit criteria:
- `UI-09` vault contract works with policy-safe storage behavior.

## PR-11 — Siaga Lifecycle (`BE-005`)

Status:
- NOT STARTED (2026-02-15)

Goal:
- Implement emergency broadcast flow and responder timeline model.

Deliverables:
- Full `siaga_*` event set and lifecycle transitions.
- Responder join/update handling.
- Close/cancel flow with required closure summary.
- Identity anonymization rule wiring (7-day policy).

Validation:
- Lifecycle transitions and timeline ordering tests pass.
- Anonymization scheduler behavior validated.

Exit criteria:
- `UI-08` lifecycle is complete and policy compliant.

## PR-12 — Feed/Search/Notifications (`BE-008`, `BE-009`, `BE-010`)

Status:
- NOT STARTED (2026-02-15)

Goal:
- Deliver derived read services for user-facing activity surfaces.

Deliverables:
- Feed service with opaque cursor pagination and filters.
- Search projection with privacy exclusions and deterministic ranking fallback.
- Notification ingestion, unread counters, and weekly digest assembly.

Validation:
- Cursor determinism and privacy redaction tests.
- Dedupe and embargo handling tests in notification paths.

Exit criteria:
- `UI-10`, `UI-11`, and `UI-12` contracts are functional.

## PR-13 — Markov Webhook Integration + Outbox

Status:
- NOT STARTED (2026-02-15)

Goal:
- Implement outbound-only webhook publishing to Markov/Tandang with replay-safe delivery.

Deliverables:
- Event outbox and delivery log schema.
- HMAC signing for outbound webhook payloads.
- Publisher worker with exponential backoff and idempotency.
- Failure visibility and dead-letter handling.
- Admin/reconciliation endpoints for outbox and dead-letter inspection.
- Outbound queue metrics and audit logs.
Notes:
- Inbound ingestion endpoint is owned by Markov; Gotong only signs outbound payloads.

Validation:
- Signature generation and verification tests.
- Retry behavior on `500`/`503` and rate-limit responses.
- Replayed `event_id` returns no-op behavior.

Exit criteria:
- Markov integration contract is operational and replay-safe.

## PR-14 — Edge-Pod Integrations (`EP-03`, `EP-05`, `EP-08`, `EP-09`, `EP-11`)

Status:
- NOT STARTED (2026-02-15)

Goal:
- Integrate AI endpoint contracts with safe fallback semantics.

Deliverables:
- Five endpoint handlers with strict schema validation.
- Prompt ID/version metadata wiring as per contract docs.
- Non-blocking fallback behavior and warning telemetry.
- Integration hooks into transitions/moderation/notification flows.

Validation:
- Contract tests against endpoint schemas pass.
- AI failure scenarios fall back without blocking core flow.

Exit criteria:
- Edge-Pod integrations are contract-complete and operationally safe.

## PR-15 — Audit, Observability, and Release Gates (`BE-012`, `BE-013`)

Status:
- NOT STARTED (2026-02-15)

Goal:
- Finalize production hardening and beta safety gates.

Deliverables:
- Retention governance matrix and immutable audit event hashing.
- Metrics/tracing dashboards and alert rules.
- Surreal beta go/no-go suite for live payload shape.
- Surreal beta go/no-go suite for ordering.
- Surreal beta go/no-go suite for permission filtering.
- Surreal beta go/no-go suite for reconnect resilience.
- Cross-instance real-time fanout strategy for WS/SSE (Redis or equivalent) with deterministic replay compatibility and permission revalidation.
- Rollback runbook and rehearsal report.

Validation:
- All P0 gates pass in staging.
- Rollback drill succeeds with documented timing and checks.
- Realtime transport can be safely scaled beyond one API instance without dropping stream delivery.

Exit criteria:
- Backend is ready for staged production rollout.

## Cross-PR Test Strategy

- Unit tests for domain invariants and policy rules.
- Integration tests with pinned Surreal runtime (`3.0.0-beta.4`).
- Webhook signing, retry, and idempotency tests for Markov integration.
- Contract tests for Edge-Pod and HTTP envelopes.
- Realtime tests for live stream semantics, reconnect recovery, and permission filtering.
- Scaling tests for multi-instance realtime fanout behavior (ordered replay and stream continuity).
- Smoke suite covering `UI-03`, `UI-07`, `UI-08`, `UI-09`, `UI-10..12`.

## Delivery Notes

- Keep PRs mergeable and independently verifiable.
- Prefer feature flags for incomplete user-facing paths.
- Treat any Surreal beta behavior drift as release-blocking until triaged.

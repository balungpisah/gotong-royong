# ADR-001: Backend Stack Lock for Implementation Planning

Date: 2026-02-15
Status: Accepted
Owner: Backend architecture

## Context
Research identified multiple viable backend stacks, but implementation planning required one canonical path. The product is chat-heavy and needs realtime delivery with strict auditability and idempotency guarantees.

## Decision
We lock the backend stack to:
- Rust 2024 (MSRV 1.88.0)
- Tokio runtime
- Axum framework
- Tower + tower-http middleware
- SurrealDB server `v3.0.0` with Rust SDK `surrealdb` `v3.0.0`
- WebSocket primary realtime transport with SSE and polling fallback
- Redis for ephemeral idempotency/rate/fanout support
- S3-compatible object storage for evidence payloads
- tracing + OpenTelemetry for observability

## Storage Engine Profile (Locked)
- Local development: in-memory engine (file-backed optional).
- Staging/production: TiKV engine for distributed, scalable storage.
- SurrealKV is deferred until stable and is not the production default.

## Rationale
- Keeps async execution model consistent across API and realtime paths.
- Supports chat/event streams while preserving fallback transport behavior.
- Adapter boundary around SurrealDB limits beta risk and enables rollback.

## Consequences
- Team pins SurrealDB v3 runtime + SDK for predictable behavior.
- All implementation tickets should assume this profile unless superseded by a later ADR.
- Alternatives (Actix, SQLx, Diesel, SeaORM) remain reference-only and are deferred.

## Guardrails
- No floating versions for toolchain, crates, or DB image tags.
- Go/no-go gates and rollback triggers must pass before production rollout.
- Realtime contract tests (ordering, permission filtering, reconnect catch-up) are mandatory.

## Supersedes
- Any prior docs marking framework/DB as TBD or language-agnostic for current implementation planning.

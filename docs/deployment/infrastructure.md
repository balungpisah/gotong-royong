# Infrastructure

## Overview

This document defines the **current deployment profile** for Gotong Royong after stack lock.

Canonical profile:
- Rust 2024 backend (Axum + Tokio + Tower)
- SurrealDB `v3.0.0-beta.4`
- Redis for ephemeral/idempotency/rate control
- Realtime transport: local bus by default; Redis pub/sub when multi-replica
- S3-compatible object storage for evidence payloads
- OpenTelemetry + structured tracing

Reference decision:
- `docs/research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

## Storage Engine Profile

- Local development: in-memory engine (default) or file-backed for persistence.
- Staging/production: TiKV engine for distributed, scalable storage.
- SurrealKV remains beta and is not the default for production.

## Environments

## Local Development

Runtime layout:
- API: local process (`cargo run`)
- SurrealDB: local `memory` (default) or file-backed mode
- Redis: container
- MinIO: container

Recommended local commands are documented in:
- `docs/development/setup-guide.md`

## Staging

Objectives:
- Run pinned stack end-to-end with production-like networking.
- Validate beta go/no-go gates before production promotion.

Minimum services:
- `api`
- `surrealdb` (`v3.0.0-beta.4`) backed by TiKV
- `redis`
- `object storage` (S3-compatible)
- telemetry collector + metrics/log backend

## Production

Constraints:
- Pin versions (no floating tags for API or DB).
- Use repository/adapter isolation to reduce Surreal beta upgrade risk.
- Enforce rollback path to stable target defined by release policy.

Minimum topology:
- 2+ API replicas behind load balancer
- SurrealDB backed by TiKV (distributed storage engine)
- Redis HA profile
- Managed/object storage with bucket policy hardening
- Observability stack with alerts

## Container Baseline (Example)

```yaml
services:
  api:
    image: gotong/api:<pinned-tag>
    env_file:
      - .env
    environment:
      - SURREAL_ENDPOINT=ws://surrealdb:8000
      - SURREAL_NS=gotong
      - SURREAL_DB=chat
      - SURREAL_USER=root
      - SURREAL_PASS=${SURREAL_PASS}
      - REDIS_URL=redis://redis:6379
      - CHAT_REALTIME_TRANSPORT=redis
      - CHAT_REALTIME_CHANNEL_PREFIX=gotong:chat:realtime
      - S3_ENDPOINT=http://minio:9000
      - S3_BUCKET=gotong-royong-evidence
      - S3_ACCESS_KEY=${S3_ACCESS_KEY}
      - S3_SECRET_KEY=${S3_SECRET_KEY}
      - JWT_SECRET=${JWT_SECRET}
      - GOTONG_ROYONG_WEBHOOK_SECRET=${WEBHOOK_SECRET}
    depends_on:
      - surrealdb
      - redis
      - minio

  surrealdb:
    image: surrealdb/surrealdb:v3.0.0-beta.4
    command: start --user root --pass ${SURREAL_PASS} file:/data/surreal.db
    ports:
      - "8000:8000"

  redis:
    image: redis:7-alpine

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=${S3_ACCESS_KEY}
      - MINIO_ROOT_PASSWORD=${S3_SECRET_KEY}
```

Note:
- The example above uses a file-backed engine suitable for local development.
- For staging/production, run SurrealDB backed by TiKV and provide the TiKV endpoint per the official deployment guide.

## Configuration Policy

Required environment variables:
- `SURREAL_ENDPOINT`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`
- `REDIS_URL`
- `CHAT_REALTIME_TRANSPORT` (`local` in dev, `redis` in multi-replica production)
- `CHAT_REALTIME_CHANNEL_PREFIX`
- `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`
- `JWT_SECRET`
- `GOTONG_ROYONG_WEBHOOK_SECRET`

Rules:
- Never use `latest` tags in staging/production.
- Keep DB and SDK versions aligned with lock file and ADR.
- Use TiKV-backed SurrealDB in staging/production; reserve memory/file-backed engines for local development.

## Reliability and Rollback

Release gates (must pass):
- live query payload-shape tests
- ordering/catch-up tests
- permission-filtering tests
- reconnect resilience tests

Execute locally/CI before production rollout:

```bash
just release-gates-surreal
```

Rollback triggers:
- Any P0 gate failure in release candidate runs
- Production SEV-1 tied to DB/runtime behavior

Rollback mechanism:
- Deploy previous pinned API image
- Switch DB/runtime profile to predefined stable target
- Validate health probes and replay queue integrity
- Validate realtime transport behavior via `chat_realtime_transport` compatibility and shared channel prefix checks
- Follow the detailed rehearsed steps in `docs/deployment/rollback-rehearsal-runbook.md`

## Security Baseline

- Use secrets manager or sealed secrets for all credentials.
- Restrict Surreal/Redis network access to private subnets.
- Enforce TLS at ingress and service boundaries where applicable.
- Apply bucket least-privilege policies for evidence storage.

## Observability Baseline

Metrics:
- request rate/error/latency
- live stream disconnect rate
- stream lag and fallback activation rate
- queue lag and idempotency collision count

Tracing/logging:
- propagate `correlation_id` across API, jobs, and event delivery
- structured logs with error codes for DB and stream failures

## Legacy Notice

Previous infrastructure examples centered on PostgreSQL/MySQL relational deployment are superseded by this profile for new implementation work.

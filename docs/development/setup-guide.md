# Development Setup Guide

## Overview

This guide defines the **current canonical local setup** for Gotong Royong.

Stack lock:
- Rust 2024
- Axum + Tokio + Tower
- SurrealDB server `v3.0.0` (Rust SDK `surrealdb` `=3.0.0`)
- Redis
- MinIO (S3-compatible object storage)

Reference decision:
- `docs/architecture/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

Engine profile:
- Local dev: SurrealDB in-memory engine (default).
- Optional local persistence: file-backed engine.
- Staging/prod: TiKV engine (see `docs/deployment/infrastructure.md`).

## Prerequisites

| Software | Version | Purpose |
|---|---|---|
| Rust toolchain | 1.88.0+ | Build and run backend |
| Cargo | bundled with Rust | Build tooling |
| SurrealDB server | 3.0.0 | Local database runtime |
| Docker Desktop | latest | Run SurrealDB + Redis (+ MinIO) locally |
| Git | 2.30+ | Version control |

Optional:
- `cargo-watch` for hot reload (`cargo install cargo-watch`)
- `redis-cli` for cache inspection
- `mc` (MinIO client) for object storage inspection

## Environment

Create `.env.local`:

```bash
# App
APP_ENV=development
PORT=3000
LOG_LEVEL=debug

# SurrealDB
SURREAL_ENDPOINT=ws://127.0.0.1:8000
SURREAL_NS=gotong
SURREAL_DB=chat
SURREAL_USER=root
SURREAL_PASS=root

# Redis
REDIS_URL=redis://127.0.0.1:6379
CHAT_REALTIME_TRANSPORT=local
CHAT_REALTIME_CHANNEL_PREFIX=gotong:chat:realtime

# S3 / MinIO
S3_ENDPOINT=http://127.0.0.1:9000
S3_BUCKET=gotong-royong-evidence-dev
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_REGION=us-east-1
CHAT_ATTACHMENT_STORAGE_BACKEND=auto
CHAT_ATTACHMENT_S3_PREFIX=chat-attachments

# Auth
JWT_SECRET=dev_jwt_secret_32_chars_minimum
AUTH_DEV_BYPASS_ENABLED=true
GOTONG_ROYONG_WEBHOOK_SECRET=dev_webhook_secret_32_chars_minimum

# Worker
WORKER_QUEUE_PREFIX=gotong:jobs
WORKER_POLL_INTERVAL_MS=1000
WORKER_PROMOTE_BATCH=50
WORKER_BACKOFF_BASE_MS=1000
WORKER_BACKOFF_MAX_MS=60000
```

## Quick Start

### 1. Start dev dependencies (Docker Desktop)

```bash
just dev-db-up
```

This starts:
- SurrealDB on `ws://127.0.0.1:8000`
- Redis on `redis://127.0.0.1:6379`

Optional local object storage (MinIO):

```bash
docker compose -f compose.dev.yaml --profile storage up -d minio
```

`CHAT_ATTACHMENT_STORAGE_BACKEND` behavior:
- `auto` (default): use S3/MinIO if reachable, otherwise fallback to local disk (non-production).
- `local`: always use local disk (`tmp/gotong-chat-attachments/<env>`).
- `s3`: fail startup if S3 bucket is unreachable (recommended for production/staging).

Live attachment storage smoke (S3 path):

```bash
just smoke-chat-attachment-s3-live
```

Lifecycle policy plan/apply (for attachment prefix):

```bash
just chat-attachment-lifecycle-plan
just chat-attachment-lifecycle-apply
just chat-attachment-lifecycle-verify
```

### 2. Build and run backend

```bash
cargo build
just api
```

Hot reload (optional):

```bash
cargo watch -x "run -p gotong-api"
```

### 3. Verify dependencies

```bash
# SurrealDB
docker compose -f compose.dev.yaml exec -T surrealdb /surreal is-ready --endpoint ws://127.0.0.1:8000

# Redis
redis-cli ping

# MinIO (optional)
curl -fsS http://127.0.0.1:9000/minio/health/live
```

## Seed / Probe Data

Run the Surreal pattern probe used by research:

```bash
just surreal-probe
```

## Operational Commands

### Stop local services

```bash
just dev-db-down

# If you enabled MinIO:
docker compose -f compose.dev.yaml --profile storage down
```

### Reset local Surreal runtime

If using `memory` mode, restart the process to reset state.

If using file-backed mode, remove the datastore file/path and restart.

## Troubleshooting

### SurrealDB live query not streaming

- Ensure app/CLI endpoint uses `ws://`, not `http://`.
- Verify the server version is `3.0.0`.

### Redis connection refused

```bash
docker compose -f compose.dev.yaml ps
```

### MinIO bucket missing

Create bucket with `mc`:

```bash
mc alias set local http://127.0.0.1:9000 minioadmin minioadmin
mc mb local/gotong-royong-evidence-dev
```

## Notes

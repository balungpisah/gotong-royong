# Development Setup Guide

## Overview

This guide defines the **current canonical local setup** for Gotong Royong.

Stack lock:
- Rust 2024
- Axum + Tokio + Tower
- SurrealDB `v3.0.0-beta.4`
- Redis
- MinIO (S3-compatible object storage)

Reference decision:
- `docs/research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

Engine profile:
- Local dev: SurrealDB in-memory engine (default).
- Optional local persistence: file-backed engine.
- Staging/prod: TiKV engine (see `docs/deployment/infrastructure.md`).

## Prerequisites

| Software | Version | Purpose |
|---|---|---|
| Rust toolchain | 1.88.0+ | Build and run backend |
| Cargo | bundled with Rust | Build tooling |
| SurrealDB CLI/server | 3.0.0-beta.4 | Local database runtime |
| Docker | 20.10+ | Run Redis + MinIO locally |
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

# S3 / MinIO
S3_ENDPOINT=http://127.0.0.1:9000
S3_BUCKET=gotong-royong-evidence-dev
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_REGION=us-east-1

# Auth
JWT_SECRET=dev_jwt_secret_32_chars_minimum
GOTONG_ROYONG_WEBHOOK_SECRET=dev_webhook_secret_32_chars_minimum

# Worker
WORKER_QUEUE_PREFIX=gotong:jobs
WORKER_POLL_INTERVAL_MS=1000
WORKER_PROMOTE_BATCH=50
WORKER_BACKOFF_BASE_MS=1000
WORKER_BACKOFF_MAX_MS=60000
```

## Quick Start

### 1. Start Redis and MinIO

```bash
docker run -d --name gotong-redis -p 6379:6379 redis:7-alpine
docker run -d --name gotong-minio \
  -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio:latest server /data --console-address ":9001"
```

### 2. Start SurrealDB 3 beta

Default local profile uses the in-memory engine. Use the pinned binary path if available:

```bash
./docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 start memory --user root --pass root --bind 127.0.0.1:8000
```

If using another installed binary, ensure it reports `3.0.0-beta.4`:

```bash
surreal version
```

### 3. Build and run backend

```bash
cargo build
cargo run -p gotong-api
```

Hot reload (optional):

```bash
cargo watch -x "run -p gotong-api"
```

### 4. Verify dependencies

```bash
# Surreal
./docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 is-ready --endpoint ws://127.0.0.1:8000

# Redis
redis-cli ping

# MinIO
curl http://127.0.0.1:9000/minio/health/live
```

## Seed / Probe Data

Run the Surreal pattern probe used by research:

```bash
SURREAL_BIN=docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 \
LOCKED_TARGET_VERSION=3.0.0-beta.4 \
docs/research/samples/surrealdb/pattern_probe.sh \
docs/research/surrealdb-pattern-sampling-v3-beta4.md
```

## Operational Commands

### Stop local services

```bash
docker rm -f gotong-redis gotong-minio
```

### Reset local Surreal runtime

If using `memory` mode, restart the process to reset state.

If using file-backed mode, remove the datastore file/path and restart.

## Troubleshooting

### SurrealDB live query not streaming

- Ensure app/CLI endpoint uses `ws://`, not `http://`.
- Verify the server version is `3.0.0-beta.4`.

### Redis connection refused

```bash
docker ps | rg gotong-redis
```

### MinIO bucket missing

Create bucket with `mc`:

```bash
mc alias set local http://127.0.0.1:9000 minioadmin minioadmin
mc mb local/gotong-royong-evidence-dev
```

## Notes

- Pre-lock polyglot setup instructions (Node/Python/PostgreSQL/Knex/Alembic/Diesel) are superseded.
- If you need historical context, check git history for older guide revisions.

# Database Migrations

## Overview

This document defines migration workflow for the locked SurrealDB stack.

Stack context:
- SurrealDB server `v3.0.0`
- Rust backend with repository/adapter boundary

## Migration Principles

- Forward-only, additive-first changes whenever possible.
- Every schema change must be reproducible and versioned.
- Every migration must include verification query checks.
- Rollback strategy must be documented before production apply.

## Repository Structure

Recommended layout:

```text
database/
  migrations/
    0001_initial_schema.surql
    0002_chat_indexes.surql
    0003_permissions_private_channels.surql
  checks/
    0001_initial_schema_check.surql
    0002_chat_indexes_check.surql
    0003_permissions_private_channels_check.surql
  MIGRATION-STATE.md
```

## Migration Format

Each migration file should include:
- Id and title
- Preconditions
- `DEFINE`/`REMOVE` statements
- Post-apply verification query snippets

Example (`0002_chat_indexes.surql`):

```sql
-- 0002_chat_indexes
DEFINE INDEX idx_message_order
ON TABLE chat_message FIELDS thread_id, created_at, message_id;

DEFINE INDEX uniq_delivery_request
ON TABLE chat_delivery_event FIELDS thread_id, request_id UNIQUE;
```

Example check (`0002_chat_indexes_check.surql`):

```sql
INFO FOR TABLE chat_message;
INFO FOR TABLE chat_delivery_event;
```

## Apply Workflow

## Runner Scripts

Use the standardized scripts:
- `scripts/db/migrate.sh`
- `scripts/db/check.sh`

## Local

```bash
surreal import \
  --endpoint ws://127.0.0.1:8000 \
  --user root \
  --pass root \
  --namespace gotong \
  --database chat \
  database/migrations/0001_initial_schema.surql
```

Run verification queries:

```bash
cat database/checks/0001_initial_schema_check.surql | surreal sql \
  --endpoint ws://127.0.0.1:8000 \
  --user root \
  --pass root \
  --namespace gotong \
  --database chat \
  --json
```

## CI/CD

Pipeline requirements:
- Apply migrations to ephemeral DB
- Run verification queries
- Run integration tests for idempotency, ordering, live stream, permissions
- Block release on any migration/check failure

## Version Tracking

Track applied migrations in one of:
- dedicated migration metadata table, or
- immutable deployment manifest tied to release SHA.

Minimum metadata:
- migration id
- applied timestamp
- release SHA
- operator/automation actor

## Rollback Strategy

Because destructive rollback can lose data, default to:
- stop rollout
- deploy previous app version
- apply compensating migration when needed

If rollback script exists, it must include:
- explicit data-loss warning
- backup checkpoint reference
- post-rollback integrity checks

## Migration Test Matrix

Each migration must be validated against:
- schema creation/update correctness
- existing data compatibility
- permission behavior
- live-query behavior (where affected)
- performance impact on critical queries

## Operational Checklist

Before production apply:
- confirm pinned SurrealDB version matches lock (`3.0.0`)
- backup/export strategy verified
- migration + checks reviewed
- rollback/compensating path approved

After production apply:
- validate health endpoints
- run post-migration query checks
- monitor error rate and stream behavior

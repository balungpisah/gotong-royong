# Database Schema Requirements

## Overview

This document defines schema requirements for the locked implementation stack:
- SurrealDB `v3.0.0-beta.4`
- Rust SDK 3 beta channel
- Chat-heavy workload with realtime subscriptions

Reference documents:
- `docs/backend-research.md`
- `docs/research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

## Design Principles

- Append-only event history for auditable transitions.
- Deterministic ordering for timeline queries.
- Idempotent write guarantees for retry-safe APIs.
- Permission-aware data access for live subscriptions.
- Adapter boundary in application layer to isolate DB-specific semantics.

## Core Records

## `chat_thread`
Purpose:
- Thread metadata and visibility control.

Required fields:
- `thread_id`
- `scope_id`
- `created_by`
- `privacy_level`
- `created_at`
- `updated_at`

## `chat_member`
Purpose:
- Membership and role state per thread.

Required fields:
- `thread_id`
- `user_id`
- `role`
- `joined_at`
- `left_at` (nullable)
- `mute_until` (nullable)

## `chat_message`
Purpose:
- Immutable message body and metadata.

Required fields:
- `thread_id`
- `message_id`
- `author_id`
- `body`
- `attachments`
- `created_at`
- `edited_at` (nullable)
- `deleted_at` (nullable)

Rules:
- `created_at` and `message_id` together form deterministic order key.
- Message order fields are immutable after insert.

## `chat_read_cursor`
Purpose:
- Per-user read position.

Required fields:
- `thread_id`
- `user_id`
- `last_read_message_id`
- `last_read_at`

## `chat_delivery_event`
Purpose:
- Delivery and replay audit trail.

Required fields:
- `event_id`
- `thread_id`
- `message_id`
- `event_type`
- `request_id`
- `correlation_id`
- `occurred_at`

## `track_state_transition`
Purpose:
- Canonical state transition ledger for governance workflow.

Required fields:
- `transition_id`
- `entity_id`
- `request_id`
- `correlation_id`
- `from_stage`
- `to_stage`
- `transition_type`
- `actor`
- `occurred_at`
- `gate`

## SurrealQL Baseline (Illustrative)

```sql
DEFINE TABLE chat_thread SCHEMAFULL;
DEFINE TABLE chat_member SCHEMAFULL;
DEFINE TABLE chat_message SCHEMAFULL;
DEFINE TABLE chat_read_cursor SCHEMAFULL;
DEFINE TABLE chat_delivery_event SCHEMAFULL;
DEFINE TABLE track_state_transition SCHEMAFULL;

DEFINE FIELD thread_id ON TABLE chat_message TYPE string;
DEFINE FIELD message_id ON TABLE chat_message TYPE string;
DEFINE FIELD created_at ON TABLE chat_message TYPE datetime;
DEFINE FIELD author_id ON TABLE chat_message TYPE string;
DEFINE FIELD body ON TABLE chat_message TYPE string;

DEFINE INDEX idx_message_order
ON TABLE chat_message FIELDS thread_id, created_at, message_id;

DEFINE INDEX uniq_delivery_request
ON TABLE chat_delivery_event FIELDS thread_id, request_id UNIQUE;

DEFINE INDEX uniq_transition_request
ON TABLE track_state_transition FIELDS entity_id, request_id UNIQUE;

DEFINE INDEX idx_member_lookup
ON TABLE chat_member FIELDS user_id, thread_id;
```

## Realtime Subscription Keys

- Message stream: `LIVE SELECT ... FROM chat_message WHERE thread_id = $thread_id`
- Membership stream: `LIVE SELECT ... FROM chat_member WHERE thread_id = $thread_id`
- Read cursor stream: `LIVE SELECT ... FROM chat_read_cursor WHERE thread_id = $thread_id`

Note:
- Use `ws://` transport for live streaming behavior.

## Permission Requirements

- Record/table permissions must prevent cross-user leakage in private channels.
- Live subscription output must respect row/field permission filters.
- Permission behavior must be validated by integration probes/tests.

## Data Retention

Minimum policy anchors:
- Keep transition and delivery events append-only.
- Apply anonymization/deletion rules for sensitive domains (e.g., Siaga identities, vault payloads) per policy documents.
- Preserve audit metadata when payload deletion is required by policy.

## Validation Requirements

Must-have checks:
- Idempotency collision handling (`unique(entity_id/request_id)` style constraints)
- Deterministic timeline ordering under same timestamp
- Reconnect catch-up query semantics
- `LIVE SELECT DIFF` payload behavior on updates
- Permission-filtered live stream behavior

## Legacy Notice

Relational PostgreSQL/MySQL schema examples from pre-lock documentation are superseded for current implementation planning.

# Backend Implementation Packet — Gotong/Royong (Rust/Axum/SurrealDB Profile)

Last updated: `2026-02-15`
Status: READY FOR BACKEND DESIGN/HANDOFF

## 1) Canonical rules that must be enforced in all implementations

1. Language/framework are fixed by this packet: Rust 2024 + Axum + Tokio + Tower/tower-http + SurrealDB SDK 3 beta.
2. All request/response contracts must follow docs under:
   - `docs/research/edgepod-endpoint-contracts.md`
   - `docs/research/edgepod-endpoint-contracts.schema.json`
   - `docs/research/edgepod-endpoint-contracts/EP-00.schema.json` ... `EP-11.schema.json`
3. All IDs and timestamps in state events must be stable and immutable once emitted.
4. All write actions must include `request_id` and `correlation_id`.
5. Event replay on retry is mandatory where `request_id` is supplied.

## 2) Implementation ticket pack (handoff-ready)

### 2.1 Track/state and gating

1. Ticket `BE-001`: Track transition command/event model
   - Scope: `UI-03`
   - Canonical: `docs/research/decision-notes.md`, `docs/research/feature-contract-map.md`, `docs/design/backend-design-contract-gotong-tandang.md`
   - Requirements:
     1. create `track_state_transition` write path
     2. support trigger types `user_action`, `timer`, `webhook`
     3. persist `request_id`, `transition_id`, `entity_id`, `from_stage`, `to_stage`, `transition_type`, `mechanism`, `actor`, `occurred_at`, `gate`, `correlation_id`
     4. enforce unique `(entity_id, request_id)`
     5. enforce role matrix:
        - `propose`: `author` or `pic`, with `system` allowed only for timer close apply
        - `object`: `participant|saksi|author|pic` with participant objection forcing vote window
        - `vote`: `author|saksi|participant|pic` with snapshot of eligibility at gate open
     6. enforce gated prerequisites:
        - Garap → Periksa requires PoR refs
        - Periksa → Tuntas requires challenge window gate config
     7. snapshot actor context at command time into immutable transition metadata (`actor_snapshot`):
        - actor user id, username, token role, and membership context (`author|pic|participant|saksi` snapshot)
        - request provenance (`request_id`, `correlation_id`, `request_ts_ms`)
     8. enforce role/gate decisions in one atomic command path (no split across eventual-consistency transitions)
     9. baseline migration prerequisite: `track_state_transition` + `uniq_transition_request` already exists in
        `database/migrations/0001_initial_schema.surql` and `database/migrations/0002_chat_indexes.surql`
   - Acceptance:
     1. replayed transitions return original canonical transition record
     2. scheduler emits closure/reject event with same `transition_id`
     3. `gate.status` transitions to applied/rejected only once at gate close
     4. retries with same `(entity_id, request_id)` are replay-safe
 - PR-07 acceptance test hooks:
    - [x] replay test: same `(entity_id, request_id)` returns canonical repeat transition
    - [x] ordering test: timeline ordering by deterministic sort keys (`occurred_at_ms`, `transition_id`)
    - [x] append-only invariants: no update/delete mutation API in service/repo surface (interface only exposes create/list/get/get_by_id)
    - [x] role matrix test: `propose/object/vote` matrix validation against `author/pic/participant/saksi`
    - [x] projection order test: timeline query is monotonic and stable under repeated reads

2. Ticket `BE-002`: Track transition state projections
   - Scope: `UI-03` + history audit
   - Requirements:
     1. create read model for stage history and active state
     2. expose timeline and audit filters by `entity_id`
     3. support query by `scope_id`, `track`, and stage state for feed and governance
     - PR-07 scope: minimal read model (`active stage + linear timeline`) derived from the append-only event stream.
     - PR-12 scope: richer filters and indexes for feed/governance surfaces.
   - Acceptance:
     1. history order is deterministic
     2. transitions never mutate; append-only event store for transitions

### 2.2 Moderation and enforcement

3. Ticket `BE-003`: Moderation source-of-truth service
   - Scope: `UI-07`
   - Canonical: `docs/research/decision-notes.md`, `docs/research/edgepod-endpoint-contracts.md`, `docs/design/specs/ai-spec/08-ai-04-content-moderation.md`
   - Requirements:
     1. persist moderation rows in Gotong DB:
        - `content_id`, `moderation_status`, `moderation_action`, `reason_code`, `confidence`, `decided_at`, `decided_by`, `hold_expires_at`, `auto_release_if_no_action`, `violations`
     2. maintain `moderation_decisions` append-only decision trail
     3. persist `AI-04` outputs and moderator actions in same audit stream
   - Acceptance:
     1. role-scoped response projection:
        - public fields: no private reason/snippets
        - author fields: risk context + reason code + appeal window
        - moderator fields: full violations, model reasoning, history

4. Ticket `BE-004`: Moderation policy runtime and fallback
   - Scope: `UI-07`
   - Requirements:
     1. integrate hold/review/publish actions into state transitions
     2. auto-release path for expired holds
     3. explicit manual-review queue and appeal hooks
   - Acceptance:
     1. hold expiry auto-release is deterministic and auditable
     2. manual review path is never blocked by moderation service timeout

### 2.3 Siaga emergency

5. Ticket `BE-005`: Siaga emergency model and lifecycle events
   - Scope: `UI-08`
   - Canonical: `docs/research/decision-notes.md`
   - Required events:
     1. `siaga_broadcast_created`
     2. `siaga_broadcast_activated`
     3. `siaga_broadcast_updated`
     4. `siaga_responder_joined`
     5. `siaga_responder_updated`
     6. `siaga_broadcast_closed`
     7. `siaga_broadcast_cancelled`
   - Required fields:
     1. `siaga_id`, `scope_id`, `author_id`, `emergency_type`, `severity`, `location`, `state`, `text/title`, `created_at`, `timeline_events`, `counters`, `closure`
   - Acceptance:
     1. append-only timeline
     2. explicit close confirmation mandatory with summary
     3. responder identities retained 7 days then anonymized

### 2.4 Catatan Saksi vault

6. Ticket `BE-006`: Vault lifecycle and persistence boundaries
   - Scope: `UI-09`
   - Canonical: `docs/research/decision-notes.md`
   - Requirements:
     1. implement states: `draft`, `sealed`, `published`, `revoked`, `expired`
     2. persist minimum set:
        - `vault_entry_id`, `author_id`, `state`, `created_at`, `sealed_at`, `sealed_hash`, `encryption_key_id`, `attachment_refs`, `wali`, `publish_target`, `retention_policy`, `audit`
     3. index only tombstone metadata for published/revoked
   - Acceptance:
     1. `draft` editable/deletable
     2. `sealed` immutable except wali revoke
     3. sealed payload deletion on revoke while retaining metadata

7. Ticket `BE-007`: Vault lifecycle events
   - Scope: `UI-09`
   - Required events:
     1. `witness_drafted`
     2. `witness_sealed`
     3. `witness_trustee_added`
     4. `witness_trustee_removed`
     5. `witness_published`
     6. `witness_revoked`
     7. `witness_expired`
   - Acceptance:
     1. timeline order preserved
     2. revoked/published transitions cannot revert

### 2.5 Feed/search/notifications and read models

8. Ticket `BE-008`: Feed service
   - Scope: `UI-10`
   - Canonical: `docs/design/backend-design-contract-gotong-tandang.md`, `docs/research/feature-contract-map.md`
   - Requirements:
     1. implement cursor pagination with opaque cursor + limit 1-50
     2. stable sort keys across updates
     3. filters for `scope_id`, `track`, `stage`, `time_range`, `involvement`, `privacy_level`
     4. integrate trust snapshots from Tandang as read-only signal
   - Acceptance:
     1. deterministic ordering by time and secondary tie-breaker
     2. privacy redaction for L2+ as required by contract

9. Ticket `BE-009`: Search projection service
   - Scope: `UI-11`
   - Requirements:
     1. derive index from app DB with redaction rules
     2. support filters and privacy exclusions
     3. exclude vault content
   - Acceptance:
     1. no PII leak for forbidden visibility
     2. ranking deterministic when model scores unavailable

10. Ticket `BE-010`: Notification and digest service
   - Scope: `UI-12`
   - Requirements:
     1. fan-out event ingestion contract from app bus
     2. unread counters and `read_at` updates
     3. weekly digest assembly using raw event list fallback
   - Acceptance:
     1. dedupe by event key
     2. exclude embargoed/private content according to privacy policy

### 2.6 Edge-Pod integration and AI endpoints

11. Ticket `EP-03`: duplicate detection endpoint
   - Canonical endpoint: `POST /api/v1/edge-pod/ai/03/duplicate-detection`
   - Prompt: `DUPLICATE-001` (`v0.2.0`)
   - Prompt metadata from `docs/research/edgepod-endpoint-contracts.schema.json`
   - Input/output must match `docs/research/edgepod-endpoint-contracts/EP-03.schema.json`
   - Acceptance: non-blocking fallback path emits warning and logs warning reason

12. Ticket `EP-05`: gaming risk endpoint
   - Canonical endpoint: `POST /api/v1/edge-pod/ai/05/gaming-risk`
   - Prompt: `GAMING-001` (`v0.2.0`)
   - Contract file: `docs/research/edgepod-endpoint-contracts/EP-05.schema.json`
   - Acceptance: advisory output only unless policy explicitly says block

13. Ticket `EP-08`: sensitive media endpoint
   - Canonical endpoint: `POST /api/v1/edge-pod/ai/08/sensitive-media`
   - Prompt: `SENSITIVE-001` (`v0.2.0`)
   - Contract file: `docs/research/edgepod-endpoint-contracts/EP-08.schema.json`
   - Acceptance: raw media passes through when scanner fails and manual moderation path is queued

14. Ticket `EP-09`: credit recommendation endpoint
   - Canonical endpoint: `POST /api/v1/edge-pod/ai/09/credit-recommendation`
   - Prompt: `CREDIT-001` (`v0.2.0`)
   - Contract file: `docs/research/edgepod-endpoint-contracts/EP-09.schema.json`
   - Acceptance: output remains advisory and manual form stays available

15. Ticket `EP-11`: siaga evaluate endpoint
   - Canonical endpoint: `POST /api/v1/edge-pod/ai/siaga/evaluate`
   - Prompt: composed path from `TRIAGE-001` + `MOD-001`
   - Contract file: `docs/research/edgepod-endpoint-contracts/EP-11.schema.json`
   - Acceptance: siaga candidate may still be manually approved/denied without AI

### 2.7 Additional cross-cutting services

16. Ticket `BE-011`: idempotency and request de-duplication service
   - Scope: all event writes and all AI calls
   - Requirements:
     1. centralized store keyed by `(entity_id, request_id)` for app writes
     2. deterministic request_id conventions for timers/jobs
     3. replay-safe response cache for idempotent requests
   - Acceptance:
     1. repeated request_id returns same status and payload shape

17. Ticket `BE-012`: audit and retention governance
   - Scope: moderation, siaga, vault, transitions
   - Requirements:
     1. append-only event log retention policy matrix
     2. anonymization window and redaction handling
     3. immutable event hashes and retention tags
   - Acceptance:
     1. 7-day responder identity rule for siaga works
     2. vault revoked payload deletion verified

18. Ticket `BE-013`: operations observability baseline
   - Scope: whole backend
   - Requirements:
     1. metrics: request success, fallback counts, queue lag, transition completion SLA
     2. structured tracing keyed by `correlation_id`
     3. structured error catalog for model and idempotency failures
   - Acceptance:
     1. per-UI feature alert rules can be configured

## 3) Dependency order for implementation

1. Establish shared primitives and validation schemas (`request_id`, `correlation_id`, envelope, base enums).
2. Implement canonical event store and idempotency layer.
3. Implement `BE-001` and `BE-002` state machine before any stage-specific workflow automation.
4. Implement moderation service `BE-003` and policy runtime `BE-004` before moderation-triggered API behavior.
5. Implement vault model `BE-006` and vault events `BE-007` in isolation.
6. Implement Siaga lifecycle `BE-005`.
7. Implement read models `BE-008`, `BE-009`, `BE-010`.
8. Implement Edge-Pod contracts EP-03, EP-05, EP-08, EP-09, EP-11 in parallel once prompt schemas are fixed.
9. Integrate Edge-Pod outputs into `BE-001`/moderation/notifications flows.
10. Implement audit/retention `BE-012` and observability `BE-013`.
11. Run consistency pass against all canonical docs and perform integration smoke scenarios across:
    1. `UI-03` state transitions
    2. `UI-07` moderation states
    3. `UI-08` closure and timeline
    4. `UI-09` vault revoke behavior
    5. `UI-10` to `UI-12` visibility and filtering

## 4) Stack mapping (active profile)

1. The implementation profile is active now; convert domain tickets into concrete Rust/Axum/SurrealDB tasks directly.
2. Keep contract, idempotency, role logic, and privacy controls unchanged while mapping technical tasks.

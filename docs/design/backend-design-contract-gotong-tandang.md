# Gotong Royong Backend Design Preparation: Tandang-Compatible Contract

**Date:** 2026-02-14  
**Status:** Draft  
**Audience:** Backend engineers, API designers, AI implementers

## 1. Purpose

This document prepares the backend implementation by combining:

1. Gotong Royong design/architecture/docs
2. Tandang (Markov Engine) integration contracts
3. Edge-Pod conventions for AI feature work

## 2. Source documents to interpolate

### Gotong Royong

1. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/design/context/DESIGN-CONTEXT.md`
2. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/architecture/system-overview.md`
3. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/architecture/integration-architecture.md`
4. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/design/specs/AI-SPEC-v0.2.md`
5. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/design/specs/ai-spec/05-ai-01-classification.md`
6. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/design/specs/ai-spec/15-model-selection.md`
7. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/api/webhook-spec.md`
8. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/api/event-payloads.md`
9. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/api/authentication.md`
10. `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong/docs/database/schema-requirements.md`

### Tandang (Markov Engine)

1. `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md`
2. `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/docs/API-PLATFORMS.md`
3. `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/crates/api/src/routes/platforms.rs`
4. `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/crates/domain/src/platform/adapter.rs`

### Edge-Pod (AI conventions)

1. `/Users/damarpanuluh/MERIDIAN-NEW/edge-pod/docs/prompts/11-project-conventions.md`
2. `/Users/damarpanuluh/MERIDIAN-NEW/edge-pod/docs/prompts/14-prompt-versioning.md`
3. `/Users/damarpanuluh/MERIDIAN-NEW/edge-pod/crates/edge-pod-llm/src/prompt/typed_output.rs`
4. `/Users/damarpanuluh/MERIDIAN-NEW/edge-pod/crates/edge-pod-runtime/src/forge/ai_processor.rs`

## 3. Contract objective

The Gotong backend implementation must produce API/events and data semantics compatible with Tandang while maintaining Gotong product behavior and AI governance constraints.

This includes:

1. Stable webhook contract to `gotong_royong`
2. Event-level traceability and idempotency
3. Signed transport and replay-safe processing
4. AI outputs that are typed, versioned, and validated before use

## 4. Tandang compatibility contract (required)

### 4.1 Platform characteristics

| Field | Required value |
|---|---|
| `platform_id` | `gotong_royong` |
| `integration_mode` | Native |
| `behavior_category` | `task_based` |
| Supported event types | `contribution_created`, `vouch_submitted`, `por_evidence` |
| Authentication for ingestion | HMAC-SHA256 webhook signature |
| Rate policy | Unlimited in integration characteristics; enforce explicit per-environment webhook limits (`100` dev, `1000` prod) for safety |

### 4.2 Ingestion endpoint

1. Primary URL: `POST /api/v1/platforms/gotong_royong/webhook`
2. Method: `POST` only
3. Required header: `X-GR-Signature: sha256=<hex>`
4. Content-Type: `application/json`
5. Success HTTP code: `200`
6. Success body:
```json
{
  "processed": 1,
  "results": [
    {
      "type": "contribution_created",
      "contribution_id": "contrib_abc123",
      "message": "Contribution created (reputation: +50)"
    }
  ]
}
```

### 4.3 Idempotency

1. Every outbound event must include `event_id`.
2. `event_id` format: `evt_[a-f0-9]{16}`.
3. Replayed `event_id` must return `200` and be treated as no-op to avoid double scoring.

### 4.4 Error contract

1. `401` for signature/auth failures.
2. `400` for malformed payloads and schema violations.
3. `429` for rate-limit events (respect `Retry-After` when available).
4. `500` and `503` are retryable.

### 4.5 Security / signing

1. Signature is HMAC-SHA256 over the raw request body.
2. Signature format must be exactly `sha256=<hex_digest>`.
3. Secret rotation: 90 days with a dual-secret transition window where feasible.
4. Avoid logging raw signatures or body that includes secrets.
5. Use compact JSON serialization for signature reproducibility.

### 4.6 Event schema minimum

#### Common top-level fields

1. `event_type`
2. `actor.user_id`
3. `actor.username`
4. `subject` object (shape depends on type)

#### `contribution_created`

1. Required: `subject.contribution_type`, `subject.title`
2. Optional: `description`, `evidence_url`, `skill_ids`, `metadata`

#### `vouch_submitted`

1. Required: `subject.vouchee_id`
2. Optional: `subject.skill_id`, `weight_hint`, `message`

#### `por_evidence`

1. Must include a `proof` block and evidence type
2. For `witness_attestation`, enforce witness array non-empty and timestamps within policy limits

#### Tandang-specific mapping requirement

1. Persist event source `actor.user_id` to `users.markov_user_id` format and keep traceability metadata.
2. Preserve evidence references and witness metadata for audit logs and reputation recomputation.

## 5. Gotong internal backend contract for implementation planning

### 5.1 Core modules

1. Task service to create/assign/complete contributions
2. Evidence service to ingest/store PoR with integrity metadata
3. Vouch service with weight-hint handling
4. Reputation dispatch worker that publishes signed webhooks
5. Retry worker for webhook delivery with exponential backoff

### 5.2 Storage contract

1. Keep `users.markov_user_id` for mapping to Markov identity.
2. Keep contribution records with fields that map directly to event `subject`.
3. Keep event outbox with `event_id`, `event_type`, `payload`, delivery status, retry count.
4. Keep webhook delivery log with request/response body hash and HTTP response code.

### 5.3 API contract (Gotong-local)

1. Authenticated user operations for task/evidence/vouch flows
2. Admin/reconciliation endpoint for webhook outbound queue and dead-letter inspection
3. Optional platform discovery proxy for `platforms/:platform_id` metadata
4. Health/readiness and operational metrics for integration paths

## 6. AI implementation contract (apply Edge-Pod conventions)

### 6.1 Core guardrails and patterns

1. Follow Edge-Pod non-negotiables: serial message processing within a session and no LLM call during history lookup.
2. Use typed AI I/O. Return/consume JSON that is validated against schema (`TypedOutput` pattern).
3. Keep prompts versioned and auditable (`TRIAGE-001`, `CLASS-001`, etc.).
4. Track latency and confidence targets per touchpoint.
5. Route failures through typed errors (code + message + optional detail).

### 6.2 AI touchpoint requirements to implement in Gotong

1. AI-00 Conversational Triage: `<2s` response, Sonnet-tier, conversational plus classification trigger.
2. AI-01 Classification: `<1s` response, Haiku-tier, structured output with `track`, `seed_type`, `esco_skills`.
3. AI-02/AI-04/AI-05/AI-06/AI-07/AI-08/AI-09 follow the touchpoint definitions in v0.2 with explicit fallback when unavailable.

### 6.3 Recommended AI data contract

1. Inputs: `{track_context, user_message, evidence_refs, user_session_id, context_state_id}`
2. Output object:
  - `result` (structured decision fields)
  - `confidence` (0.0–1.0)
  - `reasoning` (short summary)
  - `fallback_recommended` (boolean)
  - `errors` (typed list)
3. All responses must include `prompt_version` and `model_id`.
4. Persist every model outcome with request_id correlation to webhook/event IDs where relevant.

## 7. Open questions

1. Should `gotong_royong` expose `POST /platforms/gotong_royong/ingest` for service-account assisted writes or only webhooks?
2. Is `proof.timestamp` retention policy fixed to 30 days across all PoR types or only witness attestations?
3. Confirm whether `vouch_submitted.weight_hint` default should be hard-coded or mapped to platform-level policy.
4. Confirm required `schema_version` field in payload envelope for `vouch_submitted` and `por_evidence`.

## 8. Delivery checklist for backend planning ticket

1. Draft API schemas (request/response DTOs + validation rules).
2. Define database migration for outbox and delivery tracking.
3. Define webhook publisher job and idempotency index.
4. Define AI touchpoint DTOs and typed-output schemas.
5. Add replay and retry operational runbooks.

## 9. Build list strategy (recommended)

The best strategy is a **UI-driven, contract-first build list**:

1. Convert each UI workflow into backend capabilities.
2. Attach source-of-truth reference for each capability.
3. Tag each item as `Must`, `Should`, or `Nice-to-have`.
4. Mark as done only when:
   - route/API exists,
   - schema/validation is in place,
   - tests are written,
   - and observability is added.

### 9.1 Backend build checklist (checklist)

#### A. Foundation (Must)

- [ ] Bootstrap project structure, env config, secure secret loading.
  - Source: `docs/development/local-development.md`, `docs/deployment/infrastructure.md`
- [ ] DB schema + migrations for users, tasks, contributions, evidence, vouches, outbox, webhook_delivery_log.
  - Source: `docs/database/schema-requirements.md`
- [ ] API layer routing and middleware (auth, validation, rate-limit, tracing).
  - Source: `docs/api/webhook-spec.md`, `docs/api/authentication.md`
- [ ] Structured error type and API response standardization.
  - Source: `docs/api/error-handling.md`
- [ ] OpenAPI/contract publication and schema versioning baseline.
  - Source: existing API docs + internal API conventions

#### B. UI-driven feature coverage (Must/Should)

- [ ] Bagikan flow (create task/seed): task creation, draft save, validation, attachments metadata.
  - Source: `docs/design/specs/UI-UX-SPEC-v0.5.md`
- [ ] Berita/Tanda evidence flow: evidence upload, PoR metadata store, integrity checks.
  - Source: `docs/por-evidence/*`, `docs/design/specs/UI-UX-SPEC-v0.5.md`
- [ ] Vouch flow: vouch submit/listing/restrictions/history.
  - Source: `docs/design/specs/UI-UX-SPEC-v0.5.md`
- [ ] Progress/monitoring endpoints used by dashboard views.
  - Source: `docs/design/specs/UI-UX-SPEC-v0.5.md`

#### C. Tandang compatibility (Must)

- [ ] Webhook dispatcher emits signed `gotong_royong` events with idempotency keys.
  - Source: `markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md`
- [ ] `contribution_created`, `vouch_submitted`, `por_evidence` payload builders and validators.
  - Source: `docs/api/event-payloads.md`
- [ ] Replay-safe event outbox + delivery retries with exponential backoff.
  - Source: `docs/api/webhook-spec.md`
- [ ] Retryable/non-retryable response classification and DLQ handling.
  - Source: `docs/api/webhook-spec.md`
- [ ] Platform metadata endpoint support for discovery (`/platforms/{platform_id}`).
  - Source: `markov-engine/docs/API-PLATFORMS.md`

#### D. AI-backed capabilities (Should, with fallback)

- [ ] AI-00 + AI-01 orchestration with typed, validated outputs and confidence scoring.
  - Source: `docs/design/specs/ai-spec/04-ai-00-triage.md`, `docs/design/specs/ai-spec/05-ai-01-classification.md`
- [ ] AI touchpoint schema versioning and audit logging.
  - Source: `docs/design/specs/ai-spec/14-prompt-versioning.md`
- [ ] AI fallback path for low-confidence and infra errors.
  - Source: `docs/design/specs/ai-spec/v0.2` sections
- [ ] Guardrails and moderation error paths.
  - Source: `docs/design/specs/ai-spec/18-guardrails.md`

#### E. Operations (Should)

- [ ] Telemetry: webhook latency, errors by class, PoR failures, queue depth.
  - Source: integration guide + deployment monitoring docs
- [ ] Security hardening: signature audit logs, secret rotation, TLS + body handling.
  - Source: `docs/api/webhook-spec.md`, `docs/deployment/security-checklist.md`
- [ ] Recovery playbook for failed webhooks, duplicate events, and partial writes.
  - Source: integration docs + operational runbook
- [ ] End-to-end test matrix for happy path, replay, auth errors, schema errors.
  - Source: `docs/development/testing.md`

### 9.2 Suggested execution order

1. Foundation → Contract schemas → Storage
2. Tandang event pipeline + outbox/retries
3. Task/Contribution + Vouch + Evidence APIs
4. AI integrations and fallback plumbing
5. UI-dependent quality and load validation

### 9.3 Readiness gate before release

- [ ] All checklist items in section A–C are complete.
- [ ] No API contract ambiguity remains (every endpoint has a request/response schema).
- [ ] Retry/idempotency behavior is proven by test for duplicate/webhook-401/retry cases.
- [ ] AI path can degrade safely without breaking user-critical flows.

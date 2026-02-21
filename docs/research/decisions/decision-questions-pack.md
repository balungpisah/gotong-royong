# Decision Questions Pack (Research-to-Backend Handoff)

Use this file as the **questionnaire history** for PM, AI lead, and backend owners.

## STATUS: 2026-02-14 update

- `UI-03`, `UI-07`, `UI-08`, `UI-09`, `UI-10`, `UI-11`, `UI-12`: resolved and reflected in:
  - `docs/research/feature-contract-map.md`
  - `docs/research/tandang-gap-log.md`
  - `docs/research/edgepod-endpoint-contracts.md`
- Prompt registrations:
  - `AI-03` = `DUPLICATE-001`
  - `AI-05` = `GAMING-001`
  - `AI-08` = `SENSITIVE-001`
  - `AI-09` = `CREDIT-001` (no companion scoring prompt in current scope)

Use remaining questions below only if architecture lock adds new scope not covered above.

## Priority A — Immediate blockers (must be resolved before architecture lock)

### 1) Track transitions contract (`UI-03`)
- What is the canonical event name for track stage movement?
  - Example candidates: `TRACK_STATE_TRANSITION`, `TRACK_TRANSITION_PROPOSED`, `TRACK_TRANSITION_APPLIED`.
- Should transition writes be:
  - event-only (append and projected),
  - dual-write (state table + outbox),
  - or direct DB write only (with webhook replay)?
- What exact request/body fields are required?
  - Suggested minimum: `track_id`, `from_state`, `to_state`, `actor_id`, `actor_role`, `reason_code`, `timestamp`, `request_id`, `idempotency_key`.
- What is the idempotency rule?
  - Unique `(track_id, request_id)` or `(track_id, from_state, to_state, actor_id, timestamp_bucket)`?
- Which actor roles are allowed per transition, and where is it validated (app vs Tandang)?
- How is quorum/consent windows represented (fields and timer semantics)?

### 2) Siaga (`UI-08`) dedicated contract
- Does Siaga live in:
  - Tandang event stream, or
  - app-local event bus only?
- If in stream, what is the event name and payload contract?
  - Suggested fields: `siaga_id`, `scope_id`, `seed_id?`, `severity`, `location`, `title`, `instructions`, `start_at`, `escalation_tier`, `status`, `responder_ids`, `closure_reason`.
- How is lifecycle closure recorded (`AUTO_RESOLVE`, `CLOSED_BY_RESPONDER`, `CANCELLED`)?
- Should a broadcast message be auditable with immutable timeline points?

### 3) Moderation visibility (`UI-07`)
- Where does moderation source truth live?
  - AI suggestion service, app authoritative state, or Tandang metadata.
- What fields are required on feed/problem/user-visible objects?
  - Suggested: `moderation_status`, `reason_code`, `risk_score`, `action`, `visible_to`, `visible_from`, `expires_at`.
- What should be visible on public view vs author/moderator view?
- Should moderation state update Tandang (`subject.metadata`) or stay in app DB?

### 4) Vault / Catatan Saksi lifecycle (`UI-09`)
- Is vault content fully out-of-scope for Tandang persistence?
- If partially persisted, what is minimum indexable field set?
  - Suggested: `witness_record_id`, `sealed_hash`, `scope_id`, `visibility`, `retention_ttl`, `created_at`, `sealed_at`.
- Define event points: `WITNESS_DRAFTED`, `WITNESS_SEALED`, `WITNESS_PUBLISHED`, `WITNESS_EXPIRED`, `WITNESS_REVOKED`.
- Should any deletion / retention events be queryable for audit UI?

### 5) Medium blockers (`UI-10`..`UI-12`) ownership split
- For `feed/search/notification`, decide ownership:
  - `Tandang-powered only`,
  - `app-owned only`, or
  - `hybrid`.
- If hybrid:
  - what belongs in Tandang and what is computed from local read-model caches?
- Confirm API path conventions and pagination model if Tandang-backed.

## Priority B — AI/Edge-Pod prompt alignment approvals
- Confirm and register prompt IDs:
  - `AI-03` -> `DUPLICATE-001` (before EP-03)
  - `AI-05` -> `GAMING-001` (before EP-05)
  - `AI-08` -> `SENSITIVE-001` (before EP-08)
- Confirm/deny companion scoring prompt strategy for `AI-09`:
  - Reuse `CREDIT-001` if acceptable, or
  - register new `CREDIT-SCORE-001`.

## Acceptance criteria for each answer
- Each unanswered question must be resolved with:
  - owner,
  - decision (e.g., `OWNED_BY_TANDING`, `APP_ONLY`),
  - one canonical source path,
  - and a concrete schema note that can be translated into contract rows.

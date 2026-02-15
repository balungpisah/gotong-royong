# Decision Notes (Compiled)

Status: READY FOR BACKEND DESIGN HANDOFF

Update date: `2026-02-14`
Meeting owner: Research coordinator

## 1) Track workflow transitions (`UI-03`)
- Decision: App-owned transition model.
- Ownership: Gotong Backend (Workflow/State Machine), app-only.
- Canonical event: `track_state_transition`.
- Trigger model: user_action, timer, webhook-compatible.
- Required fields: `request_id`, `transition_id`, `entity_id`, `track`, `from_stage`, `to_stage`, `transition_type`, `mechanism`, `actor.user_id`, `actor.role`, `occurred_at`, `gate.gate_type`, `gate.opens_at`, `gate.closes_at`, `gate.status`, `correlation_id`.
- Idempotency rule: replay-safe per `(entity_id, request_id)`; timer/system requests use deterministic `request_id` `timer:{transition_id}:{gate.closes_at}`.
- Role rules captured: `propose` by `author`/`pic`; `object` by `participant|saksi|author|pic`; `vote` by `author|saksi|participant|pic` (snapshot at gate open); timer close by `system`.
- Gate model: single gate object with optional quorum / threshold fields and `vote_tally`.
- Next action: persist in `docs/design/backend-design-contract-gotong-tandang.md` for implementation.

## 2) Moderation visibility (`UI-07`)
- Decision: App-owned source-of-truth for moderation state.
- Persistence: `content` and `moderation_decisions` in Gotong DB.
- Public visibility fields: `content_id`, `moderation_status`, `published_at` (if approved), `warning_label`, `redacted_summary`.
- Author-only fields: includes `moderation_status`, `hold_expires_at`, `auto_release_if_no_action`, `reason_code`, `violation_categories`, `appeal_window_until`.
- Moderator-only fields: full violation details and full decision history.
- Evidence source: AI-04 + moderator actions combined into audit trail.

## 3) Siaga emergency flow (`UI-08`)
- Decision: App-owned emergency model (no Tandang prompt for now).
- Canonical event names:
  - `siaga_broadcast_created`
  - `siaga_broadcast_activated`
  - `siaga_broadcast_updated`
  - `siaga_responder_joined`
  - `siaga_responder_updated`
  - `siaga_broadcast_closed`
  - `siaga_broadcast_cancelled`
- Required timeline model: append-only events; closure requires explicit close reason and summary.
- Retention: responder identities retained 7 days, then anonymized; aggregate stats retained longer.

## 4) Catatan Saksi vault (`UI-09`)
- Decision: App-only persistence with minimal indexed tombstones.
- Canonical metadata: `vault_entry_id`, `state`, timestamps, `sealed_hash`, `encryption_key_id`, `attachment_refs`, `wali` permissions, retention policy.
- Lifecycle events:
  - `witness_drafted`
  - `witness_sealed`
  - `witness_trustee_added`
  - `witness_trustee_removed`
  - `witness_published`
  - `witness_revoked`
  - `witness_expired`
- Retention/revocation: `draft` editable; `sealed` immutable and revocable access; `published` irreversible.

## 5) Feed/search/notification ownership (`UI-10`..`UI-12`)
- Feed (`UI-10`): app-owned cursor API, `cursor`/`limit`, privacy filters; reputation from Tandang as read-only signal.
- Search (`UI-11`): app-owned search index/projection; Tandang not authoritative for ranking.
- Notifications (`UI-12`): app-owned notification feed and digest scheduler.

## 6) Prompt IDs and AI contract routing
- `AI-03` -> `DUPLICATE-001` (`REGISTERED`)
- `AI-05` -> `GAMING-001` (`REGISTERED`)
- `AI-08` -> `SENSITIVE-001` (`REGISTERED`)
- `AI-09` -> `CREDIT-001` only (`REGISTERED`, no companion required in current scope)

## 7) Approvers
- AI Lead
- Community Lead
- PM (backend design lock)


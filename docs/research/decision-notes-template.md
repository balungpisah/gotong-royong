# Decision Notes Template (Research Handoff)

Use this when running the blocker-resolution round.  
Record each decision directly into `tandang-gap-log.md` and `feature-contract-map.md`.

## Metadata
- Meeting date:
- Facilitator:
- Attendees:
- Scope:
- Output version:

## Decision Log

### 1) Track transitions (`UI-03`)
- Decision:
- Owner:
- Canonical event name:
- Contract fields (final):
- Idempotency model:
- Role validation rule:
- Notes:

### 2) Siaga (`UI-08`)
- Decision:
- Owner:
- Event name(s):
- Payload contract:
- Lifecycle transitions:
- Audit/visibility requirements:

### 3) Moderation visibility (`UI-07`)
- Decision:
- Owner:
- Source of truth:
- Public vs author/moderator field exposure:
- Where persisted:

### 4) Vault/Catatan Saksi (`UI-09`)
- Decision:
- Owner:
- In/out-of-band persistence choice:
- Event set:
- Audit retention policy:

### 5) Feed/Search/Notification ownership (`UI-10`..`UI-12`)
- `UI-10` Feed decision:
- `UI-11` Search decision:
- `UI-12` Notification decision:
- Shared schema prefix / pagination model:
- Responsibility split:

### 6) AI prompt registrations
- `AI-03` prompt id:
- `AI-05` prompt id:
- `AI-08` prompt id:
- `AI-09` companion scoring strategy:
- Sign-off owners:

## Immediate Handoff Outputs (required)
- `docs/research/tandang-gap-log.md` rows completed for HIGH blockers.
- `docs/research/feature-contract-map.md` status changed from `UNKNOWN` to `KNOWN/PARTIAL`.
- `docs/research/prompt-id-registration-log.md` updated from `PENDING` to `REGISTERED`.
- `docs/research/edgepod-endpoint-contracts.md` and schema references aligned to approved IDs.


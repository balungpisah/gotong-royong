# Triage -> Witness -> Feed Contract (Canonical v1)

Last updated: 2026-02-26
Owner: frontend + API cutover slice

This file is the single source of truth for the runtime contract between:
- `POST /v1/triage/sessions`
- `POST /v1/triage/sessions/:session_id/messages`
- `POST /v1/witnesses`
- `POST /v1/witnesses/:witness_id/stempel/propose`
- `POST /v1/witnesses/:witness_id/stempel/objections`
- `POST /v1/witnesses/:witness_id/stempel/finalize`

Supersedes legacy draft contracts in:
- `docs/design/specs/ai-spec/04a-ai-00-edge-contract.md`
- `docs/research/contracts/edgepod-endpoint-contracts.md`

## 1) Contract Versioning

- `schema_version` is mandatory and must be exactly `triage.v1`.
- Backend rejects mismatched versions with `400 validation_error`.
- `POST /v1/witnesses` is session-authoritative: client cannot send raw `triage_result`.

## 2) Triage Result Envelope (`triage.v1`)

Every triage response includes:

```json
{
  "session_id": "triage-sess-...",
  "result": {
    "schema_version": "triage.v1",
    "status": "draft | final",
    "kind": "witness | data",
    "missing_fields": ["..."],
    "blocks": {
      "conversation": ["ai_inline_card", "diff_card"],
      "structured": ["document", "list", "computed"]
    },
    "structured_payload": [
      { "type": "document", "id": "triage-doc-1", "...": "block payload" }
    ],
    "conversation_payload": [
      { "type": "ai_card", "message_id": "triage-ai-card-1", "...": "chat card payload" }
    ],
    "taxonomy": {
      "category_code": "infrastructure",
      "category_label": "Laporan Warga",
      "quality": "community_observation"
    },
    "program_refs": [
      {
        "program_id": "program:mbg",
        "label": "Makan Bergizi Gratis",
        "source": "llm_inferred",
        "confidence": 0.82
      }
    ],
    "stempel_state": {
      "state": "draft",
      "min_participants": 3,
      "participant_count": 0,
      "objection_count": 0
    },
    "bar_state": "probing | leaning | ready | vault-ready | siaga-ready",
    "route": "komunitas | vault | siaga | catatan_komunitas | kelola",
    "track_hint": "tuntaskan",
    "seed_hint": "Keresahan",
    "summary_text": "...",
    "card": {
      "icon": "construction",
      "trajectory_type": "aksi",
      "title": "...",
      "hook_line": "...",
      "body": "...",
      "sentiment": "curious",
      "intensity": 2
    },
    "confidence": { "score": 0.95, "label": "..." },
    "budget": { "total_tokens": 6000, "used_tokens": 1800, "remaining_tokens": 4200, "budget_pct": 0.3, "can_continue": true, "turn_count": 2, "max_turns": 8 }
  },
  "ai_message": "..."
}
```

`status` semantics:
- `draft`: triage is incomplete; client should continue conversation.
- `final`: triage is complete and can be materialized.

`blocks` semantics:
- optional in `draft`, required in `final` when triage is derived from `operator.v1`.
- `conversation[]` declares chat-layer blocks (e.g. `ai_inline_card`, `diff_card`, `vote_card`).
- `structured[]` declares structured-layer primitives (`list|document|form|computed|display|vote|reference`).

`structured_payload` semantics:
- optional in `draft`, expected in `final` when backend can materialize structured output.
- items are renderable block objects matching one primitive:
  `list | document | form | computed | display | vote | reference`.

`conversation_payload` semantics:
- optional in `draft`, expected in `final` when backend can materialize chat cards.
- items are renderable chat card messages (`ai_card | diff_card | vote_card`).

`kind` semantics:
- `witness`: lifecycle card candidate; valid input for `POST /v1/witnesses`.
- `data`: one-off data card candidate; not valid for `POST /v1/witnesses`.

### 2.0 Optional Operator Handoff Input (`operator.v1`)

`POST /v1/triage/sessions` and `POST /v1/triage/sessions/:session_id/messages` may include
optional `operator_output` payload:

```json
{
  "content": "....",
  "operator_output": {
    "schema_version": "operator.v1",
    "...": "triage operator envelope"
  }
}
```

Rules:
- when `operator_output` is present, backend validates it against `operator.v1` constraints
  before mapping to runtime `triage.v1`.
- invalid `operator_output` is rejected with standard `500 internal_error` envelope in current
  runtime behavior (hard gate, no silent coercion).
- when `operator_output` is omitted, backend uses internal fallback operator synthesis.

### 2.1 Data taxonomy (controlled-hybrid)

`result.taxonomy.category_code` must use a controlled vocabulary:
- `commodity_price`
- `public_service`
- `training`
- `employment`
- `health`
- `education`
- `infrastructure`
- `safety_alert`
- `environment`
- `community_event`
- `other_custom`

If `other_custom` is used, backend/frontend may carry `custom_label` for human display.

`result.taxonomy.quality`:
- `official_source`
- `community_observation`
- `unverified_claim`

### 2.2 Program identifiers

Program references use structured `program_refs[]` objects:
- `program_id` (machine id, e.g. `program:mbg`)
- `label` (human name)
- `source` (e.g. `llm_inferred`, `user_selected`, `registry`)
- `confidence` (0..1)

## 3) Witness Create (Session-Only)

Endpoint:
- `POST /v1/witnesses`

Request:

```json
{
  "schema_version": "triage.v1",
  "triage_session_id": "triage-sess-..."
}
```

Rules:
- Session owner must match authenticated user.
- Backend uses server-stored triage state/transcript only.
- Non-LLM fields (author, timestamps, IDs, persisted status) are backend-generated.

Responses:

- `201 Created` (triage `status=final` and `kind=witness`)

```json
{
  "witness_id": "....",
  "title": "...",
  "summary": "...",
  "track_hint": "tuntaskan",
  "seed_hint": "Keresahan",
  "rahasia_level": "L0",
  "author_id": "user-...",
  "created_at_ms": 1760000000000,
  "taxonomy": { "...": "copied from triage result" },
  "program_refs": [{ "...": "program refs" }],
  "stempel_state": { "...": "optional, present for mufakat-like flows" },
  "impact_verification": {
    "status": "not_open",
    "opened_at_ms": null,
    "closes_at_ms": null,
    "yes_count": 0,
    "no_count": 0,
    "min_vouches": 3
  },
  "stream_item": {
    "kind": "witness",
    "stream_id": "w-...",
    "sort_timestamp": "2026-02-26T00:00:00Z",
    "data": { "...": "feed item payload from discovery ingest" }
  }
}
```

- `409 Conflict` (triage still `draft`)

```json
{
  "error": {
    "code": "triage_incomplete",
    "message": "triage session is still draft",
    "details": {
      "triage_session_id": "triage-sess-...",
      "status": "draft"
    }
  },
  "missing_fields": ["..."]
}
```

## 4) Stempel lifecycle contract (backend-authoritative)

### 4.1 Propose consensus

Endpoint:
- `POST /v1/witnesses/:witness_id/stempel/propose`

Request (default window 24h):

```json
{
  "summary": "Kesimpulan diskusi...",
  "rationale": "Poin keberatan sudah ditutup",
  "objection_window_seconds": 86400
}
```

Response:
- `200 OK` with `stempel_state.state = objection_window`
- `409 Conflict` with `error.code = stempel_already_locked` when already locked

### 4.2 Submit objection

Endpoint:
- `POST /v1/witnesses/:witness_id/stempel/objections`

Request:

```json
{
  "reason": "Masih ada data yang belum tervalidasi"
}
```

Response:
- `201 Created` with updated `stempel_state` (including objection counters)
- `409 Conflict` when window is not open or already closed

### 4.3 Finalize lock

Endpoint:
- `POST /v1/witnesses/:witness_id/stempel/finalize`

Locking rules (default):
- minimum participants: `3`
- no active objections
- objection window elapsed

Response:
- `200 OK` with `stempel_state.state = locked`
- on successful lock, backend opens `impact_verification.status = open`
- `409 Conflict` for unmet conditions (`window open`, `has objection`, `participant threshold not met`)

## 5) PathPlan phase assist contract

`proposed_plan.branches[].phases[]` may include:

```json
{
  "assist_needs": [
    {
      "esco_skill_uri": "http://data.europa.eu/esco/skill/...",
      "skill_label": "Survey lapangan",
      "reason": "Butuh validasi lapangan",
      "urgency": "low | medium | high",
      "min_people": 2
    }
  ]
}
```

This is the canonical placement for phase-to-ESCO demand used by notification routing.

## 6) Feed materialization and impact verification

- Witness create writes the contribution and ingests discovery feed in one backend flow.
- Response `stream_item` is renderable by frontend feed stream renderer.
- Frontend must not synthesize local witness feed cards after create.
- Feed payload may carry:
  - `program_refs`
  - `stempel_state`
  - `impact_verification`
  - `dev_meta` (optional; non-production seeded cards only)

### 6.1 Dev seeded feed metadata (`dev_meta`)

`stream_item.data.dev_meta` is optional and only intended for development/test seeded cards.
Production-authored cards should omit this object.

```json
{
  "dev_meta": {
    "is_seed": true,
    "seed_batch_id": "fixture-taxonomy-v1",
    "seed_origin": "fixture | db | operator_stub"
  }
}
```

Rules:
- `is_seed` is required when `dev_meta` is present.
- `seed_batch_id` is optional but recommended for matrix/audit runs.
- `seed_origin` is optional and constrained to `fixture | db | operator_stub`.
- Frontend may show explicit `SEED` affordances only in development mode.

## 7) Related Operator Contract

Operator-side JSON output (consultation layer) is defined in:
- `docs/research/contracts/triage-operator-output-contract-v1.md`
- `docs/research/contracts/triage-operator-output-contract-v1.schema.json`

`operator.v1` outputs (`triage_draft | triage_final`) are mapped into this runtime
contract (`triage.v1`) before API response persistence.

## 8) Related Crosswalk

Trajectory taxonomy should be cross-referenced with Tandang signal behavior using:
- `docs/research/contracts/trajectory-tandang-signal-crosswalk-v1.md`

Seeded card metadata conventions for dev/test coverage are defined in:
- `docs/research/contracts/feed-seed-metadata-v1.md`

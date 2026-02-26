# Triage Operator Output Contract (Canonical v1)

Last updated: 2026-02-26  
Owner: AI orchestration + API cutover slice

This file is the single source of truth for operator JSON output consumed by the AI orchestrator during triage.

Scope:
- operator -> orchestrator consultation outputs (draft and final)
- operator taxonomy mapping to `kind` (`witness | data | kelola`)
- operator-specific `payload` final shape requirements

Related runtime contract:
- `docs/research/contracts/triage-witness-feed-contract-v1.md`

Supersedes runtime interpretation drift from:
- `docs/design/specs/ai-spec/04c-operator-skill-map.md` (kept as design reference)

## 1) Versioning

- `schema_version` is mandatory and must be exactly `operator.v1`.
- Draft and final outputs use the same envelope.
- `triage_stage` defines whether payload may be partial or must be complete.

## 2) Stage Semantics

| `triage_stage` | Meaning | Orchestrator behavior | API status mapping |
|---|---|---|---|
| `triage_draft` | Information still missing | ask follow-up questions from `questions[]` and `missing_fields[]` | `result.status = draft` |
| `triage_final` | Information sufficient to materialize | stop probing, map to final triage result and downstream create flow | `result.status = final` |

Rules:
- `triage_draft` may return partial `payload`.
- `triage_final` must return fully valid operator payload for its operator type.

## 3) Canonical Envelope (`operator.v1`)

```json
{
  "schema_version": "operator.v1",
  "operator": "musyawarah",
  "triage_stage": "triage_draft",
  "output_kind": "witness",
  "confidence": 0.77,
  "checklist": [
    {
      "field": "stakeholders",
      "filled": false,
      "required_for_final": true
    }
  ],
  "questions": [
    "Siapa pihak yang harus ikut mengambil keputusan?"
  ],
  "missing_fields": ["stakeholders"],
  "routing": {
    "route": "komunitas",
    "trajectory_type": "mufakat",
    "track_hint": "obrolkan",
    "seed_hint": "Aspirasi"
  },
  "payload": {
    "context": "proposal",
    "decision_steps": [
      {
        "question": "Apakah agenda disetujui untuk dibahas minggu ini?",
        "rationale": "Perlu titik mulai yang jelas",
        "order": 1
      }
    ]
  }
}
```

Top-level fields:
- `operator`: `masalah | musyawarah | pantau | catat | bantuan | rayakan | siaga | program | kelola`
- `triage_stage`: `triage_draft | triage_final`
- `output_kind`: `witness | data | kelola`
- `confidence`: number `0..1`
- `checklist[]`: progress items (`field`, `filled`, optional `value`, `required_for_final`)
- `questions[]`: follow-up prompts for next turn
- `missing_fields[]`: unresolved required fields
- `routing`: routing/taxonomy metadata
- `payload`: operator-specific JSON (partial in draft, full in final)

### 3.1 Final Output Examples by Kind

`witness` final (`masalah`):

```json
{
  "schema_version": "operator.v1",
  "operator": "masalah",
  "triage_stage": "triage_final",
  "output_kind": "witness",
  "checklist": [{ "field": "problem_scope", "filled": true, "required_for_final": true }],
  "routing": { "route": "komunitas", "trajectory_type": "aksi" },
  "payload": { "trajectory": "A", "path_plan": { "plan_id": "plan-1", "version": 1, "title": "...", "summary": "...", "branches": [{ "branch_id": "main", "label": "Utama", "parent_checkpoint_id": null, "phases": [{ "phase_id": "p1", "title": "...", "objective": "...", "status": "planned", "source": "ai", "locked_fields": [], "checkpoints": [{ "checkpoint_id": "c1", "title": "...", "status": "open", "source": "ai", "locked_fields": [] }] }] }] } }
}
```

`data` final (`catat`):

```json
{
  "schema_version": "operator.v1",
  "operator": "catat",
  "triage_stage": "triage_final",
  "output_kind": "data",
  "checklist": [{ "field": "claim", "filled": true, "required_for_final": true }],
  "routing": { "route": "catatan_komunitas", "trajectory_type": "data", "taxonomy": { "category_code": "commodity_price", "category_label": "Harga Komoditas", "quality": "community_observation" } },
  "payload": { "record_type": "data", "claim": "Harga telur Rp32.000/kg", "observed_at": "2026-02-26T06:00:00Z", "category": "harga_pangan" }
}
```

`kelola` final:

```json
{
  "schema_version": "operator.v1",
  "operator": "kelola",
  "triage_stage": "triage_final",
  "output_kind": "kelola",
  "checklist": [{ "field": "group_name", "filled": true, "required_for_final": true }],
  "routing": { "route": "kelola" },
  "payload": { "action": "create", "group_detail": { "name": "Ronda RT 04", "description": "Koordinasi ronda malam", "join_policy": "persetujuan", "entity_type": "kelompok" } }
}
```

## 4) Operator Taxonomy Matrix

| Operator | Trajectory targets | `output_kind` | Final payload contract |
|---|---|---|---|
| `masalah` | `aksi | advokasi` | `witness` | `MasalahPayload` |
| `musyawarah` | `mufakat | mediasi` | `witness` | `MusyawarahPayload` |
| `pantau` | `pantau` | `witness` | `PantauPayload` |
| `program` | `program` | `witness` | `ProgramPayload` |
| `catat` | `data | vault` | `data` | `CatatPayload` |
| `bantuan` | `bantuan` | `data` | `BantuanPayload` |
| `rayakan` | `pencapaian` | `data` | `RayakanPayload` |
| `siaga` | `siaga` | `data` | `SiagaPayload` |
| `kelola` | none (group lifecycle) | `kelola` | `KelolaPayload` |

Consistency requirements:
- `operator=kelola` -> `output_kind=kelola` and `routing.route=kelola`
- `output_kind=data` -> `routing.taxonomy` required
- `output_kind=witness` -> `routing.trajectory_type` must be one of `aksi|advokasi|pantau|mufakat|mediasi|program`

## 5) Final Payload Contracts (when `triage_stage=triage_final`)

### 5.1 `MasalahPayload`
Required:
- `trajectory` (`A | B`)
- `path_plan` (full path plan)

### 5.2 `MusyawarahPayload`
Required:
- `context` (`proposal | dispute`)
- `decision_steps[]` (at least one)
Optional:
- `on_consensus` (`spawn_aksi`)
- `stempel_candidate` (`summary`, `rationale`, optional `objection_window_seconds`)

### 5.3 `PantauPayload`
Required:
- `case_type`
- `timeline_seed[]` (at least one event)
- `tracking_points[]` (at least one)

### 5.4 `CatatPayload`
Required:
- `record_type` (`data | vault`)
- `claim`
- `observed_at` (ISO 8601)
- `category`
Optional:
- `location`, `proof_url`, `hash`

### 5.5 `BantuanPayload`
Required:
- `help_type`
- `description`
- `urgency` (`rendah | sedang | tinggi`)
- `matched_resources[]`

### 5.6 `RayakanPayload`
Required:
- `achievement`
- `contributors[]`
- `impact_summary`
Optional:
- `linked_witness_id`

### 5.7 `SiagaPayload`
Required:
- `threat_type`
- `severity` (`waspada | siaga | darurat`)
- `location`
- `description`
- `source`
- `expires_at` (ISO 8601)

### 5.8 `ProgramPayload`
Required:
- `activity_name`
- `frequency` (`harian | mingguan | bulanan | custom`)
- `rotation[]`
Optional:
- `frequency_detail`, `location`, `next_occurrence`

### 5.9 `KelolaPayload`
Required:
- `action` (`create | edit | invite | join | leave`)
Optional by action:
- `group_detail` for `create/edit`
- `group_id` for `edit/invite/join/leave`
- `invited_user_ids[]` for `invite`

## 6) Routing Metadata Contract

`routing` contains orchestration metadata independent from payload internals:

```json
{
  "route": "komunitas",
  "trajectory_type": "mufakat",
  "track_hint": "obrolkan",
  "seed_hint": "Aspirasi",
  "taxonomy": {
    "category_code": "public_service",
    "category_label": "Layanan Publik",
    "quality": "community_observation"
  },
  "program_refs": [
    {
      "program_id": "program:mbg",
      "label": "Makan Bergizi Gratis",
      "source": "llm_inferred",
      "confidence": 0.82
    }
  ]
}
```

For `mufakat`/`mediasi`, `routing.stempel_state` may carry draft governance context (`draft | proposed | objection_window | locked`).

## 7) Non-LLM Fields (Programmatically Filled)

The following are not trusted from operator output and must be filled by backend/application logic:
- identity and ownership: `author_id`, `group_id`, permissions
- persistence IDs: `witness_id`, `stream_id`, row keys
- timestamps: `created_at_ms`, `updated_at_ms`, lock/vote timestamps
- counters/aggregates: signal counts, participant counts, vouch totals
- final lifecycle state transitions and write ordering guarantees

## 8) Validation Artifacts

Strict JSON Schema:
- `docs/research/contracts/triage-operator-output-contract-v1.schema.json`

Validation gate expectation:
- validate every operator output JSON against this schema before mapping to runtime `TriageResult`.

## 9) Contract Relationships

- Runtime triage/witness/feed API contract: `docs/research/contracts/triage-witness-feed-contract-v1.md`
- Trajectory/signal crosswalk and gap telemetry: `docs/research/contracts/trajectory-tandang-signal-crosswalk-v1.md`

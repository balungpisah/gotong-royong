# Edge-Pod Endpoint Contracts for Gotong-Royong (Research Handoff)

This document is the practical handoff from Gotong-Royong research to Edge-Pod implementation.

Goal:
1. one endpoint per task family,
2. exact input required by Gotong-Royong,
3. strict output contract,
4. fallback behavior if model/policy path fails.

## Standard request envelope (apply to all endpoints)

| Field | Type | Required | Notes |
|---|---|---|---|
| `request_id` | string | yes | idempotency token from caller |
| `correlation_id` | string | yes | trace across webhook/jobs/UI events |
| `actor` | object | yes | `{ user_id, platform_user_id, role }` |
| `session_id` | string | no | only when used for AI-00/AI-01 flow continuity |
| `trigger` | enum | yes | `user_action` \| `timer` \| `webhook` \| `async_batch` |
| `payload_version` | string | yes | currently `2026-02-14` |
| `privacy_level` | enum | no | `open` \| `l1` \| `l2` \| `vault` |

Standard success envelope:

```json
{
  "request_id": "req_...",
  "result_version": "v0.2.0",
  "output": { /* task-specific object */ },
  "confidence": 0.92,
  "reason_code": "OK",
  "actor_context": {}
}
```

Standard error envelope:

```json
{
  "request_id": "req_...",
  "result_version": "v0.2.0",
  "error": {
    "code": "MODEL_UNAVAILABLE",
    "message": "fallback_used",
    "retryable": false
  }
}
```

## Edge-Pod Endpoint Matrix (Phase 1)

| ID | Endpoint | UI features | Trigger | Input (required) | Output | Prompt / version | Failure contract |
|---|---|---|---|---|---|---|---|
| EP-00 | `POST /api/v1/edge-pod/ai/00/triage` | UI-01, UI-09 | `user_action` (`Bagikan` first message), `user_action` (`publish` in vault path) | `actor`, `session_id`, `text`, `media_urls`, `location`, `voice_transcript`, `attachments_count`, `privacy_signal`, `rencana_rutin` | `entry_flow`, `track`, `seed_type`, `context_bar_state`, `confidence`, `reasoning`, `follow_up_question`, `is_split_candidate`, optional `split_candidates` | `14-prompt-versioning.md` -> `TRIAGE-001` (`v0.2.0`) | fallback to manual state path and keep draft open; no hard reject |
| EP-01 | `POST /api/v1/edge-pod/ai/01/classification` | UI-02, UI-20 | `user_action` (`Ubah` / `Pilih sendiri`) | `text`, `seed_candidate_id`, `session_id`, `location`, `conversation_context`, `suggested_track`, `seed_type_hint` | `track`, `seed_type`, `track_confidence`, `seed_confidence`, `esco_skills`, `ambiguity_flag`, `alternative_track`, `reasoning` | `14-prompt-versioning.md` -> `CLASS-001` (`v0.2.0`) | if model unavailable, return `manual_required=true`, empty classification body |
| EP-02 | `POST /api/v1/edge-pod/ai/02/redaction` | UI-01, UI-07 | `user_action` (`save` / `submit`) | `text`, `text_format`, `user_id`, `context_summary`, `privacy_level`, `allow_redaction` | `redacted_text`, `redacted_items`, `redaction_count`, `needs_manual_review`, `confidence`, `reason_code` | `14-prompt-versioning.md` -> `REDACT-001` (`v0.2.0`) | fallback to unmodified text + `needs_manual_review=true` |
| EP-03 | `POST /api/v1/edge-pod/ai/03/duplicate-detection` | UI-23 | `user_action` (`submit` / `pill-open`) | `seed_text`, `media_hashes`, `embedding`, `location`, `radius_km`, `scope`, `exclude_seed_ids`, `query_options` | `matches`, `top_match`, `recommendation`, `distance_km`, `confidence`, `auto_block` | `07-ai-03-duplicate-detection.md` + `DUPLICATE-001` | fallback to proceed with no-blocking warning and async retry |
| EP-04 | `POST /api/v1/edge-pod/ai/04/moderation` | UI-07 | `user_action`, `timer` (`review expiry`), `webhook` | `content_id`, `text`, `attachments`, `author_id`, `author_reputation`, `track`, `seed_type`, `location`, `report_volume` | `status`, `violations`, `action`, `confidence`, `reasoning`, `hold_duration_minutes`, `auto_release_if_no_action` | `14-prompt-versioning.md` -> `MOD-001` (`v0.2.0`) | fallback `status=manual_review`, `action=hold`, `reason_code=policy_fallback` |
| EP-05 | `POST /api/v1/edge-pod/ai/05/gaming-risk` | UI-05, UI-19 | `timer`, `webhook`, periodic `async_batch` | `query_users`, `seed_ids`, `focus_metric`, `lookback_hours`, `platform`, `window_start`, `window_end` | `flags`, `summary.total_flags`, `summary.critical_count`, `recommendation` | `09-ai-05-gaming-detection.md` + `GAMING-001` | fallback non-blocking; continue normal path |
| EP-06 | `POST /api/v1/edge-pod/ai/06/criteria-suggestions` | UI-03, UI-20 | `user_action` (`stage_transition`) | `seed_id`, `track`, `seed_text`, `discussion_summary`, `community_context`, `policy_scope` | `suggested_criteria`, `suggested_tasks`, `task_decomposition_confidence`, `user_agency_note` | `14-prompt-versioning.md` → `CRITERIA-001` (`v0.2.0`) | fallback empty suggestions + ask user input |
| EP-07 | `POST /api/v1/edge-pod/ai/07/summary` | UI-11, UI-12 | `user_action` (query), `timer` (digest), `async_batch` | `input_texts`, `messages`, `scope`, `filters`, `window_start`, `window_end`, `max_items` | `summary`, `key_points`, `sentiment`, `action_items`, `controversies`, `clusters` | `11-ai-07-discussion-summarization.md` -> `SUM-001` (`v0.2.0`) | fallback return `raw_items=true`, no summary |
| EP-08 | `POST /api/v1/edge-pod/ai/08/sensitive-media` | UI-04 | `user_action` (`upload`/`verify`) | `media_urls`, `media_types`, `seed_id`, `author_id`, `seed_text`, `hash_chain` | `scans`, `overall_safety`, `detections`, `redacted_media_url`, `summary`, `is_actionable` | `12-ai-08-sensitive-media.md` + `SENSITIVE-001` | fallback raw media + manual moderation |
| EP-09 | `POST /api/v1/edge-pod/ai/09/credit-recommendation` | UI-06, UI-05, UI-20 | `timer`, `user_action` (`tuntas`) | `user_id`, `timeline_events`, `contrib_events`, `skill_profile`, `reputation_snapshot` | `candidate_allocations`, `confidence`, `reasoning`, `dispute_window`, `confidence_source` | `13-ai-09-credit-accreditation.md` (`CREDIT-001`) | fallback manual credit form only |
| EP-10 | `POST /api/v1/edge-pod/ai/10/skill-extract` | UI-13 | `user_action` (`seed_submit`, `profile_edit`) | `text`, `language`, `seed_context`, `existing_skills`, `self_declared` | `esco_skills`, `skill_uri`, `score`, `source`, `validated` | `ESCO extraction` (not in AI touchpoint table) | fallback local heuristic or empty proposals |
| EP-11 | `POST /api/v1/edge-pod/ai/siaga/evaluate` | UI-08 | `user_action` (`Siarkan Sekarang`), `AI-00` trigger | `actor`, `text`, `location`, `confidence_bundle`, `reported_urgency`, `community_scope`, `current_track` | `is_siaga`, `severity`, `responder_payload`, `scope`, `timeline_window`, `override_policy`, `confidence` | `TRIAGE-001` + `MOD-001` (composition path) | fallback manual emergency path, create Siaga draft with human approval queue |

## Prompt bundle for implementation tickets (copy as issue descriptions)

Use these prompt headings when creating Edge-Pod endpoint tasks.

- `PROMPT-EP-00`: "Implement `/api/v1/edge-pod/ai/00/triage` endpoint using prompt `TRIAGE-001` with request envelope fields, typed JSON output schema, and fallback path."
- `PROMPT-EP-01`: "Implement `/api/v1/edge-pod/ai/01/classification` endpoint for manual override and ambiguity metadata."
- `PROMPT-EP-02`: "Implement `/api/v1/edge-pod/ai/02/redaction` endpoint with `text` masking and `redaction_count` guarantees."
- `PROMPT-EP-03`: "Implement `/api/v1/edge-pod/ai/03/duplicate-detection` endpoint using `DUPLICATE-001`."
- `PROMPT-EP-04`: "Implement `/api/v1/edge-pod/ai/04/moderation` endpoint with deterministic status mapping (`hold`, `publish`, `queue_review`)."
- `PROMPT-EP-05`: "Implement `/api/v1/edge-pod/ai/05/gaming-risk` endpoint using `GAMING-001` and make flags advisory by default."
- `PROMPT-EP-06`: "Implement `/api/v1/edge-pod/ai/06/criteria-suggestions` endpoint using `CRITERIA-001` and support idempotent re-request."
- `PROMPT-EP-07`: "Implement `/api/v1/edge-pod/ai/07/summary` endpoint for digest and search snippets with period-aware ranking."
- `PROMPT-EP-08`: "Implement `/api/v1/edge-pod/ai/08/sensitive-media` endpoint with `SENSITIVE-001`, media_url whitelist, and evidence hash handling."
- `PROMPT-EP-09`: "Implement `/api/v1/edge-pod/ai/09/credit-recommendation` endpoint with non-blocking advisory outputs."
- `PROMPT-EP-10`: "Implement `/api/v1/edge-pod/ai/10/skill-extract` endpoint for ESCO extraction and validated vs self_declared fields."
- `PROMPT-EP-11`: "Implement `/api/v1/edge-pod/ai/siaga/evaluate` endpoint with bounded emergency scoring, responder payload, and no silent suppression."

## Strict JSON schemas for Gotong-Royong consumption

Canonical strict schema is stored in:
`docs/research/edgepod-endpoint-contracts.schema.json`

Compact endpoint map is stored in:
`docs/research/edgepod-endpoint-contracts.contract-map.md`

Per-endpoint strict schemas are stored in:
`docs/research/edgepod-endpoint-contracts/EP-00.schema.json` ... `EP-11.schema.json`

The inline blocks below are kept for traceability only and are not the canonical contract.

<!-- Canonical contract for gotong-royong consumption. Legacy inline schema blocks below for context only. -->

### Shared envelope + actor schema (draft-07)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "Actor": {
      "type": "object",
      "required": ["user_id", "platform_user_id", "role"],
      "properties": {
        "user_id": { "type": "string", "minLength": 1 },
        "platform_user_id": { "type": "string", "minLength": 1 },
        "role": { "type": "string", "enum": ["member", "admin", "moderator", "humas", "bendahara", "pic", "system"] }
      },
      "additionalProperties": false
    },
    "SuccessEnvelope": {
      "type": "object",
      "required": ["request_id", "result_version", "output"],
      "properties": {
        "request_id": { "type": "string", "minLength": 1 },
        "result_version": { "type": "string", "pattern": "^v\\d+\\.\\d+\\.\\d+$" },
        "output": { "type": "object" },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "reason_code": { "type": "string" },
        "actor_context": { "type": "object", "additionalProperties": true }
      },
      "additionalProperties": true
    },
    "ErrorEnvelope": {
      "type": "object",
      "required": ["request_id", "result_version", "error"],
      "properties": {
        "request_id": { "type": "string", "minLength": 1 },
        "result_version": { "type": "string", "pattern": "^v\\d+\\.\\d+\\.\\d+$" },
        "error": {
          "type": "object",
          "required": ["code", "message", "retryable"],
          "properties": {
            "code": {
              "type": "string",
              "enum": [
                "MODEL_UNAVAILABLE",
                "INVALID_INPUT",
                "TIMEOUT",
                "SCHEMA_VIOLATION",
                "RETRYABLE_TRANSIENT",
                "POLICY_BLOCK"
              ]
            },
            "message": { "type": "string", "minLength": 1 },
            "retryable": { "type": "boolean" },
            "details": { "type": "object", "additionalProperties": true }
          },
          "additionalProperties": true
        }
      },
      "additionalProperties": true
    },
    "BaseRequest": {
      "type": "object",
      "required": ["request_id", "correlation_id", "actor", "trigger", "payload_version"],
      "properties": {
        "request_id": { "type": "string", "pattern": "^req_[a-zA-Z0-9_-]{8,}$" },
        "correlation_id": { "type": "string", "minLength": 4 },
        "actor": { "$ref": "#/definitions/Actor" },
        "session_id": { "type": "string" },
        "trigger": { "type": "string", "enum": ["user_action", "timer", "webhook", "async_batch"] },
        "payload_version": { "type": "string" },
        "privacy_level": { "type": "string", "enum": ["open", "l1", "l2", "vault"] }
      },
      "additionalProperties": true
    }
  }
}
```

### EP-00 triage (`/api/v1/edge-pod/ai/00/triage`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "allOf": [
    { "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } },
    {
      "type": "object",
      "required": ["text"],
      "properties": {
        "text": { "type": "string", "minLength": 1 },
        "media_urls": { "type": "array", "items": { "type": "string", "format": "uri" } },
        "location": {
          "type": "object",
          "required": ["lat", "lng"],
          "properties": {
            "lat": { "type": "number", "minimum": -90, "maximum": 90 },
            "lng": { "type": "number", "minimum": -180, "maximum": 180 }
          }
        },
        "voice_transcript": { "type": "string" },
        "attachments_count": { "type": "integer", "minimum": 0, "maximum": 5 },
        "privacy_signal": { "type": "string" },
        "rencana_rutin": { "type": "boolean" }
      }
    }
  ],
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope", "$defs": { "SuccessEnvelope": {} } },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["entry_flow", "track", "seed_type", "context_bar_state"],
            "properties": {
              "entry_flow": { "type": "string", "enum": ["community", "vault", "siaga"] },
              "track": { "type": "string" },
              "seed_type": { "type": "string" },
              "context_bar_state": { "type": "string" },
              "is_split_candidate": { "type": "boolean" },
              "split_candidates": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["seed_id", "reason"],
                  "properties": {
                    "seed_id": { "type": "string" },
                    "reason": { "type": "string" }
                  }
                }
              },
              "follow_up_question": { "type": "string" },
              "reasoning": { "type": "string" }
            }
          }
        }
      }
    ]
  },
  "error": { "$ref": "#/definitions/ErrorEnvelope" }
}
```

### EP-01 classification (`/api/v1/edge-pod/ai/01/classification`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "allOf": [
    { "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } },
    {
      "type": "object",
      "required": ["text", "seed_candidate_id"],
      "properties": {
        "text": { "type": "string", "minLength": 1 },
        "seed_candidate_id": { "type": "string", "minLength": 1 },
        "location": {
          "type": "object",
          "required": ["lat", "lng"],
          "properties": {
            "lat": { "type": "number", "minimum": -90, "maximum": 90 },
            "lng": { "type": "number", "minimum": -180, "maximum": 180 }
          }
        },
        "conversation_context": { "type": "object", "additionalProperties": true },
        "suggested_track": { "type": "string" },
        "seed_type_hint": { "type": "string" }
      }
    }
  ],
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "manual_required": { "type": "boolean" },
          "output": {
            "type": "object",
            "required": ["track", "track_confidence", "seed_type", "seed_confidence", "is_ambiguous"],
            "properties": {
              "track": { "type": "string" },
              "track_confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "seed_type": { "type": "string" },
              "seed_confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "esco_skills": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["uri"],
                  "properties": {
                    "uri": { "type": "string" },
                    "score": { "type": "number", "minimum": 0, "maximum": 1 }
                  }
                }
              },
              "is_ambiguous": { "type": "boolean" },
              "alternative_track": { "type": "string" },
              "alternative_confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "reasoning": { "type": "string" }
            }
          }
        }
      }
    ]
  },
  "error": { "$ref": "#/definitions/ErrorEnvelope" }
}
```

### EP-02 redaction (`/api/v1/edge-pod/ai/02/redaction`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "allOf": [
    { "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } },
    {
      "type": "object",
      "required": ["text", "text_format", "user_id", "context_summary"],
      "properties": {
        "text": { "type": "string" },
        "text_format": { "type": "string", "enum": ["plain", "markdown"] },
        "user_id": { "type": "string", "minLength": 1 },
        "context_summary": { "type": "string" },
        "privacy_level": { "type": "string", "enum": ["open", "l1", "l2", "vault"] },
        "allow_redaction": { "type": "boolean" }
      }
    }
  ],
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["redacted_text", "redaction_count", "needs_manual_review"],
            "properties": {
              "redacted_text": { "type": "string" },
              "redaction_count": { "type": "integer", "minimum": 0 },
              "redacted_items": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["kind", "start", "end", "value"],
                  "properties": {
                    "kind": { "type": "string" },
                    "start": { "type": "integer", "minimum": 0 },
                    "end": { "type": "integer", "minimum": 1 },
                    "value": { "type": "string" }
                  }
                }
              },
              "needs_manual_review": { "type": "boolean" },
              "reason_code": { "type": "string" },
              "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
            }
          }
        }
      }
    ]
  }
}
```

### EP-03 duplicate detection (`/api/v1/edge-pod/ai/03/duplicate-detection`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["seed_text"],
  "properties": {
    "seed_text": { "type": "string" },
    "media_hashes": { "type": "array", "items": { "type": "string" } },
    "embedding": { "type": "array", "items": { "type": "number" }, "minItems": 1 },
    "location": {
      "type": "object",
      "required": ["lat", "lng"],
      "properties": {
        "lat": { "type": "number", "minimum": -90, "maximum": 90 },
        "lng": { "type": "number", "minimum": -180, "maximum": 180 }
      }
    },
    "radius_km": { "type": "number", "minimum": 0 },
    "scope": { "type": "string" },
    "exclude_seed_ids": { "type": "array", "items": { "type": "string" } },
    "query_options": { "type": "object", "additionalProperties": true }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "properties": {
              "matches": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["seed_id", "similarity", "distance_km", "recommendation"],
                  "properties": {
                    "seed_id": { "type": "string" },
                    "similarity": { "type": "number", "minimum": 0, "maximum": 1 },
                    "distance_km": { "type": "number", "minimum": 0 },
                    "recommendation": { "type": "string", "enum": ["allow", "warn", "block"] }
                  }
                }
              },
              "top_match": { "type": "string" },
              "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "auto_block": { "type": "boolean" }
            }
          }
        }
      }
    ]
  }
}
```

### EP-04 moderation (`/api/v1/edge-pod/ai/04/moderation`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["content_id", "text"],
  "properties": {
    "content_id": { "type": "string", "minLength": 1 },
    "text": { "type": "string", "minLength": 1 },
    "attachments": { "type": "array", "items": { "type": "string" } },
    "author_id": { "type": "string" },
    "author_reputation": { "type": "number" },
    "track": { "type": "string" },
    "seed_type": { "type": "string" },
    "location": { "type": "object", "additionalProperties": true },
    "report_volume": { "type": "integer", "minimum": 0 }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["status", "action", "hold_duration_minutes"],
            "properties": {
              "status": { "type": "string", "enum": ["ok", "hold", "block", "escalate"] },
              "violations": { "type": "array", "items": { "type": "string" } },
              "action": { "type": "string", "enum": ["publish", "hold", "remove", "review"] },
              "hold_duration_minutes": { "type": "integer", "minimum": 0 },
              "auto_release_if_no_action": { "type": "boolean" },
              "reason_code": { "type": "string" },
              "reasoning": { "type": "string" },
              "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
            }
          }
        }
      }
    ]
  }
}
```

### EP-05 gaming risk (`/api/v1/edge-pod/ai/05/gaming-risk`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["query_users", "lookback_hours", "platform"],
  "properties": {
    "query_users": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
    "seed_ids": { "type": "array", "items": { "type": "string" } },
    "focus_metric": { "type": "string", "enum": ["posting_rate", "vouch_rate", "revenue", "dispute", "generic"] },
    "lookback_hours": { "type": "integer", "minimum": 1 },
    "platform": { "type": "string" },
    "window_start": { "type": "string", "format": "date-time" },
    "window_end": { "type": "string", "format": "date-time" }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["flags", "summary", "recommendation"],
            "properties": {
              "flags": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["user_id", "metric", "flag", "reason_code", "severity"],
                  "properties": {
                    "user_id": { "type": "string" },
                    "metric": { "type": "string" },
                    "flag": { "type": "boolean" },
                    "reason_code": { "type": "string" },
                    "severity": { "type": "string", "enum": ["low", "medium", "high"] }
                  }
                }
              },
              "summary": {
                "type": "object",
                "required": ["total_flags", "critical_count"],
                "properties": {
                  "total_flags": { "type": "integer", "minimum": 0 },
                  "critical_count": { "type": "integer", "minimum": 0 }
                }
              },
              "recommendation": {
                "type": "string",
                "enum": ["none", "manual_review", "slow_mode", "suspend_actions"]
              }
            }
          }
        }
      }
    ]
  }
}
```

### EP-06 criteria suggestions (`/api/v1/edge-pod/ai/06/criteria-suggestions`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["seed_id", "track", "seed_text"],
  "properties": {
    "seed_id": { "type": "string" },
    "track": { "type": "string" },
    "seed_text": { "type": "string" },
    "discussion_summary": { "type": "string" },
    "community_context": { "type": "object", "additionalProperties": true }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["suggested_criteria", "suggested_tasks", "task_decomposition_confidence"],
            "properties": {
              "suggested_criteria": { "type": "array", "items": { "type": "string" } },
              "suggested_tasks": { "type": "array", "items": { "type": "string" } },
              "task_decomposition_confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "user_agency_note": { "type": "string" }
            }
          }
        }
      }
    ]
  }
}
```

### EP-07 summary (`/api/v1/edge-pod/ai/07/summary`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["input_texts", "messages", "window_start", "window_end"],
  "properties": {
    "input_texts": { "type": "array", "items": { "type": "string" } },
    "messages": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "scope": { "type": "string" },
    "filters": { "type": "object", "additionalProperties": true },
    "window_start": { "type": "string", "format": "date-time" },
    "window_end": { "type": "string", "format": "date-time" },
    "max_items": { "type": "integer", "minimum": 1 }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["summary", "key_points"],
            "properties": {
              "summary": { "type": "string" },
              "key_points": { "type": "array", "items": { "type": "string" } },
              "sentiment": { "type": "string", "enum": ["positive", "neutral", "negative", "mixed"] },
              "action_items": { "type": "array", "items": { "type": "string" } },
              "controversies": { "type": "array", "items": { "type": "string" } },
              "clusters": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "label": { "type": "string" },
                    "count": { "type": "integer", "minimum": 0 }
                  }
                }
              },
              "raw_items": { "type": "boolean" }
            }
          }
        }
      }
    ]
  }
}
```

### EP-08 sensitive-media (`/api/v1/edge-pod/ai/08/sensitive-media`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["media_urls", "media_types", "seed_id", "author_id", "seed_text"],
  "properties": {
    "media_urls": { "type": "array", "items": { "type": "string", "format": "uri" } },
    "media_types": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
    "seed_id": { "type": "string" },
    "author_id": { "type": "string" },
    "seed_text": { "type": "string" },
    "hash_chain": { "type": "array", "items": { "type": "string" } }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["scans", "overall_safety"],
            "properties": {
              "scans": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["media_url", "detections", "severity", "score"],
                  "properties": {
                    "media_url": { "type": "string" },
                    "detections": { "type": "array", "items": { "type": "string" } },
                    "severity": { "type": "string", "enum": ["low", "medium", "high"] },
                    "score": { "type": "number", "minimum": 0, "maximum": 1 }
                  }
                }
              },
              "overall_safety": { "type": "string", "enum": ["safe", "review", "unsafe"] },
              "redacted_media_url": { "type": "string" },
              "summary": { "type": "string" },
              "is_actionable": { "type": "boolean" }
            }
          }
        }
      }
    ]
  }
}
```

### EP-09 credit recommendation (`/api/v1/edge-pod/ai/09/credit-recommendation`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["user_id", "timeline_events", "skill_profile"],
  "properties": {
    "user_id": { "type": "string" },
    "timeline_events": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "contrib_events": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
    "skill_profile": { "type": "array", "items": { "type": "string" } },
    "reputation_snapshot": { "type": "object", "additionalProperties": true }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["candidate_allocations", "dispute_window", "confidence_source"],
            "properties": {
              "candidate_allocations": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["candidate_id", "type", "weight"],
                  "properties": {
                    "candidate_id": { "type": "string" },
                    "type": { "type": "string" },
                    "weight": { "type": "number", "minimum": 0 }
                  }
                }
              },
              "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
              "reasoning": { "type": "string" },
              "dispute_window": { "type": "string", "enum": ["minutes", "hours", "days"] },
              "confidence_source": { "type": "string" }
            }
          }
        }
      }
    ]
  }
}
```

### EP-10 skill extract (`/api/v1/edge-pod/ai/10/skill-extract`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["text", "language", "seed_context", "existing_skills"],
  "properties": {
    "text": { "type": "string" },
    "language": { "type": "string" },
    "seed_context": { "type": "string" },
    "existing_skills": { "type": "array", "items": { "type": "string" } },
    "self_declared": { "type": "array", "items": { "type": "string" } }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["esco_skills"],
            "properties": {
              "esco_skills": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["uri", "score", "validated"],
                  "properties": {
                    "uri": { "type": "string" },
                    "score": { "type": "number", "minimum": 0, "maximum": 1 },
                    "source": { "type": "string" },
                    "validated": { "type": "boolean" }
                  }
                }
              }
            }
          }
        }
      }
    ]
  }
}
```

### EP-11 siaga evaluate (`/api/v1/edge-pod/ai/siaga/evaluate`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "allOf": [{ "$ref": "#/definitions/BaseRequest", "$defs": { "BaseRequest": {} } }],
  "required": ["actor", "text", "location", "current_track", "reported_urgency", "community_scope"],
  "properties": {
    "actor": { "$ref": "#/definitions/Actor" },
    "text": { "type": "string" },
    "location": {
      "type": "object",
      "required": ["lat", "lng"],
      "properties": {
        "lat": { "type": "number", "minimum": -90, "maximum": 90 },
        "lng": { "type": "number", "minimum": -180, "maximum": 180 }
      }
    },
    "confidence_bundle": { "type": "object", "additionalProperties": true },
    "reported_urgency": { "type": "string", "enum": ["low", "normal", "high", "critical"] },
    "community_scope": { "type": "string" },
    "current_track": { "type": "string" }
  },
  "success": {
    "allOf": [
      { "$ref": "#/definitions/SuccessEnvelope" },
      {
        "properties": {
          "output": {
            "type": "object",
            "required": ["is_siaga", "severity", "responder_payload", "scope", "timeline_window", "override_policy"],
            "properties": {
              "is_siaga": { "type": "boolean" },
              "severity": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
              "responder_payload": {
                "type": "object",
                "properties": {
                  "channels": { "type": "array", "items": { "type": "string" } },
                  "template": { "type": "string" },
                  "estimated_reach": { "type": "integer", "minimum": 0 }
                }
              },
              "scope": { "type": "string" },
              "timeline_window": { "type": "string" },
              "override_policy": {
                "type": "string",
                "enum": ["human_only", "auto_if_threshold", "manual_approve_only"]
              },
              "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
            }
          }
        }
      }
    ]
  }
}
```

<!-- End legacy inline schema blocks. -->

## Gotong-Royong consumption and fallback matrix

| Endpoint | Success branch | Model unavailable branch | Invalid schema / low confidence branch |
|---|---|---|---|
| EP-00 | apply `entry_flow`, save draft state, continue | keep session draft state, route to manual flow | show manual triage UI; set `manual_required` true if returned |
| EP-01 | apply track/seed_type and show suggestion UI | set `manual_required=true`, keep user-visible manual override | if `is_ambiguous=true`, force manual confirmation |
| EP-02 | use `redacted_text` before display | use original text and flag manual review | fallback to manual review with redaction skipped |
| EP-03 | block/warn based on `auto_block`/recommendation | continue submit with non-blocking warning | run rule-based check; if parse error, continue + warning |
| EP-04 | apply moderation `status` and visibility changes | set content to hold state and queue manual queue | set hold + request_reason to `policy` |
| EP-05 | apply risk flags as advisory labels | ignore flags and continue process | ignore flags and continue process |
| EP-06 | show criteria/task suggestions | show empty list and require manual input | same as model unavailable |
| EP-07 | render summary and snippets | render raw list, no summary | render raw list and `raw_items=true` |
| EP-08 | hide/mark assets according to scans | keep media visible but route to manual moderation | same as model unavailable |
| EP-09 | pre-fill credit UI (advisory) | open manual credit distribution screen | open manual credit distribution screen |
| EP-10 | apply ESCO proposals with `validated` metadata | keep user-entered skills only | keep user-entered skills only |
| EP-11 | generate Siaga draft and responder payload | create Siaga draft requiring approval queue | create Siaga draft requiring approval queue |

### Rule of record for Gotong-Royong implementation

- Always validate response against endpoint-specific schema before consuming fields.
- Never change state when `error` exists, unless endpoint contract states explicit manual path.
- Do not trust `confidence` alone; use boolean controls (`manual_required`, `hold`, `is_actionable`, `is_siaga`).

## Endpoint readiness tags for gotong-royong review

- `DONE`: contract complete, prompt id registered where required, fallback explicit
- `READY`: EP-00, EP-01, EP-02, EP-03, EP-04, EP-05, EP-06, EP-07, EP-08, EP-09, EP-10, EP-11
- `PENDING`: none

## Notes

- Prompt IDs for EP-03/05/08 are registered (`DUPLICATE-001`, `GAMING-001`, `SENSITIVE-001`).
- Companion credit scoring prompt (`CREDIT-SCORE-001`) is not required for current EP-09 scope.
- EP-06 uses `CRITERIA-001`, but UI-local source-tag behavior in `UI-20` is a scope extension and is not part of the prompt table yet.
- Recommendation: proceed to backend implementation with the approved prompt registry and resolved ownership decisions.
- For every endpoint, response schema must be JSON validated before persisted or used by Gotong-Royong flows.
- For all advisory outputs, state changes in Gotong-Royong must be policy-driven and never solely model-driven.

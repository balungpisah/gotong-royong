# AI Endpoint Map (Canonical v1)

Last updated: 2026-02-26
Owner: AI orchestration + API contracts

This document is the single source of truth for runtime AI endpoint registration.

Purpose:
- lock one canonical registry for every AI/LLM endpoint used by Gotong-Royong,
- pin prompt + contract + fallback + observability in one place,
- prevent drift between design docs, runtime handlers, and schema artifacts.

Related detailed contracts:
- `docs/research/contracts/triage-operator-output-contract-v1.md`
- `docs/research/contracts/triage-operator-output-contract-v1.schema.json`
- `docs/research/contracts/triage-witness-feed-contract-v1.md`
- `docs/research/contracts/edgepod-endpoint-contracts.schema.json`
- `docs/research/contracts/edgepod-endpoint-contracts/EP-*.schema.json`

## 1) Registration Template (Required Fields)

Every registered endpoint must define all fields below:

| Field | Description |
|---|---|
| `endpoint_id` | Stable endpoint ID (e.g. `AI-ENDPOINT-TRIAGE-001`) |
| `route_method` | Runtime HTTP route + method |
| `owner` | Team or slice owner |
| `prompt_id_version` | Prompt ID + version pin |
| `operator_skill` | Operator/skill mapping, or `N/A` |
| `input_contract` | Canonical input schema/doc link |
| `output_contract` | Canonical output schema/doc link |
| `validation_gate` | Code path where schema/shape is enforced |
| `fallback_behavior` | Deterministic fallback when model/output unavailable |
| `observability` | Metrics and reason-code signals |
| `idempotency_ordering` | Idempotency key + ordering requirements |
| `status` | `ACTIVE` or `PARTIAL` or `PLANNED` or `HISTORICAL` |
| `last_verified` | Date of last verification |
| `code_references` | Runtime implementation file references |

## 2) Endpoint Matrix (v1)

| endpoint_id | route_method | prompt_id_version | operator_skill | status | last_verified |
|---|---|---|---|---|---|
| `AI-ENDPOINT-TRIAGE-001` | `POST /v1/triage/sessions` | `TRIAGE-001 @ v0.2.0` | `masalah|musyawarah|pantau|catat|bantuan|rayakan|siaga|program|kelola` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-TRIAGE-002` | `POST /v1/triage/sessions/:session_id/messages` | `TRIAGE-001 @ v0.2.0` | `masalah|musyawarah|pantau|catat|bantuan|rayakan|siaga|program|kelola` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-EP03-001` | `POST /v1/edge-pod/ai/03/duplicate-detection` | `DUPLICATE-001 @ v0.2.0` | `N/A` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-EP05-001` | `POST /v1/edge-pod/ai/05/gaming-risk` | `GAMING-001 @ v0.2.0` | `N/A` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-EP08-001` | `POST /v1/edge-pod/ai/08/sensitive-media` | `SENSITIVE-001 @ v0.2.0` | `N/A` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-EP09-001` | `POST /v1/edge-pod/ai/09/credit-recommendation` | `CREDIT-001 @ v0.2.0` | `N/A` | `ACTIVE` | `2026-02-26` |
| `AI-ENDPOINT-EP11-001` | `POST /v1/edge-pod/ai/siaga/evaluate` | `TRIAGE-001 + MOD-001 (composition)` | `N/A` | `ACTIVE` | `2026-02-26` |

## 3) Detailed Registrations

### 3.1 `AI-ENDPOINT-TRIAGE-001`

- `endpoint_id`: `AI-ENDPOINT-TRIAGE-001`
- `route_method`: `POST /v1/triage/sessions`
- `owner`: frontend + API cutover slice
- `prompt_id_version`: `TRIAGE-001 @ v0.2.0`
- `operator_skill`: `masalah|musyawarah|pantau|catat|bantuan|rayakan|siaga|program|kelola`
- `input_contract`:
  - `docs/research/contracts/triage-witness-feed-contract-v1.md` (optional `operator_output` handoff)
  - `docs/research/contracts/triage-operator-output-contract-v1.md`
- `output_contract`:
  - `docs/research/contracts/triage-operator-output-contract-v1.schema.json` (`operator.v1`)
  - `docs/research/contracts/triage-witness-feed-contract-v1.md` (`triage.v1` mapping result)
- `validation_gate`:
  - parse + validate `operator.v1` in `crates/api/src/routes/mod.rs` (`triage_result_from_operator_output`, `triage_validate_operator_output`)
- `fallback_behavior`:
  - if `operator_output` omitted, use internal fallback operator synthesis
  - if `operator_output` invalid, reject with standard `500 internal_error` envelope
- `observability`:
  - generic HTTP metrics via `gotong_api_http_requests_total`, `gotong_api_http_errors_total`
  - validation failures logged in triage route validator path
- `idempotency_ordering`:
  - idempotency key namespace `triage_session_start`
  - stateful ordering is session turn order in `triage_sessions` map
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/mod.rs`
  - `crates/api/src/tests.rs`

### 3.2 `AI-ENDPOINT-TRIAGE-002`

- `endpoint_id`: `AI-ENDPOINT-TRIAGE-002`
- `route_method`: `POST /v1/triage/sessions/:session_id/messages`
- `owner`: frontend + API cutover slice
- `prompt_id_version`: `TRIAGE-001 @ v0.2.0`
- `operator_skill`: `masalah|musyawarah|pantau|catat|bantuan|rayakan|siaga|program|kelola`
- `input_contract`:
  - `docs/research/contracts/triage-witness-feed-contract-v1.md`
  - `docs/research/contracts/triage-operator-output-contract-v1.md`
- `output_contract`:
  - `docs/research/contracts/triage-operator-output-contract-v1.schema.json`
  - `docs/research/contracts/triage-witness-feed-contract-v1.md`
- `validation_gate`:
  - parse + validate `operator.v1` in `crates/api/src/routes/mod.rs`
- `fallback_behavior`:
  - if `operator_output` omitted, use internal fallback operator synthesis
  - if `operator_output` invalid, reject with standard `500 internal_error` envelope
- `observability`:
  - generic HTTP metrics via `gotong_api_http_requests_total`, `gotong_api_http_errors_total`
  - validation failures logged in triage route validator path
- `idempotency_ordering`:
  - idempotency key namespace `triage_session_continue`
  - strict per-session turn progression in runtime session state
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/mod.rs`
  - `crates/api/src/tests.rs`

### 3.3 `AI-ENDPOINT-EP03-001`

- `endpoint_id`: `AI-ENDPOINT-EP03-001`
- `route_method`: `POST /v1/edge-pod/ai/03/duplicate-detection`
- `owner`: Edge-Pod integration slice
- `prompt_id_version`: `DUPLICATE-001 @ v0.2.0`
- `operator_skill`: `N/A`
- `input_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-03.schema.json`
- `output_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-03.schema.json`
- `validation_gate`: request validation + role checks in `crates/api/src/routes/edgepod.rs` (`edgepod_duplicate_detection`)
- `fallback_behavior`: deterministic fallback output with `reason_code`; no state mutation blocked by model unavailability
- `observability`:
  - `gotong_api_edgepod_fallback_total`
  - `gotong_api_edgepod_model_unavailable_total`
- `idempotency_ordering`: key namespace `edgepod_ep03`; idempotent replay on same request key
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/edgepod.rs`
  - `docs/research/contracts/edgepod-endpoint-contracts.contract-map.md`

### 3.4 `AI-ENDPOINT-EP05-001`

- `endpoint_id`: `AI-ENDPOINT-EP05-001`
- `route_method`: `POST /v1/edge-pod/ai/05/gaming-risk`
- `owner`: Edge-Pod integration slice
- `prompt_id_version`: `GAMING-001 @ v0.2.0`
- `operator_skill`: `N/A`
- `input_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-05.schema.json`
- `output_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-05.schema.json`
- `validation_gate`: request validation + role checks in `crates/api/src/routes/edgepod.rs` (`edgepod_gaming_risk`)
- `fallback_behavior`: deterministic fallback output with `reason_code`
- `observability`:
  - `gotong_api_edgepod_fallback_total`
  - `gotong_api_edgepod_model_unavailable_total`
- `idempotency_ordering`: key namespace `edgepod_ep05`; idempotent replay on same request key
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/edgepod.rs`

### 3.5 `AI-ENDPOINT-EP08-001`

- `endpoint_id`: `AI-ENDPOINT-EP08-001`
- `route_method`: `POST /v1/edge-pod/ai/08/sensitive-media`
- `owner`: Edge-Pod integration slice
- `prompt_id_version`: `SENSITIVE-001 @ v0.2.0`
- `operator_skill`: `N/A`
- `input_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-08.schema.json`
- `output_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-08.schema.json`
- `validation_gate`: request validation + role checks in `crates/api/src/routes/edgepod.rs` (`edgepod_sensitive_media`)
- `fallback_behavior`: deterministic fallback output with `reason_code`
- `observability`:
  - `gotong_api_edgepod_fallback_total`
  - `gotong_api_edgepod_model_unavailable_total`
- `idempotency_ordering`: key namespace `edgepod_ep08`; idempotent replay on same request key
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/edgepod.rs`

### 3.6 `AI-ENDPOINT-EP09-001`

- `endpoint_id`: `AI-ENDPOINT-EP09-001`
- `route_method`: `POST /v1/edge-pod/ai/09/credit-recommendation`
- `owner`: Edge-Pod integration slice
- `prompt_id_version`: `CREDIT-001 @ v0.2.0`
- `operator_skill`: `N/A`
- `input_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-09.schema.json`
- `output_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-09.schema.json`
- `validation_gate`: request validation + role checks in `crates/api/src/routes/edgepod.rs` (`edgepod_credit_recommendation`)
- `fallback_behavior`: deterministic fallback output with `reason_code`
- `observability`:
  - `gotong_api_edgepod_fallback_total`
  - `gotong_api_edgepod_model_unavailable_total`
  - `gotong_api_markov_integration_errors_total` for markov enrichment failures
- `idempotency_ordering`: key namespace `edgepod_ep09`; idempotent replay on same request key
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/edgepod.rs`

### 3.7 `AI-ENDPOINT-EP11-001`

- `endpoint_id`: `AI-ENDPOINT-EP11-001`
- `route_method`: `POST /v1/edge-pod/ai/siaga/evaluate`
- `owner`: Edge-Pod integration slice
- `prompt_id_version`: `TRIAGE-001 + MOD-001 (composition path)`
- `operator_skill`: `N/A`
- `input_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-11.schema.json`
- `output_contract`: `docs/research/contracts/edgepod-endpoint-contracts/EP-11.schema.json`
- `validation_gate`: request validation + role checks in `crates/api/src/routes/edgepod.rs` (`edgepod_siaga_evaluate`)
- `fallback_behavior`: deterministic fallback output with `reason_code`; manual approval path preserved for uncertain output
- `observability`:
  - `gotong_api_edgepod_fallback_total`
  - `gotong_api_edgepod_model_unavailable_total`
- `idempotency_ordering`: key namespace `edgepod_ep11`; idempotent replay on same request key
- `status`: `ACTIVE`
- `last_verified`: `2026-02-26`
- `code_references`:
  - `crates/api/src/routes/edgepod.rs`

## 4) Change Management Rules

1. Any new AI endpoint or prompt/schema change must update this file in the same PR.
2. No AI endpoint is considered implementation-ready unless it has a complete registration entry here.
3. Update `last_verified` when runtime code path or schema link changes.
4. If endpoint behavior changes from advisory to system-of-record or the reverse, update `fallback_behavior` and `validation_gate` explicitly.
5. If endpoint is retired, set `status=HISTORICAL`, keep entry, and point to replacement endpoint ID.

## 5) Verification Checklist

- all registered routes exist in `crates/api/src/routes/mod.rs`.
- all linked schema/docs files resolve in repository.
- prompt IDs and versions align with endpoint schema headers or contract docs.
- fallback and observability descriptions match runtime behavior.

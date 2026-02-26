# Research Directory

This directory contains research artifacts, decision records, planning documents, and API/AI contracts produced during pre-implementation phases. Files are organized into five subdirectories by content type.

The canonical stack decision (ADR-001) has been promoted to `docs/architecture/adr/` — the copy in `adr/` is retained as the source of record.

---

## `decisions/` — Decision Records and Resolution Notes

| File | Status | Description |
|------|--------|-------------|
| `research-summary-and-decision-record.md` | [DECISION RECORD] | Gate-passed summary of all research phases; final approved decision record |
| `decision-notes.md` | [DECISION RECORD] | Final approved answers to all blockers and open questions |
| `decision-questions-pack.md` | [DECISION RECORD] | Full pack of unresolved blockers and the questions posed to resolve them |
| `decision-notes-template.md` | [TEMPLATE] | Template for capturing decision answers during blocker resolution |

## `references/` — Technical Reference Material

| File | Status | Description |
|------|--------|-------------|
| `surrealdb-pattern-sampling-v3-beta4.md` | [ACTIVE REF] | SurrealDB v3 beta pattern sampling — query patterns, record IDs, graph traversal |
| `surrealdb-go-no-go-latest.md` | [ACTIVE REF] | Go/no-go analysis for SurrealDB v3 beta adoption; underpins ADR-001 |
| `surrealql-pattern-cheatsheet.md` | [ACTIVE REF] | SurrealQL query patterns cheatsheet for the locked `=3.0.0-beta.4` version |
| `ontology-probe-report.md` | [ACTIVE REF] | Results of ontology probe against SurrealDB graph schema; informs schema design |
| `rust-backend-stack-research.md` | [ACTIVE REF] | Rust crate evaluation: Axum vs Actix, SurrealDB SDK, async patterns |
| `backend-research-notes.md` | [ACTIVE REF] | General Rust 2024 backend research notes: crates, patterns, implementation prep |
| `FRONTEND-TECHNOLOGY-RESEARCH-REPORT.md` | [ACTIVE REF] | Frontend stack evaluation: SvelteKit 2, Svelte 5 runes, Bun, Paraglide |
| `samples/` | [ACTIVE REF] | SurrealDB binary sample (`surreal-v3.0.0-beta.4`) and probe shell scripts |

## `contracts/` — API and AI Contracts

| File | Status | Description |
|------|--------|-------------|
| `triage-witness-feed-contract-v1.md` | [ACTIVE REF] | Canonical runtime contract for triage sessions and session-authoritative witness materialization |
| `feed-card-blocks-v1.md` | [ACTIVE REF] | Canonical block inventory and trajectory profiles for feed card rendering (`witness/system/data`) |
| `chat-interaction-blocks-v1.md` | [ACTIVE REF] | Canonical block catalog for conversation + structured layers in chat-driven phases |
| `feed-seed-metadata-v1.md` | [ACTIVE REF] | Canonical optional metadata contract for seeded feed cards (`dev_meta`) used in dev/test |
| `ai-endpoint-map-v1.md` | [ACTIVE REF] | Canonical registry of runtime AI endpoints: prompt/version, contract links, fallback, validation gate, observability |
| `triage-operator-output-contract-v1.md` | [ACTIVE REF] | Canonical operator->orchestrator contract (`triage_draft/triage_final`) with payload semantics per operator |
| `triage-operator-output-contract-v1.schema.json` | [ACTIVE REF] | Strict JSON Schema for validating operator outputs (`schema_version=operator.v1`) |
| `trajectory-tandang-signal-crosswalk-v1.md` | [ACTIVE REF] | Cross-reference matrix between trajectory taxonomy and Tandang signal patterns, including gap-detection contract |
| `feature-contract-map.md` | [ACTIVE REF] | Maps each UI feature to its backend API contract, ownership, and trigger |
| `edgepod-endpoint-contracts.md` | [HISTORICAL] | Legacy research matrix; superseded for runtime by `triage-witness-feed-contract-v1.md` |
| `edgepod-endpoint-contracts.schema.json` | [ACTIVE REF] | Strict JSON contract bundle for all Edge-Pod endpoints |
| `edgepod-endpoint-contracts.contract-map.md` | [ACTIVE REF] | Compact endpoint summary for quick handoff |
| `edgepod-endpoint-contracts/EP-*.schema.json` | [ACTIVE REF] | Per-endpoint JSON contract files (EP-00 through EP-11) |
| `ai-contract-log.md` | [ACTIVE REF] | AI touch point contracts: input/output schema, prompt IDs, fallback behavior |

## `logs/` — Ongoing Logs and Gap Tracking

| File | Status | Description |
|------|--------|-------------|
| `tandang-gap-log.md` | [ACTIVE REF] | Tandang signal gap analysis: KNOWN / MISSING / CONFLICT per feature |
| `tandang-integration-notes.md` | [ACTIVE REF] | Running notes on Tandang ↔ Gotong integration specifics |
| `prompt-id-registration-log.md` | [ACTIVE REF] | AI/Edge-Pod prompt and version ownership; pending prompt IDs |
| `blocker-resolution-handoff-brief.md` | [ACTIVE REF] | Stakeholder-ready blockers brief with required outputs per blocker |

## `planning/` — Implementation Planning

| File | Status | Description |
|------|--------|-------------|
| `backend-implementation-tickets.md` | [ACTIVE REF] | Language-agnostic backend ticket pack with dependency order |
| `architecture-gate-checklist.md` | [ACTIVE REF] | Architecture readiness gates before starting implementation |
| `ui-feature-inventory.md` | [ACTIVE REF] | One row per UI feature requiring backend behavior; Phase 0 output |
| `implementation-plan-pr-chunks.md` | [ACTIVE REF] | PR-sized implementation chunks for incremental delivery |
| `prd-adaptive-path-guidance.md` | [ACTIVE REF] | PRD for Adaptive Path model; input to `ADAPTIVE-PATH-SPEC-v0.1.md` |
| `frontend-foundation-implementation-plan.md` | [ACTIVE REF] | Frontend foundation setup plan; SvelteKit 2 + Svelte 5 runes |

## `adr/` — Architecture Decision Records

| File | Status | Description |
|------|--------|-------------|
| `ADR-001-rust-axum-surrealdb-stack-lock.md` | [DECISION RECORD] | Locks the implementation stack: Rust 2024 / Axum 0.7 / SurrealDB `=3.0.0-beta.4`. **Canonical copy at `docs/architecture/adr/`**. |

---

## Legend

| Tag | Meaning |
|-----|---------|
| `[ACTIVE REF]` | Currently referenced in implementation work; keep up to date |
| `[DECISION RECORD]` | Frozen record of a past decision; do not modify content |
| `[HISTORICAL]` | Superseded; preserved for traceability only |
| `[TEMPLATE]` | Reusable template; not a decision record |

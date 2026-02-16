# Refactor Plan — Align Codebase to Session 1–4 Decisions (Refined)

**Created:** 2026-02-16  
**Refined:** 2026-02-16  
**Context:** Rust backend (domain/api/infra/worker), SurrealDB schema migrations, adaptive-path + ontology redesign, no backward-compatibility requirement (dev phase).  
**Companion backlog:** `docs/design/context/REFACTOR-TICKETS.md`

---

## Current State vs Target State (Validated)

| Aspect | Current Code | Target (Sessions 1–4) |
|---|---|---|
| Lifecycle | Fixed stage transitions (`TrackStateTransition`) still active | Adaptive Path Guidance only (no fixed stage machine) |
| Adaptive path classification | `track_hint` + `seed_hint` free-text | Typed `ActionType` aligned to Schema.org actions |
| Modes | Implicit split across features | 4 explicit modes (Komunitas, CatatanKomunitas, CatatanSaksi, Siaga) |
| Discovery | Feed carries `track`/`stage`; API filters on both | Concept/mode-aware discovery, no stage lifecycle coupling |
| Ranking | `VouchWeightHint` enum (Strong/Moderate/Light) | Wilson score over vouch/challenge counts |
| Ontology schema | `0013_ontology_schema.surql` exists | Domain/infra/api wiring fully implemented |
| Migrations/checks tooling | Files exist through `0013`, but `scripts/db/{migrate,check}.sh` run only to `0012` | Scripts and checks fully aligned with actual migration set |
| Jobs | `TransitionClose`, `ModerationAutoRelease`, `WebhookRetry`, `DigestSend` | Remove `TransitionClose`, add `TTLCleanup` + `ConceptVerification` |
| Queue wiring | Shared queue field named `transition_job_queue`, used by transition + moderation + webhook | Keep shared queue, rename to neutral name (`job_queue`) |

---

## Critical Corrections Applied To This Plan

1. **Do not remove the shared job queue in transition removal PR.** It is also used by webhook retry and moderation auto-release flows.  
2. **Do not change persisted fields without same-PR schema/repository updates.** New required fields (`mode`, `action_type`) must include migration + row mappings + API payload updates.  
3. **Do not drop/rename schema entities without updating migration/check runners.** `scripts/db/migrate.sh` and `scripts/db/check.sh` must stay synchronized with files under `database/migrations` and `database/checks`.  
4. **Do not claim periodic jobs without a producer path.** Worker currently consumes queued jobs; schedule producer logic must be added explicitly.

---

## Execution Rules

- Keep each PR compile-safe (`cargo check`, `cargo test`) and migration-safe (`scripts/db/migrate.sh`, `scripts/db/check.sh`).
- Include schema/check/script updates in the same PR as any schema behavior change.
- Prefer forward migrations over rewriting old migration files; keep history auditable.
- Remove transition-specific concepts, but keep shared infrastructure that other subsystems depend on.

---

## PR Sequence

### PR 0: Tooling + Queue Baseline Hardening (No Behavior Change)

**What:** Stabilize shared plumbing before model refactor.

**Files changed:**
- UPDATE `scripts/db/migrate.sh` to include `0013_ontology_schema.surql`
- UPDATE `scripts/db/check.sh` to include `0013_ontology_schema_check.surql`
- RENAME `transition_job_queue` → `job_queue` in `api/state.rs` and call sites
- KEEP queue semantics unchanged (still used by webhook + moderation)

**Finish criteria:**
- DB scripts run through 0013 successfully
- `cargo check` + `cargo test` pass
- No behavior change in transition/moderation/webhook flows

---

### PR 1: Remove Fixed Track Lifecycle Safely

**What:** Remove transition domain/API/worker behavior without breaking unrelated queues/jobs.

**Files changed:**
- DELETE `domain/transitions.rs` and `domain/ports/transitions.rs`
- REMOVE transition repositories from `infra/repositories/mod.rs`
- REMOVE transition endpoints from `api/routes/mod.rs` (`/v1/transitions/*`)
- REMOVE `TransitionClosePayload` from `domain/jobs.rs`
- REMOVE `TransitionClose` from `JobType` in `domain/ports/jobs.rs`
- REMOVE transition handling from `worker/main.rs`
- REMOVE transition repository wiring from `api/state.rs`
- REMOVE transition feed ingestion helper in API routes
- ADD migration `0014_remove_track_state_transition.surql` (drop table/indexes)
- UPDATE checks:
  - remove transition references from `database/checks/0001_initial_schema_check.surql`
  - remove/replace `database/checks/0004_transition_prereq_check.surql`
  - register new/updated checks in `scripts/db/check.sh`

**Finish criteria:**
- Zero references to `TrackStateTransition`, `TrackTransitionRepository`, `TransitionClose`
- Worker still processes remaining job types (`ModerationAutoRelease`, `WebhookRetry`, `DigestSend`)
- `scripts/db/migrate.sh` + `scripts/db/check.sh` pass
- `cargo check` + `cargo test` pass

---

### PR 2: Remove Feed `track`/`stage` Contamination Completely

**What:** Remove lifecycle fields from discovery models, queries, repositories, API params, and tests.

**Files changed:**
- REMOVE `track`/`stage` from:
  - `domain/discovery.rs` (`FeedItem`, ingest/query structs)
  - `domain/ports/discovery.rs` (`FeedRepositoryQuery` and `FeedRepositorySearchQuery`)
  - `api/routes/mod.rs` feed/search query params
  - `infra/repositories/mod.rs` in-memory + Surreal discovery implementations and row structs
- ADD migration `0015_discovery_drop_track_stage.surql`
- ADD/UPDATE paired checks and runner entries
- UPDATE `api/src/tests.rs` and discovery/domain tests accordingly

**Finish criteria:**
- `rg "\btrack\b|\bstage\b" crates/domain/src/discovery.rs crates/domain/src/ports/discovery.rs` returns zero hits
- Feed and search endpoints work without `track`/`stage` query params
- DB scripts and Rust tests pass

---

### PR 3: Introduce Explicit Mode System (With Persistence)

**What:** Add `Mode` as first-class type and persist it where required.

**Files changed:**
- CREATE `domain/mode.rs` and export in `domain/lib.rs`
- ADD `mode` to contribution domain model and create input
- ADD `mode` to adaptive-path payload draft/payload as needed by decisions
- UPDATE API request validation to require/validate `mode` where required
- UPDATE Surreal and in-memory repositories for new mode fields
- ADD migration(s) for mode fields (e.g. contribution/path/feed tables as required)
- UPDATE checks and scripts for new migration(s)

**Finish criteria:**
- Contribution create flow requires valid `mode` end-to-end
- Persistence round-trip includes mode values
- `cargo check` + `cargo test` + DB scripts pass

---

### PR 4: Ontology Domain Types + Adaptive Path Classification Swap

**What:** Introduce ontology domain types and replace adaptive-path free-text hints with typed action classification.

**Files changed:**
- CREATE `domain/ontology.rs` (`ActionType`, `Concept`, `Triple`, `Note`, `Measurement`, etc.)
- CREATE `domain/ranking.rs` (`wilson_score`)
- REPLACE adaptive-path `track_hint`/`seed_hint` with typed fields (`action_type`, plus mode mapping as needed)
- UPDATE API payload structs + conversion helpers for adaptive-path create/update/suggest
- UPDATE persistence layer + migration(s) for path plan schema changes
- UPDATE tests that currently assert `track_hint`/`seed_hint`

**Finish criteria:**
- No `track_hint`/`seed_hint` usage in active adaptive-path code paths
- `ActionType` mapping logic is covered by tests
- Wilson score unit tests exist with known expected values
- Rust + DB verification pass

---

### PR 5: Ontology Repository Port + Infra Implementations

**What:** Implement ontology data access with Surreal patterns validated in probe docs.

**Files changed:**
- CREATE `domain/ports/ontology.rs`
- CREATE `infra/repositories/ontology.rs` (`SurrealOntologyRepository`, `InMemoryOntologyRepository`)
- Wire ontology repository constructors in infra exports/init paths
- Add repository-level tests (create note/triples, hierarchy traversal, vouch/challenge counts, ranking input)

**Finish criteria:**
- Full trait implementation present for in-memory + Surreal
- Queries follow validated patterns from `docs/research/surrealql-pattern-cheatsheet.md`
- Build/tests pass

---

### PR 6: Ontology API Surface + AppState Wiring

**What:** Expose ontology capabilities over HTTP and wire state initialization.

**Files changed:**
- CREATE `api/routes/ontology.rs`
- MOUNT ontology routes in `api/routes/mod.rs`
- ADD `ontology_repo` to `api/state.rs` (surreal + in-memory init paths)
- ADD API validation for privacy/TTL/temporal/action fields
- ADD API integration tests using in-memory ontology repository

**Finish criteria:**
- Ontology endpoints return expected data for create/feed/concept/hierarchy/ranked/vouch/challenge
- App boots in in-memory and surreal modes with ontology routes enabled
- Tests pass

---

### PR 7: Worker Jobs for TTL Cleanup + Concept Verification (With Producer)

**What:** Add new job types and explicit periodic enqueue strategy.

**Files changed:**
- ADD `TTLCleanup` and `ConceptVerification` to `JobType`
- ADD payload structs in `domain/jobs.rs` as needed
- ADD handlers in `worker/main.rs` invoking ontology repository methods
- ADD periodic enqueue mechanism (scheduler/producer loop) with config-driven intervals
- ADD idempotency/dedup for periodic enqueue keys to avoid duplicate job storms
- REMOVE any remaining transition-close assumptions from worker metrics/labels

**Finish criteria:**
- Worker handles new job types successfully
- Jobs are actually enqueued on schedule (hourly/daily) by producer logic
- Existing jobs (`WebhookRetry`, `ModerationAutoRelease`, `DigestSend`) remain intact
- Tests pass

---

### PR 8: Split `infra/repositories/mod.rs` Into Modules

**What:** Structural refactor after functional changes are stable.

**Files changed:**
- Split monolith into focused modules (`contribution.rs`, `evidence.rs`, `vouch.rs`, `adaptive_path.rs`, `vault.rs`, `discovery.rs`, `siaga.rs`, `moderation.rs`, `chat.rs`, `webhook.rs`, `ontology.rs`)
- Keep `infra/repositories/mod.rs` as exports only

**Finish criteria:**
- No behavior changes
- Build/tests pass
- `mod.rs` remains thin and readable

---

## Dependency Graph (Corrected)

```
PR 0 (tooling + queue baseline)
  └─┬─ PR 1 (remove transitions)
    └─┬─ PR 2 (remove feed track/stage)
      └─┬─ PR 3 (add mode + persistence)
        └─┬─ PR 4 (ontology domain + adaptive-path action types)
          └─┬─ PR 5 (ontology repository)
            └─┬─ PR 6 (ontology API)
              └── PR 7 (new worker jobs + periodic producer)

PR 8 (repository file split) can run after PR 5, preferably after PR 7 for minimal merge churn.
```

---

## Out of Scope (Unchanged)

| Item | Why Deferred |
|---|---|
| AI-00 triage implementation details | Needs full AI touchpoint design closure |
| Rahasia deep policy (L2 leakage, governance) | Requires dedicated security/policy session |
| Onboarding and offline UX | Not part of backend core refactor |
| Reputation redesign | Parked explicitly (Session 3 decisions) |
| Frontend implementation | Current scope is backend/domain/infra/worker |

---

*This refined plan keeps the original direction, but makes the execution path compile-safe, migration-safe, and operationally complete.*

# Refactor Tickets — Session 1–4 Alignment

**Created:** 2026-02-16  
**Source plan:** `docs/design/context/REFACTOR-PLAN.md`  
**Purpose:** Ticket-sized execution backlog for multi-session implementation.

---

## How To Use This File Across Sessions

- Update `Status` for each ticket (`todo`, `in_progress`, `blocked`, `done`).
- Do not change ticket IDs.
- Keep dependencies explicit; do not start dependent tickets early.
- For each completed ticket, append branch/PR reference in `Notes`.

---

## Global Validation Gate

Run from `/Users/damarpanuluh/MERIDIAN-NEW/gotong-royong`:

```bash
cargo check
cargo test -q
bash scripts/db/migrate.sh
bash scripts/db/check.sh
```

Use DB script commands only for tickets that touch migrations/checks/scripts.

---

## Ticket Board

| ID | Title | Depends On | Status | Notes |
|---|---|---|---|---|
| GR-001 | Migration/check runner sync to 0013 ontology | - | todo | |
| GR-002 | Rename shared API queue field to neutral name (`job_queue`) | - | todo | |
| GR-003 | Remove transition feature from domain/API/infra/worker | GR-002 | todo | |
| GR-004 | Transition schema/check cleanup migration | GR-003 | todo | |
| GR-005 | Remove discovery `track`/`stage` in code paths | GR-003 | todo | |
| GR-006 | Discovery schema migration for `track`/`stage` removal | GR-005 | todo | |
| GR-007 | Introduce `Mode` with contribution persistence | GR-006 | todo | |
| GR-008 | Adaptive-path classification swap to `ActionType` | GR-007 | todo | |
| GR-009 | Add ranking primitive (`wilson_score`) + tests | GR-008 | todo | |
| GR-010 | Ontology repository port + in-memory + Surreal impl | GR-008, GR-009 | todo | |
| GR-011 | Ontology API routes + AppState wiring | GR-010 | todo | |
| GR-012 | Worker jobs (`TTLCleanup`, `ConceptVerification`) + periodic producer | GR-010, GR-011 | todo | |
| GR-013 | Split `infra/repositories/mod.rs` monolith | GR-012 | todo | |

---

## Ticket Details

### GR-001 — Migration/check runner sync to 0013 ontology

- Scope: align runner scripts with existing `0013` files.
- Primary files:
  - `scripts/db/migrate.sh`
  - `scripts/db/check.sh`
- Acceptance:
  - runner includes `0013_ontology_schema.surql`
  - check runner includes `0013_ontology_schema_check.surql`
  - scripts run successfully

### GR-002 — Rename shared queue field to `job_queue` (no behavior change)

- Scope: rename only; keep queue semantics for webhook/moderation.
- Primary files:
  - `crates/api/src/state.rs`
  - `crates/api/src/routes/mod.rs`
- Acceptance:
  - no `transition_job_queue` identifier remains
  - webhook enqueue path still works
  - moderation auto-release enqueue path still works

### GR-003 — Remove transition feature from domain/API/infra/worker

- Scope: remove transition model/ports/repos/routes/jobs logic.
- Primary files:
  - `crates/domain/src/transitions.rs`
  - `crates/domain/src/ports/transitions.rs`
  - `crates/domain/src/jobs.rs`
  - `crates/domain/src/ports/jobs.rs`
  - `crates/api/src/routes/mod.rs`
  - `crates/api/src/state.rs`
  - `crates/worker/src/main.rs`
  - `crates/infra/src/repositories/mod.rs`
- Acceptance:
  - no references to `TrackStateTransition` or `TrackTransitionRepository`
  - `TransitionClose` removed from `JobType`
  - worker still processes non-transition jobs

### GR-004 — Transition schema/check cleanup migration

- Scope: drop transition table via forward migration and align checks.
- Primary files:
  - `database/migrations/0014_remove_track_state_transition.surql` (new)
  - `database/checks/0001_initial_schema_check.surql`
  - `database/checks/0004_transition_prereq_check.surql` (remove/replace)
  - `scripts/db/migrate.sh`
  - `scripts/db/check.sh`
- Acceptance:
  - no transition table check references remain
  - migration/check scripts still run end-to-end

### GR-005 — Remove discovery `track`/`stage` in code paths

- Scope: remove fields from domain/query/API/repository code and tests.
- Primary files:
  - `crates/domain/src/discovery.rs`
  - `crates/domain/src/ports/discovery.rs`
  - `crates/api/src/routes/mod.rs`
  - `crates/infra/src/repositories/mod.rs`
  - `crates/api/src/tests.rs`
- Acceptance:
  - feed/search no longer accept or persist `track`/`stage`
  - tests updated for new payload/query shapes

### GR-006 — Discovery schema migration for `track`/`stage` removal

- Scope: remove DB columns and align checks/runners.
- Primary files:
  - `database/migrations/0015_discovery_drop_track_stage.surql` (new)
  - `database/checks/0008_discovery_check.surql`
  - `scripts/db/migrate.sh`
  - `scripts/db/check.sh`
- Acceptance:
  - DB schema and repository row mapping match
  - DB scripts pass

### GR-007 — Introduce `Mode` with contribution persistence

- Scope: add first-class mode type and persist in contribution flow.
- Primary files:
  - `crates/domain/src/mode.rs` (new)
  - `crates/domain/src/lib.rs`
  - `crates/domain/src/contributions.rs`
  - `crates/api/src/routes/mod.rs`
  - `crates/infra/src/repositories/mod.rs`
  - `database/migrations/0016_add_mode_fields.surql` (new)
  - related checks/scripts
- Acceptance:
  - contribution create requires valid mode
  - mode persists and round-trips via API/repo

### GR-008 — Adaptive-path classification swap to `ActionType`

- Scope: replace `track_hint`/`seed_hint` with typed action field(s).
- Primary files:
  - `crates/domain/src/ontology.rs` (new)
  - `crates/domain/src/adaptive_path.rs`
  - `crates/api/src/routes/mod.rs`
  - `crates/infra/src/repositories/mod.rs`
  - `database/migrations/0017_path_plan_action_type.surql` (new)
  - related checks/scripts/tests
- Acceptance:
  - adaptive-path code no longer depends on `track_hint`/`seed_hint`
  - typed classification validated in API/domain

### GR-009 — Add ranking primitive (`wilson_score`) + tests

- Scope: add deterministic score function and unit tests.
- Primary files:
  - `crates/domain/src/ranking.rs` (new)
  - `crates/domain/src/lib.rs`
  - domain tests
- Acceptance:
  - known input/output tests pass for Wilson score
  - function reusable by ontology ranking endpoints

### GR-010 — Ontology repository port + implementations

- Scope: add ontology port and both in-memory/Surreal implementations.
- Primary files:
  - `crates/domain/src/ports/ontology.rs` (new)
  - `crates/domain/src/ports/mod.rs`
  - `crates/infra/src/repositories/ontology.rs` (new or split target)
  - `crates/infra/src/repositories/mod.rs`
- Acceptance:
  - trait methods implemented for both backends
  - repository tests cover note/triple write, concept query, hierarchy, ranking inputs

### GR-011 — Ontology API routes + state wiring

- Scope: expose ontology endpoints and wire repo in app state boot paths.
- Primary files:
  - `crates/api/src/routes/ontology.rs` (new)
  - `crates/api/src/routes/mod.rs`
  - `crates/api/src/state.rs`
  - `crates/api/src/tests.rs`
- Acceptance:
  - endpoints mounted and validated
  - in-memory and surreal init paths support ontology repo
  - integration tests pass

### GR-012 — Worker jobs + periodic producer for ontology maintenance

- Scope: add new job types and producer path for hourly/daily enqueue.
- Primary files:
  - `crates/domain/src/ports/jobs.rs`
  - `crates/domain/src/jobs.rs`
  - `crates/worker/src/main.rs`
  - `crates/infra/src/config.rs`
- Acceptance:
  - `TTLCleanup` and `ConceptVerification` handled by worker
  - producer path enqueues jobs on configured cadence
  - non-ontology jobs continue to work

### GR-013 — Split `infra/repositories/mod.rs` monolith

- Scope: pure structural split after behavioral work is stable.
- Primary files:
  - `crates/infra/src/repositories/mod.rs`
  - new module files under `crates/infra/src/repositories/`
- Acceptance:
  - no logic regressions
  - exports clean and compile/test pass

---

## Suggested Parallelization

- `GR-001` and `GR-002` can run in parallel.
- `GR-009` can run in parallel with early `GR-010` work once `GR-008` lands.
- Keep `GR-013` last to reduce merge conflicts.


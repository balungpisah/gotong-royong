# Gotong Royong — Adaptive Path Map

> Visual reference for the adaptive path model. Replaces fixed track flowcharts (archived in `design/archive/TRACK-MAP.md`).

---

## Generic Adaptive Timeline

```
┌─────────────────────────────────────────────────────────────────┐
│  ADAPTIVE PATH PLAN                                             │
│  plan_id: plan_xx · version: 3 · track_hint: tuntaskan         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ══ BRANCH: Utama (main) ══════════════════════════════════     │
│                                                                 │
│  ┌─────────────────────┐                                        │
│  │ PHASE 1: Stabilisasi│ status: completed · source: ai        │
│  │ objective: "..."     │                                       │
│  ├─────────────────────┤                                        │
│  │ ☑ checkpoint c1     │ completed · source: ai                │
│  │ ☑ checkpoint c2     │ completed · source: human (🔒 title)  │
│  └────────┬────────────┘                                        │
│           │                                                     │
│           │         ┌── BRANCH: "Jika air naik" ──┐            │
│           │         │                              │            │
│           │         │  ┌────────────────────────┐  │            │
│           ├─────────│  │ PHASE: Evakuasi        │  │            │
│           │         │  │ status: planned         │  │            │
│           │         │  ├────────────────────────┤  │            │
│           │         │  │ ☐ Koordinasi BPBD      │  │            │
│           │         │  │ ☐ Siapkan posko        │  │            │
│           │         │  └────────────────────────┘  │            │
│           │         └──────────────────────────────┘            │
│           │                                                     │
│  ┌────────▼────────────┐                                        │
│  │ PHASE 2: Koordinasi │ status: active · source: ai           │
│  ├─────────────────────┤                                        │
│  │ ☑ Kumpulkan data    │ completed                             │
│  │ ▶ Tetapkan PIC      │ active (current)                      │
│  │ ☐ Bagi tugas        │ open                                  │
│  └────────┬────────────┘                                        │
│           │                                                     │
│  ┌────────▼────────────┐                                        │
│  │ PHASE 3: Eksekusi   │ status: planned · source: ai          │
│  ├─────────────────────┤                                        │
│  │ ☐ Jalankan rencana  │ planned                               │
│  │ ☐ Laporkan progres  │ planned                               │
│  └────────┬────────────┘                                        │
│           │                                                     │
│  ┌────────▼────────────┐                                        │
│  │ PHASE 4: Verifikasi │ status: planned · source: ai          │
│  ├─────────────────────┤                                        │
│  │ ☐ Periksa hasil     │ planned                               │
│  │ ☐ Konfirmasi dampak │ planned                               │
│  └─────────────────────┘                                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Status Transitions

```
  Planned ──► Active ──► Completed
     │           │
     │           └──► Blocked ──► Active (unblocked)
     │
     └──► Skipped
```

Phases use: `Planned`, `Active`, `Completed`.
Checkpoints use: `Planned`, `Active`, `Open`, `Completed`, `Blocked`, `Skipped`.

---

## Source Tags & Locking

```
  ┌──────────────────────────────────────────┐
  │  Source Flow                              │
  │                                          │
  │  AI proposes ──► source: "ai"            │
  │       │                                  │
  │       ▼                                  │
  │  Human edits ──► source: "human"         │
  │       │            + locked_fields: [title] │
  │       │                                  │
  │       ▼                                  │
  │  AI next pass ──► SKIPS locked fields    │
  │                   proposes via diff card  │
  └──────────────────────────────────────────┘
```

**Rule**: Once a human edits a field, it gets added to `locked_fields`. The LLM can never overwrite locked fields — it must propose changes as suggestions that appear as diff cards for human review.

---

## Branching

A branch is linked to a parent checkpoint. When conditions change, the LLM can propose a new branch:

```
  Main branch
  │
  Phase 1 ── checkpoint A ── checkpoint B
  │                              │
  Phase 2 ── ...                 └── Branch: "Jika X terjadi"
                                      │
                                      Phase X1 ── checkpoint X1a
                                      Phase X2 ── checkpoint X2a
```

- Main branch: `parent_checkpoint_id: null`
- Sub-branches: `parent_checkpoint_id: "checkpoint_B"`
- Branches are labeled for clarity (e.g., "Jika air naik lagi")

---

## Dual-Tab UI Rendering

```
┌──────────────────────────────────────────┐
│  [scope ▼]    Gotong Royong    [🔍] [+]  │
├──────────────────────────────────────────┤
│  Percakapan  │  Tahapan                   │
│  ─────────────  ════════                  │
├──────────────────────────────────────────┤
│                                          │
│  Phase 1: Stabilisasi ✓                  │
│    ☑ Kumpulkan laporan lokasi            │
│    ☑ Tetapkan PIC lapangan               │
│                                          │
│  Phase 2: Koordinasi ▶                   │
│    ☑ Kumpulkan data                      │
│    ▶ Tetapkan PIC      [active]          │
│    ☐ Bagi tugas                          │
│                                          │
│  Phase 3: Eksekusi ○                     │
│    ☐ Jalankan rencana                    │
│    ☐ Laporkan progres                    │
│                                          │
│  ┌─ Branch: "Jika air naik" ──────────┐ │
│  │  Phase: Evakuasi ○                  │ │
│  │    ☐ Koordinasi BPBD               │ │
│  └─────────────────────────────────────┘ │
│                                          │
│  [🤖 AI mengusulkan perubahan]           │
│                                          │
├──────────────────────────────────────────┤
│  🏠  📋  🤝  🔔  👤                      │
└──────────────────────────────────────────┘
```

---

## Adaptive Path vs Fixed Tracks

| Aspect | Fixed Tracks (Legacy) | Adaptive Path (Current) |
|---|---|---|
| **Path definition** | 5 predefined stage sequences | LLM-generated phases + checkpoints |
| **Track role** | Lifecycle driver | Optional `track_hint` metadata |
| **Stages** | Fixed per track (4-7 stages) | Dynamic phases (N, case-specific) |
| **Branching** | None | Supported via `parent_checkpoint_id` |
| **Editing** | Fixed transitions | Role-based editing with locked_fields |
| **AI role** | Classify into track | Propose & refine path, respect locks |
| **Concurrency** | N/A | Version-based with conflict detection |
| **Audit** | Basic | SHA-256 event hashing, retention tags |

---

## API Endpoints

| Method | Route | Action |
|---|---|---|
| POST | `/v1/adaptive-path/plans` | Create plan |
| GET | `/v1/adaptive-path/plans/:plan_id` | Get plan |
| GET | `/v1/adaptive-path/plans/by-entity/:entity_id` | Get by entity |
| POST | `/v1/adaptive-path/plans/:plan_id/update` | Update plan |
| GET | `/v1/adaptive-path/plans/:plan_id/events` | List audit events |
| POST | `/v1/adaptive-path/plans/:plan_id/suggestions` | Propose suggestion |
| GET | `/v1/adaptive-path/plans/:plan_id/suggestions` | List suggestions |
| POST | `.../suggestions/:suggestion_id/accept` | Accept suggestion |
| POST | `.../suggestions/:suggestion_id/reject` | Reject suggestion |

---

## DB Tables (SurrealDB)

| Table | Purpose |
|---|---|
| `path_plan` | Canonical plan with branches (JSON), versioned |
| `path_plan_event` | Append-only audit log |
| `plan_suggestion` | AI/human proposals with status |
| `path_branch` | Normalized branch projections |
| `path_phase` | Normalized phase projections |
| `path_checkpoint` | Normalized checkpoint projections |

---

*See `docs/design/specs/ADAPTIVE-PATH-SPEC-v0.1.md` for full specification.*

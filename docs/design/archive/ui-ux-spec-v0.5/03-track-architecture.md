> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 2. Track Architecture → Adaptive Path Guidance

> **Updated 2026-02-16.** Fixed track lifecycles are superseded by Adaptive Path Guidance (`docs/design/specs/ADAPTIVE-PATH-SPEC-v0.1.md`). Tracks remain as optional classification hints only.

### 2.1 Canonical Model: Adaptive Path

Every entity starts as a signal — a testimony, idea, question, good news, or proposal. The user describes the situation in the **Percakapan** (conversation) tab. The LLM proposes a case-specific **path plan** consisting of phases and checkpoints. The **Tahapan** (timeline) tab renders the plan.

**Core hierarchy:** `PathPlan → Branch → Phase → Checkpoint`

- **Phase**: a high-level step with an objective and status.
- **Checkpoint**: a verifiable unit of progress within a phase.
- **Branch**: an alternate path forked from a parent checkpoint.
- **Source tags**: `ai` (LLM-proposed), `human` (manually edited), `system` (automated).
- **Locked fields**: any field manually edited by a privileged role becomes locked; the LLM cannot overwrite it.

Statuses: `planned`, `active`, `open`, `completed`, `blocked`, `skipped`.

### 2.2 Track Hints (Optional Metadata)

The LLM may attach `track_hint` and `seed_hint` to a plan as classification metadata. These do not drive the lifecycle — they serve as optional labels for filtering and analytics.

| Track Hint | Seed Hint | Spirit | Energy |
|---|---|---|---|
| tuntaskan | Keresahan (concern) | Fix a problem (reactive) | Tenaga + Modal |
| wujudkan | Gagasan (idea) | Build something new (proactive) | Tenaga + Modal |
| telusuri | Pertanyaan (question) | Understand / investigate | Pikiran |
| rayakan | Kabar Baik (good news) | Honor an achievement | Hati |
| musyawarah | Usul (proposal) | Decide together (governance) | Suara |

Track hints can be changed at any time by the LLM or a privileged editor without governance overhead — they are metadata, not lifecycle state.

### 2.3 Privileged Editing

Only `project_manager` or `highest_profile_user` can edit phases and checkpoints. Manual edits lock the affected fields (`locked_fields`) and increment the plan version. The LLM can propose changes as suggestions shown as diffs; users accept or reject.

### 2.4 Branching

A plan supports branches within a single timeline. Each branch anchors to a `parent_checkpoint_id` (or `null` for the main branch). Branches are labeled and rendered as forks in the Tahapan tab.

### 2.5 Legacy Track Component Patterns

The following component patterns from the original track architecture remain available as reusable UI primitives within adaptive phases. They are no longer tied to fixed stages:

- **Papan Gotong Royong** (task board) — usable in any execution phase.
- **Galang** (resource pooling) — cross-cutting feature, activatable in any phase. See Section 16.
- **Hypothesis cards & evidence board** — usable in investigation-oriented phases.
- **Validation panel & appreciation wall** — usable in celebration-oriented phases.
- **Position board & vote panel** — usable in governance-oriented phases.
- **Ketetapan** (formal decision document) — output of governance phases.

> For the full adaptive path data model and API, see `ADAPTIVE-PATH-SPEC-v0.1.md` and `docs/research/prd-adaptive-path-guidance.md`.

---

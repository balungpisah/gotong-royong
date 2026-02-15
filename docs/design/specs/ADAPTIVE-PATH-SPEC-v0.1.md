# Gotong Royong — Adaptive Path Guidance Spec v0.1

## Status
Proposed: 2026-02-15
Supersedes: fixed path choices and fixed track lifecycles described in `docs/design/specs/AI-SPEC-v0.2.md` (AI-00 triage classification) and `docs/design/specs/UI-UX-SPEC-v0.5.md` (track architecture and fixed stages). This spec replaces those assumptions for new experiences while keeping legacy references for history.

## 1. Purpose
Users describe real-world situations that do not always fit a fixed lifecycle. The system should let an LLM propose a case-specific path (phases and checkpoints) and guide the user through it. The path is still rendered through the existing parsed JSON UI component system.

## 2. Principles
- LLM guides the path, users own the outcome.
- Generate the minimum viable path first, then refine.
- The LLM must not overwrite manual edits.
- Only privileged roles can edit phases and checkpoints.
- All outputs are structured JSON, versioned, and auditable.

## 3. Entry Flow
1. User starts in the conversation tab and explains the case.
2. The LLM proposes an initial path plan with phases and checkpoints.
3. The UI shows two tabs.
4. Tab A: conversation with the LLM.
5. Tab B: the path timeline rendered from JSON.
6. The user can ask the LLM to refine the plan, but manual edits remain locked.

## 4. UI Behavior (Dual Tabs)
- Tab A: `Percakapan` (ongoing chat, clarifications, refinements).
- Tab B: `Tahapan` (scrollable timeline).
- Each phase is a section with a header, objective, and checkpoint list.
- Checkpoints are separated by clear dividers and include status badges.
- Branches appear as forked sections with a branch label and parent checkpoint reference.

## 5. Data Model (Parsed JSON)
The LLM returns a plan object. The UI renders it using the existing JSON component parser and block primitives.

```json
{
  "path_plan": {
    "plan_id": "plan_01",
    "version": 1,
    "title": "Penanganan Banjir RT 05",
    "summary": "Fokus pada keselamatan warga dan pemulihan akses.",
    "track_hint": "tuntaskan",
    "seed_hint": "Keresahan",
    "branches": [
      {
        "branch_id": "main",
        "label": "Utama",
        "parent_checkpoint_id": null,
        "phases": [
          {
            "phase_id": "p1",
            "title": "Stabilisasi",
            "objective": "Pastikan warga aman dan informasi terkumpul.",
            "status": "active",
            "source": "ai",
            "locked_fields": [],
            "checkpoints": [
              {
                "checkpoint_id": "c1",
                "title": "Kumpulkan laporan lokasi terdampak",
                "status": "open",
                "source": "ai",
                "locked_fields": []
              },
              {
                "checkpoint_id": "c2",
                "title": "Tetapkan PIC lapangan",
                "status": "open",
                "source": "ai",
                "locked_fields": []
              }
            ]
          }
        ]
      },
      {
        "branch_id": "b1",
        "label": "Jika air naik lagi",
        "parent_checkpoint_id": "c2",
        "phases": [
          {
            "phase_id": "p2",
            "title": "Evakuasi",
            "objective": "Siapkan jalur evakuasi dan posko.",
            "status": "planned",
            "source": "ai",
            "locked_fields": [],
            "checkpoints": [
              {
                "checkpoint_id": "c3",
                "title": "Koordinasi dengan BPBD",
                "status": "open",
                "source": "ai",
                "locked_fields": []
              }
            ]
          }
        ]
      }
    ]
  }
}
```

### 5.1 Field Rules
- `track_hint` and `seed_hint` are optional metadata only. They do not force a fixed lifecycle.
- `locked_fields` is a list of field names that cannot be modified by the LLM after manual edits.
- `source` must be one of `ai`, `human`, or `system`.

## 6. Editing and Governance
- Only `project_manager` or `highest_profile_user` roles can edit phases and checkpoints.
- When a privileged user edits a field, that field is added to `locked_fields` for the affected object.
- The LLM can propose edits, but the UI must show them as suggestions and never auto-apply on locked fields.
- Manual edits override AI, and the AI must reference the latest plan version when refining.

## 7. Branching Semantics
- There is one timeline with branches.
- A branch must anchor to a `parent_checkpoint_id` or to the root (null for the main branch).
- Branches must be labeled and rendered as forks to avoid confusion.

## 8. Prompt Guidance
The LLM prompt must:
- Ask clarifying questions only until a minimal plan is possible.
- Produce JSON output that follows the plan schema.
- Keep phases short and outcome-focused.
- Provide checkpoints with clear, verifiable completion criteria.
- Avoid forcing a legacy track unless the user explicitly wants it.

## 9. Non-Goals
- This spec does not redefine credit, moderation, or reputation systems.
- This spec does not remove existing track artifacts; it supersedes them for new flows.

## 10. Change Control
- All plan generations include `plan_id`, `version`, `model_id`, and `prompt_version` in storage.
- Any manual edit increments the plan version and sets `source: human` plus `locked_fields`.
- AI-generated revisions must target the latest version and respect locks.


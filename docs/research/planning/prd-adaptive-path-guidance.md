# PRD — Adaptive Path Guidance (PR-16) v0.1

Status: Proposed
Date: 2026-02-15
Owner: TBD

## Related References
- `docs/design/specs/ADAPTIVE-PATH-SPEC-v0.1.md`
- `docs/design/specs/AI-SPEC-v0.2.md`
- `docs/design/specs/UI-UX-SPEC-v0.5.md`
- `docs/design/backend-design-contract-gotong-tandang.md`

## Problem Statement
Fixed track lifecycles are too rigid for real-world cases. Users need a case-specific path that adapts to context and can branch. The system must still be correct, auditable, and governed by roles, while enabling an LLM to guide the plan.

## Goals
- Generate an adaptive, case-specific path of phases and checkpoints from user conversation.
- Make the adaptive plan the canonical source of truth for progress.
- Enforce role-based editing with immutable locks on manually edited fields.
- Support branching within a single timeline to handle multi-path cases.
- Preserve the existing parsed JSON UI component system.

## Non-Goals
- Rewriting the credit, moderation, or reputation systems.
- Removing legacy tracks entirely; they remain optional metadata or legacy views.
- Implementing frontend visuals or prototypes in this PRD.

## Personas and Roles
- `author`: creates the case and participates in discussion.
- `participant`: contributes to discussion and tasks.
- `project_manager`: privileged editor of phases and checkpoints.
- `highest_profile_user`: privileged editor based on highest reputation tier in the scope.
- `system`: non-human actor for automated jobs and validations.
- `ai`: LLM proposal source only, never an authoritative editor.

## Definitions
- `path_plan`: the canonical adaptive plan for a case.
- `phase`: a high-level step in the plan with objectives.
- `checkpoint`: a verifiable unit of progress within a phase.
- `branch`: an alternate path linked to a parent checkpoint.
- `locked_fields`: fields that cannot be modified by the LLM after manual edits.

## Primary User Journeys
1. User enters a case in the conversation tab. LLM proposes a plan. The timeline tab renders phases and checkpoints.
2. Privileged editor adjusts a phase title and checkpoint list. Those fields become locked.
3. User asks LLM to refine the plan. LLM returns a proposal that excludes locked fields. User accepts or rejects suggestions.
4. A new scenario appears. LLM proposes a branch linked to a specific checkpoint in the main timeline.

## Functional Requirements
- FR-01: The system stores a `path_plan` for each case as canonical progress state.
- FR-02: The LLM generates an initial plan in structured JSON that the UI can render.
- FR-03: Only `project_manager` or `highest_profile_user` can edit phases and checkpoints.
- FR-04: Manual edits lock affected fields and increment plan version.
- FR-05: The LLM must not overwrite locked fields and must return proposals as suggestions.
- FR-06: The UI shows suggestions as diffs, not auto-applied changes.
- FR-07: A plan supports branches linked to a parent checkpoint.
- FR-08: All writes are idempotent and auditable with `request_id` and `correlation_id`.
- FR-09: Plan history is retained for review and rollback.
- FR-10: Track and seed remain optional hints, not the lifecycle driver.

## Data Model Requirements
- `path_plan` header: `plan_id`, `entity_id`, `version`, `title`, `summary`, `track_hint`, `seed_hint`, `created_at`, `updated_at`.
- `path_branch`: `branch_id`, `label`, `parent_checkpoint_id`, `order`.
- `path_phase`: `phase_id`, `branch_id`, `title`, `objective`, `status`, `order`, `source`, `locked_fields`.
- `path_checkpoint`: `checkpoint_id`, `phase_id`, `title`, `status`, `order`, `source`, `locked_fields`.
- `plan_suggestion`: `suggestion_id`, `plan_id`, `base_version`, `proposal_json`, `created_by`, `created_at`.
- `plan_event`: append-only events for create, update, suggestion, accept, reject.

## LLM Contract Requirements
- Input includes conversation history, current plan snapshot, and locked fields.
- Output is structured JSON conforming to the plan schema or a delta proposal.
- Output includes `prompt_version` and `model_id`.
- Output must never modify locked fields and must mark changes as proposals.
- If insufficient context, the LLM asks clarifying questions instead of guessing.

## UI Requirements
- Dual tabs: `Percakapan` (conversation) and `Tahapan` (timeline).
- Timeline is scrollable, with phases grouped by branches.
- Each checkpoint shows status and is visually separated.
- Locked fields are visually indicated and non-editable in UI.
- Suggestions appear as diffs with accept or reject actions.

## Correctness and Concurrency
- Plan versions are monotonic. Updates must target the latest version.
- Conflicting edits are rejected with a conflict error and require refresh.
- AI proposals are tied to a base version and become invalid if the plan updates.
- Every update is idempotent and replay-safe.

## Security and Permissions
- Role checks are enforced on every edit and suggestion application.
- The definition of `highest_profile_user` must be deterministic and scoped.
- Access to private content and vault data must respect existing privacy rules.

## Observability
- Metrics for plan creation latency, suggestion acceptance rate, and conflict rate.
- Audit log for all plan edits and suggestions.

## Success Metrics
- Time-to-first-plan within target latency budget.
- Percentage of LLM proposals accepted by privileged editors.
- Low conflict rate for concurrent edits.
- Zero incidents of LLM overwriting locked fields.

## Risks
- LLM over-structures or under-structures the plan.
- Permissions ambiguity for `highest_profile_user`.
- Branch complexity leading to user confusion.

## Open Questions
- What is the authoritative rule for `highest_profile_user` by scope and time window?
- Should plan suggestions be stored as full plan snapshots or JSON deltas?
- What is the maximum plan size before the UI degrades?

## PR-16 Scope Mapping
- Backend: schema, repositories, event log, idempotent APIs.
- AI: prompt contracts, versioning, and suggestion-only outputs.
- UI: rendering of adaptive plan JSON and suggestion diffs.


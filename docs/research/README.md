# Backend Research Workspace

Purpose: Prepare factual, evidence-backed requirements for backend design before implementation.

This workspace is used to:
1. extract UI-driven requirements from Gotong UX/docs,
2. map each requirement to existing contract sources (Gotong + Tandang + Edge-Pod),
3. identify missing decisions or gaps,
4. only then begin backend design specification and implementation planning.

## Status

- Folder created for research-only work.
- No implementation decisions are considered final until the research passes the gates below.

## Research phases (step-by-step)

### Phase 0 — Inventory

Collect all UI-capable surfaces and features that require backend behavior.

- Read: `docs/design/specs/UI-UX-SPEC-v0.5.md`
- Read: `docs/design/prototypes/*` index references
- Read: any screen flow diagrams in `docs/design/context/DESIGN-SEQUENCE.md`

Output:
- `ui-feature-inventory.md` with one row per UI feature.

### Phase 1 — Data Contract Mapping

For each UI feature row:

1. Identify backend APIs required.
2. Identify required payload fields and response contracts.
3. Identify triggers (UI event / user action / schedule / webhook).
4. Identify ownership (`Gotong-owned`, `Tandang-owned`, `shared`).

Output:
- `feature-contract-map.md` with unresolved items marked as `UNKNOWN`.

### Phase 2 — Tandang Compatibility Validation

For each feature using reputation/score/activity, confirm:

1. Is there a direct Tandang equivalent?
2. Which event type is used (`contribution_created`, `vouch_submitted`, `por_evidence`)?
3. Is any identity, evidence, or audit metadata required?

Output:
- `tandang-gap-log.md` with `KNOWN`, `MISSING`, or `CONFLICT`.

### Phase 3 — AI/Edge-Pod Convention Alignment

For AI-backed features:

1. Confirm AI touchpoint (AI-00..AI-09).
2. Confirm input/output schema and versioning (`prompt_id`, `version`, `confidence`).
3. Confirm fallback behavior when model unavailable.

Output:
- `ai-contract-log.md`.
- `edgepod-endpoint-contracts.md` (endpoint matrix + prompt bundle for Edge-Pod).
- `prompt-id-registration-log.md` (AI/Edge-Pod prompt/version ownership and missing IDs).

### Phase 4 — Readiness Gates

Before moving to formal backend design:

- No feature row has unknown required contract references.
- Every `shared` feature has Tandang mapping and clear event contract.
- Every AI feature has prompt/version + fallback spec.
- Every trigger is explicit (`user_action`, `timer`, `webhook`, `async_batch`, etc.).

## Suggested research outputs

Create/update these files in order:

1. `ui-feature-inventory.md`
2. `feature-contract-map.md`
3. `tandang-gap-log.md`
4. `ai-contract-log.md`
5. `edgepod-endpoint-contracts.md`
6. `edgepod-endpoint-contracts.schema.json` (strict contract bundle)
7. `edgepod-endpoint-contracts.contract-map.md` (compact endpoint summary)
8. `edgepod-endpoint-contracts/` (per-endpoint JSON contract files: `EP-00.schema.json` … `EP-11.schema.json`)
9. `research-summary-and-decision-record.md`
10. `decision-questions-pack.md`
11. `decision-notes-template.md`
12. `blocker-resolution-handoff-brief.md`
13. `decision-notes.md`
14. `backend-implementation-tickets.md`

## File list

- `research/README.md` (this file)
- `research/ui-feature-inventory.md` (to be filled during Phase 0)
- `research/feature-contract-map.md` (to be filled during Phase 1)
- `research/tandang-gap-log.md` (to be filled during Phase 2)
- `research/ai-contract-log.md` (to be filled during Phase 3)
- `research/decision-questions-pack.md` (to be filled during unresolved blockers resolution)
- `research/decision-notes-template.md` (to capture final answers during blocker resolution)
- `research/decision-notes.md` (final approved decision record)
- `research/blocker-resolution-handoff-brief.md` (stakeholder-ready blockers brief and required outputs)
- `research/edgepod-endpoint-contracts.md` (to be filled during Phase 3/4 handoff)
- `research/prompt-id-registration-log.md` (prompt/version ownership and pending prompt IDs)
- `research/edgepod-endpoint-contracts.schema.json` (strict endpoint contracts)
- `research/edgepod-endpoint-contracts.contract-map.md` (compact endpoint map for quick handoff)
- `research/edgepod-endpoint-contracts/EP-00.schema.json` ... `research/edgepod-endpoint-contracts/EP-11.schema.json` (per-endpoint contract snippets)
- `research/research-summary-and-decision-record.md` (to be produced after Gate pass)
- `research/backend-implementation-tickets.md` (language-agnostic implementation ticket pack and dependency order)

## Non-coding rule

No backend design or implementation should begin until:

1. Phase 0–3 are fully completed.
2. All four gate conditions in Phase 4 are satisfied.
3. `research-summary-and-decision-record.md` is approved.

# Plan: Strip Old Fixed-Track Concept from Documentation

## Goal
Make documentation coherent with the actual code — Adaptive Path Guidance is the canonical model, not fixed track lifecycles.

## Key Principle
**Strip, don't annotate.** Remove old fixed-stage sequences entirely. Tracks remain as optional `track_hint` metadata, not lifecycle drivers.

---

## BATCH 1: Foundation (Source of Truth)

### 1.1 Rewrite `docs/design/context/DESIGN-CONTEXT.md`
- Replace the 5-track lifecycle table (lines 19-25) — remove "Lifecycle" column with fixed stage sequences
- Add new section: "Adaptive Path Terminology" (plan, branch, phase, checkpoint, source, locked_fields)
- Reframe tracks as optional hints, not lifecycle drivers
- Keep everything else (Tanah mood, reputation, credit, AI touch points, design tokens)

### 1.2 Archive `docs/design/context/TRACK-MAP.md` → `docs/design/archive/`
- Entire file is ASCII flowcharts for fixed stages — archive it

### 1.3 Archive `docs/design/context/DESIGN-SEQUENCE.md` → `docs/design/archive/`
- Phase-by-phase checklist encoding fixed card designs — archive it

### 1.4 Create `docs/design/context/ADAPTIVE-PATH-MAP.md`
- Replace TRACK-MAP with adaptive path visual reference (phases, branches, checkpoints)

---

## BATCH 2: UI-UX Spec Chapters (Major Rewrites)

### 2.1 Rewrite `ui-ux-spec/03-track-architecture.md`
- Remove fixed stage sequences from Section 2.1
- Reframe: tracks as spirit/hints, adaptive phases as canonical path

### 2.2 Archive `ui-ux-spec/04-stages-per-track.md` → `docs/design/archive/`
- Replace with new `04-adaptive-phase-patterns.md` showing common phase patterns by intent

### 2.3 Edit `ui-ux-spec/05-bahas-decision.md`
- Reframe as "Discussion Phase Pattern" — keep mechanics, remove fixed-stage assumption

### 2.4 Edit `ui-ux-spec/07-governance-voting.md`
- Reframe voting as phase-agnostic — operates on checkpoints, not fixed stages

### 2.5 Edit `ui-ux-spec/13-transitions-and-quorum.md`
- Replace "stage transition" → "checkpoint transition", keep quorum/challenge mechanics

### 2.6 Edit `ui-ux-spec/17-galang-deep-ux.md`
- Reframe Galang as cross-cutting feature usable in any adaptive path, not Wujudkan-only

### 2.7 Rewrite `ui-ux-spec/20-bagikan-create-flow.md`
- AI-00 now proposes adaptive path plan, not fixed track classification
- Keep context bar, three outcomes (Komunitas/Vault/Siaga)

### 2.8 Edit `ui-ux-spec/23-ai-layer-cross-reference.md`
- Update all 10 AI touch points to reference adaptive phases instead of fixed stages

---

## BATCH 3: Design DNA & AI Spec

### 3.1 Edit `design-dna/05-card-system.md`
- Rewrite Section 5.3: card patterns by spirit, not fixed track lifecycle counts

### 3.2 Edit `design-dna/06-entry-flows.md`
- AI-00 triage → adaptive path proposal (not fixed track classification)

### 3.3 Edit `ai-spec/01-overview-and-scope.md`
- Clarify AI-00 proposes adaptive paths; AI-01 provides optional hints

### 3.4 Edit `ai-spec/05-ai-01-classification.md`
- Reframe as "Optional Metadata Classification" — track/seed are hints, not drivers

### 3.5 Edit `ai-spec/13-ai-09-credit-accreditation.md`
- Update credit mapping: generic phase actions instead of track-specific stages

---

## BATCH 4: Prototypes & Index

### 4.1 Create `docs/design/prototypes/LEGACY.md`
- Explain B0-B5 prototypes encode fixed-track era, still valid for design DNA reference

### 4.2 Add legacy HTML comments to B0-B5 and related prototypes

### 4.3 Update `docs/DESIGN-INDEX.md`
- Add Adaptive Path docs to reading order, link archived files

### 4.4 Update `docs/design/specs/README.md`
- Reference ADAPTIVE-PATH-SPEC as primary spec

---

## BATCH 5: Verification

### 5.1 Grep all docs for orphaned fixed-stage references
- Scan for: Bahas→, Rancang→, Garap→, Periksa→, fixed stage sequences
- Fix any remaining inconsistencies

### 5.2 Verify cross-document links (no broken refs to archived files)

### 5.3 Final coherence check: terminology consistency across all updated files

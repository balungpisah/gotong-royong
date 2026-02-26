# Chat Interaction Blocks Contract (v1)

Last updated: 2026-02-26  
Owner: AI orchestration + UI contract slice  
Status: active reference

## Purpose

Define a single source of truth for the **conversation layer** and **structured layer** blocks used in chat-driven phases, so:
- operators know what they can output,
- phase logic stays consistent across trajectories,
- future taxonomy additions can be validated against a stable block catalog.

This version documents canonical blocks and their sources. It does not lock per-phase schemas.

## 1) Two-Layer Model (always-on)

Every trajectory card has two layers:
- **Conversation layer**: chat stream + inline AI cards.
- **Structured layer**: block primitives rendered in the "Tahapan / Papan / Rangkuman" tab.

The LLM **never auto-applies** changes. It proposes via diff cards, humans decide.

## 2) Conversation Layer Blocks

Canonical chat blocks in the scroll:

1. `chat_message`
- Author: `human | ai | system`
- Includes: text, timestamp, optional media, reactions.

2. `ai_inline_card`
- Generic AI inline surface (`.ai-chat-card`).
- Uses: summaries, nudges, cross-tab hints, follow-up prompts.

3. `diff_card`
- Proposes structured-layer changes with citations.
- Actions: `apply_all | review_one_by_one | dismiss`.

4. `vote_card`
- Lightweight inline vote surface (e.g., mufakat prompt, siaga confirmation).
- Tallies remain system-derived.

5. `moderation_hold_card`
- AI-04 moderation hold states (author/moderator/public views).

6. `duplicate_detection_card`
- AI-03 duplicate detection comparison prompt.

7. `credit_nudge_card`
- AI-09 contribution nudges and completion credit proposal cues.

Sources:
- `chat_message`: `human | ai | system`
- `ai_inline_card`: `ai | system`
- `diff_card`: `ai` (proposal), `system` (final apply)
- `vote_card`: `system`
- `moderation_hold_card`: `system`
- `duplicate_detection_card`: `ai` (proposal), `system` (final state)
- `credit_nudge_card`: `ai` (proposal), `system` (final state)

## 3) Structured Layer Block Primitives

The structured layer is composed from these **seven primitives**:

1. `list`
- Checklist, table, timeline, gallery, evidence list, contributors.

2. `document`
- Rich text with tracked changes.

3. `form`
- Labeled fields, with `protected` fields excluded from AI edits.

4. `computed`
- Read-only computed values (progress, readiness, tallies).

5. `display`
- Presentational card (celebration, recognition).

6. `vote`
- Voting interface with system tallies.

7. `reference`
- Cross-card pointers or linked resources.

Source tags:
- `ai`: LLM-generated, editable by humans.
- `human`: human-authored or edited; locked from AI updates.
- `system`: derived; read-only.

## 4) Minimal Block Coverage by Trajectory

Each trajectory should be implementable using the block catalog below. This is a **coverage baseline**, not a rigid schema.

| Trajectory | Required block primitives | Preferred block primitives |
|---|---|---|
| `aksi`, `advokasi` | `list`, `document`, `computed` | `reference`, `form` |
| `pantau` | `list`, `document`, `computed` | `reference` |
| `mufakat`, `mediasi` | `vote`, `list`, `document` | `reference`, `computed` |
| `program` | `list`, `form`, `computed` | `document`, `reference` |
| `data` | `form`, `document` | `computed`, `reference` |
| `bantuan` | `form`, `list`, `computed` | `document`, `reference` |
| `pencapaian` | `display`, `document` | `list`, `reference` |
| `siaga` | `form`, `list`, `computed` | `reference` |
| `vault` | `document` | `reference`, `list` (redacted) |

Notes:
- The LLM selects which blocks to use based on phase context and available info.
- `protected` fields in `form` blocks are human-only (AI never writes).
- `vote` is only meaningful for consensus phases; other trajectories should avoid it.

## 5) Cross References

- `docs/design/context/DESIGN-CONTEXT.md` (LLM ↔ UI architecture, block primitives)
- `docs/research/contracts/triage-operator-output-contract-v1.md`
- `docs/research/contracts/feed-card-blocks-v1.md`

## Appendix A) Block Variants (render patterns)

These are **render variants**, not new primitives.

### `list` variants
- `checklist`: items with `status`, `assignee`, `due_at`
- `timeline`: items with `timestamp`, `event`, `actor`
- `roster`: items with `person`, `role`, `availability`
- `evidence`: items with `title`, `media_url`, `source`, `notes`
- `gallery`: items with `media_url`, `caption`

### `form` variants
- `location`: map + address + coordinates
- `budget`: amount fields (mark `protected: true` where required)
- `contact`: name, phone, organization, role
- `upload`: file input with `mime_type`, `size`, `hash`

### `reference` variants
- `artifact_link`: downloadable file reference (`url`, `mime_type`, `size`)
- `card_link`: pointer to another witness/data card

### `chat_message` attachments
- Attachments are rendered inline with the message.
- For structured use, the same attachment should be referenced in `list.evidence` or `reference.artifact_link`.

## Appendix B) Phase → Likely Variants (guidance)

This is guidance, not a schema. Phases remain adaptive.

| Phase intent | Likely variants |
|---|---|
| Problem framing | `document` (summary), `list.checklist` (open questions) |
| Evidence gathering | `list.evidence`, `list.timeline`, `reference.artifact_link` |
| Planning | `list.checklist`, `form.contact`, `form.budget`, `computed` (readiness) |
| Execution | `list.checklist`, `list.roster`, `computed` (progress) |
| Verification | `list.evidence`, `vote`, `computed` (quorum/thresholds) |
| Consensus | `vote`, `document` (proposal), `computed` (counts) |
| Celebration | `display`, `document`, `reference.card_link` |
| Alert/Response | `form.location`, `list.timeline`, `computed` (status) |

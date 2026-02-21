# Design Specifications

The large spec files are split into themed sub-documents to reduce merge conflicts and speed up handoff.

## Primary Specs (Canonical)

| File | Status | Description |
|------|--------|-------------|
| `ADAPTIVE-PATH-SPEC-v0.1.md` | **PRIMARY** | Data model for adaptive path; supersedes all fixed-track lifecycle descriptions |
| `ADAPTIVE-PATH-ORCHESTRATION-v0.1.md` | Current | Orchestration flow — who does what, when |
| `ENTRY-PATH-MATRIX-v0.1.md` | Current | Eagle view — all 4 modes, routing logic, Catatan Komunitas spec |
| `ONTOLOGY-VOCAB-v0.1.md` | Current | Vocabulary conventions — Schema.org + Wikidata + OSM + SurrealDB graph |
| `DESIGN-DNA-v0.1.md` | Current | Modular index for design system |
| `AI-SPEC-v0.2.md` | Current | Modular index for 10 AI touch points |
| `UI-UX-SPEC-v0.5.md` | Current | Modular index for product flow and interaction patterns |

> **Note**: Any document referencing fixed Bahas→Rancang→Garap→Periksa stage sequences as lifecycle drivers is superseded. Use `ADAPTIVE-PATH-SPEC-v0.1.md` instead.

## Module Folders

- [ai-spec](./ai-spec/) — AI touch point definitions (10 AI points + governance, prompts, and metrics)
- [design-dna](./design-dna/) — Core design system, tokens, component, and card architecture
- [ui-ux-spec](./ui-ux-spec/) — Product flow, governance, privacy, permissions, and interaction patterns
- [tandang](./tandang/) — Tandang integration contract, signal inventory, and GR integration briefs/prompts

Prefer linking to these section files from implementation tickets instead of referencing the root indexes.

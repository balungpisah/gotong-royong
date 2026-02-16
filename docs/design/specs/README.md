# Design Specifications

The large spec files are split into themed sub-documents to reduce merge conflicts and speed up handoff.

- `DESIGN-DNA-v0.1.md` (modular index)
- `AI-SPEC-v0.2.md` (modular index)
- `UI-UX-SPEC-v0.5.md` (modular index)
- `ADAPTIVE-PATH-SPEC-v0.1.md` (data model, supersedes fixed path choices)
- `ADAPTIVE-PATH-ORCHESTRATION-v0.1.md` (orchestration flow — who does what, when)
- `ENTRY-PATH-MATRIX-v0.1.md` (eagle view — all 4 modes, routing logic, Catatan Komunitas spec)
- `ONTOLOGY-VOCAB-v0.1.md` (vocabulary conventions — Schema.org + Wikidata + OSM + SurrealDB graph)

## Module Folders

- [ai-spec](./ai-spec/) — AI touch point definitions (10 AI points + governance, prompts, and metrics)
- [design-dna](./design-dna/) — Core design system, tokens, component, and card architecture
- [ui-ux-spec](./ui-ux-spec/) — Product flow, governance, privacy, permissions, and interaction patterns

Prefer linking to these section files from implementation tickets instead of referencing the root indexes.

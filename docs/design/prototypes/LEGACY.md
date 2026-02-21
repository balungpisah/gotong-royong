# Legacy Track Prototypes

## Overview

The `legacy-tracks/` subdirectory contains HTML prototypes from the **fixed-track era** of Gotong Royong design (iterations B0–B5). These prototypes were built when the platform used a rigid five-track lifecycle model.

## Why These Are Legacy

The fixed-track model (Tuntaskan, Wujudkan, Telusuri, Rayakan, Musyawarah) has been superseded by the **Adaptive Path model**, in which:

- Tracks exist only as optional `track_hint` metadata on Kabar entries
- The AI layer (AI-00) proposes case-specific phases and checkpoints based on community context
- No fixed Bahas→Rancang→Garap→Periksa stage sequence is enforced

See [`docs/design/context/ADAPTIVE-PATH-MAP.md`](../context/ADAPTIVE-PATH-MAP.md) for the current model.

## File Inventory

| File | Track / Era | Description |
|------|------------|-------------|
| `B0-seed-card.html` | Seed / Pre-track | Initial card state before track assignment |
| `B1-tuntaskan-card.html` | Tuntaskan (Task) | Fixed-stage task-completion track card |
| `B2-wujudkan-card.html` | Wujudkan (Build) | Fixed-stage project/build track card |
| `B3-telusuri-card.html` | Telusuri (Research) | Fixed-stage research/investigation track card |
| `B4-rayakan-card.html` | Rayakan (Celebrate) | Celebration/milestone track card |
| `B5-musyawarah-card.html` | Musyawarah (Governance) | Community deliberation/voting track card |

## Current Prototypes

Active prototypes (adaptive-path era) live alongside these files:

| Series | Purpose |
|--------|---------|
| `A1–A4` | Design system foundations (mood, palette, typography, components) |
| `A+1–A+3` | Cross-cutting feature surfaces (triage, catatan-saksi, siaga) |
| `C1–C6` | Core interaction surfaces (overlays, AI states, navigation, share) |
| `D2` | Style guide |
| `adaptive-path/AP1-adaptive-path-card.html` | Adaptive path card (current model) |

## References

- [ADAPTIVE-PATH-MAP.md](../context/ADAPTIVE-PATH-MAP.md) — Current path model
- [design-dna/06-entry-flows.md](../specs/design-dna/06-entry-flows.md) — AI-00 adaptive path proposal flow
- [ui-ux-spec/03-track-architecture.md](../specs/ui-ux-spec/03-track-architecture.md) — Tracks as spirit/hints

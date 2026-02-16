> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 12. Checkpoint Transitions & Governance

> **Updated 2026-02-16.** Transitions now apply to checkpoint and phase status changes within an adaptive path, not fixed stage-to-stage progression.

| Transition Type | Risk | Mechanism | Duration |
|---|---|---|---|
| Checkpoint open → completed (routine) | Low | Consent | 24h |
| Phase activation (simple) | Low | Consent | 24h |
| Phase activation (complex, ≥5 participants) | Medium | Vote | 48h |
| Galang-related checkpoint | Medium-High | Vote | 48h |
| Checkpoint requiring evidence | High | Vote + evidence review | 72h |
| Plan completion (final phase → completed) | High | Vote + challenge window | 72h |
| Emergency fast-track | Critical | Fast-track + 7-day post-hoc audit | Immediate |

Governance rules apply to any adaptive path regardless of track hint. The risk level is determined by the checkpoint's governance metadata, not by its position in a fixed sequence.

---

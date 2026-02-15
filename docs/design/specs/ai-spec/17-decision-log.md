> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 17. Decision Log

Formal decisions made during AI-SPEC v0.2 design.

| Decision ID | Title | Resolution | Rationale |
|---|---|---|---|
| **D01** | Single 3-layer architecture | Adopted | Consistency across all touch points; separation of concerns |
| **D02** | Tandang owns embeddings/voting | Adopted | Decouples AI from infrastructure; Tandang is source of truth |
| **D03** | Bahasa Indonesia as default AI language | Adopted | All prompts, outputs, user-facing text in Bahasa Indonesia |
| **D04** | PIC has final say on distribution | Adopted | Preserves human agency; AI is advisor, not dictator |
| **D05** | Automatic moderation release <60% confidence | Adopted | Low confidence = ambiguity; benefit of doubt to user |
| **D06** | No community credit in vault / siaga | Adopted | Protects privacy; vault is not for credit-seeking |
| **D07** | Context bar 8-state morphing | Adopted | Real-time UX feedback; users see AI's "thinking" |
| **D08** | AI-00 replaces empty textarea | Adopted | Removes friction; conversational entry is friendlier |
| **D09** | Full conversation → first Diskusi message | Adopted | Preserves context; users don't repeat themselves |
| **D10** | Sonnet for conversational, Haiku for structured | Adopted | Cost efficiency + latency balance |
| **D11** | AI-00 conversational triage replaces textarea + auto-classify | Adopted (v0.2) | Improves UX; guides user instead of requiring choices |
| **D12** | AI-09 is passive/mixed, not real-time per-action | Adopted (v0.2) | Reduces complexity; cleaner credit calculations at end |
| **D13** | Context bar morphing replaces step-by-step bottom sheet | Adopted (v0.2) | More elegant UX; less screen real estate |
| **D14** | Zero Tandang credit while vault sealed / siaga active | Adopted (v0.2) | Protects emergency response urgency; privacy respected |

---


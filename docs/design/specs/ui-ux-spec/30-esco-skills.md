> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 29. ESCO Skill System (NEW)

### 29.1 Skill Extraction

ESCO skills auto-tagged during AI-00 triage via Tandang POST /extract-skills. Returns skill URIs with confidence scores.

### 29.2 Display on Seed Cards

Pills in Section 4 (Skill Tags) of seed card anatomy. Two visual states:

| State | Visual | Meaning |
|---|---|---|
| Tervalidasi | ● filled circle | Confirmed by Tandang (project completion, peer verification) |
| Dinyatakan | ○ outlined circle | Self-declared by user |

### 29.3 Skill on Profil (CV Hidup)

Dual-layer display: Tervalidasi ● above, Dinyatakan ○ below. "+ Tambah Keahlian" button for self-declaration. Decay nudge: visual indicator when skill hasn't been exercised recently.

### 29.4 Skill Matching (Bantu Tab)

Bantu tab surfaces opportunities matching user's ESCO skills. Seeds needing help with matching skill domains appear. Volunteer counts per seed. Validated skills weighted higher in matching.

### 29.5 Skill in Search

Search filter group includes ESCO skill tags. Matched skills highlighted in results.

---

*— End of Document —*

*Gotong Royong UI/UX Specification v0.5 · Februari 2026 · DRAFT*

*Companion documents: AI-SPEC v0.2 (full AI touch point specs), DESIGN-DNA v0.1 (formal design system reference), D2-style-guide.html (interactive token/component reference).*

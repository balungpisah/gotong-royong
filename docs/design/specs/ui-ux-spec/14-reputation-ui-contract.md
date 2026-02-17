> **UI source of truth:** [UI-GUIDELINE-v1.0.md](../UI-GUIDELINE-v1.0.md) — Domain logic in this file remains active reference. UI interaction patterns are superseded by the Chat-First model.

> [← Back to UI/UX Spec index](../UI-UX-SPEC-v0.5.md)

## 13. Reputation UI Contract (UPDATED)

### 13.1 Tier Badge System (◆◇)

| Tier | Badge | Color | Hex |
|---|---|---|---|
| 4 · Kunci | ◆◆◆◆ | Gold | #FFD700 |
| 3 · Pilar | ◆◆◆◇ | Blue | #1E88E5 |
| 2 · Kontributor | ◆◆◇◇ | Teal | #00695C |
| 1 · Pemula | ◆◇◇◇ | Brown | #795548 |
| 0 · Bayangan | ◇◇◇◇ | Red/Gray | var(--c-bahaya) |

**Full display:** `◆◆◆◇ Pilar` — for profile headers, leaderboard, detail cards.

**Compact pip:** 18px colored circle with tier number — for avatar overlays, inline bylines, comment headers. Example: avatar-md + [3] blue pip.

### 13.2 I/C/J Score Axes

| Axis | Name | Color | Hex |
|---|---|---|---|
| I | Inisiatif | Amber | #F57F17 |
| C | Kompetensi | Teal | #00695C |
| J | Penilaian | Purple | #7B1FA2 |

Radar SVG on Profil page. Self-view shows numeric values. Others see ring + tier only (no numbers).

### 13.3 GDF Weather Widget

| Weather | Emoji | Multiplier | Meaning |
|---|---|---|---|
| Cerah | ☀️ | 1.0× | Active community |
| Berawan | 🌤️ | 1.2× | Moderate activity |
| Hujan | 🌧️ | 1.5× | Low activity, needs contributions |
| Badai | ⛈️ | 2.0× | Crisis, contributions urgently needed |

Compact version in Community Pulse bar. Full version on Profil Tandang.

### 13.4 CV Hidup (Living CV)

Hero section: avatar-xl + tier badge + name + community. Score cards: I/C/J with decay info. Dual-layer skills: Tervalidasi ● (filled) vs Dinyatakan ○ (outlined). + Tambah Keahlian button. Decay nudge on unused skills. Kontribusi timeline (chronological). Vouch section: Dijamin oleh / Menjamin. Impact metrics. QR code for sharing.

### 13.5 Avatar Warmth

Last 7 days: 100% opacity. Last 30 days: 90%. >30 days: 70%.

---


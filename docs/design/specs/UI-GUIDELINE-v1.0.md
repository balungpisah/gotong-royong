# Gotong Royong — UI Guideline v1.0

> A witness-first community coordination platform built on the Tandang Markov Credential Engine
>
> February 2026 · v1.0 · SINGLE SOURCE OF TRUTH

---

## Changelog

**v1.0 (February 2026) — Major UI paradigm shift**
- **NEW: Chat-First Interaction Model** — replaces Dual-Tab Pattern (v0.5 Section 22)
- **NEW: Drawable Phase Panel + Breadcrumb Timeline** — replaces "Living Card changes faces" and "Phase-Specific Detail Pages"
- **NEW: Responsive Breakpoint Strategy** — mobile (drawable), tablet (hybrid), desktop (side panel)
- **Carried forward (unchanged):** Adaptive Path model, Navigation & Feed, Bagikan AI-00 triage, LLM Architecture (7 blocks), Governance, Roles, Reputation UI, Privacy/Rahasia, Vault, Siaga, Galang, Siarkan, Rutin, ESCO, Dispute, Onboarding
- **Supersedes:** UI-UX-SPEC-v0.5.md and all files in `ui-ux-spec/` (archived to `docs/design/archive/ui-ux-spec-v0.5/`)
- **Companion docs (still active):** DESIGN-DNA-v0.1.md (tokens, typography, color), AI-SPEC-v0.2.md (AI touch points), ADAPTIVE-PATH-SPEC-v0.1.md (data model)

---

## Table of Contents

1. [Design Pillars](#1-design-pillars)
2. [Core Interaction Model: Chat-First](#2-core-interaction-model-chat-first)
3. [Responsive Breakpoints](#3-responsive-breakpoints)
4. [Adaptive Path Model](#4-adaptive-path-model)
5. [Navigation & App Shell](#5-navigation--app-shell)
6. [Bagikan Entry Flow](#6-bagikan-entry-flow)
7. [Seed Card Anatomy](#7-seed-card-anatomy)
8. [LLM-UI Architecture](#8-llm-ui-architecture)
9. [Governance & Voting](#9-governance--voting)
10. [Roles & Permissions](#10-roles--permissions)
11. [Reputation UI Contract](#11-reputation-ui-contract)
12. [Privacy & Safety](#12-privacy--safety)
13. [Cross-Cutting Features](#13-cross-cutting-features)
14. [Special Spaces: Vault & Siaga](#14-special-spaces-vault--siaga)
15. [Onboarding & Membership](#15-onboarding--membership)
16. [ESCO Skill System](#16-esco-skill-system)
17. [Dispute & Stochastic Jury](#17-dispute--stochastic-jury)
18. [Transitions & Quorum](#18-transitions--quorum)
19. [AI Layer Cross-Reference](#19-ai-layer-cross-reference)
20. [Design Principles Summary](#20-design-principles-summary)
21. [Design Token Reference](#21-design-token-reference)

---

## 1. Design Pillars

### 1.1 Chat-First (NEW — replaces "Feed-First" as primary interaction)

The primary interaction surface is **conversation**. Users talk to the community (and the AI) through chat. Structured data (phases, checkpoints, progress) lives in an ambient layer above — visible on demand, never blocking the conversation.

> The chat is the floor. The structured reality is the ceiling — pull it down when you need to check the plan.

### 1.2 Ambient Structure

Every witness/case has a **phase breadcrumb** — a horizontal timeline of dots above the chat. Each dot represents a phase. Tapping a dot or pulling down reveals the structured content for that phase. The structure is always accessible but never dominates.

### 1.3 Witness Narrative

Every witness is a story with chapters. The engagement loop is narrative progression: "did my report lead to a real solution?" Users come back because they're invested in outcomes, not chasing likes.

### 1.4 Subtle Signals

Reputation is ambient. Trust shows through visual cues — tier badges, avatar warmth, weight in governance — not through leaderboards or point counters. Trust is the texture of the interface.

### 1.5 Smart Urgency

Notifications respect attention by default (daily digest) but escalate to real-time when it matters (challenge windows closing, tasks stalling). The system earns the right to interrupt.

### 1.6 AI is Furniture

AI is ambient, without special branding. Uses the Tanah semantic palette, not separate AI colors. Integrated seamlessly into workflows. Suggests, never overwrites.

---

## 2. Core Interaction Model: Chat-First

This section defines the **new** interaction paradigm that replaces the v0.5 Dual-Tab Pattern.

### 2.1 The Two Layers

Every witness/case has exactly two data layers:

| Layer | Content | Location |
|-------|---------|----------|
| **Conversation** | Chat messages, AI suggestions, diff cards, inline media | **Main surface** — always visible |
| **Structured** | Phase cards, checkpoint lists, progress, plan JSON | **Drawable panel** — on-demand |

**Key difference from v0.5:** In the old Dual-Tab model, both layers had equal weight as side-by-side tabs. In v1.0, **chat owns the screen**. Structured data is ambient — accessible via breadcrumb + drawable, never competing for attention.

### 2.2 Screen Anatomy (Mobile)

```
┌─────────────────────────────┐
│  ← [Title]          [⋮]    │  App bar
├─────────────────────────────┤
│  ●───●───●───◦              │  Phase breadcrumb
│──────────────────────────── │  Divider (pull handle)
│                             │
│  ┌──────────────────────┐   │
│  │ AI: Saya usulkan     │   │  Chat bubbles
│  │ 3 fase untuk...      │   │  (WhatsApp-style)
│  └──────────────────────┘   │
│         ┌───────────────┐   │
│         │ Setuju, lanjut │   │
│         └───────────────┘   │
│                             │
│  ┌──────────────────────┐   │  Inline diff cards
│  │ 📋 Diff: +2 item     │   │  (LLM suggestions)
│  │ [Terapkan] [Abaikan] │   │
│  └──────────────────────┘   │
│                             │
│  [________________] [Send]  │  Input bar
└─────────────────────────────┘
```

### 2.3 Phase Breadcrumb

The breadcrumb is a horizontal timeline fixed above the chat:

```
●───●───●───◦
```

| Symbol | Meaning | Visual |
|--------|---------|--------|
| `●` (filled) | Completed or active phase | Track-colored fill |
| `◦` (hollow) | Planned/upcoming phase | Gray outline |
| `───` (line) | Connection between phases | Gray line, track-colored between completed |
| Current dot | The active phase | Slightly larger, subtle pulse |

**Interactions:**
- **Tap a dot** → expands the drawable panel showing that phase's structured content (card with checkpoints, status, objective)
- **Pull down from breadcrumb area** → expands the full drawable panel
- **Swipe breadcrumb horizontally** → scroll if more phases than screen width

### 2.4 Drawable Phase Panel

The drawable panel slides down from below the breadcrumb, overlaying the chat. It contains the structured content for the selected phase.

**Collapsed state (default):** Only the breadcrumb `●───●───◦` is visible.

**Partially expanded:** Shows the selected phase card:

```
┌─────────────────────────────┐
│  ●───●───●───◦              │
├─────────────────────────────┤
│  ┌───────────────────────┐  │
│  │ Phase: Stabilisasi    │  │  Selected phase card
│  │ Objective: Pastikan   │  │
│  │ warga aman.           │  │
│  │                       │  │
│  │ ☑ Kumpulkan laporan   │  │  Checkpoints
│  │ ☐ Tetapkan PIC        │  │
│  │ ☐ Evaluasi dampak     │  │
│  │                       │  │
│  │ Source: 🤖 ai         │  │
│  │ Status: active        │  │
│  └───────────────────────┘  │
│  ┈┈┈┈ drag to close ┈┈┈┈┈  │  Pull handle
├─────────────────────────────┤
│  💬 Chat continues below... │
└─────────────────────────────┘
```

**Fully expanded:** Shows the complete timeline with all phases and their cards, scrollable vertically. This is the "Tahapan" view from v0.5, now accessible by pulling down fully.

**Dismiss:** Pull up, tap outside, or tap the breadcrumb dot again.

### 2.5 Chat Surface

The chat surface follows WhatsApp-style conventions:

| Element | Style |
|---------|-------|
| Other's bubble | Left-aligned, white background (`#FFFFFF`) |
| Self bubble | Right-aligned, track-soft background (e.g., `--t-tuntaskan-soft`) |
| AI inline card | Full-width card within chat flow, subtle AI badge |
| Diff card | Full-width card with action buttons `[Terapkan] [Abaikan]` |
| System message | Centered, muted text, no bubble |
| Timestamp | Centered, muted, between message groups |

**AI inline cards** (suggestions, summaries, alerts) appear as special cards within the chat flow — not in a separate tab. They use the same 7 block primitives defined in [Section 8](#8-llm-ui-architecture).

### 2.6 What Goes Where

| Content Type | Where it Appears | Reasoning |
|--------------|-----------------|-----------|
| Discussion messages | Chat | Conversation is chat |
| AI suggestions (diff cards) | Chat (inline) | Suggestions are part of the conversation |
| Phase/checkpoint summaries | Drawable panel | Structured data, on-demand |
| Checkpoint status changes | Both: system message in chat + updated in panel | Acknowledge in conversation, reflect in structure |
| Voting | Chat (inline vote card) | Voting is a conversational action |
| Vote results | Both: system message + panel update | Results affect structure |
| Evidence submission | Chat (inline evidence card) | Evidence is shared in conversation |
| Galang transactions | Chat (system message) + Galang sub-panel | Financial transparency in conversation |
| Task board (Papan GR) | Drawable panel (execution phase) | Structured task view |
| Thread summary (AI-07) | Chat (inline summary card) | Summaries are conversational |

---

## 3. Responsive Breakpoints

### 3.1 Mobile (< 768px) — Drawable Pattern

The canonical mobile experience as described in Section 2. Chat is full-screen. Phase breadcrumb at top. Pull down for structured content.

```
┌──────────────────┐
│ ●──●──●──◦       │  Breadcrumb + pull handle
│────────────────── │
│ 💬 Chat (full)   │  Chat owns the screen
│                   │
│ [___________] [>] │  Input bar
└──────────────────┘
```

### 3.2 Tablet (768–1024px) — Hybrid

Same layout as mobile but with more generous spacing. Drawable panel is taller when expanded, showing more phase detail. Chat bubbles are wider.

### 3.3 Desktop (> 1024px) — Side Panel

On desktop, the drawable becomes a **persistent sidebar**. Both conversation and structure are visible simultaneously.

```
┌──────────────────────────────────────────────────────┐
│  ← Penanganan Banjir RT 05                           │
├────────────────────────┬─────────────────────────────┤
│  TIMELINE SIDEBAR      │  CONVERSATION (main)        │
│                        │                             │
│  ●                     │  💬 Chat messages            │
│  │ Stabilisasi  ✓      │                             │
│  ●                     │  AI cards, diff cards,      │
│  │ Koordinasi   ●      │  inline votes, evidence     │
│  ●                     │                             │
│  │ Pelaksanaan  ◦      │                             │
│  ◦                     │                             │
│                        │                             │
│  [Selected phase card] │                             │
│  ┌──────────────────┐  │                             │
│  │ Koordinasi       │  │                             │
│  │ ☑ Checkpoint 1   │  │                             │
│  │ ☐ Checkpoint 2   │  │  [________________] [Send]  │
│  └──────────────────┘  │                             │
├────────────────────────┴─────────────────────────────┤
```

**Key adaptations for desktop:**
- Breadcrumb rotates to **vertical timeline** in the sidebar (same dots, 90-degree rotation)
- Clicking a phase node in sidebar shows its card below the timeline
- Sidebar is resizable (drag handle)
- Sidebar can be collapsed to icon-only rail (breadcrumb dots without labels)
- Chat panel takes remaining width

### 3.4 Breakpoint Rules

| Breakpoint | Phase View | Chat | Interaction |
|------------|-----------|------|-------------|
| Mobile (< 768px) | Horizontal breadcrumb + drawable | Full screen | Pull down to see phase card |
| Tablet (768–1024px) | Same as mobile, larger drawable | Full screen | Same pull-down, more detail |
| Desktop (> 1024px) | Vertical sidebar (always visible) | Right panel | Click phase in sidebar |

---

## 4. Adaptive Path Model

> Full data model in `ADAPTIVE-PATH-SPEC-v0.1.md`. This section covers UI rendering rules.

### 4.1 Core Hierarchy

`PathPlan → Branch → Phase → Checkpoint`

- **Phase**: a high-level step with an objective and status. Rendered as a dot in the breadcrumb.
- **Checkpoint**: a verifiable unit of progress within a phase. Rendered as items within the phase card in the drawable panel.
- **Branch**: an alternate path forked from a parent checkpoint. Rendered as a fork indicator in the breadcrumb/timeline.
- **Source tags**: `ai` (LLM-proposed), `human` (manually edited), `system` (automated).
- **Locked fields**: manually edited by a privileged role → locked from LLM.

Statuses: `planned`, `active`, `open`, `completed`, `blocked`, `skipped`.

### 4.2 Track Hints (Optional Metadata)

Track hints are color labels, not lifecycle drivers:

| Track Hint | Color | Spirit |
|------------|-------|--------|
| tuntaskan | `--t-tuntaskan` (#C05621) | Fix a problem |
| wujudkan | `--t-wujudkan` (#2E7D32) | Build something new |
| telusuri | `--t-telusuri` (#6A1B9A) | Investigate |
| rayakan | `--t-rayakan` (#F57F17) | Celebrate |
| musyawarah | `--t-musyawarah` (#4E342E) | Decide together |

Track hint determines: breadcrumb dot fill color, chat bubble tint (`--t-{track}-soft`), card border strip, section accents.

### 4.3 Common Phase Patterns

The LLM draws on these heuristic patterns but is not constrained by them:

**Problem-Solving:** Stabilisasi → Perencanaan → Pelaksanaan → Verifikasi
**Creation:** Pematangan Ide → Persiapan → Pembangunan → Perayaan
**Investigation:** Perumusan → Pengujian → Penemuan
**Celebration:** Validasi → Apresiasi → Dampak
**Governance:** Pembahasan → Keputusan → Pelaksanaan → Tinjauan

### 4.4 Reusable UI Components (Available in Any Phase)

| Component | Use in | Rendered in |
|-----------|--------|-------------|
| Papan Gotong Royong (task board) | Execution phases | Drawable panel |
| Galang sub-lifecycle | Resource-pooling phases | Chat (transactions) + Drawable (ledger) |
| Hypothesis cards + evidence board | Investigation phases | Drawable panel |
| Validation panel + appreciation wall | Celebration phases | Drawable panel |
| Position board + vote panel | Governance phases | Chat (vote cards) + Drawable (tally) |
| Ketetapan (formal decision) | Governance output | Drawable panel (document block) |

### 4.5 Privileged Editing

Only `project_manager` or `highest_profile_user` can edit phases and checkpoints. Manual edits lock affected fields and increment plan version. LLM proposes changes as diff cards in the chat; users accept or reject.

### 4.6 Branching

Branches appear as forks in the breadcrumb/timeline:

```
●───●───●───◦        (main branch)
        └───◦───◦    (branch: "Jika air naik lagi")
```

Each branch has a label and anchors to a parent checkpoint.

---

## 5. Navigation & App Shell

### 5.1 Bottom Navigation (5 Tabs)

| Tab | Icon | Label | Function |
|-----|------|-------|----------|
| 1 | 🏠 | Beranda | Community feed: seed cards, Community Pulse, horizontal track filter |
| 2 | 📝 | Catatan | Catatan Komunitas: lightweight public notes (prices, status, schedules) |
| 3 | 🤝 | Bantu | Skill-matched opportunities via ESCO |
| 4 | 🔔 | Notifikasi | Time-grouped: Hari Ini / Kemarin / Minggu Ini |
| 5 | ☰ | Lainnya | Hamburger: CV Hidup (Profil), Terlibat, Template Saya, Pengaturan |

### 5.2 App Header

```
[scope ▼]    Gotong Royong    [🔍] [+]
```

- **Scope selector** (left): current area, e.g., "RT 05 ▼" → bottom sheet picker
- **Search** 🔍 (right): full-screen overlay with filters (track, ESCO skill, time range)
- **Compose** [+] (right): opens Bagikan (AI-00 triage)

### 5.3 Scope Hierarchy (7 Levels)

| Level | Name | Example | Approx Size |
|-------|------|---------|-------------|
| 7 | Nasional | Indonesia | 275 million |
| 6 | Provinsi | Jawa Barat | ~50 million |
| 5 | Kota/Kabupaten | Kota Depok | ~2 million |
| 4 | Kecamatan | Cimanggis | ~200 thousand |
| 3 | Kelurahan/Desa | Tugu | ~15 thousand |
| 2 | RW | RW 03 | ~1,000 |
| 1 | RT | RT 05 | ~150 |

### 5.4 Community Pulse Bar

In Beranda header: `☀️ Cerah · 14 aktif · 3 baru · 1 vote`. GDF Weather emoji + live stats.

### 5.5 Feed Priority (5 Levels)

| Priority | Condition |
|----------|-----------|
| 1 — Your Action | Seed needs your action (PIC, vote open) |
| 2 — Nearing | Deadline/milestone close |
| 3 — New | Created within 24h |
| 4 — Active | Recent activity |
| 5 — Completed | Finished plans |

### 5.6 Horizontal Track Filter Tabs

Below Community Pulse: Semua (default) + 5 track-colored tabs. Swipeable.

---

## 6. Bagikan Entry Flow

### 6.1 Trigger

FAB [+] → opens Bagikan full-screen conversational interface.

### 6.2 AI-00 Conversational Triage

No empty textarea. AI-00 greets: "Ceritakan apa yang kamu lihat atau alami..." User tells their story. AI probes if needed. Context bar above keyboard morphs through states.

#### Context Bar States (8)

| State | Visual | Meaning |
|-------|--------|---------|
| Listening | Empty bar, wave indicator | AI listening |
| Probing | Bar + signal bars | AI asking follow-up |
| Leaning | Tappable track pill | AI has initial guess |
| Ready | Full card: track + confidence + seed type | Path plan proposed |
| Vault-ready | Dark card (vault palette) | Story directed to Catatan Saksi |
| Siaga-ready | Red pulsing card | Emergency detected |
| Split-ready | Split card | Story can split to 2 flows |
| Manual | Grid: 5 track hints + vault | User tapped "Pilih sendiri" |

#### Flow

1. User taps [+] → AI greets
2. User describes situation (text, voice, or mixed)
3. AI may probe: urgency, privacy, scale, evidence
4. Context bar morphs through states as confidence builds
5. When ready → context bar shows path plan summary
6. User confirms or taps "Ubah"
7. **Conversation text becomes the first messages in the witness chat** (context carries over). Phase breadcrumb appears with the proposed path.

### 6.3 Three Entry Routes

| Route | Trigger | Destination |
|-------|---------|-------------|
| Komunitas | Context bar `ready` | Witness chat-first screen (this guideline's core model) |
| Catatan Saksi | Context bar `vault-ready` | Vault dark space (Section 14.1) |
| Siaga | Context bar `siaga-ready` | Siaga emergency space (Section 14.2) |

### 6.4 Attachments & Settings

During conversation: photos/videos (max 5), auto-location (adjustable), Rahasia toggle (L0-L3), Rutin toggle.

### 6.5 Preview & Submit

Path plan summary → user confirms → plan created. ESCO skills auto-tagged. Duplicate detection (AI-03): pill "⚠ Saksi serupa (87%)" with comparison card. Redacted preview for Rahasia.

### 6.6 Edit/Delete Rules

| Action | Condition | Mechanism |
|--------|-----------|-----------|
| Edit | Within 15 min OR before first co-witness | Direct edit |
| Edit (after) | PIC flags factual error | Author 24h edit prompt |
| Delete | No co-witnesses yet | Direct delete |
| Delete (after co-witnesses) | Community consent | 24h consent window |
| Track hint change (before first checkpoint) | Author changes freely | Direct change |
| Track hint change (after first checkpoint) | Governed | Privileged editor or proposal |

---

## 7. Seed Card Anatomy

Seed cards appear in the **Beranda feed**. Tapping a seed card opens the **chat-first witness screen** (Section 2).

### 7.1 Card Structure (6 Sections)

| Section | Content |
|---------|---------|
| **1. Track Strip** | Left border 4px, track color. Never changes. |
| **2. Header** | Seed badge + Rahasia badge + title (max 2 lines) + author row (avatar-sm, name, tier pip, timestamp) |
| **3. Body** | Text 3 lines (clamp) + "...selengkapnya" + thumbnail media + location tag + PIC row + Dampak row |
| **4. Skill Tags** | ESCO-ID pills: Tervalidasi ● (filled) vs Dinyatakan ○ (outlined) |
| **5. Phase Breadcrumb** | Horizontal dots: `●───●───◦` — current dot track-colored, completed muted, planned hollow |
| **6. Footer** | 💬 comments · 👥 supporters · ⏱ time-in-phase · 🤖 AI badge (right-aligned) |

**Section 5 (Phase Breadcrumb)** is a miniature version of the breadcrumb from the chat-first screen. It gives a quick progress overview without entering the witness.

### 7.2 AI Badge Variants (Footer)

| Badge | Color | Meaning |
|-------|-------|---------|
| 🤖 Classified | Green | "🤖 Tuntaskan · 92%" |
| 🤖 Suggested | Orange | "🤖 Wujudkan? · 74%" |
| ⚠ Stalled | Red | "⚠ Macet 48j" |
| 🌱 Dampak | Green | "🌱 Dampak" |
| 📝 Ringkasan | Blue | "📝 Ringkasan" |
| ⚠ Duplikat | Orange | "⚠ Duplikat" |

---

## 8. LLM-UI Architecture

> Unchanged from v0.5. Defines how AI content renders within the chat and drawable panel.

### 8.1 Core Invariants

1. **Two data layers**: conversation (chat) + structured (drawable panel).
2. **AI never auto-applies**: always suggests via diff card. Human decides.
3. **Human edit = lock**: source flips `"ai"` → `"human"`, AI stops touching.
4. **Additive-first**: AI adds, suggests, drafts. Never deletes or overwrites human content.

### 8.2 Seven Block Primitives

| Block | Renders As | AI Rule | Source Tag |
|-------|-----------|---------|------------|
| `list` | Checklist, table, timeline, gallery | Additive. Nestable. Status-changeable. | Per-item |
| `document` | Rich text + tracked changes | AI drafts, human edits sections | Per-section |
| `form` | Labeled input fields | AI suggests per field. Protected = hands-off. | Per-field |
| `computed` | Read-only (progress bar, status) | System-derived. Nobody edits. | `system` |
| `display` | Presentation card (recognition) | One-way render. No edit. | `system` |
| `vote` | Voting interface + tally | System tallies. Not AI. | `system` |
| `reference` | Preview of linked card | Links to other cards. | `reference` |

**Where blocks render:**
- In **chat**: as inline cards (diff cards, suggestion cards, vote cards)
- In **drawable panel**: as structured phase content (checkpoint lists, task boards, ledgers)

### 8.3 Source Tags

| Tag | Meaning | Rule |
|-----|---------|------|
| `"ai"` | LLM-generated | Can be overwritten by next pass or human edit |
| `"human"` | Human-created/edited | AI stops touching. Locked. |
| `"system"` | System-computed | Nobody edits |

### 8.4 Four Trigger Modes

| Mode | When | Output Location |
|------|------|-----------------|
| Manual | User taps 🔄 Perbarui | Diff card in chat |
| Milestone | Keyword/pattern at breakpoints | Stage transition suggestion in chat |
| Time-Triggered | Scheduled interval | Alert in chat |
| Passive | Continuous monitoring | Badge/indicator only |

### 8.5 Diff Card Anatomy

For **list**: "Ditambah 2 item, dicentang 1" + evidence quotes.
For **document**: tracked-changes style.
For **form**: per-field comparison.

Actions: **[Terapkan Semua] | [Tinjau Satu-satu] | [Abaikan]**

Protected fields (financial, identity): 🔒 DILINDUNGI badge, excluded from AI access.

---

## 9. Governance & Voting

### 9.1 Vote Types

| Type | Rule | Quorum | Duration | Threshold |
|------|------|--------|----------|-----------|
| Standard | 1 person = 1 vote | 30% verified (max 50) | 48h | >50% wins |
| Weighted | I^1.5 weight | 40% weighted | 72h | >60% weighted wins |
| 1.5x Quorum | Governed proposals | 1.5x quorum | 72h + 72h challenge | Challenge → Jury |
| Consensus | Consent window | Auto-advance if no objection | Configurable | No objection = pass |

### 9.2 Vote UI in Chat-First Model

Votes appear as **inline vote cards** in the chat. The vote card uses the `vote` block primitive:

```
┌──────────────────────────────┐
│  📊 Vote: Lanjut ke fase     │
│  Pelaksanaan?                │
│                              │
│  [Setuju]  [Tolak]  [Abstain]│
│                              │
│  ⏱ 47h tersisa · 12/40 voted │
│  Standard · Quorum: 30%     │
└──────────────────────────────┘
```

Vote results update both the chat card and the phase status in the drawable panel.

### 9.3 Evidence: The Triad

- Testimony (I): Direct witness account
- Corroboration (C): Co-witness support
- Document (D): Photo, video, receipt

EQ = (I + 0.5×C + 0.3×D) / 2. Max 5 media items.

### 9.4 Member Progression

| Tier | Indonesian | Criteria | Permissions |
|------|-----------|----------|-------------|
| Newcomer | Pengguna Baru | Just joined | Browse, follow |
| New Member | Anggota Baru | Profile + 0-1 contrib | Witness, comment, no vote |
| Verified | Terverifikasi | Profile + 1+ contrib + I ≥ 0.2 + 90d | Full permissions |
| Pillar | Pilar | I ≥ 0.6, J ≥ 0.5, history | Leadership nomination |
| Key | Kunci | I ≥ 0.8, J ≥ 0.7, C_eff ≥ 0.5 | Highest trust |

---

## 10. Roles & Permissions

### 10.1 Role Definitions

| Role | Indonesian | Assignment | Requirements |
|------|-----------|------------|--------------|
| Author | Penulis | Automatic | Any user |
| Co-witness | Saksi | Self-assign | Any user |
| Participant | Peserta | Self-assign | Any user |
| PIC | Penanggung Jawab | Discussion phase → vote | Terverifikasi |
| Treasurer | Bendahara | Galang → vote | I ≥ 0.5, clean J, ≠ PIC |
| Communications | Humas | Siarkan → vote | Terverifikasi |
| Verifier | Verifikator | Auto-selected | I ≥ 0.4, not involved, min 3 |
| Reviewer | Peninjau | Auto-selected | I ≥ 0.4, impartial |
| Jury | Juri | Stochastic | I ≥ 0.4, verified, not involved |

### 10.2 Permission Matrix

| Action | Penulis | Saksi | Peserta | PIC | Bendahara | Humas | Verifikator |
|--------|---------|-------|---------|-----|-----------|-------|-------------|
| Edit seed (15min) | ✓ | | | | | | |
| Co-witness | | ✓ | | | | | |
| Comment/discuss | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Propose transition | ✓ | | | ✓ | | | |
| Vote on transition | ✓ | ✓ | ✓ | ✓ | | | |
| Assign tasks | | | | ✓ | | | |
| Claim tasks | | | ✓ | ✓ | | | |
| Submit heartbeat | | | ✓ | ✓ | | | |
| Manage funds | | | | | ✓ | | |
| Approve disbursement | | | | ✓ | ✓ | | |
| Broadcast/share | | | | | | ✓ | |
| Verify completion | | | | | | | ✓ |
| Challenge result | ✓ | ✓ | ✓ | ✓ | | | |
| Set Rahasia level | ✓ | | | ✓ | | | |
| File dispute | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## 11. Reputation UI Contract

### 11.1 Tier Badge System (◆◇)

| Tier | Badge | Color | Hex |
|------|-------|-------|-----|
| 4 · Kunci | ◆◆◆◆ | Gold | #FFD700 |
| 3 · Pilar | ◆◆◆◇ | Blue | #1E88E5 |
| 2 · Kontributor | ◆◆◇◇ | Teal | #00695C |
| 1 · Pemula | ◆◇◇◇ | Brown | #795548 |
| 0 · Bayangan | ◇◇◇◇ | Red/Gray | var(--c-bahaya) |

**Full display:** `◆◆◆◇ Pilar` — profile headers, detail cards.
**Compact pip:** 18px colored circle with tier number — avatar overlays, inline bylines.

### 11.2 I/C/J Score Axes

| Axis | Name | Color | Hex |
|------|------|-------|-----|
| I | Inisiatif | Amber | #F57F17 |
| C | Kompetensi | Teal | #00695C |
| J | Penilaian | Purple | #7B1FA2 |

Radar SVG on Profil page. Self-view: numeric values. Others: ring + tier only.

### 11.3 GDF Weather Widget

| Weather | Emoji | Multiplier | Meaning |
|---------|-------|------------|---------|
| Cerah | ☀️ | 1.0x | Active community |
| Berawan | 🌤️ | 1.2x | Moderate activity |
| Hujan | 🌧️ | 1.5x | Low activity |
| Badai | ⛈️ | 2.0x | Crisis |

### 11.4 CV Hidup (Living CV)

Hero section: avatar-xl + tier badge + name + community. Score cards with decay info. Dual-layer skills (Tervalidasi/Dinyatakan). Kontribusi timeline. Vouch section. Impact metrics. QR code.

### 11.5 Avatar Warmth

Last 7 days: 100% opacity. Last 30 days: 90%. >30 days: 70%.

---

## 12. Privacy & Safety

### 12.1 Rahasia Levels (4-Level Overlay)

| Level | Name | Author | Content | Media | Reversibility |
|-------|------|--------|---------|-------|---------------|
| L0 | Terbuka | Full name | Public | Full | — |
| L1 | Terbatas | Full name | Verified only | Full | Reversible |
| L2 | Rahasia | Anonymous (gray) | Request-access gate | Blur | Down: IRREVERSIBLE |
| L3 | Sangat Rahasia | Hidden | Redacted hatched | Hatched | IRREVERSIBLE identity |

### 12.2 AI Review States

**Dalam peninjauan:** AI-02 confidence < 0.70. Yellow badge. Excluded from search. L2+ hidden. 24h SLA.
**Menunggu moderasi:** AI-04 unavailable AND author tier ≤ 1. Not published. 1h async review.

### 12.3 Reporting

Categories: Hoax, Harassment, Danger, Spam, Other. 3 reports = flag. 5 reports = hide. Laporkan → Kontak Darurat (48h), escalate to Jury at 7d.

### 12.4 Defamation Safeguards

Evidence + corroboration before visibility. Right to response (48h). False accusation penalties. 72h mandatory delay on L2.

### 12.5 Thread Summary

>10 messages → Ringkasan button in chat. Collapsed card: "Ringkasan • [N] peserta • [sentiment]". Weekly digest: Monday 07:00. Rahasia excluded.

---

## 13. Cross-Cutting Features

### 13.1 Galang (Pool Resources)

Mini-lifecycle: Sasaran → Kumpul → Salurkan → Lapor. Bendahara manages.

**In chat-first model:** Galang activates as a checkpoint or phase within the adaptive path. Transactions appear as system messages in chat. The full ledger (date, contributor, amount, method, status) is viewable in the drawable panel.

6 financial fields always protected (🔒). Phase: Manual tracking → E-wallet → In-app wallet.

### 13.2 Siarkan (Broadcast / Amplify)

Reach tracker, share actions, media kit. Humas role manages. Activatable in any phase.

### 13.3 Rutin (Recurring)

**Template-Based Recurrence:** Completed plans can be saved as templates. Templates preserve phase structure, PIC strategy, schedule, Rahasia level. Case-specific data is cleared.

**Spawn:** Manual ("Buat dari template") or scheduled (auto-create 48h before date). LLM re-evaluates template against current context and suggests modifications in chat.

**Schedule options:** Harian, Mingguan, Bulanan, Custom (cron-like).

**PIC Rotation:** Fixed, round-robin, volunteer-based, hybrid. Fairness tracking across cycles. 3 consecutive skips → relevance prompt.

---

## 14. Special Spaces: Vault & Siaga

These are **separate UI spaces** that exit the normal chat-first model. They have their own palettes and lifecycles.

### 14.1 Catatan Saksi (Vault)

**Entry:** AI-00 vault-ready, or dedicated vault access from Profil.

**Palette:** Vault colors exclusively: `--v-deep` (#263238) through `--v-wash` (#ECEFF1). No bottom navigation — vault is a separate space.

**Header:** Stats bar (catatan / wali / diterbitkan count). 4 filter tabs: Semua / Tersegel / Dengan Wali / Diterbitkan.

**Five States:**

| State | UI | Seal Status |
|-------|------|-------------|
| Menyimpan | Compose: text from AI-00, attachments | Unsealed |
| Tersegel | SHA-256 hash, timestamp, encrypted badge | Sealed 🔒 |
| Wali | Trustee search, permission list, tier badge | Sealed 🔒 |
| Terbitkan | Orange warning, 3 consequences, track picker | Sealed → Published |
| Pola | AI pattern detection, gentle alert, resource links | Sealed 🔒 |

**Seal bar:** Bottom bar morphing: Unsealed (edit) → Sealed (locked + actions: Ganti Wali / Terbitkan).

**Wali permissions:** ✓ read, ✓ surface-with-consent, ✕ edit, ✕ share.

**Publish (Terbitkan):** Warning with 3 consequences. Track picker. L2 toggle. Split visual: dark vault → warm Tanah card preview. **IRREVERSIBLE.**

**Pattern Detection:** AI detects patterns across entries. Gentle alert with crisis resources: Telepon Sahabat 119 ext 8, Komnas Perempuan, LPSK, LBH. Dismissible.

**Reputation:** ZERO while sealed. Credit only if published.

### 14.2 Siaga Broadcast (Emergency)

**Entry:** AI-00 siaga-ready.

**Palette:** Siaga colors: `--s-deep` (#B71C1C) through `--s-border` (#FFCDD2). Red pulse animations.

**Four States:**

| State | UI | Real-time |
|-------|------|-----------|
| Kirim | Compose: text, emergency type chips, auto-location, 112 link | Siarkan Sekarang button |
| Aktif | Live card with shimmer, real-time stats, quick update input, timeline | terjangkau/melihat/merespons counters |
| Respons | Responder cards (distance/ETA/status), quick-respond button | Real-time updates |
| Selesai | Confirmation, summary (duration/responders/services), thank you | Green resolved bar |

**Emergency chips:** Kebakaran, Banjir, Gempa, Darurat Medis, Kecelakaan, Keamanan, Lainnya.

**Broadcast bar:** Bottom bar morphing: Composing → Active (pulsing) → Resolved (green). 112 always visible.

**Reputation:** ZERO. Emergency response is civic duty.

---

## 15. Onboarding & Membership

### 15.1 Registration Flow

| Screen | Content | Action |
|--------|---------|--------|
| 1 | "Gotong Royong — Komunitas digital untuk aksi nyata" | Mulai |
| 2 | Phone OTP verification | Verifikasi |
| 3 | Nama + Foto (optional) | Lanjut |
| 4 | Community GPS suggestion + search + invite code | Bergabung |
| 5 | "Selamat datang!" | Ke Beranda |

### 15.2 Community Types

| Type | Size | Example |
|------|------|---------|
| RT | 30-100 families | RT 05/RW 03 Kel. Manggarai |
| RW | 200-1000 families | RW 03 Kel. Manggarai |
| Kelurahan | Administrative unit | Kel. Manggarai |
| Custom | Varies | Pengurus Masjid, Komite Sekolah |

### 15.3 Authority Registration

RT/RW head: self-declare + verification by 3+ verified members. Annual reverification. Safety contacts: nominated → community vote. Genesis nodes: first 3-5 members, temporary elevated trust until 10 verified.

---

## 16. ESCO Skill System

### 16.1 Skill Extraction

ESCO skills auto-tagged during AI-00 triage via Tandang POST /extract-skills. Returns skill URIs with confidence scores.

### 16.2 Display

Two visual states: **Tervalidasi ●** (filled, confirmed by Tandang) and **Dinyatakan ○** (outlined, self-declared).

Shown on: seed card skill tags (Section 7), CV Hidup profil, Bantu tab matching, search results.

### 16.3 Bantu Tab Matching

Surfaces opportunities matching user's ESCO skills. Validated skills weighted higher. Volunteer counts per seed.

---

## 17. Dispute & Stochastic Jury

### 17.1 Triggers

Disputed verification, disputed transition, misuse allegation, defamation (Laporkan escalation), challenged vote.

### 17.2 Jury Selection

Pool: verified, I ≥ 0.4, not involved, active 30d. Size: 5 (<200 members) or 7 (200+). 24h accept/decline. Max 2 declines/year.

### 17.3 Process (Max ~8 Days)

| Phase | Duration | Activity |
|-------|----------|----------|
| Filed | Day 0 | Evidence submitted |
| Formation | 24h | Jury selection |
| Evidence | 72h | Review, clarification |
| Deliberation | 48h | Anonymous discussion |
| Verdict | Day 6 | Dikabulkan/Ditolak/Bukti Kurang |
| Appeal | 24h | New evidence only |
| If appeal | 5 days | New jury, compressed |

### 17.4 Outcomes

| Outcome | Entity Effect | Offender | Disputer |
|---------|--------------|----------|----------|
| Upheld | Action reversed | I penalty, possible Shadow | None |
| Rejected | No change | None | I -0.01 |
| Insufficient | No change, monitoring | None | None |
| Split (3-2) | Verdict stands, flagged | None | Mediation offered |

### 17.5 UI Elements

Keberatan button (⚠️), Laporkan button (🚩). Dispute form modal. Jury dashboard. Verdict card on entity.

---

## 18. Transitions & Quorum

| Transition Type | Risk | Mechanism | Duration |
|----------------|------|-----------|----------|
| Checkpoint open → completed (routine) | Low | Consent | 24h |
| Phase activation (simple) | Low | Consent | 24h |
| Phase activation (complex, ≥5 participants) | Medium | Vote | 48h |
| Galang-related checkpoint | Medium-High | Vote | 48h |
| Checkpoint requiring evidence | High | Vote + evidence review | 72h |
| Plan completion (final phase) | High | Vote + challenge window | 72h |
| Emergency fast-track | Critical | Fast-track + 7-day post-hoc audit | Immediate |

### Consent Window (Anti-Stall)

Low-risk transitions auto-advance after configured duration unless someone objects. Stop button: any participant can object → standard vote.

---

## 19. AI Layer Cross-Reference

| AI ID | Name | UI Element | Mode |
|-------|------|-----------|------|
| AI-00 | Conversational Triage | Bagikan screen, context bar | Conversational |
| AI-01 | Track & Seed Hint Classifier | Internal (optional metadata) | One-shot |
| AI-02 | Redaction LLM | Redacted preview | One-shot |
| AI-03 | Duplicate Detector | Context bar pill + comparison card | One-shot |
| AI-04 | Content Moderation | Invisible unless triggered | One-shot |
| AI-05 | Gaming Detection | PIC/Peninjau dashboard alerts | Async batch |
| AI-06 | Criteria Suggestion | Chat inline cards | Conversational |
| AI-07 | Discussion Summary | Thread summary card in chat | One-shot |
| AI-08 | Media Redaction | Redacted media | One-shot (CV) |
| AI-09 | Credit Accreditation | Toast, chat nudge, diff card | Mixed |

**Not AI:** ESCO extraction, difficulty estimation, PIC suggestion, jury selection, anti-collusion, PageRank, vouch graph, state machines, quorum, timers, role enforcement, OTP, ledger, notifications (all Tandang or backend).

---

## 20. Design Principles Summary

| Principle | Description |
|-----------|-------------|
| **AI is Furniture** | AI ambient, no special branding. Tanah palette. Seamless. |
| **Addiction to Contribution** | Engagement tied to ACTION, not content consumption. |
| **Zero Tandang (Vault/Siaga)** | No reputation credit while sealed/emergency. |
| **Suggest-Don't-Overwrite** | AI always diff card. Human decides. |
| **Source-Tagged Data** | Every structured item tagged: `ai` / `human` / `system`. |
| **Track Color = Identity** | Color constant throughout lifecycle. |
| **Speed Over Ceremony (Siaga)** | Minimal screens, one tap, auto-location. |
| **Context Carries Over** | AI-00 conversation becomes first chat messages. |
| **Single Entry Point** | All entry via Bagikan with AI-00 triage. |
| **Reputation is Area-Aware** | Tandang scores per scope level. |
| **GDF Weather = Difficulty Floor** | Harder conditions → bonus multiplier. |
| **No Separate Credit Screen** | Credit earned within activities. Visibility via toast + nudge. |
| **Strict Wali Permissions** | Read + surface-with-consent only. |

---

## 21. Design Token Reference

All visual tokens are defined in **DESIGN-DNA-v0.1.md** (still active, not archived). Key references:

- **Colors:** `docs/design/specs/design-dna/02-color-system.md`
- **Typography:** `docs/design/specs/design-dna/03-typography-and-spacing.md`
- **Components:** `docs/design/specs/design-dna/04-core-components.md`
- **Card System:** `docs/design/specs/design-dna/05-card-system.md` *(note: dual-tab references in this file are superseded by this guideline's chat-first model)*
- **Quick Token Reference:** `docs/design/specs/design-dna/10-token-reference.md`

---

*End of Document*

*Gotong Royong UI Guideline v1.0 · February 2026 · SINGLE SOURCE OF TRUTH*

*Companion documents: DESIGN-DNA-v0.1.md · AI-SPEC-v0.2.md · ADAPTIVE-PATH-SPEC-v0.1.md*

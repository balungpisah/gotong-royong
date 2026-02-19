# Engagement Strategy — Gotong Royong Pulse Feed

> Version 0.2 — **Octalysis × Tandang Fusion**
> Status: **DRAFT**
> Last updated: 2025-02-19
> Supersedes: [v0.1](./ENGAGEMENT-STRATEGY-v0.1.md)

---

## What Changed in v0.2

v0.1 was designed **before** we knew about tandang. It proposed generic mechanics (emoji reactions, basic streaks). v0.2 fuses the Octalysis motivational framework with tandang's 120+ reputation signals to create engagement where **every action has real consequences**.

| v0.1 (Feature-Driven) | v0.2 (Signal-Driven) |
|---|---|
| Generic emoji reactions (🔥💪👍🙏) | 5 tandang-backed chips (Vouch, Skeptis, Saksi, Bagus, Perlu Dicek) |
| "Saya Ikut" button | Removed — participation is proven by action, not declared |
| Basic streak counter | Full I/C/J dashboard + decay warnings + polymath progress |
| No trust visibility | Tier badges, vouch counts, skeptis counts, PoR badges |
| 5 generic phases | 5 signal-driven phases: INPUT → OUTPUT → PERSONAL → IMMERSION |

**Companion documents:**
- [TANDANG-SIGNAL-INVENTORY-v0.1.md](./TANDANG-SIGNAL-INVENTORY-v0.1.md) — Raw signal catalog (120+ events)
- [TANDANG-INTEGRATION-CONTRACT-v0.1.md](./TANDANG-INTEGRATION-CONTRACT-v0.1.md) — API/event contracts per phase
- [ENGAGEMENT-BACKEND-CONTRACT-v0.1.md](./ENGAGEMENT-BACKEND-CONTRACT-v0.1.md) — Feed field registry

---

## 1. Problem Statement

*(Unchanged from v0.1)*

Current social media trains users for short attention spans with zero-friction dopamine loops. Our masonry feed (Pulse) provides excellent **eagle-eye awareness** of community initiatives, but requires **effort to engage** — too many clicks to understand what's happening, no instant gratification.

**We are NOT building addictive-for-profit software.** We are making civic action *compete* with the attention economy. If TikTok can make people scroll for 3 hours watching cat videos, we can make them scroll for 30 minutes solving neighborhood problems.

### What We Want
- Eagle-eye overview (keep masonry) **+** zero-click dopamine (steal from TikTok/Instagram)
- Users **feel** community activity without clicking
- Quick actions that give instant satisfaction
- **NEW in v0.2:** Every action feeds into a real reputation system with real consequences
- Ethical hooks that serve the user's *actual* interest (civic participation)

---

## 2. Two-Lens Framework

### Lens 1: Octalysis — WHY People Engage

Eight Core Drives of human motivation (Yu-kai Chou):

| # | Drive | Category | Our Leverage |
|---|-------|----------|-------------|
| ① | **Epic Meaning & Calling** | White Hat | Gotong royong IS the calling — tier system shows your role |
| ② | **Development & Accomplishment** | White Hat | C score radar, polymath progress, verification badges |
| ③ | **Empowerment of Creativity & Feedback** | White Hat | Tandang chips, inline voting, solution submission |
| ④ | **Ownership & Possession** | Utility | Vouch budget, domain portfolio, J score, contribution trail |
| ⑤ | **Social Influence & Relatedness** | Utility | Vouch/skeptis chips, trust indicators, tier badges |
| ⑥ | **Scarcity & Impatience** | Black Hat | Vouch limits per tier (REAL), jury selection threshold |
| ⑦ | **Unpredictability & Curiosity** | Black Hat | Jury results, epoch outcomes, skeptical counters |
| ⑧ | **Loss & Avoidance** | Black Hat | Decay countdowns (REAL), slash consequences, streak reset |

### Lens 2: Tandang — WHAT We Can Measure

120+ signals across 17 categories from the tandang human-measurement engine. Key systems:

| Tandang System | What It Measures | Octalysis Drives Fed |
|---|---|---|
| **I/C/J Scores** | Integrity, Competence (per domain), Judgment | ②④⑤ |
| **Tier System** | Keystone/Pillar/Contributor/Novice/Shadow | ①⑥ |
| **Vouch System** | 5 vouch types with bleed-in, dampening, J stakes | ③⑤⑥ |
| **Decay System** | 30-day trigger, 90-day half-life on competence | ⑧ |
| **Slash System** | 5 triggers, cascade penalties, jury process | ⑦⑧ |
| **PoR System** | Proof of Reality — physical evidence attestation | ③⑤ |
| **Activity Signals** | Streaks, contribution count, quality average | ②④ |
| **Skill/Polymath** | Cross-domain expertise, 3+ domain mastery bonus | ①② |

### The Core Insight

> **Old approach**: Generic engagement mechanics (emoji reactions, streaks) with no backend weight.
> **New approach**: Every user action is a **tandang signal** with real consequences (J score stakes, C score impact, slash cascades).

This is the opposite of engagement farming — it's engagement that **costs something** (your reputation) and **builds something** (community trust graph).

---

## 3. The Hook Model (Revised)

```
TRIGGER  → Push notification: "Ramai di RT 05 — 12 orang baru bergabung"
    ↓
ACTION   → Open app, see pulsing card, tap tandang chip (LOW friction, HIGH meaning)
    ↓
REWARD   → Variable: your vouch starts bleed-in (25%→50%→100%),
           your skeptis proved right (+J), surprise jury verdict,
           someone you vouched for got verified
    ↓
INVEST   → Your I/C/J scores grow, tier progresses, vouch budget expands,
           decay timer resets → better triggers, more influence next time
    ↓
           ──── LOOP ────
```

**Key difference from v0.1**: The INVEST phase now feeds a real reputation system. Investment isn't vanity metrics — it's building your civic identity.

---

## 4. Signal Classification

### The Golden Rule

> **If it changes reputation (I/C/J), it MUST be an explicit user tap.**
> **If it only affects personalization/recommendations, implicit capture is fine.**

### 4a. Explicit Signals (5 chips + inline vote)

| Chip | Tandang Signal | Score Impact | Scarcity | Context |
|------|---------------|-------------|----------|---------|
| 🤝 **Vouch** | `vouch_created` (Positive) | J: ±0.02 to ±0.20 | Limited by tier (10–100 max) | All cards |
| 🔍 **Skeptis** | `vouch_created` (Skeptical) | J: inverted (rewarded if right) | Dampening: 10%/vouch, floor 70% | All cards |
| 🫣 **Saya Saksi** | `por_evidence_submitted` | J: CoWitness ±0.02/−0.10 | Must witness IRL | Problem cards |
| ✅ **Sudah Beres** | `solution_por_validated` | J: CoWitness ±0.02/−0.10 | Must verify IRL | Solution cards |
| 📷 **Bukti Valid** | `por_validation_passed` | J: ±0.02/−0.10 | — | Evidence cards |
| ⭐ **Bagus** | QualitySpectrum vote | Target C score affected | Min 3 verifiers | Contribution cards |
| ⚠️ **Perlu Dicek** | `problem_uncertainty_flagged` | May trigger verification/slash | — | All cards |
| 🗳️ **Inline Vote** | Governance vote | J: governance events | One vote per proposal | Voting cards |

### 4b. Contextual Chip Wording

The witness chip changes label based on card type — the user must know exactly what they're attesting:

| Card Type | Chip Label | Meaning | Tandang Signal |
|---|---|---|---|
| Problem card | 🫣 **"Saya Saksi"** | "I've seen this problem with my own eyes" | `por_evidence_submitted` (witness_attestation) |
| Solution card | ✅ **"Sudah Beres"** | "I confirm the solution was implemented" | `solution_por_validated` (completion_attestation) |
| Evidence card | 📷 **"Bukti Valid"** | "This photo/video is genuine" | `por_validation_passed` |

### 4c. Implicit Signals (captured silently)

| User Behavior | Tandang Operation | Impact |
|---|---|---|
| Posts in discussion | `process_contribution(continuous)` | C score (via contribution type) |
| Submits evidence | `submit_por_evidence` | C + J scores |
| Claims problem | `claim_problem` | Problem lifecycle — C at stake |
| Submits solution | `submit_solution` | C score (via solve_reward) |
| Acts as verifier | `record_verification_decision` | J score |
| Serves on jury | `cast_jury_vote` | J score |
| Opens card detail | `record_activity` | Resets decay timer only |
| Reads >30 seconds | Activity metric | Feed personalization only |
| Returns 3+ times | Interest pattern | Recommendation signal only |

### 4d. Why "Saya Ikut" Was Removed

The original Phase 2 had a 🙋 "Saya Ikut" (I join) button. Removed because:
1. **Hollow gesture** — tapping "I join" without doing anything is meaningless
2. **Cheapens real participation** — button-tappers sit next to evidence-submitters
3. **Tandang has no signal for intent** — reputation is built on action, not declaration
4. **Already captured implicitly** — the moment you post, submit evidence, or claim a problem, tandang records your REAL participation

The card still SHOWS participation ("5 warga terlibat" with avatar stack) — but that count comes from real tandang events, not a button.

### 4e. Captured But Never Scored

| Behavior | Use For | Never Use For |
|----------|---------|---------------|
| Card dwell time | Feed ranking | Reputation scoring |
| Scroll depth | Session quality metric | Anything |
| Topic browsing pattern | Entity suggestions | Competence scoring |
| Reaction speed | Bot detection | User scoring |

---

## 5. Feature Proposals (Revised)

### 5.1 — Pulse Ring: Live Activity Heartbeat ✅ IMPLEMENTED
> **Drives:** ⑤ Social Influence, ⑦ Unpredictability
> **Status:** ✅ Done (Phase 1)

Cards with recent activity pulse with a mood-tinted glow. Live "🟢 X aktif" badge. CSS `animate-pulse-glow` on sentiment shadow.

### 5.2 — Countdown Urgency: Real Deadlines ✅ IMPLEMENTED
> **Drives:** ⑥ Scarcity, ⑧ Loss & Avoidance
> **Status:** ✅ Done (Phase 1)

Time-sensitive countdown badges with color escalation. Quorum progress bars. All deadlines map to real system events.

### 5.3 — Story Peek: Zero-Click Dopamine ✅ IMPLEMENTED
> **Drives:** ⑦ Unpredictability, ⑤ Social Influence
> **Status:** ✅ Done (Phase 1 — originally Phase 3, pulled forward)

3-line continuous CSS scroll ticker showing live chat messages on cards. Hover to pause. `contain: strict` isolation from masonry ResizeObserver. Varied message lengths.

### 5.4 — Sinyal Aksi: Tandang-Backed Chips (NEW)
> **Drives:** ③ Empowerment, ⑤ Social Influence, ⑥ Scarcity, ⑦ Unpredictability
> **Effort:** Medium | **Impact:** Very High
> **Status:** 🔵 Spec'd — awaiting tandang SDK

**What:** 5 contextual action chips + inline vote widget, each backed by a real tandang signal.

**Replaces:** v0.1's "Quick Strike" with generic emoji reactions.

**Why it's better:** Every tap stakes your judgment score. Vouch limits create natural scarcity. Skeptical vouches create real tension. No vanity metrics.

**Chips:** See §4a above for full matrix.

**Implementation notes:**
- Chip bar on card footer, contextual to card type
- Optimistic UI — show "vouched" immediately, confirm with tandang async
- Vouch budget indicator: "12/25 remaining" next to vouch chip
- Skeptis chip shows concern count in amber
- Saya Saksi may open mini-form for optional location/media
- See [TANDANG-INTEGRATION-CONTRACT §2](./TANDANG-INTEGRATION-CONTRACT-v0.1.md#2-outbound-signals) for payloads

**Mockup concept:**
```
┌────────────────────────────────┐
│ BARU   ◆◆◇◇ Ahmad  🔥4        │ ← tier badge + streak
│                                │
│ Jalan berlubang di depan       │
│ SDN 03 sudah 2 minggu...      │
│                                │
│ 🤝 12 menjamin · 🔍 3 skeptis  │ ← trust surface
│                                │
│ ┌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┐ │
│ │💬 Sari: "Sudah foto..."    │ │ ← peek strip
│ │💬 Ahmad: "Saya hubungi..." │ │
│ └╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘ │
│                                │
│ [🤝Vouch] [🔍Skeptis] [🫣Saksi]│ ← tandang chips
│ [⭐Bagus] [⚠️Cek]              │
│                                │
│ [RK][SD] 👥15     ⏰ 2j lagi   │
│ TERLIBAT              👁 🔖    │
└────────────────────────────────┘
```

### 5.5 — Wajah Kepercayaan: Trust Surface (NEW)
> **Drives:** ① Epic Meaning, ② Accomplishment, ⑤ Social Influence, ⑦ Unpredictability
> **Effort:** Medium | **Impact:** High
> **Status:** 🔵 Spec'd — awaiting tandang SDK

**What:** Make tandang trust data VISIBLE on every card. This is the killer feature tandang enables that no other civic platform has.

**Elements:**

| UI Element | Signal Source | Placement |
|---|---|---|
| **Tier badge** (◆◆◆◇) | `get_user_tier` | Next to author name |
| **Author streak** 🔥 | `activity_streak_days` | Author avatar area |
| **Vouch count** "12 menjamin" | `get_vouch_summary` | Card header |
| **Skeptis count** "3 skeptis" | `get_vouch_summary` | Card header (amber) |
| **Verified ✓** badge | `get_verification_status` | Solution cards |
| **PoR 📷** badge | `get_por_status` | Evidence cards |
| **Shadow indicator** | `get_recovery_status` | Dimmed avatar |

**Implementation notes:**
- Tier badge: small diamond glyphs next to name, colored by tier
- Vouch/skeptis: compact counts in card header area
- All data comes from tandang query endpoints (see [Integration Contract §4](./TANDANG-INTEGRATION-CONTRACT-v0.1.md#4-query-endpoints))

### 5.6 — Jejak Saya: Personal Tandang Dashboard (NEW)
> **Drives:** ② Accomplishment, ④ Ownership, ⑥ Scarcity, ⑧ Loss & Avoidance
> **Effort:** High | **Impact:** High
> **Status:** 🔵 Spec'd — awaiting tandang SDK

**What:** Full personal dashboard showing your tandang reputation profile.

**Replaces:** v0.1's basic "Streak & Impact Counter" with the full tandang profile.

**Widgets:**

| Widget | Signal Source | Drive |
|--------|-------------|-------|
| 🔥 **Activity streak** | `activity_streak_days` | ②④ |
| 📊 **Competence radar** | C scores per ESCO domain | ②④ |
| 🤝 **Vouch budget** "12/25 used" | Tier max vouches | ④⑥ |
| ⚖️ **Judgment score** | J score (0–1) | ④ |
| ⏳ **Decay countdown** | `decay_warning_issued` | ⑧ |
| 📈 **Impact summary** | Contributions, verifications, jury service | ②④ |
| 🌟 **Polymath progress** | `skill_polymath_progress` (2/3 domains) | ② |
| 🏅 **Tier progression** | Percentile → next tier threshold | ②⑥ |

**Ethics note:** Decay warnings are shown transparently ("Your infrastruktur competence decays in 12 days — submit evidence or verify work to reset"). The mechanic is real (tandang's 30-day trigger, 90-day half-life), not manufactured.

### 5.7 — Aliran: River Mode (unchanged from v0.1)
> **Drives:** ③ Empowerment, ⑤ Social Influence, ⑦ Unpredictability
> **Effort:** High | **Impact:** Very High
> **Status:** [ ] Not started

Full-width vertical snap-scroll view. TikTok for civic issues. Reuses all Phase 2–4 tandang data in an immersive layout. See v0.1 §4.6 for full description.

---

## 6. Revised Roadmap

### Phase Ordering Principle

```
Phase 1: FEEL         → Cards come alive (no tandang needed)        ✅ DONE
Phase 2: INPUT        → User taps chips → signals flow INTO tandang
Phase 3: OUTPUT       → Tandang data flows OUT onto cards (trust)
Phase 4: PERSONAL     → Your OWN tandang profile (ownership)
Phase 5: IMMERSION    → New consumption mode amplifies everything
```

Each phase builds on the previous. Can't show trust badges (Phase 3) until people create vouch signals (Phase 2). Can't show personal dashboards (Phase 4) until there's trust data to display (Phase 3).

### Roadmap Table

| Phase | Name | Features | Tandang Dependency | Drives | Effort | Status |
|-------|------|----------|-------------------|--------|--------|--------|
| **1** | **Hidup** (Feel Alive) | Pulse glow, countdown, quorum, story peek | None (mock data) | ⑤⑥⑦⑧ | Low | ✅ Done |
| **2** | **Sinyal Aksi** (Signal Actions) | 5 tandang chips + inline vote | SDK: 7 outbound, 6 inbound, 2 queries | ③⑤⑥⑦ | Medium | 🔵 Spec'd |
| **3** | **Wajah Kepercayaan** (Trust Surface) | Tier badges, vouch counts, PoR badges | SDK: 0 outbound, 3 inbound, 7 queries | ①②⑤⑦ | Medium | 🔵 Spec'd |
| **4** | **Jejak Saya** (My Trail) | Full tandang personal dashboard | SDK: 0 outbound, 4 inbound, 3 queries | ②④⑥⑧ | High | 🔵 Spec'd |
| **5** | **Aliran** (River Mode) | Full-width snap-scroll view | Reuses Phase 2–4 | ③⑤⑦ | High | ⬜ Not started |

### Octalysis Score Projection

| Drive | Before | Ph1 ✅ | +Ph2 | +Ph3 | +Ph4 | +Ph5 | Final |
|-------|--------|--------|------|------|------|------|-------|
| ① Epic Meaning | 8 | 8 | 8 | **9** | 9 | 9 | **9** |
| ② Accomplishment | 2 | 3 | 3 | **6** | **9** | 9 | **9** |
| ③ Empowerment | 3 | 3 | **7** | 7 | 7 | **9** | **9** |
| ④ Ownership | 1 | 1 | 1 | 1 | **8** | 8 | **8** |
| ⑤ Social Influence | 4 | 6 | **8** | **9** | 9 | 9 | **9** |
| ⑥ Scarcity | 0 | 3 | **5** | 5 | **7** | 7 | **7** |
| ⑦ Unpredictability | 1 | 6 | **7** | **8** | 8 | **9** | **9** |
| ⑧ Loss & Avoidance | 0 | 3 | 3 | 3 | **7** | 7 | **7** |
| **Total** | **19** | **33** | **42** | **48** | **64** | **67** | **67/80** |

**v0.1 plan peaked at ~55/80.** Tandang integration pushes to **67/80** because every interaction is consequential.

Biggest jumps enabled by tandang:
- **④ Ownership: 1 → 8** — impossible without vouch budget, C/J scores, domain portfolio
- **② Accomplishment: 2 → 9** — C score radar, polymath, verification badges
- **⑥ Scarcity: 0 → 7** — real vouch limits (10–100 per tier), not manufactured

---

## 7. Ethical Guardrails (Revised)

### The Four Tests (expanded from three in v0.1)

1. **Truth Test**: Is this based on real data/deadlines, or manufactured urgency?
   - ✅ "Voting closes in 2 hours" (real deadline)
   - ✅ "12/25 vouches remaining" (real tandang tier limit)
   - ❌ "Only 3 spots left!" (fake scarcity)

2. **Autonomy Test**: Can the user opt out without penalty?
   - ✅ Streak freeze available, no public shame
   - ✅ Decay warnings are transparent with clear recovery action
   - ❌ "You'll lose your streak!" with no recovery

3. **Alignment Test**: Does this serve the user's stated goal (civic participation)?
   - ✅ Vouch chips that build community trust graph
   - ❌ Gamified points that incentivize quantity over quality

4. **Accountability Test** (NEW): Does every action carry real weight?
   - ✅ Vouch stakes J score — bad judgment has consequences
   - ✅ "Saya Saksi" is a PoR attestation — lying has slash consequences
   - ❌ Generic emoji reactions with no backend meaning

### The Tandang Ethics Principle

> **Every reaction chip must map to a real tandang signal with real consequences. No vanity metrics. No engagement without accountability.**

### What We Never Do
- Never manufacture fake urgency or scarcity (tandang's limits are REAL)
- Never punish inaction (only reward action; decay is transparent with clear recovery)
- Never show engagement metrics that encourage toxic competition
- Never use dark patterns to prevent leaving/closing
- Never sell attention data or optimize for time-on-app
- **NEW:** Never create a UI action without a corresponding tandang signal
- **NEW:** Never let implicit behavior affect reputation scores (only explicit taps)

---

## 8. Metrics to Track (Revised)

| Metric | What It Measures | Target | Tandang Signal |
|--------|-----------------|--------|---------------|
| **Cards-to-action ratio** | Cards seen before first chip tap | < 5 | Any outbound signal |
| **Time-to-first-action** | Seconds from app open to first chip tap | < 30s | Any outbound signal |
| **Signal diversity** | Types of chips tapped per session | > 2 types | Vouch, Skeptis, Saksi, Bagus, Vote |
| **Vouch-to-skeptis ratio** | Health of trust ecosystem | 3:1 to 5:1 | `vouch_created` by type |
| **PoR submission rate** | Real-world witnessing | > 10% of active users | `por_evidence_submitted` |
| **Decay prevention rate** | Users who act before competence decays | > 60% | `decay_prevented` vs `decay_competence_decayed` |
| **Streak retention** | % maintaining 3+ day streaks | > 40% | `activity_streak_days` |
| **Return rate** | Users returning within 24 hours | > 50% | Session tracking |
| **Daily active participants** | Users taking 1+ meaningful action/day | Growing MoM | Any outbound signal |
| **Trust surface engagement** | Do users check vouch/skeptis counts? | > 30% view trust details | Client analytics |

---

## 9. References

- [Octalysis Framework — Yu-kai Chou](https://yukaichou.com/gamification-examples/octalysis-gamification-framework/)
- [Actionable Gamification — Yu-kai Chou (Book)](https://yukaichou.com/actionable-gamification-book/)
- [Hooked: How to Build Habit-Forming Products — Nir Eyal](https://www.nirandfar.com/hooked/)
- [Gamification, Crowdsourcing and Civic Tech — The Good Lobby](https://thegoodlobby.eu/gamification-crowdsourcing-civic-tech/)
- [Gameful Civic Engagement: Review of Literature — ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0740624X19302606)
- **Tandang Engine** — `/Users/damarpanuluh/MERIDIAN-NEW/tandang/` (internal)
- [TANDANG-SIGNAL-INVENTORY-v0.1.md](./TANDANG-SIGNAL-INVENTORY-v0.1.md) — Signal catalog
- [TANDANG-INTEGRATION-CONTRACT-v0.1.md](./TANDANG-INTEGRATION-CONTRACT-v0.1.md) — Integration contract

---

## Changelog

| Date | Change |
|------|--------|
| 2025-02-19 | v0.1 — Initial brainstorm, 6 feature proposals, Octalysis mapping |
| 2025-02-19 | v0.2 — Tandang fusion: signal-driven phases, 5 chips replacing generic reactions, removed "Saya Ikut", contextual PoR wording, explicit/implicit signal classification, trust surface + personal dashboard phases, revised Octalysis projections (67/80), accountability ethics test |

# Engagement Strategy — Gotong Royong Pulse Feed
> Version 0.1 — Living Document
> Status: **DRAFT / BRAINSTORM**
> Last updated: 2025-02-19

---

## 1. Problem Statement

Current social media trains users for short attention spans with zero-friction dopamine loops. Our masonry feed (Pulse) provides excellent **eagle-eye awareness** of community initiatives, but requires **effort to engage** — too many clicks to understand what's happening, no instant gratification.

**We are NOT building addictive-for-profit software.** We are making civic action *compete* with the attention economy. If TikTok can make people scroll for 3 hours watching cat videos, we can make them scroll for 30 minutes solving neighborhood problems.

### The Pinterest Trap
Pinterest masonry = beautiful overview, but:
- Every card is a promise that requires a click to fulfill
- Click → full page → back → lost context → friction → bounce
- No *life* on the cards themselves — static thumbnails

### What We Want
- Eagle-eye overview (keep masonry) **+** zero-click dopamine (steal from TikTok/Instagram)
- Users **feel** community activity without clicking
- Quick actions that give instant satisfaction
- Ethical hooks that serve the user's *actual* interest (civic participation)

---

## 2. Theoretical Framework: Octalysis

We use Yu-kai Chou's [Octalysis Framework](https://yukaichou.com/gamification-examples/octalysis-gamification-framework/) as our engagement lens. Eight Core Drives of human motivation:

### White Hat Drives (positive, empowering — our PRIMARY tools)
| # | Drive | Description | Our Leverage |
|---|-------|-------------|--------------|
| 1 | **Epic Meaning & Calling** | Feeling part of something greater | Gotong royong IS the calling — community problem-solving |
| 2 | **Development & Accomplishment** | Making progress, overcoming challenges | Progress bars, phase completion, personal impact stats |
| 3 | **Empowerment of Creativity & Feedback** | Creative expression with feedback loops | Inline voting, quick reactions, shaping outcomes |

### Utility Drives (context-dependent)
| # | Drive | Description | Our Leverage |
|---|-------|-------------|--------------|
| 4 | **Ownership & Possession** | Feeling you own something | "Your contributions" trail, streak counters |
| 5 | **Social Influence & Relatedness** | Social elements and connections | Avatar stacks, live activity indicators, social proof |

### Black Hat Drives (use sparingly, only when TRUE — never manufactured)
| # | Drive | Description | Our Leverage |
|---|-------|-------------|--------------|
| 6 | **Scarcity & Impatience** | Wanting what you can't have | Real deadlines only — voting closes, quorum needed |
| 7 | **Unpredictability & Curiosity** | Wanting to discover what happens next | Variable card content, live previews, surprise resolutions |
| 8 | **Loss & Avoidance** | Avoiding negative outcomes | Real consequences — "fase ini selesai besok, belum ada bukti" |

### Current State Assessment

| Drive | Current State | Score |
|-------|--------------|-------|
| 1 — Epic Meaning | Inherent in platform mission | ████████░░ 8/10 |
| 2 — Accomplishment | No visible progress/milestones on cards | ██░░░░░░░░ 2/10 |
| 3 — Empowerment | Chat exists, but no quick actions | ███░░░░░░░ 3/10 |
| 4 — Ownership | No personal contribution trail | █░░░░░░░░░ 1/10 |
| 5 — Social Influence | Avatar stacks, member counts (partial) | ████░░░░░░ 4/10 |
| 6 — Scarcity | Nothing — everything always available | ░░░░░░░░░░ 0/10 |
| 7 — Unpredictability | Feed is static, no surprises | █░░░░░░░░░ 1/10 |
| 8 — Loss & Avoidance | No consequence signals | ░░░░░░░░░░ 0/10 |

**Total: 19/80 — massive room for growth.**

---

## 3. The Hook Model for Gotong Royong

Adapted from Nir Eyal's Hooked model:

```
TRIGGER  → Push notification: "Ramai di RT 05 — 12 orang baru bergabung"
    ↓
ACTION   → Open app, see pulsing card, swipe-react or tap to vote (LOW friction)
    ↓
REWARD   → Variable: sometimes your vote tips the count, sometimes you see a
           surprise resolution, sometimes someone thanked you by name
    ↓
INVEST   → Your streak grows, your impact counter ticks up, you follow more
           entities → better triggers next time
    ↓
           ──── LOOP ────
```

---

## 4. Feature Proposals

### 4.1 — Story Peek: Zero-Click Dopamine
> **Drives:** ⑦ Unpredictability, ⑤ Social Influence
> **Effort:** Medium | **Impact:** High
> **Status:** [ ] Not started

**What:** Cards show *life* without clicking — live previews of activity.

**Mechanics:**
- **Peek strip** at bottom of card: 2-line auto-rotating preview of latest chat messages (every 5 seconds)
- **Micro-animation on scroll-in**: progress bar filling, vote count ticking up, typing indicator
- Card body dynamically shows the *most interesting* recent event, not just the creation event

**Why it works:** TikTok's killer move is content plays *before* you decide to engage. We show community activity happening *on the card* — you feel the pulse without clicking.

**Implementation notes:**
- Peek strip: rotate through `messages[]` from WitnessDetail, show truncated latest 2 messages
- IntersectionObserver for scroll-in animations (only animate when card enters viewport)
- Keep animations subtle — soft fade, not flashy

**Mockup concept:**
```
┌─────────────────────────┐
│ BARU        📢          │
│                         │
│ Jalan berlubang di      │
│ depan SDN 03...         │
│                         │
│ 📍 RT 05  🏷️ Infrastruktur │
│                         │
│ [RK][SD] 👥15 🔥Ramai   │
│ TERLIBAT        👁 🔖   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 💬 Sari: "Sudah foto    │ ← peek strip
│ buktinya tadi pagi"     │   (auto-rotates)
└─────────────────────────┘
```

---

### 4.2 — Pulse Ring: Live Activity Heartbeat
> **Drives:** ⑤ Social Influence, ⑦ Unpredictability
> **Effort:** Low | **Impact:** Medium
> **Status:** [ ] Not started

**What:** Cards with recent activity get a visual "alive" signal.

**Mechanics:**
- Cards with activity in last 30 minutes: sentiment shadow **subtly pulses** (CSS animation on box-shadow opacity)
- Tiny **"🟢 3 aktif"** live indicator badge on hot cards
- Occasional **"X baru bergabung"** toast/ribbon overlaying the masonry

**Why it works:** Instagram's gradient ring around active stories. You know something is happening *before* you tap. Creates FOMO (Drive ⑥) naturally.

**Implementation notes:**
- CSS `@keyframes pulse-glow` on the existing sentiment shadow
- `is_live` computed from `latest_event.timestamp` < 30 minutes
- Toast: inject as system card in feed stream, auto-dismiss after 5s

---

### 4.3 — Quick Strike: Friction-Zero Actions
> **Drives:** ③ Empowerment, ⑤ Social, ⑦ Unpredictability
> **Effort:** Medium | **Impact:** Very High
> **Status:** [ ] Not started

**What:** One-tap/swipe actions directly on the card — no detail view needed.

**Mechanics:**
- **Long-press** card → quick-react emoji picker (🔥 semangat, 👍 setuju, 🙏 terima kasih, 💪 siap bantu)
- **Inline vote widget** visible on cards with active voting — don't make them open detail to vote
- **"Saya ikut" quick-join button** on card footer — instant participation

**Why it works:** TikTok = swipe = done. Instagram = double-tap = like. The action IS the dopamine hit. Our current flow is: see card → click → open detail → find action → do it. That's 4 steps. Quick Strike = 1 step.

**Implementation notes:**
- Long-press: Svelte `use:longpress` action, 500ms threshold
- Emoji picker: small popover anchored to card, 4 preset reactions
- Inline vote: only shown when `urgency === 'voting'`, compact yes/no buttons
- Join button: replaces/augments the "Pantau" (eye) hover action

**Mockup concept:**
```
┌─────────────────────────┐
│ 🗳️ VOTING               │
│                         │
│ Setuju perbaikan jalan  │
│ pakai dana RT?          │
│                         │
│  ┌──────┐  ┌──────┐    │ ← inline vote
│  │👍 Ya  │  │👎 Tidak│   │   (one tap!)
│  │  23   │  │   4   │   │
│  └──────┘  └──────┘    │
│                         │
│ [SD][BK] 👥27           │
│ ⏰ 2 jam lagi            │ ← countdown
└─────────────────────────┘
```

---

### 4.4 — Countdown Urgency: Real Deadlines
> **Drives:** ⑥ Scarcity, ⑧ Loss & Avoidance
> **Effort:** Low | **Impact:** Medium
> **Status:** [ ] Not started

**What:** Time-sensitive badges on cards with real deadlines.

**Mechanics:**
- **"⏰ Voting ditutup dalam 2 jam"** countdown badge — real deadline, not manufactured
- **"📢 3 orang lagi untuk kuorum"** — scarcity of participants
- **"Fase selesai besok — belum ada bukti"** — real consequence framing
- Badge color intensifies as deadline approaches (warning → destructive)

**Why it works:** Loss aversion is 2x stronger than gain motivation. But ONLY use for real deadlines — manufactured urgency erodes trust.

**Ethics rule:** Every countdown must map to a real system deadline. Never fake scarcity.

**Implementation notes:**
- New `deadline` field on FeedItem (optional ISO timestamp)
- New `quorum_needed` / `quorum_current` fields
- Badge component with live countdown (requestAnimationFrame or 1-minute interval)
- Color escalation: `text-muted-foreground` → `text-peringatan` → `text-bahaya`

---

### 4.5 — Streak & Impact Counter: Personal Ownership
> **Drives:** ② Accomplishment, ④ Ownership
> **Effort:** Medium | **Impact:** Medium
> **Status:** [ ] Not started

**What:** Personal contribution tracking that creates ownership and habit.

**Mechanics:**
- **"🔥 7 hari aktif"** streak badge on user profile — consecutive days contributing
- **"Kamu membantu 3 inisiatif minggu ini"** — weekly personal impact counter
- **Per-card contribution marker**: "Kontribusi kamu: 2 bukti, 1 suara" — shows YOUR contribution to THIS witness
- **Impact summary** in profile/settings — total witnesses joined, votes cast, evidence submitted

**Why it works:** Snapchat streaks are absurdly effective at creating daily habit. Duolingo streak anxiety drives DAU. We apply the same mechanic to civic participation.

**Ethics rule:** Streaks should encourage, not punish. "Streak freeze" available. No public shaming for broken streaks.

**Implementation notes:**
- Backend: track `user_daily_activity` events, compute streak server-side
- Per-card: `my_contributions` field on FeedItem (optional, populated when user is participant)
- Profile page: aggregate stats from user activity log
- Streak badge: shown in AppHeader or profile avatar area

---

### 4.6 — River Mode: The TikTok View
> **Drives:** ⑦ Unpredictability, ③ Empowerment, ⑤ Social
> **Effort:** High | **Impact:** Very High
> **Status:** [ ] Not started

**What:** Alternative full-width vertical scroll view — one card at a time, like TikTok/Instagram Stories but for civic issues.

**Mechanics:**
- Toggle between **"Peta" (masonry/eagle view)** and **"Aliran" (river/stream view)**
- River mode = full-width cards, one at a time, vertical snap-scroll
- Each card in river mode: hook_line → peek of activity → quick-action buttons → swipe to next
- Swipe up = next issue. Swipe right = join. Long press = react.
- Auto-advances after 10 seconds of inaction (optional, user-configurable)

**Why it works:** This is where short-attention-span users live. Masonry = for planners who want overview. River = for scrollers who want to be fed issues one by one and drop quick actions as they go.

**Implementation notes:**
- New view mode state in FeedStore: `'masonry' | 'river'`
- River component: CSS scroll-snap, full-width card variant of FeedEventCard
- Reuse existing card data, different layout/presentation
- Toggle button in feed header (Peta/Aliran icons)
- Consider: river mode as DEFAULT on mobile, masonry as default on desktop?

**Mockup concept:**
```
┌─────────────────────────────────────┐
│ ← Pulse          🔀 Peta | █Aliran█ │
├─────────────────────────────────────┤
│                                     │
│  📢 BARU                            │
│                                     │
│  Jalan berlubang di depan SDN 03    │
│  sudah 2 minggu belum diperbaiki    │
│                                     │
│  ┌─────────────────────────────┐    │
│  │ 📸 [cover photo]            │    │
│  └─────────────────────────────┘    │
│                                     │
│  💬 Sari: "Sudah foto buktinya"     │
│  💬 Ahmad: "Saya hubungi RT dulu"   │
│                                     │
│  📍 RT 05 Menteng · 👥 15 anggota   │
│                                     │
│  ┌──────┐ ┌──────┐ ┌──────┐       │
│  │🔥 Dukung│ │👍 Setuju│ │💪 Bantu │  │
│  └──────┘ └──────┘ └──────┘       │
│                                     │
│         ⬆️ swipe untuk lanjut        │
└─────────────────────────────────────┘
```

---

## 5. Priority Roadmap

| Phase | Features | Drives Hit | Effort |
|-------|----------|------------|--------|
| **Phase 1: Feel Alive** | 4.2 Pulse Ring + 4.4 Countdown | ⑤⑥⑦⑧ | Low — CSS + data fields |
| **Phase 2: Quick Dopamine** | 4.3 Quick Strike (reactions + inline vote) | ③⑤⑦ | Medium — new interactions |
| **Phase 3: Show Life** | 4.1 Story Peek (chat preview on card) | ⑤⑦ | Medium — data integration |
| **Phase 4: Ownership** | 4.5 Streak & Impact Counter | ②④ | Medium — backend needed |
| **Phase 5: The Shift** | 4.6 River Mode | ③⑤⑦ | High — new view mode |

### Phase 1 is the quick win
Pulse Ring + Countdown can be done with CSS animations and 2-3 new data fields. No new interactions, no backend changes. Makes the feed *feel alive* immediately.

### Phase 3 vs Phase 2 ordering
Quick Strike (Phase 2) before Story Peek (Phase 3) because actions create more engagement than passive viewing. Let people *do* something before showing them more content.

### Phase 5 is the moonshot
River Mode is essentially building a second app experience. Ship Phases 1-4 first, validate engagement metrics, then build River Mode with confidence.

---

## 6. Ethical Guardrails

We are building engagement mechanics for civic good. Every feature must pass these gates:

### The Three Tests
1. **Truth Test**: Is this based on real data/deadlines, or manufactured urgency?
   - ✅ "Voting closes in 2 hours" (real deadline)
   - ❌ "Only 3 spots left!" (fake scarcity)

2. **Autonomy Test**: Can the user opt out without penalty?
   - ✅ Streak freeze available, no public shame
   - ❌ "You'll lose your streak!" with no recovery

3. **Alignment Test**: Does this serve the user's stated goal (civic participation)?
   - ✅ Quick reactions on real community issues
   - ❌ Gamified points that incentivize quantity over quality

### What We Never Do
- Never manufacture fake urgency or scarcity
- Never punish inaction (only reward action)
- Never show engagement metrics that encourage toxic competition
- Never use dark patterns to prevent leaving/closing
- Never sell attention data or optimize for time-on-app at expense of user wellbeing

---

## 7. Metrics to Track

| Metric | What It Measures | Target |
|--------|-----------------|--------|
| **Cards-to-action ratio** | How many cards seen before first action | < 5 (currently ∞) |
| **Time-to-first-action** | Seconds from app open to first meaningful action | < 30s |
| **Daily active participants** | Users who take at least 1 action/day | Growing month-over-month |
| **Streak retention** | % of users maintaining 3+ day streaks | > 40% |
| **Return rate** | Users who come back within 24 hours | > 50% |
| **Action diversity** | Types of actions per session (vote, react, join, evidence) | > 2 types |

---

## 8. References

- [Octalysis Framework — Yu-kai Chou](https://yukaichou.com/gamification-examples/octalysis-gamification-framework/)
- [Actionable Gamification — Yu-kai Chou (Book)](https://yukaichou.com/actionable-gamification-book/)
- [The Dopamine Loop: How UX Designs Hook Our Brains](https://medium.com/design-bootcamp/the-dopamine-loop-how-ux-designs-hook-our-brains-bd1a50a9f22e)
- [Hooked: UX Psychology Behind Social Media's Addictive Design](https://thefuturecanvas.com/blog/hooked-the-ux-psychology-behind-social-medias-addictive-design)
- [Gamification, Crowdsourcing and Civic Tech — The Good Lobby](https://thegoodlobby.eu/gamification-crowdsourcing-civic-tech/)
- [Gameful Civic Engagement: Review of Literature — ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0740624X19302606)
- [Hooked: How to Build Habit-Forming Products — Nir Eyal](https://www.nirandfar.com/hooked/)

---

## Changelog

| Date | Change |
|------|--------|
| 2025-02-19 | v0.1 — Initial brainstorm, 6 feature proposals, Octalysis mapping |

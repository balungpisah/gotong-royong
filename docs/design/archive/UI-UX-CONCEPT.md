# Gotong Royong — UI/UX Concept (v0.1)

## Design Pillars

**Map-First.** The map is the app. Not a feature inside the app — the map IS the primary surface. Every witness has a location. Every location tells a story. Users don't scroll a feed; they look at their world.

**Community Pulse.** The first thing you see isn't your notifications — it's how the community is doing. Weather, active witnesses, recent impact. The app trains you to think collectively before acting individually.

**Subtle Signals.** Reputation is ambient. You feel who's trusted through visual cues — ring colors, avatar warmth, weight in governance — not through leaderboards or point counters. Trust is the texture of the interface, not a feature in it.

**Witness Narrative.** Every witness is a story with chapters. Users stay because they're emotionally invested in outcomes — "did my report lead to a real solution?" — not because they're chasing likes. The app's engagement loop is narrative progression, not social validation.

**Smart Urgency.** Notifications respect attention by default (daily digest) but escalate to real-time when it actually matters (challenge windows closing, tasks stalling, weather degrading). The system earns the right to interrupt you.

---

## Engagement Model: Why People Come Back

GR competes for the same screen time as TikTok, Instagram, and WhatsApp. It won't win by copying their dopamine loops — but it can't survive without its own. The engagement model is built on narrative investment, not vanity metrics.

### The Core Loop

```
WITNESS something → get co-witnessed (validation) →
  watch your witness progress through stages (narrative) →
    see real-world impact confirmed (payoff) →
      your CV Hidup grows (career capital) →
        you're trusted enough to do more (status) →
          WITNESS something new
```

Each step has a reward:
- **Co-witness notification**: "2 people confirmed what you saw." Instant. Social. Meaningful.
- **Stage transition**: "Your witness just reached Execute — someone is working on it." This is a story beat. You're the protagonist.
- **Impact confirmation**: "The community confirmed: your witness led to 50 families receiving aid." This is the climax. No social media post delivers this feeling.
- **CV Hidup update**: "New ESCO skill verified: Emergency Response (top 30%)." Real-world career reward.
- **Trust ring grows**: Your avatar subtly brightens. Others see it. You see it. No number — just warmth.

### Narrative Hooks (Per Stage)

Every witness you create or participate in becomes a story you follow:

| Stage Transition | Notification | Emotional Hook |
| --- | --- | --- |
| Seed created | — | "I just told the world what I saw" |
| First co-witness | "Someone confirmed your witness" | Validation — someone else sees what you see |
| Seed → Define | "The community is defining what to do about your witness" | Anticipation — it's becoming real |
| Define → Path | "A plan has been locked for your witness" | Relief + excitement — action is coming |
| Path → Execute | "Someone claimed a task from your witness" | Progress — the system works |
| Execute heartbeat | "Work update on your witness" (weekly) | Sustained engagement — the story continues |
| Execute → Accept | "Solution verified for your witness" | Pride — what you reported got fixed |
| Accept → Impact | "Impact confirmed: [summary]" | Climax — your witness made a real difference |
| Archive | "Your witness journey is complete" | Closure + CV Hidup update |

Users who create a witness get notifications for EVERY stage transition of that witness. Users who co-witness, vote, or execute get notifications for the stages they care about.

This is the engagement model: you're not refreshing for likes, you're checking if your story progressed.

### Smart Urgency Notification Tiers

| Tier | Trigger | Delivery | Frequency Cap |
| --- | --- | --- | --- |
| **Ambient** | Co-witness added, stage transition on your witness, governance outcome | Daily digest (evening) | 1 digest/day |
| **Timely** | "Needs You" match in your ESCO domain, new witness near you, task available | Batched every 4h during active hours | Max 3/day |
| **Urgent** | Challenge window closing (<2h), your task stalling, weather degrading to Hujan/Badai | Real-time push | No cap (rare by design) |
| **Emergency** | Emergency brake triggered, fraud slash in your vouch network | Real-time push + banner | No cap (extremely rare) |

Key principle: **the system earns the right to interrupt you.** Ambient events respect your time. Urgent events deserve your attention. Most users get 1-2 real-time pushes per week, not per hour.

### Why This Beats Social Media's Loop

Social media: post → get likes → dopamine → post more → likes fade → post again. The reward is validation from strangers. It's empty. Users know it's empty. They do it anyway because nothing better exists.

GR: witness → get co-witnessed → watch your story progress → see real impact → your career grows → you're more trusted → witness again. The reward is narrative completion and real-world capital. It's not empty. And it compounds — unlike likes, your CV Hidup and trust ring are durable assets.

The "addiction" is: **what happened to my witness today?** That's a question worth opening an app for.

---

## Core Metaphor: The Living Map

The app is a living map of community reality. Witnesses are events happening on the map. The weather overlay shows community health. You zoom in to act, zoom out to observe.

The map inherits the whitepaper's seed-to-fruit metaphor:
- Seed stage witnesses pulse gently (something just appeared)
- Define/Path witnesses glow steadier (the community is shaping this)
- Execute witnesses show activity lines (work is happening)
- Accept/Impact witnesses bloom (outcome confirmed)
- Archived witnesses fade into the terrain (part of history)

---

## Screen Architecture

### 1. Home: Community Pulse + Map

The home screen is split vertically:

**Top third — Pulse Bar (always visible, collapsible):**
- Weather indicator: single icon + label (Cerah/Berawan/Hujan/Badai) with current GDF percentage
- Active witness count in your area
- "Needs You" badge: count of witnesses in your ESCO domains awaiting your contribution (verify, vote, execute)
- Your governance budget remaining (subtle progress arc, not a number)

**Bottom two-thirds — The Map:**
- Default view: centered on user's location, ~5km radius
- Witness pins clustered by proximity
- Pin appearance encodes track + stage:

| Track | Pin Shape | Stage Encoding |
| --- | --- | --- |
| Resolve | Circle | Opacity: dim (Seed) → solid (Execute) → bright ring (Accept/Impact) |
| Celebrate | Star | Same opacity progression |
| Explore | Diamond | Same opacity progression |

- Confidential witnesses appear as anonymous pins (no content preview until you tap in)
- Tap a cluster → expand to individual pins with 1-line summaries
- Tap a pin → slide-up Witness Card (see below)
- Long-press anywhere → "I witness something here" (create new witness)

**Map Overlays (toggle):**
- Weather heatmap: colored overlay showing per-region GDF health
- Domain filter: show only witnesses matching selected ESCO domains
- Track filter: Resolve / Celebrate / Explore / All
- "My Activity" filter: only witnesses I'm involved in

### 2. Witness Card (Slide-Up Panel)

When you tap a witness pin, a bottom sheet slides up (iOS/Android pattern). Three sizes: peek (summary), half (details), full (journey).

**Peek (1/4 screen):**
- Claim summary (1-2 lines, redacted if Confidential)
- Track + Stage badge
- Witness complexity indicator (thin bar, color-coded)
- Co-witness count
- Time since last activity
- Submitter avatar with trust ring (see Subtle Signals below)

**Half (1/2 screen, swipe up from peek):**
- Full claim summary
- ESCO domain tags (chips)
- Context Triad status: three small icons (camera/pin/people) — filled if present, outlined if missing
- Current transition proposal (if any) with vote tally bar
- "What's needed" section: AI-generated prompt ("Needs 1 more co-witness", "Awaiting verifier in Civil Engineering", etc.)
- Action buttons row (context-dependent):
  - Co-witness / Vote / Claim Task / Verify / Attest Impact / Dispute

**Full (swipe to full screen):**
- Complete witness journey timeline (vertical, stages as nodes)
- Each stage shows: who proposed transition, vote results, evidence submitted, challenge window status
- Criteria checklist at each stage
- Discussion thread (if any)
- Linked tasks (if Resolve track, Execute stage)
- Impact attestations (if Impact stage)

### 3. Create Witness (Long-Press → New Witness Flow)

Mobile-first, camera-first:

**Step 1 — Capture**
- Camera opens immediately (photo/video)
- Or: text-only option ("I don't have visual evidence")
- Location auto-captured (can coarsen or override)
- Voice-to-text option for the claim description

**Step 2 — Describe**
- "What are you witnessing?" — free text (min 20 chars)
- Claim type selector: Good (celebration) / Bad (problem) / Unknown (question)
- This determines initial track suggestion: Good → Celebrate, Bad → Resolve, Unknown → Explore

**Step 3 — Disclosure**
- "Who can see this?"
  - Community-Open (default)
  - Confidential (AI redacts before publishing)
  - Fully-Open

**Step 4 — Review**
- AI-generated summary preview (what the published stub will look like)
- ESCO tags suggested by AI (editable)
- Context Triad checklist: what you have / what's missing (informational, not blocking at Seed)
- Estimated witness complexity
- Submit button

No track selection required — AI suggests, community confirms at Define. No ESCO expertise required — AI tags, sensemakers refine.

### 4. My Journey (Tab)

Personal dashboard, secondary to the map:

**Reputation Section (Subtle):**
- Your avatar with trust ring
- Tier label (Novice / Contributor / Pillar / Keystone) — text only, no percentile
- ESCO domains you've contributed in — shown as a small skill cloud (larger = more competence, no numbers)
- "Your impact": count of witnesses you touched that reached Impact stage
- CV Hidup button → opens exportable living CV

**Activity Section:**
- Active witnesses you're involved in (as mini-cards, tappable → Witness Card)
- Pending actions: votes to cast, tasks to heartbeat, verifications waiting
- Recent governance outcomes: "Your endorsement on W-042 was upheld" / "Your dispute on W-018 was dismissed"

**Budget Arc:**
- Single circular arc showing governance + vouch budget usage
- No numbers — just a visual of "how much influence you have left to spend"
- Tap for breakdown (vouches vs. governance locks)

### 5. Governance Center (Tab or Slide)

For users who participate in governance:

**Active Proposals:**
- Sorted by urgency (challenge window closing soon on top)
- Each shows: witness summary, proposed transition, current vote tally (bar), your weight if you vote, time remaining
- Swipe right to approve, swipe left to reject, tap to open full Witness Card

**Disputes:**
- Active disputes you can participate in
- Jury service notifications

**Emergency:**
- Emergency brake alerts (rare, high-visibility)
- Fast-track audit status

### 6. Weather Station (Accessible from Pulse Bar)

Full-screen weather dashboard:

- Current weather: large icon + label + GDF percentage
- 7-day trend: simple sparkline
- Recent slash events (anonymized): "Fraud detected in [ESCO domain]. GDF increased by 0.3%."
- Weather Heroes leaderboard (opt-in): top 10 community guardians this month
- Your Weather Hero status (if applicable)
- Domain Efficiency dashboard: which ESCO domains are healthy/sluggish/critical

---

## Subtle Signals: The Trust Language

### Avatar Trust Rings

Every user avatar has a thin ring around it. The ring encodes trust WITHOUT showing numbers:

| Ring Appearance | Meaning |
| --- | --- |
| No ring (just avatar) | Novice — new user, hasn't built trust yet |
| Thin gray ring | Contributor — moderate trust |
| Solid colored ring (domain-tinted) | Pillar — high trust in their primary domain |
| Double ring (inner + outer) | Keystone — top-tier trust |
| Pulsing golden ring | Genesis — bootstrap node (fades over 10 months) |
| Dashed ring | Shadow — under review / recovery |

Ring COLOR is tinted by the user's primary ESCO pillar:
- Blue-green: Tech & Build
- Orange-amber: Field & Social
- Silver-gray: Logic & Audit
- Deep red-brown: Culture & Context

Multi-domain users get a gradient ring.

### Vote Weight Visualization

When voting on proposals, users see their vote as a "stone" dropped into a pool:
- Larger stone = more weight (compound multiplier effect)
- The pool fills up toward threshold
- No exact numbers shown — you see your relative contribution

If you want numbers, long-press your stone to see: weight breakdown (I_Mult × J_Mult × ID_Mult × base_weight).

### Evidence Confidence

At the Witness Card level, evidence strength is shown as the Context Triad:
- Three icons: camera (Visual), pin (Locational), people (Corroborative)
- Filled icon = present, outlined = missing
- Simple, glanceable, no jargon

### Activity Warmth

Users who've been active recently have "warm" avatars (slightly brighter). Inactive users fade slightly (not hidden, just cooler). This makes decay visible without numbers — you can see who's currently engaged.

---

## Mobile-First Patterns

### Gestures

| Gesture | Action |
| --- | --- |
| Long-press on map | Create new witness at that location |
| Tap pin | Open Witness Card (peek) |
| Swipe up on card | Expand card (peek → half → full) |
| Swipe down on card | Collapse / dismiss |
| Swipe right on proposal | Approve (locks governance budget) |
| Swipe left on proposal | Reject |
| Pinch map | Zoom in/out (shows/hides cluster detail) |
| Pull down on map | Refresh + show Pulse Bar |
| Shake device | Quick-report (emergency witness) |

### Notifications

See "Smart Urgency Notification Tiers" in the Engagement Model section above for the full notification strategy. Quick reference for implementation:

| Event | Tier | Delivery |
| --- | --- | --- |
| Co-witness added to your witness | Ambient | Daily digest |
| Stage transition on your witness | Ambient | Daily digest |
| Governance outcome (endorsement upheld/overturned) | Ambient | Daily digest |
| New witness near you | Timely | Batched 4h |
| Task available in your ESCO domain | Timely | Batched 4h |
| "Needs You" match | Timely | Batched 4h |
| Challenge window closing (<2h) | Urgent | Real-time push |
| Your task stalling (heartbeat overdue) | Urgent | Real-time push |
| Weather degrading to Hujan/Badai | Urgent | Real-time push |
| Emergency brake triggered | Emergency | Real-time push + banner |
| Fraud slash in your vouch network | Emergency | Real-time push + banner |

### Offline Considerations

Field witnesses may have poor connectivity:
- Witness creation works offline (queued, synced when connected)
- Photos cached locally until upload completes
- Map tiles cached for recent area
- Voting requires connectivity (weights need current epoch data)

---

## Navigation Structure

```
┌─────────────────────────────────────┐
│          Pulse Bar (top)            │
│  Weather │ Active │ Needs You │ Budget │
├─────────────────────────────────────┤
│                                     │
│                                     │
│            MAP (center)             │
│         Witness pins + clusters     │
│         Overlay toggles             │
│                                     │
│                                     │
├─────────────────────────────────────┤
│  [Map]  [My Journey]  [Governance]  │
│         Bottom Tab Bar              │
└─────────────────────────────────────┘

Secondary screens (modal / push):
  - Witness Card (bottom sheet)
  - Create Witness (full-screen flow)
  - Weather Station (push from Pulse Bar)
  - CV Hidup (push from My Journey)
  - Settings / Profile (push from My Journey)
```

Three tabs only. Simple. Map is always home.

---

## Color Palette Direction

Inspired by Indonesian natural landscape + weather metaphor:

- **Primary:** Deep teal (laut / ocean — trust, depth)
- **Secondary:** Warm amber (tanah / earth — community, warmth)
- **Accent:** Coral (bunga / flower — achievement, celebration)
- **Background:** Off-white with warm undertone (not clinical white)
- **Text:** Dark charcoal (not pure black — softer)

Weather-responsive: the app's overall color temperature shifts slightly with community weather:
- Cerah (Clear): warmer, brighter background
- Berawan (Cloudy): slightly cooler
- Hujan (Rain): muted, blue-gray tint
- Badai (Storm): darker, more contrast

This is subtle — a 5-10% shift in background warmth. Users feel it more than see it.

---

## Design Tokens (Starter)

```
// Spacing
spacing-xs: 4px
spacing-sm: 8px
spacing-md: 16px
spacing-lg: 24px
spacing-xl: 32px

// Border radius
radius-sm: 8px     (chips, small buttons)
radius-md: 12px    (cards)
radius-lg: 16px    (bottom sheet)
radius-full: 50%   (avatars)

// Typography
font-family: "Plus Jakarta Sans" (Indonesian-designed, Latin + good for Bahasa)
heading-lg: 24px / 700
heading-md: 20px / 600
body: 16px / 400
body-sm: 14px / 400
caption: 12px / 400

// Shadows
shadow-card: 0 2px 8px rgba(0,0,0,0.08)
shadow-sheet: 0 -4px 16px rgba(0,0,0,0.12)

// Trust ring sizes
ring-none: 0px
ring-contributor: 2px
ring-pillar: 3px
ring-keystone: 3px + 1px gap + 2px outer
ring-genesis: 3px animated pulse
ring-shadow: 2px dashed
```

---

## Tech Stack Suggestion

| Layer | Choice | Rationale |
| --- | --- | --- |
| Mobile framework | React Native / Expo | Cross-platform, large ecosystem, good map libraries |
| Map | Mapbox GL (react-native-mapbox-gl) | Best mobile map UX, custom styling, offline tiles, clustering built-in |
| State | Zustand or Jotai | Lightweight, works well with React Native |
| API | REST (matching Tandang's Axum API) | Already defined in whitepaper |
| Offline | WatermelonDB | SQLite-based, sync-friendly, works with React Native |
| Notifications | Firebase Cloud Messaging | Standard for cross-platform push |
| Design system | Build custom (small token set above) | GR's visual language is unique enough to warrant its own |

Alternative: Flutter with Mapbox. Depends on team preference.

---

## MVP Scope (v0.1 UI)

Build these screens first, in this order:

1. **Map + Pins** — show witnesses on a map with clustering. Tap to see Witness Card (peek only). This is the skeleton.

2. **Create Witness** — long-press → camera → describe → submit. This is the core action.

3. **Witness Card (full)** — peek → half → full journey timeline. This is where governance happens.

4. **Pulse Bar** — weather + active count + needs-you badge. This trains community thinking.

5. **My Journey** — personal dashboard with reputation ring, active witnesses, pending actions.

6. **Governance** — vote on proposals (swipe), dispute list, budget arc.

Everything else (Weather Station, CV Hidup, heroes, settings) is v0.2.

---

## What This Intentionally Avoids (and Why It Still Works)

GR competes with social media for screen time. It doesn't copy their tricks — it replaces them with something that works AND means something.

- **No infinite feed.** Instead: the map is spatial content discovery. New witnesses animate in. You open the app to see what happened near you, not to scroll endlessly. The engagement hook is "what happened to my witness?" — narrative investment, not content consumption.
- **No leaderboard.** Instead: trust rings grow visibly on your avatar. Others see your growth. You see theirs. Status is ambient, not ranked. Weather Heroes are opt-in for those who want explicit recognition.
- **No vanity metrics.** Instead: CV Hidup is a real-world career asset. Your GR activity becomes a portable, ESCO-verified credential. That's not fake internet points — it's something an employer scans.
- **No algorithmic sorting.** Instead: geography is the algorithm. Witnesses appear where they are. "Needs You" matching uses your ESCO skills, not engagement optimization.
- **No onboarding tutorial.** The map is self-explanatory. Long-press to witness. Tap to see. The app teaches by doing.
- **No notification spam.** Instead: Smart Urgency tiers. Most events go in a daily digest. Real-time push only for time-sensitive governance actions and community weather changes. The system earns the right to interrupt you.

The result: users open GR because they care about what's happening in their community and what happened to their stories — not because a red badge is screaming at them.

---

*Note: This document is archived as historical concept material and is non-authoritative for implementation.*

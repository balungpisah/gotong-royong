> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

# Trajectory Map — Human-Intent Routing Model

> **Status:** Draft v0.1
> **Last updated:** 2026-02-22
>
> This document defines the 11 human-intent trajectory archetypes that
> drive AI-00 triage routing, feed presentation, and data structure
> selection. It replaces the old 5-track model (tuntaskan, wujudkan,
> telusuri, rayakan, musyawarah) with a richer intent-first taxonomy.
>
> **Canonical reference** — other spec docs link here instead of duplicating.

---

## 1. Guiding Principle

Don't start from system labels and force humans into them.
Start from **what humans actually want to do** and let the system shape around that.

The trajectory model asks: *"What does this person want to achieve?"*
— then selects the appropriate data structure, operator, and feed presentation.

---

## 2. Two Feed Categories

Not everything needs its own card type. The block system (7 block types)
provides flexibility — different trajectories use different **block
compositions** within the same card structure.

The real distinction is between two fundamental feed categories:

### 2.1 Ongoing Work → Witness (`FeedWitnessItem`)

Has lifecycle, phases, members, progress. Lives in the main feed.
The block composition inside varies by trajectory type.

**Trajectories:** A (aksi), B (advokasi), D (pantau), F (mufakat), L (mediasi), M (program)

### 2.2 One-off / Data Point → Data Item (`FeedDataItem`)

Point-in-time contribution. No lifecycle, no phases, no ongoing progression.
Appears in the feed but can also be **aggregated into special pages**
(e.g., Survey Board for C, Alert Board for J).

**Trajectories:** C (data), E (vault), G (bantuan), I (pencapaian), J (siaga)

### 2.3 Not a Feed Item (Baked into System)

- **K (Question)** → Orchestrator default behavior (pre-classification)
- **H (Offer help)** → Expertise Registry platform capability

---

## 3. Trajectory Reference Table

| Code | Name | TrajectoryType | Feed Category | Operator/Skill | Intent |
|------|------|----------------|---------------|----------------|--------|
| A | Collective Action | `aksi` | Witness | Aksi operator | "We can fix this ourselves" |
| B | Advocacy & Escalation | `advokasi` | Witness | Aksi operator | "We need authority to act" |
| C | Community Data / Survey | `data` | Data Item | Catat skill | "I'm stating a fact" |
| D | Watchdog / Monitor | `pantau` | Witness | Pantau operator | "I'm tracking an external case" |
| E | Private Vault | `vault` | Data Item | Brankas skill | "This is my proof" |
| F | Proposal / Musyawarah | `mufakat` | Witness | Mufakat operator | "I have an idea, let's decide" |
| G | Help Request | `bantuan` | Data Item | Cocok skill | "I need help" |
| H | ~~Resource Offering~~ | — | — | Expertise Registry | Platform capability |
| I | Celebration | `pencapaian` | Data Item | Rayakan skill | "Something good happened!" |
| J | Alert / Warning | `siaga` | Data Item | Catat skill | "Danger! People need to know" |
| K | Question / Inquiry | — | — | Orchestrator inline | Pre-classification default |
| L | Dispute Resolution | `mediasi` | Witness | Mufakat operator | "We have a conflict" |
| M | Ongoing Program | `program` | Witness | Jadwal skill | "We have a recurring activity" |

---

## 4. Witness Trajectories (Ongoing Work)

### 4.1 Trajectory A — Collective Action ("aksi")

> "Jalan di RT kami berlubang, kita bisa gotong royong perbaiki"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Community has a problem they can solve together |
| **Lifecycle** | Has a clear end (problem solved) |
| **Collaboration** | Full community |
| **Privacy** | Public |
| **Operator** | Aksi (trajectory = 'A') |
| **Mood color** | Amber — warm determination |

**Block composition:** `list` (phases/tasks), `computed` (progress), `form` (volunteer sign-up), `vote` (execution decisions), `reference` (linked witnesses)

**What AI asks:** What's the problem? Who's affected? What resources? Prior attempts? Scope?

**What system produces:** `PathPlan` with community-executable phases + checkpoints.

### 4.2 Trajectory B — Advocacy & Escalation ("advokasi")

> "Pabrik buang limbah ke sungai, kami butuh bantuan BLH"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Problem needs government, legal, or institutional intervention |
| **Lifecycle** | Ends when authority acts (or community escalates further) |
| **Collaboration** | Community + external authority |
| **Privacy** | Public or partial |
| **Operator** | Aksi (trajectory = 'B') |
| **Mood color** | Rose — urgent advocacy |

**Block composition:** `document` (evidence), `list` (escalation steps), `form` (evidence submission), `computed` (evidence count, days since report), `reference` (authority contacts)

**What AI asks:** What's the issue? Who is responsible? What evidence? Reported before? Urgency?

**What system produces:** `PathPlan` with case-building phases (building a case and escalating).

### 4.3 Trajectory D — Watchdog / Monitor ("pantau")

> "Kasus korupsi dana desa di Kelurahan X, saya mau pantau"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Track something you don't control — accountability/oversight |
| **Lifecycle** | Ends when external case resolves |
| **Collaboration** | Multi-watcher (anyone can contribute updates) |
| **Privacy** | Public |
| **Operator** | Pantau |
| **Mood color** | Indigo — cool observation |

**Block composition:** `list` (timeline events), `document` (evidence, summaries), `display` (media), `reference` (news articles), `computed` (days tracked, watcher count)

**What AI asks:** What case? Stage? Parties? Latest development? Hoped outcome?

**What system produces:** Timeline-oriented witness with events, evidence, and status tracking.

### 4.4 Trajectory F — Proposal / Musyawarah ("mufakat")

> "Bagaimana kalau kita bikin taman bermain di lahan kosong?"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Propose something new, then deliberate as community |
| **Lifecycle** | Ends with decision. Approved → spawns Trajectory A. Rejected → archived. |
| **Collaboration** | Community deliberation |
| **Privacy** | Public |
| **Operator** | Mufakat (context = 'proposal') |
| **Mood color** | Teal — balanced deliberation |

**Block composition:** `document` (proposal), `vote` (one per decision point), `computed` (consensus progress), `form` (community input), `list` (decision points)

AI breaks complex topics into fundamental agreement steps — each becomes a `VoteBlock`.

**Gateway:** Often converts to A (collective action) after consensus.

### 4.5 Trajectory L — Dispute Resolution / Mediation ("mediasi")

> "Ada sengketa batas tanah, perlu mediasi"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Conflict between parties, needs neutral ground |
| **Lifecycle** | Ends with documented agreement or escalation to formal process |
| **Collaboration** | Parties + mediator |
| **Privacy** | Partial |
| **Operator** | Mufakat (context = 'dispute') |
| **Mood color** | Violet — neutral mediation |

**Block composition:** `document` (party positions), `vote` (agreement points), `list` (mediation steps), `form` (position statements), `reference` (relevant documents)

**Gateway:** May escalate to Trajectory B (legal) if mediation fails.

### 4.6 Trajectory M — Ongoing Program ("program")

> "Jadwal ronda minggu ini: Senin Pak A, Selasa Pak B..."

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Coordinate a recurring community activity |
| **Lifecycle** | Ongoing (no end) |
| **Collaboration** | Rotating community participation |
| **Privacy** | Public |
| **Skill** | Jadwal |
| **Mood color** | Emerald — steady routine |

**Block composition:** `list` (schedule/rotation), `form` (attendance, swap requests), `computed` (participation rate), `display` (schedule overview)

---

## 5. Data Item Trajectories (One-off)

### 5.1 Trajectory C — Community Data / Survey ("data")

> "Harga cabai di pasar Menteng Rp 80.000/kg hari ini"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) |
| **Intent** | Share a factual observation — no action needed |
| **Collaboration** | Crowdsourced — others contribute their version |
| **Privacy** | Public |
| **Skill** | Catat |
| **Mood color** | Sky — informational |
| **Aggregates to** | Survey Board page |

### 5.2 Trajectory E — Private Vault ("vault")

> "Saya mau catat bahwa saya sudah peringatkan atasan"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item — private, only in owner's feed |
| **Intent** | Create a timestamped, tamper-evident personal record |
| **Collaboration** | Solo (until owner reveals) |
| **Privacy** | Private (encrypted, access-controlled) |
| **Skill** | Brankas |
| **Mood color** | Slate — private, sealed |

### 5.3 Trajectory G — Help Request ("bantuan")

> "Saya butuh bantuan hukum, tetangga klaim tanah saya"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) |
| **Intent** | Needs to be connected with the right person/resource |
| **Collaboration** | 1:1 match (requester + matched helper) |
| **Privacy** | Varies |
| **Skill** | Cocok |
| **Mood color** | Amber (lighter) — seeking warmth |

**Gateway:** Often becomes B (escalation) when institutional help needed.

### 5.4 Trajectory I — Celebration ("pencapaian")

> "Alhamdulillah jalan kita sudah diperbaiki!"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item — also triggered by Witness completion |
| **Intent** | Celebrate achievement, express gratitude, recognize contributors |
| **Collaboration** | Community response |
| **Privacy** | Public |
| **Skill** | Rayakan |
| **Mood color** | Yellow — bright celebration |

Also auto-triggered when any witness reaches "completed" status.

### 5.5 Trajectory J — Alert / Warning ("siaga")

> "Hati-hati ada penipuan bermodus kurir paket!"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item — with urgency treatment |
| **Intent** | Time-sensitive warning. Safety/security/health. |
| **Collaboration** | Community verification (confirm/deny) |
| **Privacy** | Public (broadcast) |
| **Skill** | Catat (with urgency detection) |
| **Mood color** | Red — urgent alert |
| **Aggregates to** | Alert Board page |

Has expiry mechanism — alerts auto-fade after danger period.

---

## 6. Gateway Transitions

Some trajectories convert into another once the real intent surfaces:

| Gateway | Often becomes... | Trigger |
|---------|-----------------|---------|
| K (Question) | A, B, or G | Deeper intent surfaces during orchestrator conversation |
| F (Proposal) | A (collective action) | All musyawarah points reach consensus |
| G (Help request) | B (escalation) | Institutional help needed |
| A (Collective action) | B (escalation) | Community realizes they can't solve it alone |
| D (Watchdog) | B (escalation) | Case stalls, community wants to push |
| J (Alert) | A (action) | Community decides to respond collectively |

---

## 7. Block Composition Summary

The 7 existing block types serve all witness trajectories:

| Block Type | Used By Trajectories |
|------------|---------------------|
| `list` | A (phases), B (escalation steps), D (timeline events), F (decision points), L (mediation steps), M (schedule) |
| `document` | B (evidence), D (case summaries), F (proposal), L (positions) |
| `form` | A (volunteer sign-up), B (evidence submission), M (attendance) |
| `computed` | A (progress %), B (evidence count), D (days tracked), F (consensus %), M (participation rate) |
| `vote` | F (decision points), L (mediation agreement), A (execution decisions) |
| `display` | D (screenshots), M (schedule overview) |
| `reference` | B (authority contacts), D (news articles), A↔B (escalation links) |

**No new block types needed.** Gaps are addressed by:
- Extending `ListItem` with optional `timestamp` + `actor` for timeline support
- Defining `DataItemRecord` for one-off trajectories
- Adding verification properties to data items (confirm/deny for alerts)

---

## 8. Property Matrices

### Ongoing Work — Witnesses

| # | Trajectory | Collab | Privacy | Lifecycle | Mood Color |
|---|-----------|--------|---------|-----------|------------|
| A | aksi | Full community | Public | Clear end | Amber |
| B | advokasi | Community + authority | Partial | Until resolved | Rose |
| D | pantau | Multi-watcher | Public | Until resolved | Indigo |
| F | mufakat | Deliberation | Public | Until decision → A | Teal |
| L | mediasi | Parties + mediator | Partial | Until resolved | Violet |
| M | program | Rotating | Public | Ongoing | Emerald |

### One-off — Data Items

| # | Trajectory | Collab | Privacy | Aggregates to | Mood Color |
|---|-----------|--------|---------|---------------|------------|
| C | data | Crowdsource | Public | Survey Board | Sky |
| E | vault | Solo | Private | Owner's vault | Slate |
| G | bantuan | 1:1 match | Varies | — | Amber (light) |
| I | pencapaian | Community | Public | Linked to witness | Yellow |
| J | siaga | Confirm/deny | Public | Alert Board | Red |

---

## 9. TypeScript Types

The trajectory model is implemented in frontend types at:

| Type | File | Purpose |
|------|------|---------|
| `TrajectoryType` | `types/card-enrichment.ts` | 11-value union type |
| `CardEnrichment` | `types/card-enrichment.ts` | AI-generated feed presentation |
| `Sentiment` | `types/card-enrichment.ts` | 7-value emotional mood |
| `FeedDataItem` | `types/feed.ts` | One-off data item feed card |
| `DataItemRecord` | `types/feed.ts` | Point-in-time data record |
| `OperatorResponse` | `types/operator.ts` | Canonical consultation envelope |
| `AksiPayload` | `types/operator.ts` | Aksi operator result |
| `MufakatPayload` | `types/operator.ts` | Mufakat operator result |
| `PantauPayload` | `types/operator.ts` | Pantau operator result |

---

## 10. Backward Compatibility

| Old Concept | New Concept | Migration |
|-------------|-------------|-----------|
| `TrackHint` (string) | `TrajectoryType` (11 values) | `track_hint` kept as optional metadata; `trajectory_type` is canonical |
| 5 tracks (tuntaskan...) | 11 trajectories (aksi...) | Old tracks map loosely: tuntaskan≈aksi, wujudkan≈mufakat, telusuri≈pantau, rayakan≈pencapaian, musyawarah≈mufakat |
| `track-colors.ts` | `trajectory-colors.ts` | Old util kept; new one is canonical |
| `getMoodColor(s, t)` | `getMoodColor(s, t, tj)` | Third param added (optional, backward compat) |
| `FeedStreamItem` = 2 kinds | `FeedStreamItem` = 3 kinds | `FeedDataItem` added to union |

---

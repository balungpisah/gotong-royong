# Trajectory Map — Scratchpad

> **Status:** Working draft v3 — added Card Enrichment + dynamic icon + mood color
> **Last updated:** 2026-02-22
>
> This document maps ALL human intent trajectories that enter through
> "Mulai di sini". Each trajectory describes a distinct pattern of
> **what the human wants** and **what structure the system produces**.
>
> This drives: operator design, data structures, and AI-00 triage behavior.

---

## Guiding Principle

Don't start from system labels ("tuntaskan", "kawal") and force humans into them.
Start from **what humans actually want to do** and let the system shape around that.

---

## Two Feed Categories

Not everything needs its own card type. The block system (7 block types) provides
the flexibility — different trajectories use different **block compositions** within
the same card structure.

The real distinction is between two fundamental feed categories:

### 1. Ongoing Work → Witness (`FeedWitnessItem`)

Has lifecycle, phases, members, progress. Lives in the main feed.
The block composition inside varies by trajectory type.

**Trajectories:** A, B, D, F, L, M

### 2. One-off / Data Point → Data Item (`FeedDataItem` — NEW)

Point-in-time contribution. No lifecycle, no phases, no ongoing progression.
Appears in the feed but can also be **aggregated into special pages**
(e.g., Survey Board for C, Alert Board for J).

**Trajectories:** C, E, G, I, J

### Not a feed item (baked into system):

- **K (Question)** → Orchestrator default behavior (pre-classification)
- **H (Offer help)** → Expertise Registry platform capability

---

## Trajectory List

---

### ONGOING WORK — Witness Trajectories

These produce witnesses with lifecycle and block compositions.

---

### A. Collective Action — "We can fix this ourselves"

> "Jalan di RT kami berlubang, kita bisa gotong royong perbaiki"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Community has a problem they can solve together |
| **Lifecycle** | Has a clear end (problem solved) |
| **Collaboration** | Full community |
| **Privacy** | Public |
| **Examples** | Road repair, drainage clearing, community cleanup, neighborhood watch setup |

**Block composition:**
- `list` → phases/tasks with status tracking
- `computed` → progress percentage, resource metrics
- `form` → volunteer sign-up, resource pledges
- `vote` → decision points within execution ("which vendor?")
- `reference` → linked related witnesses

**What AI asks:** What's the problem? Who's affected? What resources do you have? Has anyone tried before? What's the scope?

**What system produces:** `PathPlan` with community-executable phases + checkpoints.

---

### B. Advocacy & Escalation — "We need authority to act"

> "Pabrik buang limbah ke sungai, kami butuh bantuan BLH"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Problem needs government, legal, or institutional intervention |
| **Lifecycle** | Ends when authority acts (or community escalates further) |
| **Collaboration** | Community + external authority |
| **Privacy** | Public or partial (some evidence may be sensitive) |
| **Examples** | Environmental violation, public service failure, legal complaint, infrastructure demand |

**Block composition:**
- `document` → evidence compilation, formal complaint drafts
- `list` → escalation steps with status
- `form` → evidence submission (photos, documents)
- `computed` → evidence count, days since report, response tracking
- `reference` → linked to authority contacts, legal resources

**What AI asks:** What's the issue? Who is responsible? What evidence exists? Has it been reported before? Is there urgency?

**What system produces:** `PathPlan` with case-building phases. Phases are about **building a case and escalating**, not executing a solution.

---

### D. Watchdog / Monitor — "I'm tracking an external case"

> "Kasus korupsi dana desa di Kelurahan X, saya mau pantau perkembangannya"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Track something you don't control — accountability/oversight |
| **Lifecycle** | Ends when external case resolves |
| **Collaboration** | Multi-watcher (anyone can contribute updates) |
| **Privacy** | Public |
| **Examples** | Court case, government project, budget spending, election promises, institutional reform |

**Block composition:**
- `list` → timeline events (need: timestamp + actor extension — see Block Audit)
- `document` → evidence, news links, case summaries
- `display` → key media/screenshots
- `reference` → linked to formal records, news articles
- `computed` → days tracked, watcher count, activity frequency

**What AI asks:** What case? What stage is it at? Who are the parties? What's the latest development? What outcome do you hope for?

**What system produces:** Timeline-oriented witness. Events with timestamps. Evidence attachments. Status tracking (ongoing/stalled/resolved).

---

### F. Proposal / Musyawarah — "I have an idea, let's decide together"

> "Bagaimana kalau kita bikin taman bermain untuk anak-anak di lahan kosong?"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Propose something new, then deliberate as community |
| **Lifecycle** | Ends with decision. Approved → spawns Trajectory A. Rejected → archived. |
| **Collaboration** | Community deliberation |
| **Privacy** | Public |
| **Examples** | Park proposal, cooperative idea, community program, new initiative |

**Block composition:**
- `document` → proposal description, rationale
- `vote` → **one per decision point** (AI structures multi-faceted issues into fundamental agreement steps)
- `computed` → consensus progress (X/Y points agreed)
- `form` → community input, counter-proposals
- `list` → decision points checklist

**AI-structured musyawarah pattern:**

The hard part of musyawarah is multi-faceted issues where people argue everything
at once. AI breaks complex topics into fundamental agreement steps:

```
Proposal: "Bikin taman bermain di lahan kosong"

AI-structured deliberation:
  Step 1: "Apakah lahannya bisa dipakai?"        → VoteBlock → Agreed ✓
  Step 2: "Berapa relawan yang bersedia?"         → FormBlock → 14 orang ✓
  Step 3: "Estimasi biaya dan sumber dana?"       → VoteBlock → Campuran ✓
  Step 4: "Kapan mulai?"                          → VoteBlock → Maret W2 ✓
  Step 5: "Siapa koordinator?"                    → VoteBlock → Pak Ahmad ✓

ALL AGREED → Auto-spawn Trajectory A with pre-filled phases
```

Each step is a decision checkpoint. AI identifies what MUST be decided first
(fundamentals before details). People vote/discuss one point at a time.

**What AI asks:** What's the idea? Who would benefit? What would it take? What needs to be decided?

**What system produces:** Witness with `vote` blocks per decision point. On full consensus, transitions to Trajectory A.

**Gateway:** Often converts to A (collective action) after consensus.

---

### L. Dispute Resolution / Mediation — "We have a conflict"

> "Ada sengketa batas tanah antara Pak A dan Pak B, perlu mediasi"

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Conflict between parties, needs neutral ground |
| **Lifecycle** | Ends with documented agreement or escalation to formal process |
| **Collaboration** | Parties + mediator (RT head, community elder, or professional) |
| **Privacy** | **Partial** (parties' positions may be sensitive) |
| **Examples** | Land dispute, noise complaint, shared resource conflict, neighbor dispute, inheritance |

**Block composition:**
- `document` → party positions, context/history
- `vote` → agreement on mediation points (like F's musyawarah pattern)
- `list` → mediation steps with status
- `form` → position statements from each party
- `reference` → linked to relevant documents, witnesses

**What AI asks:** Who are the parties? What's the dispute about? Has mediation been attempted? Is there a preferred mediator?

**What system produces:** Mediation record with parties, positions, and AI-structured agreement points (similar to F's step-by-step voting). May escalate to Trajectory B (legal) if mediation fails.

---

### M. Ongoing Program — "We have a recurring activity"

> "Jadwal ronda minggu ini: Senin Pak A, Selasa Pak B..."

| Property | Value |
|----------|-------|
| **Feed category** | Witness (ongoing work) |
| **Intent** | Coordinate a recurring community activity |
| **Lifecycle** | Ongoing (no end) |
| **Collaboration** | Rotating community participation |
| **Privacy** | Public |
| **Examples** | Ronda (night patrol), kerja bakti schedule, arisan, posyandu, weekly cleanup |

**Block composition:**
- `list` → schedule / rotation roster
- `form` → attendance tracking, swap requests
- `computed` → participation rate, next rotation
- `display` → schedule overview

**What AI asks:** What activity? What's the schedule pattern? Who participates? Any rotation rules?

**What system produces:** Program witness with recurring schedule blocks. Participation tracker.

---

### ONE-OFF — Data Item Trajectories

These produce point-in-time records. No lifecycle, no phases.
Appear in feed AND can aggregate into dedicated pages.

---

### C. Community Data / Survey — "I'm stating a fact"

> "Harga cabai di pasar Menteng Rp 80.000/kg hari ini"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) |
| **Intent** | Share a factual observation — no action needed |
| **Lifecycle** | No end — data points accumulate. Value grows with aggregation. |
| **Collaboration** | Crowdsourced — others click to input their version |
| **Privacy** | Public |
| **Examples** | Prices, weather conditions, infrastructure status, air quality, water quality |

**Feed behavior:**
- Appears as a **survey card** in feed: shows the data point + "Tambah Data Saya" button
- Others tap it → contribute their data point for their area/time
- Multiple data items on the same topic aggregate into a survey thread
- **Aggregatable to: Survey Board page** (maps, charts, trends, comparisons)

**What AI asks:** What are you reporting? Where? Any proof (photo, receipt)? How current is this?

**What system produces:** Data point record (claim + location + timestamp + proof + category). Linkable to other data points on same topic.

---

### E. Private Vault / Sealed Record — "This is my proof"

> "Saya mau catat bahwa saya sudah peringatkan atasan soal ini"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) — private, only in owner's feed |
| **Intent** | Create a timestamped, tamper-evident personal record |
| **Lifecycle** | Indefinite — revealed when owner decides |
| **Collaboration** | Solo (until owner reveals) |
| **Privacy** | **Private** (encrypted, access-controlled) |
| **Examples** | Action record, ownership proof, witnessed event, personal testimony, agreement |

**Feed behavior:**
- Appears ONLY in the owner's personal feed
- Sealed card — shows title + timestamp, content locked
- Can be "unsealed" (revealed) to specific people or publicly when needed

**What AI asks:** What are you recording? When did it happen? Who was involved? Any supporting evidence? (Minimal probing.)

**What system produces:** Sealed timestamped document. Encrypted. Access-controlled. Maps to `vault` entry route.

---

### G. Help Request — "I need help"

> "Saya butuh bantuan hukum, tetangga klaim tanah saya"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) |
| **Intent** | Can't do it alone. Needs to be connected with the right person/resource. |
| **Lifecycle** | Ends when help is provided or requester withdraws |
| **Collaboration** | 1:1 match (requester + matched helper) |
| **Privacy** | Varies (some requests are sensitive) |
| **Examples** | Legal help, medical guidance, administrative process (KTP/BPJS), conflict mediation |

**Feed behavior:**
- Appears as a request card in feed (if public)
- System auto-matches from expertise registry
- Matched experts get notification
- Card shows match status

**What AI asks:** What kind of help? How urgent? Have you tried anything? Any constraints?

**What system produces:** Help request matched against expertise registry. May escalate to Trajectory B if institutional help needed.

**Gateway:** Often becomes B (escalation) when the help needed is institutional.

---

### I. Celebration / Recognition — "Something good happened!"

> "Alhamdulillah jalan kita sudah diperbaiki! Terima kasih semua yang ikut"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) — also triggered by Witness completion |
| **Intent** | Celebrate achievement, express gratitude, recognize contributors |
| **Lifecycle** | Point-in-time (no ongoing action) |
| **Collaboration** | Community response (congrats, appreciation) |
| **Privacy** | Public |
| **Examples** | Project completion, individual achievement, community milestone, thank-you |

**Feed behavior:**
- Appears as an achievement card in feed
- Linked to completed witness (if applicable)
- **Also auto-triggered** when any witness reaches "completed" status → system prompts celebration

**What AI asks:** What happened? Who contributed? Is this linked to an existing witness? (Minimal probing.)

**What system produces:** Achievement record. Linked to completed witness. Triggers credit accreditation (AI-09).

**Note:** Also serves as a **Completion Event** — baked into witness lifecycle. When any ongoing witness completes, the system can prompt: "Mau rayakan dan akui kontributor?"

---

### J. Alert / Warning — "Danger! People need to know NOW"

> "Hati-hati ada penipuan bermodus kurir paket di area Menteng!"

| Property | Value |
|----------|-------|
| **Feed category** | Data Item (one-off) — with urgency treatment |
| **Intent** | Time-sensitive warning. Safety/security/health. |
| **Lifecycle** | Short — expires when danger passes |
| **Collaboration** | Community verification (confirm/deny) |
| **Privacy** | Public (broadcast) |
| **Examples** | Scam warning, flood alert, road hazard, disease outbreak, crime pattern |

**Feed behavior:**
- Appears as a **high-attention card** in feed (visual urgency treatment — red, prominent)
- Structurally similar to C (stating a fact) but with urgency + verification
- Others can **confirm or deny** (truthfulness tracking, different from voting)
- Tandang tracks alert accuracy per user over time (cry wolf → lower credibility)
- Expiry mechanism — alerts auto-fade after danger period
- **Aggregatable to: Alert Board page** (active community alerts by area + verification status)
- Severe cases map to `siaga` entry route

**What AI asks:** What's the danger? Where exactly? How current? Any evidence? How urgent? (Fast triage — minimize turns.)

**What system produces:** Alert record. Location-tagged. Verification mechanism. Expiry timer.

---

### BAKED INTO SYSTEM — Not Trajectories

---

### ~~H. Resource Offering~~ → Expertise Registry + AI Matching

> See Platform Capabilities section below.

### K. Question / Inquiry → Orchestrator Default Behavior

> "Siapa yang bertanggung jawab untuk perbaikan jalan di RT kita?"

Not a trajectory — it's what the orchestrator does BEFORE classifying into
a trajectory. Every triage conversation starts as a kind of question/exploration.

If the question has a simple answer, the orchestrator answers directly using
the **Community Knowledge Base** (FAQ, procedures, contacts).

If it reveals a deeper intent, the orchestrator reroutes to the appropriate
trajectory. "Who's responsible?" → "Nobody's done anything" → Trajectory A or B.

---

## Block Composition Summary

The 7 existing block types serve all witness trajectories:

| Block Type | Description | Used By Trajectories |
|-----------|-------------|---------------------|
| `list` | Checklist items with status | A (phases), B (escalation steps), D (timeline events), F (decision points), L (mediation steps), M (schedule) |
| `document` | Rich text sections | B (evidence), D (case summaries), F (proposal), L (positions) |
| `form` | Input fields | A (volunteer sign-up), B (evidence submission), C (data input), G (request details), M (attendance) |
| `computed` | Metrics/progress | A (progress %), B (evidence count), D (days tracked), F (consensus %), M (participation rate) |
| `vote` | Structured voting | F (decision points!), L (mediation agreement), A (execution decisions) |
| `display` | Media/content | D (screenshots), I (achievement visuals), J (alert visuals) |
| `reference` | Links to other witnesses | B (authority contacts), D (news articles), A↔B (escalation links) |

### Block Audit — Gaps Found

| Need | Current Gap | Proposed Solution |
|------|------------|-------------------|
| **Timeline events** | `list` items lack timestamp + actor fields | Extend `ListItem` with optional `timestamp?: string` and `actor?: string`. Timeline is a list display variant, not a new block type. |
| **Data point with proof** | No structured "claim + evidence + location" unit | `form` + `display` combo works for input. For the data item card type, define a `DataPointRecord` structure (claim, location, timestamp, proof_url, category). |
| **Alert verify/deny** | `vote` is for decisions, not binary truth verification | Add a lightweight verification mechanism to data items: `{ confirms: number, denies: number, user_verified?: 'confirm' \| 'deny' }`. Not a block — a data item property. |
| **Sealed/tamper-evident** | `document` has no sealing semantics | Add `sealed?: boolean`, `sealed_at?: string`, `content_hash?: string` to vault-type data items. Not a block change — a data item property. |
| **Aggregation views** | No block needed | Rendering concern — platform aggregates data items by category, frontend renders as map/chart/trend. |

**Conclusion:** No new block types needed. Extend `ListItem` with optional timestamp/actor. The gaps are in the **data item** type (new feed category) and its properties, not in blocks.

---

## Property Matrix (Revised)

### Ongoing Work — Witnesses

| # | Trajectory | Block Composition | Collab | Privacy | Lifecycle |
|---|-----------|-------------------|--------|---------|-----------|
| A | Solve together | list + computed + form + vote | Full community | Public | Clear end |
| B | Need authority | document + list + form + computed + reference | Community + authority | Partial | Until resolved |
| D | Watch/monitor | list (timeline) + document + display + computed | Multi-watcher | Public | Until resolved |
| F | Musyawarah/Proposal | vote + document + computed + form + list | Deliberation | Public | Until decision → A |
| L | Resolve conflict | document + vote + list + form + reference | Parties + mediator | Partial | Until resolved |
| M | Ongoing program | list + form + computed + display | Rotating | Public | Ongoing |

### One-off — Data Items

| # | Trajectory | Structure | Collab | Privacy | Aggregates to |
|---|-----------|-----------|--------|---------|---------------|
| C | State a fact | Data point (claim + proof) | Crowdsource ("add yours") | Public | **Survey Board** page |
| E | Private record | Sealed document | Solo | **Private** | Owner's vault |
| G | Help request | Request + match | 1:1 match | Varies | — |
| I | Celebrate | Achievement record | Community response | Public | Linked to completed witness |
| J | Alert/warning | Urgent data point + verify | Confirm/deny | Public | **Alert Board** page |

---

## Gateway Trajectories

Some trajectories convert into another once the real intent surfaces:

| Gateway | Often becomes... | Trigger |
|---------|-----------------|---------|
| K (Question) | A, B, or G | Deeper intent surfaces during orchestrator conversation |
| F (Proposal) | A (collective action) | All musyawarah points reach consensus |
| G (Help request) | B (escalation) | Institutional help needed, not just individual expert |
| A (Collective action) | B (escalation) | Community realizes they can't solve it alone |
| D (Watchdog) | B (escalation) | Case stalls, community wants to push |
| J (Alert) | A (action) | Community decides to respond collectively |

---

## Platform Capabilities (Not Trajectories)

System-level features that serve multiple trajectories:

### Expertise Registry + AI Matching

Replaces old Trajectory H ("I can help"):

1. **Declare**: Users declare skills in profile (free-text or ESCO codes)
2. **Rate**: Tandang builds reputation per skill through interactions
3. **Match**: AI identifies expertise needs when generating phases
4. **Notify**: System notifies matched experts. Proven experts surface first.
5. **Recruit**: Expert joins the witness/phase as a contributor

Used by: A, B, G, L, M (any trajectory that generates phases or needs expert matching).

### Community Services Marketplace (Future Layer)

Natural extension of Expertise Registry:

```
Layer 0: Declare skills (profile)           ← v1, simple
Layer 1: Tandang rates skills               ← v1, passive
Layer 2: AI matches needs to experts        ← v1, automated
Layer 3: Contract/agreement layer           ← later, formalized
Layer 4: Compensation + marketplace         ← future, community economy
```

Contract layer uses vault/sealed record pattern (Trajectory E) for legal backing.
Each layer adds weight without changing the one below.

### Community Knowledge Base

Feeds the orchestrator's default behavior (replaces Trajectory K):
- FAQ, common procedures, responsibility maps
- "Siapa yang urus jalan?" → direct answer + offer to create witness

### Truthfulness / Verification Engine

Used by data item trajectories:
- C: accuracy of data contributions (rated over time by Tandang)
- J: confirm/deny mechanism for alerts (community verification)
- Tandang tracks per-user accuracy scores

### Completion Event Engine

Baked into witness lifecycle (partially replaces Trajectory I):
- When any witness reaches "completed" status → prompt celebration
- Auto-generate achievement record
- Trigger credit accreditation (AI-09)
- Standalone celebrations (not linked to a witness) still enter as data items

### Tandang Reputation Engine

Cross-cutting system that rates users across dimensions:
- Expertise ratings (per declared skill)
- Reliability score (follow-through on commitments)
- Community trust (peer endorsements)
- Accuracy (data contributions — C, alert truthfulness — J)
- Deliberation quality (voting participation — F, L)

### Duplicate Detection (AI-03)

Platform capability across all trajectories:
- "Is someone else already reporting this?"
- "Is there an existing witness that covers this issue?"

---

## Operator & Skill Mapping

### Architecture: 1 Orchestrator + 3 Operators + 11 Skills

```
                         User message
                              │
                              ▼
                    ┌───────────────────┐
                    │   ORCHESTRATOR    │
                    │                   │
                    │  • Converses      │
                    │  • Classifies     │  ← inline, no separate operator
                    │  • Routes         │
                    │  • Synthesizes    │
                    └──┬─────┬─────┬───┘
                       │     │     │
            ┌──────────┘     │     └──────────┐
            ▼                ▼                ▼
       ┌─────────┐    ┌──────────┐    ┌──────────┐
       │  AKSI   │    │ MUFAKAT  │    │  PANTAU  │
       │  A + B  │    │  F + L   │    │    D     │
       │ Problem │    │ Decision │    │ Timeline │
       │→ Phases │    │→ Steps   │    │→ Events  │
       └─────────┘    └──────────┘    └──────────┘

       ┌──────────────────────────────────────────┐
       │              SKILLS (shared)              │
       │                                           │
       │  Trajectory:  Catat · Brankas · Cocok     │
       │               Jadwal · Rayakan            │
       │                                           │
       │  Cross-cut:   PII Redaction · Vault Det.  │
       │               Urgency Scoring · Duplicate │
       │               Location · Confidence       │
       │               Card Enrichment ★            │
       └──────────────────────────────────────────┘
```

### Operators (LLM reasoning required)

Grouped by **reasoning pattern**, not 1:1 with trajectories:

| Operator | Reasoning Pattern | Trajectories | Output |
|----------|------------------|--------------|--------|
| **Aksi** | "What's wrong? Can we fix it? How?" | A + B | PathPlan (action phases OR escalation phases) |
| **Mufakat** | "What must we agree on? Step by step." | F + L | Decision steps → VoteBlocks |
| **Pantau** | "What's happening? What to track?" | D | Timeline structure + tracking points |

**Why 3, not 6:**
- **Aksi** handles both A and B because the KEY question (can community self-solve?) is part of the operator's assessment. One operator, two output patterns.
- **Mufakat** handles both F and L because the mechanism (break complex issue into decision steps) is identical. Context differs (proposal vs dispute), not process.
- **Pantau** stands alone — observation is fundamentally different from action or deliberation.

### Skills (lightweight, template/rule-driven)

**Trajectory-specific skills:**

| Skill | Handles | What it does |
|-------|---------|-------------|
| **Catat** (Record) | C + J | Structures data point: claim + location + timestamp + proof + category. Detects urgency (C = normal, J = alert). |
| **Brankas** (Vault) | E | Structures sealed record: what + when + who + proof. Sets privacy, generates hash. |
| **Cocok** (Match) | G | Identifies help type. Queries expertise registry. Returns ranked matches. |
| **Jadwal** (Schedule) | M | Structures recurring program: activity + frequency + participants + rotation. |
| **Rayakan** (Celebrate) | I | Generates achievement record. Links to completed witness. Identifies contributors. |

**Cross-cutting skills:**

| Skill | Purpose | Invoked |
|-------|---------|---------|
| PII Redaction | Mask personal data | Every turn |
| Vault Detection | Detect privacy signals | Every turn |
| Urgency Scoring | Detect emergency signals | Every turn |
| Location Enrichment | Resolve coords to RT/RW/Kel | When location mentioned |
| Confidence Labeling | Human-readable confidence | When confidence changes |
| Duplicate Detection | Check existing witnesses/data | When enough context |
| **Card Enrichment** | Generate title, icon, hook_line, body, sentiment, tags | At triage completion (status = ready) |

---

## Consultation Protocol

### Canonical Envelope — Same for All Operators

```typescript
interface OperatorResponse {
  status: 'need_more' | 'ready';

  // Universal — orchestrator always understands these
  checklist: ChecklistItem[];     // what's filled, what's missing
  questions?: string[];           // suggested next questions

  // Specialized — operator-specific, only when status = 'ready'
  payload: AksiPayload | MufakatPayload | PantauPayload;
}

interface ChecklistItem {
  field: string;        // e.g., "problem_scope", "stakeholders"
  filled: boolean;
  value?: string;       // summary of captured info
}
```

**Why canonical envelope + specialized payload:**

| Layer | Same for all | Different per operator |
|-------|-------------|----------------------|
| Orchestrator reads | `status`, `checklist`, `questions` | (ignores payload details) |
| System stores | — | `payload` → mapped to blocks / PathPlan |
| Adding new operator | No orchestrator changes | Just define new payload type |

### Specialized Payloads

```typescript
// ─── Aksi: problem → phases ────────────────────────────

interface AksiPayload {
  trajectory: 'A' | 'B';         // self-solve or escalate
  path_plan: PathPlan;            // existing type
}

// Checklist fields:
// - problem_scope, who_affected, resources_available,
//   prior_attempts, self_solvable (KEY: determines A vs B)


// ─── Mufakat: complex issue → decision steps ───────────

interface MufakatPayload {
  context: 'proposal' | 'dispute';
  decision_steps: DecisionStep[];
  on_consensus?: 'spawn_aksi';   // F→A gateway trigger
}

interface DecisionStep {
  question: string;               // "Apakah lahannya bisa dipakai?"
  options?: string[];             // ["Setuju", "Tidak", "Perlu survey"]
  rationale: string;              // why this must be decided
  order: number;                  // fundamentals first
}

// Each DecisionStep → VoteBlock in the witness
// All agreed → spawn Trajectory A (if proposal)
//           → sealed agreement (if dispute)


// ─── Pantau: case → timeline ───────────────────────────

interface PantauPayload {
  case_type: string;              // "legal", "political", "environmental"
  timeline_seed: TimelineEvent[];
  tracking_points: string[];      // what to watch for next
}

interface TimelineEvent {
  event: string;
  date: string;
  source: 'user' | 'news' | 'official';
  evidence_url?: string;
}
```

### Orchestration Flow Example

```
Turn 1:
  User → "Jalan rusak 3 bulan, sudah lapor RT tidak ada respons"
  Orchestrator:
    Skills: PII ✓, Urgency ✓ (not urgent), Vault ✓ (not vault)
    Classify inline: signal = 'problem', confidence = 0.5
    Route → Aksi operator
  Aksi returns: {
    status: 'need_more',
    checklist: [
      { field: 'problem_scope',    filled: true,  value: 'Jalan rusak 3 bln' },
      { field: 'who_affected',     filled: false },
      { field: 'prior_attempts',   filled: true,  value: 'Lapor RT, no response' },
      { field: 'self_solvable',    filled: false }
    ],
    questions: ["Berapa warga terdampak?", "Bisa diperbaiki sendiri atau perlu dinas?"]
  }
  Orchestrator → User: "Berapa warga yang terdampak? Dan menurut Anda,
    ini bisa ditangani warga bersama atau perlu bantuan dinas PU?"

Turn 2:
  User → "Sekitar 30 KK, kita bisa tambal sendiri kalau ada dana"
  Orchestrator:
    Skills: PII ✓, Duplicate ✓ (no match)
    Forward to Aksi
  Aksi returns: {
    status: 'ready',
    checklist: [ ...all filled... ],
    payload: {
      trajectory: 'A',
      path_plan: {
        title: "Perbaikan Jalan Jl. Mawar",
        phases: [
          { title: "Dokumentasi & Dukungan", ... },
          { title: "Pengumpulan Dana & Material", ... },
          { title: "Pelaksanaan", ... },
          { title: "Verifikasi", ... }
        ]
      }
    }
  }
  Orchestrator → User: "Triase selesai! Jalur: Aksi Bersama."
  bar_state → ready, proposed_plan → PathPlan
```

---

## Card Enrichment — AI-Generated Feed Presentation

Every trajectory — whether it becomes a Witness or a Data Item — needs to look
**compelling** in the feed. Raw user input is messy. AI transforms it into
engaging, clear, safe content that drives community engagement.

### The Principle

- **Icon** = tied to the **case content**, not the trajectory (road, scales, tree, flame...)
- **Color** = tied to the **trajectory type** as mood (amber for action, indigo for monitoring, teal for deliberation...)
- **Title** = AI-crafted, factual but compelling. Never sensationalist.
- **Hook line** = one sentence that makes you want to tap

### CardEnrichment Interface

```typescript
interface CardEnrichment {
  // ─── Visual identity ───────────────────────────────
  icon: string;                    // AI-selected, content-specific
                                   // e.g., "road", "factory", "scales", "shield",
                                   // "tree", "water", "house", "megaphone"
  trajectory_type: TrajectoryType; // drives mood color, NOT icon

  // ─── Engaging text ─────────────────────────────────
  title: string;                   // AI-crafted, max 80 chars
                                   // factual + specific + compelling
  hook_line: string;               // one-liner that draws attention
  pull_quote?: string;             // most striking phrase from user's story
  body: string;                    // 2-3 sentence summary

  // ─── Sentiment & intensity ─────────────────────────
  sentiment: Sentiment;            // existing type: 7 values
  intensity: number;               // 1-5 scale

  // ─── Discovery ─────────────────────────────────────
  entity_tags?: EntityTagSuggestion[];  // AI-suggested tags for discoverability
}

type TrajectoryType =
  | 'aksi'        // A: collective action      → amber / warm
  | 'advokasi'    // B: advocacy/escalation     → rose / urgent
  | 'pantau'      // D: watchdog/monitor        → indigo / cool
  | 'mufakat'     // F: proposal/musyawarah     → teal / balanced
  | 'mediasi'     // L: dispute resolution      → violet / neutral
  | 'program'     // M: ongoing program         → emerald / steady
  | 'data'        // C: community data/survey   → sky / informational
  | 'vault'       // E: private sealed record   → slate / private
  | 'bantuan'     // G: help request            → amber / seeking
  | 'pencapaian'  // I: celebration             → yellow / bright
  | 'siaga';      // J: alert/warning           → red / urgent

interface EntityTagSuggestion {
  label: string;           // "Jl. Mawar", "RT 05", "Dinas PU"
  entity_type: EntityType; // existing type: lingkungan, topik, etc.
  confidence: number;
}
```

### Trajectory → Mood Color Mapping

The trajectory type determines the card's **color treatment** — background tint,
accent stripe, badge color. This creates visual rhythm in the feed where you can
*feel* what kind of activity it is before reading.

| Trajectory Type | Mood | Color Token | Feel |
|-----------------|------|-------------|------|
| `aksi` | Warm determination | `amber` | "Let's do this" |
| `advokasi` | Urgent advocacy | `rose` | "This needs attention" |
| `pantau` | Cool observation | `indigo` | "Watching closely" |
| `mufakat` | Balanced deliberation | `teal` | "Let's decide together" |
| `mediasi` | Neutral mediation | `violet` | "Finding middle ground" |
| `program` | Steady routine | `emerald` | "Keeping it going" |
| `data` | Informational | `sky` | "Here's what I see" |
| `vault` | Private, sealed | `slate` | "For my records" |
| `bantuan` | Seeking warmth | `amber` (lighter) | "I need help" |
| `pencapaian` | Bright celebration | `yellow` | "We did it!" |
| `siaga` | Red alert | `red` | "Danger — act now" |

### Dynamic Icon Selection

Icons are **case-specific**, selected by the Card Enrichment skill based on
the actual content of the story — not the trajectory type.

```
Same trajectory (aksi), different icons:

  "Jalan berlubang di RT 05"          → icon: "construction"
  "Pohon tumbang halangi jalan"        → icon: "tree-pine"
  "Saluran air tersumbat"              → icon: "droplets"
  "Atap balai warga bocor"             → icon: "house"

Same trajectory (pantau), different icons:

  "Kasus korupsi dana desa"           → icon: "scale"
  "Proyek jembatan mangkrak"          → icon: "building-2"
  "Janji caleg soal fasilitas"         → icon: "clipboard-list"
```

**Strategy: LLM-selected from full Lucide set.**

The project already uses `@lucide/svelte` (1400+ icons). Instead of
maintaining a curated subset, the Card Enrichment skill's prompt simply
instructs the LLM: *"Select the Lucide icon name that best represents
this case's content."* The LLM knows the Lucide library and picks
the closest match naturally.

**Frontend dynamic resolver:**

```svelte
<script>
  import * as icons from '@lucide/svelte';

  function resolveIcon(name: string) {
    // "tree-pine" → "TreePine"
    const pascal = name.split('-')
      .map(s => s[0].toUpperCase() + s.slice(1))
      .join('');
    return icons[pascal] ?? icons.HelpCircle;
  }
</script>
```

**Fallback chain:**
1. LLM-selected Lucide icon name → resolve to component
2. If name doesn't match → trajectory-type default icon
3. If all else fails → `HelpCircle` (generic)

### Title Generation Rules

| Rule | Example |
|------|---------|
| **Factual** — no exaggeration | "Jalan Berlubang Jl. Mawar, 30 KK Terdampak" not "JALAN HANCUR!!!" |
| **Specific** — include location/scale | "Saluran Air Tersumbat di Gang 3" not "Ada masalah air" |
| **Compelling** — invoke care | "3 Bulan Tanpa Respons: Warga Jl. Mawar Turun Tangan" |
| **Safe** — no names, no accusations | Never include personal names or unverified claims in title |
| **Localized** — natural Bahasa Indonesia | Match community register, not formal bureaucratic language |

### Where Card Enrichment Fits

Card Enrichment is a **cross-cutting skill** invoked at triage completion,
just before the final `TriageResult` is returned:

```
Orchestrator → Operator (ready) → Card Enrichment skill → final response
                                        │
                                        ├─ Reads: user's story + operator payload
                                        ├─ Generates: title, hook_line, body, icon, sentiment
                                        └─ Attaches: to TriageResult alongside proposed_plan
```

The enrichment data flows to the platform, which stores it alongside the
witness/data-item. The frontend renders it directly — no client-side
text processing needed.

---

## Open Questions

- [ ] Can a single witness **shift** trajectory mid-lifecycle? (e.g., A → B)
- [ ] How do gateway transitions work? (new witness? morph existing one?)
- [ ] What properties does the new `FeedDataItem` type need?
- [ ] Should D (watchdog) timeline use extended `ListItem` or a new structure?
- [ ] What's the minimum viable set for launch? Which trajectories are v1?
- [ ] How does aggregation work for C (survey board) and J (alert board)?
- [ ] Should `ListItem` get `timestamp` and `actor` extensions now or later?
- [ ] How does Card Enrichment interact with `FeedDataItem`? (same skill, different card shape?)
- [ ] Should mood color replace the old `track-colors.ts` mapping entirely?
- [ ] Dynamic Lucide resolver: import-all vs lazy-load tradeoff for bundle size?

---

## Summary

```
SYSTEM ARCHITECTURE

1 Orchestrator     Converses, classifies inline, routes, synthesizes
3 Operators        Aksi (A+B), Mufakat (F+L), Pantau (D)
5 Traj. Skills     Catat (C+J), Brankas (E), Cocok (G), Jadwal (M), Rayakan (I)
7 Cross-cut Skills PII, Vault Det., Urgency, Location, Confidence, Duplicate,
                   Card Enrichment ★

FEED PRESENTATION

Icon               Content-specific (road, scales, tree...) — AI selects per case
Color              Trajectory mood (amber=aksi, indigo=pantau, teal=mufakat...)
Title              AI-crafted: factual + specific + compelling + safe
Hook line          One sentence that makes you want to tap

FEED CATEGORIES

FeedWitnessItem    Ongoing work with lifecycle + block compositions (A,B,D,F,L,M)
FeedDataItem       One-off contributions, aggregatable to pages (C,E,G,I,J)
FeedSystemItem     System cards — tips, milestones, prompts (existing)

CONSULTATION PROTOCOL

Canonical envelope  { status, checklist[], questions[] }  — same for all
Specialized payload { AksiPayload | MufakatPayload | PantauPayload } — per operator
Card enrichment     { icon, trajectory_type, title, hook_line, body, sentiment }

PLATFORM CAPABILITIES

Expertise Registry, Marketplace (future), Knowledge Base, Truthfulness Engine,
Completion Events, Tandang Reputation, Duplicate Detection
```

## Next Steps

- [ ] Define `FeedDataItem` TypeScript type for one-off trajectories
- [ ] Define `OperatorResponse`, `ChecklistItem`, and payload types in TypeScript
- [ ] Define `CardEnrichment`, `TrajectoryType`, `EntityTagSuggestion` in TypeScript
- [ ] Build dynamic Lucide icon resolver component (string → component + fallback)
- [ ] Define trajectory → mood color mapping (replace old `track-colors.ts`)
- [ ] Extend `ListItem` with optional `timestamp` + `actor` for timeline support
- [ ] Define entry signal patterns per trajectory (what keywords trigger classification)
- [ ] Decide minimum viable trajectory set for v1 launch
- [ ] Formalize this scratchpad into the official AI spec when stable

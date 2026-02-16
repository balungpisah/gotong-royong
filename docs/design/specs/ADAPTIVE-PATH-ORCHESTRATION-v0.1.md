# Gotong Royong — Adaptive Path Orchestration Flow v0.1

## Status
Proposed: 2026-02-16
Companion to: `ADAPTIVE-PATH-SPEC-v0.1.md` (data model), `prd-adaptive-path-guidance.md` (requirements)

## What This Document Is

The Adaptive Path Spec defines **what** a plan looks like (data model).
This document defines **how** the plan gets created, evolves, and completes — who does what at each step.

The goal: a warga tells their story. The system does the rest. Anyone can use it.

---

## 1. Conceptual Foundation

This orchestration draws from three established frameworks, adapted for community coordination:

**Adaptive Case Management (ACM)** — Each community case is unique. The plan emerges from context, not a template. Humans own decisions. The system supports, not dictates.

**Hierarchical Task Network (HTN)** — The LLM doesn't freestyle. It selects from a library of known phase patterns and composes them based on case needs. Domain knowledge is encoded in the patterns, not improvised per case.

**Case Management Model & Notation (CMMN)** — Plans have discretionary items (things the LLM can propose but aren't required), sentries (entry/exit conditions), and milestones (verifiable checkpoints). The plan is a living document, not a waterfall.

### 1.1 Core Principle

> **The LLM is a drafter, not a decider.**
> It proposes the best plan it can. The community (through privileged editors or consensus) accepts, modifies, or rejects. Speed-oriented users accept quickly. Detail-oriented users scrutinize. Both are valid.

---

## 2. The Four Layers

### Layer 1: Invariant Principles

These rules are always enforced. The LLM cannot violate them. The system enforces them at the API level.

| # | Principle | Enforcement |
|---|-----------|-------------|
| P1 | Every plan must define what "done" looks like | LLM must include a completion phase. API rejects plans without one. |
| P2 | Every plan must assign accountability (PIC) | LLM includes "Assign PIC" as a checkpoint in the first phase. |
| P3 | Human edits are sovereign | `locked_fields` enforced at write time. LLM receives locks in context. |
| P4 | Suggest, don't overwrite | AI proposals are stored as `plan_suggestion`, never auto-applied (except via consent timeout — see Section 3.5). |
| P5 | Minimal viable plan first | LLM generates the full path as headers, details only near-term phases. |
| P6 | Every checkpoint must be verifiable | LLM must phrase checkpoints as observable outcomes, not activities. |
| P7 | Progressive elaboration | Near-term phases are detailed (checkpoints listed). Future phases are visible as headers (title + objective) to give users a sense of the journey ahead. Elaborated when activated. |

### Layer 2: Diagnostic Assessment

When the LLM reads the user's story, it assesses the case along these dimensions. Each dimension maps to phase patterns (Layer 3).

| Dimension | Question the LLM asks itself | Signal source |
|-----------|------------------------------|---------------|
| **Complexity** | Is this simple (1–2 people, days) or complex (many stakeholders, weeks)? | Story length, number of actors, scope |
| **Resources** | Does this need funding, materials, or external assets? | Mentions of money, materials, "butuh dana" |
| **Knowledge gap** | Is there something unknown that needs investigation first? | Questions in the story, "siapa yang...", "kenapa..." |
| **Authority** | Does this need escalation to government, police, or institutions? | Mentions of RT/RW, pemerintah, polisi, dinas |
| **Consensus** | Does the community need to formally decide something? | "bagaimana kita...", "pilihan kita...", policy language |
| **Urgency** | How time-sensitive is this? | "segera", "darurat", timeline pressure |
| **Celebration** | Is there an achievement to recognize? | "berhasil", "sudah selesai", positive language |
| **Privacy** | Are there sensitive or personal elements? | Handled by vault detection (AI-00 4.7a), not path planning |

The LLM does not expose this assessment to the user. It reasons silently and selects patterns.

### Layer 3: Phase Pattern Graph

Phase patterns are the building blocks of any plan. Rather than prescribing fixed sequences (which would prime the LLM into reproducing old track structures), patterns are modeled as **independent nodes in a weighted graph**. Edges represent "commonly followed by" relationships with affinity weights.

The LLM does not receive a prescribed sequence. It receives the pattern library as independent blocks with activation conditions and decides composition based on case diagnostics (Layer 2).

#### 3.1 Pattern Definitions

| Pattern | Activation condition | Default checkpoints | Governance | UI components |
|---------|---------------------|---------------------|------------|---------------|
| **Diskusi** (Discussion) | Almost always first | Gather perspectives, identify stakeholders, assign PIC | Consent (24h) | Percakapan chat, summary card |
| **Investigasi** (Investigation) | Knowledge gap detected | Form hypothesis, gather evidence, synthesize findings | Consent or vote depending on scope | Hypothesis cards, evidence board |
| **Perencanaan** (Planning) | Complex case (>3 tasks expected) | Define scope, break into tasks, estimate effort, assign roles | Vote if ≥5 participants | Papan Gotong Royong (task board) |
| **Galang** (Resource Pooling) | Funding or materials needed | Set target, open collection, track progress, close & report | Vote (48h) | Galang financial tracker (6 protected fields) |
| **Pelaksanaan** (Execution) | Tasks need doing | Per-task checkpoints (assigned, in-progress, done, verified) | Consent per task; vote for phase completion | Task board, progress tracker |
| **Verifikasi** (Verification) | Quality/outcome assurance needed | Peer review, evidence submission, challenge window | Vote + evidence (72h) | Verification panel, evidence upload |
| **Keputusan** (Governance) | Formal community decision needed | Present options, deliberate, vote, record decision | Weighted vote (72h) | Position board, vote panel, Ketetapan doc |
| **Rayakan** (Celebration) | Achievement to honor | Validate achievement, recognize contributors, community messages | Consent | Appreciation wall, recognition card |
| **Siarkan** (Outreach) | Public support or awareness needed | Draft message, coordinate channels, track reach | Consent | Outreach tracker |
| **Penyelesaian** (Completion) | Always last | Credit distribution (AI-09), archive, impact assessment | Vote + challenge (72h) | Credit diff card, completion banner |

#### 3.2 Pattern Affinity Graph

Instead of a track-to-sequence mapping, the LLM receives an affinity graph that encodes how commonly one pattern follows another. This is descriptive, not prescriptive — the LLM uses it as a soft signal, not a rule.

```
         ┌──────────┐
    ┌───→│Investigasi│───→─┐
    │    └──────────┘      │
    │                      ▼
┌───────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌────────────┐
│Diskusi │─→│Perencanaan│─→│Pelaksanaan│─→│Verifikasi │─→│Penyelesaian│
└───────┘  └───────────┘  └───────────┘  └───────────┘  └────────────┘
    │         │    │           ▲               ▲
    │         │    └──→┌──────┐│               │
    │         └───────→│Galang│┘               │
    │                  └──────┘                │
    ├──→┌─────────┐                            │
    │   │Keputusan│──→─────────────────────────┘
    │   └─────────┘
    │
    ├──→┌────────┐
    │   │Rayakan │──→ Penyelesaian
    │   └────────┘
    │
    └──→┌────────┐
        │Siarkan │──→ (any subsequent pattern)
        └────────┘
```

**Edge weights** are seeded from the old track compositions but evolve over time through cross-case learning (Section 6). Initial weights:

| From → To | Weight | Origin |
|-----------|--------|--------|
| Diskusi → Perencanaan | 0.7 | Common in tuntaskan, wujudkan |
| Diskusi → Investigasi | 0.5 | Common in telusuri |
| Diskusi → Keputusan | 0.5 | Common in musyawarah |
| Diskusi → Rayakan | 0.4 | Common in rayakan |
| Perencanaan → Pelaksanaan | 0.8 | Almost always follows |
| Perencanaan → Galang | 0.5 | When resources needed |
| Galang → Pelaksanaan | 0.8 | After funding secured |
| Pelaksanaan → Verifikasi | 0.7 | Quality assurance |
| Verifikasi → Penyelesaian | 0.9 | End of path |
| Keputusan → Pelaksanaan | 0.6 | Implement the decision |
| Rayakan → Penyelesaian | 0.8 | Short path |
| Diskusi → Siarkan | 0.3 | When outreach needed |

The LLM uses these weights as soft priors. It may choose paths with low weights or skip high-weight edges entirely based on the diagnostic assessment. The graph is a compass, not a GPS.

#### 3.3 Composition Rules

- **Diskusi** is almost always first (exception: Siaga emergencies).
- **Penyelesaian** is always last.
- Middle patterns are selected and ordered by the LLM based on diagnostic assessment + graph affinity.
- Patterns can repeat (e.g., Diskusi again mid-plan if new deliberation is needed).
- The LLM can propose custom phases outside this library when no pattern fits.
- The LLM must not default to reproducing old track sequences. The graph is a soft signal, not a template.

### Layer 4: Governance Mapping

Each checkpoint inherits a default governance profile from its parent pattern. Privileged editors can override.

| Risk level | Mechanism | Duration | When used |
|-----------|-----------|----------|-----------|
| Low | Consent (auto-advance unless objection) | 24h | Routine checkpoints, non-controversial |
| Medium | Standard vote (>50% wins) | 48h | Resource allocation, role changes |
| High | Vote + evidence required | 72h | Quality verification, completion claims |
| Critical | Fast-track + 7-day post-hoc audit | Immediate | Emergencies only |

---

## 3. Orchestration Flow: Who Does What, When

### 3.0 Notation

```
[User]      = the person using the app
[LLM]       = AI-00 and related AI touch points
[System]    = backend automation (not AI)
[Editor]    = project_manager or highest_profile_user
[Community] = participants in the case
```

### 3.1 Phase A: Entry (Bagikan Screen)

```
[User]    taps [+] → Bagikan screen opens
[LLM]     greets: "Ceritakan apa yang kamu lihat atau alami..."
[User]    describes situation in natural language
[LLM]     listens, runs diagnostic assessment (Layer 2) silently
[LLM]     may ask 1–3 clarifying questions (urgency, privacy, scope)
[LLM]     internally invokes AI-01 for triple refinement (Action type, temporal class)
[LLM]     context bar morphs: Listening → Probing → Leaning → Ready
[User]    sees context bar show "Ready" with path plan summary
[User]    taps "Lanjutkan" (accept) or "Ubah" (change)
```

**What happens at "Lanjutkan":**
```
[LLM]     generates initial PathPlan JSON:
          - Runs diagnostic assessment against pattern graph
          - Selects phase patterns, orders by affinity + case needs
          - Applies Layer 1 principles (PIC, completion, minimal viable)
          - All phases visible as headers (title + objective)
          - Near phases (1–2): fully detailed with checkpoints
          - Far phases (3+): header only, marked "belum dirinci"
          - Attaches track_hint, seed_hint as metadata
[System]  stores PathPlan (version 1) via API
[System]  creates seed entity linked to plan
[System]  opens dual-tab view: Percakapan + Tahapan
[User]    sees their conversation carried over to Percakapan tab
[User]    sees the proposed timeline in Tahapan tab
          (full journey visible as headers, near-term phases detailed)
```

**Effort for user: tell your story, tap one button.** That's it. The plan exists.

### 3.2 Phase B: Living With the Plan

This is where most time is spent. The plan is a living document.

#### B1: Working through checkpoints

```
[Community]  works on current phase's checkpoints
[User]       marks checkpoint complete (with evidence if required)
[System]     updates checkpoint status, triggers governance if needed
[System]     when all checkpoints in a phase complete → phase status = completed
[System]     activates next phase → status = active
[LLM]        (if next phase was a header) elaborates it:
             proposes detailed checkpoints now that context is richer
[LLM]        posts suggestion in Percakapan: "Phase berikutnya sudah aktif.
             Saya usulkan checkpoint-checkpoint berikut..."
[User]       reviews in Tahapan tab, accepts or modifies
```

#### B2: LLM proactive suggestions (mid-case)

```
[LLM]     monitors Percakapan discussion (via AI-06, AI-07)
[LLM]     detects that case needs change:
          - New information surfaced → propose new checkpoint
          - Case is more complex than expected → propose new phase
          - A phase is unnecessary → propose skipping it
          - A branch scenario emerged → propose branch
[LLM]     creates plan_suggestion (stored as diff against current version)
[LLM]     posts in Percakapan: "Berdasarkan diskusi, saya sarankan..."
[User]    sees suggestion card in Tahapan tab (diff view)
          suggestion has a consent timeout (see Section 3.5)
[Editor]  taps Accept / Reject / Modify (or waits for auto-accept)
[System]  if accepted: applies diff, increments version, respects locked_fields
```

#### B3: Human edits (manual override)

```
[Editor]  opens Tahapan tab, taps a phase or checkpoint
[Editor]  edits title, objective, adds/removes checkpoints
[System]  adds edited fields to locked_fields for that object
[System]  increments plan version
[System]  source = "human" for edited objects
[LLM]     receives updated plan with locks in next context window
[LLM]     will not propose changes to locked fields in future suggestions
```

#### B4: Replanning triggers

The LLM may propose a plan revision (not just checkpoint changes) when:

| Trigger | Example | LLM action |
|---------|---------|------------|
| New information | Discussion reveals the problem is bigger than thought | Propose adding phases |
| Blocked checkpoint | A checkpoint can't be completed due to external dependency | Propose skip or branch |
| Phase taking too long | Phase active for >2x estimated duration | Nudge in Percakapan, suggest simplification |
| Community request | Someone in Percakapan says "kita perlu rencana ulang" | Generate revised plan as suggestion |
| Branch condition met | Parent checkpoint of a conditional branch is completed | Activate branch, elaborate its phases |

### 3.3 Phase C: Completion

```
[System]  detects all phases in main branch = completed
[LLM]     proposes Penyelesaian phase (if not already present):
          - Credit distribution (AI-09 diff card)
          - Impact assessment prompt
          - Archive confirmation
[Editor]  reviews credit distribution → Accept / Modify / Reject
[System]  distributes Tandang credits per accepted allocation
[System]  72h challenge window opens for credit disputes
[System]  after challenge window: plan status = completed, entity archived
[Community] can still view the case and its history
```

### 3.4 Phase D: Recurring Cases (Templates)

Some activities repeat — monthly cleanups, quarterly budget reviews, recurring events. Users shouldn't rebuild plans from scratch each time.

#### Creating a template

```
[System]  when a plan reaches completion, offers: "Simpan sebagai template?"
[Editor]  taps "Simpan Template"
[System]  saves a sanitized copy of the plan structure:
          - Phase titles, objectives, checkpoint titles preserved
          - All statuses reset to "planned"
          - All source tags reset to "system" (template-derived)
          - locked_fields cleared
          - PIC, participant data, evidence, dates stripped
          - Template tagged with community_id + track_hint + keywords
```

#### Using a template

```
[User]    taps [+] → Bagikan screen → tells their story
[LLM]     during diagnostic assessment, searches for matching templates:
          - Same community, similar keywords, similar track_hint
          - If match found (similarity > 0.7):
[LLM]     context bar shows: "Ada template serupa: [Kerja Bakti Bulanan].
          Mau pakai sebagai dasar?"
[User]    taps "Pakai Template" or "Buat Baru"
[System]  if template chosen:
          - Clones template structure as new PathPlan v1
          - LLM can still suggest modifications based on this case's specifics
          - User/editor can edit freely (creating new locked_fields)
```

#### Scheduled recurring cases

```
[Editor]  on a completed recurring case, taps "Jadwalkan Ulang"
[System]  creates a scheduled job:
          - Interval: weekly / biweekly / monthly / custom
          - Auto-creates new case from template at scheduled time
          - Notifies PIC and community
          - LLM reviews new instance and may suggest adjustments:
            "Kerja bakti bulan ini bertepatan dengan musim hujan.
             Saya sarankan tambah checkpoint persiapan peralatan hujan."
```

### 3.5 Suggestion Timeout (Consent-Based Auto-Accept)

AI suggestions use a tiered consent timeout. If nobody acts within the timeout window, the suggestion is auto-accepted. Anyone can object during the window to pause auto-accept and trigger manual review.

| Suggestion type | Risk | Timeout | Objection effect |
|-----------------|------|---------|------------------|
| Rephrase checkpoint title | Low | 30 seconds | Pauses, requires manual accept |
| Add/reorder checkpoints within a phase | Low | 2 minutes | Pauses, requires manual accept |
| Elaborate a header phase (add checkpoints) | Low | 2 minutes | Pauses, requires manual accept |
| Add a new phase | Medium | 10 minutes | Pauses, triggers Editor review |
| Skip or remove a phase | Medium | 10 minutes | Pauses, triggers Editor review |
| Add a branch | High | No auto-accept | Requires explicit Editor accept |
| Major replan (>2 phases changed) | High | No auto-accept | Requires explicit Editor accept |

**UI behavior during timeout:**

```
┌──────────────────────────────────────────────┐
│  🤖 AI menyarankan: tambah checkpoint         │
│  "Koordinasi material dengan Siti"            │
│                                               │
│  ████████████░░░░ auto-terima dalam 1:42      │
│                                               │
│  [✓ Terima]  [✗ Tolak]  [✏️ Ubah]  [⏸ Tunda] │
└──────────────────────────────────────────────┘
```

- Progress bar counts down visually.
- Any participant can tap **Tunda** (pause) to stop the timer and escalate to Editor.
- If no one is online/active, the low-risk suggestions quietly apply. The plan version increments and the change is logged — fully reversible.
- High-risk changes never auto-accept. They wait indefinitely for an Editor.

---

## 4. LLM Prompt Contract

When generating or revising a plan, the LLM receives this context and follows these rules.

### 4.1 Context provided to LLM

```json
{
  "conversation_history": ["..."],
  "current_plan": { "...plan JSON with locked_fields..." },
  "community_context": {
    "member_count": 45,
    "active_cases": 12,
    "community_norms": "..."
  },
  "similar_completed_cases": [
    {
      "title": "Perbaikan jalan RT 03",
      "track_hint": "tuntaskan",
      "phases_used": ["diskusi", "perencanaan", "pelaksanaan", "verifikasi", "penyelesaian"],
      "duration_days": 21,
      "participants": 12,
      "acceptance_rate": 0.9,
      "outcome": "completed"
    }
  ],
  "available_templates": [
    {
      "template_id": "tmpl_01",
      "title": "Kerja Bakti Bulanan",
      "track_hint": "tuntaskan",
      "phases": ["diskusi", "perencanaan", "pelaksanaan", "penyelesaian"],
      "times_used": 6,
      "avg_satisfaction": 4.2
    }
  ],
  "pattern_affinity_graph": { "...edge weights..." },
  "user_role": "author | participant | project_manager | highest_profile_user",
  "prompt_version": "v0.2",
  "instructions": "... (see below)"
}
```

### 4.2 LLM Instructions (embedded in system prompt)

```
You are a community coordination assistant for Gotong Royong.

Your job: help people turn their stories into actionable plans.

PATTERN LIBRARY (independent blocks — do NOT treat as a fixed sequence):
- diskusi: community discussion, gather perspectives, assign PIC
- investigasi: research unknowns, form hypotheses, gather evidence
- perencanaan: plan complex work, break into tasks, estimate effort
- galang: pool funding or materials from community
- pelaksanaan: execute tasks, track progress
- verifikasi: peer review, quality check, challenge window
- keputusan: formal community vote and decision
- rayakan: celebrate achievement, recognize contributors
- siarkan: public outreach and awareness campaign
- penyelesaian: distribute credit, archive, assess impact

RULES:
1. Assess the case first. What does it need? Select patterns based on
   actual needs, not based on track classification.
2. Use the affinity graph as a soft signal for ordering, not a template.
3. Show the full journey upfront as phase headers (title + objective).
   Detail only the first 1–2 phases with checkpoints.
   Mark far phases as "belum dirinci" (not yet detailed).
4. Every plan starts with a discussion-type phase and ends with penyelesaian.
5. Include "Tentukan PIC" as an early checkpoint.
6. Phrase checkpoints as verifiable outcomes, not activities.
   GOOD: "Daftar lokasi terdampak terkumpul"
   BAD:  "Kumpulkan data lokasi"
7. Never modify fields listed in locked_fields.
8. Output changes as suggestions, never direct overwrites.
9. If you don't have enough context, ask a question. Don't guess.
10. Attach track_hint and seed_hint as optional metadata. They don't
    constrain the plan structure.
11. If similar_completed_cases are provided, learn from their structure
    but adapt to this case's specifics. Don't copy blindly.
12. If a matching template exists and the user chose it, use it as a
    starting point but suggest modifications where this case differs.
13. Keep all text in Bahasa Indonesia.
```

### 4.3 Output format

**Initial plan generation:** Full PathPlan JSON per `ADAPTIVE-PATH-SPEC-v0.1.md` Section 5. All phases included as headers. Near-term phases have checkpoints. Far phases have empty checkpoint arrays.

**Phase elaboration (when a header phase activates):**
```json
{
  "suggestion_type": "elaborate_phase",
  "target_plan_version": 3,
  "target_phase_id": "p3",
  "proposed_checkpoints": [
    { "title": "Bagi tugas ke 4 kelompok relawan", "source": "ai" },
    { "title": "Siapkan material di lokasi", "source": "ai" },
    { "title": "Kerjakan perbaikan (target: 2 hari)", "source": "ai" },
    { "title": "Dokumentasi foto sebelum/sesudah", "source": "ai" }
  ],
  "reasoning": "Phase pelaksanaan sekarang aktif. Berdasarkan diskusi sebelumnya, ada 4 kelompok relawan dan material sudah tersedia."
}
```

**Mid-case suggestion (delta proposal):**
```json
{
  "suggestion_type": "add_phase | add_checkpoint | modify_phase | skip_phase | add_branch",
  "target_plan_version": 3,
  "changes": [
    {
      "action": "add",
      "target": "phase",
      "after": "p2",
      "data": {
        "phase_id": "p2a",
        "title": "Galang Dana Material",
        "objective": "Kumpulkan dana untuk beli material perbaikan",
        "source": "ai",
        "checkpoints": [...]
      }
    }
  ],
  "reasoning": "Diskusi menunjukkan perlu dana untuk material. Saya sarankan tambah fase Galang."
}
```

---

## 5. Progressive Elaboration: Headers First, Details Later

The LLM lays out the full possible journey at plan creation — but only as headers. This gives the user a sense of where things are headed without overwhelming them with detail they don't need yet.

### Visual: What the user sees in Tahapan at plan creation

```
Phase 1: Diskusi & Diagnosis          ← ACTIVE · 3 checkpoints shown
  ☐ Kumpulkan perspektif warga
  ☐ Tentukan PIC
  ☐ Identifikasi pihak terkait

Phase 2: Perencanaan                  ← PLANNED · 2 checkpoints shown
  ☐ Rancang rencana kerja
  ☐ Estimasi kebutuhan material

Phase 3: Pelaksanaan                  ← HEADER ONLY · "belum dirinci"
  "Akan dirinci saat fase ini aktif"

Phase 4: Verifikasi                   ← HEADER ONLY · "belum dirinci"
  "Akan dirinci saat fase ini aktif"

Phase 5: Penyelesaian                 ← HEADER ONLY · "belum dirinci"
  "Kredit & arsip"
```

**Why this works:**
- The user sees the full journey (5 phases) so they know what's ahead.
- But they only need to think about Phase 1 right now.
- No information overload. The detail gradient matches what's relevant.
- As each phase activates, the LLM proposes checkpoints based on what it learned from earlier phases.

### Elaboration lifecycle

| Plan stage | Phases 1–2 | Phases 3–4 | Phase 5 (Penyelesaian) |
|-----------|-----------|-----------|----------------------|
| Creation | Detailed checkpoints | Header only (title + objective + "belum dirinci") | Header only |
| Phase 2 completing | Completed | LLM proposes checkpoints for Phase 3 (suggestion with timeout) | Header only |
| Phase 3 active | Completed | Detailed (after user accepted elaboration) | LLM proposes checkpoints |
| Phase 4 active | Completed | Completed | Detailed |

### What if the user wants to see everything upfront?

Some detail-oriented users may want full detail immediately. The LLM instruction says "detail only first 1–2 phases" but this is a default, not a hard limit. If the user asks in Percakapan: "Tolong rinci semua fase sekarang," the LLM elaborates all phases immediately.

---

## 6. Cross-Case Learning

The LLM can reference completed cases from the same community to make better plans. This is not mandatory for v0.1 but is the natural evolution.

### 6.1 Similarity Context (v0.1 — simple)

When generating a plan, the system provides summaries of 2–3 similar completed cases from the same community. The LLM uses them as soft reference, not templates.

**What the LLM receives:**
```json
"similar_completed_cases": [
  {
    "title": "Perbaikan jalan RT 03",
    "track_hint": "tuntaskan",
    "phases_used": ["diskusi", "perencanaan", "pelaksanaan", "verifikasi", "penyelesaian"],
    "duration_days": 21,
    "participants": 12,
    "acceptance_rate": 0.9,
    "outcome": "completed"
  }
]
```

**How similarity is found:** Reuse the existing Tandang vector index (used by AI-03 duplicate detection). Embed the new case's story and find nearest neighbors among completed cases. Filter to same community.

### 6.2 Affinity Weight Evolution (v0.2 — planned)

Over time, the pattern affinity graph (Section 2, Layer 3) evolves based on actual usage:

- When a community frequently uses Galang after Perencanaan, that edge weight increases for that community.
- When a pattern is often skipped, its activation weight decreases.
- Community-specific profiles emerge naturally.

**Implementation:** Batch job aggregates completed plan structures per community. Computes edge frequencies. Updates community-scoped affinity weights. LLM receives community-specific graph, not just the global default.

### 6.3 Template Suggestions (v0.1)

When a case matches an existing template (Section 3.4), the LLM can proactively suggest it. This is the simplest form of cross-case learning — reuse a proven plan structure.

---

## 7. Edge Cases

| Scenario | Behavior |
|----------|----------|
| LLM generates a bad plan | User or editor modifies. Locked fields prevent LLM from reverting. |
| LLM is unavailable | System shows Tahapan tab with "Tambah fase manual" button. Editor builds plan by hand. |
| Case doesn't fit any pattern | LLM generates custom phases. No pattern is mandatory. |
| User wants the old track experience | If user selects a track hint manually via "Pilih sendiri," LLM uses the affinity graph seeded from that track as a stronger prior. |
| Plan gets too complex (>8 phases) | LLM suggests consolidation: "Rencana ini cukup panjang. Mau saya gabungkan beberapa fase?" |
| Two editors disagree | Governed proposal (1.5x quorum, 72h challenge) for disputed edits. |
| Case is abandoned | System detects no activity for 14 days → nudges PIC. 30 days → marks dormant. |
| Suggestion timeout fires while no one online | Low-risk changes auto-apply (logged, reversible). High-risk changes wait for Editor. |
| User asks to undo auto-accepted suggestion | Editor taps "Batalkan" on the applied suggestion within 24h. System reverts to previous version. |
| Recurring case needs adjustment | LLM proposes modifications to template-derived plan based on new context. Template itself is unchanged. |

---

## 8. Summary: What Each Actor Does

| Actor | Responsibilities | Never does |
|-------|-----------------|------------|
| **User** | Tells story, confirms plan, works on checkpoints, provides evidence, can pause suggestion timeouts | Doesn't need to understand patterns, graphs, or governance rules |
| **LLM** | Proposes plans, suggests changes, elaborates phases, nudges when stuck, references past cases | Never auto-applies high-risk changes, never overwrites locked fields |
| **Editor** | Accepts/rejects suggestions, manually edits plan, assigns PIC, saves templates | Doesn't need to build plans from scratch (LLM drafts first) |
| **Community** | Participates in discussion, votes on transitions, validates work, can object to suggestion timeouts | Doesn't see internal LLM reasoning, diagnostics, or affinity weights |
| **System** | Enforces governance, manages versions, tracks credits, runs timers, manages templates/schedules | Doesn't make subjective decisions |

---

## 9. Relationship to Other Specs

| Spec | What it covers | This doc adds |
|------|---------------|---------------|
| `ADAPTIVE-PATH-SPEC-v0.1.md` | Data model (plan, phase, checkpoint, branch) | When and how each object gets created/modified |
| `prd-adaptive-path-guidance.md` | Requirements (FR-01 through FR-10) | The flow that implements those requirements |
| `AI-SPEC-v0.2.md` (AI-00) | Triage conversation and entry flow | What AI-00 does after triage: plan generation |
| `UI-UX-SPEC-v0.5.md` (Section 3) | Adaptive Phase Patterns (UI) | The selection logic behind which patterns appear |
| `design-dna/05-card-system.md` | Card anatomy and stepper | How stepper reflects adaptive phases |

---

## 10. Open Questions

- **Elaboration timing**: Should the LLM elaborate the next phase when the current phase is 50% done, or only when it completes? (Leaning: 50% — gives time for review before activation.)
- **Affinity weight scope**: Should weights be global, per-community, or per-community-per-track-hint?
- **Template governance**: Who can save templates? Only Editors, or any PIC?
- **Recurring schedule limits**: Max frequency for auto-recurring cases? (Proposed: daily minimum.)
- **Suggestion timeout accessibility**: What if users have slow connectivity and can't respond in time? (Proposed: extend timeouts for communities with known connectivity issues.)

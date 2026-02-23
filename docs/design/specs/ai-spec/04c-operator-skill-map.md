> [← Back to AI Spec index](../AI-SPEC-v0.2.md)
> Related: [04b-trajectory-map.md](./04b-trajectory-map.md) · [04a-ai-00-edge-contract.md](./04a-ai-00-edge-contract.md)

# Operator Map — AI-00 Triage Architecture

> **Status:** Draft v0.2
> **Last updated:** 2026-02-23
>
> This document defines the 9 uniform operators for AI-00 triage.
> Each trajectory grid item maps 1:1 to an operator, plus one operator
> for group lifecycle management. All operators emit the same
> `OperatorResponse` canonical envelope with an operator-specific payload.
> Classification is inline in the orchestrator (not a separate operator).
>
> **Breaking change from v0.1:** The operator/skill distinction is removed.
> Everything that handles a trajectory is an "operator". Model tier is a
> config knob per operator, not an architectural boundary.

---

## 1. Architecture Overview

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
                    └──┬──┬──┬──┬──┬──┬──┬──┬──┬──┘
                       │  │  │  │  │  │  │  │  │
        ┌──────────────┘  │  │  │  │  │  │  │  └──────────────┐
        │     ┌───────────┘  │  │  │  │  │  └───────────┐     │
        │     │     ┌────────┘  │  │  │  └────────┐     │     │
        │     │     │     ┌─────┘  │  └─────┐     │     │     │
        ▼     ▼     ▼     ▼        ▼        ▼     ▼     ▼     ▼
     ┌─────┬─────┬─────┬─────┬──────┬──────┬─────┬──────┬──────┐
     │Masa-│Musy-│Pan- │Catat│Bantu-│Raya- │Siaga│Prog- │Kelola│
     │lah  │wrah │tau  │     │an    │kan   │     │ram   │      │
     │ A+B │ F+L │  D  │ C+E │  G   │  I   │  J  │  M   │  —   │
     └─────┴─────┴─────┴─────┴──────┴──────┴─────┴──────┴──────┘
        │     │     │     │     │      │     │     │      │
        └─────┴─────┴─────┴─────┴──────┴──────┴─────┴──────┘
                              │
                    ┌─────────┴─────────┐
                    │   All return:     │
                    │  OperatorResponse │
                    │  { status,        │
                    │    checklist,     │
                    │    questions?,    │
                    │    payload }      │
                    └───────────────────┘

     ┌──────────────────────────────────────────────────────┐
     │             CROSS-CUTTING MIDDLEWARE                  │
     │                                                      │
     │  PII Redaction · Vault Detection · Urgency Scoring   │
     │  Location · Confidence · Duplicate · Card Enrichment │
     └──────────────────────────────────────────────────────┘
```

### 1.1 Design Principle: One Grid Item = One Operator

The trajectory grid shows 8 user-facing intents. Each maps to exactly
one operator. The 9th operator (Kelola) handles group management and
is reachable from the same triage flow.

All operators return `OperatorResponse`. The orchestrator reads
`status`, `checklist`, and `questions` uniformly — the `payload`
is operator-specific but the orchestrator doesn't need to inspect it.

The only difference between operators is **model tier** (a config knob):

| Tier | Model Class | Operators |
|------|-------------|-----------|
| **Strong** | Sonnet-class | Masalah, Musyawarah, Pantau |
| **Medium** | Haiku-class | Catat, Bantuan, Rayakan, Siaga, Program, Kelola |

Strong-tier operators need multi-step LLM reasoning (problem analysis,
decision structuring, timeline building). Medium-tier operators primarily
structure data against a template. But both emit the same envelope.

---

## 2. Operators

### 2.1 Masalah — "Ada masalah, bisa kita selesaikan?"

| Property | Value |
|----------|-------|
| **Grid label** | Masalah |
| **Trajectories** | A (aksi) + B (advokasi) |
| **Reasoning** | Problem analysis → solution planning |
| **Output** | `MasalahPayload` containing `PathPlan` |
| **Key question** | "Can the community self-solve?" (determines A vs B) |
| **Model tier** | Strong (Sonnet-class) |

**Checklist fields:** `problem_scope`, `who_affected`, `resources_available`, `prior_attempts`, `self_solvable`

**Why one operator for A and B:** The KEY question (can community self-solve?) is part of the operator's assessment. One operator, two output patterns:
- `self_solvable = true` → trajectory A with community-executable phases
- `self_solvable = false` → trajectory B with escalation phases

### 2.2 Musyawarah — "Perlu diputuskan bersama"

| Property | Value |
|----------|-------|
| **Grid label** | Musyawarah |
| **Trajectories** | F (mufakat) + L (mediasi) |
| **Reasoning** | Issue decomposition → decision structuring |
| **Output** | `MusyawarahPayload` containing `DecisionStep[]` |
| **Key mechanism** | Break complex issue into fundamental agreement steps |
| **Model tier** | Strong (Sonnet-class) |

**Checklist fields:** `issue_description`, `stakeholders`, `key_decisions`, `constraints`, `preferred_outcome`

**Why one operator for F and L:** The mechanism (break issue into decision steps) is identical. Context differs (proposal vs dispute), not process. Each `DecisionStep` becomes a `VoteBlock` in the witness.

**Gateway (F only):** If all decision steps reach consensus and `on_consensus = 'spawn_aksi'`, the system spawns a new Trajectory A witness.

### 2.3 Pantau — "Saya mau pantau perkembangan ini"

| Property | Value |
|----------|-------|
| **Grid label** | Pantau |
| **Trajectories** | D (pantau) |
| **Reasoning** | Case analysis → timeline structuring |
| **Output** | `PantauPayload` containing `TimelineEvent[]` + `tracking_points` |
| **Key mechanism** | Observation — fundamentally different from action or deliberation |
| **Model tier** | Strong (Sonnet-class) |

**Checklist fields:** `case_description`, `parties_involved`, `current_stage`, `key_events`, `desired_outcome`

### 2.4 Catat — "Saya mau catat sesuatu"

| Property | Value |
|----------|-------|
| **Grid label** | Catat |
| **Trajectories** | C (data) + E (vault) |
| **Reasoning** | Structure a data point — public or private |
| **Output** | `CatatPayload` containing structured record |
| **Key question** | "Public community data or sealed private record?" (determines C vs E) |
| **Model tier** | Medium (Haiku-class) |

**Checklist fields:** `claim`, `location`, `timestamp`, `category`, `proof_attached`, `is_private`

**How C vs E is determined:**
- `is_private = false` → trajectory C (community data), public feed
- `is_private = true` → trajectory E (vault), sealed with hash, owner-only access

The operator structures the record identically; the privacy flag controls storage and visibility.

### 2.5 Bantuan — "Saya butuh bantuan"

| Property | Value |
|----------|-------|
| **Grid label** | Bantuan |
| **Trajectories** | G (bantuan) |
| **Reasoning** | Identify help type → match with expertise registry |
| **Output** | `BantuanPayload` containing help request + matched resources |
| **Model tier** | Medium (Haiku-class) + embedding similarity |

**Checklist fields:** `help_type`, `description`, `urgency`, `location`, `preferred_format`

**Gateway:** Often becomes B (escalation) when institutional help is needed.

### 2.6 Rayakan — "Ada yang patut dirayakan!"

| Property | Value |
|----------|-------|
| **Grid label** | Rayakan |
| **Trajectories** | I (pencapaian) |
| **Reasoning** | Generate achievement record, link to completed work |
| **Output** | `RayakanPayload` containing achievement details |
| **Model tier** | Medium (Haiku-class) |

**Checklist fields:** `achievement`, `contributors`, `linked_witness_id`, `impact_summary`

**Auto-trigger:** Also triggered when any witness reaches "completed" status.

### 2.7 Siaga — "Bahaya! Warga perlu tahu"

| Property | Value |
|----------|-------|
| **Grid label** | Siaga |
| **Trajectories** | J (siaga) |
| **Reasoning** | Structure urgent alert with verification hooks |
| **Output** | `SiagaPayload` containing alert data |
| **Model tier** | Medium (Haiku-class) |

**Checklist fields:** `threat_type`, `location`, `severity`, `description`, `source`, `expires_at`

**Special behaviors:**
- Urgency Scoring middleware auto-activates on siaga routing
- Alert has expiry mechanism — auto-fades after danger period
- Community verification (confirm/deny) enabled by default
- Aggregates to Alert Board page

**Gateway:** Often becomes A (collective action) when community decides to respond.

### 2.8 Program — "Kami punya kegiatan rutin"

| Property | Value |
|----------|-------|
| **Grid label** | Program |
| **Trajectories** | M (program) |
| **Reasoning** | Structure recurring activity — schedule, rotation, participants |
| **Output** | `ProgramPayload` containing schedule definition |
| **Model tier** | Medium (Haiku-class) |

**Checklist fields:** `activity_name`, `frequency`, `participants`, `rotation_rule`, `location`, `next_occurrence`

### 2.9 Kelola — "Saya mau atur kelompok"

| Property | Value |
|----------|-------|
| **Grid label** | Kelola |
| **Trajectories** | — (no trajectory, group lifecycle) |
| **Reasoning** | Group CRUD — create, configure, invite, manage membership |
| **Output** | `KelolaPayload` containing group action |
| **Model tier** | Medium (Haiku-class) |

**Checklist fields:** `action_type`, `group_name`, `description`, `join_policy`, `entity_type`, `invited_members`

**Why an operator, not a page?** The triage chat is the primary entry point. Users say "Saya mau bikin kelompok untuk ronda" — the orchestrator routes to Kelola, gathers requirements conversationally, and emits a ready payload. The `/komunitas/kelompok` page is a CRUD fallback, not the primary path.

**Action types:**
- `create` — new group with name, description, policy, entity type
- `edit` — modify group settings
- `invite` — add members by user ID
- `join` — request to join an existing group
- `leave` — leave a group

---

## 3. Cross-Cutting Middleware

Middleware runs on every turn or at specific triggers. These are NOT
operators — they don't emit `OperatorResponse`. They transform or
annotate the conversation context.

| Middleware | Purpose | Invoked | Type |
|-----------|---------|---------|------|
| **PII Redaction** | Mask personal data (names, addresses, phone, ID) | Every turn | Regex + pattern |
| **Vault Detection** | Detect privacy/sensitivity signals → hint Catat operator | Every turn | Keyword + pattern |
| **Urgency Scoring** | Detect emergency signals (fire, flood, medical) → hint Siaga operator | Every turn | Keyword + pattern |
| **Location Enrichment** | Resolve coordinates to RT/RW/Kelurahan | When location mentioned | Geocoding API |
| **Confidence Labeling** | Human-readable confidence label | When confidence changes | Template |
| **Duplicate Detection** | Check existing witnesses/data items | When enough context | Embedding + similarity |
| **Card Enrichment** | Generate title, icon, hook_line, body, sentiment, tags | At triage completion | LLM-assisted |

### 3.1 Card Enrichment (Detail)

Invoked at triage completion, just before the final `TriageResult` is returned:

```
Orchestrator → Operator (ready) → Card Enrichment middleware → final response
                                        │
                                        ├─ Reads: user's story + operator payload
                                        ├─ Generates: title, hook_line, body, icon, sentiment
                                        └─ Attaches: to TriageResult as card_enrichment field
```

Output: `CardEnrichment` interface (see `types/card-enrichment.ts`):
- `icon` — case-specific Lucide icon name (AI-selected per content, not per trajectory)
- `trajectory_type` — drives mood color, NOT icon
- `title` — AI-crafted, max 80 chars, factual + compelling + safe
- `hook_line` — one sentence that draws attention
- `body` — 2-3 sentence summary
- `sentiment` + `intensity` — for visual styling
- `entity_tags` — AI-suggested tags for discoverability

---

## 4. Consultation Protocol

### 4.1 Canonical Envelope

All 9 operators return the same envelope structure. The orchestrator
reads `status`, `checklist`, and `questions` uniformly — the `payload`
is operator-specific.

```typescript
interface OperatorResponse {
  status: 'need_more' | 'ready';
  checklist: ChecklistItem[];
  questions?: string[];
  payload: MasalahPayload | MusyawarahPayload | PantauPayload
         | CatatPayload   | BantuanPayload    | RayakanPayload
         | SiagaPayload   | ProgramPayload    | KelolaPayload;
}
```

| Layer | Same for all | Different per operator |
|-------|-------------|----------------------|
| Orchestrator reads | `status`, `checklist`, `questions` | (ignores payload details) |
| System stores | — | `payload` → mapped to blocks / PathPlan / DataItemRecord |
| Adding new operator | No orchestrator changes | Just define new payload type + add to union |

### 4.2 Payload Summary

| Operator | Payload Type | Key Fields |
|----------|-------------|------------|
| Masalah | `MasalahPayload` | `trajectory: 'A'\|'B'`, `path_plan: PathPlan` |
| Musyawarah | `MusyawarahPayload` | `context: 'proposal'\|'dispute'`, `decision_steps: DecisionStep[]` |
| Pantau | `PantauPayload` | `case_type`, `timeline_seed: TimelineEvent[]`, `tracking_points` |
| Catat | `CatatPayload` | `record_type: 'data'\|'vault'`, `claim`, `location`, `proof_url?`, `hash?` |
| Bantuan | `BantuanPayload` | `help_type`, `description`, `matched_resources: MatchedResource[]` |
| Rayakan | `RayakanPayload` | `achievement`, `contributors`, `linked_witness_id?` |
| Siaga | `SiagaPayload` | `threat_type`, `severity`, `location`, `expires_at` |
| Program | `ProgramPayload` | `activity_name`, `frequency`, `rotation: RotationEntry[]` |
| Kelola | `KelolaPayload` | `action: 'create'\|'edit'\|'invite'\|'join'\|'leave'`, `group_detail` |

---

## 5. Routing Decision Tree

```
User message arrives
  │
  ├─ Middleware: PII, Vault Detection, Urgency Scoring (every turn)
  │
  ├─ Vault signals? ──yes──→ Catat operator (is_private = true)
  ├─ Siaga signals? ──yes──→ Siaga operator
  │
  ├─ Classify inline (orchestrator)
  │   │
  │   ├─ Problem/complaint? ───────→ Masalah operator
  │   ├─ Proposal/deliberation? ───→ Musyawarah operator
  │   ├─ Monitoring/tracking? ─────→ Pantau operator
  │   ├─ Data/fact/proof? ─────────→ Catat operator
  │   ├─ Help request? ────────────→ Bantuan operator
  │   ├─ Celebration? ─────────────→ Rayakan operator
  │   ├─ Emergency/warning? ───────→ Siaga operator
  │   ├─ Schedule/recurring? ──────→ Program operator
  │   ├─ Group management? ────────→ Kelola operator
  │   └─ Ambiguous? ───────────────→ Keep probing
  │
  └─ Simple question? → Answer from Community Knowledge Base
```

---

## 6. Orchestration Flow Example

```
Turn 1:
  User → "Jalan rusak 3 bulan, sudah lapor RT tidak ada respons"
  Orchestrator:
    Middleware: PII ✓, Urgency ✓ (not urgent), Vault ✓ (not vault)
    Classify inline: signal = 'problem', confidence = 0.5
    Route → Masalah operator
  Masalah returns: {
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
    Middleware: PII ✓, Duplicate ✓ (no match)
    Forward to Masalah
  Masalah returns: {
    status: 'ready',
    checklist: [ ...all filled... ],
    payload: {
      trajectory: 'A',
      path_plan: { title: "Perbaikan Jalan Jl. Mawar", phases: [...] }
    }
  }
  Card Enrichment middleware → {
    icon: 'construction',
    trajectory_type: 'aksi',
    title: 'Jalan Berlubang Jl. Mawar, 30 KK Terdampak',
    hook_line: '3 bulan tanpa respons — warga turun tangan sendiri',
    sentiment: 'hopeful', intensity: 3
  }
  Orchestrator → User: "Triase selesai! Jalur: Aksi Bersama."
  bar_state → ready, proposed_plan → PathPlan, card_enrichment → CardEnrichment
```

---

## 7. Grid-to-Operator Mapping (Quick Reference)

| # | Grid Label | Operator | Trajectories | Feed Category | Model Tier |
|---|-----------|----------|--------------|---------------|------------|
| 1 | Masalah | Masalah | A + B | Witness | Strong |
| 2 | Musyawarah | Musyawarah | F + L | Witness | Strong |
| 3 | Pantau | Pantau | D | Witness | Strong |
| 4 | Catat | Catat | C + E | Data Item | Medium |
| 5 | Bantuan | Bantuan | G | Data Item | Medium |
| 6 | Rayakan | Rayakan | I | Data Item | Medium |
| 7 | Siaga | Siaga | J | Data Item | Medium |
| 8 | Program | Program | M | Witness | Medium |
| 9 | — | Kelola | — | — (group CRUD) | Medium |

---

## 8. Migration from v0.1

| v0.1 Concept | v0.2 Concept | Change |
|-------------|-------------|--------|
| 3 operators + 5 skills | 9 uniform operators | Skills promoted to operators |
| `AksiPayload` | `MasalahPayload` | Renamed to match grid label |
| `MufakatPayload` | `MusyawarahPayload` | Renamed to match grid label |
| `PantauPayload` | `PantauPayload` | Unchanged |
| Catat skill (C+J) | Catat operator (C+E) + Siaga operator (J) | Siaga split out, Vault moved in |
| Brankas skill (E) | Catat operator (E path) | Merged — vault is just `is_private = true` |
| Cocok skill (G) | Bantuan operator | Renamed |
| Jadwal skill (M) | Program operator | Renamed |
| Rayakan skill (I) | Rayakan operator | Promoted |
| — | Kelola operator | New — group lifecycle |
| "Cross-cutting skills" | "Cross-cutting middleware" | Terminology clarified |

---

# Tandang Integration Contract — v0.1

> **Purpose**: Define the interface between gotong-royong and the tandang human-measurement engine.
> **Status**: DRAFT — implementation-focused contract for gotong-royong ↔ tandang integration.
> **Companion docs**:
> - [ENGAGEMENT-STRATEGY-v0.2.md](../ENGAGEMENT-STRATEGY-v0.2.md) — Octalysis × Tandang phase roadmap
> - [TANDANG-SIGNAL-INVENTORY-v0.1.md](./TANDANG-SIGNAL-INVENTORY-v0.1.md) — Raw signal catalog (120+ events)
> - [ENGAGEMENT-BACKEND-CONTRACT-v0.1.md](../ENGAGEMENT-BACKEND-CONTRACT-v0.1.md) — Feed field registry
> **Last updated**: 2025-02-19

---

## Table of Contents

1. [Integration Model](#1-integration-model)
2. [Outbound Signals — gotong-royong → tandang](#2-outbound-signals)
3. [Inbound Events — tandang → gotong-royong](#3-inbound-events)
4. [Query Endpoints — gotong-royong reads from tandang](#4-query-endpoints)
5. [Per-Phase Integration Map](#5-per-phase-integration-map)
6. [Signal Classification — Explicit vs Implicit](#6-signal-classification)
7. [Data Flow Diagrams](#7-data-flow-diagrams)
8. [FeedItem Extensions](#8-feeditem-extensions)
9. [Open Questions](#9-open-questions)

---

## 1. Integration Model

### Architecture Assumption

```
┌──────────────────┐         ┌──────────────────┐
│  gotong-royong   │  ──→──  │     tandang      │
│  (frontend +     │  sends  │  (reputation     │
│   backend)       │ signals │   engine)         │
│                  │  ←──←   │                  │
│                  │  events  │                  │
│                  │  + query │                  │
└──────────────────┘         └──────────────────┘
```

**Three integration channels:**

| Channel | Direction | Transport (TBD) | Purpose |
|---------|-----------|-----------------|---------|
| **Outbound signals** | GR → Tandang | REST / gRPC / SDK call | User actions that create tandang events |
| **Inbound events** | Tandang → GR | Webhook / SSE / message queue | Tandang events that update GR UI |
| **Query** | GR → Tandang | REST / gRPC / SDK call | Read user scores, tier, vouch budget |

> **Note**: Transport mechanism TBD until SDK arrives. This contract defines the **logical interface** — what data crosses the boundary, not how.

---

## 2. Outbound Signals

### What gotong-royong sends TO tandang when users take actions.

### 2a. Explicit Chip Actions (Phase 2)

These are triggered by the user tapping a chip on a feed card.

| Chip | Tandang Operation | Payload | Response Expected |
|------|------------------|---------|-------------------|
| 🤝 **Vouch** | Create positive vouch | `{ voucher_id, vouchee_id, vouch_type: "positive", context_entity_id }` | `{ vouch_id, bleed_in_phase: 1, weight_active: 0.25 }` |
| 🔍 **Skeptis** | Create skeptical vouch | `{ voucher_id, target_id, vouch_type: "skeptical", context_entity_id, concern_note? }` | `{ vouch_id, dampening_applied, current_dampening_total }` |
| 🫣 **Saya Saksi** (problem card) | Submit PoR evidence | `{ witness_id, problem_id, evidence_type: "witness_attestation", location?, media_ids[] }` | `{ por_id, validation_status: "pending" }` |
| ✅ **Sudah Beres** (solution card) | Validate PoR for solution | `{ witness_id, solution_id, evidence_type: "completion_attestation", location?, media_ids[] }` | `{ por_id, validation_status: "pending" }` |
| 📷 **Bukti Valid** (evidence card) | Validate PoR media | `{ validator_id, por_id, validation_result: "authentic" }` | `{ validation_record_id, j_score_delta }` |
| ⭐ **Bagus** | QualitySpectrum vote | `{ voter_id, contribution_id, quality_score: 1-10 }` | `{ vote_recorded, current_avg, votes_remaining_for_quorum }` |
| ⚠️ **Perlu Dicek** | Flag uncertainty | `{ flagger_id, entity_id, uncertainty_type, note? }` | `{ flag_id, triggers_verification: bool }` |

### 2b. Inline Vote (Phase 2)

| Action | Tandang Operation | Payload |
|--------|------------------|---------|
| Vote Yes/No | Record governance vote | `{ voter_id, proposal_id, vote: "yes" \| "no", weight? }` |
| Vote result tips | Governance endorsement | Automatic — tandang computes when threshold met |

### 2c. Implicit Signals (captured silently)

| User Behavior | What We Send | Tandang Operation | When |
|---|---|---|---|
| Opens card detail | Activity ping | `record_activity(user_id)` | On detail view open |
| Reads >30 seconds | Deep engagement | `record_activity(user_id)` + internal analytics | After 30s dwell timer |
| Posts in discussion | Contribution | `process_contribution({ user_id, type: "continuous", content_ref })` | On message send |
| Submits evidence | PoR + contribution | `submit_por_evidence(...)` + `process_contribution(...)` | On evidence upload |
| Claims problem | Problem claim | `claim_problem({ user_id, problem_id })` | On claim action |
| Submits solution | Solution delivery | `submit_solution({ user_id, problem_id, solution_ref })` | On solution submit |
| Serves as verifier | Verification duty | `record_verification_decision({ verifier_id, ... })` | On verification submit |
| Serves on jury | Jury participation | `cast_jury_vote({ juror_id, case_id, vote })` | On jury vote |

---

## 3. Inbound Events

### What tandang sends TO gotong-royong when things change.

### 3a. Feed-Relevant Events (generate or update Pulse cards)

| Tandang Event | GR Action | Card Mood | Priority |
|---|---|---|---|
| `contribution_processed` | Create/update feed card | harapan (green) | Normal |
| `problem_created` | Create feed card | netral / peringatan | Normal |
| `problem_claimed` | Update card — show claimer | harapan | Normal |
| `solution_submitted` | Create feed card | harapan | Normal |
| `solution_verified` | Update card — show verified badge | harapan | High |
| `jury_verdict_reached` | Create feed card (decision) | keputusan (warm) | High |
| `slash_case_approved` | Create feed card (warning) | peringatan (red) | High |
| `slash_triggered` | Update user cards — shadow indicator | peringatan | High |
| `por_validation_passed` | Update card — show PoR badge | harapan | Normal |
| `por_tampering_detected` | Create alert card | bahaya (red) | Critical |
| `skill_polymath_achieved` | Create achievement card | harapan | Normal |
| `recovery_fully_restored` | Create recovery card | harapan | Normal |
| `decay_warning_issued` | Personal notification (not feed) | peringatan | Medium |
| `decay_threshold_crossed` | Personal notification + tier change | peringatan | High |

### 3b. Real-Time Card Updates

| Tandang Event | Card Update | Where Shown |
|---|---|---|
| `vouch_created` | Increment vouch count on card | Card header area |
| `vouch_created` (skeptical) | Show/increment skeptis count | Card header (amber) |
| `vouch_outcome_recorded` | Flash outcome indicator | Card detail view |
| `jury_vote_cast` | Update quorum bar | Card footer |
| `jury_no_quorum_reached` | Show "kuorum belum tercapai" badge | Card footer |
| `problem_status_changed` | Update card phase badge | Card header |
| `verification_decision_made` | Update verification status | Solution card |

### 3c. User-Personal Events (notifications, not feed)

| Tandang Event | Notification | Where |
|---|---|---|
| `decay_warning_issued` | "Kompetensi infrastruktur kamu akan turun dalam 12 hari" | Profile / toast |
| `jury_selected` | "Kamu dipilih sebagai juri" | Notification bell |
| `vouch_outcome_recorded` (your vouch) | "Orang yang kamu jamin berkinerja baik (+0.02 J)" | Notification bell |
| `slash_j_score_updated` (your J) | "Skor penilaian kamu berubah" | Profile alert |
| `skill_polymath_progress_updated` | "Progres polymath: 2/3 domain" | Profile |
| `vouch_phase_advanced` | "Jaminan kamu naik ke fase 2 (50% aktif)" | Notification bell |

---

## 4. Query Endpoints

### What gotong-royong reads FROM tandang on demand.

### 4a. User Profile Queries (Phase 3 + Phase 4)

| Query | Purpose | Expected Response | Phase |
|-------|---------|-------------------|-------|
| `get_user_tier(user_id)` | Show tier badge (◆◆◆◇) | `{ tier: "pillar", percentile: 94, symbol: "◆◆◆◇" }` | Phase 3 |
| `get_user_scores(user_id)` | Personal dashboard | `{ integrity: 0.72, judgment: 0.61, competence: { "infrastruktur": 0.8, "lingkungan": 0.45 } }` | Phase 4 |
| `get_vouch_budget(user_id)` | Show remaining vouches | `{ max_vouches: 50, active_vouches: 12, remaining: 38 }` | Phase 4 |
| `get_activity_signals(user_id)` | Streak, contribution count | `{ streak_days: 7, contributions_in_window: 12, quality_avg: 7.3, gap_days: 0 }` | Phase 4 |
| `get_polymath_progress(user_id)` | Polymath status | `{ domains_at_pillar: 2, needed: 3, progress: ["infrastruktur: pillar", "lingkungan: contributor"] }` | Phase 4 |
| `get_decay_status(user_id)` | Decay warnings | `{ domains_at_risk: [{ domain: "kesehatan", days_until_decay: 8 }] }` | Phase 4 |
| `get_recovery_status(user_id)` | Shadow/probation state | `{ status: "normal" }` or `{ status: "shadow", entered_at, probation_eligible_at }` | Phase 3 |

### 4b. Entity Queries (Phase 3 — card trust surface)

| Query | Purpose | Expected Response | Phase |
|-------|---------|-------------------|-------|
| `get_vouch_summary(entity_id)` | Vouch/skeptis counts on card | `{ positive_vouches: 12, skeptical_vouches: 3, total_weight: 8.4 }` | Phase 3 |
| `get_author_trust(user_id)` | Author trust indicator | `{ tier: "contributor", integrity: 0.65, streak_days: 4 }` | Phase 3 |
| `get_verification_status(entity_id)` | Solution verification | `{ verified: true, verifier_count: 3, quality_avg: 8.2 }` | Phase 3 |
| `get_por_status(entity_id)` | PoR validation | `{ has_por: true, validated: true, witness_count: 2 }` | Phase 3 |

### 4c. Feed Enrichment Queries (Phase 2+)

| Query | Purpose | Response | Phase |
|-------|---------|----------|-------|
| `get_card_signals(entity_id)` | Aggregate all signal data for a feed card | Combined vouch, PoR, verification, vote counts | Phase 2+ |
| `get_user_relation(user_id, entity_id)` | "Did I already vouch/flag/witness this?" | `{ vouched: true, vouch_type: "positive", witnessed: false, flagged: false }` | Phase 2 |

---

## 5. Per-Phase Integration Map

### Phase 1: Feel Alive ✅ (No tandang dependency)
- All data is mock/local
- Backend contract in ENGAGEMENT-BACKEND-CONTRACT-v0.1.md

### Phase 2: Sinyal Aksi (Signal Actions)

| Feature | Outbound | Inbound | Query |
|---------|----------|---------|-------|
| Vouch chip | `create_vouch` | `vouch_created` (update count) | `get_user_relation` (already vouched?) |
| Skeptis chip | `create_vouch(skeptical)` | `vouch_created` (update count) | `get_user_relation` |
| Saya Saksi chip | `submit_por_evidence` | `por_validation_passed/failed` | `get_user_relation` |
| Sudah Beres chip | `validate_por_solution` | `solution_por_validated` | `get_user_relation` |
| Bagus chip | `submit_quality_vote` | — (local update) | `get_user_relation` |
| Perlu Dicek chip | `flag_uncertainty` | `problem_uncertainty_flagged` | `get_user_relation` |
| Inline vote | `record_vote` | `jury_vote_cast` (quorum update) | Current vote state |

**Minimum tandang SDK needs for Phase 2:**
- 7 outbound operations
- 6 inbound event subscriptions
- 2 query endpoints

### Phase 3: Wajah Kepercayaan (Trust Surface)

| Feature | Outbound | Inbound | Query |
|---------|----------|---------|-------|
| Tier badge | — | `decay_threshold_crossed` (tier change) | `get_user_tier` |
| Vouch count | — | `vouch_created` | `get_vouch_summary` |
| Skeptis count | — | `vouch_created(skeptical)` | `get_vouch_summary` |
| Author trust | — | — | `get_author_trust` |
| Verified badge | — | `solution_verified` | `get_verification_status` |
| PoR badge | — | `por_validation_passed` | `get_por_status` |
| Author streak | — | — | `get_activity_signals` |
| Shadow indicator | — | `slash_user_shadowed` | `get_recovery_status` |

**Minimum tandang SDK needs for Phase 3:**
- 0 new outbound operations
- 3 new inbound event subscriptions
- 7 query endpoints (most are user/entity reads)

### Phase 4: Jejak Saya (My Trail)

| Feature | Outbound | Inbound | Query |
|---------|----------|---------|-------|
| Activity streak | — | `decay_activity_recorded` | `get_activity_signals` |
| Competence radar | — | `skill_competence_updated` | `get_user_scores` |
| Vouch budget | — | `vouch_created/withdrawn` | `get_vouch_budget` |
| J score display | — | `slash_j_score_updated` | `get_user_scores` |
| Decay countdown | — | `decay_warning_issued` | `get_decay_status` |
| Impact summary | — | — | `get_user_contributions_summary` (new) |
| Polymath progress | — | `skill_polymath_progress_updated` | `get_polymath_progress` |
| Tier progression | — | `decay_threshold_crossed` | `get_user_tier` |

**Minimum tandang SDK needs for Phase 4:**
- 0 new outbound operations
- 4 new inbound event subscriptions
- 3 new query endpoints (reuses many from Phase 3)

### Phase 5: Aliran (River Mode)
- No new tandang integration — reuses all Phase 2–4 data in a new layout

---

## 6. Signal Classification

### The Golden Rule

> **If it changes reputation (I/C/J), it MUST be an explicit user tap.**
> **If it only affects personalization/recommendations, implicit capture is fine.**

### 6a. Explicit Signals (require user tap)

| Signal | Why Explicit | Tandang Score Impact |
|--------|-------------|---------------------|
| 🤝 Vouch | Stakes J score (±0.02 to ±0.20) | J, trust graph |
| 🔍 Skeptis | Inverted J stake | J (inverted) |
| 🫣 Saya Saksi / ✅ Sudah Beres | PoR attestation — legal-grade | J (CoWitness ±0.02 to ±0.10) |
| ⭐ Bagus | Affects target's C score | Target C score |
| ⚠️ Perlu Dicek | Triggers verification pipeline | May lead to slash |
| 🗳️ Inline vote | Governance decision | J (jury/governance events) |

### 6b. Implicit Signals (captured silently)

| Behavior | What's Recorded | Tandang Impact |
|----------|----------------|----------------|
| Open card detail | `record_activity` | Resets decay timer only |
| Read >30s | Activity metric | Feed personalization |
| Post in discussion | `process_contribution` | C score (via contribution) |
| Submit evidence | `submit_por_evidence` | C + J scores |
| Claim problem | `claim_problem` | Problem lifecycle |
| Submit solution | `submit_solution` | C score (via solve_reward) |
| Hover on peek | — | Client-side analytics only |
| Share card | — | Social graph weight (TBD) |
| Return 3+ times | — | Recommendation signal |

### 6c. Captured But Not Scored (personalization only)

| Behavior | Use For | Never Use For |
|----------|---------|---------------|
| Card dwell time | Feed ranking | Reputation scoring |
| Scroll depth | Session quality | Anything |
| Topic browsing | Entity suggestions | Competence scoring |
| Reaction speed | Bot detection | User scoring |

---

## 7. Data Flow Diagrams

### 7a. Vouch Chip Flow

```
User taps 🤝 Vouch
    │
    ▼
GR Frontend: show optimistic "vouched" state
    │
    ▼
GR Backend: POST /api/tandang/vouch
    {voucher_id, vouchee_id, type: "positive", context}
    │
    ▼
Tandang SDK: create_vouch(...)
    │
    ├── Returns: {vouch_id, bleed_in_phase: 1, weight: 0.25}
    │
    ▼
GR Backend: persist vouch_id, update card cache
    │
    ▼
Tandang (async): vouch_phase_advanced (day 7 → 14)
    │
    ▼
GR Backend (webhook): update vouch weight display
```

### 7b. Saya Saksi Flow

```
User taps 🫣 Saya Saksi on problem card
    │
    ▼
GR Frontend: open witness attestation mini-form
    (optional: location, optional: media upload)
    │
    ▼
GR Backend: POST /api/tandang/por
    {witness_id, problem_id, type: "witness_attestation", location?, media[]}
    │
    ▼
Tandang SDK: submit_por_evidence(...)
    │
    ├── Returns: {por_id, status: "pending"}
    │
    ▼
Tandang (async): por_validation_passed OR por_validation_failed
    │
    ▼
GR Backend (webhook): update card PoR badge, notify witness of result
    │
    ▼
If validated: witness J score +0.02 (CoWitnessValidated)
If slashed: witness J score −0.10 (CoWitnessSlashed)
```

### 7c. Decay Warning Flow

```
Tandang (cron): decay_job_executed
    │
    ▼
Tandang: identifies users with domains approaching 30-day trigger
    │
    ▼
Tandang → GR (webhook): decay_warning_issued
    {user_id, domain: "infrastruktur", days_until_decay: 8}
    │
    ▼
GR Backend: create personal notification
    │
    ▼
GR Frontend: toast "Kompetensi infrastruktur kamu akan turun dalam 8 hari"
    │
    ├── If user takes action → Tandang: decay_prevented
    └── If no action → Tandang: decay_competence_decayed
        │
        ▼
        If crosses tier boundary → decay_threshold_crossed
            │
            ▼
            GR: update user tier badge everywhere
```

---

## 8. FeedItem Extensions

New fields needed on `FeedItem` (in `apps/web/src/lib/types/feed.ts`) for tandang integration:

### Phase 2 additions

```typescript
// ── Tandang Signal Actions ──────────────────────────────
/** Current user's relation to this entity (populated per-request) */
my_relation?: {
  vouched: boolean;
  vouch_type?: 'positive' | 'skeptical' | 'conditional' | 'mentorship';
  witnessed: boolean;
  flagged: boolean;
  quality_voted: boolean;
  vote_cast?: 'yes' | 'no';
};

/** Aggregate signal counts (from tandang) */
signal_counts?: {
  vouch_positive: number;
  vouch_skeptical: number;
  witness_count: number;    // PoR attestations
  quality_avg: number;      // 0-10
  quality_votes: number;
  flags: number;            // uncertainty flags
};
```

### Phase 3 additions

```typescript
// ── Trust Surface ───────────────────────────────────────
/** Author trust data (from tandang) */
author_trust?: {
  tier: 'keystone' | 'pillar' | 'contributor' | 'novice' | 'shadow';
  tier_symbol: string;       // '◆◆◆◇'
  integrity: number;         // 0-1
  streak_days: number;
  is_polymath: boolean;
};

/** Verification status (from tandang) */
verification?: {
  verified: boolean;
  verifier_count: number;
  quality_avg: number;
};

/** Proof of Reality status (from tandang) */
por_status?: {
  has_por: boolean;
  validated: boolean;
  witness_count: number;
};
```

### Phase 4 — no FeedItem changes (personal dashboard is a separate view)

---

## 9. Open Questions

| # | Question | Impact | Status |
|---|----------|--------|--------|
| 1 | What transport does tandang SDK use? (gRPC / REST / direct Rust FFI) | Determines GR backend adapter layer | ⚪ Waiting for SDK |
| 2 | Does tandang support event subscriptions? (webhook / SSE / message queue) | Determines real-time update architecture | ⚪ Waiting for SDK |
| 3 | Is tandang deployed as a separate service or embedded library? | Determines network topology | ⚪ Waiting for SDK |
| 4 | Can we batch-query multiple users' trust data? (feed rendering needs N author lookups) | Performance — batch vs N+1 queries | ⚪ Waiting for SDK |
| 5 | What's the latency budget for vouch creation? (user taps chip → confirmed) | Determines optimistic UI strategy | ⚪ Waiting for SDK |
| 6 | How are PoR locations verified? (GPS? proximity to problem location?) | Saya Saksi chip UX — do we require location? | 🟡 Needs design decision |
| 7 | Should Skeptical vouch require a note explaining the concern? | UX friction vs signal quality tradeoff | 🟡 Needs design decision |
| 8 | What happens when a Shadow user views cards? (can they still see chips?) | Shadow user UX | 🟡 Needs design decision |

---

## Changelog

| Date | Change |
|------|--------|
| 2025-02-19 | v0.1 — Initial pre-integration contract: 7 outbound ops, 14 inbound events, 12 queries, 5-phase map |

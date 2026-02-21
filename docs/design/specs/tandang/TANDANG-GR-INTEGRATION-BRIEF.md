# Tandang Engine Enhancement Brief — Gotong-Royong Integration

> **From**: GR Frontend/Product Team
> **To**: Tandang Engine Developer
> **Date**: 2025-02-21
> **Status**: PROPOSAL — Awaiting review before implementation
> **Context**: We're building the "Aku" (profile/CV Hidup) page for gotong-royong and need richer signals from tandang. This brief proposes engine-side changes to support a new interaction model.

---

## Table of Contents

1. [Background: The Dukung/Jamin Split](#1-background-the-dukungjamin-split)
2. [What We Verified (Engine Compatibility)](#2-what-we-verified-engine-compatibility)
3. [Proposal 1: New VouchContext Variant — `PatternObserved`](#3-proposal-1-new-vouchcontext-variant--patternobserved)
4. [Proposal 2: New VouchType — `Collective`](#4-proposal-2-new-vouchtype--collective)
5. [Proposal 3: GR-Native ESCO-ID Domain Extensions](#5-proposal-3-gr-native-esco-id-domain-extensions)
6. [Proposal 4: Dukung→J-Score Retroactive Pathway](#6-proposal-4-dukungj-score-retroactive-pathway)
7. [Proposal 5: Context-Aware Genesis Decay](#7-proposal-5-context-aware-genesis-decay)
8. [Proposal 6: Consistency Multiplier](#8-proposal-6-consistency-multiplier)
9. [Integration Contract: What GR Will Send](#9-integration-contract-what-gr-will-send)
10. [Questions for Tandang Developer](#10-questions-for-tandang-developer)

---

## 1. Background: The Dukung/Jamin Split

### Problem

In the current GR UI, users tap "Vouch" on feed cards (witness posts). But there's a semantic mismatch:

- **User intent**: "I support this project/witness" (lightweight, project-directed)
- **Engine effect**: Creates a `UserId → UserId` edge in the trust graph with J-score consequences (heavyweight, person-directed)

Users don't realize they're making a person-level trust statement when they casually tap a button on a feed card.

### Solution: Two Distinct Interactions

| | **Dukung** (Support) | **Jamin** (Vouch) |
|---|---|---|
| **Target** | Project / Witness | Person |
| **UX Weight** | Lightweight — tap on feed card | Heavyweight — deliberate action from profile page |
| **Enters tandang trust graph?** | ❌ No | ✅ Yes |
| **Stored in** | GR database (own table) | Tandang via `create_vouch` API |
| **Analogy** | Upvote / Heart | Professional recommendation |
| **I/C/J impact** | None directly (see Proposal 4 for indirect J) | Full — weight, bleed-in, dampening |

### Why This Is Safe for Tandang

We verified by reading the engine source:

- `TrustGraph` is `DiGraph<UserId, VouchWeight>` — nodes are **only** UserIds, no project/witness nodes
- `VouchContext` enum (CoWitness, VerifiedExecution, etc.) is **audit metadata only** — does not affect PageRank/trust transfer
- The Vouch struct: `voucher: UserId, vouchee: UserId` — always person-to-person

**Dukung never enters the trust graph.** Jamin maps to the existing `create_vouch` API with no changes needed.

---

## 2. What We Verified (Engine Compatibility)

We read these source files and confirmed:

| File | What We Checked | Finding |
|---|---|---|
| `trust_graph.rs` | Graph structure | `DiGraph<UserId, VouchWeight>` — person-only nodes |
| `vouch_entity.rs` | Vouch struct | `voucher: UserId, vouchee: UserId` — always 1:1 |
| `vouch_entity.rs` | VouchContext | 5 variants, all audit-only, no calc impact |
| `types/vouch.rs` | VouchType | 5 types with defined weights |
| `user_reputation/mod.rs` | I/C/J scores | IntegrityScore, CompetenceScore (per ESCO domain), JudgmentScore |
| `decay/` | Decay system | 30-day trigger, 90-day half-life for C; genesis monthly decay |
| `slash/` | Penalty system | Cascade penalties, GDF, slash cases |
| Markov `mod.rs` | PageRank calc | Standard Markov iteration, cycle detection, conductance |

**Conclusion**: The Dukung/Jamin split requires **zero changes** to the core PageRank calculation. All proposals below are additive — new enum variants, new fields, new side-channel scoring.

---

## 3. Proposal 1: New VouchContext Variant — `PatternObserved`

### Priority: HIGH | Effort: LOW

### Rationale

Currently, a `Direct` vouch has no evidence trail. When a GR user "upgrades" from repeatedly supporting someone's projects (Dukung) to vouching for them personally (Jamin), that history should be recorded.

### Proposed Change

Add to `VouchContext` enum in `vouch_entity.rs`:

```rust
pub enum VouchContext {
    // ... existing variants unchanged ...

    /// GR-originated vouch backed by observable support history.
    /// The voucher has previously supported multiple projects by the vouchee,
    /// providing an evidence trail for the trust assertion.
    PatternObserved {
        /// Number of Dukung (project-support) interactions before this Jamin
        dukung_count: u32,
        /// Timestamp of first Dukung interaction with this person's projects
        first_dukung_at: DateTime<Utc>,
        /// IDs of witnesses/projects that were supported
        witness_ids: Vec<Uuid>,
    },
}
```

### What This Enables

1. **Anti-collusion signal**: Vouches with `dukung_count > 3` are more credible than `Direct` — the voucher has demonstrated sustained interest
2. **Audit trail**: Why did this person vouch? Because they supported 7 projects over 4 months
3. **AKU page display**: "Dijamin oleh @Budi (mendukung 7 project kamu sejak Maret)"
4. **Future weight modifier** (optional): `PatternObserved` vouches could receive a small credibility bonus (e.g., 1.05×) — but this is NOT required for v1

### Impact on Existing Code

- **PageRank calculation**: NO CHANGE — VouchContext is metadata-only
- **Serialization**: New variant needs serde support
- **Database**: New JSONB variant in vouch context column
- **API**: GR will pass this context when calling `create_vouch`

---

## 4. Proposal 2: New VouchType — `Collective`

### Priority: HIGH | Effort: MEDIUM

### Rationale

Gotong-royong is inherently communal. Trust often forms collectively: "We three witnessed together that she did the work." Currently, if 3 people independently vouch for the same person from the same witness, tandang treats them as 3 separate unrelated vouches.

### Proposed Change

Add to `VouchType` enum in `types/vouch.rs`:

```rust
pub enum VouchType {
    // ... existing variants unchanged ...

    /// Collective vouch — multiple people corroborating the same trust assertion.
    /// Created when N (≥3) people vouch for the same person from the same context.
    Collective,
}
```

### Detection & Mechanics

**Option A — GR-side detection (simpler)**:
- GR detects when 3+ users Jamin the same person from the same witness
- GR sends the vouch as `VouchType::Collective` with `VouchContext::CoWitness`
- Tandang treats it with a defined weight

**Option B — Tandang-side detection (more robust)**:
- Tandang detects clusters: same vouchee + same VouchContext witness_id + N≥3 vouchers within a time window
- Automatically upgrades the cluster to Collective treatment
- Anti-collusion still applies (reciprocity dampening, temporal burst)

### Weight Suggestion

```
Collective weight: average(individual_weights) × corroboration_bonus
corroboration_bonus: 1.0 + min(0.2, (N - 2) × 0.05)
// 3 people = 1.05×, 4 = 1.10×, 5 = 1.15×, 6+ = 1.20× cap
```

### Anti-Collusion Considerations

- Existing reciprocity dampening (0.7×) still applies per pair
- Temporal burst detection catches coordinated timing
- Cycle detection catches circular vouching rings
- **Additional safeguard**: If the same group of people always vouch collectively, reduce the bonus progressively

### Impact on Existing Code

- **VouchWeight**: New weight calculation branch for Collective
- **Trust graph**: Still UserId → UserId edges, but weight is modified
- **Bleed-in**: Same 25%→50%→100% over 14 days
- **Anti-collusion**: May need new "group staleness" check

---

## 5. Proposal 3: GR-Native ESCO-ID Domain Extensions

### Priority: MEDIUM | Effort: LOW

### Rationale

CompetenceScore uses ESCO taxonomy for domains. ESCO-ID extension for Indonesian skills exists. But GR has activity domains not covered by ESCO:

- Community coordination (organizing gotong-royong events)
- Citizen investigation (gathering evidence for witnesses)
- Conflict mediation (resolving community disputes)
- Field verification (on-ground reality checking)
- Community mentorship (guiding new participants)

### Proposed Additions

```
ESCO-ID-GR-001: "Koordinasi Komunitas"     // Community Coordination
ESCO-ID-GR-002: "Investigasi Warga"         // Citizen Investigation
ESCO-ID-GR-003: "Mediasi Konflik"           // Conflict Mediation
ESCO-ID-GR-004: "Verifikasi Lapangan"       // Field Verification
ESCO-ID-GR-005: "Pendampingan Warga"        // Community Mentorship
```

### What This Enables

- Users build domain-specific C-scores for GR activities
- Decay works per-domain: active in mediation but inactive in investigation → only investigation decays
- AKU page shows competence radar per domain
- Vouch weight can be domain-qualified: "I vouch for her *in community coordination*"

### Impact on Existing Code

- **SkillId**: Just new entries in the ESCO-ID registry — no structural change
- **CompetenceScore HashMap**: Already supports arbitrary SkillId keys
- **Decay**: Already per-domain — works automatically

### Question for You

How are ESCO-ID extensions currently registered? Is there a config file, database seed, or hardcoded list?

---

## 6. Proposal 4: Dukung→J-Score Retroactive Pathway

### Priority: MEDIUM | Effort: MEDIUM-HIGH

### Rationale

Currently J-score only updates from VouchOutcome (Good/Poor/Slashed/Fraud). But "Judgment" should also capture: **can you identify good projects?**

Dukung is a lightweight signal, but it's still a judgment call. If you Dukung a project that later succeeds (verified), that says something about your judgment. If you Dukung one that gets slashed, that also says something.

### Proposed Mechanism

```
New JudgmentEvent variants:

DukungSuccess:   +0.01 J  // Supported project was verified/completed
DukungSlashed:   -0.02 J  // Supported project was slashed
```

### How It Works

1. GR records Dukung interactions in its own database
2. When a witness/project reaches a terminal state (verified ✅ or slashed ❌), GR notifies tandang
3. Tandang applies retroactive J-score adjustments to all users who Dukung'd that project
4. This creates a lightweight "prediction market" effect — Dukung has consequences

### Key Design Decisions

- **Dukung still does NOT create trust graph edges** — this is purely a J-score side-channel
- **Lower magnitude than vouch outcomes** (0.01/0.02 vs 0.02/0.05) — proportional to the lower commitment
- **Batched processing**: Can be a periodic job, not real-time
- **No bleed-in needed**: J-score adjustment is immediate upon project terminal state

### Impact on Existing Code

- **JudgmentScore**: New update pathway (currently only from VouchOutcome)
- **New API endpoint**: GR calls `report_dukung_outcome(user_id, witness_id, outcome)`
- **Batch job**: Process pending Dukung outcomes periodically
- **Trust graph**: NO CHANGE

### Open Question

Should there be a time limit? e.g., only Dukung interactions within 30 days of project completion count?

---

## 7. Proposal 5: Context-Aware Genesis Decay

### Priority: LOW | Effort: LOW

### Rationale

Genesis weight currently decays at `W_g(t) = W_g(0) × 0.9^months` regardless of activity. This penalizes genesis users who are actively contributing.

### Proposed Change

```rust
// Current:
// genesis_weight decays every month unconditionally

// Proposed:
// genesis_weight decays only during INACTIVE months
// Active month = any Dukung, Jamin, or contribution in that month

fn decay_genesis_weight(&mut self, months_since_last_decay: u32, active_months: u32) {
    let inactive_months = months_since_last_decay - active_months;
    if inactive_months > 0 {
        self.genesis_weight *= Decimal::from_str("0.9")
            .unwrap()
            .powd(inactive_months.into());
    }
}
```

### Impact

- **ActivityTracker**: Already exists — just needs to be wired to genesis decay
- **Fairness**: Active genesis users maintain their earned trust
- **Anti-gaming**: Can't game this by doing 1 tiny action per month — define "active month" threshold (e.g., ≥3 meaningful interactions)

---

## 8. Proposal 6: Consistency Multiplier

### Priority: LOW | Effort: MEDIUM

### Rationale

No mechanism currently rewards sustained consistent activity. A user active every week for 6 months is treated the same as a burst of activity in 1 week.

### Proposed Addition

New field in `UserReputation`:

```rust
pub struct ConsistencyMultiplier {
    /// Number of consecutive weeks with qualifying activity
    active_weeks_streak: u32,
    /// Computed multiplier applied to incoming vouch weight
    /// Formula: 1.0 + min(0.20, active_weeks_streak × 0.02)
    /// Range: [1.0, 1.2]
    multiplier: Decimal,
}
```

### Application

- Applied to **incoming** vouch weight: when someone vouches for a consistent user, the vouch is slightly more effective
- **Not** applied to outgoing vouches (you don't get to be more influential just by being consistent — that's handled by tier/percentile)
- Resets to 1.0 after 2 consecutive inactive weeks

### Impact

- **VouchWeight**: `effective_weight()` gets a new multiplier factor
- **Weekly job**: Update streaks based on ActivityTracker data
- **Trust graph**: Edge weights slightly modified, PageRank recalculates naturally

---

## 9. Integration Contract: What GR Will Send

### For Jamin (enters tandang)

GR will call the existing `create_vouch` API with:

```json
{
  "voucher": "user-uuid-abc",
  "vouchee": "user-uuid-xyz",
  "vouch_type": "Positive | Skeptical | Conditional | Mentorship | ProjectScoped | Collective",
  "context": {
    "type": "PatternObserved",
    "dukung_count": 7,
    "first_dukung_at": "2025-01-15T10:00:00Z",
    "witness_ids": ["witness-1", "witness-2", "witness-3"]
  }
}
```

### For Dukung (GR-native, does NOT enter tandang)

Stored in GR's own database:

```json
{
  "user_id": "user-uuid-abc",
  "witness_id": "witness-uuid-123",
  "type": "dukung",
  "timestamp": "2025-02-21T10:00:00Z"
}
```

### For Dukung Outcome Reporting (Proposal 4, if accepted)

GR will call a new endpoint when a witness reaches terminal state:

```json
{
  "witness_id": "witness-uuid-123",
  "outcome": "verified | slashed",
  "dukung_user_ids": ["user-1", "user-2", "user-3"],
  "completed_at": "2025-02-21T15:00:00Z"
}
```

---

## 10. Questions for Tandang Developer

### Must Answer Before Implementation

1. **VouchContext serialization**: Is VouchContext stored as JSONB? What's the migration path for adding a new variant without breaking existing data?

2. **Collective detection**: Do you prefer Option A (GR detects and sends as Collective) or Option B (tandang auto-detects clusters)? Option A is simpler; Option B is more robust.

3. **ESCO-ID extension registry**: How are custom skill domains registered? Config file, DB seed, or code enum?

4. **API versioning**: Should new VouchType/VouchContext variants be behind a feature flag or API version?

### Nice to Discuss

5. **Dukung→J pathway (Proposal 4)**: Is this architecturally acceptable? It introduces a new J-score update channel outside of VouchOutcome. Any concerns about score stability?

6. **Corroboration bonus cap**: Is 1.2× reasonable for Collective vouches, or should it be lower to be conservative?

7. **Genesis decay pause**: Any concern about active genesis users retaining too much influence? Should there be a hard cap (e.g., genesis weight can never exceed X after Y months regardless)?

8. **Batch processing**: For Proposal 4, is there an existing batch job infrastructure we can hook into, or does this need a new cron/scheduled task?

---

## Summary: Priority Order

| # | Proposal | Priority | Effort | Blocks GR? |
|---|---|---|---|---|
| 1 | `PatternObserved` VouchContext | 🔴 HIGH | LOW | No — GR can launch without, but it enriches Jamin |
| 2 | `Collective` VouchType | 🔴 HIGH | MEDIUM | No — GR can launch without, adds later |
| 3 | ESCO-ID GR domains | 🟡 MEDIUM | LOW | Partially — AKU page competence radar needs this |
| 4 | Dukung→J pathway | 🟡 MEDIUM | MED-HIGH | No — can add retroactively |
| 5 | Genesis decay pause | 🟢 LOW | LOW | No |
| 6 | Consistency multiplier | 🟢 LOW | MEDIUM | No |

**GR can launch Dukung/Jamin split with ZERO tandang changes.** These proposals enrich the system but are not blockers. We recommend implementing Proposals 1 and 3 first as they're low-effort high-value.

---

*This document was prepared after reading the tandang markov-engine source code (trust_graph.rs, vouch_entity.rs, types/vouch.rs, user_reputation/mod.rs, decay/, slash/) and cross-referencing with the TANDANG-SIGNAL-INVENTORY-v0.1.md spec.*

# Tandang Signal Inventory — v0.1

> **Purpose**: Living reference of all signals available from the [tandang](../../../../tandang/markov-engine) human-measurement engine.
> **Use**: Map these signals → gotong-royong UI affordances (reaction chips, badges, progress bars, trust indicators).
> **Source**: `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/crates/`
> **Date**: 2025-02-19

---

## Table of Contents

1. [User Tiers](#1-user-tiers)
2. [Three-Layer Reputation Model](#2-three-layer-reputation-model)
3. [Vouch System](#3-vouch-system)
4. [Slash System](#4-slash-system)
5. [Decay System](#5-decay-system)
6. [Contribution System](#6-contribution-system)
7. [Judgment Events](#7-judgment-events)
8. [Jury System](#8-jury-system)
9. [Problem & Solution Lifecycle](#9-problem--solution-lifecycle)
10. [Verification System](#10-verification-system)
11. [Skill & Polymath](#11-skill--polymath)
12. [Epoch System](#12-epoch-system)
13. [Genesis System](#13-genesis-system)
14. [Recovery System](#14-recovery-system)
15. [Proof of Reality (PoR)](#15-proof-of-reality-por)
16. [Identity](#16-identity)
17. [Activity Signals (Type B)](#17-activity-signals-type-b)
18. [Configuration Constants](#18-configuration-constants)
19. [UI Mapping Candidates](#19-ui-mapping-candidates)

---

## 1. User Tiers

Tier classification drives UI badging, vouch capacity, and trust weight.

| Tier | Symbol | Percentile | Vouch Weight | Max Active Vouches | Description |
|------|--------|------------|--------------|-------------------|-------------|
| **Keystone** | ◆◆◆◆ | 99–100% | 4× | 100 | Foundational experts |
| **Pillar** | ◆◆◆◇ | 90–98% | 3× | 50 | Established authorities |
| **Contributor** | ◆◆◇◇ | 60–89% | 2× | 25 | Active participants |
| **Novice** | ◇◇◇◇ | 0–59% | 1× | 10 | New / developing users |
| **Shadow** | ○ | Post-slash | 0× | 0 | Temporary restriction (verify-only) |

**Continuous Influence Formula:**
- Max vouches = `10 + (percentile × 0.9)`, clamped `[10, 100]`
- Endorsement weight = `1.0 + (percentile / 50)`, clamped `[1.0, 3.0]`
- Jury eligibility: percentile ≥ 85

---

## 2. Three-Layer Reputation Model

### 2a. Integrity Score (I) — Global, No Decay

| Property | Value |
|----------|-------|
| Default | 0.5 |
| Range | [0, 1] |
| Decay | None |
| Vouch multiplier | `0.5 + 0.5 × I` |

**Operations:**
| Operation | Effect |
|-----------|--------|
| `apply_boost(amount)` | `I += amount` (clamped to 1.0) |
| `apply_fraud_slash()` | `I = 0` (irreversible) |
| `apply_penalty(pct)` | `I = I × (1 - pct)` |
| `apply_polymath_bonus(bonus)` | Reward for 3+ domain expertise at Pillar+ |

### 2b. Competence Score (C) — Per Domain (ESCO), Decays

| Property | Value |
|----------|-------|
| Default | 0.5 per domain |
| Range | [0, 1] per domain |
| Inactivity trigger | 30 days |
| Half-life | 90 days |
| Decay formula | `C(t) = C₀ × 0.5^((days_inactive - 30) / 90)` |

**Difficulty multipliers:**
| Difficulty | Multiplier |
|------------|-----------|
| Easy | 1.0× |
| Medium | 1.5× (default) |
| Hard | 2.0× |

**Operations:**
| Operation | Effect |
|-----------|--------|
| `apply_solve_reward(base, difficulty)` | `C += base × multiplier` |
| `apply_verification_reward(amount)` | `C += amount` |
| `apply_fraud_slash()` | `C = 0` |
| `apply_penalty(pct)` | `C = C × (1 - pct)` |
| `record_activity()` | Reset decay timer |

### 2c. Judgment Score (J) — Vouch Quality, No Decay

| Property | Value |
|----------|-------|
| Default | 0.5 |
| Range | [0, 1] |
| Verification eligibility | J ≥ 0.3 |
| Vouch multiplier | `0.5 + 0.5 × J` |

---

## 3. Vouch System

### 3a. Vouch Types

| Type | Weight | Dampening | Affects I | Affects Trust Graph | Special |
|------|--------|-----------|-----------|--------------------|----|
| **Positive** | 1.0× | — | Yes | Yes | Default vouch |
| **Skeptical** | 0.75× | 10%/vouch, floor 70% | No | Yes | Inverted judgment (rewarded for flagging concerns) |
| **Conditional** | 1.0× | — | Yes | Yes | Active only while conditions met |
| **Mentorship** | 1.1× | — | Yes | Yes | 1.5× judgment multiplier (higher stakes) |
| **Project-Scoped** | 1.0× | — | Yes | Yes | Scoped to ESCO domain |

### 3b. Vouch Bleed-In Phases

| Phase | Days | Weight Active |
|-------|------|---------------|
| Phase 1 | 1–7 | 25% |
| Phase 2 | 8–14 | 50% |
| Active | 14+ | 100% |

### 3c. Vouch Outcomes

| Outcome | J Impact |
|---------|----------|
| Good (vouchee performs well) | +0.02 |
| Poor (vouchee underperforms) | −0.05 |
| Slashed (non-fraud) | −0.10 |
| Fraud (vouchee commits fraud) | −0.20 |

### 3d. Vouch Events

| Event | Description |
|-------|-------------|
| `vouch_created` | New vouch established |
| `vouch_phase_advanced` | Bleed-in phase progressed |
| `vouch_withdrawn` | Vouch withdrawn |
| `vouch_outcome_recorded` | Good / Poor / Slashed / Fraud outcome recorded |
| `vouch_weights_redistributed` | Budget weights recalculated |
| `vouch_skeptical_expired` | Skeptical vouch expired |
| `vouch_skeptical_renewed` | Skeptical vouch renewed |

---

## 4. Slash System

### 4a. Slash Triggers

| Trigger | Base Penalty | Requires Jury | Is Fraud | Description |
|---------|-------------|---------------|----------|-------------|
| `ConfirmedFraud` | 100% | Yes | Yes | Complete destruction + ban |
| `MetadataFraud` | 80% | Yes | No | Falsified credentials |
| `Collusion` | 60% | Yes | No | Coordinated manipulation |
| `RepeatedBadJudgment` | 30% | No | No | Poor vouching pattern |
| `VerificationFailure` | 20% | No | No | Failed verification duties |

### 4b. Cascade J-Impact on Vouchers

| Trigger | Voucher J Penalty |
|---------|-------------------|
| ConfirmedFraud / MetadataFraud | −0.20 |
| Collusion / RepeatedBadJudgment / VerificationFailure | −0.10 |

### 4c. Slash Events

| Event | Description |
|-------|-------------|
| `slash_triggered` | Slash initiated against user |
| `slash_cascade_calculated` | Cascade penalties computed |
| `slash_penalty_applied` | Penalty applied (with cascade level) |
| `slash_gdf_updated` | Global Defense Fund updated with burned reputation |
| `slash_user_banned` | Permanent ban |
| `slash_user_shadowed` | Temporary Shadow status |
| `slash_j_score_updated` | Voucher J score updated |
| `slash_case_created` | Case created for jury |
| `slash_case_approved` | Jury approved |
| `slash_case_rejected` | Jury rejected |
| `slash_auto_triggered` | Auto-slash from low J score |
| `slash_skeptical_collusion_detected` | Collusion farming pattern detected |

**Case Statuses:** `PendingReview` → `Approved` / `Rejected` → `Executed`

---

## 5. Decay System

| Event | Description |
|-------|-------------|
| `decay_competence_decayed` | Competence score decayed from inactivity |
| `decay_warning_issued` | Warning: "your score is about to decay" |
| `decay_activity_recorded` | Activity recorded, timer reset |
| `decay_genesis_decayed` | Genesis weight decayed (monthly) |
| `decay_genesis_sunset` | Genesis weight reached sunset threshold |
| `decay_job_executed` | Batch decay job completed |
| `decay_prevented` | Activity prevented scheduled decay |
| `decay_threshold_crossed` | Score crossed a tier boundary |

**Key parameters:**
- Inactivity trigger: **30 days**
- Competence half-life: **90 days**
- Genesis decay: monthly multiplier

---

## 6. Contribution System

### 6a. Contribution Types

| Type | Verification Model | Min Verifiers |
|------|-------------------|---------------|
| `continuous_contributions` | TimeWeighted | 1 |
| `defined_problems` | BinaryCompletion | 1 |
| `social_reputation_contribution` | QualitySpectrum | 3 |
| `subjective_creative_contribution` | PeerConsensus | 5 |

### 6b. Verification Models

| Model | Description |
|-------|-------------|
| **BinaryCompletion** | Done / not done |
| **QualitySpectrum** | 1–10 quality scale |
| **PeerConsensus** | Majority vote from peers |
| **TimeWeighted** | Consistency measured over time |

### 6c. Contribution Events

| Event | Description |
|-------|-------------|
| `contribution_processed` | Successfully processed with delta scores |
| `contribution_validation_failed` | Failed validation |
| `contribution_peer_review_outliers_detected` | Peer review detected outliers |

### 6d. Decay Policy

| Parameter | Default |
|-----------|---------|
| `half_life_days` | 180 |
| `min_retention` | 0.0 |
| `inactivity_accelerates` | true |

---

## 7. Judgment Events (JEvent)

These events modify the **J score** of the actor:

| JEvent | ΔJ | Context |
|--------|-----|---------|
| `VouchForHighPerformer` | **+0.02** | Vouchee performed well |
| `VouchForPoorPerformer` | **−0.05** | Vouchee underperformed |
| `VouchForSlashedUser` | **−0.10** | Vouchee got slashed |
| `VouchForFraud` | **−0.20** | Vouchee committed fraud |
| `GovernanceEndorsementUpheldImpact` | **+0.02** | Governance endorsement upheld (impactful) |
| `GovernanceEndorsementUpheldAccept` | **+0.01** | Governance endorsement upheld (accepted) |
| `GovernanceEndorsementOverturned` | **−0.05** | Governance endorsement overturned |
| `GovernanceEndorsementFraud` | **−0.10** | Governance endorsement was fraudulent |
| `DisputeOpenedUpheld` | **+0.02** | Dispute filed and upheld |
| `DisputeOpenedFrivolous` | **−0.03** | Dispute filed but frivolous |
| `JuryVotedWithMajority` | **+0.02** | Jury vote aligned with majority |
| `JuryVotedAgainstSubjective` | **0.00** | Dissented on subjective matter (no penalty) |
| `JuryVotedAgainstObjective` | **−0.03** | Dissented on objective fact |
| `CoWitnessValidated` | **+0.02** | Co-witness validation confirmed |
| `CoWitnessSlashed` | **−0.10** | Co-witness was slashed |

---

## 8. Jury System

| Event | Description |
|-------|-------------|
| `jury_selected` | User selected for jury duty |
| `jury_vote_cast` | Vote submitted |
| `jury_verdict_reached` | Majority verdict reached |
| `jury_j_score_updated` | J score updated from jury outcome |
| `jury_stake_returned` | Stake returned (voted with majority) |
| `jury_stake_burned` | Stake burned (voted against on objective) |
| `jury_voting_extended` | Voting period extended |
| `jury_case_created` | New jury case |
| `jury_case_finalized` | Case finalized |
| `jury_votes_revealed` | Votes revealed post-verdict |
| `jury_no_quorum_reached` | Quorum not met |

---

## 9. Problem & Solution Lifecycle

### 9a. Problem Events

| Event | Description |
|-------|-------------|
| `problem_created` | New problem posted |
| `problem_claimed` | Someone claimed to solve it |
| `problem_abandoned` | Claimer abandoned |
| `problem_solution_submitted` | Solution submitted |
| `problem_verified` | Solution verified |
| `problem_solution_rejected` | Solution rejected |
| `problem_closed` | Problem closed |
| `problem_difficulty_adjusted` | Difficulty recalibrated |
| `problem_status_changed` | Status transition |
| `problem_uncertainty_flagged` | Flagged as uncertain / ambiguous |
| `problem_uncertainty_cleared` | Uncertainty resolved |

### 9b. Solution Events

| Event | Description |
|-------|-------------|
| `solution_submitted` | Solution submitted |
| `solution_verifier_assigned` | Verifier assigned |
| `solution_verified` | Verification passed |
| `solution_rejected` | Verification rejected |
| `solution_revision_requested` | Revision requested |
| `solution_revised` | Revised solution submitted |
| `solution_por_validated` | Proof of Reality validated |
| `solution_status_changed` | Status transition |

---

## 10. Verification System

| Event | Description |
|-------|-------------|
| `verification_verifier_selected` | Verifier selected from eligible pool |
| `verification_decision_made` | Verifier submitted decision |
| `verification_record_created` | Verification record persisted |
| `verification_j_score_updated` | Verifier J score updated |
| `verification_timed_out` | Verification window expired |

---

## 11. Skill & Polymath

| Event | Description |
|-------|-------------|
| `skill_transfer_calculated` | Cross-domain transfer calculated |
| `skill_polymath_achieved` | 3+ domains at Pillar level → polymath bonus |
| `skill_polymath_lost` | Dropped below 3 Pillar-level domains |
| `skill_domain_added` | New domain competence registered |
| `skill_domain_removed` | Domain competence dropped |
| `skill_effective_competence_calculated` | Effective competence computed (with transfers) |
| `skill_competence_updated` | Domain competence score changed |
| `skill_polymath_progress_updated` | Progress toward polymath status |

---

## 12. Epoch System

| Event | Description |
|-------|-------------|
| `epoch_started` | New epoch begins |
| `epoch_completed` | Epoch completed successfully |
| `epoch_failed` | Epoch failed |
| `epoch_skipped` | Epoch skipped (no qualifying activity) |
| `epoch_job_started` | Epoch job processing started |
| `epoch_job_completed` | Epoch job completed |
| `epoch_job_failed` | Epoch job failed |

---

## 13. Genesis System

Genesis handles initial reputation bootstrap for new users.

| Event | Description |
|-------|-------------|
| `genesis_*` | Custom typed events from genesis initialization |

**Genesis Decay:** Monthly multiplier applied to genesis-bootstrapped weights. Once sunset threshold reached, genesis weight is fully organic.

---

## 14. Recovery System

| Event | Description |
|-------|-------------|
| `recovery_entered_shadow` | User entered Shadow status |
| `recovery_entered_probation` | Progressed to probation |
| `recovery_fully_restored` | Fully restored to normal status |
| `recovery_extended` | Recovery period extended (re-offense) |
| `recovery_early_release` | Early release from recovery |

---

## 15. Proof of Reality (PoR)

| Event | Description |
|-------|-------------|
| `por_evidence_submitted` | Evidence submitted |
| `por_validation_passed` | Evidence validated |
| `por_validation_failed` | Evidence failed validation |
| `por_tampering_detected` | Tampering detected |
| `por_ai_content_flagged` | AI-generated content flagged |
| `por_manual_review_completed` | Manual review completed |
| `por_evidence_expired` | Evidence expired |
| `por_media_added` | Media attachment added |
| `por_requirements_updated` | PoR requirements changed |

---

## 16. Identity

| Event | Description |
|-------|-------------|
| `identity_level_changed` | Identity verification level changed |

---

## 17. Activity Signals (Type B)

Rolling-window behavioral metrics tracked per user:

| Signal | Description | Range |
|--------|-------------|-------|
| `pattern_window_days` | Rolling measurement window | max 30 days |
| `activity_streak_days` | Consecutive active days | 0–∞ |
| `contribution_count_in_window` | Contributions in rolling window | 0–∞ |
| `gap_days_since_last` | Days since last contribution | 0–∞ |
| `quality_average_score` | Average quality across contributions | 0–10 |

**Reset thresholds:**
- `ACTIVITY_WINDOW_DAYS` = 30
- `STREAK_RESET_GAP_DAYS` = 14

---

## 18. Configuration Constants

| Constant | Value | Used By |
|----------|-------|---------|
| `ACTIVITY_WINDOW_DAYS` | 30 | Type B signals |
| `STREAK_RESET_GAP_DAYS` | 14 | Activity streak |
| `SKEPTICAL_DAMPENING_PER_VOUCH` | 0.10 | Vouch system |
| `SKEPTICAL_DAMPENING_FLOOR` | 0.70 | Vouch system |
| `VERIFICATION_THRESHOLD` (J) | 0.30 | Verification eligibility |
| `inactivity_trigger_days` | 30 | Decay system |
| `competence_half_life_days` | 90 | Decay system |
| `contribution_half_life_days` | 180 | Contribution decay |
| Jury eligibility percentile | 85 | Jury selection |

---

## 19. UI Mapping Candidates

> This section is the bridge between tandang signals and gotong-royong UI.
> Status: **Draft** — to be refined during Phase 2 implementation.

### 19a. Reaction Chips (Quick Strike — Phase 2)

Candidate signals that could power tap-to-react interaction:

| Chip Concept | Tandang Signal | Visual | User Action |
|-------------|----------------|--------|-------------|
| **Vouch** (trust) | `vouch_created` → Positive vouch | 🤝 green chip | "I trust this person / content" |
| **Skeptis** (skeptical) | `vouch_created` → Skeptical vouch | 🔍 amber chip | "I have concerns" |
| **Saya Ikut** (I join) | `problem_claimed` | 🙋 blue chip | Quick-join a problem / initiative |
| **Sudah Lihat** (witnessed) | `por_evidence_submitted` | 👁️ neutral chip | "I witnessed this in real life" |
| **Bagus** (quality upvote) | QualitySpectrum verification score | ⭐ gold chip | Quality endorsement (maps to C score) |
| **Perlu Dicek** (needs checking) | `problem_uncertainty_flagged` | ⚠️ red chip | Flag content for verification |

### 19b. Badge / Trust Indicators

| Badge | Source Signal | Where Shown |
|-------|-------------|-------------|
| Tier badge (◆◆◆◆) | User tier percentile | Avatar, profile, card author |
| Streak flame | `activity_streak_days` | Card author, profile |
| Polymath star | `skill_polymath_achieved` | Profile badge |
| Verified contributor | `identity_level_changed` | Author label |
| Shadow indicator | `recovery_entered_shadow` | Dimmed avatar |

### 19c. Progress / Status Indicators

| UI Element | Source Signal | Where Shown |
|------------|-------------|-------------|
| Quorum bar | `jury_no_quorum_reached` / vote count | Card footer |
| Competence decay warning | `decay_warning_issued` | Profile notification |
| Recovery progress | `recovery_entered_probation` → `recovery_fully_restored` | Profile |
| Bleed-in progress | `vouch_phase_advanced` (25% → 50% → 100%) | Vouch detail view |
| Verification status | `verification_decision_made` | Solution card |

### 19d. Feed Event Types (Pulse integration)

These tandang events could generate feed items in the Pulse feed:

| Feed Category | Tandang Events | Card Mood |
|---------------|---------------|-----------|
| **Aktivitas Warga** | `contribution_processed`, `problem_created`, `solution_submitted` | harapan (green) |
| **Keputusan** | `jury_verdict_reached`, `slash_case_approved/rejected` | keputusan (warm) |
| **Peringatan** | `decay_warning_issued`, `slash_triggered`, `por_tampering_detected` | peringatan (red) |
| **Pencapaian** | `skill_polymath_achieved`, `recovery_fully_restored`, tier upgrade | harapan (green) |
| **Verifikasi** | `solution_verified`, `por_validation_passed` | netral (neutral) |

### 19e. Signal-to-Octalysis Mapping

How tandang signals map to the 8 Core Drives:

| Core Drive | Tandang Signals | UI Expression |
|------------|----------------|---------------|
| ① Epic Meaning | Tier system, polymath | Tier badges, purpose framing |
| ② Accomplishment | C score, solve rewards, streaks | Progress bars, streak flames |
| ③ Empowerment | Problem claiming, solution submission | "Saya Ikut" button, solution editor |
| ④ Ownership | Vouch budget, domain competence | "My vouches" panel, domain portfolio |
| ⑤ Social Influence | Vouch system, follower count, jury | Vouch chips, trust indicators |
| ⑥ Scarcity | Vouch limits per tier, jury selection | "X vouches remaining", jury badge |
| ⑦ Unpredictability | Epoch outcomes, jury results, decay | Countdown timers, live badges |
| ⑧ Loss Avoidance | Decay warnings, slash penalties, streak reset | Warning toasts, decay countdown |

---

## Appendix: Full Event Type Index

**120+ events across 17 categories:**

| Category | Event Count | Key Events |
|----------|-------------|------------|
| Vouch | 7 | created, phase_advanced, withdrawn, outcome_recorded |
| Decay | 8 | competence_decayed, warning_issued, threshold_crossed |
| Slash | 12 | triggered, cascade, penalty_applied, user_banned/shadowed |
| Problem | 11 | created, claimed, verified, uncertainty_flagged |
| Solution | 8 | submitted, verified, rejected, por_validated |
| Verification | 5 | verifier_selected, decision_made, timed_out |
| Jury | 11 | selected, vote_cast, verdict_reached, no_quorum |
| Skill | 8 | polymath_achieved, domain_added, transfer_calculated |
| Epoch | 7 | started, completed, job_started, job_completed |
| Genesis | 11+ | Custom typed genesis events |
| Recovery | 5 | entered_shadow, entered_probation, fully_restored |
| Identity | 1 | level_changed |
| PoR | 9 | evidence_submitted, tampering_detected, ai_content_flagged |
| Contribution | 3 | processed, validation_failed, outliers_detected |
| **Total** | **~120+** | |

---

*This document will evolve as we implement Phase 2 (Quick Strike) and begin integrating tandang APIs into gotong-royong.*

# Tandang Gap Implementation Prompts

Each prompt below is self-contained. Feed one at a time to an LLM with access to the Tandang codebase at `/path/to/tandang/markov-engine/`. The prompts reference exact file paths, types, and conventions from the codebase.

---

## Gap 1: VouchContext (Entity-Scoped Vouch Metadata)

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/vouch/vouch_entity.rs` has the `Vouch` struct:

```rust
pub struct Vouch {
    id: VouchId,
    voucher: UserId,
    vouchee: UserId,
    domain: Option<SkillId>,
    weight: Decimal,
    stake: Decimal,
    voucher_reputation_at_vouch: Option<Decimal>,
    vouch_type: VouchType,  // Positive | Skeptical
    status: VouchStatus,
    created_at: DateTime<Utc>,
    phase_advanced_at: DateTime<Utc>,
    outcome: Option<VouchOutcome>,  // Good | Poor | Slashed | Fraud
    expires_at: Option<DateTime<Utc>>,
}
```

Vouches are person-to-person with optional `domain` scoping. There's no way to tag a vouch with the *reason* it was created (co-witnessing, verified execution, impact attestation, sensemaking validation).

### What to build

Add an optional `source_context` field to `Vouch` that records WHY the vouch was created, without changing PageRank computation. PageRank still operates on user-to-user edges — the context is metadata for auditability, clawback scoping, and UI display.

**Domain crate changes** (`crates/domain/src/vouch/`):

1. Add a new enum in `vouch_entity.rs` (or a new file `vouch_context.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VouchContext {
    /// Co-witnessing a claim — the voucher attests the vouchee saw something real
    CoWitness { witness_id: Uuid },
    /// Verifier accepted executor's solution — strongest GR vouch
    VerifiedExecution { task_id: Uuid },
    /// Beneficiary attests to impact from witness chain
    ImpactAttestation { witness_id: Uuid },
    /// Sensemaker's criteria led to good outcomes (retroactive)
    SensemakingValidation { witness_id: Uuid, criteria_hash: String },
    /// Manual / direct vouch (existing behavior, no GR context)
    Direct,
}
```

2. Add field to `Vouch` struct:
```rust
source_context: VouchContext,  // default: VouchContext::Direct
```

3. Update `Vouch::new()` constructor to accept optional `VouchContext`, defaulting to `Direct`.

4. Update `VouchBehavior` implementations in `vouch/behavior.rs` — no logic changes, just pass-through. `source_context` does NOT affect `effective_weight()`.

**Application crate changes** (`crates/application/src/`):

5. Update `CreateVouchInput` in `services/commands.rs` to include optional `source_context: Option<VouchContext>`.

6. Update `VouchCommands::create_vouch()` to pass context through.

**Infrastructure crate changes** (`crates/infrastructure/src/`):

7. DB migration: add `source_context JSONB DEFAULT '{"Direct":{}}' NOT NULL` column to `vouches` table. Use JSONB for the enum serialization (serde_json handles Rust enums as tagged objects).

8. Update `persistence/vouch_repo.rs` — read/write `source_context` field.

**API crate changes** (`crates/api/src/`):

9. Update `dto/vouch.rs` — add `source_context: Option<String>` to `CreateVouchRequest` and `source_context: String` to `VouchResponse`.

10. Update `routes/vouch.rs` handler to map DTO ↔ domain.

**Tests:**

11. Add BDD scenario in `tests/bdd/features/trust/`:
```gherkin
Feature: Vouch Context Metadata
  Scenario: Create vouch with CoWitness context
    Given user Alice co-witnesses Bob's claim W-001
    When system creates implicit vouch from Alice to Bob
    Then vouch source_context is CoWitness with witness_id W-001
    And vouch effective_weight is unchanged from a Direct vouch

  Scenario: Clawback scoped to witness chain
    Given vouch from Alice to Bob with context CoWitness { witness_id: W-001 }
    And vouch from Alice to Carol with context Direct
    When witness W-001 is flagged as fraud and clawback initiated
    Then Alice-to-Bob vouch is affected by clawback
    And Alice-to-Carol vouch is NOT affected
```

**Critical constraint:** PageRank computation in `crates/domain/src/markov/` must NOT change. `source_context` is metadata only.

---

## Gap 2: Witness as Target Type

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/problem/endorsement.rs` has an `Endorsement` struct that targets problems. `crates/domain/src/problem/multi_verifier.rs` has `MultiVerification` that references `problem_id`. Target types are identified by string discriminators (e.g., `"problem"`, `"solution"`, `"proposal"`, `"content"`).

### What to build

Add `"witness"`, `"transition"`, and `"impact"` as valid target discriminator strings. This is a config-level change.

**Domain crate changes:**

1. Find where target discriminators are validated (likely in `endorsement.rs` or a shared types file). Add the new strings to the allowed list:

```rust
const VALID_TARGET_TYPES: &[&str] = &[
    "problem", "solution", "proposal", "content",
    // NEW: Gotong Royong target types
    "witness",      // A witness claim
    "transition",   // A stage transition proposal
    "impact",       // An impact attestation
];
```

2. If there's a `TargetType` enum instead of strings, add variants:
```rust
pub enum TargetType {
    Problem,
    Solution,
    Proposal,
    Content,
    // NEW
    Witness,
    Transition,
    Impact,
}
```

**No DB migration needed.** No domain logic changes. The endorsement and multi-verification systems should already work with any target type — we're just expanding the allowed set.

**Tests:**

3. Add test:
```gherkin
Scenario: Endorse a witness target
    Given endorsement target type "witness" with id W-001
    When endorsement is created
    Then endorsement is valid and stored
```

---

## Gap 3: Governance Budget Primitive

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/vouch/budget.rs` has `VouchBudget`:
```rust
pub struct VouchBudget {
    total_budget: Decimal,      // R × 0.3
    allocated_budget: Decimal,
    active_vouches: u32,
}
```

This tracks vouch-specific budget. There's no governance budget concept for endorsing transitions.

`crates/domain/src/governance/account.rs` has `AccountService` with account-level operations but no budget primitive.

### What to build

A new domain module `governance_budget/` that implements a unified influence pool shared between vouching and governance endorsements.

**Domain crate changes** (`crates/domain/src/`):

1. Create new module `governance_budget/` with:

```rust
// governance_budget/mod.rs
mod budget;
mod service;
mod events;
pub use budget::*;
pub use service::*;
pub use events::*;

// governance_budget/budget.rs
use rust_decimal::Decimal;
use crate::types::ids::UserId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GovernanceBudget {
    user_id: UserId,
    /// Total influence capacity = R × budget_factor (0.3)
    total_influence: Decimal,
    /// Amount allocated to active vouches
    vouch_allocated: Decimal,
    /// Amount locked in pending governance endorsements
    governance_locked: Decimal,
}

impl GovernanceBudget {
    pub fn available(&self) -> Decimal {
        self.total_influence - self.vouch_allocated - self.governance_locked
    }

    pub fn can_endorse(&self, amount: Decimal) -> bool {
        self.available() >= amount
    }

    pub fn lock_for_endorsement(&mut self, proposal_id: Uuid, amount: Decimal) -> Result<GovernanceLock, BudgetError> {
        if !self.can_endorse(amount) {
            return Err(BudgetError::InsufficientBudget);
        }
        self.governance_locked += amount;
        Ok(GovernanceLock { proposal_id, amount, locked_at: Utc::now() })
    }

    pub fn release(&mut self, lock: &GovernanceLock, outcome: TransitionOutcome) -> BudgetRelease {
        self.governance_locked -= lock.amount;
        let returned = match outcome {
            TransitionOutcome::UpheldReachedImpact => lock.amount * Decimal::new(105, 2),  // 105%
            TransitionOutcome::UpheldReachedAccept => lock.amount,                          // 100%
            TransitionOutcome::UpheldStalled => lock.amount * Decimal::new(50, 2),          // 50%
            TransitionOutcome::Overturned => Decimal::ZERO,                                 // burned
            TransitionOutcome::FraudDiscovered => Decimal::ZERO,                            // burned + J penalty
        };
        BudgetRelease { returned, burned: lock.amount - returned.min(lock.amount) }
    }

    /// Called when vouch budget changes — keeps total in sync
    pub fn sync_vouch_allocation(&mut self, new_vouch_allocated: Decimal) {
        self.vouch_allocated = new_vouch_allocated;
    }

    /// Called when user reputation changes — recalculates total
    pub fn recalculate_total(&mut self, reputation: Decimal, budget_factor: Decimal) {
        self.total_influence = reputation * budget_factor;
    }
}

#[derive(Debug, Clone)]
pub struct GovernanceLock {
    pub proposal_id: Uuid,
    pub amount: Decimal,
    pub locked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionOutcome {
    UpheldReachedImpact,   // 100% return + 5% bonus
    UpheldReachedAccept,   // 100% return
    UpheldStalled,         // 50% return
    Overturned,            // 0% return (burned)
    FraudDiscovered,       // 0% return + J penalty
}

#[derive(Debug, Clone)]
pub struct BudgetRelease {
    pub returned: Decimal,
    pub burned: Decimal,
}

#[derive(Debug)]
pub enum BudgetError {
    InsufficientBudget,
}
```

2. Register module in `crates/domain/src/lib.rs`.

3. Integration with `VouchBudget`: when a vouch is created/withdrawn, call `GovernanceBudget::sync_vouch_allocation()` to keep the unified pool in sync.

**Application crate changes:**

4. Add commands:
```rust
pub struct LockGovernanceBudgetInput {
    pub user_id: UserId,
    pub proposal_id: Uuid,
    pub amount: Decimal,
}

pub struct ReleaseGovernanceBudgetInput {
    pub proposal_id: Uuid,
    pub outcome: TransitionOutcome,
}
```

5. Add query: `get_governance_budget(user_id) -> GovernanceBudget`

**Infrastructure crate changes:**

6. DB migration:
```sql
CREATE TABLE governance_budget (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    total_influence DECIMAL NOT NULL DEFAULT 0,
    vouch_allocated DECIMAL NOT NULL DEFAULT 0,
    governance_locked DECIMAL NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE governance_locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    proposal_id UUID NOT NULL,
    amount DECIMAL NOT NULL,
    locked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    released_at TIMESTAMPTZ,
    outcome TEXT,
    UNIQUE(user_id, proposal_id)
);
```

7. Implement `GovernanceBudgetRepository` in `persistence/`.

**API crate changes:**

8. Add endpoints:
```
GET  /api/v1/users/:id/governance-budget  → GovernanceBudgetResponse
POST /api/v1/governance/lock              → LockResponse
POST /api/v1/governance/release           → ReleaseResponse
```

**Tests:**

9. BDD scenarios:
```gherkin
Feature: Governance Budget
  Scenario: Unified pool shared between vouches and governance
    Given Alice has reputation 1.0 and budget_factor 0.3
    Then Alice total_influence is 0.30
    When Alice creates vouch with weight 0.10
    Then Alice available governance budget is 0.20
    When Alice endorses proposal P-001 locking 0.15
    Then Alice available is 0.05
    And Alice cannot endorse proposal P-002 for 0.10

  Scenario: Budget return on upheld transition reaching Impact
    Given Alice locked 0.10 on proposal P-001
    When proposal P-001 witness reaches Impact stage
    Then Alice receives 0.105 back (100% + 5% bonus)
    And Alice available increases by 0.105

  Scenario: Budget burned on overturned transition
    Given Alice locked 0.10 on proposal P-001
    When proposal P-001 is overturned by dispute
    Then Alice receives 0 back
    And Alice available does NOT increase
```

---

## Gap 4: Witness Complexity Dimension

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/problem/difficulty_estimator.rs` has:
```rust
pub trait DifficultyEstimator: Send + Sync {
    fn estimate(&self, factors: &DifficultyFactors) -> DifficultyEstimate;
}

pub struct DifficultyFactors {
    description_length: usize,
    keyword_complexity: Decimal,
    required_dependencies: u32,
}
```

This estimates TASK difficulty (binary: easy/medium/hard). There's no concept of WITNESS complexity — how complex the situation being witnessed is (multi-stakeholder, multi-domain, evidence requirements).

### What to build

Add a `WitnessComplexity` type that's separate from task difficulty. Witness complexity rewards sensemaking; task difficulty rewards execution.

**Domain crate changes:**

1. Add to `problem/` module (new file `witness_complexity.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessComplexity {
    /// AI-estimated complexity [0.1, 0.9]
    pub ai_estimate: Decimal,
    /// Human (sensemaker) override at Define stage
    pub sensemaker_override: Option<Decimal>,
    /// Weighted average: if override exists, 0.4 × AI + 0.6 × human; else AI only
    pub final_score: Decimal,
}

impl WitnessComplexity {
    pub fn from_ai(estimate: Decimal) -> Self {
        let clamped = estimate.max(dec!(0.1)).min(dec!(0.9));
        Self {
            ai_estimate: clamped,
            sensemaker_override: None,
            final_score: clamped,
        }
    }

    pub fn apply_sensemaker_override(&mut self, override_val: Decimal) {
        let clamped = override_val.max(dec!(0.1)).min(dec!(0.9));
        self.sensemaker_override = Some(clamped);
        self.final_score = (self.ai_estimate * dec!(0.4)) + (clamped * dec!(0.6));
    }
}
```

2. Extend `DifficultyEstimator` trait or create a separate `WitnessComplexityEstimator` trait:
```rust
pub trait WitnessComplexityEstimator: Send + Sync {
    fn estimate_witness_complexity(&self, factors: &WitnessComplexityFactors) -> WitnessComplexity;
}

pub struct WitnessComplexityFactors {
    pub domain_count: u32,           // how many ESCO domains
    pub stakeholder_count: u32,      // how many parties affected
    pub evidence_requirement_level: u32,  // estimated evidence burden
    pub description_length: usize,
    pub historical_median_complexity: Option<Decimal>,  // for similar witnesses
}
```

3. Integrate with sensemaker scoring formula. In the contribution scoring system (`contribution/builtin/subjective.rs` or wherever Type C scoring happens), multiply sensemaker reward by `witness_complexity.final_score`:
```rust
sensemaker_reward = base × witness_complexity.final_score × outcome_quality
```

**Infrastructure:**

4. DB migration: add `witness_complexity JSONB` column to the problems table (or whatever table stores witness-level metadata).

**API:**

5. Extend Skill API endpoint (if it exists) or create:
```
POST /api/v1/skills/estimate-complexity
  Body: { "description": "...", "domain_count": 3, "stakeholder_count": 5 }
  Response: { "ai_estimate": 0.65, "final_score": 0.65 }

PATCH /api/v1/problems/:id/witness-complexity
  Body: { "sensemaker_override": 0.72 }
  Response: { "ai_estimate": 0.65, "sensemaker_override": 0.72, "final_score": 0.692 }
```

---

## Gap 5: J Impact Table for Governance Events

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/reputation/judgment.rs` has `JudgmentScore` with outcome-based impacts. `crates/domain/src/vouch/vouch_entity.rs` has `VouchOutcome` enum:
```rust
pub enum VouchOutcome {
    Good,      // +0.02 J
    Poor,      // -0.05 J
    Slashed,   // -0.10 J
    Fraud,     // -0.20 J
}
```

These cover vouch outcomes. There are NO governance-specific J events.

### What to build

Extend the J event system with governance-specific variants.

**Domain crate changes:**

1. In `reputation/judgment.rs` (or wherever J impacts are computed), add new event types. If there's a `JEvent` or `JudgmentEvent` enum, extend it:

```rust
pub enum JEvent {
    // EXISTING vouch outcomes
    VouchForHighPerformer,           // +0.02
    VouchForPoorPerformer,           // -0.05
    VouchForSlashedUser,             // -0.10
    VouchForFraud,                   // -0.20

    // NEW: Governance endorsement outcomes
    GovernanceEndorsementUpheldImpact,   // +0.02 (endorsed transition, witness reached Impact)
    GovernanceEndorsementUpheldAccept,   // +0.01 (endorsed transition, witness reached Accept)
    GovernanceEndorsementOverturned,     // -0.05 (endorsed transition overturned by dispute)
    GovernanceEndorsementFraud,          // -0.10 (endorsed transition involved in fraud)

    // NEW: Dispute outcomes
    DisputeOpenedUpheld,                 // +0.02 (opened dispute, it was upheld — good challenge)
    DisputeOpenedFrivolous,              // -0.03 (opened dispute, it was frivolous)

    // NEW: Jury service
    JuryVotedWithMajority,               // +0.02
    JuryVotedAgainstSubjective,          // 0 (disagree on subjective matter — no penalty)
    JuryVotedAgainstObjective,           // -0.03 (objectively wrong assessment)

    // NEW: Co-witness accuracy
    CoWitnessValidated,                  // +0.02 (co-witnessed claim later validated)
    CoWitnessSlashed,                    // -0.10 (co-witnessed claim later found fraudulent)
}

impl JEvent {
    pub fn j_delta(&self) -> Decimal {
        match self {
            Self::VouchForHighPerformer => dec!(0.02),
            Self::VouchForPoorPerformer => dec!(-0.05),
            Self::VouchForSlashedUser => dec!(-0.10),
            Self::VouchForFraud => dec!(-0.20),
            Self::GovernanceEndorsementUpheldImpact => dec!(0.02),
            Self::GovernanceEndorsementUpheldAccept => dec!(0.01),
            Self::GovernanceEndorsementOverturned => dec!(-0.05),
            Self::GovernanceEndorsementFraud => dec!(-0.10),
            Self::DisputeOpenedUpheld => dec!(0.02),
            Self::DisputeOpenedFrivolous => dec!(-0.03),
            Self::JuryVotedWithMajority => dec!(0.02),
            Self::JuryVotedAgainstSubjective => dec!(0.0),
            Self::JuryVotedAgainstObjective => dec!(-0.03),
            Self::CoWitnessValidated => dec!(0.02),
            Self::CoWitnessSlashed => dec!(-0.10),
        }
    }
}
```

2. Update wherever J score is applied (likely `ReputationService` in `reputation/service.rs`) to handle the new event types.

**Application crate:**

3. Add command handlers that emit the new J events when governance outcomes are determined:
   - When a transition outcome is known → emit appropriate `GovernanceEndorsement*` event for each endorser
   - When a dispute is resolved → emit `DisputeOpened*` for challenger, `JuryVoted*` for jurors
   - When a co-witnessed claim reaches Accept/Impact → emit `CoWitnessValidated` for co-witnesses
   - When a co-witnessed claim is slashed → emit `CoWitnessSlashed` for co-witnesses

**No DB migration.** J events are computed from existing event data. The `JudgmentScore` value type doesn't change — only the inputs that modify it expand.

**Tests:**

4. BDD:
```gherkin
Feature: Governance J Events
  Scenario: Endorser gains J for upheld transition reaching Impact
    Given Alice endorsed transition T-001
    When witness for T-001 reaches Impact stage
    Then Alice judgment increased by +0.02

  Scenario: Co-witness loses J for slashed claim
    Given Alice co-witnessed claim W-001
    When W-001 is flagged as fraud and slashed
    Then Alice judgment decreased by -0.10
```

---

## Gap 6: Explore Track Composition (Configurable Consensus Thresholds)

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/problem/multi_verifier.rs` has:
```rust
pub enum ConsensusType {
    Majority,       // > 50%
    Supermajority,  // > 66%
}
```

This is hardcoded to two options. The Gotong Royong Explore track needs:
- 60% threshold for Hypotheses phase (standard)
- 70% threshold for Conclusion phase (elevated — conclusions require stronger agreement)

### What to build

Make `ConsensusType` support arbitrary thresholds.

**Domain crate changes:**

1. In `problem/multi_verifier.rs`, replace the enum:

```rust
pub enum ConsensusType {
    Majority,                    // > 50%
    Supermajority,               // > 66%
    Custom { threshold: Decimal }, // Arbitrary threshold [0.5, 1.0]
}

impl ConsensusType {
    pub fn threshold(&self) -> Decimal {
        match self {
            Self::Majority => dec!(0.5),
            Self::Supermajority => dec!(0.66),
            Self::Custom { threshold } => *threshold,
        }
    }

    pub fn is_met(&self, approvals: Decimal, total: Decimal) -> bool {
        if total == Decimal::ZERO { return false; }
        (approvals / total) >= self.threshold()
    }
}
```

2. Wherever `MultiVerification` checks consensus, use `consensus_type.is_met(approvals, total)` instead of hardcoded comparison.

3. Add composition helper for GR tracks:

```rust
pub struct TrackCompositionConfig {
    pub track_type: String,  // "explore", "resolve", "celebrate"
    pub phases: Vec<PhaseConfig>,
}

pub struct PhaseConfig {
    pub phase_name: String,
    pub contribution_type: String,   // "type_a", "type_c", "type_d"
    pub verification_model: VerificationModel,
    pub consensus_type: ConsensusType,
}
```

This is optional — the main deliverable is making `ConsensusType` support `Custom { threshold }`.

**No DB migration.** The consensus type is stored as part of `MultiVerification` state. If it's already serialized as JSON/enum, just add the new variant. If it's stored as a string, add `"custom_70"` etc.

**Tests:**

```gherkin
Feature: Configurable Consensus Thresholds
  Scenario: Explore Conclusion requires 70% consensus
    Given multi-verification with ConsensusType::Custom { threshold: 0.70 }
    And 5 verifiers total
    When 3 approve (60%)
    Then consensus is NOT met

    When 4 approve (80%)
    Then consensus IS met
```

---

## Gap 7: Celebrate Track Composition

**Codebase:** `markov-engine/`

### What exists

Same `MultiVerification` and contribution type infrastructure as Gap 6. Type D (`SocialReputationContribution`) exists in `contribution/builtin/social.rs`. Endorsement exists in `problem/endorsement.rs`.

### What to build

This is purely application-layer composition — no new Tandang primitives needed. The "gap" is documenting how GR composes existing primitives for the Celebrate track.

**What the GR application layer should do:**

```
Celebrate Phase → Tandang Primitive → Verification

Seed            → Type D contribution (SocialReputationContribution)  → Social quality spectrum
Corroborate     → Type D + Endorsement (from co-witnesses)            → Peer consensus (60%)
Recognize       → MultiVerification with Majority consensus           → Quorum vote
Impact          → Type D contribution (from beneficiaries)            → Impact attestation
```

**No Tandang code changes needed IF Gap 6 is done** (configurable consensus thresholds). The GR app layer creates `MultiVerification` instances with appropriate `ConsensusType` for each Celebrate phase.

If you want to add a composition config registry in Tandang for reusability:

```rust
// In domain/src/composition/ (new module, optional)
pub struct CompositionTemplate {
    pub name: String,
    pub phases: Vec<PhaseTemplate>,
}

pub struct PhaseTemplate {
    pub name: String,
    pub contribution_type_id: String,
    pub consensus_type: ConsensusType,
    pub evidence_requirements: Option<Vec<String>>,
}
```

Register Celebrate and Explore as templates. But this is optional — GR can hard-code the composition in its own application layer.

---

## Gap 8: Context Triad as Evidence Profile

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/por/types.rs` has:
```rust
pub struct PoREvidence {
    id: Uuid,
    task_id: String,
    media_sources: Vec<MediaSource>,
    geolocation: GeoLocation,
    timestamp_range: (DateTime<Utc>, DateTime<Utc>),
}

pub enum MediaType { Photo, Video, Audio, Document }

pub struct GeoLocation { ... }
```

PoR has photos and geolocation but NO structured "Context Triad" (Visual + Locational + Corroborative) with per-transition requirements. The validation in `por/validation.rs` checks individual evidence items but doesn't enforce "at least 2-of-3 triad elements present."

### What to build

Add `ContextTriad` and `ContextTriadRequirement` types to PoR, and a rule-based validator.

**Domain crate changes** (`crates/domain/src/por/`):

1. New file `context_triad.rs`:

```rust
use uuid::Uuid;
use crate::types::ids::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTriad {
    pub visual: Option<VisualEvidence>,
    pub locational: Option<LocationEvidence>,
    pub corroborative: Vec<CoWitnessRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEvidence {
    pub media_id: Uuid,
    pub media_type: MediaType,  // reuse existing
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationEvidence {
    pub geolocation: Option<GeoLocation>,  // reuse existing
    pub place_label: Option<String>,
    pub coarse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoWitnessRef {
    pub user_id: UserId,
    pub integrity_at_attestation: Decimal,  // must be > 0
}

/// What's required at a specific transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTriadRequirement {
    pub stage_transition: String,   // e.g. "seed_to_define"
    pub min_of_three: u8,           // 0, 1, 2, or 3
    pub require_visual: bool,       // specifically require visual
    pub require_locational: bool,   // specifically require locational
    pub min_corroborators: u8,      // minimum co-witnesses (0 = not required)
}

impl ContextTriad {
    /// Count how many of the three elements are present
    pub fn elements_present(&self) -> u8 {
        let v = if self.visual.is_some() { 1 } else { 0 };
        let l = if self.locational.is_some() { 1 } else { 0 };
        let c = if !self.corroborative.is_empty() { 1 } else { 0 };
        v + l + c
    }

    /// Validate against a requirement
    pub fn meets_requirement(&self, req: &ContextTriadRequirement) -> TriadValidationResult {
        let mut failures = Vec::new();

        if self.elements_present() < req.min_of_three {
            failures.push(format!(
                "Need {}-of-3 triad elements, have {}",
                req.min_of_three, self.elements_present()
            ));
        }
        if req.require_visual && self.visual.is_none() {
            failures.push("Visual evidence required".into());
        }
        if req.require_locational && self.locational.is_none() {
            failures.push("Locational evidence required".into());
        }
        let valid_corroborators = self.corroborative.iter()
            .filter(|c| c.integrity_at_attestation > Decimal::ZERO)
            .count() as u8;
        if valid_corroborators < req.min_corroborators {
            failures.push(format!(
                "Need {} co-witnesses with I > 0, have {}",
                req.min_corroborators, valid_corroborators
            ));
        }

        if failures.is_empty() {
            TriadValidationResult::Pass
        } else {
            TriadValidationResult::Fail { missing: failures }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TriadValidationResult {
    Pass,
    Fail { missing: Vec<String> },
}
```

2. Register in `por/mod.rs`.

3. Integrate with `PoRValidator` in `por/validation.rs` — add a method that validates `ContextTriad` against `ContextTriadRequirement`.

**Infrastructure:**

4. DB migration:
```sql
ALTER TABLE por_evidence ADD COLUMN context_triad JSONB;

CREATE TABLE context_triad_requirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    track TEXT NOT NULL,           -- 'resolve', 'celebrate', 'explore'
    stage_transition TEXT NOT NULL, -- 'seed_to_define', 'define_to_path', etc.
    min_of_three SMALLINT NOT NULL DEFAULT 1,
    require_visual BOOLEAN NOT NULL DEFAULT FALSE,
    require_locational BOOLEAN NOT NULL DEFAULT FALSE,
    min_corroborators SMALLINT NOT NULL DEFAULT 0,
    UNIQUE(track, stage_transition)
);

-- Seed default requirements
INSERT INTO context_triad_requirements (track, stage_transition, min_of_three) VALUES
    ('resolve', 'seed_to_define', 1),
    ('resolve', 'define_to_path', 2),
    ('resolve', 'path_to_execute', 2),
    ('resolve', 'execute_to_accept', 3),
    ('celebrate', 'seed_to_corroborate', 1),
    ('celebrate', 'corroborate_to_recognize', 2),
    ('explore', 'seed_to_hypotheses', 0),
    ('explore', 'hypotheses_to_experiments', 1),
    ('explore', 'experiments_to_conclusion', 2);
```

**API:**

5. Extend PoR submission endpoint to accept optional `context_triad` field.
6. Add query endpoint: `GET /api/v1/triad-requirements/:track/:transition`

---

## Gap 9: Emergency Brake Primitive

**Codebase:** `markov-engine/`

### What exists

Nothing. No emergency brake concept exists in Tandang.

### What to build

A new domain module that allows high-trust users to immediately freeze a transition and trigger a jury audit.

**Domain crate changes:**

1. New module `emergency/` (or add to `governance/`):

```rust
// emergency/brake.rs

#[derive(Debug, Clone)]
pub struct EmergencyBrake {
    pub id: Uuid,
    pub triggered_by: UserId,
    pub target_type: EmergencyTarget,
    pub reason: String,
    pub status: BrakeStatus,
    pub triggered_at: DateTime<Utc>,
    pub audit_deadline: DateTime<Utc>,  // 7 days from trigger
    pub audit_outcome: Option<AuditOutcome>,
}

#[derive(Debug, Clone)]
pub enum EmergencyTarget {
    Transition { proposal_id: Uuid },
    Witness { witness_id: Uuid },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrakeStatus {
    Active,          // Transition frozen, jury being selected
    UnderAudit,      // Jury reviewing
    Resolved,        // Audit complete
}

#[derive(Debug, Clone)]
pub struct AuditOutcome {
    pub justified: bool,
    pub jury_verdict: String,
    pub resolved_at: DateTime<Utc>,
}

/// Eligibility check: Keystone tier AND I >= 0.9 AND J >= 0.8
pub struct BrakeEligibility;

impl BrakeEligibility {
    pub fn can_trigger(rep: &UserReputation) -> bool {
        matches!(rep.tier, Tier::Keystone | Tier::Pillar | Tier::Genesis)
            && rep.integrity.value() >= Score::new(0.9)
            && rep.judgment.value() >= Score::new(0.8)
    }
}

/// Abuse penalty: 30% slash
pub const BRAKE_ABUSE_SLASH_PERCENT: Decimal = dec!(0.30);

/// J impact for audit outcomes
pub const BRAKE_JUSTIFIED_J_BONUS: Decimal = dec!(0.02);
pub const BRAKE_UNJUSTIFIED_J_PENALTY: Decimal = dec!(-0.05);
```

2. Add `EmergencyBrakeService` with methods:
   - `trigger_brake(user_id, target, reason) -> Result<EmergencyBrake>` — checks eligibility, freezes target, schedules jury
   - `resolve_brake(brake_id, outcome) -> Result<()>` — applies J bonus/penalty, optionally triggers slash

**Infrastructure:**

3. DB migration:
```sql
CREATE TABLE emergency_brakes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    triggered_by UUID NOT NULL REFERENCES users(id),
    target_type TEXT NOT NULL,
    target_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    audit_deadline TIMESTAMPTZ NOT NULL,
    audit_outcome JSONB,
    resolved_at TIMESTAMPTZ
);
```

**API:**

4. Endpoints:
```
POST /api/v1/emergency/brake         → trigger brake
GET  /api/v1/emergency/brakes/:id    → brake status
POST /api/v1/emergency/brakes/:id/resolve → resolve audit
```

---

## Gap 10: Novice On-Ramp Slot Reservation

**Codebase:** `markov-engine/`

### What exists

Tier system exists (`types/tier.rs`). Problem claiming exists. No reservation logic.

### What to build

A reservation system that holds 20% of participation slots for Novice-tier users for the first 48 hours.

**Domain crate changes:**

1. Add to `problem/` module (new file `novice_ramp.rs`):

```rust
use std::time::Duration;

pub struct NoviceReservationPolicy {
    /// Fraction of slots reserved for Novice-tier users
    pub reservation_fraction: Decimal,  // 0.20
    /// Duration of reservation window
    pub window_duration: Duration,      // 48 hours
}

impl Default for NoviceReservationPolicy {
    fn default() -> Self {
        Self {
            reservation_fraction: dec!(0.20),
            window_duration: Duration::from_secs(48 * 3600),
        }
    }
}

pub enum SlotEligibility {
    /// Open to all tiers
    Open,
    /// Still within reservation window — Novice-only
    NoviceReserved { opens_at: DateTime<Utc> },
    /// Reservation expired — open to all
    ReservationExpired,
}

impl NoviceReservationPolicy {
    /// Check if a specific slot is in the reserved pool
    pub fn check_eligibility(
        &self,
        slot_index: u32,
        total_slots: u32,
        created_at: DateTime<Utc>,
        now: DateTime<Utc>,
        user_tier: &Tier,
    ) -> SlotEligibility {
        let reserved_count = (Decimal::from(total_slots) * self.reservation_fraction)
            .ceil().to_u32().unwrap_or(0);

        if slot_index >= reserved_count {
            return SlotEligibility::Open;  // Not a reserved slot
        }

        let window_end = created_at + chrono::Duration::from_std(self.window_duration).unwrap();

        if now < window_end {
            if matches!(user_tier, Tier::Novice) {
                SlotEligibility::Open  // Novice can claim reserved slot
            } else {
                SlotEligibility::NoviceReserved { opens_at: window_end }
            }
        } else {
            SlotEligibility::ReservationExpired  // Window passed, open to all
        }
    }
}
```

2. Integrate into `ProblemService` or wherever task claiming happens — before allowing a claim, check `NoviceReservationPolicy::check_eligibility()`.

3. Apply to three slot types:
   - Witness corroboration opportunities (20% of co-witness slots)
   - Verification assignments (20% of verifier slots, with Keystone audit oversight)
   - Low-difficulty executor tasks (20% of claims on tasks where `difficulty < 0.5`)

**No DB migration needed** — this is policy logic applied at claim time. The reservation fraction and window can live in system config (`crates/domain/src/config.rs`).

**Tests:**

```gherkin
Feature: Novice On-Ramp Reservation
  Scenario: Reserved slot blocks non-Novice during window
    Given task T-001 created 12 hours ago with 10 execution slots
    And Novice reservation is 20% for 48 hours
    When Keystone user tries to claim slot 1 (reserved)
    Then claim is rejected with "NoviceReserved, opens_at: +36h"

  Scenario: Novice can claim reserved slot
    Given same setup
    When Novice user tries to claim slot 1
    Then claim succeeds

  Scenario: Reservation expires after 48h
    Given task T-001 created 50 hours ago
    When Keystone user tries to claim slot 1
    Then claim succeeds (reservation expired)
```

---

## Gap 11: Diversity Guard for Vote Quorum

**Codebase:** `markov-engine/`

### What exists

`crates/domain/src/markov/conductance.rs` has community conductance analysis — it can detect tight clusters (cartels) in the trust graph. This is used for anti-collusion dampening during PageRank.

`crates/domain/src/problem/multi_verifier.rs` has `MultiVerification` that counts votes. But it doesn't check whether votes are concentrated from a single cluster.

### What to build

A diversity check on vote quorums: no single vouch cluster can contribute more than 40% of the total quorum weight.

**Domain crate changes:**

1. Add to `governance/` module (new file `diversity_guard.rs`):

```rust
pub struct DiversityGuard {
    /// Maximum fraction of quorum weight from any single cluster
    pub max_cluster_weight_fraction: Decimal,  // 0.40
}

impl Default for DiversityGuard {
    fn default() -> Self {
        Self { max_cluster_weight_fraction: dec!(0.40) }
    }
}

pub struct ClusterVoteAnalysis {
    /// Cluster ID → total weighted vote from that cluster
    pub cluster_weights: HashMap<String, Decimal>,
    pub total_weight: Decimal,
}

impl DiversityGuard {
    /// Check if the current vote distribution passes diversity check
    pub fn check(&self, analysis: &ClusterVoteAnalysis) -> DiversityResult {
        for (cluster_id, weight) in &analysis.cluster_weights {
            let fraction = *weight / analysis.total_weight;
            if fraction > self.max_cluster_weight_fraction {
                return DiversityResult::Fail {
                    cluster_id: cluster_id.clone(),
                    fraction,
                    max_allowed: self.max_cluster_weight_fraction,
                };
            }
        }
        DiversityResult::Pass
    }
}

pub enum DiversityResult {
    Pass,
    Fail {
        cluster_id: String,
        fraction: Decimal,
        max_allowed: Decimal,
    },
}
```

2. Integrate with `MultiVerification` — after quorum + threshold are met, run `DiversityGuard::check()`. If it fails, the proposal stays in voting (doesn't approve) until weight distribution diversifies or the cluster dominance drops below 40%.

3. The cluster assignment per user comes from the existing conductance analysis in `markov/conductance.rs`. You'll need to expose a method like `get_user_cluster(user_id) -> Option<ClusterId>`.

**No DB migration** — cluster membership is computed during PageRank epochs and can be cached.

---

## Gap 12: Configurable Consensus Thresholds per Application

This is **fully covered by Gap 6** (the `ConsensusType::Custom { threshold }` variant). Once Gap 6 is implemented, any application (Explore, Celebrate, custom tracks) can specify arbitrary consensus thresholds.

No separate implementation needed. Gap 6 handles it.

---

## Gap 13: Emergency Fast-Track with Post-Hoc Audit

**Codebase:** `markov-engine/`

### What exists

Gap 9 adds the Emergency Brake. This gap adds the complementary mechanism: fast-tracking a transition in an emergency while mandating a post-hoc jury audit within 7 days.

### What to build

Extend the governance system to allow emergency transitions that skip the standard voting period but require mandatory audit.

**Domain crate changes:**

1. Add to `emergency/` module (or extend Gap 9):

```rust
#[derive(Debug, Clone)]
pub struct EmergencyFastTrack {
    pub id: Uuid,
    pub proposal_id: Uuid,      // The transition being fast-tracked
    pub endorsed_by: Vec<UserId>, // Emergency endorsers
    pub reason: String,
    pub status: FastTrackStatus,
    pub fast_tracked_at: DateTime<Utc>,
    pub audit_deadline: DateTime<Utc>,  // 7 days
    pub audit_outcome: Option<FastTrackAuditOutcome>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FastTrackStatus {
    Active,        // Transition committed, audit pending
    UnderAudit,    // Jury reviewing
    Validated,     // Audit passed — J bonus for endorsers
    Rejected,      // Audit failed — J penalty for endorsers
}

#[derive(Debug, Clone)]
pub struct FastTrackAuditOutcome {
    pub justified: bool,
    pub jury_verdict: String,
    pub resolved_at: DateTime<Utc>,
}

/// Same eligibility as brake: Keystone + I >= 0.9 + J >= 0.8
/// At least 2 eligible users must co-endorse the fast-track
pub const MIN_FAST_TRACK_ENDORSERS: usize = 2;

/// J impacts
pub const FAST_TRACK_JUSTIFIED_J_BONUS: Decimal = dec!(0.02);
pub const FAST_TRACK_UNJUSTIFIED_J_PENALTY: Decimal = dec!(-0.05);
```

2. Add `EmergencyFastTrackService`:
   - `create_fast_track(proposal_id, endorser_ids, reason) -> Result<EmergencyFastTrack>` — checks all endorsers meet eligibility, commits transition immediately, schedules 7-day audit
   - `complete_audit(fast_track_id, outcome) -> Result<()>` — applies J bonus/penalty to all endorsers

3. Add epoch job or scheduled task to check for fast-tracks past their audit deadline — auto-escalate to Stochastic Jury if no audit completed.

**Infrastructure:**

4. DB migration:
```sql
CREATE TABLE emergency_fast_tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    fast_tracked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    audit_deadline TIMESTAMPTZ NOT NULL,
    audit_outcome JSONB,
    resolved_at TIMESTAMPTZ
);

CREATE TABLE emergency_fast_track_endorsers (
    fast_track_id UUID NOT NULL REFERENCES emergency_fast_tracks(id),
    user_id UUID NOT NULL REFERENCES users(id),
    endorsed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (fast_track_id, user_id)
);
```

**API:**

5. Endpoints:
```
POST /api/v1/emergency/fast-track         → create fast-track
GET  /api/v1/emergency/fast-tracks/:id    → status
POST /api/v1/emergency/fast-tracks/:id/audit → complete audit
```

---

## General Notes for All Gaps

### Code Conventions

- **Crate structure:** Domain crate has zero I/O. Application crate orchestrates via traits. Infrastructure implements traits.
- **Error handling:** `DomainError` / `DomainResult<T>` in domain crate. `ApplicationError` / `ApplicationResult<T>` in application crate. `ApiError` in API crate.
- **IDs:** Use newtype wrappers from `types/ids.rs` — `UserId`, `VouchId`, `ProblemId`, etc. Add new ID types as needed.
- **Decimals:** Use `rust_decimal::Decimal` everywhere. Constants via `dec!()` macro.
- **Serialization:** `serde::Serialize + Deserialize` on all domain types. JSON for API DTOs.
- **Testing:** BDD feature files in `tests/bdd/features/`. Step definitions in `tests/bdd/steps/`. Unit tests inline in domain modules.
- **Migrations:** Named `YYYYMMDDNNNN_description.sql` in `migrations/` directory.
- **API:** Axum handlers with `utoipa` OpenAPI annotations. Auth via `AuthUser` extractor. State via `State<AppState>`.

### Implementation Order (Recommended)

1. **Gap 2** (Witness target type) — trivial, unblocks other gaps
2. **Gap 5** (J governance events) — small, needed by Gaps 3, 9, 13
3. **Gap 1** (VouchContext) — medium, foundational for clawback scoping
4. **Gap 6** (Configurable consensus) — small, unblocks Gaps 7 and track composition
5. **Gap 8** (Context Triad) — small, needed for evidence flow
6. **Gap 3** (Governance Budget) — medium, core economic primitive
7. **Gap 4** (Witness Complexity) — small, depends on Skill API
8. **Gap 11** (Diversity Guard) — small, depends on conductance module
9. **Gap 10** (Novice On-Ramp) — small, policy logic only
10. **Gap 9** (Emergency Brake) — medium, new module
11. **Gap 13** (Emergency Fast-Track) — medium, extends Gap 9
12. **Gap 7** (Celebrate composition) — trivial, app-layer only
13. **Gap 12** — already covered by Gap 6

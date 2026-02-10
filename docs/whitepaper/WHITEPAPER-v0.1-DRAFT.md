# Gotong Royong Whitepaper (v0.1 Draft)
Last updated: 2026-02-10

This is a living draft intended to make the concept crisp enough to build without drift.

## Abstract

Gotong Royong is a witness-first coordination platform. A user becomes a witness: they submit a claim about reality (good, bad, unknown). The platform turns witness claims into shared understanding, resolvable problems, and verified execution via Tandang. Reputation and credibility are measured throughout the journey using the Markov Credential Engine.

The central design goal is to reward high-integrity witnessing, high-signal sensemaking, reliable execution, and honest impact confirmation, while resisting spam, collusion, and identity-risk constraints.

## Mental Model

Like a seed that becomes fruit:

1. Seed: a witness signal exists.
2. Sprout: the community converges on meaning (definition / path).
3. Bloom: work happens (tasks executed, verified).
4. Fruit: the benefit is consumed (impact confirmed).

## Core Primitive: Witness

### Definition

A `Witness` is a canonical, evolving record that represents a claim about reality.

Examples:
- "There is flooding risk in neighborhood X."
- "This team delivered food to 50 families today."
- "Is this approach effective for mosquito control?"

### Two-Layer Object Model (for privacy)

To support confidential submissions:

- `WitnessSource` (private): raw text, attachments, metadata, identity.
- `WitnessPublished` (public): redacted summary + structured fields used by the shared "world".

The `WitnessPublished` record is what other users see and act on in confidential mode.

### Visibility / Disclosure Modes

Every witness chooses a disclosure mode:

1. Community-Open (pseudonymous open)
   - The community can read the witness content.
   - Source identity can be pseudonymous.

2. Confidential (AI-mediated / half-open)
   - Only the platform (and AI systems) can access raw witness content.
   - The system publishes a redacted `WitnessPublished` stub into the "world".
   - The community discusses/acts on the published stub and "data primer" fields.

3. Fully-Open (identity-open)
   - Witness content is public.
   - Source opts into stronger identity disclosure/verification (optional, policy-defined).

### Disclosure Mode vs Markov Identity Tier (Important)

Confidential mode is about *content disclosure*, not "hiding inside Markov".

Recommended policy alignment with the Markov Engine:
- A user may submit a Confidential witness while still being `Verified`/`Public` in Markov identity tier.
- Markov identity downgrades are disabled by default; exceptions (e.g., witness protection) require explicit system admin action.
- The `WitnessPublished` stub is what becomes public. Markov can still receive event signals using platform-scoped actor ids without publishing raw identity or raw content.

## Tracks: How a Witness Evolves

A witness can evolve across different tracks. Track changes are allowed, but only via governed transitions (proposal + vote), to prevent quiet goalpost moving.

### Track A: Resolve (Problem -> Work -> Solution -> Impact)

Use for solvable situations where a resolution path and success criteria can be defined.

### Track B: Celebrate (Achievement -> Recognition -> Impact)

Use for positive achievements where the outcome is mainly recognition, reputation, and optionally impact confirmation.

### Track C: Explore (Question -> Hypotheses -> Experiments -> Conclusion)

Use for uncertain claims and open questions. Explore can later become Resolve when a problem definition emerges.

## State Machines

### Common Transition Rules

All stage transitions:
- are proposed explicitly (append-only history),
- include a criteria checklist and required evidence types,
- are locked by a Markov-weighted vote (quorum + threshold + cooldown),
- are challengeable within a fixed challenge window.

### Proof of Context (Evidence Profiles)

The biggest failure mode for community coordination is decontextualization: a witness can be honest but incomplete. To reduce this, each track defines a default "evidence profile" that is enforced at key transitions.

We define a Context Triad:
- Visual: photo/video/document scan where appropriate
- Locational: coarse GPS / map node / place label where appropriate
- Corroborative: at least one co-witness with non-zero integrity

AI acts as the initial "Triad Auditor":
- It checks whether the proposed transition meets the evidence profile.
- If not, it blocks the transition proposal from entering voting and returns a friendly checklist ("missing Visual", "need 1 co-witness", etc.).
- It does not decide the outcome; it enforces minimum context standards and coaching.

Recommended default for the Resolve track:
- At `Seed -> Define`: require at least 1-of-3 (to avoid blocking early signals).
- At `Define -> Path`: require 2-of-3 (default minimum evidence standard).
- For high-risk domains (policy-defined): optionally tighten to 3-of-3 or add stronger requirements (e.g., multiple corroborators).

The evidence profile is part of the criteria checklist. AI may propose it, but it only becomes binding once the community locks the criteria via vote.

Evidence profiles are not one-size-fits-all:
- Defaults exist per track and transition, but a specific witness may require stronger or different proof depending on domain and risk.
- AI proposes the evidence profile for the next transition (and explains the rationale); humans can edit and then lock it via vote.

### State Machine: Resolve Track

Stages (recommended defaults):
- `Seed` (Keresahan / signal)
- `Define` (problem statement locked)
- `Path` (plan + success criteria locked)
- `Execute` (Tandang tasks active)
- `Accept` (criteria met; solution accepted)
- `Impact` (benefit confirmed after time delay)
- `Archive` (final state)

#### Execute Sub-States (Anti-Ghosting)

Execution is where coordination often fails due to silence and stalled work. `Execute` includes explicit sub-states:
- `Active`: tasks are progressing with regular updates
- `Stalled`: no progress heartbeat within the expected interval (policy-defined)
- `Released`: claim on a critical task is released back to the community

Recommended default:
- Executors must provide progress heartbeats (short updates or evidence) per task.
- If a critical-path task becomes `Stalled`, the task can be released and reassigned.
- Stalling emits a Markov signal that affects Judgment (J) and claim privileges (not Competence).

Markov semantic alignment:
- Markov Integrity (I) is "honest human-ness" and does not decay except fraud.
- Reliability (showing up, not blocking execution) is modeled as behavior that affects Judgment (J) and platform permissions (cooldowns/bans).

Additional policy hook:
- Repeated `Stalled -> Released` events can reduce a user's ability to claim high-difficulty/high-impact work (e.g., high-GDF tasks), without erasing competence.

### State Machine: Celebrate Track

Stages:
- `Seed` (witness of achievement)
- `Corroborate` (evidence + co-witnessing)
- `Recognize` (public recognition, badges, record)
- `Impact` (optional: downstream benefit confirmed)
- `Archive`

### State Machine: Explore Track

Stages:
- `Seed` (question / unknown)
- `Hypotheses` (candidate explanations/approaches)
- `Experiments` (tests, research tasks, observations)
- `Conclusion` (best current answer + confidence)
- `Archive`

## Governance: Proposals, Votes, and Disputes

### Proposal Types

- Stage transition proposal (e.g., `Define -> Path`)
- Track change proposal (e.g., `Explore -> Resolve`)
- Criteria update proposal (new criteria version)
- Dispute proposal (challenge an accepted transition)

### Voting (Markov-weighted)

Default mechanics:
- Eligibility gate: based on Markov integrity/judgment thresholds (policy-defined).
- Weighting: based on Markov reputation/tier and integrity/judgment multipliers.
- Requirements:
  - Minimum quorum (distinct voters)
  - Minimum weighted threshold
  - Cooldown between repeated transitions
  - Optional diversity guard (reduce single-clique fast-tracking)

### Budgeted Governance (Anti-Farming)

To prevent "point farming" and low-effort endorsements, votes are not free. Each user has a limited governance budget (attention budget) that is consumed when endorsing transitions.

Recommended default:
- Budget is derived from Markov integrity/judgment and tier (policy-defined).
- Endorsing a transition locks a portion of budget until the transition outcome is known.
- If the transition is upheld and the witness reaches higher-quality outcomes (e.g., `Accept` or `Impact`), budget is returned (optionally with a small bonus).
- If the transition is later overturned via dispute/jury, budget is withheld longer and the endorser may take a small Judgment (J) penalty (poor evaluation). Proven bad-faith/collusion can escalate to Integrity (I) slashing per policy.

This makes endorsement a high-signal action and discourages mass upvoting.

Reputation Yield (optional framing):
- Treat the bonus on successful outcomes as a "reputation yield" on attention.
- The system becomes a marketplace for truth where careful endorsements compound social capital, while lazy endorsements decay it.

### Disputes

Recommended defaults:
- Every accepted transition opens a `ChallengeWindow` (48-72h).
- Eligible users can open a dispute with a rationale and evidence.
- Disputes freeze the witness stage and route to an arbitration mechanism (jury).
- Bad-faith disputes and bad-faith transitions both carry penalties (Markov judgment/integrity impact).

## What Markov Measures (Signals Across the Journey)

The Markov Credential Engine should measure roles and quality at each stage:

### Roles
- Witness (source of the claim)
- Co-witness (corroborates)
- Sensemaker (defines, clarifies, sets criteria)
- Executor (completes Tandang tasks)
- Verifier (verifies PoR / outcomes)
- Beneficiary/Adopter (attests to impact)
- Challenger/Juror (dispute participation)

### Scoring Principles
- Reward accuracy over time: corroborated witnesses gain; contradicted witnesses lose.
- Reward clarity: criteria that reduce later disputes are valuable.
- Reward verified execution: PoR evidence and verification carry the strongest weight.
- Reward honest impact: "impact confirmed" should be time-delayed and evidence-backed.
- Avoid perverse incentives: do not over-reward raw posting or endless discussion.

### Integrity as a Multiplier for Privilege

Integrity (honesty / human-ness), judgment (quality of evaluation), and competence (domain skill) are tracked separately, but integrity and judgment gate and scale influence.

Recommended rule:
- Integrity (I) and Judgment (J) are multipliers for privileges and weights (voting, verification authority, dispute authority).
- Competence influences which domains a user can credibly shape (sensemaking, criteria, verification).

Illustrative (implementation-agnostic) weight:
- `effective_weight = base_weight * clamp(f(integrity, judgment), min, max)`
  - `base_weight` is derived from tier/reputation
  - `f(integrity, judgment)` is monotonic: low integrity/judgment strongly discounts influence

### Signal Mapping (Conceptual Events)

To make reputation computation buildable, the platform must emit granular signals to the Markov Credential Engine. These are conceptual event types; exact schemas are versioned.

| Journey Stage | Markov Signal | Intended Effect |
| --- | --- | --- |
| Seed | `witness_created(actor, domain_tags)` | Creates a pending credibility stake for the witness. |
| Corroborate | `witness_corroborated(co_witness, witness_id)` | Transfers small weight to the claim and co-witness role. |
| Define/Path | `criteria_locked(sensemaker, witness_id, criteria_hash, task_dag_hash)` | Rewards sensemaking/judgment when later validated by low dispute rate and successful outcomes. |
| Execute | `task_progress(executor, task_id)` | Rewards reliable execution behavior; does not mint competence unless paired with verified outcomes. |
| Execute | `task_released_for_stall(executor, task_id)` | Applies Judgment (J) penalties and claim cooldown/ban policies for repeated stalls; does not decay Integrity (I) unless fraud is involved. |
| Execute | `por_evidence_submitted(actor, task_or_solution_id, evidence_hash)` | Rewards verified work and domain competence where appropriate. |
| Accept | `solution_accepted(verifier_set, witness_id)` | Confirms that success criteria were met; distributes credit along roles. |
| Impact | `impact_attested(beneficiary_or_adopter, witness_id)` | Time-delayed confirmation; triggers larger distribution to early witnesses and sensemakers. |
| Dispute | `dispute_opened(challenger, witness_id)` / `dispute_resolved(outcome, jurors)` | Rewards good challenges; penalizes bad-faith disputes and rubber-stamping. |

### Markov Semantics (Gotong Mapping)

To avoid semantic drift, Gotong uses Markov terms consistently:
- Integrity (I): fraud and bad-faith (forged evidence, collusion, bribery) and supports slashing/clawback.
- Judgment (J): quality of evaluation (endorsement accuracy, dispute outcomes, rubber-stamping, abandonment/stalls) and supports cooldown/ban policies.
- Competence (C): domain skill earned from verified work and decays over time.
- Identity tier (ID_Mult): scales influence; Confidential content does not require identity downgrades.

## AI’s Role (Assistant, Not Judge)

AI is allowed to:
- summarize and redact confidential submissions into `WitnessPublished`,
- propose track classification and criteria checklists,
- suggest tasks, dependencies, and testable success criteria,
- detect duplicates, contradictions, and potential gaming patterns.

AI is not allowed to:
- finalize stage transitions unilaterally,
- override governed votes,
- reveal confidential identity or raw content.

## Data Primer (Published Stub Fields)

In Confidential mode, the platform publishes a stub with structured fields:

- `witness_id` (stable)
- `track` and `stage`
- `claim_summary` (redacted)
- `claim_type` (good / bad / unknown)
- `domain_tags` (skills/domains)
- `time_window` (coarse)
- `location_coarse` (optional, coarse)
- `evidence_types_present` (photo/gps/witness/other)
- `missing_info_questions` (what’s needed next)
- `next_transition_criteria_draft` (AI-proposed checklist)
- `risk_flags` (spam likelihood, contradiction likelihood) (optional)

## Integration With Tandang

Tandang is the execution layer for the Resolve track:
- `Path` produces a task graph (optional DAG).
- `Execute` tracks task completion, PoR evidence, and verification.
- `Accept` happens when criteria are met and verified.

## Integration With Markov (Contract-Level)

High-level event types (conceptual; exact schemas are versioned):
- `witness_created`
- `track_changed`
- `stage_transitioned`
- `task_created` / `task_completed`
- `por_evidence_submitted`
- `verification_recorded`
- `impact_attested`
- `dispute_opened` / `dispute_resolved`

The Markov Engine is responsible for:
- weighting votes and privileges (integrity/judgment/tier),
- updating reputation based on validated events,
- providing anti-abuse signals (as policy inputs).

## Recursive Accountability (Clawback / Recursive Slash)

When late-stage information shows a witness was fraudulent or grossly misleading, the system must be able to unwind credit and penalize weak verification.

Recommended default:
- Fraud discovered at late stages triggers a clawback of a portion of reputation/credit previously distributed from that witness.
- Penalties apply to:
  - the original witness (largest),
  - endorsers who spent governance budget to advance bad transitions (small, scaled),
  - verifiers who approved low-quality or contradictory evidence (small, scaled),
  - challengers who failed to challenge obvious fraud (optional, policy-defined).

This makes verification and endorsement meaningful, not rubber-stamping.

Clarification:
- Clawback is a surgical strike, not a total reset. The goal is to unwind reputation gained from the specific fraudulent chain (witness -> transitions -> verifications -> impact attestations), not to erase unrelated history.

## Emergency Override (Brake Clause)

In rare cases of systemic attack (bribery rings, coordinated fraud), the protocol may support an emergency "brake" action:
- A user above a very high integrity/judgment threshold (policy-defined) can trigger an immediate jury audit on a transition.
- The brake freezes progression temporarily and creates an audit trail.
- Abuse of the brake is penalized heavily (to prevent censorship-by-brake).

## Threat Model (Non-Exhaustive)

Even with Markov, the system must consider:
- Collusion (high-rep clique fast-tracking)
- Bribery (verifications/impact attestations)
- Griefing (blocking transitions via disputes)
- Spam witnesses (low-effort noise)
- Privacy harm (doxxing via details in witness content)

Mitigations used here:
- Weighted votes, quorum thresholds, diversity guard
- Challenge windows + jury arbitration
- Confidential mode with published stub
- Append-only history, criteria versioning
- Time-delayed impact confirmation

## Roadmap (Recommended)

v0.1 (Concept -> MVP)
- Witness primitive + tracks + stages
- Confidential mode with published stub
- Proposal/vote for track + stage transitions
- Resolve track integrated with Tandang tasks
- Markov-weighted voting and role scoring

v0.2 (Hardening)
- Disputes/jury flows
- Diversity guard tuning
- Stronger privacy controls + audit logs

v0.3 (Economics)
- Optional mutual credit layer bound to verified execution and confirmed impact

## Open Questions (to finalize before build freeze)

- What is the minimum eligibility policy for voting and disputes (integrity/judgment thresholds)?
- What are the default evidence profiles per track, and which domains require stronger profiles?
- What is the progress heartbeat policy (cadence, what counts as a heartbeat, when to declare `Stalled`)?
- What counts as "Impact confirmed" per domain (time window + required attestations)?
- What diversity guard is acceptable without over-complication?
- How to handle emergency cases (fast track with later audit)?
- Should the Emergency Brake Clause be enabled in v0.1, and what is the exact threshold/penalty policy?

## Appendix: Data Primer JSON Schema (Draft)

The Data Primer is the public stub (`WitnessPublished`) created from a witness submission, especially in Confidential mode. This schema is illustrative and versioned.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "WitnessPublished",
  "type": "object",
  "required": ["witness_id", "track", "stage", "claim_summary", "claim_type", "domain_tags", "evidence"],
  "properties": {
    "witness_id": { "type": "string", "minLength": 1 },
    "version": { "type": "integer", "minimum": 1 },
    "track": { "type": "string", "enum": ["resolve", "celebrate", "explore"] },
    "stage": { "type": "string", "minLength": 1 },
    "claim_type": { "type": "string", "enum": ["good", "bad", "unknown"] },
    "claim_summary": { "type": "string", "minLength": 1, "maxLength": 2000 },
    "domain_tags": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 },
      "maxItems": 32
    },
    "time_window": {
      "type": "object",
      "properties": {
        "start": { "type": "string", "format": "date-time" },
        "end": { "type": "string", "format": "date-time" },
        "coarse": { "type": "boolean" }
      },
      "additionalProperties": false
    },
    "location_coarse": {
      "type": "object",
      "properties": {
        "label": { "type": "string" },
        "region": { "type": "string" },
        "lat": { "type": "number", "minimum": -90, "maximum": 90 },
        "lon": { "type": "number", "minimum": -180, "maximum": 180 },
        "coarse": { "type": "boolean" }
      },
      "additionalProperties": false
    },
    "evidence": {
      "type": "object",
      "required": ["types_present"],
      "properties": {
        "types_present": {
          "type": "array",
          "items": { "type": "string", "enum": ["visual", "locational", "corroborative", "other"] }
        },
        "context_triad_minimum": {
          "type": "object",
          "properties": {
            "required_at_stage": { "type": "string" },
            "min_of_three": { "type": "integer", "minimum": 0, "maximum": 3 }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    },
    "missing_info_questions": {
      "type": "array",
      "items": { "type": "string", "maxLength": 300 },
      "maxItems": 16
    },
    "next_transition_criteria_draft": {
      "type": "array",
      "items": { "type": "string", "maxLength": 300 },
      "maxItems": 32
    },
    "risk_flags": {
      "type": "array",
      "items": { "type": "string" },
      "maxItems": 16
    },
    "source_commitment": {
      "type": "object",
      "properties": {
        "hash_alg": { "type": "string", "enum": ["sha256"] },
        "commitment": { "type": "string", "minLength": 32 }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

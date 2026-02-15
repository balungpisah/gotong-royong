# Gotong Royong Whitepaper (v0.1 Draft)
Last updated: 2026-02-11

This is a living draft intended to make the concept crisp enough to build without drift.

## Abstract

Gotong Royong is a witness-first coordination platform. A user becomes a witness: they submit a claim about reality (good, bad, unknown). The platform turns witness claims into shared understanding, resolvable problems, and verified execution via Tandang. Reputation and credibility are measured throughout the journey using the Markov Credential Engine.

The central design goal is to reward high-integrity witnessing, high-signal sensemaking, reliable execution, and honest impact confirmation, while resisting spam, collusion, and identity-risk constraints.

Gotong Royong composes on top of Tandang's full surface area: the PageRank trust graph, dual-layer scoring (Integrity, Competence, Judgment), ESCO domain taxonomy, vouch mechanics, slashing cascades, decay system, Global Difficulty Floor, genesis bootstrap, novice on-ramps, difficulty weighting, domain efficiency tracking, stochastic jury, sincere disagreement protocol, cross-pillar arbitration, and recovery paths. Every GR game-theoretic concept either IS a Tandang primitive or COMPOSES from Tandang primitives.

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

Disclosure mode and Markov identity tier are orthogonal. A Verified user (ID_Mult = 1.0) can submit a Confidential witness — content is hidden, but the system knows the identity and applies full trust weight. An Anonymous user (ID_Mult = 0.5) submitting a Confidential witness gets full content privacy but half the trust weight. The 30-day grace period (ID_Mult = 1.0 for all new users regardless of identity tier) applies regardless of disclosure mode.

Recommended policy alignment with the Markov Engine:
- A user may submit a Confidential witness while still being `Verified`/`Public` in Markov identity tier.
- Markov identity downgrades are disabled by default; exceptions (e.g., witness protection) require explicit system admin action.
- The `WitnessPublished` stub is what becomes public. Markov can still receive event signals using platform-scoped actor ids without publishing raw identity or raw content.

Identity tier multipliers (from Tandang):
- Anonymous (email only): 0.5x — minimal Sybil cost
- Pseudonymous (phone + 90-day activity): 0.75x — moderate Sybil cost
- Verified (government ID + biometric): 1.0x — high Sybil cost
- Public (real name + social proof): 1.2x — maximum Sybil cost

This creates an 8x power difference between the lowest and highest identity tiers, making bot networks economically infeasible without eliminating privacy as a valid choice.

## Tracks: How a Witness Evolves

A witness can evolve across different tracks. Track changes are allowed, but only via governed transitions (proposal + vote), to prevent quiet goalpost moving.

### Track A: Resolve (Problem -> Work -> Solution -> Impact)

Use for solvable situations where a resolution path and success criteria can be defined.

### Track B: Celebrate (Achievement -> Recognition -> Impact)

Use for positive achievements where the outcome is mainly recognition, reputation, and optionally impact confirmation. Composes from Tandang's Type D (social) contributions and Endorsement primitive.

### Track C: Explore (Question -> Hypotheses -> Experiments -> Conclusion)

Use for uncertain claims and open questions. Explore can later become Resolve when a problem definition emerges. Composes from Tandang's Type C (subjective/creative) contributions with peer consensus verification.

## State Machines

### Common Transition Rules

All stage transitions:
- are proposed explicitly (append-only history),
- include a criteria checklist and required evidence types,
- are locked by a Markov-weighted vote (quorum + threshold + cooldown),
- are challengeable within a fixed challenge window.

Vote weights for transition proposals are computed from the most recent epoch's reputation snapshot, not real-time. This prevents race conditions where a user's reputation changes mid-vote (see Epoch System).

### Proof of Context (Evidence Profiles)

The biggest failure mode for community coordination is decontextualization: a witness can be honest but incomplete. To reduce this, each track defines a default "evidence profile" that is enforced at key transitions.

We define a Context Triad (extending Tandang's Proof of Reality):
- Visual: photo/video/document scan where appropriate
- Locational: coarse GPS / map node / place label where appropriate
- Corroborative: at least one co-witness with non-zero integrity

AI acts as the initial "Triad Auditor":
- It checks whether the proposed transition meets the evidence profile.
- If not, it blocks the transition proposal from entering voting and returns a friendly checklist ("missing Visual", "need 1 co-witness", etc.).
- It does not decide the outcome; it enforces minimum context standards and coaching.

Recommended defaults for the Resolve track:
- At `Seed -> Define`: require at least 1-of-3 (to avoid blocking early signals).
- At `Define -> Path`: require 2-of-3 (default minimum evidence standard).
- At `Path -> Execute`: require 2-of-3 (plan must be grounded before execution begins).
- At `Execute -> Accept`: require 3-of-3 (all three — Visual, Locational, Corroborative — required for solution acceptance).
- For high-risk domains (health ESCO 3.2, legal ESCO 2.6, infrastructure ESCO 2.1): require 3-of-3 from Define → Path onward.

The evidence profile is part of the criteria checklist. AI may propose it, but it only becomes binding once the community locks the criteria via vote.

Evidence profiles are not one-size-fits-all:
- Defaults exist per track and transition, but a specific witness may require stronger or different proof depending on domain and risk.
- AI proposes the evidence profile for the next transition (and explains the rationale); humans can edit and then lock it via vote.

External platform evidence (from Tandang's platform integrations) can strengthen Context Triad:
- A witness about open-source project quality → link to GitHub commits/PRs (Tandang's bot-github verifies cryptographic webhook proof)
- A witness about community discussion → link to Discord threads (Tandang's bot-discord verifies)
- A witness about public discourse → link to Twitter/X posts (Tandang's bot-twitter verifies)

### State Machine: Resolve Track

Stages (recommended defaults):
- `Seed` (Keresahan / signal)
- `Define` (problem statement locked)
- `Path` (plan + success criteria locked)
- `Execute` (Tandang tasks active)
- `Accept` (criteria met; solution accepted)
- `Impact` (benefit confirmed after time delay — minimum 30 days post-Accept, requires 2 independent attestations from beneficiaries/adopters with I > 0; domain-specific windows: disaster relief 30 days, infrastructure 90 days, education 180 days, health 90 days)
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

Abandoned task penalties (from Tandang):
- 1st abandonment: Warning; -0.01 J
- 2nd abandonment: 24h claim cooldown; -0.02 J
- 3rd+ abandonment: 7-day ban from claiming; -0.05 J

Markov semantic alignment:
- Markov Integrity (I) is "honest human-ness" and does not decay except fraud.
- Reliability (showing up, not blocking execution) is modeled as behavior that affects Judgment (J) and platform permissions (cooldowns/bans).

Additional policy hook:
- Repeated `Stalled -> Released` events can reduce a user's ability to claim high-difficulty/high-impact work (e.g., high-GDF tasks), without erasing competence.

#### Accept Stage: Verifier Disagreement

When two qualified verifiers disagree on whether success criteria are met at Accept, Tandang's Sincere Disagreement Protocol applies:

- If both are Keystone/Pillar with similar domain competence (`|C_A - C_B| < 10`): Uncertainty Flag triggers.
- Status moves to "Under Peer Review".
- Third independent verifier with domain competence breaks the tie.
- If still unresolved: Stochastic Jury (5-7 users, percentile ≥85, domain competence ≥0.5).
- Jury outcomes: subjective disagreement = no penalty; objectively wrong = J penalty (-0.03); bad faith = 30% slash trigger.

When the witness spans multiple domains (e.g., flooding witness with engineering solution affecting cultural sites), Tandang's Cross-Pillar Arbitration applies: 2 jurors from each disputed pillar + 1 from Logic & Audit. Verdict requires at least one vote from each pillar represented. No pillar unilaterally overrides another's domain-specific assessment (Pillar Sovereignty Principle).

#### Accept Stage: Auto-Verify Timeout

If no verifier claims a verification within 14 days:
- Executor receives 75% reputation credit immediately.
- 25% held in escrow.
- 7-day extended dispute window.
- If disputed and overturned: full provisional credit clawed back.

This prevents verifier bottlenecks from stalling execution indefinitely.

### State Machine: Celebrate Track

Stages:
- `Seed` (witness of achievement)
- `Corroborate` (evidence + co-witnessing)
- `Recognize` (public recognition, badges, record)
- `Impact` (optional: downstream benefit confirmed)
- `Archive`

#### Celebrate: Tandang Wiring

Celebrate is a recognition-only flow — no task execution, no binary success criteria. It composes from Tandang's Type D (social) contributions and Endorsement primitive.

Contribution type mapping:

| Celebrate Role | Tandang Type | Decay | Notes |
| --- | --- | --- | --- |
| Achievement witness | D (Social) | 120 days | Reporting achievements is social capital, not domain competence |
| Co-witness (corroborator) | D (Social) | 120 days | Attesting you saw it happen |
| Recognition endorser | J-only | N/A | Governance action — endorsement accuracy tracked |
| Impact attestor | D (Social) | 120 days | Downstream benefit confirmation |

Vouch generation:
- Co-witnessing at Corroborate → implicit vouch toward achievement witness (same as Resolve)
- Recognition vote → NO vouch (governance action, not trust transfer)
- Impact attestation → weak vouch toward witness chain (same as Resolve)

Evidence profile defaults:
- Seed → Corroborate: 1-of-3 Context Triad (low bar — achievements are often self-evident)
- Corroborate → Recognize: 2-of-3 Context Triad (recognition should be grounded)

ESCO tagging: AI tags the achievement at Seed with ESCO skills. Example: "This team delivered food to 50 families" → ESCO 5.0 (Services), ESCO-ID ID.003 (Social Cohesion). Achievement recognition earns Type D social reputation in tagged domains.

Celebrate-specific signals emitted to Tandang:
- `witness_created(actor, esco_skills[], claim_type=good)`
- `witness_corroborated(co_witness, witness_id)`
- `transition_endorsed(endorser, witness_id, budget_locked)` (at Recognize)
- `impact_attested(beneficiary, witness_id)` (at Impact)

Celebrate does NOT generate Type A or Type C contributions. This is deliberate: recognition is social capital, not demonstrated competence. A user who only reports achievements builds social reputation, not domain expertise.

### State Machine: Explore Track

Stages:
- `Seed` (question / unknown)
- `Hypotheses` (candidate explanations/approaches)
- `Experiments` (tests, research tasks, observations)
- `Conclusion` (best current answer + confidence)
- `Archive`

#### Explore: Tandang Wiring

Explore handles uncertainty — questions where the answer isn't known. It composes from Tandang's Type C (subjective/creative) for analytical work and Type A (defined problem) when experiments produce concrete tasks.

Contribution type mapping:

| Explore Role | Tandang Type | Decay | Notes |
| --- | --- | --- | --- |
| Question witness | C (Subjective) | 180 days | Good questions are valuable sensemaking |
| Hypothesis proposer | C (Subjective) | 180 days | Creative/analytical contribution |
| Experimenter (task-based) | A (Defined Problem) | 365 days | When experiments are concrete tasks with binary outcomes |
| Experimenter (observational) | C (Subjective) | 180 days | When experiments are observations requiring peer consensus |
| Conclusion author | C (Subjective) | 180 days | Synthesis quality evaluated by peer consensus |
| Co-witness (question corroborator) | D (Social) | 120 days | "I also observe this question matters" |

Vouch generation:
- Co-witnessing the question → implicit vouch toward question witness (same as other tracks)
- Hypothesis endorsement → J-only governance action (no vouch)
- Experiment verification → implicit vouch from verifier to experimenter (same as Resolve's Accept)
- Conclusion acceptance → deferred vouch for hypothesis proposers whose hypotheses were validated by the conclusion

Evidence profile defaults:
- Seed → Hypotheses: 0-of-3 Context Triad (questions don't require evidence — they ARE the evidence gap)
- Hypotheses → Experiments: 1-of-3 (minimum grounding for experiment design)
- Experiments → Conclusion: 2-of-3 (conclusions must be grounded in evidence)

ESCO tagging: AI tags at Seed. Example: "Is this approach effective for mosquito control?" → ESCO 3.2 (Health), ESCO 2.1 (Engineering). Explore can span multiple ESCO domains naturally.

Verification specifics:
- Hypotheses phase: Type C peer consensus with standard 60% threshold among qualified reviewers (C_eff ≥ 0.3 in domain)
- Experiments phase: if experiments produce Tandang tasks, they follow standard Type A verification (binary, PoR). If experiments are observational, Type C peer consensus applies.
- Conclusion phase: Type C peer consensus with elevated 70% threshold (conclusions require stronger agreement than hypotheses)

Track transition to Resolve: when Explore reveals a solvable problem, a governed track change proposal transitions to Resolve. All reputation earned in Explore carries over — the question witness, hypothesis proposers, and experimenters keep their reputation. The Explore archive links to the new Resolve witness for provenance.

## Governance: Proposals, Votes, and Disputes

### Proposal Types

- Stage transition proposal (e.g., `Define -> Path`)
- Track change proposal (e.g., `Explore -> Resolve`)
- Criteria update proposal (new criteria version)
- Dispute proposal (challenge an accepted transition)

### Voting (Markov-weighted)

Default mechanics:
- Eligibility gate: based on Markov integrity/judgment thresholds (policy-defined).
- Weighting: derived from Tandang's mechanical influence formulas (continuous, not tier-gated):
  - Endorsement weight: `1.0 + (percentile / 50.0)`, clamped [1.0, 3.0]
  - Multiplied by: `J_Mult × I_Mult × ID_Mult` where `J_Mult = 0.5 + 0.5 × J`, `I_Mult = 0.5 + 0.5 × I`, `ID_Mult = identity tier multiplier (0.5-1.2)`
  - Compound effect: a fraudster (J=0, I=0, Anonymous) has 12.5% of a trusted expert's (J=1.0, I=1.0, Verified) weight
- Requirements:
  - Minimum quorum (distinct voters)
  - Minimum weighted threshold
  - Cooldown between repeated transitions
  - Optional diversity guard (reduce single-clique fast-tracking)

### Budgeted Governance (Anti-Farming)

To prevent "point farming" and low-effort endorsements, votes are not free. Each user has a limited governance budget (attention budget) that is consumed when endorsing transitions.

The governance budget shares its conceptual base with Tandang's vouch budget. Both represent a user's total influence capacity:
- Vouch budget: `R × budget_factor` (default: 0.3), split across active vouches
- Governance budget: derived from the same base, consumed when endorsing transitions

A user's total influence — whether spent vouching for people or endorsing transitions — draws from a unified pool. This prevents double-spending influence and forces prioritization: spending budget to endorse a transition is a real cost that reduces other influence.

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

Recommended defaults (aligned with Tandang's dispute mechanics):
- Every accepted transition opens a `ChallengeWindow` (48-72h).
- Eligible users can open a dispute with a rationale and evidence.
- Disputes freeze the witness stage and route to Tandang's Stochastic Jury.
- Dispute stake: 3% of disputer's R (min 0.05). Frivolous disputes = stake burned + J penalty (-0.03). Upheld disputes = stake returned + J reward (+0.02).
- Suspension quorum: 2 independent disputes (Keystone/Pillar counts as 2).
- Rate limits: max 3 open disputes per user simultaneously; 14-day cooldown between disputes by same disputer on same witness.
- Min percentile gate: 30 (prevents Novice-spam disputes).
- Bad-faith disputes and bad-faith transitions both carry penalties (Markov judgment/integrity impact).

## Game Theory: Tandang Integration

This section defines how Gotong Royong's game theory composes from Tandang's primitives. Every mechanism described here either uses an existing Tandang primitive directly or composes from multiple primitives.

### ESCO Domain Anchoring

All witness domains are anchored to ESCO (European Skills, Competences, Qualifications & Occupations) — Tandang's structured skill taxonomy with 13,890 skills and 3,008 occupations. This replaces generic "domain tags" with machine-readable, interoperable skill references.

How ESCO flows through the witness lifecycle:
- At `Seed`: AI extracts ESCO skills from witness text via Tandang Skill API (`POST /extract-skills`). Example: a flooding witness gets tagged with ESCO 2.1.4 (Civil Engineering), ESCO 3.2.1 (Emergency Response), ESCO-ID ID.003 (Social Cohesion / Community Mediation).
- At `Define`: Sensemakers can refine ESCO tags (add/remove skills, subject to vote). Deliberate ESCO mis-tagging for reputation farming triggers 80% recursive slash for the tagger + endorsers of that transition.
- At `Execute`: Executors claiming a task must have `C_eff ≥ 0.3` in the task's ESCO domain (low bar — allows learning by doing; Novice on-ramps reserve 20% of low-difficulty tasks). Tandang enforces this.
- At `Accept`: Verifiers must have `C_eff ≥ 0.5` in the domain (Tandang's standard verifier eligibility).

#### Competence Transfer (τ)

Tandang's transferability coefficients determine whether competence in one domain counts toward another. This prevents "Halo Effect" corruption (being good at engineering doesn't make you credible in social work) while allowing legitimate skill adjacency:

- Same occupation, different skill: τ = 80% (e.g., Python → JavaScript)
- Same group, different occupation: τ = 40% (e.g., Software Dev → Database Admin)
- Related groups: τ = 15% (e.g., ICT → Engineering)
- Unrelated groups: τ = 0% (e.g., ICT → Social Work)
- Transversal skills (critical thinking, communication): τ = 100%
- ESCO → ESCO-ID: τ = 10% (general reputation → cultural context boost)
- ESCO-ID internal: τ = 50% (e.g., Javanese → Sundanese cultural competence)

Effective competence formula: `C_eff(domain) = min(1.0, Σ(C_skill × τ(skill → domain)))`

Applications in GR: verifier eligibility, jury selection, sensemaker credibility in domain-specific criteria-setting, executor task claiming, cross-pillar arbitration weighting.

#### The Four Pillars

Tandang organizes ESCO into four pillars that map naturally to GR roles:

| Pillar | ESCO Anchor | GR Role Affinity |
| --- | --- | --- |
| Tech & Build | Group 2.5 (ICT) + 2.1 (Engineering) | Executors of technical tasks |
| Field & Social | Group 2.6 (Legal) + 3.2 (Health) + 5.0 (Services) | Witnesses, co-witnesses, beneficiaries (on the ground) |
| Logic & Audit | T1.1 (Critical Thinking) + T1.2 (Analytical) | Sensemakers, verifiers, jurors |
| Culture & Context | ESCO-ID extension | Cultural mediators, local wisdom keepers |

#### ESCO-ID: The Gotong Royong Soul

ESCO-ID is Tandang's Indonesian extension of ESCO. It captures cultural competencies that global taxonomies miss:

- ID.001 — Bahasa & Communication (formal Indonesian, regional languages, cultural norms)
- ID.002 — Local Wisdom / Kearifan Lokal (traditional knowledge, adat, religious context)
- ID.003 — Social Cohesion (gotong royong practices, community mediation, inter-group relations)
- ID.004 — Indonesian Craft & Heritage (batik, traditional music, culinary, architecture)

ID.003 (Social Cohesion → Gotong Royong Practices) is literally the platform's namesake. Users who demonstrate competence in gotong royong practices through the platform earn ESCO-ID credentials — the cultural recognition layer that makes this system distinctly Indonesian, not a generic Western coordination tool.

#### Polymath Bonus

Users achieving Pillar tier (top 10%) in 3+ unrelated ESCO occupation groups earn a 5% Integrity bonus ("Cross-Domain Logic"). This rewards the rare person who bridges domains — a community organizer who also codes and understands cultural mediation. Exactly the kind of person a coordination platform most needs.

### Vouch Generation from GR Activities

Tandang's reputation is the stationary distribution of a PageRank algorithm over a trust graph. The trust graph is built from vouches — directed trust relationships between users. Without vouches, there is no trust graph, no PageRank, no reputation.

GR activities must generate vouches. Not every interaction is a vouch — vouches are high-signal trust transfers with skin in the game. The following GR events generate implicit vouches:

#### Co-witnessing → Vouch

When a user co-witnesses (corroborates a witness claim), they create an implicit vouch toward the original witness in the witness's ESCO domain. Rationale: co-witnessing stakes your credibility that this person saw something real. The vouch:
- enters Tandang's 14-day bleed-in phase (25% → 50% → 100% trust transfer)
- can be burned before full power transfers if the witness is flagged as fraudulent during latency
- is domain-scoped (co-witnessing a flooding claim creates a vouch in Emergency Response, not in all domains)

#### Verified Execution → Vouch

When a verifier accepts a solution at the Accept stage, the system creates an implicit vouch from verifier to executor in the task's ESCO domain. This is the strongest GR-generated vouch because:
- it's backed by PoR evidence (not just social attestation)
- the verifier is liable at 1.5x under Tandang's verifier liability multiplier
- the verifier's own reputation is at risk if the execution is later found fraudulent

#### Impact Attestation → Weak Vouch

When a beneficiary/adopter attests to impact, the system creates a weak vouch toward the witness chain (original witness + sensemakers). Weak because impact attestation is subjective, but it compounds over time if multiple beneficiaries attest independently.

#### Sensemaking Validated → Deferred Vouch

When a witness reaches Accept or Impact with low dispute rate, and the sensemaker's locked criteria were used successfully, the system generates implicit vouches from successful executors/verifiers back toward the sensemaker. This rewards good criteria-setting retroactively — you only get credit for sensemaking when it actually led to good outcomes.

#### Skeptical Vouches for Suspect Witnesses

When a user suspects a witness is false but doesn't have enough evidence for a formal dispute, they can place a skeptical vouch. This uses Tandang's skeptical vouch mechanics directly:
- Weight multiplier: 0.75 (reduced trust transfer)
- Dampening: affects other positive vouches toward the suspect (max 30% reduction, floor = 0.70)
- Stake: `max(0.02 × R_skeptic, min_stake)` — real skin in the game
- Auto-expiry: 90 days (forces re-evaluation, prevents permanent stigma)
- Outcome: if target slashed → stake returned + J reward (+0.05); if target performs well → stake burned + J penalty (-0.03)

Skeptical vouches do NOT affect Integrity (they signal reservation, not accusation). Mass skeptical vouching is self-limiting: each vouch costs J on failure, and collusion detection catches skeptical-vouch-then-slash patterns.

### Contribution Type Mapping

Tandang computes reputation using four contribution types, each with distinct decay rates, verification methods, and scoring formulas. Every GR role maps to a specific contribution type:

| GR Role | Tandang Type | Decay Half-Life | Verification | Scoring Formula |
| --- | --- | --- | --- | --- |
| Witness | C (Subjective) | 180 days | Peer consensus via corroboration + later outcome validation | `Base × (Approvals / Total_Reviews) × Avg_Quality × Domain_Weight` |
| Co-witness | D (Social) | 120 days | Corroboration accepted by community | `Base × (Quality / 10) × Social_Impact_Mult × Consistency_Bonus` |
| Sensemaker | C (Subjective) | 180 days | Retroactive: low dispute rate + successful outcomes from their criteria | `Base × (Approvals / Total_Reviews) × Avg_Quality × Domain_Weight` |
| Executor | A (Defined Problem) | 365 days | Binary: task completed + PoR verified | `Base × Quality × Difficulty × (Peer_Endorsements / Total_Reviews)` |
| Verifier | A (Defined Problem) | 365 days | Binary: verification accuracy (not overturned) | `Base × Quality × Difficulty × (Peer_Endorsements / Total_Reviews)` |
| Beneficiary | D (Social) | 120 days | Impact attestation accepted | `Base × (Quality / 10) × Social_Impact_Mult × Consistency_Bonus` |
| Endorser | J-only | N/A | Transition outcome (upheld vs overturned) | Affects Judgment, not Competence |
| Challenger | J-only | N/A | Dispute outcome | Affects Judgment, not Competence |
| Juror | J-only | N/A | Jury verdict accuracy | Affects Judgment, not Competence |

Key design notes:
- Type C (Witness, Sensemaker) uses competence-weighted reviewer consensus: `Weighted_Quality = Σ(Q_i × C_eff_i) / Σ(C_eff_i)` — reviewers with higher domain competence have more weight in evaluating quality.
- Type D (Co-witness, Beneficiary) gets 1.5x Social Impact Multiplier when the attester has an active vouch toward the target. This incentivizes mentorship and sustained community support.
- Type A (Executor, Verifier) is the strongest reputation generator because it's binary and evidence-backed. This correctly makes "doing and verifying real work" the primary path to reputation.
- J-only roles (Endorser, Challenger, Juror) affect Judgment score but not Competence. This means governance participation builds trust quality, not domain expertise — the correct semantic.

#### Judgment (J) Impact Table for GR Governance

| GR Event | J Impact |
| --- | --- |
| Endorse transition that is upheld + witness reaches Accept/Impact | +0.02 |
| Endorse transition that is later overturned by dispute | -0.05 |
| Endorse transition involved in fraud/clawback | -0.10 |
| Open dispute that is upheld (good challenge) | +0.02 |
| Open dispute that is frivolous | -0.03 |
| Jury service, voted with majority | +0.02 |
| Jury service, voted against majority (subjective) | 0 |
| Jury service, voted against majority (objective error) | -0.03 |
| Co-witness for witness that is later validated | +0.02 |
| Co-witness for witness that is later slashed | -0.10 |
| Vouch for user who performs well | +0.02 |
| Vouch for user who is slashed | -0.10 |
| Vouch for user involved in fraud | -0.20 |

Critical threshold: J < 0.3 triggers Shadow Tier (30-day loss of verify/vouch/endorse ability).

### Platform-Contribution Compatibility

GR activities map to Tandang's platform-contribution compatibility matrix. Contributions from task-based activities (Executor, Verifier) can generate Type A reputation. Contributions from discussion-based activities (Witness, Co-witness, Sensemaker) generate Type C or D. The system validates compatibility — an executor cannot claim Type D social reputation for completing a technical task.

## What Markov Measures (Signals Across the Journey)

The Markov Credential Engine measures roles and quality at each stage:

### Roles
- Witness (source of the claim)
- Co-witness (corroborates)
- Sensemaker (defines, clarifies, sets criteria)
- Executor (completes Tandang tasks)
- Verifier (verifies PoR / outcomes)
- Beneficiary/Adopter (attests to impact)
- Challenger/Juror (dispute participation)
- Endorser (governance voter who locks budget to advance transitions)

### Scoring Principles
- Reward accuracy over time: corroborated witnesses gain; contradicted witnesses lose.
- Reward clarity: criteria that reduce later disputes are valuable.
- Reward verified execution: PoR evidence and verification carry the strongest weight.
- Reward honest impact: "impact confirmed" should be time-delayed and evidence-backed.
- Avoid perverse incentives: do not over-reward raw posting or endless discussion.
- Decay ensures freshness: reputation earned from past activity fades, forcing continued contribution.
- Collective accountability: system-wide fraud consequences (GDF) make community health everyone's business.

### Integrity as a Multiplier for Privilege

Integrity (honesty / human-ness), judgment (quality of evaluation), and competence (domain skill) are tracked separately, but integrity and judgment gate and scale influence.

Recommended rule:
- Integrity (I) and Judgment (J) are multipliers for privileges and weights (voting, verification authority, dispute authority).
- Competence influences which domains a user can credibly shape (sensemaking, criteria, verification).

Concrete weight formula (from Tandang):
- `effective_weight = base_weight × J_Mult × I_Mult × ID_Mult`
  - `base_weight = 1.0 + (percentile / 50.0)`, clamped [1.0, 3.0]
  - `J_Mult = 0.5 + 0.5 × J` (range: 0.5-1.0)
  - `I_Mult = 0.5 + 0.5 × I` (range: 0.5-1.0)
  - `ID_Mult` = identity tier multiplier (0.5-1.2)

Compound effect examples:
- Fraudster (J=0, I=0, Anonymous): 0.5 × 0.5 × 0.5 × 1.0 = **0.125** (12.5% of base)
- New user (J=0.5, I=0.5, Pseudonymous): 0.75 × 0.75 × 0.75 × 1.0 = **0.422** (42.2%)
- Good actor (J=1.0, I=1.0, Verified): 1.0 × 1.0 × 1.0 × 3.0 = **3.0** (300%)
- Trusted expert (J=1.0, I=1.0, Public, Keystone): 1.0 × 1.0 × 1.2 × 3.0 = **3.6** (360%)

### Signal Mapping (Events Emitted to Tandang)

To make reputation computation buildable, the platform must emit granular signals to the Markov Credential Engine. These are event types that map to Tandang's event processor; exact schemas are versioned.

| Journey Stage | Markov Signal | Tandang Mechanism | Intended Effect |
| --- | --- | --- | --- |
| Seed | `witness_created(actor, esco_skills[])` | Type C contribution created | Creates a pending credibility stake for the witness. |
| Seed | `esco_tags_assigned(witness_id, esco_skills[])` | Skill API tagging | Anchors witness to ESCO domains for competence routing. |
| Corroborate | `witness_corroborated(co_witness, witness_id)` | Implicit vouch created (14-day bleed-in) + Type D contribution | Transfers small trust weight; co-witness stakes credibility. |
| Define/Path | `criteria_locked(sensemaker, witness_id, criteria_hash, task_dag_hash)` | Type C contribution (deferred scoring) | Rewards sensemaking/judgment retroactively when validated by low dispute rate and successful outcomes. |
| Execute | `task_claimed(executor, task_id, esco_skill)` | Problem lifecycle: CLAIMED | Executor must meet C_eff threshold in task's ESCO domain. |
| Execute | `task_progress(executor, task_id)` | Heartbeat tracking | Rewards reliable execution behavior; does not mint competence unless paired with verified outcomes. |
| Execute | `task_released_for_stall(executor, task_id)` | Abandoned problem penalty | Applies J penalties and claim cooldown/ban policies; does not decay I unless fraud involved. |
| Execute | `por_evidence_submitted(actor, task_id, evidence_hash)` | PoR evidence recorded | Pairs with verification to mint domain competence. |
| Accept | `solution_accepted(verifier_set, witness_id)` | Implicit vouch (verifier→executor) + Type A scoring | Distributes credit along roles; verifier takes 1.5x liability. |
| Accept | `verifier_disagreement(verifier_a, verifier_b, witness_id)` | Sincere Disagreement Protocol | Triggers Uncertainty Flag or Stochastic Jury. |
| Impact | `impact_attested(beneficiary, witness_id)` | Weak vouch + Type D contribution | Time-delayed; triggers larger distribution to early witnesses and sensemakers. |
| Dispute | `dispute_opened(challenger, witness_id, stake)` | Dispute lifecycle + stake bonded | Stake locked; dispute routed to jury. |
| Dispute | `dispute_resolved(outcome, jurors[])` | Jury verdict + J updates | Rewards good challenges; penalizes bad-faith disputes and rubber-stamping. |
| Governance | `transition_endorsed(endorser, witness_id, budget_locked)` | Governance budget consumed | Endorsement accuracy tracked for J scoring. |
| Governance | `transition_outcome(witness_id, upheld/overturned)` | J update for all endorsers | Budget returned/withheld; J penalties/rewards applied. |
| Slash | `fraud_detected(actor, witness_id, severity)` | Slash cascade triggered | Recursive penalties through vouch graph; GDF increase. |
| Slash | `clawback_initiated(witness_id, chain)` | Clawback along witness chain | Unwinds reputation from specific fraudulent chain. |
| Skeptical | `skeptical_vouch_placed(skeptic, target, stake)` | Skeptical vouch (0.75 weight, dampening) | Trust-but-verify signal; stake bonded. |

### Markov Semantics (Gotong Mapping)

To avoid semantic drift, Gotong uses Markov terms consistently:
- Integrity (I): fraud and bad-faith (forged evidence, collusion, bribery) and supports slashing/clawback. Does not decay except on fraud.
- Judgment (J): quality of evaluation (endorsement accuracy, dispute outcomes, rubber-stamping, abandonment/stalls, co-witness accuracy, vouch accuracy) and supports cooldown/ban policies.
- Competence (C): domain skill earned from verified work, anchored to ESCO skills, decays over time per contribution type.
- Identity tier (ID_Mult): scales influence (0.5x Anonymous → 1.2x Public); Confidential content does not require identity downgrades.
- Tier (Keystone/Pillar/Contributor/Novice/Shadow): percentile-based classification that gates mechanical influence (jury eligibility, vouch capacity, endorsement weight).

## Economic Mechanisms

### Decay: Reputation is Perishable

Tandang's decay system ensures reputation reflects current capability, not past achievement.

Competence decay formula: `C_new = C_old × (0.5)^(days / half_life)`

Decay rates by GR contribution type:

| GR Role | Tandang Type | Decay Half-Life | Inactivity Trigger | Rationale |
| --- | --- | --- | --- | --- |
| Executor | A (Defined Problem) | 365 days | 90 days | Deep skills persist longest |
| Verifier | A (Defined Problem) | 365 days | 90 days | Verification skill is durable |
| Witness | C (Subjective) | 180 days | 60 days | Observational credibility has natural cycles |
| Sensemaker | C (Subjective) | 180 days | 60 days | Criteria-setting quality needs freshness |
| Co-witness | D (Social) | 120 days | 14 days | Social attestation is most perishable |
| Beneficiary | D (Social) | 120 days | 14 days | Impact attestation is most perishable |

When a user has multiple contribution types, inactivity trigger uses the longest applicable (prevents penalizing a deep executor who also co-witnesses occasionally).

Integrity does NOT decay (unless fraud). A user who was honest but went inactive is still recognized as honest.

This system prevents "legacy sensemakers" who set criteria once and coast on that reputation forever, "one-time witnesses" who observed something important years ago but haven't contributed since, and "retired executors" whose technical skills have atrophied.

### Global Difficulty Floor (GDF) & Weather System

When fraud is detected and a user is slashed, 10% of the slashed reputation burns into the Global Difficulty Floor. The GDF affects everyone's earning rate:

`R_earned = R_base × (1 / (1 + GDF_normalized))`

Where `GDF_normalized = Global_Difficulty_Floor / Total_System_Reputation`.

GDF parameters:
- Burn rate: 10% of slashed reputation
- GDF half-life: 14 days (temporary, self-healing)
- Max GDF impact: 20% earning reduction cap

This creates collective accountability: "If my neighbor commits fraud and I didn't report it, my earning rate drops. I have skin in game for community health." Without GDF, fraud is someone else's problem. With GDF, every fraud event has tangible impact on everyone.

Sabotage protection: an attacker loses 100% of their reputation while only 10% burns to GDF — always net negative for the attacker.

#### Weather Report

GDF is displayed as a public "Weather Report" on the platform. Abstract percentages become visceral, shared experience:

| GDF Range | Weather | Message |
| --- | --- | --- |
| 0-5% | Cerah (Clear) ☀️ | "Clear skies. System healthy. Keep building." |
| 5-10% | Berawan (Cloudy) 🌤️ | "Light clouds forming. Stay vigilant." |
| 10-15% | Hujan (Rain) 🌧️ | "Rain incoming. Community audit recommended." |
| 15-20% | Badai (Storm) ⛈️ | "Storm warning. Active fraud. All hands on deck." |

Every witness page shows the current weather. Push notifications on degradation. Weekly "Weather Forecast" summarizing trends. The weather metaphor works because it's universal, removes blame, focuses on collective state, and fits Indonesian cultural context.

#### Weather Heroes

Users who actively contribute to lowering GDF receive recognition:

| Action | Hero Points |
| --- | --- |
| Report fraud leading to confirmed slash | +50 |
| Successful verification preventing fraud | +20 |
| Shadow Tier audit (as assigned Keystone) | +10 per cycle |
| Voluntary GDF Recovery Pool contribution | +5 per unit |

Hero tiers: Penjaga Cuaca (Watcher, 100pts) → Pembersih Langit (Sky Cleaner, 500pts, +5% J boost) → Pahlawan Badai (Storm Hero, 2000pts, eligible for governance proposals) → Legenda Cerah (Clear Sky Legend, 10000pts, permanent badge, Genesis-equivalent trust).

Weather Heroes provide an alternative status path for users who aren't executors but are system guardians. Community health maintenance IS gotong royong.

### Difficulty Weighting

Tandang's three-stage difficulty estimation applies to GR tasks:

- Stage 1 — AI Baseline: instant via Tandang Skill API (`POST /estimate-difficulty`), clamped to [0.1, 0.9]
- Stage 2 — Market Adjustment: tasks unclaimed after 48h get bounty increase (higher difficulty = more reputation reward)
- Stage 3 — Reviewer Calibration: verifier can nudge ±10% based on actual evidence complexity

GR extension: witness complexity (not just task difficulty) affects reputation rewards for witnesses and sensemakers. A well-documented witness about a complex multi-stakeholder situation earns more than "there's a pothole." AI proposes witness complexity at Seed; sensemakers refine at Define.

Adversarial guardrails: final difficulty = weighted average of AI + verifier calibrations. 5% audit sampling of completed problems. Historical anchoring against median similar problems. ESCO mis-tagging for reputation farming = 80% recursive slash for tagger + verifier.

### Domain Efficiency

Tandang tracks verification health per ESCO domain:

`E_domain = Problems_Verified / (Problems_Verified + Problems_Awaiting_Verification)`

| E Score | Status | GR Consequence |
| --- | --- | --- |
| ≥0.9 | Healthy | Normal operations |
| ≥0.7, <0.9 | Sluggish | Warning on domain dashboard |
| ≥0.5, <0.7 | Backlogged | Bounty multiplier 1.2x for verifications |
| <0.5 | Critical | All Keystones/Pillars in domain: -5% earning until E ≥ 0.7 |

This creates collective accountability for verification throughput. High-reputation users in a domain can't just solve problems — they must also verify, or everyone suffers. Fits the gotong royong ethos: the community's work is everyone's responsibility.

Exemptions: users who verified 3+ in past 7 days; new Keystones/Pillars get 60-day grace. Cold start: domains with <10 total problems exempt from E tracking.

### Novice On-Ramps

To prevent incumbent lockout and ensure new users can build reputation:

- 20% of witness corroboration opportunities reserved for Novice-only participation in first 48h
- 20% of verification assignments reserved for Novice-tier verifiers (with Keystone audit oversight)
- 20% of low-difficulty Executor tasks reserved for Novice claimers in first 48h

This prevents a scenario where established Keystones and Pillars monopolize all participation and newcomers can never build reputation. Forces healthy distribution of opportunity.

## Bootstrap & Recovery

### Genesis Layer

GR needs initial trusted users to seed the trust graph. Tandang's genesis layer provides the bootstrap mechanism:

| Pillar | Genesis Source | Required Proof |
| --- | --- | --- |
| Tech & Build | GitHub/GitLab/StackOverflow | Top 2% contributors |
| Field & Social | NGO/UN/PMI (Red Cross) | 5+ years field leadership |
| Logic & Audit | Scopus/IFCN/AJI | H-index or certification |
| Culture & Context | Matan/Majelis/Dikbud | Years of public discourse |

Triangulation Rule (no single "God Node"): Genesis Badge = Source A + Source B + Peer Challenge. All three must pass. Example for Field & Social: NGO verification (5+ years) + LinkedIn (verified community leadership history) + Peer Challenge (3 early-adopters vouch Integrity).

Genesis Sunset: genesis influence is a loan, not a gift. 10% decay per month: `W_g(t) = W_g(0) × 0.9^t`. After ~10 months, genesis users who haven't earned organic GR reputation become Novice like everyone else. This prevents founding-team dynasties — directly aligned with gotong royong's community ownership ethos.

Genesis fraud multiplier: 2.0x (genesis nodes carry highest trust, highest risk). A genesis user caught committing fraud is penalized twice as hard as a regular user.

Matan Genesis (Culture & Context): Matan/Majelis genesis nodes serve as Integrity verifiers (confirm "Good Human") and ESCO-ID taxonomy stewards. They anchor the system's cultural soul and ensure the Indonesian context is preserved, not overwritten by global defaults.

### Recovery Path

Slashed users (non-fraud) can recover through Tandang's structured rehabilitation:

| Stage | Duration | Allowed GR Actions |
| --- | --- | --- |
| Shadow Tier | 30 days | Verify witness evidence only (lowest risk, Keystone-supervised) |
| Probation | Next 30 days | Verify + Execute tasks (no vouch, no endorse) |
| Full Restoration | After probation | All actions restored |

Mentorship Tax during Shadow Tier:
- 75% of earned reputation goes to the recovering user (incentive to continue)
- 15% goes to assigned Keystone/Pillar Recovery Auditor (compensates real audit labor)
- 10% goes to GDF Recovery Pool (recovering user helps heal system damage they contributed to)

GDF Recovery Pool accelerates GDF decay: `GDF_decay_accelerated = standard_decay / (1 + Pool_Factor)` where `Pool_Factor = Recovery_Pool / Total_GDF`.

The recovery path prevents brain drain from "one strike and you're out" cynicism. Good people make mistakes. The system provides a way back that's supervised, gradual, and contributes to community healing.

## AI's Role (Assistant, Not Judge)

AI is allowed to:
- summarize and redact confidential submissions into `WitnessPublished`,
- propose track classification and criteria checklists,
- suggest tasks, dependencies, and testable success criteria,
- detect duplicates, contradictions, and potential gaming patterns,
- extract ESCO skills from witness text via Tandang Skill API,
- estimate witness complexity and task difficulty,
- enforce Context Triad minimum evidence standards (Triad Auditor),
- flag potential ESCO mis-tagging.

AI is not allowed to:
- finalize stage transitions unilaterally,
- override governed votes,
- reveal confidential identity or raw content,
- assign reputation or modify scores directly.

## Data Primer (Published Stub Fields)

In Confidential mode, the platform publishes a stub with structured fields:

- `witness_id` (stable)
- `track` and `stage`
- `claim_summary` (redacted)
- `claim_type` (good / bad / unknown)
- `esco_skills` (ESCO skill references extracted by AI, refined by sensemakers)
- `esco_id_skills` (ESCO-ID references where applicable)
- `pillar_affinity` (primary Four Pillar classification)
- `time_window` (coarse)
- `location_coarse` (optional, coarse)
- `evidence_types_present` (photo/gps/witness/other)
- `context_triad_status` (which of Visual/Locational/Corroborative are present)
- `missing_info_questions` (what's needed next)
- `next_transition_criteria_draft` (AI-proposed checklist)
- `witness_complexity` (AI-estimated, sensemaker-refined)
- `risk_flags` (spam likelihood, contradiction likelihood) (optional)

## Integration With Tandang (Comprehensive)

Tandang is more than the execution layer — it is the reputation substrate on which all GR game theory is built.

### Composability: Target Abstraction

GR registers its primitives as composable targets in Tandang's framework:

| GR Concept | Tandang Target Type | Primitives Used |
| --- | --- | --- |
| Witness | `EndorsementTarget::witness(id)` | Endorsement, MultiVerification |
| Stage Transition | `VerificationTarget::transition(id)` | MultiVerification (quorum vote) |
| Task (Execute) | `ContributionTarget::task(id)` | ContributionAggregate (multi-contributor) |
| Dispute | Existing dispute primitive | Stochastic Jury |
| Impact Attestation | `EndorsementTarget::impact(id)` | Endorsement (from beneficiaries) |

### Execution Layer (Resolve Track)

- `Path` produces a task graph (optional DAG) with ESCO-tagged tasks and AI-estimated difficulty.
- `Execute` tracks task completion via Tandang's problem lifecycle (SUBMITTED → CLAIMED → SOLVED → VERIFIED → CLOSED), PoR evidence, heartbeat monitoring, and verification.
- `Accept` happens when criteria are met and verified through Tandang's verification protocol, with Sincere Disagreement and Cross-Pillar Arbitration available for conflicts.
- Collaborative execution uses Tandang's CollaborativeProblem primitive (gotong royong solving with multiple helpers on a single task, credit distributed via ContributionAggregate).

### Reputation Layer (All Tracks)

- PageRank trust graph built from GR-generated vouches (co-witnessing, verified execution, impact attestation, sensemaking validation).
- Anti-collusion preprocessing before PageRank computation: cycle dampening (Tarjan SCC detection), reciprocity dampening (mutual vouches = 0.7x), temporal burst dampening (3+ vouches in 24h = 0.5x), community conductance dampening (cartel subgraphs with low external edges).
- Dual-layer scoring: I/C/J computed from GR signals per contribution type mapping.
- Tier assignment: percentile-based (Keystone ≥99th, Pillar ≥90th, Contributor ≥60th, Novice <60th, Shadow post-slash). Small-domain policy: <5 users all Novice; 5-19 users max Contributor; ≥20 full range.
- Epoch system: all reputation recalculation happens at deterministic epoch boundaries. Vote weights come from most recent epoch snapshot.

### Accountability Layer

- Slash cascade with specific GR triggers (see Recursive Accountability).
- GDF burn on every slash event.
- Domain Efficiency tracking per ESCO domain.
- Recovery path for non-fraud slashed users.

## Integration With Markov (Contract-Level)

Event types emitted from GR to Tandang (exact schemas versioned):

Witness lifecycle:
- `witness_created(actor, esco_skills[], esco_id_skills[], complexity)`
- `witness_corroborated(co_witness, witness_id)` → creates implicit vouch
- `track_changed(witness_id, old_track, new_track)`
- `stage_transitioned(witness_id, old_stage, new_stage, criteria_hash)`

Execution lifecycle:
- `task_created(witness_id, task_id, esco_skill, difficulty)`
- `task_claimed(executor, task_id)` → C_eff eligibility check
- `task_progress(executor, task_id)` → heartbeat
- `task_released_for_stall(executor, task_id)` → J penalty
- `task_completed(executor, task_id)`
- `por_evidence_submitted(actor, task_id, evidence_hash)` → PoR recorded

Verification lifecycle:
- `verification_recorded(verifier, task_id, outcome)` → implicit vouch if accepted
- `verifier_disagreement(verifier_a, verifier_b, task_id)` → Sincere Disagreement
- `auto_verify_triggered(task_id)` → 75% credit + escrow

Impact lifecycle:
- `impact_attested(beneficiary, witness_id)` → weak vouch + Type D

Governance lifecycle:
- `transition_endorsed(endorser, witness_id, budget_locked)`
- `transition_outcome(witness_id, upheld/overturned)` → J updates
- `dispute_opened(challenger, witness_id, stake)` → dispute lifecycle
- `dispute_resolved(outcome, jurors[])` → J updates + stake resolution

Trust lifecycle:
- `skeptical_vouch_placed(skeptic, target, stake)`
- `skeptical_vouch_resolved(skeptic, target, outcome)`
- `implicit_vouch_created(from, to, esco_skill, source_event)` → 14-day bleed-in

Slash lifecycle:
- `fraud_detected(actor, witness_id, severity, trigger)`
- `slash_executed(actor, severity, cascade_depth)`
- `clawback_initiated(witness_id, chain[])`
- `gdf_burn(amount)`

The Markov Engine is responsible for:
- building and maintaining the trust graph from vouch events,
- computing PageRank with anti-collusion preprocessing at each epoch,
- updating I/C/J scores based on validated events per contribution type,
- weighting votes and privileges via compound multiplier,
- enforcing decay per contribution type and inactivity triggers,
- tracking GDF and domain efficiency,
- providing anti-abuse signals (as policy inputs),
- managing tier assignment, jury selection, and verifier eligibility.

## Recursive Accountability (Clawback / Recursive Slash)

When late-stage information shows a witness was fraudulent or grossly misleading, the system must be able to unwind credit and penalize weak verification.

### Tandang Cascade Formula

`Penalty_B = S × (w_{B→A} / R_B) × δ^level`

Where:
- S = base slash percentage (0.5 default)
- w_{B→A} = weight of vouch B gave to A (including implicit vouches from GR activities)
- R_B = B's reputation at time of vouch
- δ = cascade decay per hop (0.5 default)
- Max depth: 3 levels
- Min penalty to stop: 5%

### GR-Specific Slash Triggers

| GR Trigger | Tandang Severity | Who Gets Hit |
| --- | --- | --- |
| Fabricated witness evidence (forged photos, false claims) | 100% (Confirmed Fraud) | Witness → co-witnesses → endorsers who advanced transitions |
| ESCO mis-tagging for reputation farming | 80% (Metadata Fraud) | Tagger + endorsers of that transition |
| Collusion ring fast-tracking transitions | 60% (Vouch-Ring / Collusion) | All ring members (detected via community conductance) |
| Rubber-stamp verification (approved false evidence) | 20% (Verification Failure) at 1.5x verifier liability | Verifier + their vouchers |
| Repeated bad endorsements (3+ overturned transitions) | 30% (Bad Judgment) | Endorser |
| Bribery for impact attestation | 100% (Confirmed Fraud) | Attestor + witness + intermediaries |

### Cascade Flow in GR Context

```
FRAUD DETECTED: Witness A fabricated evidence
        │
        ▼
LEVEL 0: Slash A → R_A = 0, I_A = 0, Shadow Tier
        │
        ▼
LEVEL 1: Penalize direct vouchers (co-witnesses, endorsers who advanced transitions)
         Apply: Penalty = S × (w/R) × δ^1
         Apply: J = J - 0.10
        │
        ▼
CHECK: R_new < 0.7 × R_old?
       YES → Cascade to Level 2
       NO  → Stop
        │
        ▼
LEVEL 2: Penalize vouchers of vouchers
         Apply: Penalty = S × (w/R) × δ^2
         Max Depth: 3 levels; Min Penalty: 5%
        │
        ▼
GDF BURN: 10% of total slashed reputation → Global Difficulty Floor
         Weather degrades; everyone's earning drops
```

Clawback is a surgical strike, not a total reset. The goal is to unwind reputation gained from the specific fraudulent chain (witness → transitions → verifications → impact attestations), not to erase unrelated history.

## Emergency Override (Brake Clause)

In rare cases of systemic attack (bribery rings, coordinated fraud), the protocol supports an emergency "brake" action:
- Eligibility: Keystone tier AND I ≥ 0.9 AND J ≥ 0.8. This high threshold prevents casual use while ensuring the brake is available when genuinely needed.
- The brake triggers an immediate jury audit on a transition, freezes progression temporarily, and creates an audit trail.
- Abuse penalty: 30% slash. This heavy penalty prevents censorship-by-brake.
- Emergency fast-track transitions (bypassing standard voting) require mandatory 7-day post-hoc Stochastic Jury audit. Unjustified fast-tracks → endorser J penalty (-0.05). Justified → endorser J bonus (+0.02).

## Threat Model (Non-Exhaustive)

Even with Markov, the system must consider specific threats. Each threat is paired with the specific Tandang mechanisms that defend against it:

| Threat | Description | Tandang Defense |
| --- | --- | --- |
| Collusion | High-rep clique fast-tracking transitions | Cycle dampening (Tarjan SCC), reciprocity dampening (0.7x mutual vouches), temporal burst dampening (3+ vouches in 24h = 0.5x), community conductance dampening (cartel subgraph detection), diversity guard on votes |
| Bribery | Paid verifications or impact attestations | GDF + Weather (fraud hurts everyone's earning → collective vigilance), verifier 1.5x liability, recursive slash cascade, time-delayed impact confirmation |
| Griefing | Blocking transitions via frivolous disputes | Dispute stake (3% R, min 0.05), suspension quorum (2 independent), rate limits (3 max open, 14-day cooldown), percentile gate (≥30), J penalty for frivolous disputes |
| Spam witnesses | Low-effort noise flooding the system | Identity tiers (8x power difference Anonymous→Public), vouch budget caps, ESCO skill requirements for endorsement weight, governance budget consumption per endorsement |
| Privacy harm | Doxxing via details in witness content | Confidential mode with published stub, identity tier orthogonal to disclosure mode, downgrade prevention |
| Incumbent lockout | Keystones monopolizing all opportunities | Novice on-ramps (20% reservation, 48h window), competence decay (90-day+ half-lives), genesis sunset (10-month founder fadeout) |
| Sybil attack | Bot networks creating fake trust | PageRank flow math (bots get zero inbound trust), identity tiers, genesis triangulation, vouch bleed-in (14 days — bots can't instant-trust) |
| Legacy hoarding | Inactive users retaining influence | Competence decay per contribution type, inactivity triggers (14-120 days), genesis sunset |
| Verification bottleneck | Accept stage stalling due to no verifiers | Auto-verify (14-day timeout, 75% + escrow), domain efficiency tracking (E < 0.5 → Keystone/Pillar earning penalty), bounty multiplier for backlogged domains |
| Rubber-stamping | Verifiers approving without checking | Verifier 1.5x liability, J tracking (3+ bad verifications → Shadow), random audit sampling (5% re-verification) |

Additional mitigations carried from GR design:
- Challenge windows + jury arbitration at every transition
- Append-only history, criteria versioning
- Budgeted governance (endorsements cost attention budget)
- Cross-pillar arbitration for multi-domain disputes

## CV Hidup Output

Users who participate in GR accumulate ESCO-verified credentials exported via Tandang's CV Hidup (Living CV):

- Dynamic QR code (live link, not static PDF — becomes stale if user goes inactive due to decay)
- ESCO skills with percentile ranking (EU-compatible, interoperable)
- ESCO-ID skills for Indonesian cultural competencies
- Verified contributions (anonymized if Confidential mode was used)
- Integrity score tier
- Timestamp (credentials are time-sensitive due to decay)
- GR-specific: roles performed (Witness, Sensemaker, Executor, Verifier), tracks contributed to, witnesses that reached Impact

This gives GR participants a tangible, portable credential output beyond platform reputation.

## Tandang Extensions (Implemented)

GR required 13 extensions to Tandang. All have been implemented. Specifications in the "Tandang Gap Specifications" section; implementation prompts used during development in `gotong-royong/docs/design/context/TANDANG-GAP-PROMPTS.md`.

| Gap | Description | Status |
| --- | --- | --- |
| 1. Vouch-to-entity (VouchContext) | VouchContext enum + field on Vouch struct | Implemented |
| 2. Witness as Target type | "witness", "transition", "impact" discriminators | Implemented |
| 3. Governance Budget primitive | New module: unified pool with vouch budget | Implemented |
| 4. Witness complexity dimension | WitnessComplexity type + scoring formula | Implemented |
| 5. J impact table for governance | 11 new JEvent variants for governance outcomes | Implemented |
| 6. Configurable consensus thresholds | ConsensusType::Custom { threshold } variant | Implemented |
| 7. Celebrate track composition | App-layer composition from Type D + Endorsement | Implemented |
| 8. Context Triad evidence profile | ContextTriad + ContextTriadRequirement in PoR | Implemented |
| 9. Emergency Brake primitive | New emergency/ module with eligibility + audit | Implemented |
| 10. Novice On-Ramp slot reservation | NoviceReservationPolicy (20%, 48h window) | Implemented |
| 11. Diversity Guard for vote quorum | Max 40% cluster weight check on votes | Implemented |
| 12. (Covered by Gap 6) | — | — |
| 13. Emergency Fast-Track + post-hoc audit | Fast-track with mandatory 7-day jury audit | Implemented |

Total: 2 new domain modules, 6 DB migrations, 2 enum extensions, 2 policy additions, 1 app-layer composition. No PageRank formula changes. No architectural changes. Everything else uses Tandang as-is.

## Roadmap (Recommended)

v0.1 (Concept → MVP)
- **Tandang gaps:** Vouch-to-entity (Gap 1), Witness target type (Gap 2), Governance Budget (Gap 3), J governance events (Gap 5), Context Triad (Gap 8), Emergency Brake (Gap 9)
- **GR core:** Witness primitive + Resolve track + stage state machine
- Confidential mode with AI redaction pipeline + published stub
- ESCO skill tagging via Tandang Skill API (auto at Seed, refinable at Define)
- Proposal/vote for stage transitions with Markov-weighted voting (compound multiplier)
- Resolve track integrated with Tandang task lifecycle (SUBMITTED → CLAIMED → SOLVED → VERIFIED → CLOSED)
- Contribution type mapping: Executor (A), Verifier (A), Witness (C), Sensemaker (C), Co-witness (D), Beneficiary (D), Endorser/Challenger/Juror (J-only)
- Implicit vouch generation: co-witnessing → vouch (14-day bleed-in), verified execution → vouch
- Budgeted governance: unified pool with vouch budget, J scoring for endorsement accuracy
- Competence decay per contribution type (120d-365d half-lives)
- Emergency Brake (Keystone + I ≥ 0.9 + J ≥ 0.8) + Emergency Fast-Track with post-hoc audit (Gap 13)
- Event bus: GR → Tandang event emission (all lifecycle events)
- AI pipeline: redaction, ESCO tagging, complexity estimation, Triad Auditor, duplicate detection

v0.2 (Hardening)
- **Tandang gaps:** Witness complexity (Gap 4), Diversity Guard (Gap 11), Novice On-Ramp (Gap 10)
- Disputes/jury flows: Stochastic Jury, Cross-Pillar Arbitration
- Sincere Disagreement Protocol at Accept stage
- Skeptical vouches for suspect witnesses
- GDF & Weather System integration + Weather Report UI
- Domain Efficiency tracking per ESCO domain
- Auto-verify timeout handling (14-day → 75% + escrow)
- Diversity guard (max 40% quorum weight from single cluster)
- Novice On-Ramps (20% reservation, 48h windows)
- Anti-collusion preprocessing validation (cycle, reciprocity, temporal burst, conductance dampening)
- Adversarial simulation: all 7 scenarios passing
- Stronger privacy controls + audit logs

v0.3 (Bootstrap & Scale)
- **Tandang gaps:** Configurable consensus thresholds (Gap 6), Celebrate composition (Gap 7)
- Genesis Layer activation (triangulation, rapid sunset, Matan genesis for Culture & Context)
- Recovery Path (Shadow → Probation → Full Restoration + Mentorship Tax)
- Weather Heroes recognition system (Penjaga Cuaca → Legenda Cerah)
- Celebrate track fully wired (Type D + Endorsement)
- Explore track fully wired (Type C + A composition, 70% conclusion threshold)
- CV Hidup output for GR participants (ESCO + ESCO-ID credentials)
- Polymath Bonus for cross-domain contributors
- ESCO-ID expansion: ID.002 (Local Wisdom) + ID.004 (Indonesian Craft & Heritage)

v0.4 (Economics)
- Optional mutual credit layer bound to verified execution and confirmed impact
- Difficulty weighting with three-stage estimation (AI + market + reviewer calibration)
- Advanced gaming pattern detection (ML-based endorsement anomaly detection)
- Federation readiness (event log replication, deterministic replay verification)

## Sustainability Model (Draft Idea)

The coordination engine at GR's core — LLM-driven block composition, dual-layer conversation+structured data, iterative human-AI editing, 7 universal block primitives — is domain-agnostic. It coordinates neighborhoods, but the same engine coordinates teams, projects, and organizations.

**Potential model**: offer the coordination engine as a private/enterprise service. Organizations pay for their own instances (project management, internal coordination, incident response). Revenue feeds back to fund the public community platform.

This inverts the typical model: instead of the community subsidizing a product, the product subsidizes the community. The public Gotong Royong instance becomes free and sustainable — funded by private adoption of the same engine, not by ads or VC runway.

Open questions for later: licensing structure, data isolation between private and public instances, whether private instances participate in the Tandang trust graph or operate independently, feature differentiation (private may not need Weather/GDF/reputation).

*This section is a recorded idea, not a locked decision. Revisit during v0.4 Economics phase.*

## Technical Architecture

### Data Model

Core entities for the GR application layer (Tandang entities like Reputation, Vouch, Score are managed by the engine):

```
Witness {
  id: UUID
  source_id: UUID              → WitnessSource (private)
  published_id: UUID           → WitnessPublished (public stub)
  track: enum(resolve, celebrate, explore)
  stage: String                → current stage
  esco_skills: ESCO_Ref[]      → AI-tagged, sensemaker-refined
  esco_id_skills: ESCO_ID_Ref[]
  pillar_affinity: enum(tech_build, field_social, logic_audit, culture_context)
  claim_type: enum(good, bad, unknown)
  complexity: Decimal           → witness complexity score
  disclosure_mode: enum(community_open, confidential, fully_open)
  created_at: Timestamp
  archived_at: Option<Timestamp>
}

WitnessSource {
  id: UUID
  witness_id: UUID
  actor_id: UUID               → platform-scoped, links to Markov identity
  raw_text: String
  attachments: Attachment[]
  metadata: JSON
}

WitnessPublished {
  id: UUID
  witness_id: UUID
  version: u32                 → increments on criteria/stage changes
  claim_summary: String        → redacted by AI in confidential mode
  ... (full schema in Appendix A)
}

Proposal {
  id: UUID
  witness_id: UUID
  proposal_type: enum(stage_transition, track_change, criteria_update, dispute)
  proposer: UUID
  target_stage: Option<String>
  target_track: Option<String>
  criteria_hash: Hash          → SHA256 of criteria checklist
  evidence_profile: ContextTriadRequirement
  status: enum(voting, approved, rejected, challenged, expired)
  quorum_met: bool
  weighted_threshold_met: bool
  challenge_window_end: Option<Timestamp>
  created_at: Timestamp
}

Vote {
  id: UUID
  proposal_id: UUID
  voter: UUID
  weight: Decimal              → computed from Tandang compound multiplier
  budget_locked: Decimal       → governance budget consumed
  direction: enum(approve, reject)
  cast_at: Timestamp
}

Task {
  id: UUID
  witness_id: UUID
  esco_skill: ESCO_Ref
  difficulty: Decimal           → three-stage estimated
  status: enum(submitted, claimed, solved, verified, closed, expired, abandoned, rejected, disputed, auto_verified)
  executor: Option<UUID>
  verifier: Option<UUID>
  por_evidence: EvidenceRef[]
  last_heartbeat: Option<Timestamp>
  created_at: Timestamp
}

Dispute {
  id: UUID
  witness_id: UUID
  proposal_id: Option<UUID>    → if disputing a specific transition
  challenger: UUID
  stake: Decimal               → 3% of challenger's R
  status: enum(open, jury_selected, resolved)
  jury: UUID[]                 → stochastic jury members
  outcome: Option<enum(upheld, dismissed, subjective)>
  created_at: Timestamp
}

SkepticalVouchRecord {
  id: UUID
  skeptic: UUID
  target: UUID
  witness_id: Option<UUID>     → scoped to specific witness if applicable
  stake: Decimal
  status: enum(active, resolved, expired)
  outcome: Option<enum(correct, incorrect)>
  expires_at: Timestamp         → 90-day auto-expiry
}
```

### API Surface

```
Witness lifecycle:
  POST   /witnesses                          → create witness
  GET    /witnesses/:id                      → get published stub (or full if authorized)
  POST   /witnesses/:id/corroborate          → co-witness
  PATCH  /witnesses/:id/esco-tags            → refine ESCO tags (requires vote)
  GET    /witnesses/:id/history              → append-only stage/criteria history

Governance:
  POST   /witnesses/:id/proposals            → propose transition/track change/criteria update
  POST   /proposals/:id/votes                → cast vote (locks governance budget)
  GET    /proposals/:id                      → proposal status + vote tally
  POST   /witnesses/:id/disputes             → open dispute (bonds stake)
  GET    /disputes/:id                       → dispute status + jury

Execution (Resolve track):
  GET    /witnesses/:id/tasks                → task DAG for witness
  POST   /tasks/:id/claim                    → claim task (C_eff check)
  POST   /tasks/:id/heartbeat               → progress heartbeat
  POST   /tasks/:id/solve                    → submit solution + PoR evidence
  POST   /tasks/:id/verify                   → verify solution (verifier)
  POST   /tasks/:id/dispute                  → dispute verification outcome

Impact:
  POST   /witnesses/:id/impact               → attest impact (beneficiary/adopter)
  GET    /witnesses/:id/impact               → impact attestation status

Trust:
  POST   /witnesses/:id/skeptical-vouch      → place skeptical vouch
  GET    /users/:id/vouches                  → vouch portfolio (active, skeptical, burned)

System:
  GET    /weather                            → current GDF weather report
  GET    /domains/:esco_skill/efficiency     → domain efficiency score
  GET    /users/:id/reputation               → tier + public scores (from Tandang)
  GET    /users/:id/cv-hidup                 → living CV export
```

### Event Bus

GR → Tandang communication is event-driven, append-only, and deterministic:

```
GR Application → Event Bus → Tandang Event Processor → Reputation Store
                                    ↓
                              Epoch Boundary
                                    ↓
                              PageRank Recompute
                                    ↓
                              Tandang → GR (reputation updates for UI)
```

Events are sequenced with monotonically increasing sequence numbers. Ordering is by sequence number, NOT timestamp. Duplicate event IDs are silently ignored (idempotency). All state-changing GR operations emit exactly one event to Tandang.

GR subscribes to Tandang's reputation update stream for rendering current tier, scores, weather, and domain efficiency in the UI.

### AI Pipeline

```
                   WitnessSource (raw input)
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
         Redaction LLM        ESCO Tagger
         (confidential)       (Tandang Skill API)
                │                   │
                ▼                   ▼
        WitnessPublished     esco_skills[]
         (claim_summary)     esco_id_skills[]
                │                   │
                └─────────┬─────────┘
                          ▼
                   Complexity Estimator
                   (Tandang Skill API)
                          │
                          ▼
                    Triad Auditor
                    (rule-based)
                          │
                          ▼
                  Duplicate Detector
                  (embedding similarity)
                          │
                          ▼
                  Gaming Pattern Detector
                  (statistical anomaly)
```

Redaction: LLM takes WitnessSource → produces claim_summary with PII stripped. Identity references replaced with role labels ("the reporter", "the affected community"). Location coarsened to region level. Specific dates coarsened to time windows.

ESCO Tagger: Tandang Skill API `POST /extract-skills` with witness text. Returns ranked ESCO skill URIs with confidence scores. Sensemakers can override at Define stage.

Complexity Estimator: Tandang Skill API `POST /estimate-difficulty` with `complexity_mode: "witness"`. Returns [0.1, 0.9] score based on domain count, stakeholder count, evidence requirements, and historical complexity of similar witnesses.

Triad Auditor: Rule-based check against evidence profile. Returns pass/fail per triad element + friendly checklist of missing items. Does NOT block Seed creation — only blocks transition proposals from entering voting.

Duplicate Detector: Embedding similarity search against active witnesses. Threshold: cosine similarity > 0.85 flags as potential duplicate. AI explains the overlap; human decides whether to merge or keep separate.

Gaming Pattern Detector: Statistical anomaly detection on endorsement patterns. Flags: rapid mutual endorsement cycles, endorsement concentration from single cluster, temporal burst patterns. Feeds into Tandang's anti-collusion preprocessing.

### State Machine Engine

Each witness has a state machine instance. The engine is deterministic: same events in same order → same state.

```
Transition lifecycle:
  1. Proposer submits transition proposal
  2. AI Triad Auditor checks evidence profile → blocks if insufficient
  3. Proposal enters voting period (48h default)
  4. Votes accumulate (Markov-weighted, budget-consuming)
  5. If quorum + threshold met → transition approved
  6. Challenge window opens (48-72h)
  7. If no dispute → transition commits
  8. If dispute → freeze, route to Stochastic Jury
  9. Jury verdict → commit or revert
  10. All outcomes emit events to Tandang
```

Track changes follow the same lifecycle but require higher quorum (1.5x standard) and longer challenge window (72h minimum).

Criteria updates are versioned. Each criteria version is identified by SHA256 hash. The hash is recorded in Tandang events for auditability.

## Tandang Gap Specifications

Precise specs for the 8 extensions GR requires in Tandang:

### Gap 1: Vouch-to-Entity

**Current:** `VouchTarget::User(UserId)` — vouches are person-to-person only.

**Extension:** Add entity-scoped metadata to vouches without changing PageRank computation:

```rust
struct Vouch {
    from: UserId,
    to: UserId,
    skill: EscoSkillRef,
    weight: Decimal,
    phase: BleedInPhase,
    // NEW: optional entity scope
    source_context: Option<VouchContext>,
}

enum VouchContext {
    CoWitness { witness_id: UUID },
    VerifiedExecution { task_id: UUID },
    ImpactAttestation { witness_id: UUID },
    SensemakingValidation { witness_id: UUID, criteria_hash: Hash },
}
```

PageRank computation is unchanged — it still operates on the user-to-user trust graph. The `VouchContext` is metadata for auditability, clawback scoping, and UI display. When a clawback targets a specific witness chain, only vouches with matching `VouchContext.witness_id` are affected.

**Scope:** Domain crate change (Vouch struct), application crate change (vouch creation commands), infrastructure crate change (DB migration for context column).

### Gap 2: Witness as Target Type

**Current:** Target discriminators include `problem`, `solution`, `proposal`, `content`.

**Extension:** Add `witness`, `transition`, `impact` discriminators:

```rust
// No structural change — already uses String discriminator
let target = EndorsementTarget::new(witness.id(), "witness");
let target = VerificationTarget::new(transition.id(), "transition");
let target = EndorsementTarget::new(impact_attestation.id(), "impact");
```

**Scope:** Config-level change. Add discriminator strings to allowed list. No domain logic changes.

### Gap 3: Governance Budget

**New primitive** in domain crate:

```rust
struct GovernanceBudget {
    user_id: UserId,
    total: Decimal,       // = R × budget_factor - vouch_allocated
    locked: Decimal,      // committed to active proposals
    available: Decimal,   // total - locked
}

// Commands
LockGovernanceBudget { user_id, proposal_id, amount }
ReleaseGovernanceBudget { proposal_id, outcome }

// Budget return policy
enum TransitionOutcome {
    UpheldReachedImpact,   // 100% return + 5% bonus
    UpheldReachedAccept,   // 100% return
    UpheldStalled,         // 50% return
    Overturned,            // 0% return (burned)
    FraudDiscovered,       // 0% return + J penalty
}
```

Unified with vouch budget: `total_influence = R × 0.3`. This total is split between vouch allocations and governance endorsements. Spending on one reduces availability for the other.

**Scope:** New domain module (`governance_budget/`), application commands/queries, infrastructure DB table, API endpoints for budget display.

### Gap 4: Witness Complexity

**Extension to existing difficulty system:**

```rust
struct WitnessComplexity {
    ai_estimate: Decimal,           // [0.1, 0.9] from Skill API
    sensemaker_override: Option<Decimal>,  // human refinement at Define
    final_score: Decimal,           // weighted average
}

// Sensemaker scoring formula
sensemaker_reward = base × witness_complexity.final_score × outcome_quality
```

Skill API extension: `POST /estimate-difficulty` gains `complexity_mode` parameter. When `complexity_mode = "witness"`, the API considers domain count, stakeholder count, evidence requirements, and historical complexity of similar witnesses (vs. task-level binary difficulty for `complexity_mode = "task"`).

**Scope:** Domain crate (add field to Problem or new WitnessComplexity type), Skill API endpoint extension, scoring formula modification.

### Gap 5: J Impact Table for Governance

**Extension to existing J scoring engine:**

```rust
enum JEvent {
    // Existing
    VouchForHighPerformer,      // +0.02
    VouchForSlashedUser,        // -0.10
    VouchForFraud,              // -0.20
    // ... existing events

    // NEW: Governance
    GovernanceEndorsementUpheldImpact,   // +0.02
    GovernanceEndorsementUpheldAccept,   // +0.01
    GovernanceEndorsementOverturned,     // -0.05
    GovernanceEndorsementFraud,          // -0.10
    DisputeOpenedUpheld,                 // +0.02
    DisputeOpenedFrivolous,              // -0.03
    JuryVotedWithMajority,               // +0.02
    JuryVotedAgainstSubjective,          // 0
    JuryVotedAgainstObjective,           // -0.03
    CoWitnessValidated,                  // +0.02
    CoWitnessSlashed,                    // -0.10
}
```

**Scope:** Domain crate (add variants to existing J event enum), application crate (emit new events from governance handlers). Small change.

### Gap 6: Explore Track Composition

No new primitive needed. Compose from existing types:

```
Explore Phase → Tandang Primitive → Verification

Seed          → Type C contribution       → Peer consensus (60%)
Hypotheses    → Type C contribution       → Peer consensus (60%)
Experiments   → Type A task (if binary)   → Binary verification + PoR
              → Type C observation (else)  → Peer consensus (60%)
Conclusion    → Type C contribution       → Peer consensus (70% — elevated threshold)
```

The elevated 70% threshold for Conclusion is configured per-application in the composability framework — Tandang's MultiVerification primitive already supports configurable `ConsensusType::SuperMajority(threshold)`.

**Scope:** Application-layer composition only. No domain changes.

### Gap 7: Celebrate Track Composition

No new primitive needed. Compose from existing types:

```
Celebrate Phase → Tandang Primitive → Verification

Seed            → Type D contribution       → Social quality spectrum
Corroborate     → Type D + Endorsement      → Peer consensus (60%)
Recognize       → MultiVerification         → Quorum vote
Impact          → Type D contribution       → Impact attestation
```

**Scope:** Application-layer composition only. No domain changes.

### Gap 8: Context Triad as Evidence Profile

**Extension to existing PoR:**

```rust
struct ContextTriad {
    visual: Option<EvidenceRef>,          // photo/video/doc scan
    locational: Option<LocationEvidence>, // coarse GPS/map/label
    corroborative: Vec<CoWitnessRef>,     // co-witnesses with I > 0
}

struct ContextTriadRequirement {
    required_at_stage: String,
    min_of_three: u8,  // 0, 1, 2, or 3
    // Optional per-element requirements
    require_visual: bool,
    require_locational: bool,
    min_corroborators: u8,
}
```

Added as optional structured field to PoR submission API. Triad Auditor (AI, rule-based) validates submissions against requirements before allowing transition proposals to enter voting.

**Scope:** Domain crate (new types in `por/` module), API extension (structured evidence field), infrastructure DB migration.

### Gap Summary

| Gap | New Domain Code | New API | DB Migration | Complexity |
| --- | --- | --- | --- | --- |
| 1. Vouch-to-entity | VouchContext enum + field | No | Yes (context column) | Medium |
| 2. Witness target | Config addition | No | No | Trivial |
| 3. Governance Budget | New module | Yes | Yes (new table) | Medium |
| 4. Witness complexity | Field + scoring formula | API param | Yes (column) | Small |
| 5. J governance events | Enum variants | No | No | Small |
| 6. Explore composition | None | Composition config | No | Trivial |
| 7. Celebrate composition | None | Composition config | No | Trivial |
| 8. Context Triad | New types in PoR | API field | Yes (structured evidence) | Small |

Total: 1 new domain module, 4 DB migrations, 2 enum extensions, 1 API parameter, 2 composition configs. No architectural changes. No PageRank formula changes.

## Simulation & Parameter Validation

GR-specific adversarial scenarios to validate using Tandang's simulation framework (5 agent archetypes: HonestAgent, StrategicAgent, SybilAgent, GriefingAgent, CartelAgent).

### Scenario 1: Witness Fast-Track Collusion

**Setup:** 10 CartelAgents create witnesses and rapidly endorse each other's transitions. 990 HonestAgents operate normally.

**Expected defenses:**
- Community conductance dampening detects the tight cluster (low external edges)
- Reciprocity dampening (0.7x) reduces mutual vouch weight from co-witnessing
- Temporal burst dampening (0.5x) triggers if 3+ vouches in 24h
- Diversity guard blocks if >40% of quorum weight comes from same vouch cluster

**Pass criteria:** Cartel witnesses advance no faster than honest witnesses of similar quality. Cartel members hold ≤10% of top-tier positions.

### Scenario 2: Dispute Flooding (Griefing)

**Setup:** 1 GriefingAgent opens maximum disputes (3 simultaneous) on valid transitions. 999 HonestAgents.

**Expected defenses:**
- Dispute stake: 3% R × 3 disputes = 9% R at risk
- All disputes frivolous → 3 × (stake burned + J penalty -0.03) = -0.09 J
- After 2-3 cycles: griefer hits J < 0.3 → Shadow Tier (no more disputes for 30 days)
- 14-day cooldown prevents re-targeting same witness

**Pass criteria:** Griefer's net ROI is deeply negative (>20% R loss). Valid transitions delayed by at most one challenge window (72h). Griefer reaches Shadow within 60 days.

### Scenario 3: Sybil Witness Spam

**Setup:** 50 SybilAgents (Anonymous tier) each submit 10 low-effort witnesses. 500 HonestAgents.

**Expected defenses:**
- Anonymous tier = 0.5x ID_Mult
- No inbound vouches → zero PageRank → zero endorsement weight
- Witnesses cannot advance through governance (no weighted endorsements meet quorum)
- Governance budget = 0 (R × 0.3 where R ≈ 0)

**Pass criteria:** Zero Sybil witnesses advance past Seed. Sybil accounts remain Novice. System overhead (reviewing spam proposals) is bounded by rate limits.

### Scenario 4: Rubber-Stamp Verification Ring

**Setup:** 5 StrategicAgents alternate verifying each other's Execute→Accept transitions without checking evidence. 995 HonestAgents.

**Expected defenses:**
- 5% audit sampling catches invalid verifications
- First catch: 20% slash at 1.5x verifier liability = 30% effective penalty
- J degrades: -0.10 per bad verification
- After 3 bad verifications: J < 0.3 → Shadow (no more verification)
- Cascade hits their vouchers (co-witnesses who vouched for them)

**Pass criteria:** Ring detected within 20 verification cycles (expected: 1 in 20 sampled). All ring members in Shadow within 90 days. Cascade penalties distributed to vouchers.

### Scenario 5: Legacy Sensemaker Hoarding

**Setup:** 10 agents set criteria early, stop contributing after 90 days. 990 active agents.

**Expected defenses:**
- Type C decay: 180-day half-life
- After 180 days inactive: competence = 50% of peak
- After 360 days inactive: competence = 25% of peak
- Tier drops (percentile shrinks as active users earn)
- Governance influence drops proportionally (weight = f(percentile))

**Pass criteria:** Inactive sensemakers drop to Novice tier within 12 months. Active sensemakers with equal initial competence surpass them within 6 months.

### Scenario 6: Bribery for Impact Attestation

**Setup:** 1 CartelAgent bribes 3 HonestAgent-appearing beneficiaries to attest false impact.

**Expected defenses:**
- Impact confirmation requires 30-day delay (detection window)
- Minimum 2 independent attestations (briber needs 2+ accomplices)
- If discovered: 100% fraud slash on all attestors + witness
- 10% GDF burn from total slashed reputation
- Cascade to co-witnesses and endorsers who advanced transitions

**Pass criteria:** Bribery net ROI is negative (combined slash loss > any benefit). GDF weather degrades visibly, increasing community vigilance. Briber's vouch network takes collateral damage.

### Scenario 7: Cross-Pillar Manipulation

**Setup:** 3 CartelAgents from Tech pillar try to override a Culture & Context assessment using technical authority.

**Expected defenses:**
- Cross-Pillar Arbitration: jury must include 2 from each disputed pillar + 1 from Logic & Audit
- Verdict requires at least one vote from each pillar represented
- 3-2 where all 3 from same pillar = INVALID
- Pillar Sovereignty: no pillar unilaterally overrides another's domain assessment
- Tech-only Competence cannot substitute for Culture & Context C_eff in jury eligibility

**Pass criteria:** Tech cartel cannot override Culture assessment. Cross-pillar jury produces balanced verdict. Cartel members gain no reputation in Culture & Context domain.

### Simulation Implementation

Each scenario runs as a BDD feature file in Tandang's `tests/bdd/features/gotong-royong/` directory:

```gherkin
Feature: Witness Fast-Track Collusion Resistance
  Scenario: Cartel of 10 users cannot fast-track witness transitions
    Given 990 honest agents and 10 cartel agents
    And cartel agents create witnesses and endorse each other
    When 100 governance cycles complete
    Then cartel witnesses advance no faster than honest witnesses
    And cartel members hold less than 10% of Keystone positions
    And community conductance dampening triggered for cartel cluster
```

Pass criteria are hard-coded assertions. Any parameter change that breaks a scenario is rejected until recalibrated.

## Design Decisions (Resolved)

Previously listed as open questions. These are now firm decisions with rationale.

### Governance Parameters

**Minimum eligibility for voting:** I ≥ 0.3 AND J ≥ 0.3 AND not Shadow Tier. For disputes: same requirements plus percentile ≥ 30. Rationale: low bar allows broad participation while excluding proven bad actors. The compound multiplier (I_Mult × J_Mult) already de-weights low-quality voters without gatekeeping them entirely.

**Diversity guard:** Max 40% of quorum weight from any single vouch cluster (detected via community conductance analysis). Rationale: simple to implement (Tandang already computes community conductance for anti-collusion), strong enough to prevent clique capture, not so strict that it blocks legitimate domain communities.

**Governance budget pool:** SINGLE POOL unified with vouch budget. `total_influence = R × 0.3`, split between vouch allocations and governance endorsements. Spending on one reduces availability for the other. Rationale: prevents double-spending influence. A user who vouches heavily has less governance power, and vice versa. Forces prioritization of influence.

**Governance budget return rate:** Transition upheld + witness reaches Impact → 100% return + 5% bonus. Transition upheld + witness reaches Accept → 100% return. Transition upheld + witness stalls → 50% return. Transition overturned → 0% return (burned). Transition involved in fraud → 0% return + J penalty. Rationale: graduated return incentivizes endorsing witnesses that go the distance, not just witnesses that pass the next gate.

### Evidence & Verification

**Default evidence profiles per track:**

| Track | Transition | Context Triad Minimum |
| --- | --- | --- |
| Resolve | Seed → Define | 1-of-3 |
| Resolve | Define → Path | 2-of-3 |
| Resolve | Path → Execute | 2-of-3 |
| Resolve | Execute → Accept | 3-of-3 (all three required) |
| Celebrate | Seed → Corroborate | 1-of-3 |
| Celebrate | Corroborate → Recognize | 2-of-3 |
| Explore | Seed → Hypotheses | 0-of-3 (questions don't need evidence) |
| Explore | Hypotheses → Experiments | 1-of-3 |
| Explore | Experiments → Conclusion | 2-of-3 |

High-risk domains (policy-defined per ESCO group): health (3.2), legal (2.6), and infrastructure (2.1) require 3-of-3 from Define → Path onward. AI proposes the evidence profile; community locks it via vote.

**Progress heartbeat policy:** Every 7 days for standard tasks (difficulty < 0.5). Every 3 days for high-difficulty tasks (difficulty ≥ 0.5). A heartbeat is any of: text update (≥50 characters), evidence upload (photo/document), linked external artifact (commit, PR, document). 14 days of silence = Stalled. Stalled with no response for 7 more days = Released.

**Impact confirmed definition:** Minimum 30-day delay after Accept. Minimum 2 independent attestations from beneficiaries/adopters with I > 0 (non-zero Integrity) to confirm impact and distribute reputation to the witness chain. Domain-specific time windows: disaster relief = 30 days, infrastructure = 90 days, education = 180 days, health = 90 days. Attestations during the window are collected; after the window closes, impact is evaluated. Note: the higher bar of 3 independent attestations (see Vouch Mechanics below) applies specifically to triggering deferred sensemaker vouches — generating trust graph edges requires stronger evidence than confirming impact alone.

### ESCO & Domain

**C_eff threshold for executor task claiming:** 0.3 (low bar — allows learning by doing). Verifier threshold remains 0.5 (higher — verification requires demonstrated competence). Rationale: the system should encourage participation. Verification is the quality gate, not claiming. Novice on-ramps (20% reservation) ensure low-C_eff users get opportunities.

**Witness complexity vs. task difficulty:** Separate dimensions. Witness complexity rewards sensemaking; task difficulty rewards execution. A complex witness (multi-stakeholder flooding situation) can produce simple tasks (deliver sandbags), and vice versa. Both independently affect reputation rewards for their respective roles.

**ESCO-ID skills at launch:** ID.001 (Bahasa & Communication) and ID.003 (Social Cohesion — includes gotong royong practices, community mediation). ID.002 (Local Wisdom) and ID.004 (Indonesian Craft & Heritage) added in v0.3 via community governance: any Pillar-tier user in Culture & Context can propose new ESCO-ID skills; accepted by cross-pillar vote.

### Bootstrap

**Genesis sources for Indonesian context:**

| Pillar | Primary Source | Secondary Source | Peer Challenge |
| --- | --- | --- | --- |
| Tech & Build | GitHub top 2% Indonesian contributors | Tokopedia/Gojek/Bukalapak alumni networks | 3 early-adopter vouches |
| Field & Social | PMI (Red Cross Indonesia), BNPB | Muhammadiyah/NU social service arms, PKBI | 3 early-adopter vouches |
| Logic & Audit | UI/UGM/ITB faculty (H-index ≥ 10) | AJI (journalist union), Tempo investigative | 3 early-adopter vouches |
| Culture & Context | Matan | Nahdlatul Ulama Majelis, Dikbud cultural board | 3 early-adopter vouches |

**Genesis sunset:** Rapid (10 months). Rationale: GR is designed for organic community growth. Long genesis windows create dependency on founders and violate gotong royong's community ownership principle. If genesis users are good, they'll earn organic reputation fast enough that sunset doesn't matter. If they're not contributing, they should fade.

### Safety

**Emergency Brake:** Enabled in v0.1 with high threshold: Keystone tier AND I ≥ 0.9 AND J ≥ 0.8. Penalty for abuse: 30% slash. Rationale: the brake is a safety valve against systemic attack. High threshold prevents casual use. Heavy penalty prevents censorship-by-brake. Enabled early because coordinated fraud can appear before dispute infrastructure is mature.

**Emergency fast track:** Allowed with mandatory 7-day post-hoc audit. All emergency fast-tracked transitions are flagged and reviewed by a Stochastic Jury within 7 days. If audit finds the fast-track was unjustified, endorsers take J penalty (-0.05). If justified, endorsers get J bonus (+0.02).

**Skeptical vouch minimum stake:** 0.02 × R (same as Tandang default). Minimum absolute stake: 0.05. Rationale: Tandang's default is already calibrated against griefing (mass skeptical vouching costs J per vouch). No GR-specific adjustment needed.

### Vouch Mechanics

**Co-witnessing auto-creates vouches:** YES, automatically. Rationale: co-witnessing IS staking your credibility that the witness saw something real. Opt-in would add friction and reduce trust graph density. The 14-day bleed-in provides the safety valve — vouches can be burned during latency if the witness is flagged. If a user doesn't want to stake credibility, they shouldn't co-witness.

**Minimum impact attestation count for deferred sensemaker vouches:** 3 independent attestations from beneficiaries/adopters with I > 0. This is intentionally higher than the 2-attestation minimum for general impact confirmation — generating trust graph edges (vouches) toward sensemakers requires stronger evidence than confirming impact alone. Rationale: 3 is enough to signal real impact without being so high that sensemakers never get credit. "Independent" means no two attestors share more than 30% of their vouch network (prevents coordinated attestation).

**Vouch bleed-in timeline:** Keep at 14 days (no adjustment). GR governance cycles (48-72h challenge windows) are shorter than bleed-in, which is correct — trust should build slower than governance decisions. A co-witness's vouch reaches full weight only after the challenge window has closed, which means the trust transfer is informed by whether the transition was disputed.

## Remaining Open Items

These are implementation details, not design decisions. To be resolved during build:

- Exact quorum sizes per track and stage (likely 3-5 distinct voters for small communities, scaling with domain population)
- Rate limit tuning for AI pipeline (ESCO tagging, complexity estimation) to prevent API abuse
- Specific embedding model for duplicate detection (candidate: all-MiniLM-L6-v2 for multilingual, or IndoBERT for Indonesian-first)
- Dashboard UX for weather report, vouch portfolio, governance budget display
- Mobile-first vs. web-first for witness submission (likely mobile — field witnesses need phone cameras)
- Notification system design (push notifications for weather changes, stall warnings, dispute activity)
- Data retention policy for archived witnesses (keep indefinitely for provenance, or expire after N years?)

## Appendix A: Data Primer JSON Schema (Draft)

The Data Primer is the public stub (`WitnessPublished`) created from a witness submission, especially in Confidential mode. This schema is illustrative and versioned.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "WitnessPublished",
  "type": "object",
  "required": ["witness_id", "track", "stage", "claim_summary", "claim_type", "esco_skills", "evidence"],
  "properties": {
    "witness_id": { "type": "string", "minLength": 1 },
    "version": { "type": "integer", "minimum": 1 },
    "track": { "type": "string", "enum": ["resolve", "celebrate", "explore"] },
    "stage": { "type": "string", "minLength": 1 },
    "claim_type": { "type": "string", "enum": ["good", "bad", "unknown"] },
    "claim_summary": { "type": "string", "minLength": 1, "maxLength": 2000 },
    "esco_skills": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["skill_uri", "label"],
        "properties": {
          "skill_uri": { "type": "string", "pattern": "^http://data\\.europa\\.eu/esco/" },
          "label": { "type": "string" },
          "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
        },
        "additionalProperties": false
      },
      "maxItems": 16
    },
    "esco_id_skills": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["skill_id", "label"],
        "properties": {
          "skill_id": { "type": "string", "pattern": "^ID\\." },
          "label": { "type": "string" },
          "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
        },
        "additionalProperties": false
      },
      "maxItems": 8
    },
    "pillar_affinity": {
      "type": "string",
      "enum": ["tech_build", "field_social", "logic_audit", "culture_context"]
    },
    "witness_complexity": {
      "type": "object",
      "properties": {
        "ai_estimate": { "type": "number", "minimum": 0.1, "maximum": 0.9 },
        "sensemaker_override": { "type": "number", "minimum": 0.1, "maximum": 0.9 },
        "final": { "type": "number", "minimum": 0.1, "maximum": 0.9 }
      },
      "additionalProperties": false
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
        "context_triad_status": {
          "type": "object",
          "properties": {
            "visual": { "type": "boolean" },
            "locational": { "type": "boolean" },
            "corroborative": { "type": "boolean" }
          },
          "additionalProperties": false
        },
        "context_triad_minimum": {
          "type": "object",
          "properties": {
            "required_at_stage": { "type": "string" },
            "min_of_three": { "type": "integer", "minimum": 0, "maximum": 3 }
          },
          "additionalProperties": false
        },
        "external_platform_links": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "platform": { "type": "string", "enum": ["github", "discord", "twitter", "stackoverflow"] },
              "url": { "type": "string", "format": "uri" },
              "verified": { "type": "boolean" }
            },
            "additionalProperties": false
          },
          "maxItems": 8
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

## Appendix B: Key Tandang Parameters Used by GR

| Parameter | Value | Symbol | GR Usage |
| --- | --- | --- | --- |
| PageRank Damping | 0.85 | α | Trust graph computation |
| PageRank Convergence | 1e-6 | ε | Iteration stopping criterion |
| Vouch Budget Factor | 0.3 | — | Base for governance + vouch budget |
| Vouch Latency | 14 days | — | Bleed-in for GR-generated vouches |
| Base Slash | 50% | S | Cascade penalty base |
| Cascade Decay | 0.5 | δ | Penalty reduction per hop |
| Max Cascade Depth | 3 | — | Limit on recursive penalty |
| Burn Rate | 10% | — | Slashed rep → GDF |
| GDF Half-Life | 14 days | — | Weather self-healing |
| Max GDF Impact | 20% | — | Earning reduction cap |
| Competence Half-Life | 90 days (base) | — | Overridden per contribution type |
| Genesis Monthly Decay | 10% | — | Sunset protocol |
| Verifier Liability | 1.5x | — | Enhanced penalty for verification failure |
| Genesis Fraud Multiplier | 2.0x | — | Genesis node accountability |
| Shadow J Threshold | J < 0.3 | — | Triggers Shadow Tier |
| Novice Reservation | 20% | — | On-ramp slots |
| Skeptical Vouch Weight | 0.75 | — | Reduced trust transfer |
| Skeptical Dampening Floor | 0.70 | — | Max 30% dampening |
| Skeptical Auto-Expiry | 90 days | — | Forces re-evaluation |
| Dispute Stake | 3% R (min 0.05) | — | Skin in game for challenges |
| Mentorship Tax (Auditor) | 15% | — | Recovery auditor compensation |
| Mentorship Tax (GDF Pool) | 10% | — | System healing contribution |
| Reciprocity Dampening | 0.7x | — | Anti-collusion: mutual vouches |
| Temporal Burst Dampening | 0.5x | — | Anti-collusion: 3+ vouches in 24h |
| Identity Grace Period | 30 days | — | New user full weight regardless of tier |
| Polymath Bonus | 5% I boost | — | 3+ Pillar-level domains |

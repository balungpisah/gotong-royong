# Gotong Gameplay Rules Mapped to Tandang Signals

Status: baseline ruleset (v1.1, contract-locked)  
Last updated: 2026-02-24

This document defines the deterministic gameplay behavior in Gotong that is driven by Tandang signals. It is the canonical reference for product, frontend, backend, and analytics.

## 1. Scope and Inputs

Gameplay decisions must only use these Tandang signal families:

- Reputation and tier
- Skill graph and skill relevance
- PoR requirements and PoR verification status
- Leaderboards and distribution metrics

Primary read endpoints:

- `GET /api/v1/users/{id}/reputation`
- `GET /api/v1/users/{id}/tier`
- `GET /api/v1/users/{id}/activity`
- `GET /api/v1/cv-hidup/{user_id}`
- `GET /api/v1/skills/search`
- `GET /api/v1/por/requirements/{task_type}`
- `GET /api/v1/por/triad-requirements/{track}/{transition}`
- `GET /api/v1/por/status/{evidence_id}`
- `GET /api/v1/reputation/leaderboard`
- `GET /api/v1/reputation/distribution`

## 2. Gameplay Rule Matrix (Authoritative)

| Rule ID | Trigger / User Action | Tandang Signal(s) | Condition | Gotong Outcome | UI Surface |
|---|---|---|---|---|---|
| `GR-TIER-001` | Open profile/home | Tier + reputation | Always | Show current tier badge, reputation value, trend | Profile header, dashboard hero |
| `GR-TIER-002` | Attempt to create high-impact task | Tier | Tier below configured minimum | Block action with reason and next-step hint | Task creation form |
| `GR-TIER-003` | Attempt to submit vouch | Tier | Tier below `Contributor` | Disable vouch action, explain unlock path | Contribution detail, user card |
| `GR-TIER-004` | Attempt verifier-only actions | Tier | Tier below `Pillar` | Hide or disable verifier controls | Verification queue |
| `GR-TIER-005` | Governance-sensitive operation | Tier | Tier below `Keystone` | Deny action, show governance eligibility criteria | Governance/admin screens |
| `GR-SKILL-001` | Task recommendation generation | Skills | Skill overlap exists | Rank tasks by skill relevance | Task feed |
| `GR-SKILL-002` | Contribution display | Skills + CV hidup | Verified skill exists | Show verified skill badges on contribution card | Feed cards, profile |
| `GR-SKILL-003` | New skill claim flow | Skills + vouch history | Skill not yet validated | Mark as provisional until vouch/PoR confirmation | Skills tab |
| `GR-POR-001` | User enters proof step | PoR requirements | Always | Render required evidence checklist per task type | Evidence upload screen |
| `GR-POR-002` | Transition between track phases | PoR triad requirements | Requirements unmet | Prevent phase transition and show missing items | Task detail timeline |
| `GR-POR-003` | Contribution reward calculation | PoR status | `pending` | Hold final reward, show pending state | Contribution status pill |
| `GR-POR-004` | Contribution reward calculation | PoR status | `verified` | Apply full gameplay reward + reputation dispatch | Completion summary |
| `GR-POR-005` | Contribution reward calculation | PoR status | `rejected` or `revoked` | Remove/rollback pending reward and show remediation path | Contribution detail |
| `GR-LB-001` | Open leaderboard | Leaderboard | Always | Show ranked list (scoped for platform service token) | Leaderboard screen |
| `GR-LB-002` | End of cycle reward run | Leaderboard rank + tier | User in reward bracket | Grant cycle badge/reward according to bracket | Rewards center |
| `GR-LB-003` | Anti-gaming check | Distribution + leaderboard deltas | Anomaly threshold exceeded | Flag for review before reward finalization | Admin moderation panel |
| `GR-CV-001` | Open profile CV section | CV hidup | Always | Display live CV sections from Tandang | Profile CV panel |
| `GR-CV-002` | Share profile/CV | CV hidup export + QR | Export available | Enable share links/QR | Share modal |
| `GR-ACT-001` | Open activity timeline | Activity feed | Always | Show reputation-impacting events in order | Activity tab |

## 3. Reward and Badge Rules

| Reward Rule ID | Input Signals | Logic | Output |
|---|---|---|---|
| `GR-RWD-001` | Tier + verified contribution | Base reward multiplied by tier policy | Credits + progress XP |
| `GR-RWD-002` | PoR status | Only `verified` finalizes reward | Final reward commit |
| `GR-RWD-003` | Leaderboard rank at cycle close | Top-N brackets map to badge tiers | Cycle badge grant |
| `GR-RWD-004` | Skill validation milestone | New verified skill count threshold crossed | Skills milestone badge |
| `GR-RWD-005` | Reputation streak from activity | Consecutive verified-impact days threshold | Streak badge / bonus |

## 4. UI Surface Mapping (No Ambiguity)

| UI Surface | Must Read | Rule IDs |
|---|---|---|
| Profile header | reputation, tier | `GR-TIER-001` |
| Task creation form | tier, skills | `GR-TIER-002`, `GR-SKILL-001` |
| Evidence upload | PoR requirements | `GR-POR-001` |
| Task transition timeline | PoR triad requirements, PoR status | `GR-POR-002`, `GR-POR-003`, `GR-POR-004`, `GR-POR-005` |
| Contribution card/detail | skills, PoR status | `GR-SKILL-002`, `GR-POR-003..005` |
| Verification queue | tier | `GR-TIER-004` |
| Governance screens | tier | `GR-TIER-005` |
| Leaderboard screen | leaderboard, distribution | `GR-LB-001`, `GR-LB-003` |
| Rewards center | leaderboard rank, tier, skills milestones | `GR-LB-002`, `GR-RWD-001..005` |
| CV panel/share modal | cv hidup + export/qr | `GR-CV-001`, `GR-CV-002` |
| Activity tab | activity | `GR-ACT-001` |

## 5. Runtime Policy Notes

- Linking is transparent to end users; gameplay rules must never require manual account linking.
- Tandang service-token reads are platform-scoped and must be treated as authoritative.
- If Tandang read fails, Gotong should show stale marker/fallback state and avoid destructive gameplay state changes.
- Write-side rewards are finalized only after successful webhook processing + PoR verification state.
- Formula shape is locked by Rule ID (`GR-*`): triggers, signal families, and decision branches must not be changed by ad-hoc tuning.
- Numeric balancing values are tunable, but only through versioned rule config (never hardcoded one-off edits).

## 6. Versioning and Change Control

- Breaking rule changes require incrementing this document version and updating:
  - `docs/architecture/tandang-full-integration.md`
  - `docs/architecture/tandang-signal-mapping.md`
  - Frontend behavior spec references
- Any rule added must include:
  - Rule ID
  - Trigger
  - Exact signal input
  - Deterministic condition
  - UI surface impact
- Balancing changes must be deployed as config versions with explicit activation timestamp and rollback target.

## 7. Rule Config Governance (Mandatory)

Rule behavior is split into:
- **Logic contract (locked):** condition tree + outputs per Rule ID.
- **Parameter set (tunable):** weights, thresholds, caps, decay windows, and reward multipliers.

Required config envelope for every balancing change:

| Field | Requirement |
|---|---|
| `rule_version` | Monotonic version string; immutable after publish |
| `effective_at_ms` | Future activation timestamp (epoch ms) |
| `params` | Explicit key/value set used by deterministic logic |
| `bounds` | Per-parameter min/max guardrails enforced at load time |
| `rollback_to` | Prior `rule_version` to restore if release is unhealthy |
| `change_reason` | Human-readable reason for audit and postmortem use |

Operational rules:
- No runtime tuning outside versioned config.
- New parameters require default + bounds before activation.
- Old versions stay queryable for audit/replay.
- Rollback must be single-step and deterministic.

## 8. Frontend/Backend Execution Priority

Until all hot paths meet SLOs, implementation order is fixed:
1. Chat fast path (`send`, `catch-up`, `stream`).
2. Feed + trust surfaces (relevance, gating, profile trust widgets).
3. Notifications polish and digest tuning.

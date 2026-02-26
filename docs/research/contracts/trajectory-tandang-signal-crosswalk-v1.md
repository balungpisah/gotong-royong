# Trajectory × Tandang Signal Crosswalk (v1)

Last updated: 2026-02-26
Status: Active reference (v1 defaults locked)

## Purpose

Define how `TrajectoryType` (intent semantics) should be cross-referenced with Tandang-facing signal patterns so we can:
- improve triage quality and operator routing,
- detect unmodeled behavior (taxonomy gaps),
- evolve taxonomy safely using evidence instead of guesswork.

This preserves separation of concerns:
- `trajectory` = what the story is
- `signal` = how trusted/contested/verified the story is

## Canonical Inputs

- Trajectory taxonomy (11): `apps/web/src/lib/types/card-enrichment.ts`
- Feed signal model: `apps/web/src/lib/types/feed.ts`
- I/C/J semantics: `docs/design/specs/ui-ux-spec/14-reputation-ui-contract.md`
- Tandang signal inventory: `docs/design/specs/tandang/TANDANG-SIGNAL-INVENTORY-v0.1.md`

## Signal Families (runtime today)

- `vouch_positive` (person/content trust)
- `vouch_skeptical` (healthy doubt / reservation)
- `saksi` (PoR witness attestation)
- `perlu_dicek` (uncertainty/verification flag)
- `inline_vote_yes/no` (proposal decision context)
- `dukung` (social support, non-Tandang scoring lane)

## Crosswalk Matrix (v1)

| Trajectory | Feed Kind | Primary expected signals | Secondary expected signals | Suspicious / anomaly pattern |
|---|---|---|---|---|
| `aksi` | witness | `saksi`, `vouch_positive` | `perlu_dicek` | high `perlu_dicek` with no `saksi` over time |
| `advokasi` | witness | `vouch_positive`, `vouch_skeptical` | `saksi` | polarized vouch only, zero evidence growth |
| `pantau` | witness | `perlu_dicek`, `saksi` | `vouch_skeptical` | high confidence claim, zero verification actions |
| `mufakat` | witness | `inline_vote_yes/no`, `stempel_state` | `vouch_positive` | repeated voting without quorum/evidence changes |
| `mediasi` | witness | `vouch_skeptical`, `vouch_positive` | `inline_vote_yes/no` | one-sided signaling, no counter-signal |
| `program` | witness | `vouch_positive`, `saksi` | `inline_vote_yes/no` | many claims, no completion attestations |
| `data` | data | `saksi` | `perlu_dicek` | high controversy, no source/proof updates |
| `vault` | data | `saksi` (private/controlled) | none | public signal mismatch with privacy expectation |
| `bantuan` | data | `vouch_positive` | `saksi` | repeated help requests with no resolution transitions |
| `pencapaian` | data | `saksi`, `vouch_positive` | `dukung` | celebration claims without upstream witness linkage |
| `siaga` | data | `saksi`, `perlu_dicek` | `vouch_skeptical` | strong alert with zero confirms and rising denies |

## Known Gaps (candidate taxonomy/signal expansion)

1. `data` trajectories lack a dedicated “source quality” signal family (currently overloaded into `saksi/perlu_dicek`).
2. `bantuan` lacks explicit “match quality / assistance fulfilled” signal lane.
3. `pencapaian` lacks explicit “impact verified” signal type (currently inferred indirectly).
4. `mufakat/mediasi` now use `stempel_state` lifecycle (`draft -> proposed -> objection_window -> locked`) as consensus-integrity signal. Monitor for quality, not just adoption.

These should be tracked as evidence-backed candidates, not immediate taxonomy explosion.

## Gap Detection Contract (proposed)

Emit analytics event when trajectory-signal behavior falls outside expected envelope:

```json
{
  "event": "trajectory_signal_gap_detected",
  "schema_version": "trajectory-signal-gap.v1",
  "trajectory_type": "pantau",
  "entity_id": "witness-123",
  "window_days": 14,
  "signal_snapshot": {
    "vouch_positive": 1,
    "vouch_skeptical": 9,
    "witness_count": 0,
    "flags": 12
  },
  "gap_code": "high_flag_low_evidence",
  "suggested_action": "triage_followup_needed"
}
```

`gap_code` (v1):
- `high_flag_low_evidence`
- `high_claim_low_verification`
- `vote_without_context_growth`
- `celebration_without_upstream_link`
- `privacy_signal_mismatch`

## Rollout Plan

1. Introduce read-model query that computes signal snapshots per trajectory.
2. Run passive logging for 2-4 weeks (`observe-only`, no UX behavior changes).
3. Review top `gap_code` frequencies weekly.
4. Promote only high-frequency + actionably distinct patterns into:
   - new trajectory subtype, or
   - new signal family, or
   - operator follow-up question policy.

## Locked defaults (implemented)

- `stempel_state` governance baseline:
  - min participants = `3`
  - objection window = `24h`
  - lock only with no objection after window
- terminal impact verification opens when stempel locks (`impact_verification.status = open`)

## Guardrails

- Do not merge trajectory and signal into one taxonomy.
- Do not auto-create new trajectory based on single incidents.
- Keep backward compatibility with current `FeedItem.signal_counts` and `MyRelation` types.

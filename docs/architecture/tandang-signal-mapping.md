# Gotong to Tandang Signal Mapping (Exhaustive Draft)

This document enumerates every Gotong Royong user action and system action and maps it to either a Tandang webhook event or a deliberate "no event" decision. It is the single checklist to ensure we never miss a signal.

Status legend: `existing` means already supported in `docs/api/event-payloads.md`. `proposed` means new event type or payload extension. `no_event` means local-only, no Tandang update.

## Event Envelope (All Webhooks)

| Field | Required | Notes |
|---|---|---|
| `event_id` | Yes | Idempotency key for dedupe. Required for all events. |
| `event_type` | Yes | Event name (see tables below). |
| `schema_version` | Yes | Positive integer (or numeric string). Current baseline is `1`. |
| `request_id` | Yes | End-to-end trace ID; must match `X-Request-ID` header. |
| `timestamp` | No | RFC3339 event creation time. |
| `actor.user_id` | Yes | Gotong user ID. |
| `actor.username` | Yes | Gotong display name. |
| `subject` | Yes | Event-specific payload. |

## A. Identity and Account Lifecycle

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| User signs up | no_event | At account creation | none | Auto-provision occurs on first valid webhook | no_event |
| User updates profile | no_event | On profile save | none | No rep change | no_event |
| User deactivates account | account_disabled | On deactivation | `reason` | Freeze updates, hide CV | proposed |
| User reactivates account | account_reenabled | On reactivation | `reason` | Resume updates | proposed |
| User merges accounts | account_merged | On merge complete | `from_user_id`, `to_user_id` | Merge reputations | proposed |

## B. Task Lifecycle

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Task created | no_event | On task create | none | No rep change | no_event |
| Task updated | no_event | On task update | none | No rep change | no_event |
| Task assigned | no_event | On assignment | none | No rep change | no_event |
| Task started | no_event | On start | none | No rep change | no_event |
| Task completed | contribution_created | On completion submit | `contribution_type`, `title`, `task_id` | Reputation increase | existing |
| Task cancelled | contribution_voided | On cancel with prior contribution | `task_id`, `reason` | Reverse rep | proposed |
| Task deleted | no_event | On delete | none | No rep change | no_event |

## C. Contribution Lifecycle

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Contribution submitted | contribution_created | On submit | `contribution_type`, `title`, `description`, `evidence_url`, `skill_ids`, `metadata` | Reputation update | existing |
| Contribution edited | contribution_updated | On edit save | `task_id`, `changes` | Recalculate rep if needed | proposed |
| Contribution voided | contribution_voided | On admin void | `task_id`, `reason` | Reverse rep | proposed |
| Contribution restored | contribution_restored | On restore | `task_id`, `reason` | Re-apply rep | proposed |
| Contribution rescored | contribution_rescored | On recalculation | `task_id`, `previous_score`, `new_score` | Adjust rep | proposed |

## D. PoR Evidence Lifecycle

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Evidence submitted | por_evidence | On evidence upload | `contribution_id`, `evidence_type`, `evidence_data`, `status=submitted` | Start PoR validation | existing |
| Evidence updated | por_evidence | On evidence edit | `contribution_id`, `evidence_type`, `evidence_data`, `status=updated` | Re-validate | proposed |
| Evidence verified | por_evidence | On verifier approval | `contribution_id`, `status=verified`, `verifier_id`, `verifier_notes` | Unlock higher rep | proposed |
| Evidence rejected | por_evidence | On verifier rejection | `contribution_id`, `status=rejected`, `rejection_reason` | Reduce or block rep | proposed |
| Evidence revoked | por_evidence | On revocation | `contribution_id`, `status=revoked`, `reason` | Reverse rep | proposed |

## E. Vouching and Endorsements

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| User vouches for another | vouch_submitted | On vouch submit | `target_user_id`, `skill_id`, `message`, `confidence` | Skills and trust signals | existing |
| Vouch revoked | vouch_revoked | On revoke | `vouch_id`, `reason` | Remove skill credit | proposed |
| Vouch expired | vouch_revoked | On expiry | `vouch_id`, `reason=expired` | Remove skill credit | proposed |

## F. Moderation and Disputes

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Contribution flagged | contribution_flagged | On flag | `task_id`, `reason` | May pause rep | proposed |
| Evidence flagged | por_evidence_flagged | On flag | `contribution_id`, `reason` | May pause rep | proposed |
| Dispute opened | dispute_opened | On dispute create | `target_id`, `reason` | Freeze rep until resolved | proposed |
| Dispute resolved | dispute_resolved | On resolve | `target_id`, `resolution`, `delta` | Apply adjustment | proposed |
| Penalty applied | reputation_adjusted | On admin action | `reason`, `delta` | Decrease rep | proposed |
| Penalty removed | reputation_adjusted | On reversal | `reason`, `delta` | Increase rep | proposed |

## G. Rewards, Badges, and Achievements

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Badge awarded | badge_awarded | On award | `badge_id`, `reason` | Optional CV enrichment | proposed |
| Badge revoked | badge_revoked | On revoke | `badge_id`, `reason` | Remove CV entry | proposed |
| Milestone reached | milestone_reached | On threshold | `milestone_id`, `metric` | Optional CV enrichment | proposed |

## H. Backfill and System Events

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Historical backfill | contribution_created | During import | Full contribution payload | Build history | existing |
| Replay from DLQ | original event | On replay | Original payload | Idempotent | existing |
| Platform recalculation | contribution_rescored | On batch recompute | `task_id`, `previous_score`, `new_score` | Adjust rep | proposed |

## I. Local-Only Actions (Explicitly No Event)

| Gotong action | Event type | Trigger timing | Required subject fields | Tandang effect | Status |
|---|---|---|---|---|---|
| Login, logout | no_event | On auth | none | No rep change | no_event |
| Notification preferences | no_event | On save | none | No rep change | no_event |
| UI theme, language | no_event | On save | none | No rep change | no_event |
| Draft creation | no_event | On save | none | No rep change | no_event |

## Read Mapping (Gotong UI -> Tandang Read APIs)

| Gotong UI surface | Tandang endpoint(s) | Purpose |
|---|---|---|
| Profile header | `/api/v1/users/{id}/reputation`, `/api/v1/users/{id}/tier` | Reputation and tier badge |
| Activity feed | `/api/v1/users/{id}/activity` | Timeline of contributions and vouches |
| Live CV panel | `/api/v1/cv-hidup/{user_id}` | CV data |
| Share CV | `/api/v1/cv-hidup/{user_id}/export`, `/api/v1/cv-hidup/{user_id}/qr` | Shareable CV |
| Skills tab | `/api/v1/skills/search`, `/api/v1/skills/nodes` | Skills and graph |
| PoR guidance | `/api/v1/por/requirements`, `/api/v1/por/triad-requirements` | Evidence guidance |
| PoR status | `/api/v1/por/status?user_id=...` | Verification status |
| Leaderboards | `/api/v1/reputation/leaderboard` | Rankings |
| Distribution | `/api/v1/reputation/distribution` | Global reputation curve |

## Implementation Notes

All proposed event types should be added to `docs/api/event-payloads.md` and to the Markov adapter once approved. If a Gotong action maps to `no_event`, that is a deliberate decision and should not emit webhooks.

Gameplay outcomes driven by these signals are defined in `docs/architecture/tandang-gameplay-rules.md`.

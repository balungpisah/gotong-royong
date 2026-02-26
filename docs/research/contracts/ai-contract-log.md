# AI Contract Log (Research Phase 3)

Track AI features, contracts, versioning, and fallback behavior before backend integration.

Canonical endpoint registry:
- `docs/research/contracts/ai-endpoint-map-v1.md`

Notes:
- use `ai-endpoint-map-v1.md` as system-of-record for endpoint-level runtime contracts and gate behavior.
- keep this file as feature-level AI touchpoint traceability and planning log.

## AI contract table

| Feature ID | AI touchpoint | Inputs expected | Output contract fields | Version/Prompt source | Fallback strategy | Status |
|---|---|---|---|---|---|---|
| UI-01 | AI-00 — triage (primary) | `text`, `media_urls`, `location`, `conversation_history`, `user_id`, `session_id`, `privacy_signals` | `entry_flow`, `track`, `confidence`, `seed_type`, `context_bar_state`, `esco_skills`, `reasoning`, `follow_up_question`, `is_split_candidate`, `split_flows`, `conversation_summary` | `14-prompt-versioning.md` → `TRIAGE-001` (`v0.2.0`) | `Manual` state path if model unavailable; keep context bar and continue with empty classification | PARTIAL (output contract present, endpoint contract pending) |
| UI-01 | AI-02/Sensitive fallback (auxiliary) | `text`, `text_format`, `user_id`, `context` | `redacted_text`, `redaction_count`, `redacted_items[]`, `needs_manual_review`, `confidence` | `14-prompt-versioning.md` → `REDACT-001` (`v0.2.0`) | proceed with original text and force manual review if redaction model unavailable | PARTIAL |
| UI-01 | AI-03 — duplicate detection (pre-submission) | `query_text`, `query_embedding`, `location`, `radius_km`, `exclude_seed_ids` | `matches[]`, `top_match`, `confidence_level`, `recommendation` | `07-ai-03-duplicate-detection.md` + `14-prompt-versioning.md` (`DUPLICATE-001`) | proceed without duplicate block and enqueue async retry | PARTIAL |
| UI-02 | AI-01 — classification | `text`, `location`, `seed_candidate_id`, `conversation_context`, `metadata` | `track`, `track_confidence`, `seed_type`, `seed_confidence`, `esco_skills[]`, `reasoning`, `is_ambiguous`, `alternative_track`, `alternative_confidence` | `14-prompt-versioning.md` → `CLASS-001` (`v0.2.0`) | return `null` and trigger manual manual selection flow | PARTIAL |
| UI-03 | AI-06 — criteria + task decomposition | `seed_id`, `track`, `seed_text`, `discussion_summary`, `community_context` | `suggested_criteria[]`, `criteria_confidence`, `suggested_tasks[]`, `task_decomposition_confidence`, `user_agency_note` | `14-prompt-versioning.md` → `CRITERIA-001` (`v0.2.0`) | show empty suggestions and require manual task input | UNKNOWN |
| UI-04 | AI-08 — sensitive media scan | `media_urls`, `media_types`, `seed_id`, `author_id`, `seed_text` | `scans[]` with `detections`, `overall_safety`, `redacted_media_url`, `summary` | `12-ai-08-sensitive-media.md` + `14-prompt-versioning.md` (`SENSITIVE-001`) | continue with original media, escalate to manual moderation queue | PARTIAL |
| UI-04 | AI-02 — redaction | same as AI-02 | redacted text fields + risk flags | `REDACT-001` (`v0.2.0`) | manual moderation path only if redaction API fails | PARTIAL |
| UI-05 | AI-05 — gaming detection | `lookback_hours`, `seed_ids`, `user_ids`, `focus_metric` | `flags[]`, `summary.total_flags`, `summary.critical_count` | `09-ai-05-gaming-detection.md` + `14-prompt-versioning.md` (`GAMING-001`) | non-blocking unless same-pattern critical threshold repeats | PARTIAL |
| UI-06 | AI-09 — credit recommendation and review | lifecycle actions, audit stream snapshots | `user_id`, `candidate_allocations`, `confidence`, `reasoning`, `dispute_window` | `13-ai-09-credit-accreditation.md` → `CREDIT-001` (`v0.2.0`) | show empty credit form with manual totals at Tuntas | PARTIAL |
| UI-07 | AI-04 — moderation | `seed_id`, `text`, `attachments`, `author_id`, `author_reputation`, `track`, `seed_type`, `location` | `status`, `confidence`, `violations[]`, `action`, `moderation_hold_duration_minutes`, `auto_release_if_no_action`, `reasoning` | `14-prompt-versioning.md` → `MOD-001` (`v0.2.0`) | hold for moderator manually and hide content until manual decision | PARTIAL |
| UI-08 | AI-00 + AI-04 fallback bridge | crisis signals from `text`/`location`, emergency keywords, policy check | emergency flag, siaga payload, emergency follow-up hints, moderation override state | `TRIAGE-001` (`v0.2.0`) + `MOD-001` (`v0.2.0`) | zero-mod path: immediate broadcast with manual review in moderator queue | UNKNOWN |
| UI-09 | AI-00 — vault routing | story, location, privacy hints, urgency + safety signals | `entry_flow=vault`, `context_bar_state=vault-ready`, `reasoning` | `TRIAGE-001` (`v0.2.0`) | allow user to force public flow with explicit warning if pattern confidence is borderline | PARTIAL |
| UI-10 | AI for feed ranking/search (TBD) | query text, scope, filters, user reputation band | optional ranking boost fields, blocked/hidden result flags | not yet defined in docs | fallback to deterministic lexical + timestamp ranking | UNKNOWN |
| UI-11 | AI-07 — summary snippets in search/digest | `messages`, `summary_style`, `max_length_words` | `summary`, `key_points[]`, `sentiment`, `action_items[]`, `controversies[]` | `11-ai-07-discussion-summarization.md` → `SUM-001` (`v0.2.0`) | show latest raw messages when <5 messages or summarizer unavailable | PARTIAL |
| UI-12 | AI-07 — notification digest | same as UI-11 + period window | digest text and sentiment clusters | `SUM-001` (`v0.2.0`) + internal batch schedule | skip digest generation and show raw event list | UNKNOWN |
| UI-13 | AI skill suggestion hooks | `text`, `seed metadata` (if provided) | `esco_skills` proposals + confidence | extraction handled by Tandang extractors (not in AI spec prompt table) | use rule-based/manual fallback if extraction returns empty | UNKNOWN |
| UI-14 | AI assist on onboarding (none) | N/A | N/A | N/A | no AI contract specified in v0.2 | N/A |
| UI-15 | AI assist on invite/share (none) | N/A | N/A | N/A | no AI contract specified in v0.2 | N/A |
| UI-16 | AI suggestion in recurrent scheduling (none) | N/A | N/A | N/A | none | N/A |
| UI-17 | AI moderation of governance proposals (future) | optional discussion text | optional risk/clarity scores (TBD) | N/A | default to manual governance process | N/A |
| UI-18 | AI role conflict risk scoring (future) | N/A | N/A | N/A | no AI dependency required for baseline | N/A |
| UI-19 | AI-05 optional signals on dispute patterns | reports, user ids, focus metric inputs | flags + recommendation | `09-ai-05-gaming-detection.md` + `14-prompt-versioning.md` (`GAMING-001`) | keep dispute as manual flow if flags unavailable | PARTIAL |
| UI-20 | AI-06 source-tag suggestion (optional) | patch blocks, context state | `suggested_tasks`, `user_agency_note`, patch confidence | `10-ai-06-criteria-suggestions.md` (scope extension) | skip AI suggestions; allow manual edit-only mode | UNKNOWN |
| UI-21 | AI not used in Galang ledger | N/A | N/A | N/A | no AI in base flow | N/A |
| UI-22 | AI not used in Siarkan amplify | N/A | N/A | N/A | no AI in base flow | N/A |
| UI-23 | AI-03 + AI-05 duplicate review | `query_text`, `query_embedding`, location + context metadata | `matches`, `top_match`, `recommendation` | `07-ai-03-duplicate-detection.md` + `14-prompt-versioning.md` (`DUPLICATE-001`) | merge can be skipped; continue user submission with warning log | PARTIAL |

## Edge-Pod conventions to enforce during research

1. Prompt IDs and versions must be referenced on every AI workflow.
2. AI outputs must be JSON schema-validated before persistence.
3. All AI outcomes must include `confidence`, `reason_code`, `timestamp`, and `actor_context`.
4. Serial processing for AI stateful interactions (AI-00/01 chain) must preserve conversation order.

## Open AI gaps to resolve

- No prompt-registration blockers remain for this phase.
- Confirm advisory vs system-of-record semantics in implementation stories for each endpoint (`EP-03`, `EP-05`, `EP-08`, `EP-09`).
- Confirm deterministic fallback observability metrics (fallback counters and reason codes) in service SLOs.
- Confirm per-flow persistence targets (audit-only vs persisted transition) during implementation.

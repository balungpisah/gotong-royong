# Feed Card Blocks Contract (v1)

Last updated: 2026-02-26
Owner: frontend + API cutover slice
Status: active reference

## Purpose

Define a single source of truth for feed card blocks currently rendered in the app, so:
- DB seed payloads are intentionally varied,
- operator outputs can target stable card structure,
- taxonomy/trajectory profiles map to explicit block presence.

This version documents current runtime behavior (as-built), not a future redesign.

## 1) Witness Card Block Inventory (kind=`witness`)

Canonical block IDs in render order:

1. `cover_media`
- Data: `cover_url`
- Visible when `cover_url` exists.

2. `seed_badge`
- Data: `dev_meta.is_seed`
- Visible in dev mode only.

3. `urgency_badge`
- Data: `urgency`
- Visible when urgency exists.

4. `presence_chip`
- Data: `active_now`
- Visible when `active_now > 0`.

5. `event_emoji`
- Data: `latest_event.event_type`
- Always visible.

6. `countdown_strip`
- Data: `deadline`, `deadline_label`
- Visible when deadline exists.

7. `quorum_progress`
- Data: `quorum_target`, `quorum_current`
- Visible when both values exist.

8. `repost_attribution`
- Data: `repost`
- Visible when repost frame exists.

9. `headline`
- Data: `hook_line`, `title`
- Always visible (title fallback).

10. `event_meta`
- Data: `latest_event.verb`, `latest_event.timestamp`
- Always visible.

11. `body_narrative`
- Data: `body` or fallback `latest_event.snippet`
- Visible when one exists.

12. `story_peek`
- Data: `peek_messages[]`
- Visible when at least 2 messages exist.

13. `entity_tags_row`
- Data: `entity_tags[]`
- Visible when non-empty.

14. `signal_chip_bar`
- Data: `signal_counts`, `my_relation`, `signal_labels`
- Visible when relation or counts are available.

15. `resolution_badge`
- Data: terminal status + resolved signals
- Visible on terminal witnesses with at least one resolution.

16. `member_footer`
- Data: `members_preview[]`, `member_count`
- Always visible (count), avatar stack when preview exists.

17. `action_cluster`
- Actions: `dukung`, `pantau`, `simpan`, `bagikan`
- Always visible.

## 1.1 Block Sources (origin of data)

Each block is derived from one of three sources:
- `chat-derived` = LLM summary/extraction from discussion/chat.
- `system-derived` = backend/system state or Tandang signals.
- `user-action` = explicit user action in the feed (support/monitor/share).

Block → source:
- `cover_media`: `system-derived` (uploaded/linked evidence).
- `seed_badge`: `system-derived` (dev_meta).
- `urgency_badge`: `system-derived` (backend rules/flags).
- `presence_chip`: `system-derived` (activity snapshot).
- `event_emoji`: `system-derived` (latest_event.event_type).
- `countdown_strip`: `system-derived` (deadline + label).
- `quorum_progress`: `system-derived` (quorum counters).
- `repost_attribution`: `system-derived` (repost frame).
- `headline`: `chat-derived` (`hook_line` or `title`).
- `event_meta`: `system-derived` (latest_event).
- `body_narrative`: `chat-derived` (`body` or snippet fallback).
- `story_peek`: `chat-derived` (recent messages/peep window).
- `entity_tags_row`: `chat-derived` (AI suggestions) + `system-derived` (user-confirmed tags).
- `signal_chip_bar`: `system-derived` (Tandang counts/relations) + `chat-derived` (signal_labels).
- `resolution_badge`: `system-derived` (terminal status + resolved signals).
- `member_footer`: `system-derived` (member preview/count).
- `action_cluster`: `user-action` (local state + request).

## 2) System Card Blocks (kind=`system`)

Shared blocks:
- `sys_header` (`icon`, `title`, optional dismiss action)
- `sys_description`

Variant blocks:
- `sys_suggestion_entities` for `payload.variant=suggestion`
- `sys_tip_link` for `payload.variant=tip`
- `sys_milestone_metric` for `payload.variant=milestone`
- `sys_prompt_cta` for `payload.variant=prompt`

## 3) Data Stream Blocks (kind=`data`)

Current runtime maps `kind=data` into a system prompt card (`prompt` variant):
- `sys_header`
- `sys_description` (uses `claim`)
- `sys_prompt_cta` (open data detail action)

Note: dedicated `data` card renderer is not active yet; this contract reflects current behavior.

## 4) Trajectory/Taxonomy Block Profiles (v1 baseline)

Profiles indicate emphasis, not strict hard validation.
Trajectory types follow `apps/web/src/lib/types/card-enrichment.ts`:
`aksi | advokasi | pantau | mufakat | mediasi | program | data | vault | bantuan | pencapaian | siaga`.

### 4.1 Core profile (`aksi`, `advokasi`, `mediasi`, `program`, `bantuan`)
- Required: `headline`, `event_meta`, `action_cluster`
- Preferred: `body_narrative`, `entity_tags_row`, `signal_chip_bar`, `member_footer`
- Optional: `cover_media`, `story_peek`, `countdown_strip`, `quorum_progress`

### 4.2 Mufakat profile (`mufakat`)
- Required: `headline`, `event_meta`, `countdown_strip`, `quorum_progress`, `action_cluster`
- Preferred: `member_footer`, `signal_chip_bar`
- Optional: `cover_media`, `story_peek`, `entity_tags_row`

### 4.3 Pantau/Data profile (`pantau`, `data`)
- Required: `headline`, `event_meta`, `entity_tags_row`
- Preferred: `body_narrative`, `signal_chip_bar`
- Optional: `story_peek`, `countdown_strip`, `quorum_progress`

### 4.4 Siaga profile (`siaga`)
- Required: `headline`, `event_meta`, `urgency_badge`, `action_cluster`
- Preferred: `cover_media`, `body_narrative`, `presence_chip`
- Optional: `story_peek`, `member_footer`

### 4.5 Pencapaian profile (`pencapaian`)
- Required: `headline`, `event_meta`, `resolution_badge`, `member_footer`
- Preferred: `cover_media`, `body_narrative`, `signal_chip_bar`
- Optional: `story_peek`, `repost_attribution`

### 4.6 Vault profile (`vault`)
- Current feed behavior follows core witness blocks when emitted into feed.
- Privacy-specific redaction is out of scope for this v1 block inventory.

### 4.7 Bantuan profile (`bantuan`)
- Required: `headline`, `event_meta`, `action_cluster`
- Preferred: `body_narrative`, `signal_chip_bar`, `member_footer`
- Optional: `cover_media`, `story_peek`, `urgency_badge`

## 5) Seed Generation Guidance

To make seeded cards visually varied and taxonomy-representative:
- each trajectory profile should have at least 2 seed rows,
- one row should include all preferred blocks,
- one row should intentionally omit optional blocks,
- seed rows should set `payload.dev_meta` using `feed-seed-metadata-v1.md`.

## 6) Cross References

- `docs/research/contracts/feed-seed-metadata-v1.md`
- `docs/research/contracts/triage-witness-feed-contract-v1.md`
- `docs/research/contracts/chat-interaction-blocks-v1.md`
- `apps/web/src/lib/components/pulse/feed-event-card.svelte`
- `apps/web/src/lib/components/pulse/feed-system-card.svelte`
- `apps/web/src/lib/components/pulse/feed-stream-registry.ts`

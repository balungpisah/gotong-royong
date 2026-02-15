-- 0009_audit_retention_fields_check
INFO FOR TABLE track_state_transition;
INFO FOR TABLE vault_entry;
INFO FOR TABLE vault_timeline_event;
INFO FOR TABLE siaga_broadcast;
INFO FOR TABLE siaga_timeline_event;
INFO FOR TABLE content_moderation;
INFO FOR TABLE moderation_decision;

SELECT
  count() AS transition_rows_total,
  count(event_hash) AS transition_rows_with_hash,
  count(retention_tag) AS transition_rows_with_retention_tag,
  (count() - count(event_hash)) AS transition_rows_missing_event_hash,
  (count() - count(retention_tag)) AS transition_rows_missing_retention_tag
FROM track_state_transition
;

SELECT
  count() AS vault_entry_count,
  count(event_hash) AS vault_entry_rows_with_hash,
  count(retention_tag) AS vault_entry_rows_with_retention_tag,
  (count() - count(event_hash)) AS vault_entry_rows_missing_event_hash,
  (count() - count(retention_tag)) AS vault_entry_rows_missing_retention_tag
FROM vault_entry
;

SELECT
  count() AS vault_timeline_event_count,
  count(event_hash) AS vault_timeline_rows_with_hash,
  count(retention_tag) AS vault_timeline_rows_with_retention_tag,
  (count() - count(event_hash)) AS vault_timeline_rows_missing_event_hash,
  (count() - count(retention_tag)) AS vault_timeline_rows_missing_retention_tag
FROM vault_timeline_event
;

SELECT
  count() AS siaga_broadcast_count,
  count(event_hash) AS siaga_broadcast_rows_with_hash,
  count(retention_tag) AS siaga_broadcast_rows_with_retention_tag,
  (count() - count(event_hash)) AS siaga_broadcast_rows_missing_event_hash,
  (count() - count(retention_tag)) AS siaga_broadcast_rows_missing_retention_tag
FROM siaga_broadcast
;

SELECT
  count() AS siaga_timeline_event_count,
  count(event_hash) AS siaga_timeline_rows_with_hash,
  count(retention_tag) AS siaga_timeline_rows_with_retention_tag,
  (count() - count(event_hash)) AS siaga_timeline_rows_missing_event_hash,
  (count() - count(retention_tag)) AS siaga_timeline_rows_missing_retention_tag
FROM siaga_timeline_event
;

SELECT
  count() AS content_moderation_count,
  count(event_hash) AS content_rows_with_hash,
  count(retention_tag) AS content_rows_with_retention_tag,
  (count() - count(event_hash)) AS content_rows_missing_event_hash,
  (count() - count(retention_tag)) AS content_rows_missing_retention_tag
FROM content_moderation
;

SELECT
  count() AS moderation_decision_count,
  count(event_hash) AS moderation_rows_with_hash,
  count(retention_tag) AS moderation_rows_with_retention_tag,
  (count() - count(event_hash)) AS moderation_rows_missing_event_hash,
  (count() - count(retention_tag)) AS moderation_rows_missing_retention_tag
FROM moderation_decision
;

-- 0026_hot_path_pack_b_indexes
-- Performance hardening index pack for hot-path feed/notification queries.
-- Preconditions: 0001-0025 applied

DEFINE INDEX idx_notification_unread
ON TABLE discovery_notification FIELDS user_id, read_at, created_at, notification_id;

DEFINE INDEX idx_feed_privacy_latest
ON TABLE discovery_feed_item FIELDS privacy_level, occurred_at, feed_id;

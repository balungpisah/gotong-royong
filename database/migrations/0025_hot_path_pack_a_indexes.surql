-- 0025_hot_path_pack_a_indexes
-- Must-have index pack for hot-path chat/feed/notification stability.
-- Preconditions: 0001-0024 applied

DEFINE INDEX uniq_chat_thread_id
ON TABLE chat_thread FIELDS thread_id UNIQUE;

DEFINE INDEX idx_chat_thread_scope_latest
ON TABLE chat_thread FIELDS scope_id, created_at, thread_id;

DEFINE INDEX idx_chat_member_thread_active
ON TABLE chat_member FIELDS thread_id, left_at, user_id;

DEFINE INDEX idx_chat_member_user_active
ON TABLE chat_member FIELDS user_id, left_at, thread_id;

DEFINE INDEX uniq_chat_read_cursor_key
ON TABLE chat_read_cursor FIELDS thread_id, user_id UNIQUE;

DEFINE INDEX uniq_feed_id
ON TABLE discovery_feed_item FIELDS feed_id UNIQUE;

DEFINE INDEX uniq_notification_id
ON TABLE discovery_notification FIELDS notification_id UNIQUE;

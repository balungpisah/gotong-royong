-- 0002_chat_indexes

DEFINE INDEX idx_message_order
ON TABLE chat_message FIELDS thread_id, created_at, message_id;

DEFINE INDEX idx_member_lookup
ON TABLE chat_member FIELDS user_id, thread_id;

DEFINE INDEX idx_read_cursor_lookup
ON TABLE chat_read_cursor FIELDS user_id, thread_id;

DEFINE INDEX uniq_delivery_request
ON TABLE chat_delivery_event FIELDS thread_id, request_id UNIQUE;

DEFINE INDEX uniq_transition_request
ON TABLE track_state_transition FIELDS entity_id, request_id UNIQUE;

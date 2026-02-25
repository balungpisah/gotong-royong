-- 0028_chat_message_request_index_order
-- Reorder chat-message idempotency index fields to avoid planner bias on catch-up reads.
-- Preconditions: 0001-0027 applied

REMOVE INDEX uniq_message_request ON TABLE chat_message;

DEFINE INDEX uniq_message_request
ON TABLE chat_message FIELDS request_id, thread_id UNIQUE;

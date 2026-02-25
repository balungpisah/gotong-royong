-- 0028_chat_message_request_index_order_check
-- Verify chat message idempotency index reorder is applied.

INFO FOR TABLE chat_message;

SELECT message_id, request_id
FROM chat_message
WHERE thread_id = "probe-thread" AND request_id = "probe-request"
LIMIT 1
EXPLAIN FULL;

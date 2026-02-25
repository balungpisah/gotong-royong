-- 0030_webhook_payload_flexible
-- SurrealDB v3 schemafull object fields require FLEXIBLE for nested webhook payload keys.
-- Preconditions: 0001-0029 applied

DEFINE FIELD OVERWRITE payload ON TABLE webhook_outbox_event TYPE object FLEXIBLE;

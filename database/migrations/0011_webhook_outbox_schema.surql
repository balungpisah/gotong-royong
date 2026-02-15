-- 0011_webhook_outbox_schema
-- Outbound webhook delivery outbox + delivery log schema.

DEFINE TABLE webhook_outbox_event SCHEMAFULL;
DEFINE TABLE webhook_delivery_log SCHEMAFULL;

DEFINE FIELD event_id ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD event_type ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD payload ON TABLE webhook_outbox_event TYPE object;
DEFINE FIELD actor_id ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD actor_username ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD request_id ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD correlation_id ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD status ON TABLE webhook_outbox_event TYPE string;
DEFINE FIELD attempts ON TABLE webhook_outbox_event TYPE number;
DEFINE FIELD max_attempts ON TABLE webhook_outbox_event TYPE number;
DEFINE FIELD next_attempt_at ON TABLE webhook_outbox_event TYPE option datetime;
DEFINE FIELD last_status_code ON TABLE webhook_outbox_event TYPE option number;
DEFINE FIELD last_error ON TABLE webhook_outbox_event TYPE option string;
DEFINE FIELD created_at ON TABLE webhook_outbox_event TYPE datetime;
DEFINE FIELD updated_at ON TABLE webhook_outbox_event TYPE datetime;

DEFINE FIELD log_id ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD event_id ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD attempt ON TABLE webhook_delivery_log TYPE number;
DEFINE FIELD outcome ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD status_code ON TABLE webhook_delivery_log TYPE option number;
DEFINE FIELD request_id ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD correlation_id ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD request_body_sha256 ON TABLE webhook_delivery_log TYPE string;
DEFINE FIELD response_body_sha256 ON TABLE webhook_delivery_log TYPE option string;
DEFINE FIELD error_message ON TABLE webhook_delivery_log TYPE option string;
DEFINE FIELD created_at ON TABLE webhook_delivery_log TYPE datetime;

DEFINE INDEX uniq_outbox_event_id
ON TABLE webhook_outbox_event FIELDS event_id UNIQUE;

DEFINE INDEX uniq_outbox_request
ON TABLE webhook_outbox_event FIELDS request_id UNIQUE;

DEFINE INDEX idx_outbox_status
ON TABLE webhook_outbox_event FIELDS status, created_at, event_id;

DEFINE INDEX idx_log_event_attempt
ON TABLE webhook_delivery_log FIELDS event_id, attempt;

-- 0024_discovery_payload_flexible
-- SurrealDB v3 schemafull object fields require FLEXIBLE for nested object keys.
-- Preconditions: 0001-0023 applied

DEFINE FIELD OVERWRITE payload ON TABLE discovery_feed_item TYPE option<object> FLEXIBLE;
DEFINE FIELD OVERWRITE payload ON TABLE discovery_notification TYPE option<object> FLEXIBLE;

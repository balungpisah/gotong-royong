-- 0008_discovery_schema
-- Discovery feed/search and notification read-model tables.

DEFINE TABLE discovery_feed_item SCHEMAFULL;
DEFINE TABLE discovery_notification SCHEMAFULL;

DEFINE FIELD feed_id ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD source_type ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD source_id ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD actor_id ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD actor_username ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD title ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD summary ON TABLE discovery_feed_item TYPE option string;
DEFINE FIELD track ON TABLE discovery_feed_item TYPE option string;
DEFINE FIELD stage ON TABLE discovery_feed_item TYPE option string;
DEFINE FIELD scope_id ON TABLE discovery_feed_item TYPE option string;
DEFINE FIELD privacy_level ON TABLE discovery_feed_item TYPE option string;
DEFINE FIELD occurred_at ON TABLE discovery_feed_item TYPE datetime;
DEFINE FIELD created_at ON TABLE discovery_feed_item TYPE datetime;
DEFINE FIELD request_id ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD correlation_id ON TABLE discovery_feed_item TYPE string;
DEFINE FIELD participant_ids ON TABLE discovery_feed_item TYPE array;
DEFINE FIELD payload ON TABLE discovery_feed_item TYPE option object;

DEFINE FIELD notification_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD user_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD actor_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD actor_username ON TABLE discovery_notification TYPE string;
DEFINE FIELD notification_type ON TABLE discovery_notification TYPE string;
DEFINE FIELD source_type ON TABLE discovery_notification TYPE string;
DEFINE FIELD source_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD title ON TABLE discovery_notification TYPE string;
DEFINE FIELD body ON TABLE discovery_notification TYPE string;
DEFINE FIELD payload ON TABLE discovery_notification TYPE option object;
DEFINE FIELD created_at ON TABLE discovery_notification TYPE datetime;
DEFINE FIELD read_at ON TABLE discovery_notification TYPE option datetime;
DEFINE FIELD privacy_level ON TABLE discovery_notification TYPE option string;
DEFINE FIELD request_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD correlation_id ON TABLE discovery_notification TYPE string;
DEFINE FIELD dedupe_key ON TABLE discovery_notification TYPE string;

DEFINE INDEX uniq_feed_source_request
ON TABLE discovery_feed_item FIELDS source_type, source_id, request_id UNIQUE;

DEFINE INDEX idx_feed_scope
ON TABLE discovery_feed_item FIELDS scope_id, occurred_at, feed_id;

DEFINE INDEX idx_feed_actor
ON TABLE discovery_feed_item FIELDS actor_id, occurred_at, feed_id;

DEFINE INDEX idx_feed_time
ON TABLE discovery_feed_item FIELDS occurred_at, feed_id;

DEFINE INDEX uniq_notification_dedupe
ON TABLE discovery_notification FIELDS user_id, dedupe_key UNIQUE;

DEFINE INDEX idx_notification_user
ON TABLE discovery_notification FIELDS user_id, created_at, notification_id;


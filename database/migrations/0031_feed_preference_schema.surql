-- 0031_feed_preference_schema
-- Persisted follow/monitor preferences for FEED-PREF-001.

DEFINE TABLE feed_monitor_preference SCHEMAFULL;
DEFINE FIELD user_id ON TABLE feed_monitor_preference TYPE string;
DEFINE FIELD witness_id ON TABLE feed_monitor_preference TYPE string;
DEFINE FIELD monitored ON TABLE feed_monitor_preference TYPE bool;
DEFINE FIELD updated_at_ms ON TABLE feed_monitor_preference TYPE int;

DEFINE INDEX uniq_feed_monitor_preference
ON TABLE feed_monitor_preference FIELDS user_id, witness_id UNIQUE;

DEFINE INDEX idx_feed_monitor_preference_user_updated
ON TABLE feed_monitor_preference FIELDS user_id, updated_at_ms, witness_id;

DEFINE TABLE feed_follow_preference SCHEMAFULL;
DEFINE FIELD user_id ON TABLE feed_follow_preference TYPE string;
DEFINE FIELD entity_id ON TABLE feed_follow_preference TYPE string;
DEFINE FIELD followed ON TABLE feed_follow_preference TYPE bool;
DEFINE FIELD updated_at_ms ON TABLE feed_follow_preference TYPE int;

DEFINE INDEX uniq_feed_follow_preference
ON TABLE feed_follow_preference FIELDS user_id, entity_id UNIQUE;

DEFINE INDEX idx_feed_follow_preference_user_updated
ON TABLE feed_follow_preference FIELDS user_id, updated_at_ms, entity_id;

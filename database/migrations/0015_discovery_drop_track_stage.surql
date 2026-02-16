-- 0015_discovery_drop_track_stage
-- Remove deprecated lifecycle coupling fields from discovery feed items.
-- Preconditions: 0001-0014 applied.

REMOVE FIELD track ON TABLE discovery_feed_item;
REMOVE FIELD stage ON TABLE discovery_feed_item;

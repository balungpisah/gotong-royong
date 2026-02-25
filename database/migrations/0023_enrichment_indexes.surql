-- 0023_enrichment_indexes
-- Indexes to support ontology→feed enrichment and anti-gaming idempotency.
-- Preconditions: 0001–0022 applied

-- Lookup latest feed item for a given (source_type, source_id) efficiently.
DEFINE INDEX idx_feed_source_latest
ON TABLE discovery_feed_item FIELDS source_type, source_id, occurred_at, feed_id;

-- Prevent double-vouch / double-challenge by the same actor on the same note.
DEFINE INDEX uniq_vouches_edge
ON TABLE VOUCHES FIELDS in, out UNIQUE;

DEFINE INDEX uniq_challenges_edge
ON TABLE CHALLENGES FIELDS in, out UNIQUE;


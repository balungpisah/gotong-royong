-- 0027_hot_path_pack_c_feed_participant_edge
-- Pack C Phase C1: schema-only participant edge read model for involvement feed lane.
-- Preconditions: 0001-0026 applied

DEFINE TABLE feed_participant_edge SCHEMAFULL
    PERMISSIONS
        FOR select WHERE actor_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

DEFINE FIELD edge_id ON TABLE feed_participant_edge TYPE string;
DEFINE FIELD actor_id ON TABLE feed_participant_edge TYPE string;
DEFINE FIELD feed_id ON TABLE feed_participant_edge TYPE string;
DEFINE FIELD occurred_at ON TABLE feed_participant_edge TYPE datetime;
DEFINE FIELD scope_id ON TABLE feed_participant_edge TYPE option<string>;
DEFINE FIELD privacy_level ON TABLE feed_participant_edge TYPE option<string>;
DEFINE FIELD source_type ON TABLE feed_participant_edge TYPE string;
DEFINE FIELD source_id ON TABLE feed_participant_edge TYPE string;
DEFINE FIELD created_at ON TABLE feed_participant_edge TYPE datetime;
DEFINE FIELD request_id ON TABLE feed_participant_edge TYPE string;

DEFINE INDEX uniq_feed_participant_edge
ON TABLE feed_participant_edge FIELDS actor_id, feed_id UNIQUE;

DEFINE INDEX idx_feed_participant_actor_latest
ON TABLE feed_participant_edge FIELDS actor_id, occurred_at, feed_id;

DEFINE INDEX idx_feed_participant_actor_scope_latest
ON TABLE feed_participant_edge FIELDS actor_id, scope_id, occurred_at, feed_id;

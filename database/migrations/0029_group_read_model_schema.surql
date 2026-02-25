-- 0029_group_read_model_schema
-- Group/Kelompok read-model tables for GROUP-API-001 persistence.

DEFINE TABLE group_summary SCHEMAFULL;
DEFINE TABLE group_member SCHEMAFULL;
DEFINE TABLE group_join_request SCHEMAFULL;

DEFINE FIELD group_id ON TABLE group_summary TYPE string;
DEFINE FIELD name ON TABLE group_summary TYPE string;
DEFINE FIELD description ON TABLE group_summary TYPE string;
DEFINE FIELD entity_type ON TABLE group_summary TYPE string;
DEFINE FIELD join_policy ON TABLE group_summary TYPE string;
DEFINE FIELD member_count ON TABLE group_summary TYPE int;
DEFINE FIELD witness_count ON TABLE group_summary TYPE int;
DEFINE FIELD updated_at ON TABLE group_summary TYPE datetime;

DEFINE FIELD group_id ON TABLE group_member TYPE string;
DEFINE FIELD user_id ON TABLE group_member TYPE string;
DEFINE FIELD name ON TABLE group_member TYPE string;
DEFINE FIELD avatar_url ON TABLE group_member TYPE option<string>;
DEFINE FIELD role ON TABLE group_member TYPE string;
DEFINE FIELD joined_at ON TABLE group_member TYPE datetime;

DEFINE FIELD request_id ON TABLE group_join_request TYPE string;
DEFINE FIELD group_id ON TABLE group_join_request TYPE string;
DEFINE FIELD user_id ON TABLE group_join_request TYPE string;
DEFINE FIELD name ON TABLE group_join_request TYPE string;
DEFINE FIELD avatar_url ON TABLE group_join_request TYPE option<string>;
DEFINE FIELD message ON TABLE group_join_request TYPE option<string>;
DEFINE FIELD status ON TABLE group_join_request TYPE string;
DEFINE FIELD requested_at ON TABLE group_join_request TYPE datetime;

DEFINE INDEX uniq_group_summary_id
ON TABLE group_summary FIELDS group_id UNIQUE;

DEFINE INDEX idx_group_summary_updated
ON TABLE group_summary FIELDS updated_at, group_id;

DEFINE INDEX idx_group_summary_policy_updated
ON TABLE group_summary FIELDS join_policy, updated_at, group_id;

DEFINE INDEX uniq_group_member
ON TABLE group_member FIELDS group_id, user_id UNIQUE;

DEFINE INDEX idx_group_member_user
ON TABLE group_member FIELDS user_id, group_id, joined_at;

DEFINE INDEX uniq_group_join_request_id
ON TABLE group_join_request FIELDS request_id UNIQUE;

DEFINE INDEX idx_group_join_request_group_status
ON TABLE group_join_request FIELDS group_id, status, requested_at, request_id;

DEFINE INDEX idx_group_join_request_user_status
ON TABLE group_join_request FIELDS user_id, status, requested_at, request_id;

-- 0007_siaga_schema
-- Emergency broadcast lifecycle schema for BE-005.

DEFINE TABLE siaga_broadcast SCHEMAFULL;
DEFINE TABLE siaga_timeline_event SCHEMAFULL;

DEFINE FIELD siaga_id ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD scope_id ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD author_id ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD author_username ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD emergency_type ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD severity ON TABLE siaga_broadcast TYPE number;
DEFINE FIELD location ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD title ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD text ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD state ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD created_at ON TABLE siaga_broadcast TYPE datetime;
DEFINE FIELD updated_at ON TABLE siaga_broadcast TYPE datetime;
DEFINE FIELD request_id ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD correlation_id ON TABLE siaga_broadcast TYPE string;
DEFINE FIELD responders ON TABLE siaga_broadcast TYPE array;
DEFINE FIELD closure ON TABLE siaga_broadcast TYPE option object;

DEFINE FIELD siaga_id ON TABLE siaga_timeline_event TYPE string;
DEFINE FIELD event_id ON TABLE siaga_timeline_event TYPE string;
DEFINE FIELD event_type ON TABLE siaga_timeline_event TYPE string;
DEFINE FIELD actor ON TABLE siaga_timeline_event TYPE object;
DEFINE FIELD request_id ON TABLE siaga_timeline_event TYPE string;
DEFINE FIELD correlation_id ON TABLE siaga_timeline_event TYPE string;
DEFINE FIELD occurred_at ON TABLE siaga_timeline_event TYPE datetime;
DEFINE FIELD metadata ON TABLE siaga_timeline_event TYPE option object;

DEFINE INDEX uniq_siaga_broadcast_id
ON TABLE siaga_broadcast FIELDS siaga_id UNIQUE;

DEFINE INDEX uniq_siaga_author_request
ON TABLE siaga_broadcast FIELDS author_id, request_id UNIQUE;

DEFINE INDEX idx_siaga_broadcast_scope
ON TABLE siaga_broadcast FIELDS scope_id, created_at, siaga_id;

DEFINE INDEX idx_siaga_broadcast_state
ON TABLE siaga_broadcast FIELDS state, scope_id;

DEFINE INDEX uniq_siaga_broadcast_request
ON TABLE siaga_broadcast FIELDS siaga_id, request_id UNIQUE;

DEFINE INDEX uniq_siaga_timeline_id
ON TABLE siaga_timeline_event FIELDS event_id UNIQUE;

DEFINE INDEX idx_siaga_timeline_request_type
ON TABLE siaga_timeline_event FIELDS event_type, request_id, occurred_at, event_id;

DEFINE INDEX idx_siaga_timeline
ON TABLE siaga_timeline_event FIELDS siaga_id, occurred_at, event_id;

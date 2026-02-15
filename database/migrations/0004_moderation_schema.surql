-- 0004_moderation_schema
-- Moderation state and decision ledger for BE-003 / BE-004.

DEFINE TABLE content_moderation SCHEMAFULL;
DEFINE TABLE moderation_decision SCHEMAFULL;

DEFINE FIELD content_id ON TABLE content_moderation TYPE string;
DEFINE FIELD content_type ON TABLE content_moderation TYPE option string;
DEFINE FIELD author_id ON TABLE content_moderation TYPE string;
DEFINE FIELD author_username ON TABLE content_moderation TYPE option string;
DEFINE FIELD moderation_status ON TABLE content_moderation TYPE string;
DEFINE FIELD moderation_action ON TABLE content_moderation TYPE string;
DEFINE FIELD reason_code ON TABLE content_moderation TYPE option string;
DEFINE FIELD confidence ON TABLE content_moderation TYPE float;
DEFINE FIELD decided_at ON TABLE content_moderation TYPE datetime;
DEFINE FIELD decided_by ON TABLE content_moderation TYPE string;
DEFINE FIELD hold_expires_at ON TABLE content_moderation TYPE option datetime;
DEFINE FIELD auto_release_if_no_action ON TABLE content_moderation TYPE bool;
DEFINE FIELD appeal_window_until ON TABLE content_moderation TYPE option datetime;
DEFINE FIELD reasoning ON TABLE content_moderation TYPE option string;
DEFINE FIELD violations ON TABLE content_moderation TYPE array;
DEFINE FIELD last_decision_id ON TABLE content_moderation TYPE option string;
DEFINE FIELD request_id ON TABLE content_moderation TYPE string;
DEFINE FIELD correlation_id ON TABLE content_moderation TYPE string;
DEFINE FIELD request_ts_ms ON TABLE content_moderation TYPE number;

DEFINE FIELD decision_id ON TABLE moderation_decision TYPE string;
DEFINE FIELD content_id ON TABLE moderation_decision TYPE string;
DEFINE FIELD content_type ON TABLE moderation_decision TYPE option string;
DEFINE FIELD moderation_status ON TABLE moderation_decision TYPE string;
DEFINE FIELD moderation_action ON TABLE moderation_decision TYPE string;
DEFINE FIELD reason_code ON TABLE moderation_decision TYPE option string;
DEFINE FIELD confidence ON TABLE moderation_decision TYPE float;
DEFINE FIELD decided_at ON TABLE moderation_decision TYPE datetime;
DEFINE FIELD actor ON TABLE moderation_decision TYPE object;
DEFINE FIELD hold_expires_at ON TABLE moderation_decision TYPE option datetime;
DEFINE FIELD auto_release_if_no_action ON TABLE moderation_decision TYPE bool;
DEFINE FIELD appeal_window_until ON TABLE moderation_decision TYPE option datetime;
DEFINE FIELD reasoning ON TABLE moderation_decision TYPE option string;
DEFINE FIELD violations ON TABLE moderation_decision TYPE array;
DEFINE FIELD request_id ON TABLE moderation_decision TYPE string;
DEFINE FIELD correlation_id ON TABLE moderation_decision TYPE string;

DEFINE INDEX uniq_moderation_content_request
ON TABLE moderation_decision FIELDS content_id, request_id UNIQUE;

DEFINE INDEX uniq_content_decision_id
ON TABLE moderation_decision FIELDS decision_id UNIQUE;

DEFINE INDEX idx_content_status
ON TABLE content_moderation FIELDS moderation_status;

DEFINE INDEX idx_content_decision
ON TABLE moderation_decision FIELDS content_id;

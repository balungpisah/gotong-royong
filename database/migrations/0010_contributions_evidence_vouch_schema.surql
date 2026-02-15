-- 0010_contributions_evidence_vouch_schema
-- Core contribution/evidence/vouch baseline schema.

DEFINE TABLE contribution SCHEMAFULL;
DEFINE TABLE evidence SCHEMAFULL;
DEFINE TABLE vouch SCHEMAFULL;

DEFINE FIELD contribution_id ON TABLE contribution TYPE string;
DEFINE FIELD author_id ON TABLE contribution TYPE string;
DEFINE FIELD author_username ON TABLE contribution TYPE string;
DEFINE FIELD contribution_type ON TABLE contribution TYPE string;
DEFINE FIELD title ON TABLE contribution TYPE string;
DEFINE FIELD description ON TABLE contribution TYPE option<string>;
DEFINE FIELD evidence_url ON TABLE contribution TYPE option<string>;
DEFINE FIELD skill_ids ON TABLE contribution TYPE array;
DEFINE FIELD metadata ON TABLE contribution TYPE option<object>;
DEFINE FIELD request_id ON TABLE contribution TYPE string;
DEFINE FIELD correlation_id ON TABLE contribution TYPE string;
DEFINE FIELD created_at ON TABLE contribution TYPE datetime;
DEFINE FIELD updated_at ON TABLE contribution TYPE datetime;

DEFINE FIELD evidence_id ON TABLE evidence TYPE string;
DEFINE FIELD contribution_id ON TABLE evidence TYPE string;
DEFINE FIELD actor_id ON TABLE evidence TYPE string;
DEFINE FIELD actor_username ON TABLE evidence TYPE string;
DEFINE FIELD evidence_type ON TABLE evidence TYPE string;
DEFINE FIELD evidence_data ON TABLE evidence TYPE object;
DEFINE FIELD proof ON TABLE evidence TYPE object;
DEFINE FIELD request_id ON TABLE evidence TYPE string;
DEFINE FIELD correlation_id ON TABLE evidence TYPE string;
DEFINE FIELD created_at ON TABLE evidence TYPE datetime;
DEFINE FIELD updated_at ON TABLE evidence TYPE datetime;

DEFINE FIELD vouch_id ON TABLE vouch TYPE string;
DEFINE FIELD voucher_id ON TABLE vouch TYPE string;
DEFINE FIELD voucher_username ON TABLE vouch TYPE string;
DEFINE FIELD vouchee_id ON TABLE vouch TYPE string;
DEFINE FIELD skill_id ON TABLE vouch TYPE option<string>;
DEFINE FIELD weight_hint ON TABLE vouch TYPE option<string>;
DEFINE FIELD message ON TABLE vouch TYPE option<string>;
DEFINE FIELD request_id ON TABLE vouch TYPE string;
DEFINE FIELD correlation_id ON TABLE vouch TYPE string;
DEFINE FIELD created_at ON TABLE vouch TYPE datetime;
DEFINE FIELD updated_at ON TABLE vouch TYPE datetime;

DEFINE INDEX idx_contribution_author
ON TABLE contribution FIELDS author_id, created_at, contribution_id;

DEFINE INDEX idx_evidence_contribution
ON TABLE evidence FIELDS contribution_id, created_at, evidence_id;

DEFINE INDEX idx_vouch_vouchee
ON TABLE vouch FIELDS vouchee_id, created_at, vouch_id;

DEFINE INDEX idx_vouch_voucher
ON TABLE vouch FIELDS voucher_id, created_at, vouch_id;

-- 0012_adaptive_path_schema
-- Adaptive path canonical plan + audit/suggestion timeline.

DEFINE TABLE path_plan SCHEMAFULL;
DEFINE TABLE path_plan_event SCHEMAFULL;
DEFINE TABLE plan_suggestion SCHEMAFULL;

-- Optional normalized detail tables for downstream projections.
DEFINE TABLE path_branch SCHEMAFULL;
DEFINE TABLE path_phase SCHEMAFULL;
DEFINE TABLE path_checkpoint SCHEMAFULL;

DEFINE FIELD plan_id ON TABLE path_plan TYPE string;
DEFINE FIELD entity_id ON TABLE path_plan TYPE string;
DEFINE FIELD version ON TABLE path_plan TYPE int;
DEFINE FIELD title ON TABLE path_plan TYPE string;
DEFINE FIELD summary ON TABLE path_plan TYPE option<string>;
DEFINE FIELD track_hint ON TABLE path_plan TYPE option<string>;
DEFINE FIELD seed_hint ON TABLE path_plan TYPE option<string>;
DEFINE FIELD author_id ON TABLE path_plan TYPE string;
DEFINE FIELD author_username ON TABLE path_plan TYPE string;
DEFINE FIELD branches ON TABLE path_plan TYPE array;
DEFINE FIELD request_id ON TABLE path_plan TYPE string;
DEFINE FIELD correlation_id ON TABLE path_plan TYPE string;
DEFINE FIELD created_at_ms ON TABLE path_plan TYPE int;
DEFINE FIELD updated_at_ms ON TABLE path_plan TYPE int;
DEFINE FIELD event_hash ON TABLE path_plan TYPE string;
DEFINE FIELD retention_tag ON TABLE path_plan TYPE string;

DEFINE FIELD event_id ON TABLE path_plan_event TYPE string;
DEFINE FIELD plan_id ON TABLE path_plan_event TYPE string;
DEFINE FIELD event_type ON TABLE path_plan_event TYPE string;
DEFINE FIELD actor ON TABLE path_plan_event TYPE object;
DEFINE FIELD request_id ON TABLE path_plan_event TYPE string;
DEFINE FIELD correlation_id ON TABLE path_plan_event TYPE string;
DEFINE FIELD base_version ON TABLE path_plan_event TYPE int;
DEFINE FIELD next_version ON TABLE path_plan_event TYPE int;
DEFINE FIELD occurred_at_ms ON TABLE path_plan_event TYPE int;
DEFINE FIELD metadata ON TABLE path_plan_event TYPE option<object>;
DEFINE FIELD event_hash ON TABLE path_plan_event TYPE string;
DEFINE FIELD retention_tag ON TABLE path_plan_event TYPE string;

DEFINE FIELD suggestion_id ON TABLE plan_suggestion TYPE string;
DEFINE FIELD plan_id ON TABLE plan_suggestion TYPE string;
DEFINE FIELD base_version ON TABLE plan_suggestion TYPE int;
DEFINE FIELD proposal ON TABLE plan_suggestion TYPE object;
DEFINE FIELD status ON TABLE plan_suggestion TYPE string;
DEFINE FIELD created_by ON TABLE plan_suggestion TYPE string;
DEFINE FIELD created_by_role ON TABLE plan_suggestion TYPE string;
DEFINE FIELD rationale ON TABLE plan_suggestion TYPE option<string>;
DEFINE FIELD model_id ON TABLE plan_suggestion TYPE option<string>;
DEFINE FIELD prompt_version ON TABLE plan_suggestion TYPE option<string>;
DEFINE FIELD request_id ON TABLE plan_suggestion TYPE string;
DEFINE FIELD correlation_id ON TABLE plan_suggestion TYPE string;
DEFINE FIELD created_at_ms ON TABLE plan_suggestion TYPE int;
DEFINE FIELD updated_at_ms ON TABLE plan_suggestion TYPE int;
DEFINE FIELD event_hash ON TABLE plan_suggestion TYPE string;
DEFINE FIELD retention_tag ON TABLE plan_suggestion TYPE string;

DEFINE FIELD branch_id ON TABLE path_branch TYPE string;
DEFINE FIELD plan_id ON TABLE path_branch TYPE string;
DEFINE FIELD label ON TABLE path_branch TYPE string;
DEFINE FIELD parent_checkpoint_id ON TABLE path_branch TYPE option<string>;
DEFINE FIELD branch_order ON TABLE path_branch TYPE int;
DEFINE FIELD locked_fields ON TABLE path_branch TYPE array;

DEFINE FIELD phase_id ON TABLE path_phase TYPE string;
DEFINE FIELD plan_id ON TABLE path_phase TYPE string;
DEFINE FIELD branch_id ON TABLE path_phase TYPE string;
DEFINE FIELD title ON TABLE path_phase TYPE string;
DEFINE FIELD objective ON TABLE path_phase TYPE string;
DEFINE FIELD status ON TABLE path_phase TYPE string;
DEFINE FIELD phase_order ON TABLE path_phase TYPE int;
DEFINE FIELD source ON TABLE path_phase TYPE string;
DEFINE FIELD locked_fields ON TABLE path_phase TYPE array;

DEFINE FIELD checkpoint_id ON TABLE path_checkpoint TYPE string;
DEFINE FIELD plan_id ON TABLE path_checkpoint TYPE string;
DEFINE FIELD phase_id ON TABLE path_checkpoint TYPE string;
DEFINE FIELD title ON TABLE path_checkpoint TYPE string;
DEFINE FIELD status ON TABLE path_checkpoint TYPE string;
DEFINE FIELD checkpoint_order ON TABLE path_checkpoint TYPE int;
DEFINE FIELD source ON TABLE path_checkpoint TYPE string;
DEFINE FIELD locked_fields ON TABLE path_checkpoint TYPE array;

DEFINE INDEX uniq_path_plan_id ON TABLE path_plan FIELDS plan_id UNIQUE;
DEFINE INDEX uniq_path_plan_entity ON TABLE path_plan FIELDS entity_id UNIQUE;
DEFINE INDEX uniq_path_plan_request ON TABLE path_plan FIELDS entity_id, request_id UNIQUE;
DEFINE INDEX idx_path_plan_updated ON TABLE path_plan FIELDS updated_at_ms, plan_id;

DEFINE INDEX uniq_path_plan_event_id ON TABLE path_plan_event FIELDS event_id UNIQUE;
DEFINE INDEX uniq_path_plan_event_request ON TABLE path_plan_event FIELDS request_id UNIQUE;
DEFINE INDEX idx_path_plan_event_timeline
ON TABLE path_plan_event FIELDS plan_id, occurred_at_ms, event_id;

DEFINE INDEX uniq_plan_suggestion_id ON TABLE plan_suggestion FIELDS suggestion_id UNIQUE;
DEFINE INDEX uniq_plan_suggestion_request
ON TABLE plan_suggestion FIELDS plan_id, request_id UNIQUE;
DEFINE INDEX idx_plan_suggestion_plan_created
ON TABLE plan_suggestion FIELDS plan_id, created_at_ms, suggestion_id;

DEFINE INDEX uniq_path_branch_id ON TABLE path_branch FIELDS branch_id UNIQUE;
DEFINE INDEX idx_path_branch_plan ON TABLE path_branch FIELDS plan_id, branch_order, branch_id;

DEFINE INDEX uniq_path_phase_id ON TABLE path_phase FIELDS phase_id UNIQUE;
DEFINE INDEX idx_path_phase_branch ON TABLE path_phase FIELDS branch_id, phase_order, phase_id;

DEFINE INDEX uniq_path_checkpoint_id ON TABLE path_checkpoint FIELDS checkpoint_id UNIQUE;
DEFINE INDEX idx_path_checkpoint_phase
ON TABLE path_checkpoint FIELDS phase_id, checkpoint_order, checkpoint_id;

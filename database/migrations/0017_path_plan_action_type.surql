-- 0017_path_plan_action_type
-- Replace free-text adaptive path hints with typed Schema.org action classification.
-- Preconditions: 0001-0016 applied.

DEFINE FIELD action_type ON TABLE path_plan TYPE string DEFAULT "schema:InformAction";

UPDATE path_plan
SET action_type = "schema:InformAction"
WHERE action_type IS NONE OR action_type = "";

REMOVE FIELD track_hint ON TABLE path_plan;
REMOVE FIELD seed_hint ON TABLE path_plan;

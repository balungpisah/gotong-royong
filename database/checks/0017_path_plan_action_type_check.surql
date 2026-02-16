-- 0017_path_plan_action_type_check
INFO FOR TABLE path_plan;
SELECT count() AS path_plan_rows_missing_action_type
FROM path_plan
WHERE action_type IS NONE OR action_type = "";

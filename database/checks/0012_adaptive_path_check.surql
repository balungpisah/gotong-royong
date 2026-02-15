-- 0012_adaptive_path_check
INFO FOR TABLE path_plan;
INFO FOR TABLE path_plan_event;
INFO FOR TABLE plan_suggestion;
INFO FOR TABLE path_branch;
INFO FOR TABLE path_phase;
INFO FOR TABLE path_checkpoint;

SELECT count() AS path_plan_rows FROM path_plan;
SELECT count() AS path_plan_event_rows FROM path_plan_event;
SELECT count() AS plan_suggestion_rows FROM plan_suggestion;
SELECT count() AS path_branch_rows FROM path_branch;
SELECT count() AS path_phase_rows FROM path_phase;
SELECT count() AS path_checkpoint_rows FROM path_checkpoint;

-- 0029_group_read_model_schema_check
-- Verify group read-model tables and indexes exist.

INFO FOR TABLE group_summary;
INFO FOR TABLE group_member;
INFO FOR TABLE group_join_request;

SELECT count() AS group_summary_rows FROM group_summary;
SELECT count() AS group_member_rows FROM group_member;
SELECT count() AS group_join_request_rows FROM group_join_request;

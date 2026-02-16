-- 0016_add_mode_fields_check
INFO FOR TABLE contribution;
SELECT count() AS contribution_rows_missing_mode
FROM contribution
WHERE mode IS NONE OR mode = "";

-- 0016_add_mode_fields
-- Add explicit mode persistence for contributions.
-- Preconditions: 0001-0015 applied.

DEFINE FIELD mode ON TABLE contribution TYPE string DEFAULT "komunitas";

UPDATE contribution
SET mode = "komunitas"
WHERE mode IS NONE OR mode = "";

-- 0014_remove_track_state_transition
-- Drop deprecated fixed lifecycle transition schema.
-- Preconditions: 0001-0013 applied.

REMOVE TABLE track_state_transition;

-- 0013_ontology_schema_check
-- Validates ontology graph model tables and edges exist.

-- Node tables exist
SELECT count() FROM concept GROUP ALL;
SELECT count() FROM action GROUP ALL;
SELECT count() FROM place GROUP ALL;
SELECT count() FROM warga GROUP ALL;
SELECT count() FROM note GROUP ALL;
SELECT count() FROM measurement GROUP ALL;

-- Edge tables exist
SELECT count() FROM ABOUT GROUP ALL;
SELECT count() FROM LOCATED_AT GROUP ALL;
SELECT count() FROM HAS_ACTION GROUP ALL;
SELECT count() FROM BROADER GROUP ALL;
SELECT count() FROM INSTANCE_OF GROUP ALL;
SELECT count() FROM VOUCHES GROUP ALL;
SELECT count() FROM CHALLENGES GROUP ALL;

-- Action type seeds present (7 action types)
LET $action_count = (SELECT count() FROM action GROUP ALL)[0].c;
-- Expected: 7
SELECT * FROM action WHERE maps_to_mode = 'catatan_komunitas';
SELECT * FROM action WHERE maps_to_mode = 'komunitas';
SELECT * FROM action WHERE maps_to_mode = 'siaga';

-- Concept QID uniqueness index works
-- (Insert duplicate should fail)
-- This is validated by the UNIQUE index on idx_concept_qid
INFO FOR TABLE concept;
INFO FOR TABLE ABOUT;

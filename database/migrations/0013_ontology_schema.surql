-- 0013_ontology_schema
-- RDF-triple graph model for ONTOLOGY-VOCAB-v0.1
-- Preconditions: 0001–0012 applied
--
-- Design: SCHEMALESS tables to accommodate varied edge metadata.
-- Concept nodes store Wikidata QIDs. Edge tables carry RDF predicate
-- and object metadata as fields on the RELATE record.
-- Ref: docs/design/specs/ONTOLOGY-VOCAB-v0.1.md

-- ================================================================
-- Node tables
-- ================================================================

-- Wikidata concept nodes (pre-seeded ~200, lazily grown)
DEFINE TABLE concept SCHEMALESS;
DEFINE FIELD qid ON TABLE concept TYPE string;
DEFINE FIELD label_id ON TABLE concept TYPE string;
DEFINE FIELD label_en ON TABLE concept TYPE string;
DEFINE FIELD verified ON TABLE concept TYPE bool DEFAULT false;
DEFINE FIELD created_at ON TABLE concept TYPE datetime DEFAULT time::now();
DEFINE FIELD last_referenced ON TABLE concept TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_concept_qid ON TABLE concept FIELDS qid UNIQUE;

-- Schema.org Action type reference nodes
DEFINE TABLE action SCHEMALESS;
DEFINE FIELD action_type ON TABLE action TYPE string;
DEFINE FIELD maps_to_mode ON TABLE action TYPE string;
DEFINE FIELD display_label ON TABLE action TYPE option<string>;
DEFINE INDEX idx_action_type ON TABLE action FIELDS action_type UNIQUE;

-- Hyper-local places (no Wikidata QID)
DEFINE TABLE place SCHEMALESS;
DEFINE FIELD name ON TABLE place TYPE string;
DEFINE FIELD osm_tags ON TABLE place TYPE array DEFAULT [];
DEFINE FIELD location ON TABLE place TYPE option<geometry<point>>;
DEFINE FIELD source ON TABLE place TYPE string DEFAULT 'community';
DEFINE FIELD created_at ON TABLE place TYPE datetime DEFAULT time::now();

-- Community members
DEFINE TABLE warga SCHEMALESS;
DEFINE FIELD username ON TABLE warga TYPE string;
DEFINE FIELD voucher_tier ON TABLE warga TYPE string DEFAULT 'warga';
DEFINE FIELD community_id ON TABLE warga TYPE string;

-- Community notes (Catatan Komunitas + Catatan Saksi)
DEFINE TABLE note SCHEMALESS;
DEFINE FIELD content ON TABLE note TYPE string;
DEFINE FIELD author ON TABLE note TYPE record<warga>;
DEFINE FIELD community_id ON TABLE note TYPE string;
DEFINE FIELD created_at ON TABLE note TYPE datetime DEFAULT time::now();
DEFINE FIELD temporal_class ON TABLE note TYPE string;
DEFINE FIELD ttl_expires ON TABLE note TYPE option<datetime>;
DEFINE FIELD ai_readable ON TABLE note TYPE bool DEFAULT true;
DEFINE FIELD rahasia_level ON TABLE note TYPE int DEFAULT 0;
DEFINE FIELD confidence ON TABLE note TYPE float DEFAULT 0.0;
DEFINE INDEX idx_note_community ON TABLE note FIELDS community_id, created_at;
DEFINE INDEX idx_note_ttl ON TABLE note FIELDS ttl_expires;
DEFINE INDEX idx_note_temporal ON TABLE note FIELDS temporal_class;
DEFINE INDEX idx_note_rahasia ON TABLE note FIELDS rahasia_level, ai_readable;

-- Numeric measurements (time-series)
DEFINE TABLE measurement SCHEMALESS;
DEFINE FIELD note ON TABLE measurement TYPE record<note>;
DEFINE FIELD concept ON TABLE measurement TYPE record<concept>;
DEFINE FIELD location ON TABLE measurement TYPE option<record>;
DEFINE FIELD predicate ON TABLE measurement TYPE string;
DEFINE FIELD value ON TABLE measurement TYPE number;
DEFINE FIELD unit ON TABLE measurement TYPE string;
DEFINE FIELD per ON TABLE measurement TYPE option<string>;
DEFINE FIELD observed_at ON TABLE measurement TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_measurement_concept ON TABLE measurement FIELDS concept, observed_at;
DEFINE INDEX idx_measurement_location ON TABLE measurement FIELDS location, observed_at;

-- ================================================================
-- Edge tables (RELATE targets)
-- ================================================================

-- note/plan/siaga -> concept (carries predicate + object metadata)
DEFINE TABLE ABOUT SCHEMALESS;
DEFINE INDEX idx_about_in ON TABLE ABOUT FIELDS in;
DEFINE INDEX idx_about_out ON TABLE ABOUT FIELDS out;

-- note/plan/siaga -> concept|place (spatial relation)
DEFINE TABLE LOCATED_AT SCHEMALESS;
DEFINE INDEX idx_located_in ON TABLE LOCATED_AT FIELDS in;
DEFINE INDEX idx_located_out ON TABLE LOCATED_AT FIELDS out;

-- note/plan/siaga -> action (intent/routing signal)
DEFINE TABLE HAS_ACTION SCHEMALESS;
DEFINE INDEX idx_action_edge_in ON TABLE HAS_ACTION FIELDS in;
DEFINE INDEX idx_action_edge_out ON TABLE HAS_ACTION FIELDS out;

-- concept -> concept (Wikidata P279 subclass hierarchy)
DEFINE TABLE BROADER SCHEMALESS;
DEFINE INDEX idx_broader_in ON TABLE BROADER FIELDS in;
DEFINE INDEX idx_broader_out ON TABLE BROADER FIELDS out;

-- place -> concept (P31 instance of)
DEFINE TABLE INSTANCE_OF SCHEMALESS;
DEFINE INDEX idx_instance_in ON TABLE INSTANCE_OF FIELDS in;
DEFINE INDEX idx_instance_out ON TABLE INSTANCE_OF FIELDS out;

-- warga -> note (trust signal)
DEFINE TABLE VOUCHES SCHEMALESS;
DEFINE INDEX idx_vouch_in ON TABLE VOUCHES FIELDS in;
DEFINE INDEX idx_vouch_out ON TABLE VOUCHES FIELDS out;

-- warga -> note (dispute signal)
DEFINE TABLE CHALLENGES SCHEMALESS;
DEFINE INDEX idx_challenge_in ON TABLE CHALLENGES FIELDS in;
DEFINE INDEX idx_challenge_out ON TABLE CHALLENGES FIELDS out;

-- ================================================================
-- Seed: Action type reference data
-- ================================================================

CREATE action:InformAction SET action_type = 'schema:InformAction', maps_to_mode = 'catatan_komunitas', display_label = 'Informasi';
CREATE action:RepairAction SET action_type = 'schema:RepairAction', maps_to_mode = 'komunitas', display_label = 'Tuntaskan';
CREATE action:CreateAction SET action_type = 'schema:CreateAction', maps_to_mode = 'komunitas', display_label = 'Wujudkan';
CREATE action:SearchAction SET action_type = 'schema:SearchAction', maps_to_mode = 'komunitas', display_label = 'Telusuri';
CREATE action:AchieveAction SET action_type = 'schema:AchieveAction', maps_to_mode = 'komunitas', display_label = 'Rayakan';
CREATE action:AssessAction SET action_type = 'schema:AssessAction', maps_to_mode = 'komunitas', display_label = 'Musyawarah';
CREATE action:AlertAction SET action_type = 'schema:AlertAction', maps_to_mode = 'siaga', display_label = 'Siaga';

-- 0014_witness_graph_schema
-- Frontend-aligned Witness aggregate + graph relationships
-- Implements: paginated messages, real-time WebSocket, as-needed graph traversal
-- Preconditions: 0001, 0002 (chat tables), 0013 (ontology)

-- ================================================================
-- PRIMARY AGGREGATE: Witness
-- ================================================================

DEFINE TABLE witness SCHEMAFULL;
DEFINE FIELD witness_id ON TABLE witness TYPE string;
DEFINE FIELD title ON TABLE witness TYPE string;
DEFINE FIELD summary ON TABLE witness TYPE string;
DEFINE FIELD status ON TABLE witness TYPE string;  -- "draft" | "open" | "active" | "resolved" | "closed"
DEFINE FIELD rahasia_level ON TABLE witness TYPE string;  -- "L0" | "L1" | "L2" | "L3"
DEFINE FIELD track_hint ON TABLE witness TYPE option<string>;  -- "tuntaskan" | "rayakan" | "teliti"
DEFINE FIELD seed_hint ON TABLE witness TYPE option<string>;
DEFINE FIELD close_reason ON TABLE witness TYPE option<string>;  -- only when resolved/closed
DEFINE FIELD created_by ON TABLE witness TYPE string;
DEFINE FIELD created_at ON TABLE witness TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE witness TYPE datetime DEFAULT time::now();
-- Denormalized counts for quick display
DEFINE FIELD member_count ON TABLE witness TYPE number DEFAULT 0;
DEFINE FIELD message_count ON TABLE witness TYPE number DEFAULT 0;
DEFINE FIELD unread_count ON TABLE witness TYPE number DEFAULT 0;
-- Audit fields
DEFINE FIELD request_id ON TABLE witness TYPE string;
DEFINE FIELD correlation_id ON TABLE witness TYPE string;

DEFINE INDEX idx_witness_id ON TABLE witness FIELDS witness_id UNIQUE;
DEFINE INDEX idx_witness_status ON TABLE witness FIELDS status;
DEFINE INDEX idx_witness_created_by ON TABLE witness FIELDS created_by;
DEFINE INDEX idx_witness_created_at ON TABLE witness FIELDS created_at;

-- ================================================================
-- GRAPH STRUCTURE: Witness Members (role-based relationships)
-- ================================================================

DEFINE TABLE witness_member SCHEMAFULL;
DEFINE FIELD witness_member_id ON TABLE witness_member TYPE string;
DEFINE FIELD witness_id ON TABLE witness_member TYPE string;
DEFINE FIELD user_id ON TABLE witness_member TYPE string;
DEFINE FIELD name ON TABLE witness_member TYPE string;
DEFINE FIELD avatar_url ON TABLE witness_member TYPE option<string>;
DEFINE FIELD role ON TABLE witness_member TYPE string;  -- "pelapor" | "relawan" | "koordinator" | "saksi"
DEFINE FIELD tier ON TABLE witness_member TYPE option<number>;
DEFINE FIELD joined_at ON TABLE witness_member TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_witness_member_witness ON TABLE witness_member FIELDS witness_id;
DEFINE INDEX idx_witness_member_user ON TABLE witness_member FIELDS user_id;
DEFINE INDEX idx_witness_member_role ON TABLE witness_member FIELDS witness_id, role;

-- ================================================================
-- MESSAGES: Paginated Chat (separate for streaming)
-- ================================================================

DEFINE TABLE witness_message SCHEMAFULL;
DEFINE FIELD message_id ON TABLE witness_message TYPE string;
DEFINE FIELD witness_id ON TABLE witness_message TYPE string;
DEFINE FIELD type ON TABLE witness_message TYPE string;  -- "user" | "ai_card" | "diff_card" | "vote_card" | "system" | "evidence" | "galang"
DEFINE FIELD timestamp ON TABLE witness_message TYPE datetime DEFAULT time::now();
-- Author (for user/evidence messages)
DEFINE FIELD author ON TABLE witness_message TYPE option<object>;  -- { user_id, name, avatar_url, tier, role }
DEFINE FIELD is_self ON TABLE witness_message TYPE option<bool>;  -- for user messages
-- Message content
DEFINE FIELD content ON TABLE witness_message TYPE option<string>;
DEFINE FIELD attachments ON TABLE witness_message TYPE option<array>;  -- [{ type, url, alt }]
-- AI card metadata
DEFINE FIELD ai_badge ON TABLE witness_message TYPE option<object>;  -- { variant, hint, reasoning }
DEFINE FIELD blocks ON TABLE witness_message TYPE option<array>;  -- Block primitives (list, document, form, computed, vote, reference)
-- System message
DEFINE FIELD subtype ON TABLE witness_message TYPE option<string>;  -- checkpoint_completed, phase_activated, etc.
DEFINE FIELD data ON TABLE witness_message TYPE option<object>;  -- structured data for rendering
-- Diff card
DEFINE FIELD operations ON TABLE witness_message TYPE option<array>;
-- Vote card
DEFINE FIELD options ON TABLE witness_message TYPE option<array>;  -- VoteOption[]
-- Evidence/Galang specific
DEFINE FIELD amount ON TABLE witness_message TYPE option<number>;
DEFINE FIELD currency ON TABLE witness_message TYPE option<string>;
DEFINE FIELD evidence_data ON TABLE witness_message TYPE option<object>;
-- Audit
DEFINE FIELD request_id ON TABLE witness_message TYPE string;
DEFINE FIELD correlation_id ON TABLE witness_message TYPE string;

DEFINE INDEX idx_message_witness_time ON TABLE witness_message FIELDS witness_id, timestamp, message_id;
DEFINE INDEX idx_message_type ON TABLE witness_message FIELDS witness_id, type;

-- ================================================================
-- PLAN: Adaptive Path (tree structure - separate for updates)
-- ================================================================

DEFINE TABLE witness_plan SCHEMAFULL;
DEFINE FIELD plan_id ON TABLE witness_plan TYPE string;
DEFINE FIELD witness_id ON TABLE witness_plan TYPE string;
DEFINE FIELD version ON TABLE witness_plan TYPE number DEFAULT 1;
DEFINE FIELD status ON TABLE witness_plan TYPE string;  -- "draft" | "active" | "completed" | "paused"
DEFINE FIELD phases ON TABLE witness_plan TYPE array;  -- Phase[]
DEFINE FIELD branches ON TABLE witness_plan TYPE array;  -- Branch[]
DEFINE FIELD created_at ON TABLE witness_plan TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE witness_plan TYPE datetime DEFAULT time::now();
DEFINE FIELD request_id ON TABLE witness_plan TYPE string;
DEFINE FIELD correlation_id ON TABLE witness_plan TYPE string;

DEFINE INDEX idx_plan_witness ON TABLE witness_plan FIELDS witness_id UNIQUE;

-- ================================================================
-- ENRICHMENT: Card/Content Metadata
-- ================================================================

DEFINE TABLE witness_card_enrichment SCHEMAFULL;
DEFINE FIELD enrichment_id ON TABLE witness_card_enrichment TYPE string;
DEFINE FIELD message_id ON TABLE witness_card_enrichment TYPE string;
DEFINE FIELD trajectory_type ON TABLE witness_card_enrichment TYPE string;  -- aksi, advokasi, pantau, mufakat, etc.
DEFINE FIELD sentiment ON TABLE witness_card_enrichment TYPE string;  -- positive, neutral, negative
DEFINE FIELD entity_tags ON TABLE witness_card_enrichment TYPE array;  -- EntityTagSuggestion[]
DEFINE FIELD signal_labels ON TABLE witness_card_enrichment TYPE object;
DEFINE FIELD created_at ON TABLE witness_card_enrichment TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_enrichment_message ON TABLE witness_card_enrichment FIELDS message_id UNIQUE;

-- ================================================================
-- GRAPH EDGES: Cross-aggregate relationships
-- ================================================================

-- User → Signal → Witness (content-directed signals)
DEFINE TABLE signal SCHEMAFULL;
DEFINE FIELD signal_id ON TABLE signal TYPE string;
DEFINE FIELD from_user ON TABLE signal TYPE string;
DEFINE FIELD to_witness ON TABLE signal TYPE string;
DEFINE FIELD signal_type ON TABLE signal TYPE string;  -- "bagus" | "vouch" | "skeptis" | "proof_of_resolve"
DEFINE FIELD created_at ON TABLE signal TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_signal_witness ON TABLE signal FIELDS to_witness;
DEFINE INDEX idx_signal_user ON TABLE signal FIELDS from_user;
DEFINE INDEX idx_signal_uniq ON TABLE signal FIELDS from_user, to_witness, signal_type UNIQUE;

-- User → Vouch → User (peer endorsement, linked to skill)
DEFINE TABLE vouch SCHEMAFULL;
DEFINE FIELD vouch_id ON TABLE vouch TYPE string;
DEFINE FIELD from_user ON TABLE vouch TYPE string;
DEFINE FIELD to_user ON TABLE vouch TYPE string;
DEFINE FIELD skill_id ON TABLE vouch TYPE option<string>;
DEFINE FIELD weight_hint ON TABLE vouch TYPE option<string>;  -- "strong" | "moderate" | "light"
DEFINE FIELD message ON TABLE vouch TYPE option<string>;
DEFINE FIELD created_at ON TABLE vouch TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_vouch_to_user ON TABLE vouch FIELDS to_user;
DEFINE INDEX idx_vouch_from_user ON TABLE vouch FIELDS from_user;
DEFINE INDEX idx_vouch_uniq ON TABLE vouch FIELDS from_user, to_user, skill_id UNIQUE;

-- ================================================================
-- POST-APPLY VALIDATION
-- ================================================================

-- Verify all witness tables exist
SELECT count() FROM witness GROUP ALL;
SELECT count() FROM witness_member GROUP ALL;
SELECT count() FROM witness_message GROUP ALL;
SELECT count() FROM witness_plan GROUP ALL;
SELECT count() FROM witness_card_enrichment GROUP ALL;
SELECT count() FROM signal GROUP ALL;
SELECT count() FROM vouch GROUP ALL;

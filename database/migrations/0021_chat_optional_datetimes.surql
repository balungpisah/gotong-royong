-- 0021_chat_optional_datetimes
-- SurrealDB v3 enforces SCHEMAFULL datetime types strictly.
-- These fields are optional in the domain model, so represent them as option<datetime>.

DEFINE FIELD OVERWRITE left_at ON TABLE chat_member TYPE option<datetime>;
DEFINE FIELD OVERWRITE mute_until ON TABLE chat_member TYPE option<datetime>;

DEFINE FIELD OVERWRITE edited_at ON TABLE chat_message TYPE option<datetime>;
DEFINE FIELD OVERWRITE deleted_at ON TABLE chat_message TYPE option<datetime>;

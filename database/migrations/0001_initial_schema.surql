-- 0001_initial_schema
-- Preconditions: none (fresh database)

DEFINE TABLE chat_thread SCHEMAFULL;
DEFINE TABLE chat_member SCHEMAFULL;
DEFINE TABLE chat_message SCHEMAFULL;
DEFINE TABLE chat_read_cursor SCHEMAFULL;
DEFINE TABLE chat_delivery_event SCHEMAFULL;
DEFINE TABLE track_state_transition SCHEMAFULL;

DEFINE FIELD thread_id ON TABLE chat_thread TYPE string;
DEFINE FIELD scope_id ON TABLE chat_thread TYPE string;
DEFINE FIELD created_by ON TABLE chat_thread TYPE string;
DEFINE FIELD privacy_level ON TABLE chat_thread TYPE string;
DEFINE FIELD created_at ON TABLE chat_thread TYPE datetime;
DEFINE FIELD updated_at ON TABLE chat_thread TYPE datetime;

DEFINE FIELD thread_id ON TABLE chat_member TYPE string;
DEFINE FIELD user_id ON TABLE chat_member TYPE string;
DEFINE FIELD role ON TABLE chat_member TYPE string;
DEFINE FIELD joined_at ON TABLE chat_member TYPE datetime;
DEFINE FIELD left_at ON TABLE chat_member TYPE datetime;
DEFINE FIELD mute_until ON TABLE chat_member TYPE datetime;

DEFINE FIELD thread_id ON TABLE chat_message TYPE string;
DEFINE FIELD message_id ON TABLE chat_message TYPE string;
DEFINE FIELD author_id ON TABLE chat_message TYPE string;
DEFINE FIELD body ON TABLE chat_message TYPE string;
DEFINE FIELD attachments ON TABLE chat_message TYPE array;
DEFINE FIELD request_id ON TABLE chat_message TYPE string;
DEFINE FIELD correlation_id ON TABLE chat_message TYPE string;
DEFINE FIELD created_at ON TABLE chat_message TYPE datetime;
DEFINE FIELD edited_at ON TABLE chat_message TYPE datetime;
DEFINE FIELD deleted_at ON TABLE chat_message TYPE datetime;

DEFINE FIELD thread_id ON TABLE chat_read_cursor TYPE string;
DEFINE FIELD user_id ON TABLE chat_read_cursor TYPE string;
DEFINE FIELD last_read_message_id ON TABLE chat_read_cursor TYPE string;
DEFINE FIELD last_read_at ON TABLE chat_read_cursor TYPE datetime;

DEFINE FIELD event_id ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD thread_id ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD message_id ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD event_type ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD request_id ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD correlation_id ON TABLE chat_delivery_event TYPE string;
DEFINE FIELD occurred_at ON TABLE chat_delivery_event TYPE datetime;

DEFINE FIELD transition_id ON TABLE track_state_transition TYPE string;
DEFINE FIELD entity_id ON TABLE track_state_transition TYPE string;
DEFINE FIELD request_id ON TABLE track_state_transition TYPE string;
DEFINE FIELD correlation_id ON TABLE track_state_transition TYPE string;
DEFINE FIELD track ON TABLE track_state_transition TYPE string;
DEFINE FIELD from_stage ON TABLE track_state_transition TYPE string;
DEFINE FIELD to_stage ON TABLE track_state_transition TYPE string;
DEFINE FIELD transition_type ON TABLE track_state_transition TYPE string;
DEFINE FIELD mechanism ON TABLE track_state_transition TYPE string;
DEFINE FIELD actor ON TABLE track_state_transition TYPE object;
DEFINE FIELD occurred_at ON TABLE track_state_transition TYPE datetime;
DEFINE FIELD gate ON TABLE track_state_transition TYPE object;

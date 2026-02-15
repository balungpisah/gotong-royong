-- 0003_permissions_private_channels
-- Baseline permission rules for chat tables.

DEFINE TABLE chat_thread SCHEMAFULL
    PERMISSIONS
        FOR select WHERE privacy_level = "public"
            OR thread_id IN (
                SELECT thread_id FROM chat_member
                WHERE user_id = $auth.id AND left_at IS NONE
            )
        FOR create WHERE created_by = $auth.id
        FOR update WHERE created_by = $auth.id
        FOR delete NONE;

DEFINE TABLE chat_member SCHEMAFULL
    PERMISSIONS
        FOR select WHERE user_id = $auth.id
            OR thread_id IN (
                SELECT thread_id FROM chat_member
                WHERE user_id = $auth.id AND left_at IS NONE
            )
        FOR create WHERE user_id = $auth.id
        FOR update WHERE user_id = $auth.id
        FOR delete NONE;

DEFINE TABLE chat_message SCHEMAFULL
    PERMISSIONS
        FOR select WHERE thread_id IN (
            SELECT thread_id FROM chat_member
            WHERE user_id = $auth.id AND left_at IS NONE
        )
        FOR create WHERE author_id = $auth.id
        FOR update WHERE author_id = $auth.id
        FOR delete NONE;

DEFINE TABLE chat_read_cursor SCHEMAFULL
    PERMISSIONS
        FOR select WHERE user_id = $auth.id
        FOR create WHERE user_id = $auth.id
        FOR update WHERE user_id = $auth.id
        FOR delete WHERE user_id = $auth.id;

DEFINE TABLE chat_delivery_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE thread_id IN (
            SELECT thread_id FROM chat_member
            WHERE user_id = $auth.id AND left_at IS NONE
        )
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

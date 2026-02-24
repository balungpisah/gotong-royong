-- 0019_record_permissions
-- Record-user permissions for SurrealDB ACCESS account (TYPE RECORD).
-- Preconditions: 0001–0018 applied

-- Helper expression (inline): auth user id part derived from $auth.id (record<warga>).
-- We store user ids as raw strings across application tables, so we compare against this:
-- (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))

-- ================================================================
-- Chat (overwrite 0003 with record-id aware comparisons)
-- ================================================================

DEFINE TABLE OVERWRITE chat_thread SCHEMAFULL
    PERMISSIONS
        FOR select WHERE privacy_level = "public"
            OR created_by = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
            OR thread_id IN (
                SELECT thread_id FROM chat_member
                WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
                  AND left_at IS NONE
            )
        FOR create WHERE created_by = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR update WHERE created_by = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR delete NONE;

DEFINE TABLE OVERWRITE chat_member SCHEMAFULL
    PERMISSIONS
        FOR select WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR create WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR update WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR delete NONE;

DEFINE TABLE OVERWRITE chat_message SCHEMAFULL
    PERMISSIONS
        FOR select WHERE thread_id IN (
            SELECT thread_id FROM chat_member
            WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
              AND left_at IS NONE
        )
        FOR create WHERE author_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR update WHERE author_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR delete NONE;

DEFINE TABLE OVERWRITE chat_read_cursor SCHEMAFULL
    PERMISSIONS
        FOR select WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR create WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR update WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
        FOR delete WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id));

DEFINE TABLE OVERWRITE chat_delivery_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE thread_id IN (
            SELECT thread_id FROM chat_member
            WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
              AND left_at IS NONE
        )
        FOR create WHERE thread_id IN (
            SELECT thread_id FROM chat_member
            WHERE user_id = (string::split(type::string($auth.id), ':')[1] ?? type::string($auth.id))
              AND left_at IS NONE
        )
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Core: contributions / evidence / vouches
-- ================================================================

DEFINE TABLE OVERWRITE contribution SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE author_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"];

DEFINE TABLE OVERWRITE evidence SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE actor_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE actor_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete WHERE actor_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"];

DEFINE TABLE OVERWRITE vouch SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE voucher_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE voucher_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete WHERE voucher_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"];

-- ================================================================
-- Vault
-- ================================================================

DEFINE TABLE OVERWRITE vault_entry SCHEMAFULL
    PERMISSIONS
        FOR select WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR string::split(type::string($auth.id), ':')[1] IN wali
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR create WHERE author_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR string::split(type::string($auth.id), ':')[1] IN wali
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"];

DEFINE TABLE OVERWRITE vault_timeline_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE vault_entry_id IN (
            SELECT vault_entry_id FROM vault_entry
            WHERE author_id = string::split(type::string($auth.id), ':')[1]
               OR string::split(type::string($auth.id), ':')[1] IN wali
        )
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR create WHERE vault_entry_id IN (
            SELECT vault_entry_id FROM vault_entry
            WHERE author_id = string::split(type::string($auth.id), ':')[1]
               OR string::split(type::string($auth.id), ':')[1] IN wali
        )
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Discovery read models
-- ================================================================

DEFINE TABLE OVERWRITE discovery_feed_item SCHEMAFULL
    PERMISSIONS
        FOR select WHERE privacy_level = "public"
            OR actor_id = string::split(type::string($auth.id), ':')[1]
            OR string::split(type::string($auth.id), ':')[1] IN participant_ids
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE discovery_notification SCHEMAFULL
    PERMISSIONS
        FOR select WHERE user_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR create NONE
        FOR update WHERE user_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete WHERE user_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"];

-- ================================================================
-- Moderation (platform staff only)
-- ================================================================

DEFINE TABLE OVERWRITE content_moderation SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE moderation_decision SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

-- ================================================================
-- Adaptive path
-- ================================================================

DEFINE TABLE OVERWRITE path_plan SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE author_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE path_plan_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE actor.user_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE plan_suggestion SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE created_by = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE created_by = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE path_branch SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE path_phase SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE path_checkpoint SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Siaga (emergency broadcasts)
-- ================================================================

DEFINE TABLE OVERWRITE siaga_broadcast SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE author_id = string::split(type::string($auth.id), ':')[1]
        FOR update WHERE author_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE siaga_timeline_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE actor.user_id = string::split(type::string($auth.id), ':')[1]
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Webhook outbox (system/internal)
-- ================================================================

DEFINE TABLE OVERWRITE webhook_outbox_event SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE webhook_delivery_log SCHEMAFULL
    PERMISSIONS
        FOR select WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR create NONE
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Ontology graph
-- ================================================================

DEFINE TABLE OVERWRITE concept SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE action SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE place SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth IS NOT NONE
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

-- NOTE: warga permissions are intentionally left unchanged to avoid impacting
-- record access SIGNIN/SIGNUP queries (which must query by email/pass).

DEFINE TABLE OVERWRITE note SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE author = $auth.id
        FOR update WHERE author = $auth.id
            OR $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE measurement SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR delete NONE;

DEFINE TABLE OVERWRITE ABOUT SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth IS NOT NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE LOCATED_AT SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth IS NOT NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE HAS_ACTION SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth IS NOT NONE
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE BROADER SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE INSTANCE_OF SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE $auth.platform_role IN ["admin", "moderator"]
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE VOUCHES SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE in = $auth.id
        FOR update NONE
        FOR delete NONE;

DEFINE TABLE OVERWRITE CHALLENGES SCHEMALESS
    PERMISSIONS
        FOR select WHERE $auth IS NOT NONE
        FOR create WHERE in = $auth.id
        FOR update NONE
        FOR delete NONE;

-- ================================================================
-- Auth internal tables
-- ================================================================

DEFINE TABLE OVERWRITE token SCHEMAFULL
    PERMISSIONS
        FOR select WHERE warga = $auth
        FOR create WHERE warga = $auth
        FOR update WHERE warga = $auth
        FOR delete NONE;

DEFINE TABLE OVERWRITE auth_audit SCHEMAFULL
    PERMISSIONS
        FOR select NONE
        FOR create WHERE $auth IS NOT NONE
        FOR update NONE
        FOR delete NONE;

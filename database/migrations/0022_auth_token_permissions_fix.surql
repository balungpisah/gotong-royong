-- 0022_auth_token_permissions_fix
-- Ensure token/audit tables remain writable by the authenticated record-user session.
-- In SurrealDB record auth, `$auth` is the authenticated record id.

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

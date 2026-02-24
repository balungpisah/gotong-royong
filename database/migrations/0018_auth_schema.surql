-- 0018_auth_schema
-- Native SurrealDB record-user authentication for Gotong Royong.
-- Preconditions: 0001–0017 applied

-- Extend warga to support first-party authentication.
DEFINE FIELD email ON TABLE warga TYPE option<string>
    ASSERT $value IS NONE OR string::is_email($value);
DEFINE FIELD pass ON TABLE warga TYPE option<string>;
DEFINE FIELD platform_role ON TABLE warga TYPE string DEFAULT 'user';
DEFINE FIELD enabled ON TABLE warga TYPE bool DEFAULT true;
DEFINE FIELD created_at ON TABLE warga TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE warga TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_warga_email_unique ON TABLE warga FIELDS email UNIQUE;
DEFINE INDEX idx_warga_username_unique ON TABLE warga FIELDS username UNIQUE;

-- Token audit / revocation store (jti-based).
DEFINE TABLE token SCHEMAFULL;
DEFINE FIELD exp ON TABLE token TYPE int;
DEFINE FIELD revoked ON TABLE token TYPE bool DEFAULT false;
DEFINE FIELD revoked_at ON TABLE token TYPE option<datetime>;
DEFINE FIELD warga ON TABLE token TYPE record<warga>;
DEFINE FIELD created_at ON TABLE token TYPE datetime DEFAULT time::now();
DEFINE FIELD last_seen_at ON TABLE token TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_token_warga ON TABLE token FIELDS warga, created_at;

DEFINE TABLE auth_audit SCHEMAFULL;
DEFINE FIELD token ON TABLE auth_audit TYPE record<token>;
DEFINE FIELD time ON TABLE auth_audit TYPE datetime DEFAULT time::now();
DEFINE FIELD event ON TABLE auth_audit TYPE string DEFAULT 'authenticate';

DEFINE INDEX idx_auth_audit_token ON TABLE auth_audit FIELDS token, time;

-- Record access method used by application users.
-- SurrealDB will issue JWT access tokens (and refresh tokens when supported by the runtime).
DEFINE ACCESS account ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE warga SET
            email = $email,
            username = $username,
            community_id = $community_id,
            pass = crypto::argon2::generate($pass),
            platform_role = 'user',
            enabled = true,
            created_at = time::now(),
            updated_at = time::now()
    )
    SIGNIN (
        SELECT * FROM warga
        WHERE email = $email
          AND crypto::argon2::compare(pass, $pass)
    )
    AUTHENTICATE (
        IF !$auth.enabled THEN THROW "This user is not enabled"
        ELSE (
            IF (type::record("token", $token.jti).revoked ?? false) = true THEN THROW "This token has been revoked"
            ELSE $auth
            END
        )
        END
    )
    DURATION FOR TOKEN 30d, FOR SESSION 1h
;

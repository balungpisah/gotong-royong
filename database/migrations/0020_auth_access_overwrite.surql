-- 0020_auth_access_overwrite
-- Ensure record access definition is in the expected SurrealDB v3 shape.

DEFINE ACCESS OVERWRITE account ON DATABASE TYPE RECORD
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

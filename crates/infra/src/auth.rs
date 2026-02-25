use crate::db::DbConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{AccessToken, Record, RefreshToken, Token};
use surrealdb_types::SurrealValue;

#[derive(Clone)]
pub struct SurrealAuthService {
    config: DbConfig,
    db: Surreal<Client>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthIdentity {
    pub user_id: String,
    pub username: String,
    pub platform_role: String,
}

#[derive(Debug, Clone)]
pub struct SurrealDbSession {
    client: Arc<Surreal<Client>>,
}

impl SurrealDbSession {
    pub fn client(&self) -> Arc<Surreal<Client>> {
        self.client.clone()
    }
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub identity: AuthIdentity,
    pub db_session: SurrealDbSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, surrealdb_types::SurrealValue)]
pub struct SignupParams {
    pub email: String,
    pub pass: String,
    pub username: String,
    pub community_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, surrealdb_types::SurrealValue)]
pub struct SigninParams {
    pub email: String,
    pub pass: String,
}

#[derive(Debug, Deserialize)]
struct WargaRow {
    id: String,
    username: Option<String>,
    platform_role: Option<String>,
}

impl SurrealAuthService {
    pub async fn new(config: DbConfig) -> anyhow::Result<Self> {
        let db = Surreal::<Client>::init();
        db.connect::<Ws>(&config.endpoint)
            .await
            .with_context(|| format!("connect surrealdb endpoint {}", config.endpoint))?;
        Ok(Self { config, db })
    }

    pub async fn signup(&self, params: SignupParams) -> anyhow::Result<(AuthTokens, AuthIdentity)> {
        let session = self.db.clone();
        let token = session
            .signup(Record {
                namespace: self.config.namespace.clone(),
                database: self.config.database.clone(),
                access: "account".to_string(),
                params,
            })
            .await
            .context("surreal signup")?;
        self.identity_from_token(token).await
    }

    pub async fn signin_password(
        &self,
        params: SigninParams,
    ) -> anyhow::Result<(AuthTokens, AuthIdentity)> {
        let session = self.db.clone();
        let token = session
            .signin(Record {
                namespace: self.config.namespace.clone(),
                database: self.config.database.clone(),
                access: "account".to_string(),
                params,
            })
            .await
            .context("surreal signin")?;
        self.identity_from_token(token).await
    }

    pub async fn refresh(
        &self,
        access_token: &str,
        refresh_token: &str,
    ) -> anyhow::Result<(AuthTokens, AuthIdentity)> {
        let session = self.db.clone();
        let token = session
            .authenticate(Token::from((
                AccessToken::from(access_token),
                RefreshToken::from(refresh_token),
            )))
            .refresh()
            .await
            .context("surreal refresh")?;
        self.identity_from_token(token).await
    }

    pub async fn validate(&self, access_token: &str) -> anyhow::Result<AuthSession> {
        let session = self.db.clone();
        session
            .authenticate(access_token.to_string())
            .await
            .context("surreal authenticate")?;
        session
            .use_ns(&self.config.namespace)
            .use_db(&self.config.database)
            .await
            .context("select surrealdb namespace/database")?;

        let jti = decode_jwt_claim(access_token, "jti")
            .and_then(|value| value.as_str().map(str::to_string));
        let exp = decode_jwt_claim(access_token, "exp").and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|v| i64::try_from(v).ok()))
        });

        if let Some(jti) = jti {
            let mut revoked_response = session
                .query(
                    "SELECT revoked FROM token\n\
                     WHERE id = type::record('token', $jti)\n\
                     LIMIT 1",
                )
                .bind(("jti", jti.clone()))
                .await
                .context("select token revoked state")?;
            let revoked_rows: Vec<serde_json::Value> = revoked_response
                .take(0)
                .context("decode token revoked rows")?;
            let revoked = revoked_rows
                .first()
                .and_then(|row| row.get("revoked"))
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if revoked {
                return Err(anyhow::anyhow!("token revoked"));
            }

            if let Some(exp) = exp {
                session
                    .query(
                        "UPSERT type::record('token', $jti) SET\n\
                            exp = $exp,\n\
                            warga = $auth,\n\
                            last_seen_at = time::now();\n\
                         CREATE auth_audit CONTENT { token: type::record('token', $jti), event: 'authenticate' };",
                    )
                    .bind(("jti", jti))
                    .bind(("exp", exp))
                    .await
                    .context("upsert auth token record")?
                    .check()
                    .context("upsert auth token record check")?;
            } else {
                session
                    .query(
                        "UPDATE type::record('token', $jti) SET\n\
                            warga = $auth,\n\
                            last_seen_at = time::now();\n\
                         CREATE auth_audit CONTENT { token: type::record('token', $jti), event: 'authenticate' };",
                    )
                    .bind(("jti", jti))
                    .await
                    .context("update auth token record")?
                    .check()
                    .context("update auth token record check")?;
            }
        }

        let mut response = session
            .query("SELECT type::string(id) AS id, username, platform_role FROM $auth.id")
            .await
            .context("select $auth identity")?;
        let rows: Vec<serde_json::Value> =
            response.take(0).context("decode $auth identity rows")?;
        let row: WargaRow = rows
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .context("decode $auth identity row")?
            .context("missing $auth identity row")?;

        let identity = AuthIdentity {
            user_id: record_id_to_raw(&row.id).to_string(),
            username: row
                .username
                .unwrap_or_else(|| record_id_to_raw(&row.id).to_string()),
            platform_role: row.platform_role.unwrap_or_else(|| "user".to_string()),
        };
        Ok(AuthSession {
            identity,
            db_session: SurrealDbSession {
                client: Arc::new(session),
            },
        })
    }

    pub async fn revoke_access_token(&self, access_token: &str) -> anyhow::Result<()> {
        let jti = decode_jwt_claim(access_token, "jti")
            .and_then(|value| value.as_str().map(str::to_string))
            .context("missing jti claim")?;
        let jti_verify = jti.clone();

        let session = self.db.clone();
        session
            .authenticate(access_token.to_string())
            .await
            .context("surreal authenticate for revoke")?;
        session
            .use_ns(&self.config.namespace)
            .use_db(&self.config.database)
            .await
            .context("select surrealdb namespace/database")?;

        session
            .query(
                "UPSERT type::record('token', $jti) SET\n\
                    revoked=true,\n\
                    revoked_at=time::now(),\n\
                    warga=$auth;",
            )
            .bind(("jti", jti))
            .await
            .context("revoke token")?
            .check()
            .context("revoke token check")?;

        let mut verify = session
            .query(
                "SELECT revoked FROM token\n\
                 WHERE id = type::record('token', $jti)\n\
                 LIMIT 1",
            )
            .bind(("jti", jti_verify))
            .await
            .context("verify revoked state")?;
        let rows: Vec<serde_json::Value> = verify.take(0).context("decode revoked verify rows")?;
        let revoked = rows
            .first()
            .and_then(|row| row.get("revoked"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !revoked {
            return Err(anyhow::anyhow!("failed to revoke token"));
        }
        Ok(())
    }

    async fn identity_from_token(
        &self,
        token: Token,
    ) -> anyhow::Result<(AuthTokens, AuthIdentity)> {
        let access_token = token.access.as_insecure_token().to_string();
        let refresh_token = token
            .refresh
            .as_ref()
            .map(|value| value.as_insecure_token().to_string());

        let session = self.validate(&access_token).await?;
        Ok((
            AuthTokens {
                access_token,
                refresh_token,
            },
            session.identity,
        ))
    }
}

fn record_id_to_raw(value: &str) -> &str {
    value.split_once(':').map(|(_, id)| id).unwrap_or(value)
}

fn decode_jwt_claim(token: &str, claim: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64_url_decode(payload).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get(claim).cloned()
}

fn base64_url_decode(input: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine;

    let mut s = input.to_string();
    let pad = s.len() % 4;
    if pad != 0 {
        s.extend(std::iter::repeat('=').take(4 - pad));
    }
    let engine = base64::engine::general_purpose::URL_SAFE;
    engine.decode(s).context("base64 decode")
}

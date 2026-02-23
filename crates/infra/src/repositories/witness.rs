use std::sync::Arc;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use async_trait::async_trait;
use time::OffsetDateTime;

use crate::db::DbConfig;

/// Frontend contract types — maps to apps/web/src/lib/types/witness.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub witness_id: String,
    pub title: String,
    pub summary: String,
    pub track_hint: Option<String>,
    pub seed_hint: Option<String>,
    pub status: String, // "draft" | "open" | "active" | "resolved" | "closed"
    pub close_reason: Option<String>,
    pub rahasia_level: String, // "L0" | "L1" | "L2" | "L3"
    pub created_at: String, // ISO 8601
    pub updated_at: String,
    pub created_by: String,
    pub member_count: i64,
    pub message_count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessMember {
    pub user_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub role: String, // "pelapor" | "relawan" | "koordinator" | "saksi"
    pub tier: Option<i64>,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: String,
    pub witness_id: String,
    pub r#type: String, // discriminator
    pub timestamp: String,
    pub author: Option<serde_json::Value>,
    pub is_self: Option<bool>,
    pub content: Option<String>,
    pub attachments: Option<Vec<serde_json::Value>>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessDetail {
    #[serde(flatten)]
    pub witness: Witness,
    pub messages: Vec<ChatMessage>,
    pub members: Vec<WitnessMember>,
    pub plan: Option<serde_json::Value>,
    pub blocks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWitnessInput {
    pub title: String,
    pub summary: String,
    pub rahasia_level: String,
    pub track_hint: Option<String>,
    pub seed_hint: Option<String>,
    pub created_by: String,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOptions {
    pub status: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub cursor: Option<String>,
}

/// WitnessRepository — abstracts all witness-related queries
#[async_trait]
pub trait WitnessRepository: Send + Sync {
    /// Create a new witness (returns full detail)
    async fn create(&self, input: CreateWitnessInput) -> Result<WitnessDetail, String>;

    /// Get witness by ID with all related data (messages, members, plan)
    async fn get_detail(&self, witness_id: &str) -> Result<WitnessDetail, String>;

    /// List witnesses with pagination
    async fn list(&self, opts: ListOptions) -> Result<Paginated<Witness>, String>;

    /// Get paginated messages for a witness
    async fn get_messages(
        &self,
        witness_id: &str,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Paginated<ChatMessage>, String>;

    /// Add message to witness (creates new ChatMessage)
    async fn add_message(
        &self,
        witness_id: &str,
        message: ChatMessage,
    ) -> Result<ChatMessage, String>;

    /// Get members for a witness
    async fn get_members(&self, witness_id: &str) -> Result<Vec<WitnessMember>, String>;

    /// Add member to witness
    async fn add_member(
        &self,
        witness_id: &str,
        member: WitnessMember,
    ) -> Result<WitnessMember, String>;

    /// Update witness status
    async fn update_status(
        &self,
        witness_id: &str,
        status: String,
        close_reason: Option<String>,
    ) -> Result<(), String>;
}

/// SurrealWitnessRepository — production implementation
pub struct SurrealWitnessRepository {
    client: Arc<Surreal<Client>>,
}

impl SurrealWitnessRepository {
    pub async fn new(db_config: &DbConfig) -> Result<Self, String> {
        let db = Surreal::<Client>::init();
        db.connect::<surrealdb::engine::remote::ws::Ws>(&db_config.endpoint)
            .await
            .map_err(|e| format!("surreal connect failed: {}", e))?;

        db.signin(surrealdb::opt::auth::Root {
            username: db_config.username.clone(),
            password: db_config.password.clone(),
        })
        .await
        .map_err(|e| format!("surreal signin failed: {}", e))?;

        db.use_ns(&db_config.namespace)
            .use_db(&db_config.database)
            .await
            .map_err(|e| format!("surreal use_ns/use_db failed: {}", e))?;

        Ok(Self {
            client: Arc::new(db),
        })
    }
}

#[async_trait]
impl WitnessRepository for SurrealWitnessRepository {
    async fn create(&self, input: CreateWitnessInput) -> Result<WitnessDetail, String> {
        // Generate witness ID
        let witness_id = format!("witness:{}", uuid::Uuid::new_v4());
        let now = OffsetDateTime::now_utc().to_rfc3339();

        // Insert witness
        let _: Witness = self
            .client
            .create((&witness_id,))
            .content(serde_json::json!({
                "witness_id": witness_id,
                "title": input.title,
                "summary": input.summary,
                "status": "draft",
                "rahasia_level": input.rahasia_level,
                "track_hint": input.track_hint,
                "seed_hint": input.seed_hint,
                "created_by": input.created_by,
                "created_at": now,
                "updated_at": now,
                "member_count": 0,
                "message_count": 0,
                "unread_count": 0,
                "request_id": input.request_id,
                "correlation_id": input.correlation_id,
            }))
            .await
            .map_err(|e| format!("create witness failed: {}", e))?;

        // Fetch and return detail
        self.get_detail(&witness_id).await
    }

    async fn get_detail(&self, witness_id: &str) -> Result<WitnessDetail, String> {
        // Fetch witness
        let witness: Witness = self
            .client
            .select((witness_id,))
            .await
            .map_err(|e| format!("select witness failed: {}", e))?
            .ok_or_else(|| "witness not found".to_string())?;

        // Fetch messages (limit to last 50 for initial load)
        let messages: Vec<ChatMessage> = self
            .client
            .query("SELECT * FROM witness_message WHERE witness_id = $witness_id ORDER BY timestamp DESC LIMIT 50")
            .bind(("witness_id", witness_id))
            .await
            .map_err(|e| format!("select messages failed: {}", e))?
            .take(0)
            .map_err(|e| format!("parse messages failed: {}", e))?;

        // Fetch members
        let members: Vec<WitnessMember> = self
            .client
            .query("SELECT * FROM witness_member WHERE witness_id = $witness_id")
            .bind(("witness_id", witness_id))
            .await
            .map_err(|e| format!("select members failed: {}", e))?
            .take(0)
            .map_err(|e| format!("parse members failed: {}", e))?;

        // Fetch plan (optional)
        let plan: Option<serde_json::Value> = self
            .client
            .query("SELECT * FROM witness_plan WHERE witness_id = $witness_id")
            .bind(("witness_id", witness_id))
            .await
            .map_err(|e| format!("select plan failed: {}", e))?
            .take::<Vec<serde_json::Value>>(0)
            .ok()
            .and_then(|mut v| v.pop());

        Ok(WitnessDetail {
            witness,
            messages,
            members,
            plan,
            blocks: None, // TODO: load from witness_card_enrichment
        })
    }

    async fn list(&self, opts: ListOptions) -> Result<Paginated<Witness>, String> {
        let limit = opts.limit.unwrap_or(20).min(100);
        let mut query = "SELECT * FROM witness".to_string();

        if let Some(status) = &opts.status {
            query.push_str(&format!(" WHERE status = '{}' ", status));
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} ", limit + 1));

        let witnesses: Vec<Witness> = self
            .client
            .query(&query)
            .await
            .map_err(|e| format!("list witnesses failed: {}", e))?
            .take(0)
            .map_err(|e| format!("parse witnesses failed: {}", e))?;

        let has_more = witnesses.len() > limit as usize;
        let items = if has_more {
            witnesses[..limit as usize].to_vec()
        } else {
            witnesses
        };

        let total = self
            .client
            .query("SELECT count() FROM witness")
            .await
            .map_err(|e| format!("count failed: {}", e))?
            .take::<i64>(0)
            .map_err(|e| format!("parse count failed: {}", e))?;

        Ok(Paginated {
            items,
            total,
            cursor: if has_more {
                items.last().map(|w| w.witness_id.clone())
            } else {
                None
            },
        })
    }

    async fn get_messages(
        &self,
        witness_id: &str,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<Paginated<ChatMessage>, String> {
        let limit = limit.min(100);
        let mut query =
            "SELECT * FROM witness_message WHERE witness_id = $witness_id ORDER BY timestamp DESC"
                .to_string();

        if cursor.is_some() {
            query.push_str(" AND timestamp < $cursor_time");
        }

        query.push_str(&format!(" LIMIT {} ", limit + 1));

        let messages: Vec<ChatMessage> = self
            .client
            .query(&query)
            .bind(("witness_id", witness_id))
            .await
            .map_err(|e| format!("select messages failed: {}", e))?
            .take(0)
            .map_err(|e| format!("parse messages failed: {}", e))?;

        let has_more = messages.len() > limit as usize;
        let items = if has_more {
            messages[..limit as usize].to_vec()
        } else {
            messages
        };

        Ok(Paginated {
            items,
            total: 0, // Don't count all for pagination
            cursor: if has_more {
                items.last().map(|m| m.timestamp.clone())
            } else {
                None
            },
        })
    }

    async fn add_message(
        &self,
        witness_id: &str,
        mut message: ChatMessage,
    ) -> Result<ChatMessage, String> {
        message.message_id = format!("msg:{}", uuid::Uuid::new_v4());
        message.witness_id = witness_id.to_string();
        message.timestamp = OffsetDateTime::now_utc().to_rfc3339();

        let _: ChatMessage = self
            .client
            .create((&message.message_id,))
            .content(&message)
            .await
            .map_err(|e| format!("create message failed: {}", e))?;

        Ok(message)
    }

    async fn get_members(&self, witness_id: &str) -> Result<Vec<WitnessMember>, String> {
        self.client
            .query("SELECT * FROM witness_member WHERE witness_id = $witness_id")
            .bind(("witness_id", witness_id))
            .await
            .map_err(|e| format!("select members failed: {}", e))?
            .take(0)
            .map_err(|e| format!("parse members failed: {}", e))
    }

    async fn add_member(
        &self,
        witness_id: &str,
        mut member: WitnessMember,
    ) -> Result<WitnessMember, String> {
        let member_id = format!("member:{}", uuid::Uuid::new_v4());
        member.joined_at = OffsetDateTime::now_utc().to_rfc3339();

        let _: WitnessMember = self
            .client
            .create((&member_id,))
            .content(serde_json::json!({
                "witness_id": witness_id,
                "user_id": member.user_id,
                "name": member.name,
                "avatar_url": member.avatar_url,
                "role": member.role,
                "tier": member.tier,
                "joined_at": member.joined_at,
            }))
            .await
            .map_err(|e| format!("create member failed: {}", e))?;

        Ok(member)
    }

    async fn update_status(
        &self,
        witness_id: &str,
        status: String,
        close_reason: Option<String>,
    ) -> Result<(), String> {
        let now = OffsetDateTime::now_utc().to_rfc3339();

        self.client
            .query("UPDATE $witness SET status = $status, close_reason = $close_reason, updated_at = $now")
            .bind(("witness", witness_id))
            .bind(("status", &status))
            .bind(("close_reason", close_reason))
            .bind(("now", now))
            .await
            .map_err(|e| format!("update status failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_input_serializes() {
        let input = CreateWitnessInput {
            title: "Test".to_string(),
            summary: "Test summary".to_string(),
            rahasia_level: "L0".to_string(),
            track_hint: None,
            seed_hint: None,
            created_by: "user-1".to_string(),
            request_id: "req-1".to_string(),
            correlation_id: "corr-1".to_string(),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test"));
    }
}

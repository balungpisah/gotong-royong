use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::auth::Role;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::{DomainResult, ports::vault::VaultRepository};

const MAX_ATTACHMENT_REFS: usize = 25;
const MAX_PAYLOAD_BYTES: usize = 128_000;
const MAX_PUBLISH_TARGET_LEN: usize = 128;
const MAX_RETENTION_DAYS: i64 = 3_650;
const MAX_WALI_COUNT: usize = 20;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VaultState {
    Draft,
    Sealed,
    Published,
    Revoked,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VaultActorSnapshot {
    pub user_id: String,
    pub username: String,
    pub token_role: String,
    pub is_author: bool,
    pub is_wali: bool,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: i64,
}

impl VaultActorSnapshot {
    pub fn new(
        actor: &ActorIdentity,
        token_role: &Role,
        is_author: bool,
        is_wali: bool,
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        request_ts_ms: i64,
    ) -> Self {
        Self {
            user_id: actor.user_id.clone(),
            username: actor.username.clone(),
            token_role: token_role.as_str().to_string(),
            is_author,
            is_wali,
            request_id: request_id.into(),
            correlation_id: correlation_id.into(),
            request_ts_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VaultEntry {
    pub vault_entry_id: String,
    pub author_id: String,
    pub author_username: String,
    pub state: VaultState,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub sealed_at_ms: Option<i64>,
    pub sealed_hash: Option<String>,
    pub encryption_key_id: Option<String>,
    pub attachment_refs: Vec<String>,
    pub wali: Vec<String>,
    pub payload: Option<serde_json::Value>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<serde_json::Value>,
    pub audit: Option<serde_json::Value>,
    pub request_id: String,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VaultTimelineEventType {
    WitnessDrafted,
    WitnessSealed,
    WitnessTrusteeAdded,
    WitnessTrusteeRemoved,
    WitnessPublished,
    WitnessRevoked,
    WitnessExpired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VaultTimelineEvent {
    pub event_id: String,
    pub vault_entry_id: String,
    pub event_type: VaultTimelineEventType,
    pub actor: VaultActorSnapshot,
    pub request_id: String,
    pub correlation_id: String,
    pub occurred_at_ms: i64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct CreateVaultDraft {
    pub payload: Option<serde_json::Value>,
    pub attachment_refs: Vec<String>,
    pub wali: Vec<String>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<serde_json::Value>,
    pub audit: Option<serde_json::Value>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct UpdateVaultDraft {
    pub payload: Option<serde_json::Value>,
    pub attachment_refs: Option<Vec<String>>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<serde_json::Value>,
    pub audit: Option<serde_json::Value>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct SealVault {
    pub sealed_hash: String,
    pub encryption_key_id: Option<String>,
    pub sealed_payload: Option<serde_json::Value>,
    pub publish_target: Option<String>,
    pub retention_policy: Option<serde_json::Value>,
    pub audit: Option<serde_json::Value>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
    pub sealed_at_ms: Option<i64>,
}

#[derive(Clone)]
pub struct PublishVault {
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct RevokeVault {
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct ExpireVault {
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct AddTrustee {
    pub wali_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct RemoveTrustee {
    pub wali_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub request_ts_ms: Option<i64>,
}

#[derive(Clone)]
pub struct VaultService {
    repository: Arc<dyn VaultRepository>,
}

impl VaultService {
    pub fn new(repository: Arc<dyn VaultRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_draft(
        &self,
        actor: ActorIdentity,
        role: &Role,
        command: CreateVaultDraft,
    ) -> DomainResult<VaultEntry> {
        let command = validate_create_command(command)?;
        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        let entry = VaultEntry {
            vault_entry_id: crate::util::uuid_v7_without_dashes(),
            author_id: actor.user_id.clone(),
            author_username: actor.username.clone(),
            state: VaultState::Draft,
            created_at_ms: now,
            updated_at_ms: now,
            sealed_at_ms: None,
            sealed_hash: None,
            encryption_key_id: None,
            attachment_refs: command.attachment_refs,
            wali: command.wali,
            payload: command.payload,
            publish_target: command.publish_target,
            retention_policy: command.retention_policy,
            audit: command.audit,
            request_id: command.request_id.clone(),
            correlation_id: command.correlation_id.clone(),
        };
        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            &entry.vault_entry_id,
            VaultTimelineEventType::WitnessDrafted,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "source": "create_draft" })),
        );
        match self.repository.create_entry(&entry, &event).await {
            Ok(entry) => Ok(entry),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_actor_request(&actor.user_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }

    pub async fn get(&self, vault_entry_id: &str) -> DomainResult<VaultEntry> {
        self.repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn list_by_author(&self, actor: ActorIdentity) -> DomainResult<Vec<VaultEntry>> {
        self.repository.list_by_author(&actor.user_id).await
    }

    pub async fn list_timeline(
        &self,
        vault_entry_id: &str,
        actor: ActorIdentity,
    ) -> DomainResult<Vec<VaultTimelineEvent>> {
        let entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        if !can_read_vault(&entry, &actor) {
            return Err(DomainError::Forbidden("vault entry is not visible".into()));
        }
        self.repository.list_timeline(vault_entry_id).await
    }

    pub async fn update_draft(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: UpdateVaultDraft,
    ) -> DomainResult<VaultEntry> {
        let command = validate_update_command(command)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_draft_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        if let Some(payload) = command.payload {
            entry.payload = Some(payload);
        }
        if let Some(attachment_refs) = command.attachment_refs {
            entry.attachment_refs = attachment_refs;
        }
        if command.publish_target.is_some() {
            entry.publish_target = command.publish_target;
        }
        if command.retention_policy.is_some() {
            entry.retention_policy = command.retention_policy;
        }
        if command.audit.is_some() {
            entry.audit = command.audit;
        }
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessDrafted,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "source": "update_draft" })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn delete_draft(
        &self,
        actor: ActorIdentity,
        vault_entry_id: &str,
    ) -> DomainResult<bool> {
        let entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_draft_only(&entry)?;
        self.repository.delete_entry(vault_entry_id).await
    }

    pub async fn seal(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: SealVault,
    ) -> DomainResult<VaultEntry> {
        let command = validate_seal_command(command)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_draft_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        entry.state = VaultState::Sealed;
        entry.sealed_hash = Some(command.sealed_hash);
        entry.encryption_key_id = command.encryption_key_id;
        entry.sealed_at_ms = Some(command.sealed_at_ms.unwrap_or(now));
        if command.sealed_payload.is_some() {
            entry.payload = command.sealed_payload;
        }
        if command.publish_target.is_some() {
            entry.publish_target = command.publish_target;
        }
        if command.retention_policy.is_some() {
            entry.retention_policy = command.retention_policy;
        }
        if command.audit.is_some() {
            entry.audit = command.audit;
        }
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessSealed,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({
                "source": "seal",
                "sealed_hash": entry.sealed_hash,
                "encryption_key_id": entry.encryption_key_id,
            })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn publish(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: PublishVault,
    ) -> DomainResult<VaultEntry> {
        validate_request_context(&command.request_id, &command.correlation_id)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_sealed_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        entry.state = VaultState::Published;
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessPublished,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "source": "publish" })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn revoke(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: RevokeVault,
    ) -> DomainResult<VaultEntry> {
        validate_request_context(&command.request_id, &command.correlation_id)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        authorize_actor_for_revoke(&entry, &actor)?;
        ensure_sealed_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        entry.state = VaultState::Revoked;
        entry.payload = None;
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            entry.author_id == actor.user_id,
            entry.wali.contains(&actor.user_id),
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessRevoked,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "source": "revoke" })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn expire(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: ExpireVault,
    ) -> DomainResult<VaultEntry> {
        validate_request_context(&command.request_id, &command.correlation_id)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_sealed_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        entry.state = VaultState::Expired;
        entry.payload = None;
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessExpired,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "source": "expire" })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn add_trustee(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: AddTrustee,
    ) -> DomainResult<VaultEntry> {
        let command = validate_wali_command(command)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_draft_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        if entry.wali.contains(&command.wali_id) {
            return Err(DomainError::Validation("wali already added".into()));
        }
        if entry.wali.len() >= MAX_WALI_COUNT {
            return Err(DomainError::Validation(format!(
                "maximum wali count is {MAX_WALI_COUNT}"
            )));
        }
        entry.wali.push(command.wali_id.clone());
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessTrusteeAdded,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "wali_id": command.wali_id })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    pub async fn remove_trustee(
        &self,
        actor: ActorIdentity,
        role: &Role,
        vault_entry_id: &str,
        command: RemoveTrustee,
    ) -> DomainResult<VaultEntry> {
        let command = validate_remove_trustee_command(command)?;
        let mut entry = self
            .repository
            .get_entry(vault_entry_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        ensure_actor_is_author(&entry, &actor)?;
        ensure_draft_only(&entry)?;

        let now = command.request_ts_ms.unwrap_or_else(now_ms);
        let index = entry
            .wali
            .iter()
            .position(|item| item == &command.wali_id)
            .ok_or(DomainError::NotFound)?;
        entry.wali.remove(index);
        entry.updated_at_ms = now;
        entry.request_id = command.request_id.clone();
        entry.correlation_id = command.correlation_id.clone();

        let snapshot = VaultActorSnapshot::new(
            &actor,
            role,
            true,
            false,
            command.request_id,
            command.correlation_id,
            now,
        );
        let event = make_event(
            vault_entry_id,
            VaultTimelineEventType::WitnessTrusteeRemoved,
            snapshot,
            entry.request_id.clone(),
            entry.correlation_id.clone(),
            now,
            Some(serde_json::json!({ "wali_id": command.wali_id })),
        );
        self.update_with_idempotency(&entry, event).await
    }

    async fn update_with_idempotency(
        &self,
        entry: &VaultEntry,
        event: VaultTimelineEvent,
    ) -> DomainResult<VaultEntry> {
        match self.repository.update_entry(entry, &event).await {
            Ok(entry) => Ok(entry),
            Err(DomainError::Conflict) => self
                .repository
                .get_by_request(&entry.vault_entry_id, &event.request_id)
                .await?
                .ok_or(DomainError::Conflict),
            Err(err) => Err(err),
        }
    }
}

fn validate_request_context(request_id: &str, correlation_id: &str) -> DomainResult<()> {
    if request_id.trim().is_empty() {
        return Err(DomainError::Validation("request_id is required".into()));
    }
    if correlation_id.trim().is_empty() {
        return Err(DomainError::Validation("correlation_id is required".into()));
    }
    Ok(())
}

fn validate_create_command(mut command: CreateVaultDraft) -> DomainResult<CreateVaultDraft> {
    validate_request_context(&command.request_id, &command.correlation_id)?;
    if let Some(payload) = command.payload.as_ref() {
        ensure_json_not_large(payload, MAX_PAYLOAD_BYTES, "payload")?;
    }
    command.attachment_refs = dedupe_and_trim(command.attachment_refs);
    if command.attachment_refs.len() > MAX_ATTACHMENT_REFS {
        return Err(DomainError::Validation(format!(
            "attachment_refs exceeds max of {MAX_ATTACHMENT_REFS}"
        )));
    }
    command.wali = dedupe_and_trim(command.wali);
    if command.wali.len() > MAX_WALI_COUNT {
        return Err(DomainError::Validation(format!(
            "wali exceeds max of {MAX_WALI_COUNT}"
        )));
    }
    if let Some(target) = normalize_optional(command.publish_target.take()) {
        ensure_len_lte(&target, MAX_PUBLISH_TARGET_LEN, "publish_target")?;
        command.publish_target = Some(target);
    }
    if let Some(policy) = command.retention_policy.take() {
        validate_retention_policy(&policy)?;
        command.retention_policy = Some(policy);
    }
    Ok(command)
}

fn validate_update_command(mut command: UpdateVaultDraft) -> DomainResult<UpdateVaultDraft> {
    validate_request_context(&command.request_id, &command.correlation_id)?;
    if command.payload.is_none()
        && command.attachment_refs.is_none()
        && command.publish_target.is_none()
        && command.retention_policy.is_none()
        && command.audit.is_none()
    {
        return Err(DomainError::Validation(
            "at least one field is required for update".into(),
        ));
    }
    if let Some(payload) = command.payload.as_ref() {
        ensure_json_not_large(payload, MAX_PAYLOAD_BYTES, "payload")?;
    }
    if let Some(attachment_refs) = command.attachment_refs.take() {
        command.attachment_refs = Some(dedupe_and_trim(attachment_refs));
        if command
            .attachment_refs
            .as_ref()
            .is_some_and(|value| value.len() > MAX_ATTACHMENT_REFS)
        {
            return Err(DomainError::Validation(format!(
                "attachment_refs exceeds max of {MAX_ATTACHMENT_REFS}"
            )));
        }
    }
    if let Some(target) = normalize_optional(command.publish_target.take()) {
        ensure_len_lte(&target, MAX_PUBLISH_TARGET_LEN, "publish_target")?;
        command.publish_target = Some(target);
    }
    if let Some(policy) = command.retention_policy.take() {
        validate_retention_policy(&policy)?;
        command.retention_policy = Some(policy);
    }
    Ok(command)
}

fn validate_seal_command(mut command: SealVault) -> DomainResult<SealVault> {
    validate_request_context(&command.request_id, &command.correlation_id)?;
    let sealed_hash = command.sealed_hash.trim().to_string();
    if sealed_hash.is_empty() {
        return Err(DomainError::Validation(
            "sealed_hash cannot be empty".into(),
        ));
    }
    command.sealed_hash = sealed_hash;
    if let Some(payload) = command.sealed_payload.as_ref() {
        ensure_json_not_large(payload, MAX_PAYLOAD_BYTES, "sealed_payload")?;
    }
    if let Some(target) = command.publish_target.take() {
        let target = normalize_optional(Some(target))
            .ok_or_else(|| DomainError::Validation("publish_target cannot be blank".into()))?;
        ensure_len_lte(&target, MAX_PUBLISH_TARGET_LEN, "publish_target")?;
        command.publish_target = Some(target);
    }
    if let Some(key) = command.encryption_key_id.as_ref() {
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(DomainError::Validation(
                "encryption_key_id cannot be empty".into(),
            ));
        }
        command.encryption_key_id = Some(key);
    }
    if let Some(policy) = command.retention_policy.take() {
        validate_retention_policy(&policy)?;
        command.retention_policy = Some(policy);
    }
    Ok(command)
}

fn validate_wali_command(mut command: AddTrustee) -> DomainResult<AddTrustee> {
    validate_request_context(&command.request_id, &command.correlation_id)?;
    let wali_id = command.wali_id.trim().to_string();
    if wali_id.is_empty() {
        return Err(DomainError::Validation("wali_id cannot be empty".into()));
    }
    command.wali_id = wali_id;
    Ok(command)
}

fn validate_retention_policy(policy: &serde_json::Value) -> DomainResult<()> {
    if let Some(days) = policy.get("days").and_then(serde_json::Value::as_i64) {
        if days <= 0 || days > MAX_RETENTION_DAYS {
            return Err(DomainError::Validation(format!(
                "retention_policy.days must be between 1 and {MAX_RETENTION_DAYS}"
            )));
        }
    }
    Ok(())
}

fn validate_remove_trustee_command(command: RemoveTrustee) -> DomainResult<RemoveTrustee> {
    let command = AddTrustee {
        wali_id: command.wali_id,
        request_id: command.request_id,
        correlation_id: command.correlation_id,
        request_ts_ms: command.request_ts_ms,
    };
    let validated = validate_wali_command(command)?;
    Ok(RemoveTrustee {
        wali_id: validated.wali_id,
        request_id: validated.request_id,
        correlation_id: validated.correlation_id,
        request_ts_ms: validated.request_ts_ms,
    })
}

fn ensure_json_not_large(
    value: &serde_json::Value,
    max_bytes: usize,
    field: &str,
) -> DomainResult<()> {
    if value.to_string().len() > max_bytes {
        return Err(DomainError::Validation(format!(
            "{field} exceeds max length of {max_bytes}"
        )));
    }
    Ok(())
}

fn ensure_len_lte(value: &str, max: usize, field: &str) -> DomainResult<()> {
    if value.len() > max {
        return Err(DomainError::Validation(format!(
            "{field} exceeds max length of {max}"
        )));
    }
    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn dedupe_and_trim(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::with_capacity(values.len());
    let mut seen = HashSet::new();
    for value in values {
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.clone()) {
            result.push(value);
        }
    }
    result
}

fn ensure_actor_is_author(entry: &VaultEntry, actor: &ActorIdentity) -> DomainResult<()> {
    if entry.author_id != actor.user_id {
        return Err(DomainError::Forbidden(
            "only the author can perform this action".into(),
        ));
    }
    Ok(())
}

fn can_read_vault(entry: &VaultEntry, actor: &ActorIdentity) -> bool {
    entry.author_id == actor.user_id || entry.wali.contains(&actor.user_id)
}

fn authorize_actor_for_revoke(entry: &VaultEntry, actor: &ActorIdentity) -> DomainResult<()> {
    if entry.author_id == actor.user_id || entry.wali.contains(&actor.user_id) {
        return Ok(());
    }
    Err(DomainError::Forbidden(
        "only author or trustee can revoke this entry".into(),
    ))
}

fn ensure_draft_only(entry: &VaultEntry) -> DomainResult<()> {
    if !matches!(entry.state, VaultState::Draft) {
        return Err(DomainError::Validation("entry must be in draft".into()));
    }
    Ok(())
}

fn ensure_sealed_only(entry: &VaultEntry) -> DomainResult<()> {
    if !matches!(entry.state, VaultState::Sealed) {
        return Err(DomainError::Validation(
            "entry must be sealed to perform this action".into(),
        ));
    }
    Ok(())
}

fn make_event(
    vault_entry_id: &str,
    event_type: VaultTimelineEventType,
    actor: VaultActorSnapshot,
    request_id: String,
    correlation_id: String,
    occurred_at_ms: i64,
    metadata: Option<serde_json::Value>,
) -> VaultTimelineEvent {
    VaultTimelineEvent {
        event_id: crate::util::uuid_v7_without_dashes(),
        vault_entry_id: vault_entry_id.to_string(),
        event_type,
        actor,
        request_id,
        correlation_id,
        occurred_at_ms,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::BoxFuture;
    use crate::ports::vault::VaultRepository;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MockVaultRepository {
        by_id: Arc<RwLock<HashMap<String, VaultEntry>>>,
        by_actor_request: Arc<RwLock<HashMap<(String, String), String>>>,
        by_entry_request: Arc<RwLock<HashMap<(String, String), String>>>,
        timeline: Arc<RwLock<HashMap<String, Vec<VaultTimelineEvent>>>>,
    }

    impl MockVaultRepository {
        fn actor_request_key(actor_id: &str, request_id: &str) -> (String, String) {
            (actor_id.to_string(), request_id.to_string())
        }

        fn entry_request_key(vault_entry_id: &str, request_id: &str) -> (String, String) {
            (vault_entry_id.to_string(), request_id.to_string())
        }
    }

    impl VaultRepository for MockVaultRepository {
        fn create_entry(
            &self,
            entry: &VaultEntry,
            event: &VaultTimelineEvent,
        ) -> BoxFuture<'_, DomainResult<VaultEntry>> {
            let entry = entry.clone();
            let event = event.clone();
            let by_id = self.by_id.clone();
            let by_actor_request = self.by_actor_request.clone();
            let by_entry_request = self.by_entry_request.clone();
            let timeline = self.timeline.clone();
            Box::pin(async move {
                if let Some(existing_id) = by_actor_request
                    .read()
                    .await
                    .get(&Self::actor_request_key(
                        &entry.author_id,
                        &event.request_id,
                    ))
                    .cloned()
                {
                    let items = by_id.read().await;
                    return items
                        .get(&existing_id)
                        .cloned()
                        .ok_or(DomainError::Conflict);
                }

                let mut by_id = by_id.write().await;
                if by_id.contains_key(&entry.vault_entry_id) {
                    return Err(DomainError::Conflict);
                }

                by_id.insert(entry.vault_entry_id.clone(), entry.clone());
                by_actor_request.write().await.insert(
                    Self::actor_request_key(&entry.author_id, &event.request_id),
                    entry.vault_entry_id.clone(),
                );
                by_entry_request.write().await.insert(
                    Self::entry_request_key(&entry.vault_entry_id, &event.request_id),
                    entry.vault_entry_id.clone(),
                );
                timeline
                    .write()
                    .await
                    .entry(entry.vault_entry_id.clone())
                    .or_default()
                    .push(event);
                Ok(entry)
            })
        }

        fn update_entry(
            &self,
            entry: &VaultEntry,
            event: &VaultTimelineEvent,
        ) -> BoxFuture<'_, DomainResult<VaultEntry>> {
            let entry = entry.clone();
            let event = event.clone();
            let by_id = self.by_id.clone();
            let by_entry_request = self.by_entry_request.clone();
            let timeline = self.timeline.clone();

            Box::pin(async move {
                if let Some(stored_id) = by_entry_request
                    .read()
                    .await
                    .get(&Self::entry_request_key(
                        &entry.vault_entry_id,
                        &event.request_id,
                    ))
                    .cloned()
                {
                    let items = by_id.read().await;
                    if stored_id == entry.vault_entry_id {
                        return items.get(&stored_id).cloned().ok_or(DomainError::Conflict);
                    }
                    return Err(DomainError::Conflict);
                }

                let mut by_id = by_id.write().await;
                if !by_id.contains_key(&entry.vault_entry_id) {
                    return Err(DomainError::NotFound);
                }
                by_id.insert(entry.vault_entry_id.clone(), entry.clone());
                by_entry_request.write().await.insert(
                    Self::entry_request_key(&entry.vault_entry_id, &event.request_id),
                    entry.vault_entry_id.clone(),
                );
                timeline
                    .write()
                    .await
                    .entry(entry.vault_entry_id.clone())
                    .or_default()
                    .push(event);
                Ok(entry)
            })
        }

        fn delete_entry(&self, vault_entry_id: &str) -> BoxFuture<'_, DomainResult<bool>> {
            let vault_entry_id = vault_entry_id.to_string();
            let by_id = self.by_id.clone();
            let by_actor_request = self.by_actor_request.clone();
            let by_entry_request = self.by_entry_request.clone();
            let timeline = self.timeline.clone();
            Box::pin(async move {
                let was_removed = by_id.write().await.remove(&vault_entry_id).is_some();
                if !was_removed {
                    return Ok(false);
                }
                by_actor_request
                    .write()
                    .await
                    .retain(|_, value| value != &vault_entry_id);
                by_entry_request
                    .write()
                    .await
                    .retain(|_, value| value != &vault_entry_id);
                timeline.write().await.remove(&vault_entry_id);
                Ok(true)
            })
        }

        fn get_entry(
            &self,
            vault_entry_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
            let vault_entry_id = vault_entry_id.to_string();
            let by_id = self.by_id.clone();
            Box::pin(async move {
                let items = by_id.read().await;
                Ok(items.get(&vault_entry_id).cloned())
            })
        }

        fn list_by_author(&self, author_id: &str) -> BoxFuture<'_, DomainResult<Vec<VaultEntry>>> {
            let author_id = author_id.to_string();
            let by_id = self.by_id.clone();
            Box::pin(async move {
                let mut entries: Vec<_> = by_id
                    .read()
                    .await
                    .values()
                    .filter(|entry| entry.author_id == author_id)
                    .cloned()
                    .collect();
                entries.sort_by(|left, right| {
                    right
                        .created_at_ms
                        .cmp(&left.created_at_ms)
                        .then_with(|| right.vault_entry_id.cmp(&left.vault_entry_id))
                });
                Ok(entries)
            })
        }

        fn list_timeline(
            &self,
            vault_entry_id: &str,
        ) -> BoxFuture<'_, DomainResult<Vec<VaultTimelineEvent>>> {
            let vault_entry_id = vault_entry_id.to_string();
            let timeline = self.timeline.clone();
            Box::pin(async move {
                let mut timeline = timeline
                    .read()
                    .await
                    .get(&vault_entry_id)
                    .cloned()
                    .unwrap_or_default();
                timeline.sort_by(|left, right| {
                    left.occurred_at_ms
                        .cmp(&right.occurred_at_ms)
                        .then_with(|| left.event_id.cmp(&right.event_id))
                });
                Ok(timeline)
            })
        }

        fn get_by_actor_request(
            &self,
            actor_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
            let actor_id = actor_id.to_string();
            let request_id = request_id.to_string();
            let by_actor_request = self.by_actor_request.clone();
            let by_id = self.by_id.clone();
            Box::pin(async move {
                let by_actor_request = by_actor_request.read().await;
                let Some(vault_entry_id) = by_actor_request.get(&(actor_id, request_id)) else {
                    return Ok(None);
                };
                let items = by_id.read().await;
                Ok(items.get(vault_entry_id).cloned())
            })
        }

        fn get_by_request(
            &self,
            vault_entry_id: &str,
            request_id: &str,
        ) -> BoxFuture<'_, DomainResult<Option<VaultEntry>>> {
            let vault_entry_id = vault_entry_id.to_string();
            let request_id = request_id.to_string();
            let by_entry_request = self.by_entry_request.clone();
            let by_id = self.by_id.clone();
            Box::pin(async move {
                let by_entry_request = by_entry_request.read().await;
                let Some(stored_id) = by_entry_request.get(&(vault_entry_id, request_id)) else {
                    return Ok(None);
                };
                let items = by_id.read().await;
                Ok(items.get(stored_id).cloned())
            })
        }
    }

    fn actor() -> ActorIdentity {
        ActorIdentity {
            user_id: "user-1".to_string(),
            username: "Alice".to_string(),
        }
    }

    fn service() -> VaultService {
        VaultService::new(std::sync::Arc::new(MockVaultRepository::default()))
    }

    #[tokio::test]
    async fn create_draft_and_publish_flow_works() {
        let service = service();
        let role = Role::User;
        let created = service
            .create_draft(
                actor(),
                &role,
                CreateVaultDraft {
                    payload: Some(serde_json::json!({"note": "witness"})),
                    attachment_refs: vec!["att-1".to_string()],
                    wali: vec!["wali-1".to_string()],
                    publish_target: None,
                    retention_policy: Some(serde_json::json!({"days": 7})),
                    audit: Some(serde_json::json!({"source": "test"})),
                    request_id: "req-1".to_string(),
                    correlation_id: "corr-1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");
        let _updated = service
            .update_draft(
                actor(),
                &role,
                &created.vault_entry_id,
                UpdateVaultDraft {
                    payload: None,
                    attachment_refs: Some(vec!["att-1".to_string(), "att-2".to_string()]),
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-2".to_string(),
                    correlation_id: "corr-2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("updated");
        let sealed = service
            .seal(
                actor(),
                &role,
                &created.vault_entry_id,
                SealVault {
                    sealed_hash: "hash-1".to_string(),
                    encryption_key_id: Some("kms-key".to_string()),
                    sealed_payload: Some(serde_json::json!({"note": "sealed"})),
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-3".to_string(),
                    correlation_id: "corr-3".to_string(),
                    request_ts_ms: Some(3),
                    sealed_at_ms: Some(3),
                },
            )
            .await
            .expect("sealed");
        let published = service
            .publish(
                actor(),
                &role,
                &created.vault_entry_id,
                PublishVault {
                    request_id: "req-4".to_string(),
                    correlation_id: "corr-4".to_string(),
                    request_ts_ms: Some(4),
                },
            )
            .await
            .expect("published");
        assert!(matches!(published.state, VaultState::Published));
        assert_eq!(sealed.request_id, "req-3");
        assert_eq!(sealed.correlation_id, "corr-3");
        assert_eq!(sealed.payload, Some(serde_json::json!({"note": "sealed"})));
    }

    #[tokio::test]
    async fn sealed_entry_cannot_be_updated() {
        let service = service();
        let role = Role::User;
        let created = service
            .create_draft(
                actor(),
                &role,
                CreateVaultDraft {
                    payload: Some(serde_json::json!({"note": "witness"})),
                    attachment_refs: vec![],
                    wali: vec![],
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "r-seal".to_string(),
                    correlation_id: "c-seal".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");
        let _ = service
            .seal(
                actor(),
                &role,
                &created.vault_entry_id,
                SealVault {
                    sealed_hash: "hash-1".to_string(),
                    encryption_key_id: None,
                    sealed_payload: None,
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "r-seal-2".to_string(),
                    correlation_id: "c-seal-2".to_string(),
                    request_ts_ms: Some(2),
                    sealed_at_ms: Some(2),
                },
            )
            .await
            .expect("sealed");
        let result = service
            .update_draft(
                actor(),
                &role,
                &created.vault_entry_id,
                UpdateVaultDraft {
                    payload: Some(serde_json::json!({"note": "bad"})),
                    attachment_refs: None,
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "r-seal-3".to_string(),
                    correlation_id: "c-seal-3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn revoke_keeps_sealed_metadata_and_clears_payload() {
        let service = service();
        let role = Role::User;
        let created = service
            .create_draft(
                actor(),
                &role,
                CreateVaultDraft {
                    payload: Some(serde_json::json!({"note": "confidential"})),
                    attachment_refs: vec!["att-1".to_string()],
                    wali: vec!["wali-1".to_string()],
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-r1".to_string(),
                    correlation_id: "corr-r1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");
        let sealed = service
            .seal(
                actor(),
                &role,
                &created.vault_entry_id,
                SealVault {
                    sealed_hash: "hash-revoked".to_string(),
                    encryption_key_id: Some("kms-key".to_string()),
                    sealed_payload: Some(serde_json::json!({"note": "sealed"})),
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-r2".to_string(),
                    correlation_id: "corr-r2".to_string(),
                    request_ts_ms: Some(2),
                    sealed_at_ms: Some(2),
                },
            )
            .await
            .expect("sealed");
        let revoked = service
            .revoke(
                actor(),
                &role,
                &created.vault_entry_id,
                RevokeVault {
                    request_id: "req-r3".to_string(),
                    correlation_id: "corr-r3".to_string(),
                    request_ts_ms: Some(3),
                },
            )
            .await
            .expect("revoked");
        assert!(matches!(revoked.state, VaultState::Revoked));
        assert!(revoked.payload.is_none());
        assert_eq!(revoked.sealed_hash, sealed.sealed_hash);
    }

    #[tokio::test]
    async fn trustee_can_revoke_and_duplicate_wali_is_rejected() {
        let service = service();
        let author = actor();
        let wali_actor = ActorIdentity {
            user_id: "wali-2".to_string(),
            username: "Wali".to_string(),
        };
        let created = service
            .create_draft(
                author.clone(),
                &Role::User,
                CreateVaultDraft {
                    payload: Some(serde_json::json!({"note": "sensitive"})),
                    attachment_refs: vec![],
                    wali: vec!["wali-1".to_string()],
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-w1".to_string(),
                    correlation_id: "corr-w1".to_string(),
                    request_ts_ms: Some(1),
                },
            )
            .await
            .expect("created");

        let _ = service
            .add_trustee(
                author.clone(),
                &Role::User,
                &created.vault_entry_id,
                AddTrustee {
                    wali_id: "wali-2".to_string(),
                    request_id: "req-w2".to_string(),
                    correlation_id: "corr-w2".to_string(),
                    request_ts_ms: Some(2),
                },
            )
            .await
            .expect("add wali");

        let _ = service
            .seal(
                actor(),
                &Role::User,
                &created.vault_entry_id,
                SealVault {
                    sealed_hash: "hash-wali".to_string(),
                    encryption_key_id: Some("kms-key".to_string()),
                    sealed_payload: None,
                    publish_target: None,
                    retention_policy: None,
                    audit: None,
                    request_id: "req-w3".to_string(),
                    correlation_id: "corr-w3".to_string(),
                    request_ts_ms: Some(3),
                    sealed_at_ms: Some(3),
                },
            )
            .await
            .expect("sealed");

        let revoked = service
            .revoke(
                wali_actor,
                &Role::User,
                &created.vault_entry_id,
                RevokeVault {
                    request_id: "req-w4".to_string(),
                    correlation_id: "corr-w4".to_string(),
                    request_ts_ms: Some(4),
                },
            )
            .await
            .expect("revoked by wali");
        assert!(matches!(revoked.state, VaultState::Revoked));
    }
}

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::DomainResult;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::vouches::VouchRepository;

const MAX_MESSAGE_LENGTH: usize = 500;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VouchWeightHint {
    Strong,
    Moderate,
    Light,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vouch {
    pub vouch_id: String,
    pub voucher_id: String,
    pub voucher_username: String,
    pub vouchee_id: String,
    pub skill_id: Option<String>,
    pub weight_hint: Option<VouchWeightHint>,
    pub message: Option<String>,
    pub request_id: String,
    pub correlation_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug)]
pub struct VouchCreate {
    pub vouchee_id: String,
    pub skill_id: Option<String>,
    pub weight_hint: Option<VouchWeightHint>,
    pub message: Option<String>,
}

#[derive(Clone)]
pub struct VouchService {
    repository: Arc<dyn VouchRepository>,
}

impl VouchService {
    pub fn new(repository: Arc<dyn VouchRepository>) -> Self {
        Self { repository }
    }

    pub async fn submit(
        &self,
        actor: ActorIdentity,
        request_id: String,
        correlation_id: String,
        input: VouchCreate,
    ) -> DomainResult<Vouch> {
        let payload = validate_vouch_create(&input)?;
        let now = now_ms();
        let vouch = Vouch {
            vouch_id: crate::util::uuid_v7_without_dashes(),
            voucher_id: actor.user_id,
            voucher_username: actor.username,
            vouchee_id: payload.vouchee_id,
            skill_id: payload.skill_id,
            weight_hint: payload.weight_hint,
            message: payload.message,
            request_id,
            correlation_id,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.repository.create(&vouch).await
    }

    pub fn into_tandang_event_payload(vouch: &Vouch) -> serde_json::Value {
        serde_json::json!({
            "event_type": "vouch_submitted",
            "actor": {
                "user_id": vouch.voucher_id,
                "username": vouch.voucher_username,
            },
            "subject": {
                "vouchee_id": vouch.vouchee_id,
                "skill_id": vouch.skill_id,
                "weight_hint": vouch_weight_hint_to_str(&vouch.weight_hint),
                "message": vouch.message,
            },
            "event_id": vouch_to_event_id(&vouch.vouch_id),
            "schema_version": "1",
            "request_id": vouch.request_id,
            "timestamp": format_rfc3339(vouch.created_at_ms),
        })
    }

    pub async fn list_by_vouchee(&self, vouchee_id: &str) -> DomainResult<Vec<Vouch>> {
        self.repository.list_by_vouchee(vouchee_id).await
    }

    pub async fn list_by_voucher(&self, voucher_id: &str) -> DomainResult<Vec<Vouch>> {
        self.repository.list_by_voucher(voucher_id).await
    }
}

fn vouch_to_event_id(vouch_id: &str) -> String {
    let hex = vouch_id
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .collect::<String>();
    let short = hex.chars().take(16).collect::<String>();
    let short = if short.len() < 16 {
        format!("{}{}", short, "0".repeat(16 - short.len()))
    } else {
        short
    };
    format!("evt_{short}")
}

fn vouch_weight_hint_to_str(value: &Option<VouchWeightHint>) -> Option<&'static str> {
    value.as_ref().map(|hint| match hint {
        VouchWeightHint::Strong => "strong",
        VouchWeightHint::Moderate => "moderate",
        VouchWeightHint::Light => "light",
    })
}

fn format_rfc3339(epoch_ms: i64) -> String {
    crate::util::format_ms_rfc3339(epoch_ms)
}

fn validate_vouch_create(input: &VouchCreate) -> Result<VouchCreate, DomainError> {
    let vouchee_id = input.vouchee_id.trim().to_string();
    if vouchee_id.is_empty() {
        return Err(DomainError::Validation("vouchee_id is required".into()));
    }

    if let Some(skill_id) = &input.skill_id {
        if skill_id.trim().is_empty() {
            return Err(DomainError::Validation("skill_id cannot be empty".into()));
        }
    }

    if let Some(message) = &input.message {
        if message.chars().count() > MAX_MESSAGE_LENGTH {
            return Err(DomainError::Validation(format!(
                "message exceeds max length of {MAX_MESSAGE_LENGTH}"
            )));
        }
    }

    Ok(VouchCreate {
        vouchee_id,
        skill_id: input.skill_id.clone(),
        weight_hint: input.weight_hint.clone(),
        message: input.message.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_vouchee_id() {
        let input = VouchCreate {
            vouchee_id: " ".to_string(),
            skill_id: None,
            weight_hint: None,
            message: None,
        };
        assert!(validate_vouch_create(&input).is_err());
    }

    #[test]
    fn message_length_is_validated() {
        let long_message = "x".repeat(501);
        let input = VouchCreate {
            vouchee_id: "target-user".to_string(),
            skill_id: None,
            weight_hint: None,
            message: Some(long_message),
        };
        assert!(validate_vouch_create(&input).is_err());
    }

    #[test]
    fn tandang_payload_contains_schema_version_and_request_id() {
        let vouch = Vouch {
            vouch_id: "018f9b2cd4f1aa11bbee223344556677".to_string(),
            voucher_id: "user-123".to_string(),
            voucher_username: "user-123-name".to_string(),
            vouchee_id: "user-456".to_string(),
            skill_id: Some("skill-1".to_string()),
            weight_hint: Some(VouchWeightHint::Strong),
            message: Some("great work".to_string()),
            request_id: "req-vouch-123".to_string(),
            correlation_id: "corr-vouch-123".to_string(),
            created_at_ms: 1_739_750_400_000,
            updated_at_ms: 1_739_750_400_000,
        };
        let payload = VouchService::into_tandang_event_payload(&vouch);
        assert_eq!(
            payload
                .get("schema_version")
                .and_then(|value| value.as_str()),
            Some("1")
        );
        assert_eq!(
            payload.get("request_id").and_then(|value| value.as_str()),
            Some("req-vouch-123")
        );
    }
}

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::DomainError;
use crate::jobs::now_ms;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookOutboxStatus {
    Pending,
    InFlight,
    Delivered,
    Retrying,
    DeadLetter,
}

impl WebhookOutboxStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InFlight => "in_flight",
            Self::Delivered => "delivered",
            Self::Retrying => "retrying",
            Self::DeadLetter => "dead_letter",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Delivered | Self::DeadLetter)
    }
}

impl FromStr for WebhookOutboxStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "in_flight" => Ok(Self::InFlight),
            "delivered" => Ok(Self::Delivered),
            "retrying" => Ok(Self::Retrying),
            "dead_letter" => Ok(Self::DeadLetter),
            _ => Err("unknown webhook status"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDeliveryResult {
    Success,
    RetryableFailure,
    TerminalFailure,
}

impl WebhookDeliveryResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::RetryableFailure => "retryable_failure",
            Self::TerminalFailure => "terminal_failure",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromStr for WebhookDeliveryResult {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "success" => Ok(Self::Success),
            "retryable_failure" => Ok(Self::RetryableFailure),
            "terminal_failure" => Ok(Self::TerminalFailure),
            _ => Err("unknown webhook delivery result"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookOutboxEvent {
    pub event_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub actor_id: String,
    pub actor_username: String,
    pub request_id: String,
    pub correlation_id: String,
    pub status: WebhookOutboxStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub next_attempt_at_ms: Option<i64>,
    pub last_status_code: Option<u16>,
    pub last_error: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookOutboxUpdate {
    pub status: WebhookOutboxStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub next_attempt_at_ms: Option<i64>,
    pub last_status_code: Option<u16>,
    pub last_error: Option<String>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl WebhookOutboxEvent {
    pub fn new(
        payload: serde_json::Value,
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
        max_attempts: u32,
    ) -> Result<Self, DomainError> {
        let event_id = payload
            .get("event_id")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| DomainError::Validation("missing event_id in webhook payload".into()))?
            .to_string();

        let event_type = payload
            .get("event_type")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| DomainError::Validation("missing event_type in webhook payload".into()))?
            .to_string();

        let _schema_version = payload
            .get("schema_version")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                DomainError::Validation("missing schema_version in webhook payload".into())
            })?
            .to_string();

        let payload_request_id = payload
            .get("request_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| DomainError::Validation("missing request_id in webhook payload".into()))?
            .to_string();

        let actor_id = payload
            .get("actor")
            .and_then(|value| value.get("user_id"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("")
            .to_string();

        if actor_id.is_empty() {
            return Err(DomainError::Validation(
                "missing actor.user_id in webhook payload".into(),
            ));
        }

        let actor_username = payload
            .get("actor")
            .and_then(|value| value.get("username"))
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&actor_id)
            .to_string();

        let request_id = request_id.into();
        let correlation_id = correlation_id.into();
        if request_id.is_empty() || correlation_id.is_empty() {
            return Err(DomainError::Validation(
                "request_id and correlation_id are required".into(),
            ));
        }
        if payload_request_id != request_id {
            return Err(DomainError::Validation(
                "payload request_id must match webhook request_id".into(),
            ));
        }

        if max_attempts == 0 {
            return Err(DomainError::Validation(
                "max_attempts must be greater than zero".into(),
            ));
        }

        let now_ms = now_ms();
        Ok(Self {
            event_id,
            event_type,
            payload,
            actor_id,
            actor_username,
            request_id,
            correlation_id,
            status: WebhookOutboxStatus::Pending,
            attempts: 0,
            max_attempts,
            next_attempt_at_ms: None,
            last_status_code: None,
            last_error: None,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookOutboxListQuery {
    pub status: Option<WebhookOutboxStatus>,
    pub limit: usize,
}

impl Default for WebhookOutboxListQuery {
    fn default() -> Self {
        Self {
            status: None,
            limit: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookDeliveryLog {
    pub log_id: String,
    pub event_id: String,
    pub attempt: u32,
    pub outcome: WebhookDeliveryResult,
    pub status_code: Option<u16>,
    pub request_id: String,
    pub correlation_id: String,
    pub request_body_sha256: String,
    pub response_body_sha256: Option<String>,
    pub error_message: Option<String>,
    pub created_at_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_payload() -> serde_json::Value {
        json!({
            "event_id": "evt_a1b2c3d4e5f67890",
            "event_type": "contribution_created",
            "schema_version": "1",
            "request_id": "req-1",
            "actor": {
                "user_id": "user-123",
                "username": "user-123-name"
            },
            "subject": {
                "contribution_type": "task_completion",
                "title": "test"
            }
        })
    }

    #[test]
    fn webhook_outbox_event_requires_schema_version() {
        let mut payload = valid_payload();
        payload
            .as_object_mut()
            .expect("payload object")
            .remove("schema_version");
        let err = WebhookOutboxEvent::new(payload, "req-1", "corr-1", 3).expect_err("error");
        assert!(
            matches!(err, DomainError::Validation(message) if message.contains("schema_version"))
        );
    }

    #[test]
    fn webhook_outbox_event_requires_payload_request_id() {
        let mut payload = valid_payload();
        payload
            .as_object_mut()
            .expect("payload object")
            .remove("request_id");
        let err = WebhookOutboxEvent::new(payload, "req-1", "corr-1", 3).expect_err("error");
        assert!(matches!(err, DomainError::Validation(message) if message.contains("request_id")));
    }

    #[test]
    fn webhook_outbox_event_rejects_mismatched_payload_request_id() {
        let payload = valid_payload();
        let err = WebhookOutboxEvent::new(payload, "req-2", "corr-1", 3).expect_err("error");
        assert!(matches!(err, DomainError::Validation(message) if message.contains("must match")));
    }

    #[test]
    fn webhook_outbox_event_accepts_valid_schema_and_request_id() {
        let payload = valid_payload();
        let event = WebhookOutboxEvent::new(payload, "req-1", "corr-1", 3).expect("event");
        assert_eq!(event.request_id, "req-1");
    }
}

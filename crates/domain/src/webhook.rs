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

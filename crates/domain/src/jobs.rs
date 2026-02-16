use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ports::jobs::{JobEnvelope, JobType};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JobPayload {
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModerationAutoReleasePayload {
    pub content_id: String,
    pub hold_decision_request_id: String,
    pub request_id: String,
    pub correlation_id: String,
    pub scheduled_ms: i64,
    pub request_ts_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookRetryPayload {
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TTLCleanupPayload {
    pub scheduled_ms: i64,
    pub cutoff_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConceptVerificationPayload {
    pub qid: String,
    pub scheduled_ms: i64,
}

#[derive(Clone, Debug)]
pub struct JobDefaults {
    pub max_attempts: u32,
}

impl Default for JobDefaults {
    fn default() -> Self {
        Self { max_attempts: 5 }
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn backoff_ms(base_ms: u64, attempt: u32, max_ms: u64) -> u64 {
    if attempt == 0 {
        return 0;
    }
    let pow = 2u64.saturating_pow(attempt.saturating_sub(1));
    let delay = base_ms.saturating_mul(pow);
    delay.min(max_ms)
}

pub fn new_job(
    job_id: String,
    job_type: JobType,
    payload: serde_json::Value,
    request_id: String,
    correlation_id: String,
    defaults: JobDefaults,
) -> JobEnvelope {
    let now = now_ms();
    JobEnvelope {
        job_id,
        job_type,
        payload,
        request_id,
        correlation_id,
        attempt: 1,
        max_attempts: defaults.max_attempts,
        run_at_ms: now,
        created_at_ms: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn backoff_ms_returns_zero_for_zero_attempt() {
        assert_eq!(backoff_ms(1_000, 0, 60_000), 0);
    }

    #[test]
    fn backoff_ms_grows_geometrically() {
        assert_eq!(backoff_ms(1_000, 1, 60_000), 1_000);
        assert_eq!(backoff_ms(1_000, 2, 60_000), 2_000);
        assert_eq!(backoff_ms(1_000, 3, 60_000), 4_000);
    }

    #[test]
    fn backoff_ms_caps_at_maximum() {
        assert_eq!(backoff_ms(1_000, 10, 3_000), 3_000);
    }

    #[test]
    fn new_job_populates_payload_and_retries() {
        let job = new_job(
            "job-1".to_string(),
            JobType::WebhookRetry,
            json!({"event_id":"evt-1"}),
            "req-1".to_string(),
            "corr-1".to_string(),
            JobDefaults { max_attempts: 9 },
        );
        assert_eq!(job.job_id, "job-1");
        assert_eq!(job.attempt, 1);
        assert_eq!(job.max_attempts, 9);
        assert!(job.created_at_ms >= 0);
        assert_eq!(job.created_at_ms, job.run_at_ms);
        assert_eq!(job.payload, json!({"event_id":"evt-1"}));
    }
}

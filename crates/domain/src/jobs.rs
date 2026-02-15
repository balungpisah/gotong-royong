use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ports::jobs::{JobEnvelope, JobType};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JobPayload {
    pub data: serde_json::Value,
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

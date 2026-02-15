use crate::error::DomainError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

pub fn uuid_v7_without_dashes() -> String {
    Uuid::now_v7().simple().to_string()
}

pub fn format_ms_rfc3339(epoch_ms: i64) -> String {
    let fallback = OffsetDateTime::from_unix_timestamp(0).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let value =
        OffsetDateTime::from_unix_timestamp_nanos(epoch_ms as i128 * 1_000_000).unwrap_or(fallback);
    value
        .format(&Rfc3339)
        .unwrap_or("1970-01-01T00:00:00Z".to_string())
}

pub fn immutable_event_hash<T>(value: &T) -> crate::DomainResult<String>
where
    T: Serialize,
{
    let payload = serde_json::to_vec(value).map_err(|err| {
        DomainError::Validation(format!("failed to serialize audit payload: {err}"))
    })?;
    let digest = Sha256::digest(&payload);
    Ok(hex::encode(digest))
}

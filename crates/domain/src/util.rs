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

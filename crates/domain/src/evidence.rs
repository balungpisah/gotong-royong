use std::sync::Arc;

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

use crate::DomainResult;
use crate::error::DomainError;
use crate::identity::ActorIdentity;
use crate::jobs::now_ms;
use crate::ports::evidence::EvidenceRepository;

const MAX_EVIDENCE_AGE_DAYS: i64 = 30;
const MIN_MEDIA_HASH_LENGTH: usize = 32;
const MAX_PROOF_WITNESSES: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    PhotoWithTimestamp,
    GpsVerification,
    WitnessAttestation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub evidence_id: String,
    pub contribution_id: String,
    pub actor_id: String,
    pub actor_username: String,
    pub evidence_type: EvidenceType,
    pub evidence_data: serde_json::Value,
    pub proof: serde_json::Value,
    pub request_id: String,
    pub correlation_id: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug)]
pub struct EvidenceCreate {
    pub contribution_id: String,
    pub evidence_type: EvidenceType,
    pub evidence_data: serde_json::Value,
    pub proof: serde_json::Value,
}

#[derive(Clone)]
pub struct EvidenceService {
    repository: Arc<dyn EvidenceRepository>,
}

impl EvidenceService {
    pub fn new(repository: Arc<dyn EvidenceRepository>) -> Self {
        Self { repository }
    }

    pub async fn submit(
        &self,
        actor: ActorIdentity,
        request_id: String,
        correlation_id: String,
        input: EvidenceCreate,
    ) -> DomainResult<Evidence> {
        let payload = validate_evidence_create(&input)?;
        let now = now_ms();
        let evidence = Evidence {
            evidence_id: crate::util::uuid_v7_without_dashes(),
            contribution_id: payload.contribution_id,
            actor_id: actor.user_id,
            actor_username: actor.username,
            evidence_type: payload.evidence_type,
            evidence_data: payload.evidence_data,
            proof: payload.proof,
            request_id,
            correlation_id,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.repository.create(&evidence).await
    }

    pub async fn get(&self, evidence_id: &str) -> DomainResult<Evidence> {
        self.repository
            .get(evidence_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn list_by_contribution(&self, contribution_id: &str) -> DomainResult<Vec<Evidence>> {
        self.repository.list_by_contribution(contribution_id).await
    }

    pub fn into_tandang_event_payload(evidence: &Evidence) -> serde_json::Value {
        serde_json::json!({
            "event_type": "por_evidence",
            "actor": {
                "user_id": evidence.actor_id,
                "username": evidence.actor_username,
            },
            "subject": {
                "contribution_id": evidence.contribution_id,
                "evidence_type": to_evidence_type_name(&evidence.evidence_type),
                "evidence_data": evidence.evidence_data,
            },
            "proof": evidence.proof,
            "event_id": evidence_to_event_id(&evidence.evidence_id),
            "timestamp": format_rfc3339(evidence.created_at_ms),
        })
    }
}

fn evidence_to_event_id(evidence_id: &str) -> String {
    let hex = evidence_id
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

fn to_evidence_type_name(value: &EvidenceType) -> &'static str {
    match value {
        EvidenceType::PhotoWithTimestamp => "photo_with_timestamp",
        EvidenceType::GpsVerification => "gps_verification",
        EvidenceType::WitnessAttestation => "witness_attestation",
    }
}

fn format_rfc3339(epoch_ms: i64) -> String {
    crate::util::format_ms_rfc3339(epoch_ms)
}

fn validate_evidence_create(input: &EvidenceCreate) -> Result<EvidenceCreate, DomainError> {
    let contribution_id = input.contribution_id.trim().to_string();
    if contribution_id.is_empty() {
        return Err(DomainError::Validation(
            "contribution_id is required".into(),
        ));
    }

    if !input.evidence_data.is_object() {
        return Err(DomainError::Validation(
            "evidence_data must be an object".into(),
        ));
    }

    if !input.proof.is_object() {
        return Err(DomainError::Validation("proof must be an object".into()));
    }

    match &input.evidence_type {
        EvidenceType::PhotoWithTimestamp => validate_photo_evidence(input)?,
        EvidenceType::GpsVerification => validate_gps_evidence(input)?,
        EvidenceType::WitnessAttestation => validate_witness_evidence(input)?,
    }

    Ok(EvidenceCreate {
        contribution_id,
        evidence_type: input.evidence_type.clone(),
        evidence_data: input.evidence_data.clone(),
        proof: input.proof.clone(),
    })
}

fn validate_photo_evidence(input: &EvidenceCreate) -> Result<(), DomainError> {
    let timestamp = extract_str(input, "timestamp")?;
    let media_hash = extract_optional_str(input, "media_hash")?;
    if media_hash.is_none() {
        return Err(DomainError::Validation(
            "proof.media_hash is required".into(),
        ));
    }
    let media_hash = media_hash.unwrap_or_default();
    if media_hash.len() < MIN_MEDIA_HASH_LENGTH
        || !media_hash.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err(DomainError::Validation(
            "proof.media_hash must be a hex string (at least 32 characters)".into(),
        ));
    }
    validate_timestamp_freshness(&timestamp)?;
    Ok(())
}

fn validate_gps_evidence(input: &EvidenceCreate) -> Result<(), DomainError> {
    let timestamp = extract_str(input, "timestamp")?;
    let (lat, lon) = evidence_location(input)?;
    if !(-90.0..=90.0).contains(&lat) {
        return Err(DomainError::Validation(
            "proof.location.lat must be between -90 and 90".into(),
        ));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(DomainError::Validation(
            "proof.location.lon must be between -180 and 180".into(),
        ));
    }
    validate_timestamp_freshness(&timestamp)?;
    Ok(())
}

fn validate_witness_evidence(input: &EvidenceCreate) -> Result<(), DomainError> {
    let timestamp = extract_str(input, "timestamp")?;
    let witnesses = extract_optional_witness_array(input)?;
    if witnesses.is_none() || witnesses.as_ref().is_some_and(|w| w.is_empty()) {
        return Err(DomainError::Validation(
            "proof.witnesses must contain at least one witness".into(),
        ));
    }
    if let Some(witnesses) = witnesses {
        if witnesses.len() > MAX_PROOF_WITNESSES {
            return Err(DomainError::Validation(
                "proof.witnesses exceeds max of 50".into(),
            ));
        }
    }
    validate_timestamp_freshness(&timestamp)?;
    Ok(())
}

fn extract_str(input: &EvidenceCreate, field: &str) -> Result<String, DomainError> {
    let value = input.proof.get(field).and_then(|v| v.as_str());
    let value = value
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| DomainError::Validation(format!("proof.{field} is required")))?;
    Ok(value.to_string())
}

fn extract_optional_str(
    input: &EvidenceCreate,
    field: &str,
) -> Result<Option<String>, DomainError> {
    Ok(input
        .proof
        .get(field)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string()))
}

fn extract_optional_witness_array(
    input: &EvidenceCreate,
) -> Result<Option<Vec<serde_json::Value>>, DomainError> {
    let value = input.proof.get("witnesses");
    let Some(value) = value else {
        return Ok(None);
    };
    let arr = value
        .as_array()
        .ok_or_else(|| DomainError::Validation("proof.witnesses must be an array".into()))?;
    Ok(Some(arr.clone()))
}

fn evidence_location(input: &EvidenceCreate) -> Result<(f64, f64), DomainError> {
    let lat = input
        .proof
        .get("location")
        .and_then(|value| value.get("lat"))
        .and_then(|value| value.as_f64())
        .ok_or_else(|| DomainError::Validation("proof.location.lat is required".into()))?;
    let lon = input
        .proof
        .get("location")
        .and_then(|value| value.get("lon"))
        .and_then(|value| value.as_f64())
        .ok_or_else(|| DomainError::Validation("proof.location.lon is required".into()))?;
    Ok((lat, lon))
}

fn validate_timestamp_freshness(timestamp: &str) -> Result<(), DomainError> {
    let parsed = OffsetDateTime::parse(timestamp, &Rfc3339)
        .map_err(|_| DomainError::Validation("proof.timestamp must be RFC3339".into()))?;
    let now = OffsetDateTime::now_utc();
    if parsed > now {
        return Err(DomainError::Validation(
            "proof.timestamp cannot be in the future".into(),
        ));
    }
    let age = now - parsed;
    if age > Duration::days(MAX_EVIDENCE_AGE_DAYS) {
        return Err(DomainError::Validation(format!(
            "proof.timestamp is too old (max {MAX_EVIDENCE_AGE_DAYS} days)"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    fn witness_input() -> serde_json::Value {
        serde_json::json!({
            "timestamp": timestamp_today_utc(),
            "witnesses": [
                { "witness_name": "alice" }
            ]
        })
    }

    fn timestamp_today_utc() -> String {
        OffsetDateTime::now_utc()
            .saturating_sub(Duration::days(1))
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
    }

    #[test]
    fn witness_requires_one_witness() {
        let input = EvidenceCreate {
            contribution_id: "contrib-1".to_string(),
            evidence_type: EvidenceType::WitnessAttestation,
            evidence_data: serde_json::json!({ "notes": "ok" }),
            proof: serde_json::json!({"timestamp": timestamp_today_utc(), "witnesses": []}),
        };
        let result = validate_evidence_create(&input);
        assert!(result.is_err());
    }

    #[test]
    fn photo_evidence_requires_hash() {
        let input = EvidenceCreate {
            contribution_id: "contrib-1".to_string(),
            evidence_type: EvidenceType::PhotoWithTimestamp,
            evidence_data: serde_json::json!({ "photo_url": "https://example.test/photo.jpg" }),
            proof: serde_json::json!({"timestamp": timestamp_today_utc()}),
        };
        let result = validate_evidence_create(&input);
        assert!(result.is_err());
    }

    #[test]
    fn witness_payload_is_accepted_when_valid() {
        let input = EvidenceCreate {
            contribution_id: "contrib-1".to_string(),
            evidence_type: EvidenceType::WitnessAttestation,
            evidence_data: serde_json::json!({ "notes": "ok" }),
            proof: witness_input(),
        };
        let result = validate_evidence_create(&input);
        assert!(result.is_ok());
    }
}

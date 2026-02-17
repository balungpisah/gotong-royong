use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::{Json, response::Response};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use validator::Validate;

use crate::error::ApiError;
use crate::middleware::AuthContext;
use crate::state::AppState;
use crate::validation;
use gotong_domain::idempotency::BeginOutcome;
use gotong_domain::ports::idempotency::{IdempotencyKey, IdempotencyResponse};
use gotong_infra::markov_client::CachedJson;

use super::{actor_identity, correlation_id_from_headers, request_id_from_headers, to_response};
use crate::observability;

const EDGE_POD_RESULT_VERSION: &str = "v0.2.0";
const EDGE_POD_PAYLOAD_VERSION: &str = "2026-02-14";
const EDGE_POD_LOOKBACK_MIN_HOURS: i64 = 1;
const EDGE_POD_VALID_ROLES: [&str; 7] = [
    "member",
    "admin",
    "moderator",
    "humas",
    "bendahara",
    "pic",
    "system",
];
const EDGE_POD_GEOPOINT_MIN_LAT: f64 = -90.0;
const EDGE_POD_GEOPOINT_MAX_LAT: f64 = 90.0;
const EDGE_POD_GEOPOINT_MIN_LNG: f64 = -180.0;
const EDGE_POD_GEOPOINT_MAX_LNG: f64 = 180.0;

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct EdgePodActor {
    #[validate(length(min = 1))]
    pub user_id: String,
    #[validate(length(min = 1))]
    pub platform_user_id: String,
    #[validate(length(min = 1))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct EdgePodGeoPoint {
    pub lat: f64,
    pub lng: f64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Validate)]
pub(crate) struct EdgePodBaseRequest {
    #[validate(length(min = 1))]
    pub request_id: String,
    #[validate(length(min = 4))]
    pub correlation_id: String,
    #[validate(nested)]
    pub actor: EdgePodActor,
    #[validate(length(min = 1))]
    pub trigger: String,
    #[validate(length(min = 1))]
    pub payload_version: String,
    pub session_id: Option<String>,
    pub privacy_level: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct EdgePodDuplicateRequest {
    #[serde(flatten)]
    pub base: EdgePodBaseRequest,
    pub seed_text: String,
    pub media_hashes: Option<Vec<String>>,
    pub embedding: Option<Vec<f64>>,
    pub location: Option<EdgePodGeoPoint>,
    pub radius_km: Option<f64>,
    pub scope: Option<String>,
    pub exclude_seed_ids: Option<Vec<String>>,
    pub query_options: Option<Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct EdgePodGamingRequest {
    #[serde(flatten)]
    pub base: EdgePodBaseRequest,
    pub query_users: Vec<String>,
    pub lookback_hours: i64,
    pub platform: String,
    pub seed_ids: Option<Vec<String>>,
    pub focus_metric: Option<String>,
    pub window_start: Option<String>,
    pub window_end: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct EdgePodSensitiveMediaRequest {
    #[serde(flatten)]
    pub base: EdgePodBaseRequest,
    pub media_urls: Vec<String>,
    pub media_types: Vec<String>,
    pub seed_id: String,
    pub author_id: String,
    pub seed_text: String,
    pub hash_chain: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct EdgePodCreditRequest {
    #[serde(flatten)]
    pub base: EdgePodBaseRequest,
    pub user_id: String,
    pub timeline_events: Vec<Value>,
    pub skill_profile: Vec<String>,
    pub contrib_events: Option<Vec<Value>>,
    pub reputation_snapshot: Option<Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct EdgePodSiagaRequest {
    #[serde(flatten)]
    pub base: EdgePodBaseRequest,
    pub text: String,
    pub location: EdgePodGeoPoint,
    pub confidence_bundle: Option<Value>,
    pub reported_urgency: String,
    pub community_scope: String,
    pub current_track: String,
}

#[derive(Debug, Serialize)]
struct EdgePodDuplicateMatch {
    pub seed_id: String,
    pub similarity: f64,
    pub distance_km: f64,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
struct EdgePodDuplicateOutput {
    pub matches: Vec<EdgePodDuplicateMatch>,
    pub top_match: String,
    pub confidence: f64,
    pub auto_block: bool,
}

#[derive(Debug, Serialize)]
struct EdgePodGamingFlag {
    pub user_id: String,
    pub metric: String,
    pub flag: bool,
    pub reason_code: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
struct EdgePodGamingSummary {
    pub total_flags: usize,
    pub critical_count: usize,
}

#[derive(Debug, Serialize)]
struct EdgePodGamingOutput {
    pub flags: Vec<EdgePodGamingFlag>,
    pub summary: EdgePodGamingSummary,
    pub recommendation: String,
}

#[derive(Debug, Serialize)]
struct EdgePodSensitiveScan {
    pub media_url: String,
    pub detections: Vec<String>,
    pub severity: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
struct EdgePodSensitiveOutput {
    pub scans: Vec<EdgePodSensitiveScan>,
    pub overall_safety: String,
    pub redacted_media_url: String,
    pub summary: String,
    pub is_actionable: bool,
}

#[derive(Debug, Serialize)]
struct EdgePodCreditAllocation {
    pub candidate_id: String,
    #[serde(rename = "type")]
    pub allocation_type: String,
    pub weight: f64,
}

#[derive(Debug, Serialize)]
struct EdgePodCreditOutput {
    pub candidate_allocations: Vec<EdgePodCreditAllocation>,
    pub confidence: f64,
    pub reasoning: String,
    pub dispute_window: String,
    pub confidence_source: String,
}

#[derive(Debug, Serialize)]
struct EdgePodSiagaResponderPayload {
    pub channels: Vec<String>,
    pub template: String,
    pub estimated_reach: Option<i64>,
}

#[derive(Debug, Serialize)]
struct EdgePodSiagaOutput {
    pub is_siaga: bool,
    pub severity: String,
    pub responder_payload: EdgePodSiagaResponderPayload,
    pub scope: String,
    pub timeline_window: String,
    pub override_policy: String,
    pub confidence: f64,
}

#[derive(Serialize)]
struct EdgePodSuccessEnvelope<T: Serialize> {
    pub request_id: String,
    pub result_version: String,
    pub output: T,
    pub confidence: f64,
    pub reason_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_context: Option<Value>,
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_003_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

fn validate_base_request(base: &EdgePodBaseRequest) -> Result<(), ApiError> {
    validation::validate(base)?;
    validation::validate(&base.actor)?;
    if !base.request_id.starts_with("req_") || base.request_id.len() < 12 {
        return Err(ApiError::Validation("invalid request_id format".into()));
    }
    if !base
        .request_id
        .chars()
        .skip(4)
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(ApiError::Validation("invalid request_id format".into()));
    }
    if base.payload_version != EDGE_POD_PAYLOAD_VERSION {
        return Err(ApiError::Validation(format!(
            "unsupported payload_version {0}",
            base.payload_version
        )));
    }

    if base.trigger.as_str() != "user_action"
        && base.trigger.as_str() != "timer"
        && base.trigger.as_str() != "webhook"
        && base.trigger.as_str() != "async_batch"
    {
        return Err(ApiError::Validation("invalid trigger".into()));
    }

    if let Some(privacy_level) = &base.privacy_level {
        if privacy_level != "open"
            && privacy_level != "l1"
            && privacy_level != "l2"
            && privacy_level != "vault"
        {
            return Err(ApiError::Validation("invalid privacy_level".into()));
        }
    }

    if base.actor.user_id.is_empty() || base.actor.platform_user_id.is_empty() {
        return Err(ApiError::Validation("actor identity is required".into()));
    }
    if !EDGE_POD_VALID_ROLES.contains(&base.actor.role.as_str()) {
        return Err(ApiError::Validation("invalid actor role".into()));
    }

    Ok(())
}

fn validate_gaming_focus_metric(value: &str) -> bool {
    matches!(
        value,
        "posting_rate" | "vouch_rate" | "revenue" | "dispute" | "generic"
    )
}

fn validate_geopoint(location: &EdgePodGeoPoint) -> Result<(), ApiError> {
    if !(EDGE_POD_GEOPOINT_MIN_LAT..=EDGE_POD_GEOPOINT_MAX_LAT).contains(&location.lat)
        || !(EDGE_POD_GEOPOINT_MIN_LNG..=EDGE_POD_GEOPOINT_MAX_LNG).contains(&location.lng)
    {
        return Err(ApiError::Validation("invalid location".into()));
    }
    Ok(())
}

fn idempotent_response<T: Serialize>(
    request_id: &str,
    result_code: StatusCode,
    payload: T,
    confidence: f64,
    reason_code: &str,
    actor_context: Option<Value>,
) -> Result<IdempotencyResponse, ApiError> {
    let envelope = EdgePodSuccessEnvelope {
        request_id: request_id.to_string(),
        result_version: EDGE_POD_RESULT_VERSION.to_string(),
        output: payload,
        confidence,
        reason_code: reason_code.to_string(),
        actor_context,
    };
    let body = serde_json::to_value(envelope).map_err(|_| ApiError::Internal)?;
    Ok(IdempotencyResponse {
        status_code: result_code.as_u16(),
        body,
    })
}

fn should_fallback_from_request(request_id: &str, text: &str) -> bool {
    request_id.contains("fallback") || text.to_lowercase().contains("fallback")
}

fn duplicate_output(payload: &EdgePodDuplicateRequest) -> EdgePodDuplicateOutput {
    if should_fallback_from_request(&payload.base.request_id, &payload.seed_text) {
        return EdgePodDuplicateOutput {
            matches: vec![],
            top_match: "none".to_string(),
            confidence: 0.00,
            auto_block: false,
        };
    }

    let seed = format!("{}{}", payload.base.request_id, payload.seed_text);
    let signature = stable_hash(&seed);
    let score = ((signature % 10_000) as f64) / 10_000.0;
    let similarity = 0.15 + score * 0.85;
    let auto_block = similarity > 0.82;
    let recommendation = if auto_block {
        "block"
    } else if similarity > 0.55 {
        "warn"
    } else {
        "allow"
    }
    .to_string();

    let match_count = if recommendation == "allow" { 0 } else { 1 };
    let mut matches = Vec::with_capacity(match_count);
    if match_count > 0 {
        let seed_id = format!("seed-{}", signature % 1_000_000);
        matches.push(EdgePodDuplicateMatch {
            seed_id,
            similarity,
            distance_km: (1.0 - similarity) * 10.0,
            recommendation,
        });
    }

    let top_match = matches
        .first()
        .map(|first| first.seed_id.clone())
        .unwrap_or_else(|| "none".to_string());

    EdgePodDuplicateOutput {
        matches,
        top_match,
        confidence: (0.45 + similarity * 0.5).clamp(0.45, 0.99),
        auto_block,
    }
}

fn gaming_output(
    payload: &EdgePodGamingRequest,
    privileged_requester: bool,
) -> EdgePodGamingOutput {
    if should_fallback_from_request(&payload.base.request_id, &payload.platform) {
        return EdgePodGamingOutput {
            flags: vec![],
            summary: EdgePodGamingSummary {
                total_flags: 0,
                critical_count: 0,
            },
            recommendation: "none".to_string(),
        };
    }

    let focus_metric = payload
        .focus_metric
        .clone()
        .unwrap_or_else(|| "generic".to_string());
    let mut total_flags = 0_usize;
    let mut critical_count = 0_usize;
    let mut flags = Vec::with_capacity(payload.query_users.len());

    for (idx, user_id) in payload.query_users.iter().enumerate() {
        let signature = stable_hash(&format!(
            "{}{}{}",
            payload.base.request_id, user_id, focus_metric
        ));
        let score = ((signature % 1000) as f64) / 1000.0;
        let flag = score > 0.55 || privileged_requester;
        let severity = if score > 0.85 {
            "high"
        } else if score > 0.55 {
            "medium"
        } else {
            "low"
        };
        if flag {
            total_flags += 1;
            if severity == "high" {
                critical_count += 1;
            }
            flags.push(EdgePodGamingFlag {
                user_id: user_id.clone(),
                metric: focus_metric.clone(),
                flag,
                reason_code: match severity {
                    "high" => "spike_pattern",
                    _ => "behavioral_outlier",
                }
                .to_string(),
                severity: severity.to_string(),
            });
        }
        if idx > 6 {
            break;
        }
    }

    let recommendation = if critical_count > 1 {
        "suspend_actions"
    } else if total_flags > 0 {
        "manual_review"
    } else {
        "none"
    };
    EdgePodGamingOutput {
        flags,
        summary: EdgePodGamingSummary {
            total_flags,
            critical_count,
        },
        recommendation: recommendation.to_string(),
    }
}

fn sensitive_media_output(
    payload: &EdgePodSensitiveMediaRequest,
) -> (EdgePodSensitiveOutput, String) {
    if should_fallback_from_request(&payload.base.request_id, &payload.seed_text) {
        let scans = payload
            .media_urls
            .iter()
            .map(|media_url| EdgePodSensitiveScan {
                media_url: media_url.clone(),
                detections: vec!["manual_review_required".to_string()],
                severity: "high".to_string(),
                score: 0.00,
            })
            .collect::<Vec<_>>();
        return (
            EdgePodSensitiveOutput {
                scans,
                overall_safety: "review".to_string(),
                redacted_media_url: payload
                    .media_urls
                    .first()
                    .cloned()
                    .unwrap_or_else(String::new),
                summary: "scanner_unavailable".to_string(),
                is_actionable: true,
            },
            "MODEL_UNAVAILABLE".to_string(),
        );
    }

    let mut max_score = 0.0_f64;
    let mut scans = Vec::with_capacity(payload.media_urls.len());
    for media_url in &payload.media_urls {
        let signature = stable_hash(&format!("{}{}", payload.base.request_id, media_url));
        let score = ((signature % 1000) as f64) / 1000.0;
        let (severity, detection) = if score > 0.75 {
            ("high", "sensitive_content")
        } else if score > 0.45 {
            ("medium", "possible_flag")
        } else {
            ("low", "none")
        };
        let detections = if score > 0.45 {
            vec![detection.to_string()]
        } else {
            vec!["none".to_string()]
        };
        max_score = max_score.max(score);
        scans.push(EdgePodSensitiveScan {
            media_url: media_url.clone(),
            detections,
            severity: severity.to_string(),
            score,
        });
    }

    let overall_safety = if max_score > 0.75 {
        "unsafe"
    } else if max_score > 0.45 {
        "review"
    } else {
        "safe"
    };
    let first_url = payload
        .media_urls
        .first()
        .cloned()
        .unwrap_or_else(String::new);
    let redacted_media_url = if overall_safety == "safe" {
        first_url.clone()
    } else {
        format!("{first_url}.redacted")
    };
    let summary = if overall_safety == "unsafe" {
        "unsafe content detected"
    } else {
        "scan_complete"
    };

    (
        EdgePodSensitiveOutput {
            scans,
            overall_safety: overall_safety.to_string(),
            redacted_media_url,
            summary: summary.to_string(),
            is_actionable: overall_safety != "safe",
        },
        "OK".to_string(),
    )
}

fn credit_output(payload: &EdgePodCreditRequest) -> EdgePodCreditOutput {
    if should_fallback_from_request(&payload.base.request_id, &payload.user_id) {
        return EdgePodCreditOutput {
            candidate_allocations: vec![],
            confidence: 0.2,
            reasoning: "manual_override_required".to_string(),
            dispute_window: "days".to_string(),
            confidence_source: "manual".to_string(),
        };
    }
    let mut allocations = Vec::new();
    for (idx, candidate) in payload.skill_profile.iter().enumerate() {
        if idx > 3 {
            break;
        }
        let signature = stable_hash(&format!(
            "{}{}{}",
            payload.base.request_id, payload.user_id, candidate
        ));
        let weight = (((signature % 1000) as f64) + 1.0) / 1400.0;
        allocations.push(EdgePodCreditAllocation {
            candidate_id: candidate.clone(),
            allocation_type: if idx % 2 == 0 {
                "skill"
            } else {
                "contribution"
            }
            .to_string(),
            weight: (weight).min(1.0),
        });
    }

    let denominator = allocations.len() as f64;
    if denominator > 0.0 {
        for allocation in &mut allocations {
            allocation.weight = (allocation.weight / denominator).min(1.0);
        }
    }

    let has_reputation_snapshot = payload.reputation_snapshot.is_some();
    let mut confidence = if payload.timeline_events.is_empty() {
        0.35
    } else {
        0.55 + ((payload.timeline_events.len() as f64) / 50.0).min(0.35)
    };
    if has_reputation_snapshot {
        confidence = (confidence + 0.08).min(0.97);
    }

    let dispute_window = if confidence >= 0.8 {
        "minutes"
    } else {
        "hours"
    };

    EdgePodCreditOutput {
        candidate_allocations: allocations,
        confidence,
        reasoning: if has_reputation_snapshot {
            "deterministic heuristic from timeline, skill profile, and tandang reputation snapshot"
                .to_string()
        } else {
            "deterministic heuristic from timeline and skill profile".to_string()
        },
        dispute_window: dispute_window.to_string(),
        confidence_source: if has_reputation_snapshot {
            "heuristic+tandang_snapshot".to_string()
        } else {
            "heuristic".to_string()
        },
    }
}

fn markov_enrichment_context(cached: &CachedJson) -> Value {
    json!({
        "used": true,
        "cache": {
            "status": cached.meta.status.as_str(),
            "stale": cached.meta.stale,
            "age_ms": cached.meta.age_ms,
            "cached_at_epoch_ms": cached.meta.cached_at_epoch_ms,
        }
    })
}

fn siaga_output(payload: &EdgePodSiagaRequest) -> (EdgePodSiagaOutput, String) {
    let urgency_weight = match payload.reported_urgency.as_str() {
        "critical" => 1.0,
        "high" => 0.7,
        "normal" => 0.4,
        "low" => 0.2,
        _ => 0.5,
    };
    if should_fallback_from_request(&payload.base.request_id, &payload.text) {
        return (
            EdgePodSiagaOutput {
                is_siaga: false,
                severity: "low".to_string(),
                responder_payload: EdgePodSiagaResponderPayload {
                    channels: vec!["manual-review".to_string()],
                    template: "Manual review required".to_string(),
                    estimated_reach: Some(0),
                },
                scope: payload.community_scope.clone(),
                timeline_window: "4h".to_string(),
                override_policy: "manual_approve_only".to_string(),
                confidence: 0.01,
            },
            "MODEL_UNAVAILABLE".to_string(),
        );
    }

    let signature = stable_hash(&format!(
        "{}{}",
        payload.base.request_id, payload.current_track
    ));
    let content_score = ((signature % 1000) as f64) / 1000.0;
    let emergency = urgency_weight > 0.6 || content_score > 0.8;
    let severity = if !emergency {
        "low"
    } else if urgency_weight > 0.8 {
        "critical"
    } else if urgency_weight > 0.55 {
        "high"
    } else {
        "medium"
    };
    let (channels, override_policy, timeline_window, confidence) = match severity {
        "critical" => (
            vec![
                "chat".to_string(),
                "push".to_string(),
                "sms".to_string(),
                "broadcast".to_string(),
            ],
            "auto_if_threshold".to_string(),
            "15m".to_string(),
            0.82,
        ),
        "high" => (
            vec!["chat".to_string(), "push".to_string()],
            "manual_approve_only".to_string(),
            "1h".to_string(),
            0.76,
        ),
        _ => (
            vec!["chat".to_string()],
            "manual_approve_only".to_string(),
            "4h".to_string(),
            0.51,
        ),
    };

    (
        EdgePodSiagaOutput {
            is_siaga: emergency,
            severity: severity.to_string(),
            responder_payload: EdgePodSiagaResponderPayload {
                channels: channels.clone(),
                template: if emergency {
                    "Escalate to emergency responders".to_string()
                } else {
                    "Manual siaga review".to_string()
                },
                estimated_reach: Some(if emergency { 24 } else { 0 }),
            },
            scope: payload.community_scope.clone(),
            timeline_window,
            override_policy,
            confidence,
        },
        "OK".to_string(),
    )
}

pub(crate) async fn edgepod_duplicate_detection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<EdgePodDuplicateRequest>,
) -> Result<Response, ApiError> {
    validate_base_request(&payload.base)?;
    let actor = actor_identity(&auth)?;
    if actor.user_id != payload.base.actor.user_id {
        return Err(ApiError::Forbidden);
    }
    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    if request_id != payload.base.request_id {
        return Err(ApiError::Validation(
            "request_id mismatch with body request_id".into(),
        ));
    }
    if payload.seed_text.is_empty() {
        return Err(ApiError::Validation("seed_text is required".into()));
    }
    if let Some(location) = &payload.location {
        validate_geopoint(location)?;
    }
    if let Some(radius_km) = payload.radius_km {
        if radius_km.is_sign_negative() {
            return Err(ApiError::Validation(
                "radius_km must be non-negative".into(),
            ));
        }
    }

    let idempotency_key = IdempotencyKey::new("edgepod_ep03", actor.user_id, request_id.clone());
    let outcome = state
        .idempotency
        .begin(&idempotency_key)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "idempotency begin failed");
            ApiError::Internal
        })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let output = duplicate_output(&payload);
            let reason_code = if should_fallback_from_request(&request_id, &payload.seed_text) {
                "MODEL_UNAVAILABLE"
            } else {
                "OK"
            };
            if reason_code != "OK" {
                observability::register_edgepod_fallback("ep03_duplicate_detection", reason_code);
            }
            let confidence = output.confidence;
            let actor_context = if reason_code == "MODEL_UNAVAILABLE" {
                Some(json!({
                    "fallback": true,
                    "reason": "model_unavailable_or_parse_error",
                    "endpoint": "ep03",
                }))
            } else {
                None
            };

            let response = idempotent_response(
                &request_id,
                StatusCode::OK,
                output,
                if reason_code == "MODEL_UNAVAILABLE" {
                    0.20
                } else {
                    confidence
                },
                reason_code,
                actor_context,
            )?;
            state
                .idempotency
                .complete(&idempotency_key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

pub(crate) async fn edgepod_gaming_risk(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<EdgePodGamingRequest>,
) -> Result<Response, ApiError> {
    validate_base_request(&payload.base)?;
    let actor = actor_identity(&auth)?;
    if actor.user_id != payload.base.actor.user_id {
        return Err(ApiError::Forbidden);
    }
    if payload.query_users.is_empty() {
        return Err(ApiError::Validation("query_users is required".into()));
    }
    if payload
        .query_users
        .iter()
        .any(|user_id| user_id.trim().is_empty())
    {
        return Err(ApiError::Validation(
            "query_users must not contain empty values".into(),
        ));
    }
    if payload.lookback_hours < EDGE_POD_LOOKBACK_MIN_HOURS {
        return Err(ApiError::Validation("lookback_hours must be >= 1".into()));
    }
    if let Some(focus_metric) = &payload.focus_metric
        && !validate_gaming_focus_metric(focus_metric)
    {
        return Err(ApiError::Validation("invalid focus_metric".into()));
    }

    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    if request_id != payload.base.request_id {
        return Err(ApiError::Validation(
            "request_id mismatch with body request_id".into(),
        ));
    }

    let key = IdempotencyKey::new("edgepod_ep05", actor.user_id, request_id.clone());
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let is_privileged_requester = auth.role.can_moderate();
            let output = gaming_output(&payload, is_privileged_requester);
            let reason_code = if should_fallback_from_request(&request_id, &payload.platform) {
                "MODEL_UNAVAILABLE"
            } else {
                "OK"
            };
            if reason_code != "OK" {
                observability::register_edgepod_fallback("ep05_gaming_risk", reason_code);
            }
            let actor_context = if reason_code == "MODEL_UNAVAILABLE" {
                Some(json!({
                    "fallback": true,
                    "reason": "non_blocking_fallback",
                    "endpoint": "ep05",
                    "warning": "gaming_risk_model_unavailable",
                }))
            } else {
                None
            };
            let confidence = if reason_code == "MODEL_UNAVAILABLE" {
                0.2
            } else {
                (0.45
                    + (output.summary.total_flags as f64 / 2.0).clamp(0.0, 0.5)
                    + (payload.lookback_hours as f64 / 1_000.0).min(0.2))
                .clamp(0.2, 0.98)
            };
            let response = idempotent_response(
                &request_id,
                StatusCode::OK,
                output,
                confidence,
                reason_code,
                actor_context,
            )?;
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

pub(crate) async fn edgepod_sensitive_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<EdgePodSensitiveMediaRequest>,
) -> Result<Response, ApiError> {
    validate_base_request(&payload.base)?;
    let actor = actor_identity(&auth)?;
    if actor.user_id != payload.base.actor.user_id {
        return Err(ApiError::Forbidden);
    }
    if payload.media_urls.is_empty() {
        return Err(ApiError::Validation("media_urls is required".into()));
    }
    if payload.media_types.is_empty() {
        return Err(ApiError::Validation("media_types is required".into()));
    }
    if payload.media_urls.iter().any(|url| url.trim().is_empty()) {
        return Err(ApiError::Validation(
            "media_urls must not contain empty values".into(),
        ));
    }
    if payload
        .media_types
        .iter()
        .any(|media_type| media_type.trim().is_empty())
    {
        return Err(ApiError::Validation(
            "media_types must not contain empty values".into(),
        ));
    }
    if payload.author_id.trim().is_empty() {
        return Err(ApiError::Validation("author_id is required".into()));
    }
    if payload.seed_id.trim().is_empty() {
        return Err(ApiError::Validation("seed_id is required".into()));
    }

    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    if request_id != payload.base.request_id {
        return Err(ApiError::Validation(
            "request_id mismatch with body request_id".into(),
        ));
    }

    let key = IdempotencyKey::new("edgepod_ep08", actor.user_id, request_id.clone());
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let (output, reason_code) = sensitive_media_output(&payload);
            if reason_code != "OK" {
                observability::register_edgepod_fallback("ep08_sensitive_media", &reason_code);
            }
            let actor_context = if reason_code != "OK" {
                Some(json!({
                    "fallback": true,
                    "reason": "manual_moderation_queued",
                    "endpoint": "ep08",
                }))
            } else {
                None
            };
            let confidence = if reason_code != "OK" {
                0.30
            } else {
                output
                    .scans
                    .iter()
                    .map(|scan| scan.score)
                    .fold(0.0, |acc, score| acc + score)
                    / (output.scans.len() as f64).max(1.0)
            };
            let response = idempotent_response(
                &request_id,
                StatusCode::OK,
                output,
                confidence,
                if reason_code == "OK" {
                    "OK"
                } else {
                    "MODEL_UNAVAILABLE"
                },
                actor_context,
            )?;
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

pub(crate) async fn edgepod_credit_recommendation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(mut payload): Json<EdgePodCreditRequest>,
) -> Result<Response, ApiError> {
    validate_base_request(&payload.base)?;
    let actor = actor_identity(&auth)?;
    if actor.user_id != payload.base.actor.user_id {
        return Err(ApiError::Forbidden);
    }
    if payload.user_id.is_empty() {
        return Err(ApiError::Validation("user_id is required".into()));
    }
    if payload.timeline_events.is_empty() {
        return Err(ApiError::Validation("timeline_events is required".into()));
    }
    if payload.skill_profile.is_empty() {
        return Err(ApiError::Validation("skill_profile is required".into()));
    }
    if payload
        .skill_profile
        .iter()
        .any(|candidate| candidate.trim().is_empty())
    {
        return Err(ApiError::Validation(
            "skill_profile must not contain empty values".into(),
        ));
    }

    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    if request_id != payload.base.request_id {
        return Err(ApiError::Validation(
            "request_id mismatch with body request_id".into(),
        ));
    }

    let key = IdempotencyKey::new("edgepod_ep09", actor.user_id, request_id.clone());
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let mut markov_enrichment: Option<Value> = None;
            if payload.reputation_snapshot.is_none() {
                match state.markov_client.get_user_reputation(&payload.user_id).await {
                    Ok(cached) => {
                        markov_enrichment = Some(markov_enrichment_context(&cached));
                        payload.reputation_snapshot = Some(cached.value);
                    }
                    Err(err) => {
                        observability::register_edgepod_fallback(
                            "ep09_credit_recommendation",
                            "MARKOV_REPUTATION_UNAVAILABLE",
                        );
                        tracing::warn!(
                            error = %err,
                            user_id = %payload.user_id,
                            "edgepod credit recommendation markov reputation enrichment failed"
                        );
                        markov_enrichment = Some(json!({
                            "used": false,
                            "reason": "markov_reputation_unavailable",
                        }));
                    }
                }
            }

            let output = credit_output(&payload);
            let reason_code = if should_fallback_from_request(&request_id, &payload.user_id) {
                "MODEL_UNAVAILABLE"
            } else {
                "OK"
            };
            if reason_code != "OK" {
                observability::register_edgepod_fallback("ep09_credit_recommendation", reason_code);
            }
            let mut actor_context_map = Map::new();
            if let Some(markov_enrichment) = markov_enrichment {
                actor_context_map.insert("markov_reputation".to_string(), markov_enrichment);
            }
            if reason_code == "MODEL_UNAVAILABLE" {
                actor_context_map.insert("fallback".to_string(), json!(true));
                actor_context_map.insert("reason".to_string(), json!("manual_form_only"));
                actor_context_map.insert("endpoint".to_string(), json!("ep09"));
            }
            let actor_context = if actor_context_map.is_empty() {
                None
            } else {
                Some(Value::Object(actor_context_map))
            };
            let response = idempotent_response(
                &request_id,
                StatusCode::OK,
                output,
                if reason_code == "OK" {
                    payload
                        .reputation_snapshot
                        .as_ref()
                        .map(|_| 0.80)
                        .unwrap_or(0.72)
                } else {
                    0.30
                },
                reason_code,
                actor_context,
            )?;
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

pub(crate) async fn edgepod_siaga_evaluate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<EdgePodSiagaRequest>,
) -> Result<Response, ApiError> {
    validate_base_request(&payload.base)?;
    let actor = actor_identity(&auth)?;
    if actor.user_id != payload.base.actor.user_id {
        return Err(ApiError::Forbidden);
    }
    if payload.text.is_empty() {
        return Err(ApiError::Validation("text is required".into()));
    }
    if payload.current_track.trim().is_empty() {
        return Err(ApiError::Validation("current_track is required".into()));
    }
    if payload.community_scope.trim().is_empty() {
        return Err(ApiError::Validation("community_scope is required".into()));
    }
    if !matches!(
        payload.reported_urgency.as_str(),
        "low" | "normal" | "high" | "critical"
    ) {
        return Err(ApiError::Validation("invalid reported_urgency".into()));
    }
    validate_geopoint(&payload.location)?;

    let request_id = request_id_from_headers(&headers)?;
    let _correlation_id = correlation_id_from_headers(&headers)?;
    if request_id != payload.base.request_id {
        return Err(ApiError::Validation(
            "request_id mismatch with body request_id".into(),
        ));
    }

    let key = IdempotencyKey::new("edgepod_ep11", actor.user_id, request_id.clone());
    let outcome = state.idempotency.begin(&key).await.map_err(|err| {
        tracing::error!(error = %err, "idempotency begin failed");
        ApiError::Internal
    })?;

    match outcome {
        BeginOutcome::Replay(response) => Ok(to_response(response)),
        BeginOutcome::InProgress => Err(ApiError::Conflict),
        BeginOutcome::Started => {
            let (output, reason_code) = siaga_output(&payload);
            if reason_code != "OK" {
                observability::register_edgepod_fallback("ep11_siaga_evaluate", &reason_code);
            }
            let confidence = if reason_code == "OK" {
                output.confidence
            } else {
                0.25
            };
            let actor_context = if reason_code != "OK" {
                Some(json!({
                    "fallback": true,
                    "reason": "manual_siaga_path",
                    "endpoint": "ep11",
                }))
            } else {
                None
            };
            let response = idempotent_response(
                &request_id,
                StatusCode::OK,
                output,
                confidence,
                if reason_code == "OK" {
                    "OK"
                } else {
                    "MODEL_UNAVAILABLE"
                },
                actor_context,
            )?;
            state
                .idempotency
                .complete(&key, response.clone())
                .await
                .map_err(|err| {
                    tracing::error!(error = %err, "idempotency complete failed");
                    ApiError::Internal
                })?;
            Ok(to_response(response))
        }
    }
}

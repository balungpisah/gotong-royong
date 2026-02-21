# Validation Rules

## Overview

This document specifies the validation rules for Proof of Reality (PoR) evidence. All validation occurs in the Rust backend before evidence is accepted and forwarded to the Markov Credential Engine.

Implementation: `crates/domain/src/evidence/validation.rs`

## General Validation Rules

### Age Limit

**Rule**: Evidence timestamp must be ≤ 30 days old

**Rationale**: Prevents submission of stale or backdated evidence

**Implementation**:
```rust
use chrono::{DateTime, Utc, Duration};

const MAX_EVIDENCE_AGE_DAYS: i64 = 30;

pub fn validate_timestamp(timestamp: &DateTime<Utc>) -> Result<(), ValidationError> {
    let age_days = (Utc::now() - *timestamp).num_days();

    if age_days < 0 {
        return Err(ValidationError::FutureTimestamp);
    }

    if age_days > MAX_EVIDENCE_AGE_DAYS {
        return Err(ValidationError::TimestampTooOld { days: age_days });
    }

    Ok(())
}
```

**Error Messages**:
- `"Timestamp is too old: 45 days"` — Evidence older than 30 days
- `"Timestamp is in the future"` — Clock skew detected
- `"Invalid timestamp format. Expected RFC3339"` — Malformed timestamp

---

### File Type Validation

**Allowed MIME Types**:

| Type | MIME | Max Size |
|------|------|---------|
| Photo | `image/jpeg`, `image/png`, `image/webp` | 15 MB |
| Video | `video/mp4`, `video/webm` | 100 MB |
| Document | `application/pdf` | 10 MB |
| GPS Log | `application/json`, `text/plain` | 1 MB |

**Implementation**:
```rust
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg", "image/png", "image/webp",
    "video/mp4", "video/webm",
    "application/pdf",
    "application/json", "text/plain",
];

pub fn validate_mime_type(mime: &str) -> Result<(), ValidationError> {
    if !ALLOWED_MIME_TYPES.contains(&mime) {
        return Err(ValidationError::UnsupportedMimeType {
            provided: mime.to_string(),
            allowed: ALLOWED_MIME_TYPES.iter().map(|s| s.to_string()).collect(),
        });
    }
    Ok(())
}
```

**Error Messages**:
- `"Unsupported file type: application/x-executable"`
- `"File size exceeds limit: 120.5 MB (max: 100 MB)"`

---

### GPS Coordinate Validation

**Rule**: Coordinates must be valid WGS-84 values within Indonesia's bounding box

**Bounding Box** (Indonesia):
- Latitude: −11.0° to 6.0°
- Longitude: 95.0° to 141.0°

**Implementation**:
```rust
pub struct GpsCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn validate_gps(coord: &GpsCoordinate) -> Result<(), ValidationError> {
    if !(-90.0..=90.0).contains(&coord.latitude) {
        return Err(ValidationError::InvalidLatitude { value: coord.latitude });
    }
    if !(-180.0..=180.0).contains(&coord.longitude) {
        return Err(ValidationError::InvalidLongitude { value: coord.longitude });
    }

    // Indonesia bounding box check (warn only, not error)
    let in_indonesia = (-11.0..=6.0).contains(&coord.latitude)
        && (95.0..=141.0).contains(&coord.longitude);

    if !in_indonesia {
        tracing::warn!(
            lat = coord.latitude,
            lng = coord.longitude,
            "GPS coordinates outside Indonesia bounding box"
        );
    }

    Ok(())
}
```

---

### Hash Integrity

**Rule**: SHA-256 hash of uploaded file must match provided hash

**Implementation**:
```rust
use sha2::{Sha256, Digest};

pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn verify_integrity(data: &[u8], expected_hash: &str) -> Result<(), ValidationError> {
    let computed = compute_sha256(data);
    if computed != expected_hash {
        return Err(ValidationError::HashMismatch {
            expected: expected_hash.to_string(),
            computed,
        });
    }
    Ok(())
}
```

---

### Image EXIF Validation

**Rule**: For photos, EXIF data is extracted and stored; GPS coordinates from EXIF are validated if present

**Implementation** (using `kamadak-exif` crate):
```rust
use exif::{Reader, Tag, In};

pub fn extract_exif_metadata(image_bytes: &[u8]) -> ExifMetadata {
    let mut cursor = std::io::Cursor::new(image_bytes);
    let exif = Reader::new().read_from_container(&mut cursor);

    match exif {
        Ok(exif) => ExifMetadata {
            gps_latitude: read_gps_lat(&exif),
            gps_longitude: read_gps_lng(&exif),
            timestamp: read_datetime(&exif),
            camera_model: read_string(&exif, Tag::Model, In::PRIMARY),
        },
        Err(_) => ExifMetadata::default(), // No EXIF data; proceed without it
    }
}
```

---

## Evidence Type Rules

### Photo Evidence

| Rule | Requirement |
|------|------------|
| File type | JPEG, PNG, or WebP |
| Max size | 15 MB |
| Min resolution | 480×480 px |
| EXIF stripping | Strip personal metadata before storage; keep GPS + timestamp |
| Timestamp | Must be within 30 days |

### Video Evidence

| Rule | Requirement |
|------|------------|
| File type | MP4 or WebM |
| Max size | 100 MB |
| Max duration | 5 minutes |
| Min resolution | 480p |
| Timestamp | Must be within 30 days |

### GPS Log Evidence

| Rule | Requirement |
|------|------------|
| File type | JSON or plain text |
| Format | GeoJSON `LineString` or NMEA sentences |
| Min points | 3 GPS waypoints |
| Max size | 1 MB |
| Timestamp | Must be within 30 days |

### Document Evidence

| Rule | Requirement |
|------|------------|
| File type | PDF |
| Max size | 10 MB |
| Max pages | 50 |
| Timestamp | Must be within 30 days |

---

## Validation Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `TIMESTAMP_TOO_OLD` | 422 | Evidence older than 30 days |
| `FUTURE_TIMESTAMP` | 422 | Timestamp is in the future |
| `INVALID_TIMESTAMP_FORMAT` | 422 | Expected RFC3339 |
| `UNSUPPORTED_MIME_TYPE` | 422 | File type not in allowed list |
| `FILE_TOO_LARGE` | 413 | File size exceeds limit |
| `HASH_MISMATCH` | 422 | Integrity check failed |
| `INVALID_GPS_COORDINATES` | 422 | Out of range lat/lng |
| `LOW_RESOLUTION` | 422 | Image below minimum resolution |
| `EXIF_EXTRACTION_FAILED` | 422 | Corrupt image metadata |
| `PDF_TOO_MANY_PAGES` | 422 | PDF exceeds 50 pages |

---

## Validation Error Response Format

```json
{
  "error": "Evidence validation failed",
  "code": "TIMESTAMP_TOO_OLD",
  "details": {
    "field": "timestamp",
    "provided_value": "2025-12-01T10:00:00Z",
    "age_days": 81,
    "max_age_days": 30
  }
}
```

---

## Post-Validation Pipeline

After successful validation:

1. **EXIF Strip**: Personal metadata removed, GPS + timestamp retained
2. **Hash Record**: SHA-256 stored in `evidence` table for integrity audit
3. **S3 Upload**: File stored at `evidence/{community_id}/{year}/{month}/{uuid}.{ext}`
4. **AI-08 Scan**: Content moderation check for sensitive material
5. **Markov Webhook**: `por_evidence` event dispatched if all checks pass

---

## References

- [Evidence Format](evidence-format.md) — Evidence type definitions
- [Storage Requirements](storage-requirements.md) — S3 storage spec
- [AI Spec — AI-08](../design/specs/AI-SPEC-v0.2.md) — Media scan touch point
- [Webhook Spec](../api/webhook-spec.md) — Webhook delivery after validation

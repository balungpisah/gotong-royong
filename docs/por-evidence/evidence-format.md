# Evidence Format

## Overview

Proof of Reality (PoR) evidence provides cryptographic and multi-perspective verification that claimed contributions actually occurred. This document specifies the format, metadata requirements, and storage specifications for each evidence type.

## Evidence Types

Gotong Royong supports three evidence types:

1. **photo_with_timestamp** - Visual proof with EXIF metadata
2. **gps_verification** - Location-based proof
3. **witness_attestation** - Third-party confirmation

## 1. photo_with_timestamp

### Purpose

Visual documentation of work completion with embedded timestamp proof.

### Use Cases

- Construction/building work (photos of completed structure)
- Agricultural tasks (photos of planted crops, harvested produce)
- Community cleanup (before/after photos)
- Equipment delivery (photos at delivery location)
- Event attendance (photos of participants)

### Required Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `photo_url` | string (URL) | Yes | S3 URL to photo file |
| `media_hash` | string (hex) | Yes | SHA-256 hash of photo file |
| `timestamp` | string (RFC3339) | Yes | Photo capture time |
| `exif_data` | object | No | Extracted EXIF metadata |

### EXIF Metadata

**Recommended fields to extract**:
- `camera` - Camera model (e.g., "iPhone 14 Pro")
- `captured_at` - Timestamp from EXIF (validate matches `timestamp`)
- `location.latitude` - GPS latitude from EXIF
- `location.longitude` - GPS longitude from EXIF
- `altitude` - Altitude in meters
- `focal_length` - Camera focal length
- `iso` - ISO sensitivity
- `aperture` - Aperture value

### Example Payload

```json
{
  "event_type": "por_evidence",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi"
  },
  "subject": {
    "contribution_id": "contrib_abc123",
    "evidence_type": "photo_with_timestamp",
    "evidence_data": {
      "photo_url": "https://cdn.gotong-royong.app/evidence/photo_xyz.jpg",
      "description": "Completed raised garden beds with irrigation",
      "exif": {
        "camera": "iPhone 14 Pro",
        "captured_at": "2026-02-09T10:30:00Z",
        "location": {
          "latitude": -6.2088,
          "longitude": 106.8456
        },
        "altitude": 125.5,
        "focal_length": "6.86mm",
        "iso": 100,
        "aperture": "f/1.78"
      }
    }
  },
  "proof": {
    "timestamp": "2026-02-09T10:30:00Z",
    "media_hash": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456"
  }
}
```

### File Format Requirements

**Supported formats**:
- JPEG (.jpg, .jpeg) - Recommended for photos
- PNG (.png) - For screenshots or diagrams
- HEIC (.heic) - iPhone native format

**File size limits**:
- Minimum: 50KB (prevents placeholder images)
- Maximum: 10MB (for reasonable upload times)

**Resolution requirements**:
- Minimum: 640×480 pixels
- Recommended: 1920×1080 pixels or higher

### Hash Computation

**Algorithm**: SHA-256

**Example (Rust)**:
```rust
use sha2::{Sha256, Digest};

fn compute_media_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// Usage (from file bytes)
let data = std::fs::read("evidence.jpg")?;
let hash = compute_media_hash(&data);
// a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

### EXIF Extraction

**Example (Rust — `kamadak-exif` crate)**:
```rust
use exif::{Reader, Tag, In};

pub struct ExifMetadata {
    pub camera: Option<String>,
    pub captured_at: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
}

pub fn extract_exif(image_bytes: &[u8]) -> ExifMetadata {
    let mut cursor = std::io::Cursor::new(image_bytes);
    let exif = match Reader::new().read_from_container(&mut cursor) {
        Ok(e) => e,
        Err(_) => return ExifMetadata::default(), // No EXIF — proceed without it
    };

    let camera = {
        let make = exif.get_field(Tag::Make, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into());
        let model = exif.get_field(Tag::Model, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into());
        match (make, model) {
            (Some(m), Some(mo)) => Some(format!("{} {}", m, mo)),
            _ => None,
        }
    };

    ExifMetadata {
        camera,
        captured_at: exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
            .map(|f| f.display_value().to_string()),
        gps_latitude: read_gps_decimal(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef),
        gps_longitude: read_gps_decimal(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef),
    }
}
```

See also: `crates/domain/src/evidence/validation.rs` for the full implementation used in the validation pipeline.

---

## 2. gps_verification

### Purpose

Location-based proof that the contributor was physically present at the task location.

### Use Cases

- Delivery verification (GPS at delivery address)
- Field work verification (GPS at farm/worksite)
- Event attendance (GPS at event venue)
- Infrastructure inspection (GPS at inspection site)

### Required Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | string (RFC3339) | Yes | GPS reading time |
| `location.lat` | number | Yes | Latitude (-90 to 90) |
| `location.lon` | number | Yes | Longitude (-180 to 180) |
| `accuracy` | number | No | Accuracy in meters |
| `altitude` | number | No | Altitude in meters |
| `speed` | number | No | Speed in m/s |

### Example Payload

```json
{
  "event_type": "por_evidence",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi"
  },
  "subject": {
    "contribution_id": "contrib_abc123",
    "evidence_type": "gps_verification",
    "evidence_data": {
      "location_name": "Kampung Bersama Community Garden",
      "gps_device": "smartphone",
      "accuracy_meters": 5.2,
      "altitude_meters": 125.5,
      "speed_mps": 0.0
    }
  },
  "proof": {
    "timestamp": "2026-02-09T10:30:00Z",
    "location": {
      "lat": -6.2088,
      "lon": 106.8456
    }
  }
}
```

### Coordinate Validation

**Latitude**: -90 (South Pole) to 90 (North Pole)
**Longitude**: -180 to 180 (wraps around at International Date Line)

**Example validation**:
```javascript
function isValidCoordinates(lat, lon) {
  return lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180;
}
```

### GPS Accuracy

**Accuracy levels**:
- **<5m**: Excellent (urban areas with clear sky)
- **5-10m**: Good (typical smartphone GPS)
- **10-20m**: Fair (partial obstruction)
- **>20m**: Poor (heavy obstruction or indoor)

**Recommendation**: Require accuracy <20m for verification.

### Capturing GPS Data

**Example (Browser Geolocation API)**:
```javascript
function captureGPS() {
  return new Promise((resolve, reject) => {
    if (!navigator.geolocation) {
      reject(new Error('Geolocation not supported'));
      return;
    }

    navigator.geolocation.getCurrentPosition(
      (position) => {
        resolve({
          timestamp: new Date(position.timestamp).toISOString(),
          location: {
            lat: position.coords.latitude,
            lon: position.coords.longitude,
          },
          accuracy: position.coords.accuracy,
          altitude: position.coords.altitude,
          speed: position.coords.speed,
        });
      },
      (error) => reject(error),
      {
        enableHighAccuracy: true,
        timeout: 10000,
        maximumAge: 0,
      }
    );
  });
}

// Usage
const gps = await captureGPS();
console.log(gps);
```

**Example (Mobile - React Native)**:
```javascript
import Geolocation from '@react-native-community/geolocation';

function captureGPS() {
  return new Promise((resolve, reject) => {
    Geolocation.getCurrentPosition(
      (position) => {
        resolve({
          timestamp: new Date(position.timestamp).toISOString(),
          location: {
            lat: position.coords.latitude,
            lon: position.coords.longitude,
          },
          accuracy: position.coords.accuracy,
          altitude: position.coords.altitude,
          speed: position.coords.speed,
        });
      },
      (error) => reject(error),
      { enableHighAccuracy: true, timeout: 10000 }
    );
  });
}
```

---

## 3. witness_attestation

### Purpose

Third-party confirmation from individuals who observed the work completion.

### Use Cases

- Training/workshop verification (attendees confirm)
- Community service verification (beneficiaries confirm)
- Collaborative work verification (team members confirm)
- Event organization verification (participants confirm)

### Required Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | string (RFC3339) | Yes | Attestation time |
| `witnesses` | array | Yes | Array of witness objects (min 1) |

### Witness Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `witness_id` | string | No | User ID or external identifier |
| `witness_name` | string | Yes | Full name |
| `relationship` | string | No | supervisor, peer, beneficiary, other |
| `statement` | string | No | Attestation statement |
| `contact` | string | No | Email or phone (for verification) |

### Example Payload

```json
{
  "event_type": "por_evidence",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi"
  },
  "subject": {
    "contribution_id": "contrib_abc123",
    "evidence_type": "witness_attestation",
    "evidence_data": {
      "contribution_summary": "Led 3-day organic farming workshop",
      "event_date": "2026-02-06 to 2026-02-08",
      "participants_count": 25,
      "location": "Bogor Training Center"
    }
  },
  "proof": {
    "timestamp": "2026-02-09T10:30:00Z",
    "witnesses": [
      {
        "witness_id": "farmer_002",
        "witness_name": "Siti Nurhaliza",
        "relationship": "peer",
        "statement": "Budi taught us safe pesticide alternatives for 3 days. Very knowledgeable and patient instructor.",
        "contact": "siti@example.com"
      },
      {
        "witness_id": "coordinator_001",
        "witness_name": "Ibu Nurni",
        "relationship": "supervisor",
        "statement": "Workshop was comprehensive and well-organized. Received excellent feedback from participants.",
        "contact": "nurni@example.com"
      },
      {
        "witness_name": "Ahmad Dahlan",
        "relationship": "beneficiary",
        "statement": "The training helped me reduce pesticide costs by 40%. Highly recommend.",
        "contact": "+62-812-3456-7890"
      }
    ]
  }
}
```

### Relationship Types

| Type | Description | Example |
|------|-------------|---------|
| `supervisor` | Direct manager or organizer | Event coordinator, project manager |
| `peer` | Co-worker or collaborator | Fellow volunteer, team member |
| `beneficiary` | Person who benefited from work | Training attendee, service recipient |
| `other` | Other relationship | External observer, auditor |

### Digital Signatures (Future Enhancement)

For stronger verification, witnesses can cryptographically sign their attestations:

```json
{
  "witness_name": "Siti Nurhaliza",
  "statement": "Budi taught us safe pesticide alternatives",
  "signature": "0x1234abcd...",  // Ethereum signature
  "signature_algorithm": "ECDSA",
  "public_key": "0x5678efgh..."
}
```

---

## Multi-Evidence Contributions

### Purpose

Combining multiple evidence types increases confidence and tamper resistance.

### Example: Comprehensive Evidence

```json
// Evidence 1: Photo
{
  "evidence_type": "photo_with_timestamp",
  "photo_url": "https://cdn.gotong-royong.app/evidence/photo_001.jpg",
  "media_hash": "a1b2c3d4...",
  "timestamp": "2026-02-09T10:30:00Z"
}

// Evidence 2: GPS
{
  "evidence_type": "gps_verification",
  "location": { "lat": -6.2088, "lon": 106.8456 },
  "timestamp": "2026-02-09T10:30:15Z"
}

// Evidence 3: Witness
{
  "evidence_type": "witness_attestation",
  "witnesses": [
    { "witness_name": "Siti", "statement": "Confirmed completion" }
  ],
  "timestamp": "2026-02-09T12:00:00Z"
}
```

### Evidence Quality Score

Markov Engine calculates evidence quality based on:
- Number of evidence types (1-3)
- Timestamp consistency (all within same day)
- GPS accuracy (if present)
- Witness count (if present)
- Photo metadata completeness (if present)

**Score calculation**:
```
base_score = 0.5
+ (has_photo * 0.2)
+ (has_gps * 0.15)
+ (has_witness * 0.15)
+ (gps_accuracy_bonus * 0.05)  // if accuracy <10m
+ (multiple_witnesses_bonus * 0.05)  // if >2 witnesses
```

**Reputation multiplier**:
- Score 0.5-0.6: 1.0x (single evidence)
- Score 0.6-0.8: 1.2x (two evidence types)
- Score 0.8-1.0: 1.5x (comprehensive evidence)

---

## Storage Requirements

See [Storage Requirements](storage-requirements.md) for:
- File storage backend options
- CDN configuration
- Backup and retention policies
- Access control

## Validation Rules

See [Validation Rules](validation-rules.md) for:
- Field validation requirements
- Business logic validation
- Security checks
- Error messages

## References

- [Event Payloads](../api/event-payloads.md) - JSON schemas
- [Validation Rules](validation-rules.md) - Validation logic
- [Storage Requirements](storage-requirements.md) - Storage infrastructure
- [Markov Integration Guide](../../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md) - Processing logic

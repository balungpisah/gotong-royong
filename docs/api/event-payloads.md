# Event Payloads

## Overview

This document defines the JSON schemas for all webhook event types published by Gotong Royong to the Markov Credential Engine.

## Event Types

1. [contribution_created](#1-contribution_created) - Task completion with evidence
2. [vouch_submitted](#2-vouch_submitted) - Peer endorsement
3. [por_evidence](#3-por_evidence) - Proof of Reality evidence submission

## Common Fields

All events share these top-level fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_type` | string | Yes | Event type identifier |
| `actor` | object | Yes | User who triggered the event |
| `subject` | object | Yes | Event-specific data |
| `event_id` | string | No | Unique event ID (for idempotency) |
| `timestamp` | string (RFC3339) | No | Event creation time |

### Actor Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `user_id` | string | Yes | Unique user identifier |
| `username` | string | Yes | Display username |

## 1. contribution_created

**Purpose**: Records task completion with optional evidence.

### JSON Schema

```json
{
  "type": "object",
  "required": ["event_type", "actor", "subject"],
  "properties": {
    "event_type": {
      "type": "string",
      "const": "contribution_created"
    },
    "event_id": {
      "type": "string",
      "pattern": "^evt_[a-f0-9]{16}$"
    },
    "actor": {
      "type": "object",
      "required": ["user_id", "username"],
      "properties": {
        "user_id": { "type": "string", "minLength": 1 },
        "username": { "type": "string", "minLength": 1 }
      }
    },
    "subject": {
      "type": "object",
      "required": ["contribution_type", "title"],
      "properties": {
        "contribution_type": {
          "type": "string",
          "enum": [
            "task_completion",
            "code_review",
            "documentation",
            "mentoring",
            "event_organization",
            "community_service",
            "custom"
          ]
        },
        "title": { "type": "string", "minLength": 1, "maxLength": 200 },
        "description": { "type": "string", "maxLength": 2000 },
        "evidence_url": { "type": "string", "format": "uri" },
        "skill_ids": {
          "type": "array",
          "items": { "type": "string" },
          "maxItems": 10
        },
        "metadata": {
          "type": "object",
          "additionalProperties": true
        }
      }
    }
  }
}
```

### Contribution Types

| Type | Description | Use Case |
|------|-------------|----------|
| `task_completion` | Physical or digital task completed | Default for most tasks |
| `code_review` | Code review contribution | Software development |
| `documentation` | Documentation or guides | Knowledge sharing |
| `mentoring` | Mentorship or teaching | Education |
| `event_organization` | Event planning and execution | Community organizing |
| `community_service` | Volunteer work | Social impact |
| `custom` | Platform-specific contribution | Use metadata to describe |

### Metadata Fields

Common metadata fields (all optional):

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `location` | string | Task location | "Jakarta" |
| `duration_hours` | number | Time spent | 8 |
| `task_id` | string | Internal task ID | "task_456" |
| `difficulty` | string | Task difficulty | "intermediate" |
| `impact_score` | number | Self-reported impact | 85 |

### Example 1: Basic Task Completion

```json
{
  "event_type": "contribution_created",
  "event_id": "evt_a1b2c3d4e5f6789a",
  "actor": {
    "user_id": "user_123",
    "username": "alice"
  },
  "subject": {
    "contribution_type": "task_completion",
    "title": "Plant seedlings in community garden",
    "description": "Prepared soil and planted 200 vegetable seedlings",
    "skill_ids": ["gardening", "soil_preparation"]
  }
}
```

### Example 2: Task with Evidence and Metadata

```json
{
  "event_type": "contribution_created",
  "event_id": "evt_b2c3d4e5f6789012",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi_farmer"
  },
  "subject": {
    "contribution_type": "task_completion",
    "title": "Delivered 50kg rice to warehouse",
    "description": "Successfully delivered relief supplies to central warehouse",
    "evidence_url": "https://gotong-royong.app/evidence/task_789",
    "skill_ids": ["logistics", "delivery"],
    "metadata": {
      "task_id": "task_789",
      "location": "Jakarta Warehouse District",
      "duration_hours": 4,
      "distance_km": 25,
      "families_served": 50
    }
  }
}
```

### Example 3: Mentoring Contribution

```json
{
  "event_type": "contribution_created",
  "event_id": "evt_c3d4e5f6789abc12",
  "actor": {
    "user_id": "mentor_005",
    "username": "dr_siti"
  },
  "subject": {
    "contribution_type": "mentoring",
    "title": "Led 3-day organic farming workshop",
    "description": "Taught 25 farmers about sustainable pest control methods",
    "skill_ids": ["organic_farming", "teaching"],
    "metadata": {
      "participants_count": 25,
      "duration_hours": 24,
      "location": "Bogor Training Center",
      "topics": ["pest_control", "composting", "crop_rotation"]
    }
  }
}
```

### Field Validation Rules

| Field | Validation |
|-------|------------|
| `title` | 1-200 characters |
| `description` | Max 2000 characters |
| `evidence_url` | Valid URL format |
| `skill_ids` | Max 10 skills per contribution |
| `metadata` | Max 50 keys, values serializable to JSON |

### Processing in Markov Engine

1. Creates `DomainCommand::CreateContribution`
2. Base reputation increase: 50 points (configurable by contribution type)
3. Skills tagged in contributor profile
4. Evidence URL stored for manual review
5. Metadata preserved for analytics

---

## 2. vouch_submitted

**Purpose**: Records peer endorsements between contributors.

### JSON Schema

```json
{
  "type": "object",
  "required": ["event_type", "actor", "subject"],
  "properties": {
    "event_type": {
      "type": "string",
      "const": "vouch_submitted"
    },
    "event_id": {
      "type": "string",
      "pattern": "^evt_[a-f0-9]{16}$"
    },
    "actor": {
      "type": "object",
      "required": ["user_id", "username"],
      "properties": {
        "user_id": { "type": "string", "minLength": 1 },
        "username": { "type": "string", "minLength": 1 }
      }
    },
    "subject": {
      "type": "object",
      "required": ["vouchee_id"],
      "properties": {
        "vouchee_id": { "type": "string", "minLength": 1 },
        "skill_id": { "type": "string" },
        "weight_hint": {
          "type": "string",
          "enum": ["strong", "moderate", "light"]
        },
        "message": { "type": "string", "maxLength": 500 }
      }
    }
  }
}
```

### Weight Hints

| Weight | Meaning | Use Case |
|--------|---------|----------|
| `strong` | High confidence in skill | Expert-level mastery observed |
| `moderate` | Medium confidence | Competent execution seen |
| `light` | Basic confidence | Exploratory or developing skill |
| (omitted) | Default weight | System determines based on voucher reputation |

### Example 1: Skill-Specific Vouch

```json
{
  "event_type": "vouch_submitted",
  "event_id": "evt_d4e5f6789abc1234",
  "actor": {
    "user_id": "expert_002",
    "username": "dr_siti"
  },
  "subject": {
    "vouchee_id": "farmer_001",
    "skill_id": "organic_farming",
    "weight_hint": "strong",
    "message": "Budi consistently applies best practices and mentors others in the community"
  }
}
```

### Example 2: General Endorsement

```json
{
  "event_type": "vouch_submitted",
  "event_id": "evt_e5f6789abc123456",
  "actor": {
    "user_id": "coordinator_001",
    "username": "pak_ahmad"
  },
  "subject": {
    "vouchee_id": "volunteer_042",
    "message": "Reliable and dedicated volunteer, always follows through on commitments"
  }
}
```

### Field Validation Rules

| Field | Validation |
|-------|------------|
| `vouchee_id` | Must exist in system |
| `skill_id` | Must be valid skill identifier (if provided) |
| `weight_hint` | One of: strong, moderate, light |
| `message` | Max 500 characters |

### Processing in Markov Engine

1. Creates `DomainCommand::CreateVouch`
2. Links voucher (actor) to vouchee (subject)
3. Validates skill exists (if skill_id provided)
4. Weight hint influences reputation impact calculation
5. Message stored for audit trail
6. Reputation delta calculated based on voucher's own reputation

---

## 3. por_evidence

**Purpose**: Submits Proof of Reality (PoR) evidence for contribution verification.

### JSON Schema

```json
{
  "type": "object",
  "required": ["event_type", "actor", "subject", "proof"],
  "properties": {
    "event_type": {
      "type": "string",
      "const": "por_evidence"
    },
    "event_id": {
      "type": "string",
      "pattern": "^evt_[a-f0-9]{16}$"
    },
    "actor": {
      "type": "object",
      "required": ["user_id", "username"],
      "properties": {
        "user_id": { "type": "string", "minLength": 1 },
        "username": { "type": "string", "minLength": 1 }
      }
    },
    "subject": {
      "type": "object",
      "required": ["contribution_id", "evidence_type", "evidence_data"],
      "properties": {
        "contribution_id": { "type": "string", "minLength": 1 },
        "evidence_type": {
          "type": "string",
          "enum": ["photo_with_timestamp", "gps_verification", "witness_attestation"]
        },
        "evidence_data": {
          "type": "object",
          "additionalProperties": true
        }
      }
    },
    "proof": {
      "type": "object",
      "properties": {
        "timestamp": { "type": "string", "format": "date-time" },
        "location": {
          "type": "object",
          "properties": {
            "lat": { "type": "number", "minimum": -90, "maximum": 90 },
            "lon": { "type": "number", "minimum": -180, "maximum": 180 }
          }
        },
        "media_hash": { "type": "string", "minLength": 32 },
        "witnesses": {
          "type": "array",
          "items": { "type": "object" }
        }
      }
    }
  }
}
```

### Evidence Types

#### 1. photo_with_timestamp

**Purpose**: Visual proof of work completion

**Required Proof Fields**:
- `timestamp` (RFC3339 datetime)
- `media_hash` (SHA-256 hash of photo file)

**Example**:
```json
{
  "event_type": "por_evidence",
  "event_id": "evt_f6789abc12345678",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi"
  },
  "subject": {
    "contribution_id": "contrib_abc123",
    "evidence_type": "photo_with_timestamp",
    "evidence_data": {
      "photo_url": "https://cdn.gotong-royong.app/evidence/photo_xyz.jpg",
      "exif": {
        "camera": "iPhone 14 Pro",
        "captured_at": "2026-02-09T10:30:00Z",
        "location": {
          "latitude": -6.2088,
          "longitude": 106.8456
        }
      }
    }
  },
  "proof": {
    "timestamp": "2026-02-09T10:30:00Z",
    "media_hash": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456"
  }
}
```

#### 2. gps_verification

**Purpose**: Location-based proof of presence

**Required Proof Fields**:
- `timestamp` (RFC3339 datetime)
- `location.lat` (latitude)
- `location.lon` (longitude)

**Example**:
```json
{
  "event_type": "por_evidence",
  "event_id": "evt_g789abc123456789",
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
      "accuracy_meters": 5.2
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

#### 3. witness_attestation

**Purpose**: Third-party confirmation of work

**Required Proof Fields**:
- `timestamp` (RFC3339 datetime)
- `witnesses` (array with at least 1 witness)

**Witness Object**:
```json
{
  "witness_id": "string",
  "witness_name": "string",
  "relationship": "supervisor|peer|beneficiary|other",
  "statement": "string"
}
```

**Example**:
```json
{
  "event_type": "por_evidence",
  "event_id": "evt_h89abc1234567890",
  "actor": {
    "user_id": "farmer_001",
    "username": "budi"
  },
  "subject": {
    "contribution_id": "contrib_abc123",
    "evidence_type": "witness_attestation",
    "evidence_data": {
      "contribution_summary": "Led 3-day training on organic farming",
      "participants_count": 25
    }
  },
  "proof": {
    "timestamp": "2026-02-09T10:30:00Z",
    "witnesses": [
      {
        "witness_id": "farmer_002",
        "witness_name": "Siti",
        "relationship": "peer",
        "statement": "Budi taught us safe pesticide alternatives for 3 days"
      },
      {
        "witness_id": "coordinator_001",
        "witness_name": "Ibu Nurni",
        "relationship": "supervisor",
        "statement": "Training was comprehensive and well-organized"
      }
    ]
  }
}
```

### Field Validation Rules

See [Validation Rules](../por-evidence/validation-rules.md) for detailed validation specifications.

**Summary**:
- `timestamp`: Must be <= 30 days old
- `media_hash`: Hex string, minimum 32 characters
- `location.lat`: -90.0 to 90.0
- `location.lon`: -180.0 to 180.0
- `witnesses`: Minimum 1 witness for attestation

### Processing in Markov Engine

1. Validates PoR proof structure based on evidence type
2. Creates `DomainCommand::SubmitVerification`
3. Sets outcome to "approved" if validation passes
4. Stores evidence for forensic analysis
5. Reputation bonus applied for verified evidence
6. Rejects with clear error message if validation fails

---

## Event ID Generation

### Purpose

Unique event IDs enable idempotent webhook delivery, preventing duplicate processing.

### Format

**Pattern**: `evt_{random_hex}`

**Example**: `evt_a1b2c3d4e5f6789a`

### Generation Examples

**Node.js**:
```javascript
const crypto = require('crypto');
const eventId = `evt_${crypto.randomBytes(8).toString('hex')}`;
```

**Python**:
```python
import secrets
event_id = f"evt_{secrets.token_hex(8)}"
```

**Rust**:
```rust
use rand::Rng;
let random_bytes: [u8; 8] = rand::thread_rng().gen();
let event_id = format!("evt_{}", hex::encode(random_bytes));
```

---

## Timestamp Format

All timestamps MUST use **RFC3339** format:

**Format**: `YYYY-MM-DDTHH:MM:SSZ`

**Example**: `2026-02-09T10:30:00Z`

**Generation Examples**:

**Node.js**:
```javascript
const timestamp = new Date().toISOString();
```

**Python**:
```python
from datetime import datetime
timestamp = datetime.utcnow().isoformat() + 'Z'
```

**Rust**:
```rust
use chrono::Utc;
let timestamp = Utc::now().to_rfc3339();
```

---

## Size Limits

| Field | Maximum Size |
|-------|--------------|
| Entire payload | 1 MB |
| `title` | 200 characters |
| `description` | 2000 characters |
| `message` | 500 characters |
| `skill_ids` array | 10 items |
| `metadata` object | 50 keys |
| `witnesses` array | 10 items |

Payloads exceeding limits will be rejected with `400 Bad Request`.

---

## Testing Payloads

Test payloads are available in the Markov Engine repository:

**File**: [tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json](../../../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json)

**Load and use**:
```javascript
const testPayloads = require('./gotong_royong_payloads.json');
const validContribution = testPayloads.valid_contribution;
const validVouch = testPayloads.valid_vouch;
const validPorPhoto = testPayloads.valid_por_photo;
```

---

## References

- [Webhook Specification](webhook-spec.md) - Delivery mechanism
- [Authentication](authentication.md) - HMAC-SHA256 implementation
- [Validation Rules](../por-evidence/validation-rules.md) - PoR evidence validation
- [Markov Integration Guide](../../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md) - Complete integration guide

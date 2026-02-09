# Validation Rules

## Overview

This document specifies the validation rules for Proof of Reality (PoR) evidence. All validation occurs before evidence is accepted and forwarded to the Markov Credential Engine.

## General Validation Rules

### Age Limit

**Rule**: Evidence timestamp must be <= 30 days old

**Rationale**: Prevents submission of stale or backdated evidence

**Implementation**:
```javascript
const MAX_EVIDENCE_AGE_DAYS = 30;

function isTimestampValid(timestamp) {
  const eventTime = new Date(timestamp).getTime();
  const now = Date.now();
  const ageDays = (now - eventTime) / (1000 * 60 * 60 * 24);

  if (ageDays < 0) {
    throw new Error('Timestamp is in the future');
  }

  if (ageDays > MAX_EVIDENCE_AGE_DAYS) {
    throw new Error(`Timestamp is too old: ${Math.floor(ageDays)} days (max: ${MAX_EVIDENCE_AGE_DAYS} days)`);
  }

  return true;
}
```

**Error Messages**:
- `"Timestamp is too old: 45 days"` - Evidence older than 30 days
- `"Timestamp is in the future"` - Clock skew detected
- `"Invalid timestamp format. Expected RFC3339"` - Malformed timestamp

### Timestamp Format

**Rule**: Must be RFC3339 format

**Valid formats**:
- `2026-02-09T10:30:00Z` (UTC)
- `2026-02-09T10:30:00+07:00` (with timezone)
- `2026-02-09T10:30:00.123Z` (with milliseconds)

**Invalid formats**:
- `2026-02-09` (date only)
- `1644408600` (Unix timestamp)
- `02/09/2026` (US date format)

**Validation**:
```javascript
function isValidRFC3339(timestamp) {
  const rfc3339Regex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/;
  return rfc3339Regex.test(timestamp);
}
```

## Evidence-Specific Validation

### 1. photo_with_timestamp

#### Required Fields

| Field | Validation |
|-------|------------|
| `proof.timestamp` | RFC3339, <= 30 days old |
| `proof.media_hash` | Hex string, >= 32 characters |

#### Media Hash Validation

**Rule**: SHA-256 hash (64 hex characters) or SHA-1 (40 characters) minimum

**Implementation**:
```javascript
const MIN_MEDIA_HASH_LENGTH = 32;

function isValidMediaHash(hash) {
  if (!hash || hash.length < MIN_MEDIA_HASH_LENGTH) {
    throw new Error(`Invalid media_hash format. Expected hex string (min ${MIN_MEDIA_HASH_LENGTH} chars)`);
  }

  const hexRegex = /^[a-f0-9]+$/i;
  if (!hexRegex.test(hash)) {
    throw new Error('Invalid media_hash format. Hash must contain only hexadecimal characters (0-9, a-f)');
  }

  return true;
}
```

**Examples**:
```
✅ Valid:
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456 (SHA-256, 64 chars)
abc123def456789012345678901234567890abcd (SHA-1, 40 chars)

❌ Invalid:
abc123 (too short, 6 chars)
zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz (contains non-hex character 'z')
```

#### Optional EXIF Validation

If EXIF data is provided, validate consistency:

**Rule**: EXIF timestamp should match proof.timestamp (within 1 minute)

```javascript
function validateExifConsistency(proofTimestamp, exifTimestamp) {
  if (!exifTimestamp) return true; // Optional field

  const proofTime = new Date(proofTimestamp).getTime();
  const exifTime = new Date(exifTimestamp).getTime();
  const diffSeconds = Math.abs(proofTime - exifTime) / 1000;

  if (diffSeconds > 60) {
    console.warn(`EXIF timestamp mismatch: ${diffSeconds}s difference`);
    // Warning only, not a failure
  }

  return true;
}
```

### 2. gps_verification

#### Required Fields

| Field | Validation |
|-------|------------|
| `proof.timestamp` | RFC3339, <= 30 days old |
| `proof.location.lat` | Number, -90 to 90 |
| `proof.location.lon` | Number, -180 to 180 |

#### Latitude Validation

**Rule**: Must be between -90 (South Pole) and 90 (North Pole)

**Implementation**:
```javascript
const MIN_LATITUDE = -90.0;
const MAX_LATITUDE = 90.0;

function isValidLatitude(lat) {
  if (typeof lat !== 'number' || isNaN(lat)) {
    throw new Error('Latitude must be a number');
  }

  if (lat < MIN_LATITUDE || lat > MAX_LATITUDE) {
    throw new Error(`Invalid latitude: ${lat}. Must be between ${MIN_LATITUDE} and ${MAX_LATITUDE}`);
  }

  return true;
}
```

#### Longitude Validation

**Rule**: Must be between -180 and 180

**Implementation**:
```javascript
const MIN_LONGITUDE = -180.0;
const MAX_LONGITUDE = 180.0;

function isValidLongitude(lon) {
  if (typeof lon !== 'number' || isNaN(lon)) {
    throw new Error('Longitude must be a number');
  }

  if (lon < MIN_LONGITUDE || lon > MAX_LONGITUDE) {
    throw new Error(`Invalid longitude: ${lon}. Must be between ${MIN_LONGITUDE} and ${MAX_LONGITUDE}`);
  }

  return true;
}
```

#### Coordinate Precision

**Recommendation**: Store coordinates with 6-8 decimal places

**Precision levels**:
- 1 decimal place: ~11 km precision
- 2 decimal places: ~1.1 km
- 3 decimal places: ~110 m
- 4 decimal places: ~11 m
- 5 decimal places: ~1.1 m
- 6 decimal places: ~11 cm (recommended)
- 7 decimal places: ~1.1 cm
- 8 decimal places: ~1.1 mm

#### GPS Accuracy Validation (Optional)

If `accuracy` field is provided:

```javascript
const MAX_ACCEPTABLE_ACCURACY = 100; // meters

function validateGPSAccuracy(accuracy) {
  if (accuracy === null || accuracy === undefined) {
    return true; // Optional field
  }

  if (accuracy > MAX_ACCEPTABLE_ACCURACY) {
    console.warn(`GPS accuracy poor: ${accuracy}m (recommended: <${MAX_ACCEPTABLE_ACCURACY}m)`);
    // Warning only, not a failure
  }

  return true;
}
```

### 3. witness_attestation

#### Required Fields

| Field | Validation |
|-------|------------|
| `proof.timestamp` | RFC3339, <= 30 days old |
| `proof.witnesses` | Array, minimum 1 witness |

#### Witnesses Array Validation

**Rule**: At least one witness required

**Implementation**:
```javascript
function isValidWitnessArray(witnesses) {
  if (!Array.isArray(witnesses)) {
    throw new Error('witness_attestation requires proof.witnesses array');
  }

  if (witnesses.length === 0) {
    throw new Error('witness_attestation requires at least one witness in proof.witnesses');
  }

  return true;
}
```

#### Individual Witness Validation

**Rule**: Each witness must have at minimum a `witness_name`

**Implementation**:
```javascript
function validateWitness(witness, index) {
  if (!witness.witness_name || witness.witness_name.trim() === '') {
    throw new Error(`Witness ${index + 1} missing required field: witness_name`);
  }

  if (witness.witness_name.length > 255) {
    throw new Error(`Witness ${index + 1} name too long (max 255 characters)`);
  }

  // Optional field validation
  if (witness.relationship) {
    const validRelationships = ['supervisor', 'peer', 'beneficiary', 'other'];
    if (!validRelationships.includes(witness.relationship)) {
      throw new Error(`Witness ${index + 1} invalid relationship: ${witness.relationship}`);
    }
  }

  if (witness.statement && witness.statement.length > 2000) {
    throw new Error(`Witness ${index + 1} statement too long (max 2000 characters)`);
  }

  return true;
}

function validateAllWitnesses(witnesses) {
  witnesses.forEach((witness, index) => {
    validateWitness(witness, index);
  });
  return true;
}
```

#### Maximum Witnesses

**Rule**: Maximum 10 witnesses per attestation

**Rationale**: Prevent abuse and keep payload size reasonable

**Implementation**:
```javascript
const MAX_WITNESSES = 10;

function checkMaxWitnesses(witnesses) {
  if (witnesses.length > MAX_WITNESSES) {
    throw new Error(`Too many witnesses: ${witnesses.length} (max: ${MAX_WITNESSES})`);
  }
  return true;
}
```

## Cross-Field Validation

### Timestamp Consistency

When multiple evidence items exist for the same contribution, timestamps should be reasonably close:

**Rule**: All evidence for a contribution should be within 7 days

**Implementation**:
```javascript
function validateTimestampConsistency(evidenceList) {
  if (evidenceList.length < 2) return true;

  const timestamps = evidenceList.map(e => new Date(e.timestamp).getTime());
  const minTime = Math.min(...timestamps);
  const maxTime = Math.max(...timestamps);
  const diffDays = (maxTime - minTime) / (1000 * 60 * 60 * 24);

  if (diffDays > 7) {
    console.warn(`Evidence timestamps span ${Math.floor(diffDays)} days`);
  }

  return true;
}
```

### Location Consistency (Optional)

If multiple evidence items include GPS data, they should be geographically consistent:

**Rule**: GPS coordinates should be within 1 km of each other

**Implementation**:
```javascript
function haversineDistance(lat1, lon1, lat2, lon2) {
  const R = 6371; // Earth radius in km
  const dLat = (lat2 - lat1) * Math.PI / 180;
  const dLon = (lon2 - lon1) * Math.PI / 180;
  const a = Math.sin(dLat/2) * Math.sin(dLat/2) +
            Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) *
            Math.sin(dLon/2) * Math.sin(dLon/2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a));
  return R * c;
}

function validateLocationConsistency(evidenceList) {
  const locations = evidenceList
    .filter(e => e.latitude && e.longitude)
    .map(e => ({ lat: e.latitude, lon: e.longitude }));

  if (locations.length < 2) return true;

  for (let i = 0; i < locations.length - 1; i++) {
    for (let j = i + 1; j < locations.length; j++) {
      const distance = haversineDistance(
        locations[i].lat, locations[i].lon,
        locations[j].lat, locations[j].lon
      );

      if (distance > 1.0) {
        console.warn(`Evidence locations ${i} and ${j} are ${distance.toFixed(2)}km apart`);
      }
    }
  }

  return true;
}
```

## Payload Size Limits

### Maximum Payload Size

**Rule**: Entire webhook payload must be <= 1 MB

**Implementation**:
```javascript
const MAX_PAYLOAD_SIZE = 1024 * 1024; // 1 MB

function validatePayloadSize(payload) {
  const payloadSize = Buffer.byteLength(JSON.stringify(payload), 'utf-8');

  if (payloadSize > MAX_PAYLOAD_SIZE) {
    throw new Error(`Payload too large: ${payloadSize} bytes (max: ${MAX_PAYLOAD_SIZE} bytes)`);
  }

  return true;
}
```

### Field-Specific Limits

| Field | Maximum Size |
|-------|--------------|
| `title` | 200 characters |
| `description` | 2000 characters |
| `witness.statement` | 2000 characters |
| `metadata` object | 50 keys |
| `witnesses` array | 10 items |

## Security Validation

### SQL Injection Prevention

**Rule**: Sanitize all user input

**Implementation**: Use parameterized queries (examples from schema setup):
```javascript
// ✅ Good: Parameterized query
db.query('INSERT INTO evidence (contribution_id, media_hash) VALUES ($1, $2)', [contributionId, mediaHash]);

// ❌ Bad: String interpolation
db.query(`INSERT INTO evidence (contribution_id, media_hash) VALUES ('${contributionId}', '${mediaHash}')`);
```

### XSS Prevention

**Rule**: Escape HTML in user-provided text

**Implementation**:
```javascript
function escapeHtml(text) {
  const map = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return text.replace(/[&<>"']/g, m => map[m]);
}

// Usage
const safeStatement = escapeHtml(witness.statement);
```

### Path Traversal Prevention

**Rule**: Validate file URLs don't contain path traversal sequences

**Implementation**:
```javascript
function isValidFileUrl(url) {
  if (url.includes('..') || url.includes('/../')) {
    throw new Error('Invalid file URL: path traversal detected');
  }

  if (!url.startsWith('https://')) {
    throw new Error('Invalid file URL: must use HTTPS');
  }

  return true;
}
```

## Error Response Format

When validation fails, return structured error:

```json
{
  "error": "Human-readable error message",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "proof.timestamp",
    "reason": "exceeds_max_age",
    "max_age_days": 30,
    "actual_age_days": 45
  }
}
```

**Example error responses**:

```json
// Timestamp too old
{
  "error": "Timestamp is too old: 45 days",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "proof.timestamp",
    "reason": "exceeds_max_age"
  }
}

// Invalid coordinates
{
  "error": "Invalid latitude: 95.0. Must be between -90 and 90",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "proof.location.lat",
    "reason": "out_of_range",
    "min": -90,
    "max": 90,
    "actual": 95.0
  }
}

// Missing witnesses
{
  "error": "witness_attestation requires at least one witness in proof.witnesses",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "proof.witnesses",
    "reason": "array_empty",
    "min_length": 1
  }
}
```

## Complete Validation Function

**Example (Node.js)**:
```javascript
class PorValidator {
  constructor() {
    this.MAX_EVIDENCE_AGE_DAYS = 30;
    this.MIN_MEDIA_HASH_LENGTH = 32;
    this.MIN_LATITUDE = -90.0;
    this.MAX_LATITUDE = 90.0;
    this.MIN_LONGITUDE = -180.0;
    this.MAX_LONGITUDE = 180.0;
    this.MAX_WITNESSES = 10;
  }

  validate(evidenceType, proof) {
    // Common validation
    this.validateTimestamp(proof.timestamp);

    // Type-specific validation
    switch (evidenceType) {
      case 'photo_with_timestamp':
        return this.validatePhotoEvidence(proof);
      case 'gps_verification':
        return this.validateGPSEvidence(proof);
      case 'witness_attestation':
        return this.validateWitnessEvidence(proof);
      default:
        throw new Error(`Invalid evidence_type: ${evidenceType}`);
    }
  }

  validateTimestamp(timestamp) {
    if (!timestamp) {
      throw new Error('Missing required field: proof.timestamp');
    }

    const eventTime = new Date(timestamp).getTime();
    if (isNaN(eventTime)) {
      throw new Error('Invalid timestamp format. Expected RFC3339');
    }

    const now = Date.now();
    const ageDays = (now - eventTime) / (1000 * 60 * 60 * 24);

    if (ageDays < 0) {
      throw new Error('Timestamp is in the future');
    }

    if (ageDays > this.MAX_EVIDENCE_AGE_DAYS) {
      throw new Error(`Timestamp is too old: ${Math.floor(ageDays)} days`);
    }
  }

  validatePhotoEvidence(proof) {
    if (!proof.media_hash) {
      throw new Error('photo_with_timestamp requires proof.media_hash');
    }

    if (proof.media_hash.length < this.MIN_MEDIA_HASH_LENGTH) {
      throw new Error(`Invalid media_hash format. Expected hex string (min ${this.MIN_MEDIA_HASH_LENGTH} chars)`);
    }

    if (!/^[a-f0-9]+$/i.test(proof.media_hash)) {
      throw new Error('Invalid media_hash format. Hash must contain only hexadecimal characters');
    }

    return true;
  }

  validateGPSEvidence(proof) {
    if (!proof.location) {
      throw new Error('gps_verification requires proof.location');
    }

    const { lat, lon } = proof.location;

    if (typeof lat !== 'number' || isNaN(lat)) {
      throw new Error('Invalid latitude: must be a number');
    }

    if (lat < this.MIN_LATITUDE || lat > this.MAX_LATITUDE) {
      throw new Error(`Invalid latitude: ${lat}. Must be between ${this.MIN_LATITUDE} and ${this.MAX_LATITUDE}`);
    }

    if (typeof lon !== 'number' || isNaN(lon)) {
      throw new Error('Invalid longitude: must be a number');
    }

    if (lon < this.MIN_LONGITUDE || lon > this.MAX_LONGITUDE) {
      throw new Error(`Invalid longitude: ${lon}. Must be between ${this.MIN_LONGITUDE} and ${this.MAX_LONGITUDE}`);
    }

    return true;
  }

  validateWitnessEvidence(proof) {
    if (!Array.isArray(proof.witnesses)) {
      throw new Error('witness_attestation requires proof.witnesses array');
    }

    if (proof.witnesses.length === 0) {
      throw new Error('witness_attestation requires at least one witness');
    }

    if (proof.witnesses.length > this.MAX_WITNESSES) {
      throw new Error(`Too many witnesses: ${proof.witnesses.length} (max: ${this.MAX_WITNESSES})`);
    }

    proof.witnesses.forEach((witness, index) => {
      if (!witness.witness_name || witness.witness_name.trim() === '') {
        throw new Error(`Witness ${index + 1} missing required field: witness_name`);
      }
    });

    return true;
  }
}

// Usage
const validator = new PorValidator();
try {
  validator.validate('photo_with_timestamp', proof);
  console.log('Validation passed');
} catch (error) {
  console.error('Validation failed:', error.message);
}
```

## Testing Validation

### Unit Tests

```javascript
describe('PoR Validation', () => {
  const validator = new PorValidator();

  describe('Timestamp validation', () => {
    it('accepts recent timestamp', () => {
      const timestamp = new Date().toISOString();
      expect(() => validator.validateTimestamp(timestamp)).not.toThrow();
    });

    it('rejects old timestamp', () => {
      const timestamp = new Date(Date.now() - 40 * 24 * 60 * 60 * 1000).toISOString();
      expect(() => validator.validateTimestamp(timestamp)).toThrow('too old');
    });

    it('rejects future timestamp', () => {
      const timestamp = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
      expect(() => validator.validateTimestamp(timestamp)).toThrow('future');
    });
  });

  describe('GPS validation', () => {
    it('accepts valid coordinates', () => {
      const proof = { location: { lat: -6.2088, lon: 106.8456 } };
      expect(() => validator.validateGPSEvidence(proof)).not.toThrow();
    });

    it('rejects invalid latitude', () => {
      const proof = { location: { lat: 95.0, lon: 106.8456 } };
      expect(() => validator.validateGPSEvidence(proof)).toThrow('Invalid latitude');
    });

    it('rejects invalid longitude', () => {
      const proof = { location: { lat: -6.2088, lon: -185.0 } };
      expect(() => validator.validateGPSEvidence(proof)).toThrow('Invalid longitude');
    });
  });

  describe('Witness validation', () => {
    it('accepts valid witnesses', () => {
      const proof = { witnesses: [{ witness_name: 'John Doe' }] };
      expect(() => validator.validateWitnessEvidence(proof)).not.toThrow();
    });

    it('rejects empty witnesses array', () => {
      const proof = { witnesses: [] };
      expect(() => validator.validateWitnessEvidence(proof)).toThrow('at least one witness');
    });

    it('rejects witness without name', () => {
      const proof = { witnesses: [{}] };
      expect(() => validator.validateWitnessEvidence(proof)).toThrow('missing required field');
    });
  });
});
```

## References

- [Evidence Format](evidence-format.md) - Evidence structure
- [Event Payloads](../api/event-payloads.md) - JSON schemas
- [Error Handling](../api/error-handling.md) - Error response format
- [Markov Integration Guide](../../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md) - Server-side validation

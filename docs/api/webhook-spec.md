# Webhook Specification

## Overview

Gotong Royong publishes webhook events to the Markov Credential Engine for real-time reputation updates. This document specifies the webhook endpoint requirements, authentication, and delivery guarantees.

## Endpoint

### Target URL

```
POST https://api.markov.local/api/v1/platforms/gotong_royong/webhook
```

**Environment-specific URLs**:
- **Development**: `http://localhost:3000/api/v1/platforms/gotong_royong/webhook`
- **Staging**: `https://staging-api.markov.local/api/v1/platforms/gotong_royong/webhook`
- **Production**: `https://api.markov.local/api/v1/platforms/gotong_royong/webhook`

### HTTP Method

**POST** only. All other methods return `405 Method Not Allowed`.

## Request Format

All events MUST include `event_id` for idempotency, `schema_version` for contract versioning, and `request_id` for end-to-end traceability.

### Headers

| Header | Required | Description | Example |
|--------|----------|-------------|---------|
| `Content-Type` | Yes | Must be `application/json` | `application/json` |
| `X-GR-Signature` | Yes | HMAC-SHA256 signature | `sha256=a1b2c3...` |
| `X-Request-ID` | Yes | Unique request ID for tracing; must match payload `request_id` | `req_xyz123` |
| `User-Agent` | No | Client identifier | `GotongRoyong/1.0` |

### Body

JSON payload containing event data. Structure varies by event type.

**General Structure**:
```json
{
  "event_id": "string (required, format: evt_<16-hex>)",
  "event_type": "string (required)",
  "schema_version": "string (required, current: \"1\")",
  "request_id": "string (required, must match X-Request-ID)",
  "actor": {
    "user_id": "string (required)",
    "username": "string (required)"
  },
  "subject": {
    /* event-specific fields */
  }
}
```

See [Event Payloads](event-payloads.md) for complete schemas.

## Authentication

### HMAC-SHA256 Signature

All webhook requests MUST include a cryptographic signature to prevent tampering and spoofing.

**Header Format**:
```
X-GR-Signature: sha256={hex_hash}
```

**Signature Computation**:
```
signature = HMAC-SHA256(webhook_secret, raw_request_body)
hex_hash = hex_encode(signature)
```

**Important**: Compute signature over the **raw request body bytes**, not a parsed/serialized version.

### Example: Signature Computation (Python)

```python
import hmac
import hashlib
import json

webhook_secret = "your-webhook-secret-32-chars"
payload = {
    "event_type": "contribution_created",
    "actor": {"user_id": "user123", "username": "alice"},
    "subject": {"contribution_type": "task_completion", "title": "Test"}
}

# Serialize to JSON (compact, no extra whitespace)
payload_bytes = json.dumps(payload, separators=(',', ':')).encode('utf-8')

# Compute HMAC-SHA256
signature = hmac.new(
    webhook_secret.encode('utf-8'),
    payload_bytes,
    hashlib.sha256
).hexdigest()

# Add to header
headers = {
    'Content-Type': 'application/json',
    'X-GR-Signature': f'sha256={signature}'
}
```

### Example: Signature Computation (Node.js)

```javascript
const crypto = require('crypto');

const webhookSecret = 'your-webhook-secret-32-chars';
const payload = {
  event_type: 'contribution_created',
  actor: { user_id: 'user123', username: 'alice' },
  subject: { contribution_type: 'task_completion', title: 'Test' }
};

// Serialize to JSON
const payloadBytes = JSON.stringify(payload);

// Compute HMAC-SHA256
const signature = crypto
  .createHmac('sha256', webhookSecret)
  .update(payloadBytes)
  .digest('hex');

// Add to header
const headers = {
  'Content-Type': 'application/json',
  'X-GR-Signature': `sha256=${signature}`
};
```

### Example: Signature Computation (Rust)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

fn compute_signature(secret: &str, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

// Usage
let secret = "your-webhook-secret-32-chars";
let payload = r#"{"event_type":"contribution_created","actor":{"user_id":"user123","username":"alice"},"subject":{"contribution_type":"task_completion","title":"Test"}}"#;
let signature = compute_signature(secret, payload.as_bytes());
let header_value = format!("sha256={}", signature);
```

### Secret Management

**Generation**:
```bash
# Generate a cryptographically secure 32-byte (256-bit) secret
openssl rand -hex 32
```

**Storage**:
- **Development**: `.env.local` file (never commit to version control)
- **Production**: Secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)

**Environment Variable**:
```bash
GOTONG_ROYONG_WEBHOOK_SECRET=a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

**Rotation Policy**:
- Rotate every 90 days
- Support dual secrets during transition period (7 days)
- Log all signature verification failures for security monitoring

## Response Format

### Success Response

**HTTP Status**: `200 OK`

**Body**:
```json
{
  "processed": 1,
  "results": [
    {
      "type": "contribution_created",
      "contribution_id": "contrib_abc123",
      "message": "Contribution created (reputation: +50)"
    }
  ]
}
```

### Error Responses

#### Invalid Signature

**HTTP Status**: `401 Unauthorized`

**Body**:
```json
{
  "error": "Signature verification failed",
  "code": "UNAUTHORIZED"
}
```

#### Invalid Payload

**HTTP Status**: `400 Bad Request`

**Body**:
```json
{
  "error": "Missing required field: actor.user_id",
  "code": "INVALID_PAYLOAD"
}
```

#### Rate Limit Exceeded

**HTTP Status**: `429 Too Many Requests`

**Headers**:
```
Retry-After: 60
```

**Body**:
```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "retry_after_seconds": 60
}
```

#### Server Error

**HTTP Status**: `500 Internal Server Error`

**Body**:
```json
{
  "error": "Internal server error",
  "code": "INTERNAL_ERROR"
}
```

#### Service Unavailable

**HTTP Status**: `503 Service Unavailable`

**Headers**:
```
Retry-After: 30
```

**Body**:
```json
{
  "error": "Service temporarily unavailable",
  "code": "SERVICE_UNAVAILABLE"
}
```

## Retry Policy

### When to Retry

| HTTP Status | Retry? | Backoff Strategy |
|-------------|--------|------------------|
| 200 | No | - |
| 400 | No | Permanent failure, fix payload |
| 401 | No | Permanent failure, check secret |
| 429 | Yes | Respect `Retry-After` header |
| 500 | Yes | Exponential backoff |
| 503 | Yes | Exponential backoff |
| Timeout | Yes | Exponential backoff |

### Exponential Backoff

**Algorithm**:
```
delay = min(base_delay * (2 ^ attempt), max_delay)
```

**Configuration**:
- `base_delay`: 1 second
- `max_delay`: 60 seconds
- `max_attempts`: 5

**Schedule**:
| Attempt | Delay | Cumulative Time |
|---------|-------|-----------------|
| 1 | 0s | 0s |
| 2 | 1s | 1s |
| 3 | 2s | 3s |
| 4 | 4s | 7s |
| 5 | 8s | 15s |

### Example: Retry Logic (Node.js)

```javascript
async function sendWebhookWithRetry(url, payload, signature, maxAttempts = 5) {
  let attempt = 0;
  let lastError;

  while (attempt < maxAttempts) {
    try {
      const response = await axios.post(url, payload, {
        headers: {
          'Content-Type': 'application/json',
          'X-GR-Signature': signature,
        },
        timeout: 10000, // 10 second timeout
      });

      if (response.status === 200) {
        return response.data; // Success
      }
    } catch (error) {
      lastError = error;
      const status = error.response?.status;

      // Don't retry on permanent failures
      if (status === 400 || status === 401) {
        throw error;
      }

      // Don't retry on last attempt
      if (attempt === maxAttempts - 1) {
        break;
      }

      // Calculate backoff delay
      const delay = Math.min(1000 * Math.pow(2, attempt), 60000);
      console.log(`Webhook failed (attempt ${attempt + 1}), retrying in ${delay}ms`);
      await sleep(delay);
    }

    attempt++;
  }

  throw new Error(`Webhook failed after ${maxAttempts} attempts: ${lastError.message}`);
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

## Idempotency

### Problem

Network issues or retries may cause the same event to be delivered multiple times.

### Solution

Include a unique `event_id` in the payload:

```json
{
  "event_id": "evt_a1b2c3d4e5f6",
  "event_type": "contribution_created",
  "actor": { "user_id": "user123", "username": "alice" },
  "subject": { /* ... */ }
}
```

### Markov Engine Behavior

1. Check if `event_id` already processed
2. If yes: Return `200 OK` (idempotent response)
3. If no: Process event and store `event_id`

### Event ID Format

**Pattern**: `evt_{random_string}`

**Generation** (Node.js):
```javascript
const crypto = require('crypto');
const eventId = `evt_${crypto.randomBytes(8).toString('hex')}`;
// Example: evt_a1b2c3d4e5f6789a
```

**Generation** (Python):
```python
import secrets
event_id = f"evt_{secrets.token_hex(8)}"
# Example: evt_a1b2c3d4e5f6789a
```

## Rate Limiting

### Limits

| Tier | Requests per Minute | Burst Limit |
|------|---------------------|-------------|
| Development | 100 | 200 |
| Production | 1000 | 2000 |

**Note**: Native platform (Gotong Royong) has unlimited rate limits, but these are enforced for consistency with other platforms.

### Rate Limit Headers

Markov Engine includes rate limit information in response headers:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 950
X-RateLimit-Reset: 1644489600
```

### Handling Rate Limits

When receiving `429 Too Many Requests`:
1. Read `Retry-After` header (seconds to wait)
2. Wait specified duration
3. Retry request

**DO NOT**: Immediately retry without waiting.

## Timeouts

### Client Timeout

Set a reasonable timeout to prevent hanging connections:

**Recommended**: 10 seconds

```javascript
// Node.js
axios.post(url, payload, { timeout: 10000 });

// Python
requests.post(url, json=payload, timeout=10.0)

// Rust
reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .build()
```

### Server Timeout

Markov Engine will respond within 5 seconds under normal load.

If processing takes longer:
- Markov returns `202 Accepted` (event queued for async processing)
- Client treats as success
- Markov processes event in background

## Connection Management

### Keep-Alive

Reuse HTTP connections for better performance:

```javascript
// Node.js
const axiosInstance = axios.create({
  baseURL: 'https://api.markov.local',
  timeout: 10000,
  httpAgent: new http.Agent({ keepAlive: true }),
  httpsAgent: new https.Agent({ keepAlive: true }),
});
```

### TLS Version

**Minimum**: TLS 1.2

**Recommended**: TLS 1.3

## Testing

### Test Endpoint

**URL**: `https://webhook-test.markov.local/v1/platforms/gotong_royong/webhook`

- Does NOT verify signatures (for testing signature generation)
- Logs received payloads
- Always returns `200 OK`

### cURL Example

```bash
#!/bin/bash

WEBHOOK_SECRET="test-secret-32-chars-minimum-req"
MARKOV_URL="https://api.markov.local/v1/platforms/gotong_royong/webhook"

PAYLOAD='{"event_type":"contribution_created","actor":{"user_id":"user123","username":"alice"},"subject":{"contribution_type":"task_completion","title":"Test Task"}}'

# Compute HMAC-SHA256 signature
SIGNATURE="sha256=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" -hex | cut -d' ' -f2)"

# Send webhook
curl -X POST "$MARKOV_URL" \
  -H "Content-Type: application/json" \
  -H "X-GR-Signature: $SIGNATURE" \
  -d "$PAYLOAD" \
  -w "\nHTTP Status: %{http_code}\n"
```

### Mock Server

For local development, run a mock Markov server:

```javascript
// mock-markov-server.js
const express = require('express');
const crypto = require('crypto');

const app = express();
app.use(express.json());

const WEBHOOK_SECRET = process.env.GOTONG_ROYONG_WEBHOOK_SECRET || 'test-secret';

app.post('/v1/platforms/gotong_royong/webhook', (req, res) => {
  const signature = req.headers['x-gr-signature'];
  const payload = JSON.stringify(req.body);

  // Verify signature
  const expectedSig = crypto
    .createHmac('sha256', WEBHOOK_SECRET)
    .update(payload)
    .digest('hex');

  if (signature !== `sha256=${expectedSig}`) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  console.log('Received event:', req.body.event_type);
  res.json({ processed: 1, results: [{ type: req.body.event_type }] });
});

app.listen(3000, () => console.log('Mock Markov server running on port 3000'));
```

## Security Best Practices

1. **Always verify signatures** - Never trust unsigned webhooks
2. **Use HTTPS in production** - Never send webhooks over HTTP
3. **Rotate secrets regularly** - Every 90 days minimum
4. **Log signature failures** - Potential security incidents
5. **Validate payloads** - Check all required fields exist
6. **Limit payload size** - Max 1MB per request
7. **Set timeouts** - Prevent hanging connections
8. **Use constant-time comparison** - Prevent timing attacks on signature verification

## Monitoring

### Metrics to Track

- `gotong_worker_webhook_delivery_total{result,status_code}` - Total webhook attempts by outcome and response status
- `gotong_worker_webhook_delivery_duration_ms{result,status_code}` - Response time (p50, p95, p99)
- `gotong_worker_webhook_dead_letter_total` - Current dead-letter queue depth

### Alerting

**Critical**:
- Success rate < 95% for 5 minutes
- Signature verification failures > 10 per minute (potential attack)

**Warning**:
- Success rate < 98% for 15 minutes
- Average latency > 1000ms

## References

- [Event Payloads](event-payloads.md) - Complete event schemas
- [Authentication](authentication.md) - Detailed HMAC implementation
- [Error Handling](error-handling.md) - Error codes and recovery strategies
- [Markov Integration Guide](../../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md) - Markov Engine documentation

# Authentication

## Overview

Gotong Royong uses **HMAC-SHA256** cryptographic signatures to authenticate webhook requests to the Markov Credential Engine. This prevents request tampering, spoofing, and replay attacks.

## HMAC-SHA256 Signature

### What is HMAC?

**HMAC** (Hash-based Message Authentication Code) is a cryptographic function that combines:
- A **secret key** (known only to sender and receiver)
- A **message** (the webhook payload)
- A **hash function** (SHA-256)

The result is a unique signature that:
- ✅ Proves the sender knows the secret
- ✅ Proves the message hasn't been tampered with
- ✅ Cannot be forged without the secret

### Algorithm

```
signature = HMAC-SHA256(secret_key, message)
hex_signature = hex_encode(signature)
header_value = "sha256=" + hex_signature
```

### Header Format

```
X-GR-Signature: sha256={hex_encoded_hmac}
```

**Example**:
```
X-GR-Signature: sha256=a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

## Implementation Guide

### 1. Secret Key Management

#### Generation

**Requirement**: Minimum 32 characters (256 bits)

**Generate using OpenSSL**:
```bash
openssl rand -hex 32
```

**Output**:
```
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

#### Storage

**Development**:
```bash
# .env.local (NEVER commit this file)
GOTONG_ROYONG_WEBHOOK_SECRET=a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

**Production**:
Use a secrets manager:
- **AWS**: AWS Secrets Manager
- **Google Cloud**: Secret Manager
- **HashiCorp**: Vault
- **Kubernetes**: Sealed Secrets

**Example (AWS Secrets Manager)**:
```bash
# Store secret
aws secretsmanager create-secret \
  --name gotong-royong-webhook-secret \
  --secret-string "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456"

# Retrieve secret in application
aws secretsmanager get-secret-value \
  --secret-id gotong-royong-webhook-secret \
  --query SecretString \
  --output text
```

#### Rotation Policy

**Best Practices**:
- Rotate every **90 days**
- Support **dual secrets** during transition (7-day overlap)
- Log all signature failures for security monitoring
- Alert on sudden spike in signature failures (potential attack)

**Dual Secret Example**:
```javascript
const OLD_SECRET = process.env.GOTONG_ROYONG_WEBHOOK_SECRET_OLD;
const NEW_SECRET = process.env.GOTONG_ROYONG_WEBHOOK_SECRET;

function verifySignature(payload, signature) {
  // Try new secret first
  if (isValidSignature(payload, signature, NEW_SECRET)) {
    return true;
  }
  // Fall back to old secret during transition
  if (OLD_SECRET && isValidSignature(payload, signature, OLD_SECRET)) {
    console.warn('Webhook using old secret, migration needed');
    return true;
  }
  return false;
}
```

### 2. Signature Computation (Sender Side)

#### Node.js / TypeScript

```typescript
import crypto from 'crypto';

function computeSignature(secret: string, payload: string): string {
  return crypto
    .createHmac('sha256', secret)
    .update(payload, 'utf-8')
    .digest('hex');
}

// Usage
const payload = JSON.stringify(event);
const signature = computeSignature(process.env.GOTONG_ROYONG_WEBHOOK_SECRET!, payload);
const headerValue = `sha256=${signature}`;

// Send webhook
await axios.post(webhookUrl, payload, {
  headers: {
    'Content-Type': 'application/json',
    'X-GR-Signature': headerValue,
  },
});
```

#### Python

```python
import hmac
import hashlib
import json

def compute_signature(secret: str, payload: str) -> str:
    return hmac.new(
        secret.encode('utf-8'),
        payload.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()

# Usage
import os
import requests

secret = os.environ['GOTONG_ROYONG_WEBHOOK_SECRET']
payload = json.dumps(event, separators=(',', ':'))  # Compact JSON
signature = compute_signature(secret, payload)
header_value = f'sha256={signature}'

# Send webhook
response = requests.post(
    webhook_url,
    data=payload,
    headers={
        'Content-Type': 'application/json',
        'X-GR-Signature': header_value,
    }
)
```

#### Rust

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

fn compute_signature(secret: &str, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

// Usage
use reqwest;

let secret = std::env::var("GOTONG_ROYONG_WEBHOOK_SECRET")?;
let payload = serde_json::to_string(&event)?;
let signature = compute_signature(&secret, payload.as_bytes());
let header_value = format!("sha256={}", signature);

let client = reqwest::Client::new();
let response = client
    .post(&webhook_url)
    .header("Content-Type", "application/json")
    .header("X-GR-Signature", header_value)
    .body(payload)
    .send()
    .await?;
```

#### Go

```go
package main

import (
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "net/http"
    "os"
    "strings"
)

func computeSignature(secret string, payload []byte) string {
    h := hmac.New(sha256.New, []byte(secret))
    h.Write(payload)
    return hex.EncodeToString(h.Sum(nil))
}

func main() {
    secret := os.Getenv("GOTONG_ROYONG_WEBHOOK_SECRET")
    event := map[string]interface{}{
        "event_type": "contribution_created",
        // ... event data
    }

    payloadBytes, _ := json.Marshal(event)
    signature := computeSignature(secret, payloadBytes)
    headerValue := fmt.Sprintf("sha256=%s", signature)

    req, _ := http.NewRequest("POST", webhookUrl, strings.NewReader(string(payloadBytes)))
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("X-GR-Signature", headerValue)

    client := &http.Client{}
    resp, _ := client.Do(req)
    defer resp.Body.Close()
}
```

### 3. Signature Verification (Receiver Side)

#### Critical: Constant-Time Comparison

**⚠️ Security Warning**: Always use constant-time comparison to prevent timing attacks.

**Vulnerable Code** (DO NOT USE):
```javascript
// INSECURE: Timing attack possible
if (computedSignature === providedSignature) {
  return true;
}
```

**Secure Code** (USE THIS):
```javascript
// SECURE: Constant-time comparison
const crypto = require('crypto');

function isValidSignature(payload, providedSignature, secret) {
  const computedSignature = computeSignature(secret, payload);

  // Constant-time comparison prevents timing attacks
  return crypto.timingSafeEqual(
    Buffer.from(computedSignature, 'utf-8'),
    Buffer.from(providedSignature, 'utf-8')
  );
}
```

#### Node.js / TypeScript

```typescript
import crypto from 'crypto';

function verifyWebhookSignature(
  payload: string,
  signatureHeader: string,
  secret: string
): boolean {
  // Extract hex hash from header
  if (!signatureHeader.startsWith('sha256=')) {
    throw new Error('Invalid signature format');
  }
  const providedSignature = signatureHeader.substring(7); // Remove "sha256="

  // Compute expected signature
  const computedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload, 'utf-8')
    .digest('hex');

  // Constant-time comparison
  if (computedSignature.length !== providedSignature.length) {
    return false;
  }

  return crypto.timingSafeEqual(
    Buffer.from(computedSignature, 'utf-8'),
    Buffer.from(providedSignature, 'utf-8')
  );
}

// Express middleware
app.use(express.json({
  verify: (req, res, buf) => {
    req.rawBody = buf.toString('utf-8');
  }
}));

app.post('/webhook', (req, res) => {
  const signature = req.headers['x-gr-signature'];
  const secret = process.env.GOTONG_ROYONG_WEBHOOK_SECRET;

  if (!verifyWebhookSignature(req.rawBody, signature, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  // Process webhook
  res.json({ processed: 1 });
});
```

#### Python

```python
import hmac
import hashlib

def verify_webhook_signature(payload: bytes, signature_header: str, secret: str) -> bool:
    """Verify HMAC-SHA256 webhook signature."""
    if not signature_header.startswith('sha256='):
        return False

    provided_signature = signature_header[7:]  # Remove "sha256="

    # Compute expected signature
    computed_signature = hmac.new(
        secret.encode('utf-8'),
        payload,
        hashlib.sha256
    ).hexdigest()

    # Constant-time comparison
    return hmac.compare_digest(computed_signature, provided_signature)

# Flask example
from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route('/webhook', methods=['POST'])
def webhook():
    signature = request.headers.get('X-GR-Signature')
    secret = os.environ['GOTONG_ROYONG_WEBHOOK_SECRET']
    payload = request.get_data()  # Raw bytes

    if not verify_webhook_signature(payload, signature, secret):
        return jsonify({'error': 'Invalid signature'}), 401

    # Process webhook
    event = request.json
    return jsonify({'processed': 1})
```

#### Rust (from Markov Engine)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

fn verify_hmac(secret: &str, payload: &[u8], signature: &str) -> Result<(), Error> {
    // Extract hex hash from "sha256={hash}" format
    let expected_hash = signature
        .strip_prefix("sha256=")
        .ok_or(Error::InvalidSignature)?;

    // Compute HMAC-SHA256
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| Error::InvalidSignature)?;
    mac.update(payload);
    let computed_hash = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison to prevent timing attacks
    if computed_hash.len() != expected_hash.len() {
        return Err(Error::InvalidSignature);
    }

    let computed_bytes = computed_hash.as_bytes();
    let expected_bytes = expected_hash.as_bytes();

    if computed_bytes.ct_eq(expected_bytes).into() {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
```

## Security Best Practices

### 1. Always Verify Signatures

**❌ NEVER accept webhooks without signature verification**:
```javascript
// INSECURE - DO NOT DO THIS
app.post('/webhook', (req, res) => {
  processWebhook(req.body);  // No signature check!
  res.json({ ok: true });
});
```

**✅ Always verify**:
```javascript
// SECURE
app.post('/webhook', (req, res) => {
  if (!verifySignature(req.rawBody, req.headers['x-gr-signature'])) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  processWebhook(req.body);
  res.json({ ok: true });
});
```

### 2. Use Raw Request Body

**Critical**: Compute signature over **raw bytes**, not parsed JSON.

**❌ Wrong**:
```javascript
const signature = computeSignature(JSON.stringify(req.body));  // WRONG
```

**✅ Correct**:
```javascript
const signature = computeSignature(req.rawBody);  // Correct
```

**Why**: JSON serialization may add/remove whitespace, changing the signature.

### 3. Validate Timestamp

**Prevent replay attacks** by checking event timestamp:

```javascript
function isTimestampValid(timestamp, maxAgeSeconds = 300) {
  const eventTime = new Date(timestamp).getTime();
  const now = Date.now();
  const ageSeconds = (now - eventTime) / 1000;

  return ageSeconds >= 0 && ageSeconds <= maxAgeSeconds;
}

app.post('/webhook', (req, res) => {
  // Verify signature first
  if (!verifySignature(req.rawBody, req.headers['x-gr-signature'])) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  // Check timestamp (if present)
  const event = req.body;
  if (event.timestamp && !isTimestampValid(event.timestamp)) {
    return res.status(400).json({ error: 'Timestamp too old or in future' });
  }

  processWebhook(event);
  res.json({ ok: true });
});
```

### 4. Log Signature Failures

**Monitor for attacks** by logging verification failures:

```javascript
if (!verifySignature(payload, signature, secret)) {
  console.error('Webhook signature verification failed', {
    source_ip: req.ip,
    signature_prefix: signature.substring(0, 20),
    timestamp: new Date().toISOString(),
  });

  // Alert on high failure rate
  alertIfTooManyFailures();

  return res.status(401).json({ error: 'Invalid signature' });
}
```

### 5. Rate Limit by Signature Failure

**Prevent brute-force attacks**:

```javascript
const failedAttempts = new Map();

function checkRateLimit(ip) {
  const attempts = failedAttempts.get(ip) || 0;
  if (attempts > 10) {
    return false;  // Rate limited
  }
  return true;
}

function recordFailure(ip) {
  const attempts = failedAttempts.get(ip) || 0;
  failedAttempts.set(ip, attempts + 1);

  // Reset after 1 hour
  setTimeout(() => failedAttempts.delete(ip), 3600000);
}

app.post('/webhook', (req, res) => {
  if (!checkRateLimit(req.ip)) {
    return res.status(429).json({ error: 'Rate limit exceeded' });
  }

  if (!verifySignature(req.rawBody, req.headers['x-gr-signature'])) {
    recordFailure(req.ip);
    return res.status(401).json({ error: 'Invalid signature' });
  }

  // Process webhook
  res.json({ ok: true });
});
```

## Testing

### Unit Tests

**Test signature generation**:
```javascript
describe('Signature computation', () => {
  it('generates valid HMAC-SHA256 signature', () => {
    const secret = 'test_secret_32_chars_minimum_here';
    const payload = '{"event_type":"contribution_created"}';

    const signature = computeSignature(secret, payload);

    expect(signature).toHaveLength(64);  // SHA-256 = 32 bytes = 64 hex chars
    expect(signature).toMatch(/^[a-f0-9]{64}$/);
  });

  it('produces different signatures for different payloads', () => {
    const secret = 'test_secret_32_chars_minimum_here';
    const payload1 = '{"event_type":"contribution_created"}';
    const payload2 = '{"event_type":"vouch_submitted"}';

    const sig1 = computeSignature(secret, payload1);
    const sig2 = computeSignature(secret, payload2);

    expect(sig1).not.toEqual(sig2);
  });
});
```

**Test signature verification**:
```javascript
describe('Signature verification', () => {
  it('accepts valid signature', () => {
    const secret = 'test_secret_32_chars_minimum_here';
    const payload = '{"event_type":"contribution_created"}';
    const signature = computeSignature(secret, payload);
    const header = `sha256=${signature}`;

    expect(verifySignature(payload, header, secret)).toBe(true);
  });

  it('rejects invalid signature', () => {
    const secret = 'test_secret_32_chars_minimum_here';
    const payload = '{"event_type":"contribution_created"}';
    const header = 'sha256=invalid_signature_here';

    expect(verifySignature(payload, header, secret)).toBe(false);
  });

  it('rejects tampered payload', () => {
    const secret = 'test_secret_32_chars_minimum_here';
    const payload = '{"event_type":"contribution_created"}';
    const signature = computeSignature(secret, payload);
    const header = `sha256=${signature}`;

    const tamperedPayload = '{"event_type":"contribution_deleted"}';

    expect(verifySignature(tamperedPayload, header, secret)).toBe(false);
  });
});
```

### Integration Tests

**Test with real Markov Engine**:
```javascript
describe('Webhook integration', () => {
  it('accepts webhook with valid signature', async () => {
    const secret = process.env.GOTONG_ROYONG_WEBHOOK_SECRET;
    const payload = {
      event_type: 'contribution_created',
      actor: { user_id: 'test_user', username: 'test' },
      subject: { contribution_type: 'task_completion', title: 'Test' },
    };

    const payloadStr = JSON.stringify(payload);
    const signature = computeSignature(secret, payloadStr);

    const response = await axios.post(
      'https://api.markov.local/v1/platforms/gotong_royong/webhook',
      payload,
      {
        headers: {
          'Content-Type': 'application/json',
          'X-GR-Signature': `sha256=${signature}`,
        },
      }
    );

    expect(response.status).toBe(200);
    expect(response.data.processed).toBe(1);
  });
});
```

## Common Issues

### Issue 1: Signature Mismatch

**Symptom**: Webhook returns `401 Unauthorized`

**Causes**:
1. Wrong secret on sender or receiver
2. JSON serialization differences (whitespace)
3. Character encoding issues (UTF-8 vs ASCII)

**Solution**:
```javascript
// Use compact JSON serialization
const payload = JSON.stringify(event, null, 0);  // No pretty-printing

// Ensure UTF-8 encoding
const signature = computeSignature(secret, Buffer.from(payload, 'utf-8'));
```

### Issue 2: Timing Attacks

**Symptom**: Security audit flags timing vulnerability

**Cause**: Using `===` comparison for signatures

**Solution**: Use constant-time comparison (see above)

### Issue 3: Replay Attacks

**Symptom**: Old webhooks can be re-sent

**Cause**: No timestamp validation

**Solution**: Check event timestamp (max 5 minutes old)

## References

- [Webhook Specification](webhook-spec.md) - Complete webhook protocol
- [Event Payloads](event-payloads.md) - JSON schemas
- [Error Handling](error-handling.md) - Error codes and recovery
- [HMAC RFC 2104](https://datatracker.ietf.org/doc/html/rfc2104) - HMAC specification

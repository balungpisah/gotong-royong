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

**Dual Secret Example (Rust)**:
```rust
async fn verify_with_rotation(
    payload: &[u8],
    signature: &str,
    new_secret: &str,
    old_secret: Option<&str>,
) -> bool {
    if verify_hmac(new_secret, payload, signature).is_ok() {
        return true;
    }
    if let Some(old) = old_secret {
        if verify_hmac(old, payload, signature).is_ok() {
            tracing::warn!("Webhook using old secret — rotation needed");
            return true;
        }
    }
    false
}
```

### 2. Signature Computation (Sender Side)

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

### 3. Signature Verification (Receiver Side)

#### Critical: Constant-Time Comparison

**⚠️ Security Warning**: Always use constant-time comparison to prevent timing attacks.

**Vulnerable Code** (DO NOT USE):
```rust
// INSECURE: Early-exit comparison leaks timing info
if computed_signature == provided_signature {
    return true;
}
```

**Secure Code** (USE THIS — `subtle` crate):
```rust
use subtle::ConstantTimeEq;

// SECURE: Constant-time comparison
let computed_bytes = computed_signature.as_bytes();
let provided_bytes = provided_signature.as_bytes();

if computed_bytes.len() == provided_bytes.len()
    && computed_bytes.ct_eq(provided_bytes).into()
{
    return Ok(());
}
```

#### Rust (Axum handler)

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
```rust
// INSECURE - DO NOT DO THIS
async fn webhook_handler(body: Bytes) -> impl IntoResponse {
    process_webhook(body).await; // No signature check!
    Json(json!({"ok": true}))
}
```

**✅ Always verify** (use `WebhookSignature` extractor middleware):
```rust
// SECURE — Axum extractor rejects bad signatures before reaching the handler
async fn webhook_handler(
    WebhookSignature(event): WebhookSignature<WebhookEvent>,
) -> impl IntoResponse {
    process_webhook(event).await;
    Json(json!({"ok": true}))
}
```

### 2. Use Raw Request Body

**Critical**: Compute signature over **raw bytes**, not parsed JSON.

**❌ Wrong**:
```rust
// INSECURE: re-serializing parsed JSON may reorder/modify whitespace
let payload_str = serde_json::to_string(&parsed_event)?;
verify_hmac(&secret, payload_str.as_bytes(), &signature)?;
```

**✅ Correct** — verify over raw bytes before parsing:
```rust
// SECURE: operate on the original request bytes
let raw_body: Bytes = request.into_body().collect().await?.to_bytes();
verify_hmac(&secret, &raw_body, &signature)?;
let event: WebhookEvent = serde_json::from_slice(&raw_body)?;
```

**Why**: JSON serialization may reorder keys or add whitespace, changing the hash.

### 3. Validate Timestamp

**Prevent replay attacks** by checking event timestamp:

```rust
fn is_timestamp_valid(timestamp: &DateTime<Utc>, max_age: Duration) -> bool {
    let age = Utc::now() - *timestamp;
    age >= Duration::zero() && age <= max_age
}

// In Axum extractor — check timestamp after signature verification:
let age = Utc::now() - event.timestamp;
if age < Duration::zero() || age > Duration::minutes(5) {
    return Err(StatusCode::BAD_REQUEST);
}
```

### 4. Log Signature Failures

**Monitor for attacks** by logging verification failures:

```rust
if verify_hmac(&secret, &raw_body, &signature).is_err() {
    warn!(
        source_ip = %client_ip,
        signature_prefix = &signature[..20.min(signature.len())],
        "Webhook signature verification failed"
    );
    // Prometheus counter — alert triggers if rate spikes
    counter!("gotong_webhook_signature_failures_total").increment(1);
    return Err(StatusCode::UNAUTHORIZED);
}
```

### 5. Rate Limit by Signature Failure

**Prevent brute-force attacks**:

Use Redis to track failures per IP with TTL-based reset:

```rust
// In the webhook signature extractor middleware:
let failure_key = format!("webhook:sig_fail:{}", client_ip);

if verify_hmac(&secret, &raw_body, &signature).is_err() {
    let failures: u32 = redis.incr(&failure_key).await?;
    redis.expire(&failure_key, 3600).await?; // Reset window: 1 hour

    if failures > 10 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    return Err(StatusCode::UNAUTHORIZED);
}
// Clear failure count on success
redis.del(&failure_key).await.ok();
```

## Testing

### Unit Tests

**Test signature generation and verification**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_secret_32_chars_minimum_here";
    const PAYLOAD: &str = r#"{"event_type":"contribution_created"}"#;

    #[test]
    fn compute_signature_produces_64_char_hex() {
        let sig = compute_signature(TEST_SECRET, PAYLOAD.as_bytes());
        assert_eq!(sig.len(), 64, "SHA-256 hex is always 64 chars");
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_payloads_produce_different_signatures() {
        let payload2 = r#"{"event_type":"vouch_submitted"}"#;
        let sig1 = compute_signature(TEST_SECRET, PAYLOAD.as_bytes());
        let sig2 = compute_signature(TEST_SECRET, payload2.as_bytes());
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn verify_hmac_accepts_valid_signature() {
        let sig = compute_signature(TEST_SECRET, PAYLOAD.as_bytes());
        let header = format!("sha256={}", sig);
        assert!(verify_hmac(TEST_SECRET, PAYLOAD.as_bytes(), &header).is_ok());
    }

    #[test]
    fn verify_hmac_rejects_invalid_signature() {
        let header = "sha256=invalid_signature_here";
        assert!(verify_hmac(TEST_SECRET, PAYLOAD.as_bytes(), header).is_err());
    }

    #[test]
    fn verify_hmac_rejects_tampered_payload() {
        let sig = compute_signature(TEST_SECRET, PAYLOAD.as_bytes());
        let header = format!("sha256={}", sig);
        let tampered = r#"{"event_type":"contribution_deleted"}"#;
        assert!(verify_hmac(TEST_SECRET, tampered.as_bytes(), &header).is_err());
    }

    #[test]
    fn verify_hmac_rejects_missing_prefix() {
        let sig = compute_signature(TEST_SECRET, PAYLOAD.as_bytes());
        // Header without "sha256=" prefix
        assert!(verify_hmac(TEST_SECRET, PAYLOAD.as_bytes(), &sig).is_err());
    }
}
```

### Integration Tests

**Test webhook endpoint with mock Markov Engine** (using `wiremock`):
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header_exists};

#[tokio::test]
async fn webhook_sender_includes_valid_signature() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/platforms/gotong_royong/webhook"))
        .and(header_exists("X-GR-Signature"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({ "processed": 1 })
        ))
        .mount(&mock_server)
        .await;

    let secret = "test_secret_32_chars_minimum_here";
    let event = serde_json::json!({
        "event_type": "contribution_created",
        "actor": { "user_id": "test_user", "username": "test" },
        "subject": { "contribution_type": "task_completion", "title": "Test" },
    });
    let payload = serde_json::to_string(&event).unwrap();
    let signature = compute_signature(secret, payload.as_bytes());

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/platforms/gotong_royong/webhook", mock_server.uri()))
        .header("Content-Type", "application/json")
        .header("X-GR-Signature", format!("sha256={}", signature))
        .body(payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["processed"], 1);
}

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

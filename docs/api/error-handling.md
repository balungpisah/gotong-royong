# Error Handling

## Overview

This document specifies how Gotong Royong should handle errors when delivering webhooks to the Markov Credential Engine, including HTTP status codes, error response formats, retry strategies, and idempotency.

## HTTP Status Codes

### Success Codes

| Code | Meaning | Client Action |
|------|---------|---------------|
| **200 OK** | Event processed successfully | Mark complete, no retry |
| **202 Accepted** | Event queued for async processing | Mark complete, no retry |

### Client Error Codes (4xx)

| Code | Meaning | Retry? | Client Action |
|------|---------|--------|---------------|
| **400 Bad Request** | Invalid payload format | ❌ No | Fix payload, log error |
| **401 Unauthorized** | Signature verification failed | ❌ No | Check webhook secret |
| **403 Forbidden** | IP not whitelisted | ❌ No | Contact ops team |
| **404 Not Found** | Endpoint doesn't exist | ❌ No | Fix webhook URL |
| **413 Payload Too Large** | Payload exceeds 1MB | ❌ No | Reduce payload size |
| **429 Too Many Requests** | Rate limit exceeded | ✅ Yes | Respect Retry-After header |

### Server Error Codes (5xx)

| Code | Meaning | Retry? | Client Action |
|------|---------|--------|---------------|
| **500 Internal Server Error** | Markov server error | ✅ Yes | Retry with exponential backoff |
| **502 Bad Gateway** | Proxy error | ✅ Yes | Retry with exponential backoff |
| **503 Service Unavailable** | Temporary maintenance | ✅ Yes | Retry with exponential backoff |
| **504 Gateway Timeout** | Request took too long | ✅ Yes | Retry with exponential backoff |

### Network Errors

| Error Type | Retry? | Client Action |
|------------|--------|---------------|
| **Connection Refused** | ✅ Yes | Retry with exponential backoff |
| **DNS Resolution Failed** | ✅ Yes | Retry with exponential backoff |
| **Timeout (>10s)** | ✅ Yes | Retry with exponential backoff |
| **SSL/TLS Error** | ❌ No | Check certificate configuration |

## Error Response Format

### Standard Error Response

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": {
    "field": "field_name",
    "reason": "specific reason"
  }
}
```

### Example: Invalid Payload

**Request**:
```json
{
  "event_type": "contribution_created",
  "actor": {
    "user_id": "user123"
    // Missing required "username" field
  },
  "subject": {
    "contribution_type": "task_completion",
    "title": "Test"
  }
}
```

**Response**:
```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "error": "Missing required field: actor.username",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "actor.username",
    "reason": "required"
  }
}
```

### Example: Signature Verification Failed

**Request**: (with invalid signature)

**Response**:
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "error": "Signature verification failed",
  "code": "UNAUTHORIZED"
}
```

### Example: Rate Limit Exceeded

**Response**:
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 60

{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "details": {
    "retry_after_seconds": 60,
    "limit": 1000,
    "window_seconds": 60
  }
}
```

### Example: PoR Validation Failed

**Request**:
```json
{
  "event_type": "por_evidence",
  "actor": { "user_id": "user123", "username": "alice" },
  "subject": {
    "contribution_id": "contrib_abc",
    "evidence_type": "photo_with_timestamp",
    "evidence_data": {}
  },
  "proof": {
    "timestamp": "2025-01-01T00:00:00Z",  // Too old (>30 days)
    "media_hash": "abc123"  // Too short
  }
}
```

**Response**:
```http
HTTP/1.1 400 Bad Request
Content-Type: application/json

{
  "error": "Timestamp is too old: 40 days",
  "code": "INVALID_PAYLOAD",
  "details": {
    "field": "proof.timestamp",
    "reason": "exceeds_max_age",
    "max_age_days": 30,
    "actual_age_days": 40
  }
}
```

## Error Codes

### Authentication Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Signature verification failed |
| `FORBIDDEN` | 403 | IP not whitelisted |
| `INVALID_SIGNATURE_FORMAT` | 401 | Signature header malformed |

### Payload Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_PAYLOAD` | 400 | JSON parsing failed or missing required field |
| `INVALID_EVENT_TYPE` | 400 | Unknown event_type |
| `PAYLOAD_TOO_LARGE` | 413 | Payload exceeds 1MB |
| `INVALID_JSON` | 400 | JSON syntax error |

### Validation Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_TIMESTAMP` | 400 | Timestamp format invalid or too old |
| `INVALID_COORDINATES` | 400 | GPS coordinates out of bounds |
| `INVALID_MEDIA_HASH` | 400 | Media hash format invalid |
| `INVALID_EVIDENCE_TYPE` | 400 | Unknown evidence type |
| `MISSING_WITNESSES` | 400 | Witness attestation requires witnesses |

### Rate Limiting Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests in time window |

### Server Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INTERNAL_ERROR` | 500 | Unexpected server error |
| `SERVICE_UNAVAILABLE` | 503 | Temporary maintenance |

## Retry Strategy

### Exponential Backoff Algorithm

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
| 1 | 0s | 0s (immediate) |
| 2 | 1s | 1s |
| 3 | 2s | 3s |
| 4 | 4s | 7s |
| 5 | 8s | 15s |
| Failed | - | Move to Dead Letter Queue |

### Implementation (Rust)

```rust
use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;

async fn send_webhook_with_retry(
    client: &reqwest::Client,
    url: &str,
    payload: &str,
    signature: &str,
    max_attempts: u32,
) -> Result<serde_json::Value, Error> {
    for attempt in 0..max_attempts {
        match client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-GR-Signature", signature)
            .body(payload.to_owned())
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => return Ok(r.json().await?),
            Ok(r) => {
                let status = r.status();
                // Don't retry on permanent 4xx (except 429 rate limit)
                if status.is_client_error() && status != StatusCode::TOO_MANY_REQUESTS {
                    return Err(Error::PermanentFailure(status));
                }
                // Respect Retry-After on 429
                if status == StatusCode::TOO_MANY_REQUESTS {
                    let wait = r
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60);
                    tracing::warn!(wait_secs = wait, "Rate limited by Markov Engine");
                    sleep(Duration::from_secs(wait)).await;
                    continue;
                }
            }
            Err(e) if attempt == max_attempts - 1 => return Err(Error::Request(e)),
            Err(e) => tracing::warn!(attempt, error = %e, "Webhook delivery failed"),
        }

        if attempt < max_attempts - 1 {
            let delay = std::cmp::min(1u64 << attempt, 60);
            sleep(Duration::from_secs(delay)).await;
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

## Idempotency

### Problem

Network issues or retries may cause the same event to be delivered multiple times. Without idempotency, this could lead to:
- Duplicate reputation updates
- Incorrect contribution counts
- Data inconsistencies

### Solution: Event IDs

Include a unique `event_id` in every webhook payload:

```json
{
  "event_id": "evt_a1b2c3d4e5f6789a",
  "event_type": "contribution_created",
  "actor": { "user_id": "user123", "username": "alice" },
  "subject": { /* ... */ }
}
```

### Markov Engine Behavior

1. Extract `event_id` from payload
2. Check if `event_id` already processed (query database)
3. If processed: Return `200 OK` with cached result (idempotent)
4. If new: Process event, store `event_id`, return result

### Database Schema (Markov Engine Side)

```sql
CREATE TABLE processed_events (
  event_id VARCHAR(50) PRIMARY KEY,
  event_type VARCHAR(50) NOT NULL,
  processed_at TIMESTAMP DEFAULT NOW(),
  result JSONB
);

CREATE INDEX idx_processed_events_type ON processed_events(event_type);
CREATE INDEX idx_processed_events_time ON processed_events(processed_at);
```

### Idempotency Check (Pseudo-code)

```javascript
async function processWebhook(event) {
  const eventId = event.event_id;

  // Check if already processed
  const existing = await db.query(
    'SELECT result FROM processed_events WHERE event_id = $1',
    [eventId]
  );

  if (existing.rows.length > 0) {
    console.log(`Event ${eventId} already processed (idempotent)`);
    return existing.rows[0].result;  // Return cached result
  }

  // Process event
  const result = await processEvent(event);

  // Store event_id to prevent duplicates
  await db.query(
    'INSERT INTO processed_events (event_id, event_type, result) VALUES ($1, $2, $3)',
    [eventId, event.event_type, JSON.stringify(result)]
  );

  return result;
}
```

### Event ID Generation

**Format**: `evt_{16_hex_chars}`

**Rust**:
```rust
use rand::Rng;
let random_bytes: [u8; 8] = rand::thread_rng().gen();
let event_id = format!("evt_{}", hex::encode(random_bytes));
// Example: evt_a1b2c3d4e5f6789a
```

## Dead Letter Queue (DLQ)

### Purpose

Store failed webhooks after max retries for manual reprocessing or investigation.

### Database Schema

```sql
CREATE TABLE webhook_failures (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id VARCHAR(50),
  event_type VARCHAR(50) NOT NULL,
  payload JSONB NOT NULL,
  error_message TEXT,
  status_code INT,
  attempts INT DEFAULT 0,
  last_attempt_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_webhook_failures_type ON webhook_failures(event_type);
CREATE INDEX idx_webhook_failures_created ON webhook_failures(created_at);
```

### Moving to DLQ

```javascript
async function moveToDeadLetterQueue(event, error, attempts) {
  await db.query(
    `INSERT INTO webhook_failures
     (event_id, event_type, payload, error_message, status_code, attempts, last_attempt_at)
     VALUES ($1, $2, $3, $4, $5, $6, NOW())`,
    [
      event.event_id,
      event.event_type,
      JSON.stringify(event),
      error.message,
      error.response?.status || null,
      attempts,
    ]
  );

  console.error(`Event ${event.event_id} moved to DLQ after ${attempts} attempts`);

  // Alert operations team
  await alertOps({
    message: `Webhook DLQ: ${event.event_type}`,
    event_id: event.event_id,
    error: error.message,
  });
}
```

### Manual Reprocessing

```javascript
async function reprocessFailedWebhooks(limit = 10) {
  // Get failed webhooks
  const failures = await db.query(
    'SELECT * FROM webhook_failures ORDER BY created_at ASC LIMIT $1',
    [limit]
  );

  for (const failure of failures.rows) {
    try {
      const event = failure.payload;
      await sendWebhook(event);

      // Remove from DLQ on success
      await db.query('DELETE FROM webhook_failures WHERE id = $1', [failure.id]);
      console.log(`Reprocessed event ${failure.event_id}`);

    } catch (error) {
      console.error(`Failed to reprocess ${failure.event_id}: ${error.message}`);
    }
  }
}
```

## Monitoring and Alerting

### Metrics to Track

**Success Rate**:
```promql
sum(rate(gotong_worker_webhook_delivery_total{result="success"}[5m])) / sum(rate(gotong_worker_webhook_delivery_total[5m])) * 100
```

**Error Rate by Status Code**:
```promql
sum(rate(gotong_worker_webhook_delivery_total{result!="success"}[5m])) by (status_code)
```

**Retry Count Distribution**:
```promql
histogram_quantile(0.95, sum(rate(gotong_worker_webhook_delivery_duration_ms_bucket[5m])) by (le))
```

**DLQ Size**:
```promql
gotong_worker_webhook_dead_letter_total
```

### Alerting Rules

**Critical Alerts** (PagerDuty):
- Success rate < 95% for 5 minutes
- DLQ size > 100 events
- Signature verification failures > 10/min (potential attack)

**Warning Alerts** (Slack):
- Success rate < 98% for 15 minutes
- Average retry count > 2
- DLQ size > 50 events

### Dashboard Panels

**Webhook Health Dashboard**:
1. Success rate over time (line chart)
2. Error rate by status code (bar chart)
3. Latency percentiles (p50, p95, p99)
4. DLQ size over time
5. Retry count distribution
6. Top error messages (table)

## Testing Error Scenarios

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn retries_on_500_then_succeeds() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST")).and(path("/webhook"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server).await;

        Mock::given(method("POST")).and(path("/webhook"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"processed": 1})))
            .mount(&mock_server).await;

        let client = reqwest::Client::new();
        let result = send_webhook_with_retry(
            &client, &format!("{}/webhook", mock_server.uri()),
            r#"{}"#, "sha256=test", 5,
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn does_not_retry_on_400() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST")).and(path("/webhook"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&mock_server).await;

        let client = reqwest::Client::new();
        let result = send_webhook_with_retry(
            &client, &format!("{}/webhook", mock_server.uri()),
            r#"{}"#, "sha256=test", 5,
        ).await;

        assert!(matches!(result, Err(Error::PermanentFailure(_))));
        // Verify only one request was made (no retry)
        mock_server.verify().await;
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn moves_to_dlq_after_max_retries() {
    let mock_server = MockServer::start().await;
    let db = test_db().await;

    // Always return 500
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(5)
        .mount(&mock_server).await;

    let result = send_webhook_with_retry(
        &reqwest::Client::new(),
        &format!("{}/webhook", mock_server.uri()),
        r#"{"event_id":"evt_test123"}"#,
        "sha256=test",
        5,
    ).await;

    assert!(result.is_err());
    mock_server.verify().await;

    // Verify DLQ entry was recorded
    let failures: Vec<WebhookFailure> = db
        .query("SELECT * FROM webhook_failures WHERE event_id = $event_id")
        .bind(("event_id", "evt_test123"))
        .await?
        .take(0)?;
    assert_eq!(failures.len(), 1);
}
```

## References

- [Webhook Specification](webhook-spec.md) - Complete webhook protocol
- [Authentication](authentication.md) - Signature verification
- [Event Payloads](event-payloads.md) - JSON schemas
- [Monitoring](../deployment/monitoring.md) - Metrics and dashboards

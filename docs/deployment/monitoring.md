# Monitoring

## Overview

This document specifies the monitoring, observability, and alerting requirements for the Gotong Royong platform.

## Monitoring Stack

### Recommended Stack

**Core Components**:
- **Metrics**: Prometheus
- **Visualization**: Grafana
- **Logging**: Loki or ELK Stack
- **Tracing**: Jaeger or Tempo
- **Alerting**: Alertmanager

**Alternative**: All-in-one solutions (Datadog, New Relic, Grafana Cloud)

## Metrics Collection

### Application Metrics

**What to Track**:

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total` | Counter | Total HTTP requests |
| `http_request_duration_seconds` | Histogram | Request latency |
| `http_requests_in_progress` | Gauge | Concurrent requests |
| `gotong_worker_webhook_delivery_total{result,status_code}` | Counter | Webhook attempts by outcome/status |
| `gotong_worker_webhook_delivery_duration_ms{result,status_code}` | Histogram | Webhook latency |
| `gotong_worker_webhook_dead_letter_total` | Gauge | Current dead-letter queue depth |
| `db_connections_active` | Gauge | Active DB connections |
| `db_query_duration_seconds` | Histogram | Query duration |
| `cache_hits_total` | Counter | Cache hits |
| `cache_misses_total` | Counter | Cache misses |
| `file_uploads_total` | Counter | File uploads |
| `file_upload_size_bytes` | Histogram | Upload size |

### Implementation (Rust with `metrics` + `metrics-exporter-prometheus`)

```rust
// Cargo.toml dependencies:
// metrics = "0.22"
// metrics-exporter-prometheus = "0.13"
// axum = "0.7"

use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use axum::{Router, routing::get, response::IntoResponse};

pub fn setup_metrics() -> Router {
    // Install Prometheus exporter
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Register metric descriptions
    describe_counter!("http_requests_total", "Total HTTP requests");
    describe_histogram!("http_request_duration_seconds", "HTTP request latency");
    describe_counter!(
        "gotong_worker_webhook_delivery_total",
        "Total webhook delivery attempts"
    );
    describe_histogram!(
        "gotong_worker_webhook_delivery_duration_ms",
        "Webhook delivery duration in milliseconds"
    );
    describe_gauge!(
        "gotong_worker_webhook_dead_letter_total",
        "Current dead-letter queue depth"
    );
    describe_gauge!("db_connections_active", "Active database connections");

    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler() -> impl IntoResponse {
    let handle = metrics_exporter_prometheus::PrometheusHandle::current();
    handle.render()
}

// Axum middleware for request tracking
pub async fn track_request_metrics<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("http_requests_total", "method" => method.clone(), "route" => path.clone(), "status_code" => status).increment(1);
    histogram!("http_request_duration_seconds", "method" => method, "route" => path).record(duration);

    response
}

// Track webhook deliveries
async fn send_webhook(event: &WebhookEvent) -> Result<(), WebhookError> {
    counter!("gotong_worker_webhook_delivery_total", "event_type" => event.event_type.clone()).increment(1);

  try {
    await deliverWebhook(event);
    webhookDeliverySuccess.labels(event.event_type).inc();
  } catch (error) {
    console.error('Webhook delivery failed:', error);
  }
}
```

### Prometheus Configuration

**prometheus.yml**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'gotong-royong-api'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - gotong-royong
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        regex: gotong-royong-api
        action: keep
      - source_labels: [__meta_kubernetes_pod_ip]
        target_label: __address__
        replacement: ${1}:3000

  - job_name: 'surrealdb'
    static_configs:
      - targets: ['surrealdb-exporter:9399']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

## Dashboards

### Grafana Dashboards

#### 1. Application Overview Dashboard

**Panels**:
- Request rate (req/sec)
- Request latency (p50, p95, p99)
- Error rate (%)
- Active connections
- Memory usage
- CPU usage

**PromQL Queries**:

```promql
# Request rate
rate(http_requests_total[5m])

# P95 latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Error rate
rate(http_requests_total{status_code=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100

# Active connections
db_connections_active
```

#### 2. Webhook Health Dashboard

**Panels**:
- Webhook delivery rate
- Success rate
- Failure rate by event type
- Retry count distribution
- Delivery latency

**PromQL Queries**:

```promql
# Success rate
sum(rate(gotong_worker_webhook_delivery_total{result="success"}[5m])) / sum(rate(gotong_worker_webhook_delivery_total[5m])) * 100

# Failures by status code
sum(rate(gotong_worker_webhook_delivery_total{result!="success"}[5m])) by (status_code)

# P95 delivery latency
histogram_quantile(0.95, sum(rate(gotong_worker_webhook_delivery_duration_ms_bucket[5m])) by (le))
```

#### 3. Database Dashboard

**Panels**:
- Query rate
- Query latency
- Connection pool utilization
- Slow queries
- Cache hit rate

**PromQL Queries**:

```promql
# Query latency
histogram_quantile(0.95, rate(db_query_duration_seconds_bucket[5m]))

# Connection pool utilization
db_connections_active / db_connections_max * 100

# Cache hit rate
rate(cache_hits_total[5m]) / (rate(cache_hits_total[5m]) + rate(cache_misses_total[5m])) * 100
```

#### 4. Evidence Upload Dashboard

**Panels**:
- Upload rate
- Upload success rate
- Upload size distribution
- Upload latency
- Storage usage

**PromQL Queries**:

```promql
# Upload rate
rate(file_uploads_total[5m])

# Success rate
rate(file_uploads_total{status="success"}[5m]) / rate(file_uploads_total[5m]) * 100

# Average upload size
rate(file_upload_size_bytes_sum[5m]) / rate(file_upload_size_bytes_count[5m])
```

### Dashboard JSON Export

**Example (Grafana JSON)**:
```json
{
  "dashboard": {
    "title": "Gotong Royong - Application Overview",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{route}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total{status_code=~\"5..\"}[5m]) / rate(http_requests_total[5m]) * 100",
            "legendFormat": "Error Rate %"
          }
        ],
        "alert": {
          "conditions": [
            {
              "evaluator": {
                "params": [5],
                "type": "gt"
              },
              "operator": {
                "type": "and"
              },
              "query": {
                "params": ["A", "5m", "now"]
              },
              "reducer": {
                "params": [],
                "type": "avg"
              },
              "type": "query"
            }
          ],
          "name": "High Error Rate"
        }
      }
    ]
  }
}
```

## Logging

### Structured Logging

**Format**: JSON

**Required Fields**:
- `timestamp` (ISO 8601)
- `level` (debug, info, warn, error)
- `service` (gotong-royong-api)
- `message`
- `context` (additional data)

**Implementation (Rust — `tracing` + `tracing-subscriber`)**:

```rust
// Cargo.toml:
// tracing = "0.1"
// tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let json_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .init();
}

// Usage in application code:
use tracing::{info, error, warn, instrument};

#[instrument(fields(user_id = %user.id))]
async fn create_user(user: &User) {
    info!(username = %user.username, "User created");
}

// Error logging
error!(
    db_host = %db_host,
    error = %e,
    "Database connection failed"
);
```

### Log Levels

| Level | Use Case | Example |
|-------|----------|---------|
| **debug** | Detailed debugging info | Function entry/exit, variable values |
| **info** | General information | User created, webhook delivered |
| **warn** | Warnings, non-critical issues | Slow query, high memory usage |
| **error** | Errors, failures | Database connection failed, webhook failed |

### What to Log

**DO log**:
- Authentication attempts (success/failure)
- Authorization failures
- Webhook deliveries (success/failure)
- File uploads (success/failure)
- Database errors
- API errors (5xx)
- Suspicious activity

**DON'T log**:
- Passwords
- JWT tokens
- Webhook secrets
- Full credit card numbers
- Full PII (only user IDs)

### Log Aggregation (Loki)

**Promtail Configuration**:

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: gotong-royong-api
    static_configs:
      - targets:
          - localhost
        labels:
          job: gotong-royong-api
          __path__: /var/log/gotong-royong/*.log
```

**LogQL Queries**:

```logql
# All errors in last hour
{job="gotong-royong-api"} | json | level="error" | line_format "{{.timestamp}} {{.message}}"

# Failed webhook deliveries
{job="gotong-royong-api"} | json | message=~"Webhook.*failed"

# Slow queries (>1 second)
{job="gotong-royong-api"} | json | duration > 1000
```

## Tracing (Distributed)

### OpenTelemetry Setup

**Implementation (Rust — `opentelemetry` + `tracing-opentelemetry`)**:

```rust
// Cargo.toml:
// opentelemetry = "0.22"
// opentelemetry-otlp = { version = "0.15", features = ["http-proto"] }
// tracing-opentelemetry = "0.23"

use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_otel_tracing(otlp_endpoint: &str) {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(otlp_endpoint),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize OTel tracer");

    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

**Trace Context Propagation**:

```rust
use tracing::{info, Span};

// Axum middleware — trace ID is automatically propagated via tracing spans
// Access trace ID in handlers:
let trace_id = Span::current()
    .context()
    .span()
    .span_context()
    .trace_id();

info!(
    trace_id = %trace_id,
    method = %method,
    path = %path,
    "Request received"
);
```

## Alerting

### Alertmanager Configuration

**alertmanager.yml**:

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'slack-notifications'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
    - match:
        severity: warning
      receiver: 'slack-notifications'

receivers:
  - name: 'slack-notifications'
    slack_configs:
      - channel: '#gotong-royong-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'
```

### Alert Rules

**alerts.yml**:

```yaml
groups:
  - name: application
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          rate(http_requests_total{status_code=~"5.."}[5m])
          / rate(http_requests_total[5m]) * 100 > 5
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }}% (threshold: 5%)"

      # High latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95,
            rate(http_request_duration_seconds_bucket[5m])
          ) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High request latency"
          description: "P95 latency is {{ $value }}s (threshold: 1s)"

      # Webhook delivery failures
      - alert: WebhookDeliveryFailures
        expr: |
          sum(rate(gotong_worker_webhook_delivery_total{result!="success"}[5m])) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High webhook failure rate"
          description: "{{ $value }} webhook failures per second"

      # Database connection failures
      - alert: DatabaseConnectionFailures
        expr: db_connections_active == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database connection failed"
          description: "No active database connections"

      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          process_resident_memory_bytes
          / node_memory_MemTotal_bytes * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}% (threshold: 80%)"

      # Disk space low
      - alert: DiskSpaceLow
        expr: |
          node_filesystem_avail_bytes{mountpoint="/"}
          / node_filesystem_size_bytes{mountpoint="/"} * 100 < 20
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space"
          description: "Only {{ $value }}% disk space remaining"
```

### Alert Severity Levels

| Severity | Response Time | Notification Channel |
|----------|---------------|----------------------|
| **Critical** | Immediate | PagerDuty (page on-call) |
| **Warning** | Next business day | Slack channel |
| **Info** | Monitor only | Dashboard only |

## Health Checks

### Liveness Probe

**Purpose**: Is the application running?

**Endpoint**: `GET /health`

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2026-02-10T10:30:00Z"
}
```

**Implementation**:
```javascript
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
  });
});
```

### Readiness Probe

**Purpose**: Is the application ready to accept traffic?

**Endpoint**: `GET /ready`

**Response**:
```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "s3": "ok"
  }
}
```

**Implementation**:
```javascript
app.get('/ready', async (req, res) => {
  const checks = {
    database: await checkDatabase(),
    redis: await checkRedis(),
    s3: await checkS3(),
  };

  const ready = Object.values(checks).every(c => c === 'ok');

  res.status(ready ? 200 : 503).json({
    status: ready ? 'ready' : 'not_ready',
    checks,
  });
});

async function checkDatabase() {
  try {
    await db.query('SELECT 1');
    return 'ok';
  } catch (error) {
    return 'error';
  }
}
```

## Performance Monitoring

### Key Performance Indicators (KPIs)

| KPI | Target | Measurement |
|-----|--------|-------------|
| **API Response Time (P95)** | <200ms | Histogram |
| **Webhook Delivery Time (P95)** | <5s | Histogram |
| **Database Query Time (P95)** | <100ms | Histogram |
| **Evidence Upload Time (P95)** | <2s | Histogram |
| **Error Rate** | <1% | Counter ratio |
| **Uptime** | >99.9% | Uptime check |

### SLA Monitoring

**Service Level Objectives (SLOs)**:
- API availability: 99.9% (43.2 minutes downtime/month)
- API latency: 95% of requests <200ms
- Webhook delivery: 95% success rate

**Error Budget**:
- Monthly error budget: 43.2 minutes
- Track consumed budget daily
- Alert when 80% consumed

## Cost Monitoring

### Cloud Cost Tracking

**Metrics to Track**:
- EC2/Compute costs
- RDS/Database costs
- S3 storage costs
- Data transfer costs
- Total monthly cost

**Alerts**:
- Monthly cost >$500
- Daily cost >$20
- S3 storage >1TB

## Monitoring Checklist

### Pre-Production

- [ ] Prometheus installed and configured
- [ ] Grafana dashboards created
- [ ] Alertmanager configured
- [ ] Alerts defined and tested
- [ ] Log aggregation configured
- [ ] Tracing enabled
- [ ] Health checks working
- [ ] On-call rotation defined

### Post-Deployment

- [ ] Verify metrics are being collected
- [ ] Verify dashboards display data
- [ ] Verify alerts fire correctly
- [ ] Verify logs are aggregated
- [ ] Verify traces are collected
- [ ] Test on-call notifications
- [ ] Document runbooks for common alerts

## References

- [Infrastructure](infrastructure.md) - Deployment architecture
- [Security Checklist](security-checklist.md) - Security monitoring
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)

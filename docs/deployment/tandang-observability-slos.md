# Gotong-Tandang Observability SLOs

Last updated: 2026-02-17

This document defines the minimum dashboard and alert set for native Gotong-Tandang integration.

## 1. Required metrics

Webhook delivery:
- `gotong_worker_webhook_delivery_total{result,status_code}`
- `gotong_worker_webhook_delivery_duration_ms{result,status_code}`
- `gotong_worker_webhook_dead_letter_total`

Queue health:
- `gotong_worker_queue_ready_total`
- `gotong_worker_queue_delayed_total`
- `gotong_worker_queue_processing_total`
- `gotong_worker_queue_lag_ms`

Read-side integration:
- `gotong_api_markov_integration_errors_total{reason}`

## 2. Dashboard panels

Create a dashboard named `gotong-tandang-integration`.

Panels:
1. Webhook success rate (5m):
   - `sum(rate(gotong_worker_webhook_delivery_total{result="success"}[5m])) / sum(rate(gotong_worker_webhook_delivery_total[5m]))`
2. Webhook retryable failures (5m):
   - `sum(rate(gotong_worker_webhook_delivery_total{result="retryable_failure"}[5m]))`
3. Webhook terminal failures (5m):
   - `sum(rate(gotong_worker_webhook_delivery_total{result="terminal_failure"}[5m]))`
4. Webhook delivery p95 latency:
   - `histogram_quantile(0.95, sum(rate(gotong_worker_webhook_delivery_duration_ms_bucket[5m])) by (le))`
5. DLQ depth:
   - `gotong_worker_webhook_dead_letter_total`
6. Markov read errors by reason:
   - `sum(rate(gotong_api_markov_integration_errors_total[5m])) by (reason)`
7. Queue lag:
   - `gotong_worker_queue_lag_ms`

## 3. Alert rules

Alert `GotongTandangWebhookSuccessRateLow`:
- Trigger: webhook success rate `< 0.99` for `15m`
- Severity: warning

Alert `GotongTandangWebhookTerminalFailuresHigh`:
- Trigger: `sum(rate(gotong_worker_webhook_delivery_total{result="terminal_failure"}[5m])) > 0.05`
- For: `10m`
- Severity: critical

Alert `GotongTandangDLQGrowing`:
- Trigger: `gotong_worker_webhook_dead_letter_total > 0`
- For: `10m`
- Severity: warning

Alert `GotongTandangMarkovReadErrorsSustained`:
- Trigger: `sum(rate(gotong_api_markov_integration_errors_total[5m])) > 0.1`
- For: `15m`
- Severity: warning

## 4. SLO targets

- Webhook delivery success: `>= 99%` (rolling 30 days)
- Webhook p95 latency: `< 5s`
- Markov read error rate: `< 1%` of read traffic
- DLQ depth steady-state: `0`

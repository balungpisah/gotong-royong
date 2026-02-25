# Pack C Stage A Go/No-Go Report

Date: 2026-02-25T09:40:37Z
Stage: `stage-a`
Prometheus URL: `http://127.0.0.1:9090`
Window: `30m`
Step: `60s`
Mode: `dry-run`

## Decision Summary

| Decision | PASS | WARN | FAIL |
|---|---:|---:|---:|
| DRY_RUN | 0 | 0 | 0 |

Recommendation: Dry-run only. Execute without --dry-run against Prometheus for a real decision.

## Signal Evaluation

| Signal | Latest | Max | Warn threshold | Crit threshold | Warn streak | Crit streak | Status | Notes |
|---|---:|---:|---|---|---:|---:|---|---|
| Edge error ratio | n/a | n/a | >0.001 for 10m | >0.005 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Edge partial ratio | n/a | n/a | >0.003 for 10m | >0.01 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Shadow mismatch increase[30m] | n/a | n/a | >3 for 5m | >10 for 5m | 0/5 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Feed p95 latency (seconds) | n/a | n/a | >0.18 for 15m | >0.25 for 10m | 0/15 | 0/10 | SKIPPED | dry-run (no Prometheus query executed) |
| Feed 5xx ratio | n/a | n/a | >0.01 for 10m | >0.02 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |

Policy reference:
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`

Notes:
- GO requires zero sustained critical and warning breaches for this observation window.
- NO_GO is emitted when any critical threshold is sustained for its configured `for` duration.
- HOLD blocks progression without forcing immediate rollback.

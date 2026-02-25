# Pack C Stage C Go/No-Go Report

Date: 2026-02-25T09:52:30Z
Stage: `stage-c`
Prometheus URL: `http://127.0.0.1:9090`
Window: `24h`
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
| Edge error ratio | n/a | n/a | >0.0001 for 10m | >0.0005 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Edge partial ratio | n/a | n/a | >0.0005 for 10m | >0.002 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Feed p95 latency (seconds) | n/a | n/a | >0.18 for 15m | >0.25 for 10m | 0/15 | 0/10 | SKIPPED | dry-run (no Prometheus query executed) |
| Feed 5xx ratio | n/a | n/a | >0.01 for 10m | >0.02 for 5m | 0/10 | 0/5 | SKIPPED | dry-run (no Prometheus query executed) |
| Fallback/legacy lane RPS | n/a | n/a | n/a | >0 for 15m | n/a | 0/15 | SKIPPED | dry-run (no Prometheus query executed) |

Policy reference:
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`

Notes:
- GO requires zero sustained critical and warning breaches for this observation window.
- NO_GO is emitted when any critical threshold is sustained for its configured `for` duration.
- HOLD blocks progression without forcing immediate rollback.

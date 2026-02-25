# Pack C Stage B Go/No-Go Report

Date: 2026-02-25T10:15:16Z
Stage: `stage-b`
Prometheus URL: `http://127.0.0.1:9090`
Window: `4h`
Step: `60s`
Mode: `live`

## Decision Summary

| Decision | PASS | WARN | FAIL |
|---|---:|---:|---:|
| ERROR | 0 | 0 | 0 |

Recommendation: Prometheus query failed. Fix connectivity/auth and rerun.

## Query Error

`<urlopen error [Errno 61] Connection refused>`

## Signal Evaluation

| Signal | Latest | Max | Warn threshold | Crit threshold | Warn streak | Crit streak | Status | Notes |
|---|---:|---:|---|---|---:|---:|---|---|

Policy reference:
- `docs/deployment/feed-involvement-fallback-alert-thresholds.md`
- `docs/deployment/feed-involvement-fallback-removal-runbook.md`

Notes:
- GO requires zero sustained critical and warning breaches for this observation window.
- NO_GO is emitted when any critical threshold is sustained for its configured `for` duration.
- HOLD blocks progression without forcing immediate rollback.

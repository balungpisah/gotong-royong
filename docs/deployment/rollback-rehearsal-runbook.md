# Rollback Rehearsal Runbook (Realtime + Surreal Beta Guardrail)

Last updated: `2026-02-16`

## Objective

Validate that rollback from a release candidate works without dropping realtime delivery or violating
Surreal beta safety constraints.

## Precondition

- Staging has two API replicas behind a load balancer.
- SurrealDB pinned to `3.0.0` with TiKV.
- Redis available for idempotency and realtime fanout.
- `CHAT_REALTIME_TRANSPORT=redis` on all API replicas.
- Shared channel prefix configured (`CHAT_REALTIME_CHANNEL_PREFIX`).
- Baseline load test running or replay-safe synthetic check available.

## Rehearsal Steps

1. **Capture baseline evidence**
   - Record:
     - API image SHA and commit.
     - SurrealDB image tag and checksum.
     - Migration checksum from migration runbook or release artifact.
     - `CHAT_REALTIME_CHANNEL_PREFIX` in use.
   - Run:
     - `just release-gates-surreal`
     - `cargo test -p gotong-api --test '*'`
   - Export pass/fail and timestamps to a rehearsal log.

2. **Validate realtime fanout before rollback**
   - Open two websocket/chat stream sessions to two different API replicas using same `thread_id`.
   - Send a message from one replica and verify both receivers observe the event.
   - Record event IDs and receive timestamps for both sessions.

3. **Introduce controlled issue**
   - Deploy release-candidate image as “blue-next” replica set behind LB.
   - Route 10–20% synthetic traffic, keep old image healthy.
   - Ensure no spike in:
     - 5xx rates,
     - websocket disconnect error bursts,
     - idempotency collision anomalies.

4. **Trigger rollback condition**
 - If any P0 gate or guardrail is violated, trigger rollback immediately:
   - Remove `blue-next` from service selectors.
   - Repoint all traffic to the last stable image.
   - Keep Redis and Surreal on the same runtime profile.

5. **Rollback verification**
   - Re-run:
     - `just release-gates-surreal`
     - two-replica realtime fanout check from step 2.
   - Confirm:
     - same `CHAT_REALTIME_CHANNEL_PREFIX` across both old image replicas,
     - both replicas can read stream events for same thread,
     - ordering and replay checks remain monotonic.

6. **Post-rehearsal cleanup**
   - Remove temporary synthetic data used for fanout verification.
   - Keep logs for:
     - command output,
     - deployment timestamps,
     - replica traffic distribution,
     - replay/fanout sample payload IDs.

## Evidence Checklist

- [ ] Surreal guardrail script succeeded (`just release-gates-surreal`).
- [ ] API rollback path executed with bounded RTO.
- [ ] Realtime events still delivered across replicas after rollback.
- [ ] No data loss in chat stream order or catch-up replay for synthetic thread.
- [ ] Incident report stored with command output and evidence IDs.

## Exit Criteria

Rehearsal is considered complete only when:
- all evidence checklist items are checked,
- rollback can be executed and verified in the planned window,
- no open follow-up blockers remain for PR-15 hardening tasks.

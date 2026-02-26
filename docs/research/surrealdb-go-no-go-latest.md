# SurrealDB Pattern Sampling Report

Date: 2026-02-25T20:16:23Z
Environment: onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
3.0.0 for linux on aarch64
Namespace/DB: gotong_probe/chat

## Objective
Probe key backend patterns with a runnable SurrealDB sample before implementation planning.

## Result Summary
| Pattern | Result | What was checked |
|---|---|---|
| Idempotent write guard (unique entity_id + request_id) | PASS | Duplicate write blocked by unique composite index |
| Deterministic timeline ordering + catch-up query | PASS | ORDER BY created_at, message_id and reconnect predicate |
| Live stream behavior | PASS | WS live subscription receives create event |
| Live diff payload contract | PASS | DIFF stream includes change operation for body update |
| Permission-filtered live subscription | PASS | Record auth sees own rows and not other owners' rows |

## Pattern 1: Idempotent write guard
Command pattern:
DEFINE INDEX uniq_entity_request ON TABLE chat_delivery_event FIELDS entity_id, request_id UNIQUE;

First insert output:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:yr0ssd4q6ffnvshiotzu","occurred_at":"2026-02-25T20:16:27.256238595Z","request_id":"req-123"}]]
~~~

Duplicate insert output:
~~~json
["Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `chat_delivery_event:yr0ssd4q6ffnvshiotzu`"]
~~~

Rows after duplicate attempt:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:yr0ssd4q6ffnvshiotzu","occurred_at":"2026-02-25T20:16:27.256238595Z","request_id":"req-123"}]]
~~~

## Pattern 2: Deterministic ordering and reconnect catch-up
Ordered query output:
~~~json
[[{"body":"first","created_at":"2026-02-15T03:30:00Z","message_id":"msg-001"},{"body":"second","created_at":"2026-02-15T03:30:00Z","message_id":"msg-002"},{"body":"third","created_at":"2026-02-15T03:30:00Z","message_id":"msg-003"}]]
~~~

Catch-up query output:
~~~json
[[{"body":"second","created_at":"2026-02-15T03:30:00Z","message_id":"msg-002"},{"body":"third","created_at":"2026-02-15T03:30:00Z","message_id":"msg-003"}]]
~~~

Derived order:
- Ordered IDs: msg-001,msg-002,msg-003
- Catch-up IDs after cursor (created_at=03:30:00Z, message_id=msg-001): msg-002,msg-003

## Pattern 3: Live stream behavior (protocol detail)
HTTP endpoint live query result:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
Unable to perform the realtime query
~~~

WS endpoint live query result:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'059329a6-80d4-4989-99b9-f6c2f51f2db9']


{ action: 'CREATE', id: u'059329a6-80d4-4989-99b9-f6c2f51f2db9', result: { author_id: 'user:live', body: 'live event', created_at: d'2026-02-25T20:16:31.675153555Z', id: chat_message:317h1ue548h3ksuottxs, message_id: 'msg-live-1', thread_id: 'thread:live' } }
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Pattern 4: Live diff payload contract
WS DIFF live query result:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'2e4de488-fdbb-4e05-bc24-0e3cfccbf332']


{ action: 'CREATE', id: u'2e4de488-fdbb-4e05-bc24-0e3cfccbf332', result: [{ op: 'replace', path: '', value: { author_id: 'user:diff', body: 'hello', id: chat_message:rx8qeljhzxibyv4kud5a, message_id: 'msg-diff-1', thread_id: 'thread:diff' } }] }


{ action: 'UPDATE', id: u'2e4de488-fdbb-4e05-bc24-0e3cfccbf332', result: [{ op: 'change', path: '/body', value: '@@ -1,5 +1,12 @@\n hello\n+ edited\n' }] }
~~~

Observation:
- DIFF stream returned change entries for updated fields (/body).

## Pattern 5: Permission-filtered live subscriptions
Token auth setup:
- Alice record id: user:6t0u07wbllcyee7ibjh5
- Bob record id: user:328851eeery3l9a99oxj

Alice live query result while inserting Bob then Alice rows:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'2f42b603-8d77-4b0e-8041-962ac97f3470']


{ action: 'CREATE', id: u'2f42b603-8d77-4b0e-8041-962ac97f3470', result: { body: 'alice private', id: chat_private:owkh24tneb232i10b1jc, owner: user:6t0u07wbllcyee7ibjh5 } }
~~~

Observation:
- Alice subscription received only Alice-owned row and did not receive Bob-owned row.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
3.0.0 for linux on aarch64; this matches locked target 3.0.0.
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.

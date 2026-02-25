# SurrealDB Pattern Sampling Report

Date: 2026-02-25T06:10:48Z
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
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:b9w7jpgpea1o2njxzqzp","occurred_at":"2026-02-25T06:10:52.925010877Z","request_id":"req-123"}]]
~~~

Duplicate insert output:
~~~json
["Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `chat_delivery_event:b9w7jpgpea1o2njxzqzp`"]
~~~

Rows after duplicate attempt:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:b9w7jpgpea1o2njxzqzp","occurred_at":"2026-02-25T06:10:52.925010877Z","request_id":"req-123"}]]
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
[u'e38bdfaa-d957-4f5e-809b-e5c5507929c8']


{ action: 'CREATE', id: u'e38bdfaa-d957-4f5e-809b-e5c5507929c8', result: { author_id: 'user:live', body: 'live event', created_at: d'2026-02-25T06:10:57.955107754Z', id: chat_message:c8575jjsqn09w9jhqfq7, message_id: 'msg-live-1', thread_id: 'thread:live' } }
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Pattern 4: Live diff payload contract
WS DIFF live query result:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'af82961c-8333-4103-a67a-232c5eb3be11']


{ action: 'CREATE', id: u'af82961c-8333-4103-a67a-232c5eb3be11', result: [{ op: 'replace', path: '', value: { author_id: 'user:diff', body: 'hello', id: chat_message:q7f2dijm2r1iqolgffeh, message_id: 'msg-diff-1', thread_id: 'thread:diff' } }] }


{ action: 'UPDATE', id: u'af82961c-8333-4103-a67a-232c5eb3be11', result: [{ op: 'change', path: '/body', value: '@@ -1,5 +1,12 @@\n hello\n+ edited\n' }] }
~~~

Observation:
- DIFF stream returned change entries for updated fields (/body).

## Pattern 5: Permission-filtered live subscriptions
Token auth setup:
- Alice record id: user:mfl9qq10sp15g05lnzsz
- Bob record id: user:f5ln5a8e45ptf77iiab0

Alice live query result while inserting Bob then Alice rows:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'3aa804ae-c9e3-4b8c-a3ed-1fa6d8232dad']


{ action: 'CREATE', id: u'3aa804ae-c9e3-4b8c-a3ed-1fa6d8232dad', result: { body: 'alice private', id: chat_private:11e84qo70fccp49dsrfm, owner: user:mfl9qq10sp15g05lnzsz } }
~~~

Observation:
- Alice subscription received only Alice-owned row and did not receive Bob-owned row.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
3.0.0 for linux on aarch64; this matches locked target 3.0.0.
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.

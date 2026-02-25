# SurrealDB Pattern Sampling Report

Date: 2026-02-25T18:04:26Z
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
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:altmzbfl2skmc7mnfcfn","occurred_at":"2026-02-25T18:04:29.657196541Z","request_id":"req-123"}]]
~~~

Duplicate insert output:
~~~json
["Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `chat_delivery_event:altmzbfl2skmc7mnfcfn`"]
~~~

Rows after duplicate attempt:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:altmzbfl2skmc7mnfcfn","occurred_at":"2026-02-25T18:04:29.657196541Z","request_id":"req-123"}]]
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
[u'c9ff5b56-2cbf-4c1e-8a04-1755756d9552']


{ action: 'CREATE', id: u'c9ff5b56-2cbf-4c1e-8a04-1755756d9552', result: { author_id: 'user:live', body: 'live event', created_at: d'2026-02-25T18:04:33.336480918Z', id: chat_message:7772fs0kcgnp3o4iijwu, message_id: 'msg-live-1', thread_id: 'thread:live' } }
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Pattern 4: Live diff payload contract
WS DIFF live query result:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'e6e9af31-5655-4523-a6ed-93df9c587693']


{ action: 'CREATE', id: u'e6e9af31-5655-4523-a6ed-93df9c587693', result: [{ op: 'replace', path: '', value: { author_id: 'user:diff', body: 'hello', id: chat_message:xjnmi9amd0lyaeaussqw, message_id: 'msg-diff-1', thread_id: 'thread:diff' } }] }


{ action: 'UPDATE', id: u'e6e9af31-5655-4523-a6ed-93df9c587693', result: [{ op: 'change', path: '/body', value: '@@ -1,5 +1,12 @@\n hello\n+ edited\n' }] }
~~~

Observation:
- DIFF stream returned change entries for updated fields (/body).

## Pattern 5: Permission-filtered live subscriptions
Token auth setup:
- Alice record id: user:gp089iopcqlc332oe2r9
- Bob record id: user:1ytbnej10gwqxb5s5yrw

Alice live query result while inserting Bob then Alice rows:
~~~text
onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
[u'729660e2-f938-4f9e-8e83-d3bb97aaa44e']


{ action: 'CREATE', id: u'729660e2-f938-4f9e-8e83-d3bb97aaa44e', result: { body: 'alice private', id: chat_private:55r8foou2vfslt243141, owner: user:gp089iopcqlc332oe2r9 } }
~~~

Observation:
- Alice subscription received only Alice-owned row and did not receive Bob-owned row.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server onnxruntime cpuid_info warning: Unknown CPU vendor. cpuinfo_vendor value: 0
3.0.0 for linux on aarch64; this matches locked target 3.0.0.
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.

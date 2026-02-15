# SurrealDB Pattern Sampling Report

Date: 2026-02-15T03:27:40Z
Environment: 2.3.10 for macos on aarch64
Namespace/DB: gotong_probe/chat

## Objective
Probe key backend patterns with a runnable SurrealDB sample before implementation planning.

## Result Summary
| Pattern | Result | What was checked |
|---|---|---|
| Idempotent write guard (unique entity_id + request_id) | PASS | Duplicate write blocked by unique composite index |
| Deterministic timeline ordering + catch-up query | PASS | ORDER BY created_at, message_id and reconnect predicate |
| Live stream behavior | PASS | WS live subscription receives create event |

## Pattern 1: Idempotent write guard
Command pattern:
DEFINE INDEX uniq_entity_request ON TABLE chat_delivery_event FIELDS entity_id, request_id UNIQUE;

First insert output:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:q5iz53ikeuh1qv3a83lt","occurred_at":"2026-02-15T03:27:41.405903Z","request_id":"req-123"}]]
~~~

Duplicate insert output:
~~~json
["Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `chat_delivery_event:q5iz53ikeuh1qv3a83lt`"]
~~~

Rows after duplicate attempt:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:q5iz53ikeuh1qv3a83lt","occurred_at":"2026-02-15T03:27:41.405903Z","request_id":"req-123"}]]
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
There was a problem with the database: The protocol or storage engine does not support live queries on this architecture
~~~

WS endpoint live query result:
~~~text
[u'39578bf1-26c0-4548-8abf-391581d4de80']


{ action: 'CREATE', id: u'39578bf1-26c0-4548-8abf-391581d4de80', result: { author_id: 'user:live', body: 'live event', created_at: d'2026-02-15T03:27:43.130132Z', id: chat_message:iz4mvp7ajo0dhx270von, message_id: 'msg-live-1', thread_id: 'thread:live' } }
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server 2.3.10 for macos on aarch64 (not the locked v3.0.0-beta-4 target).
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.

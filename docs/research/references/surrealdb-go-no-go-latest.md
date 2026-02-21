# SurrealDB Pattern Sampling Report

Date: 2026-02-15T17:38:13Z
Environment: 3.0.0-beta.4 for macos on aarch64
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
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:t3hitwm8yc3w9531vgxv","occurred_at":"2026-02-15T17:38:15.178006Z","request_id":"req-123"}]]
~~~

Duplicate insert output:
~~~json
["Thrown error: Database index `uniq_entity_request` already contains ['thread:alpha', 'req-123'], with record `chat_delivery_event:t3hitwm8yc3w9531vgxv`"]
~~~

Rows after duplicate attempt:
~~~json
[[{"correlation_id":"corr-1","entity_id":"thread:alpha","id":"chat_delivery_event:t3hitwm8yc3w9531vgxv","occurred_at":"2026-02-15T17:38:15.178006Z","request_id":"req-123"}]]
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
Thrown error: Unable to perform the realtime query
~~~

WS endpoint live query result:
~~~text
[u'0b4e0e3c-560f-4f34-bdfe-1a1ba9de0dfe']


{ action: 'CREATE', id: u'0b4e0e3c-560f-4f34-bdfe-1a1ba9de0dfe', result: { author_id: 'user:live', body: 'live event', created_at: d'2026-02-15T17:38:17.972202Z', id: chat_message:orfr74fy4ops5drj5p98, message_id: 'msg-live-1', thread_id: 'thread:live' } }
~~~

Observation:
- In this environment, live query streaming worked over WS and not over HTTP.

## Pattern 4: Live diff payload contract
WS DIFF live query result:
~~~text
[u'4ebbf9b8-c5c0-4839-bcce-af1f00b4b57d']


{ action: 'CREATE', id: u'4ebbf9b8-c5c0-4839-bcce-af1f00b4b57d', result: [{ op: 'replace', path: '', value: { author_id: 'user:diff', body: 'hello', id: chat_message:pdkfn56szcqc2v1svpbi, message_id: 'msg-diff-1', thread_id: 'thread:diff' } }] }


{ action: 'UPDATE', id: u'4ebbf9b8-c5c0-4839-bcce-af1f00b4b57d', result: [{ op: 'change', path: '/body', value: '@@ -1,5 +1,12 @@\n hello\n+ edited\n' }] }
~~~

Observation:
- DIFF stream returned change entries for updated fields (/body).

## Pattern 5: Permission-filtered live subscriptions
Token auth setup:
- Alice record id: user:d3c4cb219yo41wbnu8hs
- Bob record id: user:qn6ak8wkr24eipuailjf

Alice live query result while inserting Bob then Alice rows:
~~~text
[u'5d6472de-3854-4bbc-aec9-4c658ce79504']


{ action: 'CREATE', id: u'5d6472de-3854-4bbc-aec9-4c658ce79504', result: { body: 'alice private', id: chat_private:81lbyx9pujsii9rewe5x, owner: user:d3c4cb219yo41wbnu8hs } }
~~~

Observation:
- Alice subscription received only Alice-owned row and did not receive Bob-owned row.

## Notes for Stack Lock
- This probe validates core data patterns and live-stream behavior relevant to chat workloads.
- Local runtime here is SurrealDB CLI/server 3.0.0-beta.4 for macos on aarch64; this matches locked target 3.0.0-beta.4.
- Re-run this same probe against the pinned beta runtime during implementation bootstrap.

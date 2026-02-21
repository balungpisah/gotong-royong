# Tandang Compatibility Gap Log (Research Phase 2)

Track every shared feature that depends on Tandang contracts.

## Gap table

| Feature ID | Expected behavior | Tandang source reference | Status (`KNOWN` / `PARTIAL` / `UNKNOWN` / `MISSING` / `CONFLICT`) | Evidence from docs | Required decision |
|---|---|---|---|---|---|
| UI-01 | triage session capture and metadata correlation | `[tandang]docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md` (webhook pattern only) | PARTIAL | integration mode exists; triage session is not an explicit event type in payload docs | decide if triage metadata is emitted via `subject.metadata` on later events or via new event |
| UI-02 | classification persistence for seeds | `[tandang]docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md` + `docs/design/specs/ai-spec/05-ai-01-classification.md` | UNKNOWN | explicit track/seed fields are not documented in `subject.metadata` examples | define canonical `subject.metadata` keys for track/seed_type/skills_confidence |
| UI-03 | workflow transitions / lifecycle state machine | `docs/design/backend-design-contract-gotong-tandang.md` + `[tandang]docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md` (app-only write path) | KNOWN | integration guide lacks dedicated TrackStateTransition endpoint; contract is implemented app-owned with `track_state_transition` payload for local audit | fixed as app-owned canonical event: `track_state_transition`, idempotent `(entity_id, request_id)`, role and gate fields as specified |
| UI-04 | Proof of Reality evidence submit + verification | `docs/api/event-payloads.md`<br/>`[tandang]docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md`<br/>`[tandang]crates/api/src/routes/platforms.rs` | KNOWN | `por_evidence` exists and is ingested at `POST /platforms/gotong_royong/webhook` with proof fields | align Gotong PoR payload fields to local schema for image/video witness proofs |
| UI-05 | vouch command + weight_hint scoring | `docs/api/event-payloads.md`<br/>`[tandang]docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md`<br/>`[tandang]crates/api/src/dto/platform.rs` | KNOWN | `vouch_submitted` event contract and `ReputationQueried`/`weight_hint` path are explicit | document exact weight_hint score mapping in `Decision Log` before implementation |
| UI-06 | reputation UI fields (`I/C/J`, tier, score breakdown) | `[tandang]crates/api/src/routes/users/mod.rs`<br/>`[tandang]crates/api/src/dto/user.rs` | KNOWN | endpoint exists: `/api/v1/users/{platform_user_id}/reputation` | verify how `platform_user_id` format (`platform:user_id`) is normalized in UI calls |
| UI-07 | moderation status propagation to public feed | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | no moderation fields are represented in integration docs | decided as app-owned moderation source-of-truth; `moderation_state` persisted in Gotong DB and exposed by API/DB views with actor-role-limited visibility |
| UI-08 | emergency flow urgency state + follow-up tracking | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | no dedicated emergency payload in Tandang taxonomy | decided app-only event model with explicit names: `siaga_broadcast_created`, `siaga_broadcast_activated`, `siaga_broadcast_updated`, `siaga_responder_joined`, `siaga_broadcast_closed`, `siaga_broadcast_cancelled` |
| UI-09 | encrypted witness/secret-sealed lifecycle | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | no vault-seal event contract documented in integration guide | decided app-only persistence; only tombstone metadata (`sealed_hash`, state, timestamps) may be indexed |
| UI-10 | feed ranking and scope filters | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | no explicit feed/search contract in integration docs | decided app-owned read-model API with cursor pagination and privacy filter; Tandang used as read-only reputation signal |
| UI-11 | search and discovery results | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | search behavior documented in UI specs only | decided app-owned search index/projection; Tandang not authoritative for ranking |
| UI-12 | notifications and digest generation | `docs/design/backend-design-contract-gotong-tandang.md` | KNOWN | no notification endpoint documented for events in integration guide | decided app-owned notification and digest service, with event input contract from app bus |
| UI-13 | skill extraction + competency storage | `docs/design/specs/ui-ux-spec/30-esco-skills.md` (via Tandang skill model) | PARTIAL | skill identifiers exist but UI-facing validated/self-declared split needs mapping | define API fields for `validated=true|false` and decay metadata |
| UI-14 | onboarding and membership bootstrap | no membership endpoints in integration docs | UNKNOWN | onboarding is app-level user flow | decide whether scope/authority bootstrap is local DB only or persisted via user profile in Tandang |
| UI-15 | share/invite links | no direct share/invite endpoint in integration docs | UNKNOWN | invite tokens and OG assets appear only in UI docs | define share/invite resource ownership and audit for open links |
| UI-16 | recurring routines scheduling | no recurring event contract in Tandang docs | UNKNOWN | rutin is app domain scheduling behavior | decide whether recurrence is local only or also emitted as `ProblemScheduled` events |
| UI-17 | governance voting + quorum proof | no voting endpoint in integration docs | UNKNOWN | governance logic appears in design-only docs | decide whether Tandang event stream needs voting snapshots for audit replay |
| UI-18 | role assignment / permission checks | role model appears in app specs; no dedicated role API contract | UNKNOWN | no roles endpoints in current Tandang docs | define whether role change events are persisted through existing problem ACL tables or new event type |
| UI-19 | dispute/jury life-cycle | no jury event type in integration docs | UNKNOWN | dispute + jury flows are app-level in design docs | decide whether dispute and jury selections are represented via generic notes/events or custom type |
| UI-20 | LLM diff blocks and source-tag audit | no explicit source-tag events in Tandang docs | UNKNOWN | source-tag concept is app-level contract | decide whether AI edits are kept as local state only or emitted in audit metadata |
| UI-21 | fundraising ledger and finance freeze rules | no accounting contract in integration docs | PARTIAL | finance surface is in cross-cutting docs, but contribution/event schemas can carry `contribution_type` | define whether Galang transactions require custom contribution type and protected field redaction |
| UI-22 | amplifier (Siarkan) lifecycle and reach metrics | no Siarkan endpoint in Tandang docs | UNKNOWN | Siarkan is cross-cutting feature in app specs | decide whether reach tracking is internal metrics only or evented for analytics |
| UI-23 | duplicate similarity search and merge result | `docs/design/specs/ai-spec/07-ai-03-duplicate-detection.md` relies on vector index (Tandang) | PARTIAL | AI-03 expects embedding + vector search path; Tandang adapter mention in AI pipeline | confirm exact vector endpoint and whether similarity metadata is stored as `metadata` in triage/event payload |

## Priority tags

- `HIGH`: blocks API/flow design
- `MEDIUM`: can be deferred with explicit workaround
- `LOW`: documentation cleanup only

## Next action

1. Resolve `UNKNOWN` items with concrete schema examples and exact ownership (`app` vs `Tandang`) decisions.
2. Add references to exact source lines/files once confirmed.

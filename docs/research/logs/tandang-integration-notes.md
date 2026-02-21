# Tandang ↔ Gotong Royong Integration Research Notes

Status: in progress
Date: 2026-02-16

## Scope
- Tandang (Markov Credential Engine) APIs, adapters, integration guides
- Gotong Royong integration architecture, webhook/events, identity linking
- UI/UX touchpoints for reputation and CV (if specified)

## Notes

## Gotong Royong: system-overview.md
- Defines Gotong Royong as mutual credit platform with PoR evidence and native integration to Markov Engine.
- Integration points:
  - POST /api/v1/platforms/gotong_royong/webhook (Markov receives events)
  - GET /api/v1/users/{user_id}/reputation (GR queries reputation)
- Webhook publisher responsibilities: serialize events, sign with HMAC-SHA256, POST, retry, idempotency, DLQ.
- Event types: contribution_created, vouch_submitted, por_evidence.
- User management includes cached reputation from Markov.

## Gotong Royong: integration-architecture.md
- Integration mode: Native (webhooks GR→Markov + API reads GR←Markov). No DB access by Markov.
- Prereqs: shared secret; identity format gotong_royong:{user_id}; reputation GET accepts Markov UUID or platform-scoped ID.
- Detailed webhook flow (async queue, retries) and reputation query flow (Redis cache TTL 5m).
- Retry: max 5, exponential backoff, DLQ schema in doc.

## Gotong Royong: webhook-spec.md
- Webhook endpoint target: /v1/platforms/gotong_royong/webhook (env-specific URLs).
- Required headers: Content-Type, X-GR-Signature (sha256=hex).
- Signature computed on raw JSON bytes (compact serialization).
- Response format: 200 OK with processed/results; 400/401 permanent failures; 429/5xx retry.
- Idempotency via event_id; Markov should accept duplicates.
- Rate limits documented (but native platform effectively unlimited).
- Timeouts: client 10s; Markov may return 202 Accepted.

## Gotong Royong: event-payloads.md
- Defines JSON schemas for three webhook events: contribution_created, vouch_submitted, por_evidence.
- Common fields: event_id, event_type, schema_version, request_id, actor (user_id, username), subject, optional timestamp.
- Contribution payload supports contribution_type enum, title, description, evidence_url, skill_ids (max 10), metadata (max 50 keys).
- Vouch payload requires vouchee_id, optional skill_id, weight_hint (strong/moderate/light), message.
- PoR payload includes subject (contribution_id, evidence_type, evidence_data) and proof (timestamp, location, media_hash, witnesses).
- Validation rules summarized; references full PoR validation rules.

## Gotong Royong: authentication.md
- HMAC-SHA256 signature guidance for webhooks; minimum 32-char secret; dual-secret rotation.
- Must compute signature over raw request body bytes; constant-time compare for verification.
- Includes replay-attack timestamp validation guidance.

## Gotong Royong: data-flow.md
- End-to-end flows for task creation, completion, evidence submission, vouching, reputation query.
- Webhook enqueue after DB transaction; async worker signs and sends to Markov.
- Reputation read uses Markov GET /api/v1/users/{id}/reputation with Redis cache TTL 5m.
- PoR evidence validation rules and multi-perspective evidence multiplier described.

## Gotong Royong: schema-requirements.md
- Focused on SurrealDB v3 schema for chat and governance-related records; not directly about Markov integration.
- Highlights idempotent writes, append-only audit trails, and permission-aware access.

## Gotong Royong: por-evidence/evidence-format.md
- Defines three PoR evidence types and required fields (photo_with_timestamp, gps_verification, witness_attestation).
- Specifies hash computation, EXIF extraction, GPS capture, witness fields.
- Describes evidence quality scoring concept and reputation multiplier.

## Gotong Royong: por-evidence/validation-rules.md
- Validation rules for timestamps (<=30 days, RFC3339), media_hash length/hex, GPS bounds, witnesses array.
- Includes payload size limits and error response format.
- Emphasizes security validation and parameterized queries.

## Gotong Royong: backend-design-contract-gotong-tandang.md
- Defines required Tandang compatibility contract: platform_id gotong_royong, integration_mode Native, behavior_category task_based.
- Reiterates webhook endpoint contract, idempotency requirements, error codes, security/signing, event schema minimums.
- Requires user mapping to markov_user_id (platform-scoped), event outbox + delivery log.
- AI touchpoints: typed I/O, versioned prompts, AI-00 triage and AI-01 classification with fallbacks.
- Calls out open questions (schema_version, proof timestamp policy, vouch default weight, service-account writes).

## Gotong Royong: UI-UX reputation-ui-contract.md
- Specifies UI widgets for tier badge, I/C/J axes, GDF weather, CV Hidup sections, and avatar warmth.
- Implies data needed from Markov: tier, I/C/J, GDF weather, CV content, skill validation, vouch lists.

## Gotong Royong: AI specs (selected)
- AI-SPEC v0.2 is an index; key sections read: AI-00, AI-01, AI-03, Tandang-handled AI, three-layer architecture, model selection.
- Tandang-handled AI: embeddings/semantic search, real-time feeds, consensus voting, financial calculations, audit logs, geo indexing, roles/permissions are owned by Tandang.
- Three-layer architecture: Backend layer → Tandang integration layer (feature vectors, caching, fallback) → AI decision layer (LLM/ML).
- AI-00: conversational triage triggered by [+] compose button; calls AI-01; outputs entry_flow, track, seed_type, esco_skills; UX states and fallback.
- AI-01: triple refinement, extracts 0–3 ESCO skills; track hint auto-generated by AI (not fixed); used by AI-00; outputs structured fields.
- AI-03: duplicate detection uses Tandang vector search and geo filter; thresholds for UX behavior.
- Model selection: AI-00 Sonnet-class; AI-01 Haiku-class; AI-03 uses embedding model via Tandang.

## Tandang: GOTONG-ROYONG-INTEGRATION-GUIDE.md
- Confirms platform_id gotong_royong, integration mode Native, behavior_category task_based, supported event types + PoR evidence types.
- Webhook endpoint: POST /api/v1/platforms/gotong_royong/webhook (note /api/v1 prefix here).
- Describes adapter flow: signature verify → parse event → validate PoR proof → DomainCommand → IngestionService.
- Notes celebrate track composition uses existing Tandang primitives (consensus, verification models).
- Mentions witness complexity estimation and override endpoints as supporting capabilities.
- Provides testing instructions and fixtures.

## Tandang: API-PLATFORMS.md
- Platform discovery endpoints: GET /platforms, GET /platforms/{platform_id}, GET /platforms/{platform_id}/commands.
- Account linking endpoints: POST /account/link, GET /account/links, DELETE /account/links/{link_id}.
- Webhook endpoint: POST /platforms/{platform_id}/webhook; signature-based auth.
- Direct ingestion endpoint: POST /platforms/{platform_id}/ingest (JWT).
- OpenAPI spec at GET /api-docs/openapi.json; swagger at /swagger-ui.
- Documents error codes and rate limits.

## Tandang: PLATFORM-INTEGRATION.md
- Describes account linking flows (verification codes, OAuth) and bot commands for other platforms.
- Gotong Royong is native, no bot commands, uses HMAC webhooks with PoR evidence.
- Reiterates webhook payload examples for gotong_royong.

## Tandang API routes: platforms.rs (routes summary)
- Endpoints (adapter dispatch for all platforms):
  - GET /api/v1/platforms
  - GET /api/v1/platforms/{platform_id}
  - GET /api/v1/platforms/{platform_id}/commands
  - POST /api/v1/platforms/{platform_id}/webhook
  - POST /api/v1/platforms/{platform_id}/bot-command
  - GET /api/v1/platforms/{platform_id}/linked-users
  - POST /api/v1/platforms/{platform_id}/poll-ingest
  - POST /api/v1/platforms/{platform_id}/ingest
- Webhook headers include x-gr-signature and optional x-gr-event; event type can come from payload.
- Note: file output truncated after mid-file, but endpoint list is explicitly documented at top.

## Tandang API routes: users/mod.rs
- GET /api/v1/users/{id} (UserResponse), privacy-checked.
- GET /api/v1/users/{id}/reputation accepts UUID or platform-scoped id {platform}:{user_id}; uses AccountLinkRepository for lookup.
- GET /api/v1/users/{id}/tier
- GET /api/v1/users/{id}/activity
- GET /api/v1/users (list with filters)

## Tandang API routes: reputation/mod.rs
- POST /api/v1/reputation/calculate (Keystone-only)
- GET /api/v1/reputation/distribution
- GET /api/v1/reputation/novice-advancement
- GET /api/v1/reputation/leaderboard
- GET /api/v1/reputation/me/rankings

## Tandang API routes: cv_hidup/mod.rs
- GET /api/v1/cv-hidup/{user_id}
- GET /api/v1/cv-hidup/me
- POST /api/v1/cv-hidup/export
- GET /api/v1/cv-hidup/qr
- GET /api/v1/cv-hidup/settings
- PUT /api/v1/cv-hidup/settings
- GET /api/v1/cv-hidup/verify/{export_id}

## Tandang API routes: skills.rs
- POST /api/v1/skills/suggest (uses TandangSkillApiClient)
- POST /api/v1/skills/estimate-complexity (witness complexity)
- GET /api/v1/skills/nodes/{id}
- GET /api/v1/skills/{id}/parent
- GET /api/v1/skills/nodes/{id}/labels
- GET /api/v1/skills/nodes/{id}/relations
- GET /api/v1/skills/search

## Tandang API routes: por.rs
- POST /api/v1/por/validate
- GET /api/v1/por/requirements/{task_type}
- GET /api/v1/por/triad-requirements/{track}/{transition}
- POST /api/v1/por/submit
- GET /api/v1/por/status/{evidence_id}
- POST /api/v1/por/check-requirements

## Tandang API routes: account_link.rs
- POST /api/v1/account/link
- GET /api/v1/account/links
- DELETE /api/v1/account/links/{link_id}
- Account linking flow uses verification code; 14-day expiry.

## Tandang API routes: auth.rs
- POST /api/v1/auth/register
- POST /api/v1/auth/login
- POST /api/v1/auth/refresh
- POST /api/v1/auth/logout
- POST /api/v1/auth/verify-email
- POST /api/v1/auth/forgot-password
- POST /api/v1/auth/reset-password

## Tandang API routes: me.rs
- GET /api/v1/me
- PUT /api/v1/me/profile
- GET /api/v1/me/sessions
- DELETE /api/v1/me/sessions/{session_id}

## Tandang adapter: crates/infrastructure/src/adapters/gotong_royong.rs
- HMAC verification with header format sha256=..., constant-time compare.
- Event handling:
  - contribution_created → DomainCommand::CreateContribution with platform_user_id "gotong_royong:{user_id}" and metadata.
  - vouch_submitted → DomainCommand::CreateVouch with voucher/vouchee platform IDs.
  - por_evidence → validates proof, then DomainCommand::SubmitVerification outcome "approved" with evidence payload.
- PoR validation rules enforced: timestamp <= 30 days; media_hash hex; GPS lat/lon bounds; witnesses non-empty.
- parse_bot_command not supported (native integration only).
- parse_ingestion uses PlatformMetadata with event_type to reuse webhook parsing.

## Tandang ingestion + account linking (crates/application/src/services/ingestion.rs, account_link.rs)
- IngestionService resolves platform_user_id → markov_user_id via AccountLinkService.
- For direct ingestion (JWT), it enforces ownership: linked account must belong to requesting user.
- execute_single_command resolves platform users for CreateContribution/CreateVouch/SubmitVerification.
- Account links are created via verification code flow; resolve() fails if no verified link.
- Implication: for webhook events using platform_user_id (gotong_royong:user_id), a verified account link must exist or ingestion will error unless there is special-case handling (not seen in snippets).

## Gotong Royong domain: webhook.rs + ports/webhook.rs
- Defines WebhookOutboxEvent with required fields: event_id, event_type, actor.user_id/username, payload, request_id, correlation_id, status.
- WebhookOutboxEvent::new validates event_id/event_type/actor.user_id present (payload must include these).
- WebhookOutboxRepository supports create/get/list/update + delivery logs.

## Gotong Royong API routes (mod.rs excerpt)
- Admin endpoints for webhook outbox: list events, get event by id, list logs (admin-only).
- enqueue_webhook_outbox_event persists event and enqueues WebhookRetry job; uses webhook_max_attempts from config.

## Gotong Royong worker: crates/worker/src/main.rs
- Webhook retry worker sends signed HTTP POST to config.webhook_markov_url with X-GR-Signature; classifies responses and updates outbox + logs.
- Backoff + retry logic built in; marks Delivered/Retrying/DeadLetter accordingly.

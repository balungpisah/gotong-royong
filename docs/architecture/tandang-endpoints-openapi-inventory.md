# Tandang API — OpenAPI Inventory (Generated)

Generated: `2026-02-24T14:40:23+00:00`

Source: `/Users/damarpanuluh/MERIDIAN-NEW/tandang/markov-engine/contracts/openapi/api-v1.openapi.json`


Notes:
- This is generated from Tandang's OpenAPI contract.
- Some endpoints' real auth behavior may differ from the contract; treat Tandang router middleware as source of truth.


## Admin

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/admin/ui` | `bearerAuth` | Serve the admin dashboard UI. |
| `GET` | `/api/v1/admin/audit-log` | `bearerAuth` | Search audit log entries |
| `GET` | `/api/v1/admin/audit-log/export` | `bearerAuth` | Export audit log entries as CSV for compliance reporting |
| `POST` | `/api/v1/admin/bootstrap` | `public/unspecified` | Bootstrap the first administrative account (idempotent). |
| `GET` | `/api/v1/admin/config` | `bearerAuth` | Get current system configuration. |
| `PUT` | `/api/v1/admin/config` | `bearerAuth` | Update system configuration (in-memory). |
| `GET` | `/api/v1/admin/genesis/applications` | `bearerAuth` | List genesis applications. |
| `POST` | `/api/v1/admin/jobs/capture-domain-snapshots` | `bearerAuth` | Capture daily domain efficiency snapshots (Admin role + Keystone tier) |
| `POST` | `/api/v1/admin/jobs/hero/reset-weekly` | `bearerAuth` | Reset weekly hero points for all users (Admin role + Keystone tier) |
| `POST` | `/api/v1/admin/jobs/process-auto-verify` | `bearerAuth` | Process lifecycle timeouts (Admin role + Keystone tier) |
| `POST` | `/api/v1/admin/jobs/process-market-adjustment` | `bearerAuth` | Apply market adjustment to long-open problems (Admin role + Keystone tier) |
| `POST` | `/api/v1/admin/jobs/recalc-domain-efficiency` | `bearerAuth` | POST /api/v1/admin/jobs/recalc-domain-efficiency |
| `GET` | `/api/v1/admin/metrics` | `bearerAuth` | Get system metrics summary. |
| `POST` | `/api/v1/admin/platform-service-tokens/issue` | `bearerAuth` | issue_platform_service_token |
| `POST` | `/api/v1/admin/platform-service-tokens/rotate` | `bearerAuth` | rotate_platform_service_token |
| `GET` | `/api/v1/admin/slash/cases` | `bearerAuth` | List pending slash cases. |
| `GET` | `/api/v1/admin/users` | `bearerAuth` | List cached users/reputations. |
| `GET` | `/api/v1/admin/users/{user_id}` | `bearerAuth` | Get auth and role details for a specific user. |
| `PUT` | `/api/v1/admin/users/{user_id}/roles` | `bearerAuth` | Replace RBAC roles for a user. |
| `POST` | `/api/v1/admin/users/{user_id}/sessions/revoke` | `bearerAuth` | Revoke active sessions for a user. |
| `PATCH` | `/api/v1/admin/users/{user_id}/status` | `bearerAuth` | Update account lifecycle status (`active`, `suspended`, `banned`). |

## Auth

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/auth/federation/exchange` | `public/unspecified` | auth_federation_exchange |
| `POST` | `/api/v1/auth/forgot-password` | `public/unspecified` | auth_forgot_password |
| `POST` | `/api/v1/auth/login` | `public/unspecified` | auth_login |
| `POST` | `/api/v1/auth/logout` | `public/unspecified` | auth_logout |
| `POST` | `/api/v1/auth/refresh` | `public/unspecified` | auth_refresh |
| `POST` | `/api/v1/auth/register` | `public/unspecified` | auth_register |
| `POST` | `/api/v1/auth/reset-password` | `public/unspecified` | auth_reset_password |
| `POST` | `/api/v1/auth/verify-email` | `public/unspecified` | auth_verify_email |
| `GET` | `/api/v1/me` | `bearerAuth` | auth_me |
| `PUT` | `/api/v1/me/profile` | `bearerAuth` | auth_update_me_profile |
| `GET` | `/api/v1/me/sessions` | `bearerAuth` | auth_list_me_sessions |
| `DELETE` | `/api/v1/me/sessions/{session_id}` | `bearerAuth` | auth_revoke_me_session |

## Problems

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/problems` | `public/unspecified` | List problems with pagination and filters |
| `POST` | `/api/v1/problems` | `bearerAuth` | Create a new problem |
| `GET` | `/api/v1/problems/{id}` | `public/unspecified` | Get problem by ID |
| `POST` | `/api/v1/problems/{id}/abandon` | `bearerAuth` | Abandon a claimed problem |
| `POST` | `/api/v1/problems/{id}/claim` | `bearerAuth` | Claim a problem |
| `GET` | `/api/v1/problems/{id}/dispute` | `bearerAuth` | Get the active dispute case for a problem |
| `POST` | `/api/v1/problems/{id}/dispute` | `bearerAuth` | Raise a dispute on a problem's solution |
| `POST` | `/api/v1/problems/{id}/dispute/finalize` | `bearerAuth` | Finalize the verdict for the active dispute on a problem |
| `POST` | `/api/v1/problems/{id}/dispute/vote` | `bearerAuth` | Cast a vote in the active dispute jury for a problem |
| `GET` | `/api/v1/problems/{id}/dispute/votes` | `bearerAuth` | Get revealed votes for a problem dispute (after verdict + 24h) |
| `POST` | `/api/v1/problems/{id}/solutions` | `bearerAuth` | Submit a solution to a problem |
| `PATCH` | `/api/v1/problems/{id}/witness-complexity` | `bearerAuth` | Apply sensemaker override to a problem's witness complexity. |

## CV Hidup

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/cv-hidup/export` | `bearerAuth` | Export CV Hidup in various formats |
| `GET` | `/api/v1/cv-hidup/exports/{export_id}` | `public/unspecified` | get_cv_hidup_export |
| `GET` | `/api/v1/cv-hidup/me` | `bearerAuth` | Get authenticated user's CV Hidup |
| `GET` | `/api/v1/cv-hidup/qr` | `bearerAuth` | Get QR code for authenticated user's CV |
| `GET` | `/api/v1/cv-hidup/qr/{export_id}` | `public/unspecified` | Get QR code for exported CV |
| `GET` | `/api/v1/cv-hidup/settings` | `bearerAuth` | Get CV Hidup privacy settings |
| `PUT` | `/api/v1/cv-hidup/settings` | `bearerAuth` | Update CV Hidup privacy settings |
| `GET` | `/api/v1/cv-hidup/verify/{export_id}` | `public/unspecified` | Verify an exported CV is authentic |
| `GET` | `/api/v1/cv-hidup/{user_id}` | `public/unspecified` | Get CV Hidup for a specific user |
| `POST` | `/api/v1/cv-hidup/{user_id}/export` | `bearerAuth, platform_service_token` | Export CV Hidup for a specific user (JWT self-only or platform-scoped service token). |
| `GET` | `/api/v1/cv-hidup/{user_id}/qr` | `bearerAuth, platform_service_token` | Get QR code for a specific user's live CV (JWT self-only or platform-scoped service token). |

## Reputation

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/decay/warnings` | `bearerAuth` | Get decay warnings for the authenticated user. |
| `POST` | `/api/v1/dukung-outcomes` | `platform_service_token` | Report witness/project terminal outcomes for Dukung retroactive J-score adjustments. |
| `POST` | `/api/v1/reputation/calculate` | `bearerAuth` | Trigger reputation recalculation |
| `POST` | `/api/v1/reputation/competence/projection` | `bearerAuth, platform_service_token` | Project competence progression under a regular cadence. |
| `GET` | `/api/v1/reputation/distribution` | `bearerAuth` | Get tier distribution statistics |
| `GET` | `/api/v1/reputation/explain/{user_id}` | `bearerAuth, platform_service_token` | get_reputation_explainability |
| `GET` | `/api/v1/reputation/leaderboard` | `bearerAuth` | Get reputation leaderboard |
| `GET` | `/api/v1/reputation/me/rankings` | `bearerAuth` | Get personal rankings across leaderboards |
| `GET` | `/api/v1/reputation/novice-advancement` | `bearerAuth` | Get Novice advancement metrics between the two most recent reputation runs. |
| `GET` | `/api/v1/users/{id}/decay/warnings` | `bearerAuth, platform_service_token` | Get decay warnings for a specific user (JWT self-only or platform-scoped service token). |

## Users

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/account/link` | `bearerAuth` | Request a new account link |
| `POST` | `/api/v1/account/link/challenge` | `bearerAuth` | issue_account_link_challenge |
| `POST` | `/api/v1/account/link/challenge/verify` | `bearerAuth` | verify_account_link_challenge |
| `GET` | `/api/v1/account/links` | `bearerAuth` | List user's linked accounts |
| `DELETE` | `/api/v1/account/links/{link_id}` | `bearerAuth` | Unlink a platform account |
| `GET` | `/api/v1/users` | `public/unspecified` | List users with pagination and filters |
| `GET` | `/api/v1/users/{id}` | `public/unspecified` | Get user by ID |
| `GET` | `/api/v1/users/{id}/activity` | `public/unspecified` | Get user activity summary |
| `GET` | `/api/v1/users/{id}/reputation` | `public/unspecified` | Get user reputation details |
| `GET` | `/api/v1/users/{id}/tier` | `public/unspecified` | Get user tier information |

## Platforms

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/platforms` | `public/unspecified` | GET /api/v1/platforms |
| `GET` | `/api/v1/platforms/{platform_id}` | `public/unspecified` | GET /api/v1/platforms/:platform_id |
| `POST` | `/api/v1/platforms/{platform_id}/bot-command` | `bot_service_token` | POST /api/v1/platforms/:platform_id/bot-command |
| `GET` | `/api/v1/platforms/{platform_id}/commands` | `public/unspecified` | GET /api/v1/platforms/:platform_id/commands |
| `POST` | `/api/v1/platforms/{platform_id}/ingest` | `bearerAuth` | POST /api/v1/platforms/:platform_id/ingest |
| `GET` | `/api/v1/platforms/{platform_id}/linked-users` | `bot_service_token` | GET /api/v1/platforms/:platform_id/linked-users |
| `POST` | `/api/v1/platforms/{platform_id}/poll-ingest` | `bot_service_token` | POST /api/v1/platforms/:platform_id/poll-ingest |
| `POST` | `/api/v1/platforms/{platform_id}/webhook` | `public/unspecified` | POST /api/v1/platforms/:platform_id/webhook |

## Privacy

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/privacy/audit` | `bearerAuth` | Get privacy change history |
| `POST` | `/api/v1/privacy/delete-request` | `bearerAuth` | Request account deletion (GDPR right to erasure) |
| `POST` | `/api/v1/privacy/export` | `bearerAuth` | Request GDPR-style data export |
| `GET` | `/api/v1/privacy/export/{export_id}` | `bearerAuth` | Check data export status |
| `GET` | `/api/v1/privacy/export/{export_id}/download` | `bearerAuth` | Download completed data export |
| `POST` | `/api/v1/privacy/preset` | `bearerAuth` | Apply a privacy preset |
| `GET` | `/api/v1/privacy/settings` | `bearerAuth` | Get current privacy settings |
| `PUT` | `/api/v1/privacy/settings` | `bearerAuth` | Update privacy settings |

## Account

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/account/check-identifier/{hash}` | `public/unspecified` | Check if an identifier is banned (for registration) |
| `POST` | `/api/v1/account/death-attestation` | `bearerAuth` | Attest to a user's death (requires 3 Keystone attestations) |
| `POST` | `/api/v1/account/exit` | `bearerAuth` | Request voluntary exit (30-day cooling-off period) |
| `POST` | `/api/v1/account/exit/cancel` | `bearerAuth` | Cancel voluntary exit request |
| `GET` | `/api/v1/account/status` | `bearerAuth` | Get current account termination status |
| `POST` | `/api/v1/account/terminate-fraud` | `bearerAuth` | Terminate account for fraud (admin only) |
| `GET` | `/api/v1/account/{user_id}/memorial-influence` | `public/unspecified` | Get memorial influence (for viewing memorial accounts) |

## Cross-Pillar

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/cross-pillar/cases` | `bearerAuth` | List cross-pillar arbitration cases |
| `POST` | `/api/v1/cross-pillar/cases` | `bearerAuth` | Create a new cross-pillar arbitration case |
| `GET` | `/api/v1/cross-pillar/cases/{id}` | `bearerAuth` | Get a cross-pillar arbitration case by ID |
| `POST` | `/api/v1/cross-pillar/cases/{id}/finalize` | `bearerAuth` | Finalize verdict for a cross-pillar case |
| `POST` | `/api/v1/cross-pillar/cases/{id}/recuse` | `bearerAuth` | Recuse from a cross-pillar case |
| `POST` | `/api/v1/cross-pillar/cases/{id}/select-jury` | `bearerAuth` | Select jury for a cross-pillar case |
| `POST` | `/api/v1/cross-pillar/cases/{id}/vote` | `bearerAuth` | Cast a vote in a cross-pillar case |

## Jury

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/jury/cases` | `bearerAuth` | List jury cases for the authenticated user |
| `GET` | `/api/v1/jury/cases/{id}` | `bearerAuth` | Get jury case by ID |
| `POST` | `/api/v1/jury/cases/{id}/finalize` | `bearerAuth` | Finalize the verdict for a jury case |
| `POST` | `/api/v1/jury/cases/{id}/vote` | `bearerAuth` | Cast a vote in a jury case |
| `GET` | `/api/v1/jury/cases/{id}/votes` | `bearerAuth` | Get votes for a jury case (only after verdict is reached) |
| `GET` | `/api/v1/jury/eligibility` | `bearerAuth` | Check jury eligibility for the authenticated user |
| `GET` | `/api/v1/jury/pool` | `bearerAuth` | Get current jury pool for a domain |

## Recovery

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/recovery/process-transitions` | `bearerAuth` | Process batch transitions |
| `GET` | `/api/v1/recovery/stats` | `bearerAuth` | Get recovery statistics |
| `GET` | `/api/v1/recovery/status` | `bearerAuth` | Get current user's recovery status |
| `GET` | `/api/v1/recovery/users` | `bearerAuth` | List all users in recovery |
| `GET` | `/api/v1/recovery/users/{id}` | `bearerAuth` | Get specific user's recovery info |
| `POST` | `/api/v1/recovery/users/{id}/extend` | `bearerAuth` | Extend Shadow period for a user |
| `POST` | `/api/v1/recovery/users/{id}/release` | `bearerAuth` | Release user early from recovery |

## Skills

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/skills/estimate-complexity` | `public/unspecified` | Estimate witness complexity for sensemaking reward weighting. |
| `GET` | `/api/v1/skills/nodes/{id}` | `public/unspecified` | Get a skill taxonomy node (name + parent + children). |
| `GET` | `/api/v1/skills/nodes/{id}/labels` | `public/unspecified` | List localized labels for a taxonomy node. |
| `GET` | `/api/v1/skills/nodes/{id}/relations` | `public/unspecified` | List relations for a taxonomy node. |
| `GET` | `/api/v1/skills/search` | `public/unspecified` | Search taxonomy nodes by label. |
| `POST` | `/api/v1/skills/suggest` | `public/unspecified` | Suggest skill IDs for a given piece of text. |
| `GET` | `/api/v1/skills/{id}/parent` | `public/unspecified` | Query the parent of a skill id. |

## Vouches

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/users/{id}/vouch-budget` | `bearerAuth, platform_service_token` | Get vouch budget for a specific user (JWT self-only or platform-scoped service token). |
| `GET` | `/api/v1/users/{id}/vouches/given` | `bearerAuth` | Get vouches given by a user |
| `GET` | `/api/v1/users/{id}/vouches/received` | `bearerAuth` | Get vouches received by a user |
| `POST` | `/api/v1/vouches` | `bearerAuth` | Create a new vouch |
| `GET` | `/api/v1/vouches/budget` | `bearerAuth` | Get vouch budget for authenticated user |
| `DELETE` | `/api/v1/vouches/{id}` | `bearerAuth` | Withdraw a vouch |
| `GET` | `/api/v1/vouches/{id}` | `bearerAuth` | Get vouch by ID |

## Emergency

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/emergency/brakes` | `bearerAuth` | Trigger an emergency brake. |
| `GET` | `/api/v1/emergency/brakes/{id}` | `bearerAuth` | Get emergency brake status by ID. |
| `POST` | `/api/v1/emergency/brakes/{id}/resolve` | `bearerAuth` | Resolve an emergency brake after jury audit. |
| `POST` | `/api/v1/emergency/fast-tracks` | `bearerAuth` | Create an emergency fast-track for a transition. |
| `GET` | `/api/v1/emergency/fast-tracks/{id}` | `bearerAuth` | Get emergency fast-track status by ID. |
| `POST` | `/api/v1/emergency/fast-tracks/{id}/audit` | `bearerAuth` | Complete emergency fast-track post-hoc audit. |

## PoR

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/por/check-requirements` | `bearerAuth` | Check what requirements are missing for evidence |
| `GET` | `/api/v1/por/requirements/{task_type}` | `public/unspecified` | Get PoR requirements for a task type |
| `GET` | `/api/v1/por/status/{evidence_id}` | `bearerAuth, platform_service_token` | Get PoR validation status |
| `POST` | `/api/v1/por/submit` | `bearerAuth` | Submit PoR evidence for a solution |
| `GET` | `/api/v1/por/triad-requirements/{track}/{transition}` | `public/unspecified` | Get Context Triad requirements for a specific track transition. |
| `POST` | `/api/v1/por/validate` | `bearerAuth` | Validate PoR evidence |

## Tandang

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/tandang/batch` | `public/unspecified` | Batch query multiple users |
| `GET` | `/api/v1/tandang/domains/{domain}/leaders` | `public/unspecified` | Get top users in a domain |
| `GET` | `/api/v1/tandang/search` | `public/unspecified` | Search users by criteria |
| `GET` | `/api/v1/tandang/stats` | `public/unspecified` | Get system-wide activity statistics |
| `POST` | `/api/v1/tandang/webhook` | `public/unspecified` | Handle Twitter/X webhook |
| `GET` | `/api/v1/tandang/{username}` | `public/unspecified` | Query user by username |

## Genesis

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/genesis/peer-challenge` | `bearerAuth` | Get peer challenge status |
| `POST` | `/api/v1/genesis/peer-vouch` | `bearerAuth` | Submit a peer vouch for another user |
| `GET` | `/api/v1/genesis/sources` | `bearerAuth` | List available verification sources |
| `GET` | `/api/v1/genesis/status` | `bearerAuth` | Get genesis status for current user |
| `POST` | `/api/v1/genesis/verify/{source}` | `bearerAuth` | Start verification for a source |

## Domain Efficiency

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/domain-efficiency` | `public/unspecified` | GET /domain-efficiency |
| `GET` | `/api/v1/domain-efficiency/alerts` | `bearerAuth` | GET /domain-efficiency/alerts |
| `GET` | `/api/v1/domain-efficiency/weekly-report` | `public/unspecified` | GET /domain-efficiency/weekly-report |
| `GET` | `/api/v1/domain-efficiency/{domain_code}` | `public/unspecified` | GET /domain-efficiency/:domain_code |

## Slash

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/slash/cases/{id}` | `bearerAuth` | Get slash case by ID |
| `POST` | `/api/v1/slash/flag` | `bearerAuth` | Flag a user for potential fraud |
| `GET` | `/api/v1/slash/gdf` | `public/unspecified` | Get current GDF value |
| `POST` | `/api/v1/slash/gdf/contribute` | `bearerAuth` | Contribute to the GDF recovery pool |

## Solutions

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/problems/{id}/solutions` | `public/unspecified` | Validates PoR evidence for a solution verification |
| `GET` | `/api/v1/solutions/{id}` | `public/unspecified` | Get solution by ID |
| `POST` | `/api/v1/solutions/{id}/revise` | `bearerAuth` | Revise a solution after a "Needs Revision" decision. |
| `POST` | `/api/v1/solutions/{id}/verify` | `bearerAuth` | Verify a solution |

## Community Pulse

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/community/pulse/insights` | `public/unspecified` | Community pulse insights |
| `GET` | `/api/v1/community/pulse/overview` | `public/unspecified` | Community pulse overview for ecosystem-facing charts |
| `GET` | `/api/v1/community/pulse/trends` | `public/unspecified` | Community pulse trends |

## Governance Budget

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `POST` | `/api/v1/governance/lock` | `bearerAuth` | Lock governance budget for a proposal endorsement. |
| `POST` | `/api/v1/governance/release` | `bearerAuth` | Release governance budget locks for a proposal. |
| `GET` | `/api/v1/users/{id}/governance-budget` | `bearerAuth` | Get governance budget for a user. |

## Health

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/health` | `public/unspecified` | Basic health check endpoint |
| `GET` | `/health/live` | `public/unspecified` | Liveness check endpoint |
| `GET` | `/health/ready` | `public/unspecified` | Readiness check endpoint |

## Weather Heroes

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/api/v1/hero/leaderboard` | `public/unspecified` | Get weekly hero points leaderboard |
| `GET` | `/api/v1/hero/status` | `bearerAuth` | Get hero status for authenticated user |
| `GET` | `/api/v1/hero/{user_id}` | `public/unspecified` | Get hero status for a specific user |

## Monitoring

| Method | Path | Auth (per OpenAPI) | Summary |
|---|---|---|---|
| `GET` | `/metrics` | `public/unspecified` | Prometheus metrics endpoint. |

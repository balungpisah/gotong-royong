# User Authentication (SurrealDB Native)

## Overview

Gotong Royong user auth is implemented using **SurrealDB v3 record access** (`DEFINE ACCESS ... TYPE RECORD`) and **row-level authorization** via SurrealDB `PERMISSIONS`.

Key traits:
- Tokens are **issued by SurrealDB** and validated by the API via `AUTHENTICATE`.
- Browser sessions use `httpOnly` cookies:
  - `gr_session`: SurrealDB access token
  - `gr_refresh`: SurrealDB refresh token (optional, when returned by server)
- Logout is **token revocation** (jti-based) stored in the `token` table.

## Endpoints

All endpoints are under the API service (not SurrealDB directly):

- `POST /v1/auth/signup`
  - Body: `{ "email": "...", "pass": "...", "username": "...", "community_id": "..." }`
  - Returns: access token (+ optional refresh token) and user identity
- `POST /v1/auth/signin`
  - Body: `{ "email": "...", "pass": "..." }`
  - Returns: access token (+ optional refresh token) and user identity
- `POST /v1/auth/refresh`
  - Body: `{ "refresh": "..." }`
  - Returns: new access token (+ optional refresh token) and user identity
- `POST /v1/auth/logout`
  - Revokes the current access token (reads token from `Authorization: Bearer` or `gr_session` cookie)
- `GET /v1/auth/me`
  - Returns current user identity (derived from `$auth`)

## Authorization Model (Database-Enforced)

Protected API routes run SurrealDB queries under the **record-authenticated session**, so SurrealDB enforces:
- Row-level access (e.g., only members can read private chat)
- Role gating (e.g., moderation tables require `platform_role` in `["admin","moderator"]`)

Schema and permission rules live in:
- `database/migrations/0018_auth_schema.surql`
- `database/migrations/0019_record_permissions.surql`


# Migration State

This file tracks the migration ordering and execution state for the SurrealDB schema.

## Applied Migrations

- None yet.

## Pending Migrations (ordered)

1. `0001_initial_schema.surql`
2. `0002_chat_indexes.surql`
3. `0003_permissions_private_channels.surql` (reserved for PR-04)

## Notes

- Migrations are forward-only. Additive changes are preferred.
- Each migration must have a paired check file under `database/checks/`.
- Record applied timestamps and release SHA once CI automation is in place.
- Core domain tables (users, contributions, evidence, vouches, outbox, webhook delivery log) are deferred to PR-06 and PR-13.

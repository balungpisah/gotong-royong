# Gotong Web

Frontend application for Gotong Royong using SvelteKit 2 + Svelte 5.

## Local Commands

```sh
bun install
bun run dev
bun run check
bun run lint
bun run test:unit
bun run test:e2e
bun run build
```

## Baseline Structure

```text
src/
  routes/          # Route entries and layouts
  lib/
    api/           # API client and transport wrappers
    components/    # Shared UI components
    stores/        # Svelte stores and state modules
    types/         # Shared TypeScript contracts
    utils/         # Utility helpers
```

This structure is intentionally lightweight and will be expanded during the frontend foundation sprint.

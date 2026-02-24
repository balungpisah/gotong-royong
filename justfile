set dotenv-load := false

web := "apps/web"

# Default: list available recipes
default:
    @just --list

# ── Web (SvelteKit) ──────────────────────────────────────────────────

# Start web dev server
dev:
    cd {{web}} && npm run dev

# Type-check web with svelte-check
check:
    cd {{web}} && npx svelte-check

# Run web unit tests
web-test:
    cd {{web}} && npx vitest run

# Production build web
web-build:
    cd {{web}} && npx vite build

# Format web with prettier
web-fmt:
    cd {{web}} && npx prettier --write .

# All web checks: type-check + test + build
web-verify: check web-test web-build

# Preview production build
preview:
    cd {{web}} && npx vite preview

# Install web dependencies
web-install:
    cd {{web}} && npm install

# ── Rust (API / Worker) ──────────────────────────────────────────────

fmt:
	cargo fmt

test:
	cargo test

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

api:
	cargo run -p gotong-api

worker:
	cargo run -p gotong-worker

db-migrate:
	scripts/db/migrate.sh

db-check:
	scripts/db/check.sh

dev-db-up:
	scripts/dev/up.sh

dev-db-down:
	scripts/dev/down.sh

surreal-probe:
	SURREAL_BIN=scripts/tools/surreal-docker.sh \
	LOCKED_TARGET_VERSION=3.0.0 \
	docs/research/references/samples/surrealdb/pattern_probe.sh docs/research/surrealdb-pattern-sampling-v3.0.0.md

surreal-ontology-probe:
	SURREAL_BIN=scripts/tools/surreal-docker.sh \
	LOCKED_TARGET_VERSION=3.0.0 \
	docs/research/references/samples/surrealdb/ontology_probe.sh docs/research/ontology-probe-report.md

release-gates-surreal:
	SURREAL_BIN=scripts/tools/surreal-docker.sh \
	LOCKED_TARGET_VERSION=3.0.0 \
	scripts/surrealdb-go-no-go.sh docs/research/surrealdb-go-no-go-latest.md

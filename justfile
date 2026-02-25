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

ontology-feed-backfill-expired *args:
	cargo run -p gotong-worker -- ontology-feed-backfill-expired {{args}}

feed-participant-edge-backfill *args:
	cargo run -p gotong-worker -- feed-participant-edge-backfill {{args}}

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

chat-bench-surreal:
	scripts/surrealdb-chat-bench.sh docs/research/surrealdb-chat-bench-latest.md

feed-index-bench-surreal:
	scripts/surrealdb-feed-index-bench.sh docs/research/surrealdb-feed-index-bench-latest.md

feed-involvement-bench-surreal:
	scripts/surrealdb-feed-involvement-bench.sh docs/research/surrealdb-feed-involvement-bench-latest.md

notification-bench-surreal:
	scripts/surrealdb-notification-bench.sh docs/research/surrealdb-notification-bench-latest.md

smoke-ontology-enrichment-live:
	scripts/smoke/ontology_enrichment_live.sh

smoke-feed-involvement-edge-cutover-live:
	scripts/smoke/feed_involvement_edge_cutover_live.sh

ontology-enrichment-check:
	scripts/docs/check_ontology_enrichment_guardrails.sh

ontology-enrichment-check-full:
	scripts/docs/check_ontology_enrichment_guardrails.sh --with-tests

pack-c-alerts-stage-a namespace="monitoring":
	scripts/deploy/pack_c_prometheus_rules.sh --stage stage-a --namespace {{namespace}}

pack-c-alerts-stage-b namespace="monitoring":
	scripts/deploy/pack_c_prometheus_rules.sh --stage stage-b --namespace {{namespace}}

pack-c-alerts-stage-c namespace="monitoring":
	scripts/deploy/pack_c_prometheus_rules.sh --stage stage-c --namespace {{namespace}}

pack-c-alerts-plan stage="stage-a" namespace="monitoring":
	scripts/deploy/pack_c_prometheus_rules.sh --stage {{stage}} --namespace {{namespace}} --dry-run

pack-c-alerts-verify:
	scripts/deploy/verify_pack_c_monitoring_assets.sh

pack-c-slice-gate namespace="monitoring":
	scripts/deploy/pack_c_slice_gate.sh {{namespace}}

pack-c-cutover-readiness namespace="monitoring":
	scripts/deploy/pack_c_cutover_readiness.sh --namespace {{namespace}}

pack-c-stage-kickoff stage="stage-a" namespace="monitoring":
	scripts/deploy/pack_c_stage_kickoff.sh --stage {{stage}} --namespace {{namespace}}

pack-c-stage-a-kickoff namespace="monitoring":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-a --namespace {{namespace}}

pack-c-stage-b-kickoff namespace="monitoring":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-b --namespace {{namespace}}

pack-c-stage-c-kickoff namespace="monitoring":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-c --namespace {{namespace}}

pack-c-stage-go-no-go stage="stage-b" prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage {{stage}} --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-a-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-a --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-b-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-b --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-c-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-c --prom-url {{prom_url}} --window {{window}} --step {{step}}

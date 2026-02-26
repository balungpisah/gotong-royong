set dotenv-load := false

web := "apps/web"

# Default: list available recipes
default:
    @just --list

# ── Web (SvelteKit) ──────────────────────────────────────────────────

# Start web dev server
dev:
    cd {{web}} && bun run dev

# Start web dev server with local auth bypass enabled (development only)
dev-bypass-auth target="http://127.0.0.1:3100" user_id="dev-user":
    cd {{web}} && GR_AUTH_DEV_BYPASS_ENABLED=1 GR_AUTH_DEV_BYPASS_USER_ID={{user_id}} GR_API_PROXY_TARGET={{target}} bun run dev

# Type-check web with svelte-check
check:
    cd {{web}} && bun run check

# Run web unit tests
web-test:
    cd {{web}} && bun run test:unit

# Run Playwright live API proxy smoke against a running backend
web-test-e2e-live-api target="http://127.0.0.1:3100":
    cd {{web}} && GR_API_PROXY_TARGET={{target}} bun run test:e2e:live-api

# Run Playwright live API smoke against an already deployed frontend host
web-test-e2e-live-api-external base_url:
    cd {{web}} && PLAYWRIGHT_EXTERNAL_BASE_URL={{base_url}} bun run test:e2e:live-api:external

# Production build web
web-build:
    cd {{web}} && bun run build

# Format web with prettier
web-fmt:
    cd {{web}} && bun run format

# All web checks: type-check + test + build
web-verify: check web-test web-build

# Preview production build
preview:
    cd {{web}} && bun run preview

# Install web dependencies
web-install:
    cd {{web}} && bun install

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

dev-api-up: dev-db-up
	docker compose -f compose.dev.yaml --profile api up -d --build gotong-api

dev-api-down:
	docker compose -f compose.dev.yaml stop gotong-api

dev-api-logs:
	docker compose -f compose.dev.yaml logs -f gotong-api

dev-worker-up: dev-db-up
	docker compose -f compose.dev.yaml --profile worker up -d --build gotong-worker

dev-worker-down:
	docker compose -f compose.dev.yaml stop gotong-worker

dev-worker-logs:
	docker compose -f compose.dev.yaml logs -f gotong-worker

dev-seed:
	scripts/dev/seed-api.sh

# Backward-compatible alias
seed-api: dev-seed

dev-monitoring-up: dev-api-up
	docker compose -f compose.dev.yaml --profile monitoring up -d prometheus

dev-monitoring-down:
	docker compose -f compose.dev.yaml stop prometheus

dev-monitoring-logs:
	docker compose -f compose.dev.yaml logs -f prometheus

dev-full-up:
	docker compose -f compose.dev.yaml --profile api --profile worker --profile monitoring up -d --build gotong-api gotong-worker prometheus

dev-full-down:
	docker compose -f compose.dev.yaml down

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
	scripts/release-gates-surreal.sh docs/research/release-gates-surreal-latest.md

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

smoke-chat-attachment-s3-live:
	scripts/smoke/chat_attachment_s3_live.sh

chat-attachment-lifecycle-plan:
	scripts/deploy/chat_attachment_lifecycle_policy.sh --dry-run

chat-attachment-lifecycle-apply:
	scripts/deploy/chat_attachment_lifecycle_policy.sh

chat-attachment-lifecycle-verify:
	scripts/deploy/verify_chat_attachment_lifecycle_rules.sh

chat-attachment-alerts-apply namespace="monitoring":
	scripts/deploy/chat_attachment_prometheus_rules.sh --namespace {{namespace}}

chat-attachment-alerts-plan namespace="monitoring":
	scripts/deploy/chat_attachment_prometheus_rules.sh --namespace {{namespace}} --dry-run

chat-attachment-alerts-verify:
	scripts/deploy/verify_chat_attachment_monitoring_assets.sh

chat-attachment-branch-protection-check repo branch="main":
	scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo {{repo}} --branch {{branch}}

chat-attachment-branch-protection-plan repo branch="main":
	scripts/deploy/verify_surreal_release_gate_branch_protection.sh --repo {{repo}} --branch {{branch}} --dry-run

frontend-live-cutover-gate:
	scripts/deploy/frontend_live_cutover_gate.sh --dry-run

frontend-live-cutover-gate-live frontend_url:
	scripts/deploy/frontend_live_cutover_gate.sh --frontend-url {{frontend_url}}

frontend-live-cutover-rollout env frontend_url:
	scripts/deploy/frontend_live_cutover_rollout.sh --env {{env}} --frontend-url {{frontend_url}}

frontend-live-cutover-rollout-dry-run env="staging":
	scripts/deploy/frontend_live_cutover_rollout.sh --env {{env}} --dry-run

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

pack-c-stage-end-to-end stage="stage-b" namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s" go_no_go_dry_run="false":
	scripts/deploy/pack_c_stage_kickoff.sh --stage {{stage}} --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run {{go_no_go_dry_run}}

pack-c-stage-end-to-end-dry-run stage="stage-b" namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s":
	scripts/deploy/pack_c_stage_kickoff.sh --stage {{stage}} --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run true

pack-c-stage-a-end-to-end namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s" go_no_go_dry_run="false":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-a --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run {{go_no_go_dry_run}}

pack-c-stage-a-end-to-end-dry-run namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-a --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run true

pack-c-stage-b-end-to-end namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s" go_no_go_dry_run="false":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-b --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run {{go_no_go_dry_run}}

pack-c-stage-b-end-to-end-dry-run namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-b --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run true

pack-c-stage-c-end-to-end namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s" go_no_go_dry_run="false":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-c --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run {{go_no_go_dry_run}}

pack-c-stage-c-end-to-end-dry-run namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s":
	scripts/deploy/pack_c_stage_kickoff.sh --stage stage-c --namespace {{namespace}} --run-go-no-go true --go-no-go-prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run true

pack-c-stage-go-no-go stage="stage-b" prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage {{stage}} --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-a-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-a --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-b-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-b --prom-url {{prom_url}} --window {{window}} --step {{step}}

pack-c-stage-c-go-no-go prom_url="http://127.0.0.1:9090" window="30m" step="60s":
	scripts/deploy/pack_c_stage_go_no_go.sh --stage stage-c --prom-url {{prom_url}} --window {{window}} --step {{step}}

hot-path-rollout namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s" go_no_go_dry_run="false" apply_chat_alerts="true" apply_chat_lifecycle="false" require_cluster="true" run_readiness="true":
	scripts/deploy/hot_path_rollout.sh --namespace {{namespace}} --prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run {{go_no_go_dry_run}} --apply-chat-alerts {{apply_chat_alerts}} --apply-chat-lifecycle {{apply_chat_lifecycle}} --require-cluster {{require_cluster}} --run-readiness {{run_readiness}}

hot-path-rollout-dry-run namespace="monitoring" prom_url="http://127.0.0.1:9090" go_no_go_step="60s":
	scripts/deploy/hot_path_rollout.sh --namespace {{namespace}} --prom-url {{prom_url}} --go-no-go-step {{go_no_go_step}} --go-no-go-dry-run true --apply-chat-alerts true --apply-chat-lifecycle false --require-cluster false --run-readiness false

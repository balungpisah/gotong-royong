set dotenv-load := false

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

surreal-probe:
	SURREAL_BIN=docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 \
	LOCKED_TARGET_VERSION=3.0.0-beta.4 \
	docs/research/samples/surrealdb/pattern_probe.sh docs/research/surrealdb-pattern-sampling-v3-beta4.md

surreal-ontology-probe:
	SURREAL_BIN=docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 \
	LOCKED_TARGET_VERSION=3.0.0-beta.4 \
	docs/research/samples/surrealdb/ontology_probe.sh docs/research/ontology-probe-report.md

release-gates-surreal:
	SURREAL_BIN=docs/research/samples/surrealdb/bin/surreal-v3.0.0-beta.4 \
	LOCKED_TARGET_VERSION=3.0.0-beta.4 \
	scripts/surrealdb-go-no-go.sh docs/research/surrealdb-go-no-go-latest.md

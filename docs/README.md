# Gotong Royong Documentation

This directory contains technical specifications and design documentation for building and deploying the Gotong Royong mutual credit platform.

## Stack Lock Notice

Implementation planning and execution are locked to:
- Rust 2024 + Axum + Tokio + Tower/tower-http
- SurrealDB `v3.0.0-beta.4` (SDK 3 beta channel)
- Redis + S3-compatible object storage

Canonical decision:
- `research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

## Table of Contents

### Architecture
- [System Overview](architecture/system-overview.md) - High-level component architecture
- [Integration Architecture](architecture/integration-architecture.md) - Markov Engine integration patterns
- [Data Flow](architecture/data-flow.md) - Task and evidence flow diagrams
- [Gameplay Rules (Tandang Signals)](architecture/tandang-gameplay-rules.md) - Deterministic gameplay gates/rewards driven by Tandang

### API Specifications
- [Webhook Specification](api/webhook-spec.md) - Webhook endpoint requirements
- [Event Payloads](api/event-payloads.md) - JSON schemas for all event types
- [Authentication](api/authentication.md) - HMAC-SHA256 implementation
- [Error Handling](api/error-handling.md) - Status codes and retry logic

### Design Documentation
- [Design Index](DESIGN-INDEX.md) - Canonical entrypoint for design references and links
- [Design Context](design/context/DESIGN-CONTEXT.md) - Locked terminology and cross-feature conventions
- [Design DNA](design/specs/DESIGN-DNA-v0.1.md) - Formalized design system and component catalog
- [AI Spec](design/specs/AI-SPEC-v0.2.md) - 10 AI touch point definitions
- [UI/UX Spec](design/specs/UI-UX-SPEC-v0.5.md) - 29-section interaction and screen spec
- [Prototype Gallery](design/README.md) - HTML reference collection
- [Design Archive](design/archive/README.md) - Legacy Word drafts and historical artifacts

### Database
- [Schema Requirements](database/schema-requirements.md) - Required tables and relationships
- [Migrations](database/migrations.md) - Schema versioning strategy

### Proof of Reality (PoR) Evidence
- [Evidence Format](por-evidence/evidence-format.md) - Evidence types and metadata
- [Validation Rules](por-evidence/validation-rules.md) - Evidence validation criteria
- [Storage Requirements](por-evidence/storage-requirements.md) - Storage backend specifications

### Deployment
- [Infrastructure](deployment/infrastructure.md) - Deployment architecture options
- [Security Checklist](deployment/security-checklist.md) - Security hardening guide
- [Monitoring](deployment/monitoring.md) - Metrics and alerting
- [Tandang Integration Config](deployment/tandang-integration-config.md) - Required env vars, toggles, and safe defaults
- [Webhook Backfill + Replay](deployment/webhook-backfill-replay.md) - Historical backfill and DLQ replay tooling
- [Tandang Observability SLOs](deployment/tandang-observability-slos.md) - Integration dashboards, SLOs, and alert rules

### Development
- [Setup Guide](development/setup-guide.md) - Local development environment setup
- [Testing Integration](development/testing-integration.md) - Integration testing guide
- [Local Development](development/local-development.md) - Development workflow

## Reading Paths by Role

### Backend Developer (New to Project)
1. Start: [System Overview](architecture/system-overview.md)
2. Then: [Database Schema](database/schema-requirements.md)
3. Then: [Webhook Specification](api/webhook-spec.md)
4. Then: [Setup Guide](development/setup-guide.md)
5. Finally: [Testing Integration](development/testing-integration.md)

### Frontend Developer
1. Start: [System Overview](architecture/system-overview.md)
2. Then: [API Event Payloads](api/event-payloads.md)
3. Then: [Evidence Format](por-evidence/evidence-format.md)
4. Then: [Error Handling](api/error-handling.md)

### DevOps Engineer
1. Start: [Infrastructure](deployment/infrastructure.md)
2. Then: [Security Checklist](deployment/security-checklist.md)
3. Then: [Monitoring](deployment/monitoring.md)
4. Then: [Database Schema](database/schema-requirements.md) (for capacity planning)

### Security Auditor
1. Start: [Authentication](api/authentication.md)
2. Then: [Security Checklist](deployment/security-checklist.md)
3. Then: [Validation Rules](por-evidence/validation-rules.md)
4. Then: [Webhook Specification](api/webhook-spec.md)

### Product Designer
1. Start: [Design Index](DESIGN-INDEX.md)
2. Then: [DESIGN-CONTEXT.md](design/context/DESIGN-CONTEXT.md)
3. Then: [DESIGN-DNA-v0.1.md](design/specs/DESIGN-DNA-v0.1.md)
4. Then: [UI-UX-SPEC-v0.5.md](design/specs/UI-UX-SPEC-v0.5.md)
5. Then: [C3-navigation-feed.html](design/prototypes/C3-navigation-feed.html)

### Product Manager
1. Start: [System Overview](architecture/system-overview.md)
2. Then: [Data Flow](architecture/data-flow.md)
3. Then: [Evidence Format](por-evidence/evidence-format.md)

## External References

- **Markov Engine Integration Guide**: [../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md](../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md)
- **Tandang Signal Mapping**: [architecture/tandang-signal-mapping.md](architecture/tandang-signal-mapping.md)
- **Full Gotong ↔ Tandang Integration Spec**: [architecture/tandang-full-integration.md](architecture/tandang-full-integration.md)
- **Gameplay Rules Mapping**: [architecture/tandang-gameplay-rules.md](architecture/tandang-gameplay-rules.md)
- **Markov Engine Adapter**: [../tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs](../../tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs)
- **Test Payloads**: [../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json](../../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json)

## Document Conventions

- **Code blocks** use language tags for syntax highlighting
- **[TBD]** marks non-blocking future enhancements only (not core stack decisions)
- **Mermaid diagrams** are used for visual flow representation
- **Tables** structure comparative data
- **Cross-references** use relative paths within this docs directory

## Contributing to Documentation

When updating these docs:
1. Maintain consistency with existing formatting
2. Update cross-references if files are renamed
3. Keep diagrams in sync with implementation
4. Do not mark core stack/runtime choices as [TBD]; follow ADR lock
5. Include code examples where appropriate

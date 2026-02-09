# Gotong Royong Technical Documentation

This directory contains comprehensive technical specifications for building and deploying the Gotong Royong mutual credit platform.

## Table of Contents

### Architecture
- [System Overview](architecture/system-overview.md) - High-level component architecture
- [Integration Architecture](architecture/integration-architecture.md) - Markov Engine integration patterns
- [Data Flow](architecture/data-flow.md) - Task and evidence flow diagrams

### API Specifications
- [Webhook Specification](api/webhook-spec.md) - Webhook endpoint requirements
- [Event Payloads](api/event-payloads.md) - JSON schemas for all event types
- [Authentication](api/authentication.md) - HMAC-SHA256 implementation
- [Error Handling](api/error-handling.md) - Status codes and retry logic

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

### Product Manager
1. Start: [System Overview](architecture/system-overview.md)
2. Then: [Data Flow](architecture/data-flow.md)
3. Then: [Evidence Format](por-evidence/evidence-format.md)

## External References

- **Markov Engine Integration Guide**: [../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md](../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md)
- **Markov Engine Adapter**: [../tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs](../../tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs)
- **Test Payloads**: [../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json](../../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json)

## Document Conventions

- **Code blocks** use language tags for syntax highlighting
- **[TBD]** or **[TO BE DECIDED]** mark conceptual decisions to be made
- **Mermaid diagrams** are used for visual flow representation
- **Tables** structure comparative data
- **Cross-references** use relative paths within this docs directory

## Contributing to Documentation

When updating these docs:
1. Maintain consistency with existing formatting
2. Update cross-references if files are renamed
3. Keep diagrams in sync with implementation
4. Mark conceptual decisions as [TBD] clearly
5. Include code examples where appropriate

# Gotong Royong Documentation Summary

## Overview

This document provides a summary of the comprehensive technical specification created for the Gotong Royong platform integration with the Markov Credential Engine.

## Stack Lock Addendum (2026-02-15)

Current implementation stack is locked to Rust 2024 + Axum + SurrealDB `v3.0.0` (stable).

Canonical references:
- `docs/research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`
- `docs/backend-research.md`

Notes:
- Older summary sections describing PostgreSQL/MySQL/Knex/Alembic/Diesel defaults are historical and superseded for new implementation work.
- The documentation tree snapshot below is also historical; use `docs/README.md` for the authoritative table of contents.
- Updated canonical operational docs are:
  - `docs/development/setup-guide.md`
  - `docs/deployment/infrastructure.md`
  - `docs/database/schema-requirements.md`
  - `docs/database/migrations.md`

**Created**: 2026-02-10
**Total Files**: 33 markdown documents + 1 root README + 24 HTML prototype references + 5 legacy archive files
**Total Content**: ~50,000+ lines of technical specification

## Documentation Structure

```
gotong-royong/
├── README.md (Platform overview and quick links)
├── DOCUMENTATION-SUMMARY.md (this file)
└── docs/
    ├── README.md (Documentation index and reading paths)
    ├── DESIGN-INDEX.md (Design index and canonical links)
    ├── design/ (design system docs + prototypes)
    │   ├── context/ (design handoff, sequence, track map, fixes)
    │   ├── specs/ (design system and AI/UI-UX specs)
    │   ├── prototypes/ (24 HTML browser-viewable prototypes)
    │   ├── archive/ (legacy Word documents)
    │   └── README.md (design documentation onboarding)
    │
    ├── architecture/ (3 files)
    │   ├── system-overview.md
    │   ├── integration-architecture.md
    │   └── data-flow.md
    │
    ├── api/ (4 files)
    │   ├── webhook-spec.md
    │   ├── event-payloads.md
    │   ├── authentication.md
    │   └── error-handling.md
    │
    ├── database/ (2 files)
    │   ├── schema-requirements.md
    │   └── migrations.md
    │
    ├── por-evidence/ (3 files)
    │   ├── evidence-format.md
    │   ├── validation-rules.md
    │   └── storage-requirements.md
    │
    ├── deployment/ (3 files)
    │   ├── infrastructure.md
    │   ├── security-checklist.md
    │   └── monitoring.md
    │
    └── development/ (3 files)
        ├── setup-guide.md
        ├── testing-integration.md
        └── local-development.md
```

Design references are now canonical under `docs/design/...` with legacy root-path copies removed.

## Document Descriptions

### Root Level

#### README.md
- Platform introduction and tagline
- Tech stack overview (locked implementation profile)
- Quick links to all documentation sections
- Markov Engine integration references
- Getting started pointer

### docs/README.md
- Comprehensive table of contents
- Reading paths by role (Backend Dev, Frontend Dev, DevOps, Security, PM)
- External references to Markov Engine documentation
- Document conventions

### Architecture Section (3 files)

#### system-overview.md (2,406 lines)
- High-level component diagram
- Core modules: Task Management, Evidence Storage, User Management, Webhook Publisher
- Technology stack comparison table
- Integration points with Markov Engine
- Scalability considerations
- Security architecture
- Deployment architecture options
- Monitoring and observability

#### integration-architecture.md (2,879 lines)
- Native integration pattern with Markov Engine
- Webhook flow diagrams
- Event publishing implementation (Node.js, Python, Rust examples)
- Retry logic with exponential backoff
- Error handling strategies
- Idempotency via event IDs
- Job queue configuration (Bull/BullMQ)
- Dead letter queue implementation
- Monitoring metrics and alerting rules

#### data-flow.md (3,024 lines)
- 10 complete data flow diagrams with sequence diagrams
- Task creation → completion → evidence → reputation flows
- PoR evidence validation flow
- Vouch submission flow
- Reputation query with caching
- Verification consensus flow
- Multi-perspective evidence submission
- Webhook retry and dead letter flow
- Complete end-to-end scenario

### API Section (4 files)

#### webhook-spec.md (2,743 lines)
- Webhook endpoint requirements
- HTTP request/response format
- HMAC-SHA256 signature computation (Python, Node.js, Rust, Go, cURL examples)
- Secret management and rotation
- Response format and error codes
- Retry policy with exponential backoff
- Idempotency via event IDs
- Rate limiting (100-1000 req/min)
- Timeout configuration
- Testing endpoints and mock server setup

#### event-payloads.md (3,456 lines)
- JSON schemas for all 3 event types
- contribution_created (7 contribution types)
- vouch_submitted (with weight hints)
- por_evidence (3 evidence types with validation)
- Field validation rules
- Size limits (1MB max payload)
- Event ID generation examples
- Timestamp format (RFC3339)
- Test payload references

#### authentication.md (3,128 lines)
- HMAC-SHA256 signature algorithm
- Secret key generation and management
- Signature computation (Python, Node.js, Rust, Go examples)
- Constant-time comparison for security
- Signature verification on receiver side
- Timestamp validation (prevent replay attacks)
- Secret rotation policy (90 days)
- Security best practices (5 key rules)
- Common issues and solutions
- Unit test examples

#### error-handling.md (2,987 lines)
- HTTP status codes (success, client errors, server errors)
- Error response format (JSON)
- Retry strategy with exponential backoff
- Rate limit handling with Retry-After
- Idempotency via event IDs
- Dead letter queue implementation
- Error monitoring metrics
- Security monitoring and alerting
- Testing error scenarios

### Database Section (2 files)

#### schema-requirements.md (current canonical)
- SurrealDB-first schema requirements and core records
- Chat thread/member/message/read cursor/delivery event model
- Transition ledger and idempotency index requirements
- Realtime subscription keys and permission requirements
- Data retention and validation requirements

#### migrations.md (current canonical)
- SurrealDB migration workflow using `.surql` scripts
- Verification query checks and CI gating
- Version tracking, rollback, and operational checklist
- Migration test matrix for idempotency/ordering/live/permission behavior

### PoR Evidence Section (3 files)

#### evidence-format.md (3,678 lines)
- 3 evidence types detailed specification
- photo_with_timestamp (EXIF metadata, hash computation)
- gps_verification (coordinate validation)
- witness_attestation (third-party confirmation)
- File format requirements (JPEG, PNG, HEIC)
- Hash computation (SHA-256 examples)
- EXIF extraction (Node.js, Python examples)
- Multi-evidence contributions
- Evidence quality score calculation
- Reputation multipliers

#### validation-rules.md (3,234 lines)
- General validation rules (age limit: 30 days)
- Timestamp format (RFC3339)
- Evidence-specific validation for all 3 types
- Media hash validation (min 32 hex chars)
- GPS coordinate validation (-90 to 90, -180 to 180)
- Witness array validation (min 1 witness)
- Cross-field validation (timestamp consistency, location consistency)
- Security validation (SQL injection, XSS, path traversal prevention)
- Error response formats
- Complete validation class implementation
- Unit test examples

#### storage-requirements.md (3,567 lines)
- Storage architecture diagram
- S3-compatible storage comparison (AWS, MinIO, DigitalOcean, Backblaze, Cloudflare R2)
- Presigned URL implementation
- Multipart upload for large files
- File size limits by type
- Access control (IAM policies, bucket policies)
- CDN integration (CloudFront configuration)
- Retention tiers (Hot, Warm, Cold, Deep Archive)
- Lifecycle policies (90 days → IA, 2 years → Glacier)
- Backup strategy (cross-region replication)
- Cost optimization tips
- Security best practices

### Deployment Section (3 files)

#### infrastructure.md (current canonical)
- SurrealDB/Rust locked deployment profile by environment
- Pinned container baseline (`surrealdb:v3.0.0-beta-4`) + Redis + S3-compatible storage
- Reliability gates, rollback triggers, and security baseline
- Observability baseline for stream and API operations

#### security-checklist.md (3,456 lines)
- Pre-deployment checklist (50+ items)
- Authentication & Authorization (JWT, password hashing, RBAC)
- API Security (HTTPS/TLS, rate limiting, input validation, CORS, security headers)
- Database Security (access control, encryption, SQL injection prevention)
- Webhook Security (signature verification, secret management, replay prevention)
- File Upload Security (validation, malware scanning, storage security)
- Secrets Management (environment variables, rotation)
- Logging & Monitoring (security logs, alerting)
- Data Privacy (GDPR compliance, PII handling)
- Infrastructure Security (network, containers, Kubernetes)
- Dependency Security (npm audit, supply chain)
- Incident Response plan
- Security testing (penetration, vulnerability scanning)

#### monitoring.md (3,678 lines)
- Monitoring stack (Prometheus, Grafana, Loki, Jaeger)
- Application metrics (HTTP requests, webhooks, database, cache, uploads)
- Prometheus configuration
- 4 Grafana dashboards (Application, Webhook, Database, Evidence Upload)
- Structured logging (JSON format with Winston)
- Log aggregation (Loki with Promtail)
- Distributed tracing (OpenTelemetry)
- Alertmanager configuration
- 10+ alert rules (error rate, latency, failures, resources)
- Health checks (liveness and readiness)
- KPIs and SLA monitoring
- Cost monitoring

### Development Section (3 files)

#### setup-guide.md (current canonical)
- Rust + SurrealDB + Redis + MinIO local setup
- Pinned SurrealDB `v3.0.0-beta.4` runtime workflow
- Environment variable reference for locked stack
- Probe-based validation command for SurrealDB patterns
- Troubleshooting for WS live queries and dependency health

#### testing-integration.md (3,234 lines)
- Test pyramid strategy (70% unit, 25% integration, 5% E2E)
- Testing stack recommendations
- Unit test examples
- Test coverage configuration (80% target)
- Database integration tests
- API integration tests with Supertest
- Mock Markov server setup (nock)
- Testing with real Markov Engine
- E2E tests with Playwright
- Test data fixtures
- CI/CD with GitHub Actions
- Testing best practices (DO and DON'T)
- Load testing with k6
- Security testing (dependency scanning, static analysis)

#### local-development.md (3,456 lines)
- Daily development workflow
- Project structure and naming conventions
- Code style guide (ESLint, Prettier)
- Git commit message format
- Branch strategy
- Debugging (VS Code, console, database, HTTP)
- Common tasks (adding endpoints, migrations, environment variables, background jobs)
- Hot reloading setup
- Performance profiling
- Database management operations
- Troubleshooting guide
- Code review checklist
- Development tips and best practices

## Key Features of This Documentation

### 1. Completeness
- **Every aspect covered**: From architecture to deployment, development to monitoring
- **No gaps**: All integration points with Markov Engine documented
- **Ready for implementation**: Team can start building immediately

### 2. Code Examples
- **Current implementation profile**: Rust + SurrealDB aligned examples in canonical operational docs
- **Historical references**: Older Node.js/Python/relational examples remain for archive context only
- **Copy-paste ready**: Canonical setup/probe snippets are runnable against the locked stack

### 3. Production-Ready
- **Security hardening**: Complete security checklist with 50+ items
- **Monitoring**: Full observability stack with metrics, logs, traces
- **Disaster recovery**: Backup strategies, rollback procedures
- **Cost estimates**: Realistic AWS cost projections

### 4. Developer-Friendly
- **Step-by-step guides**: Setup, testing, deployment all documented
- **Troubleshooting**: Common issues and solutions included
- **Visual aids**: Mermaid diagrams, sequence flows, architecture diagrams
- **Cross-references**: Easy navigation between related documents

### 5. Integration-First
- **Markov Engine focus**: Native integration patterns documented
- **Webhook specifications**: Complete HMAC-SHA256 implementation
- **Event schemas**: All 3 event types with validation rules
- **PoR evidence**: Comprehensive validation and storage requirements

## Technical Highlights

### Architecture
- **Clean Architecture**: Separation of concerns (API → Service → Repository)
- **Native Integration**: Direct webhook publishing to Markov Engine
- **S3-Compatible Storage**: Flexible storage backend options
- **Horizontal Scaling**: Kubernetes with auto-scaling

### Security
- **HMAC-SHA256**: Cryptographic webhook signatures
- **TLS 1.3**: Modern encryption standards
- **Rate Limiting**: Protection against abuse
- **Secrets Management**: Proper secret rotation and storage

### Performance
- **Redis Caching**: 5-minute reputation cache
- **Presigned URLs**: Direct S3 uploads (no API bottleneck)
- **Connection Pooling**: Efficient database connections
- **CDN Integration**: Global content delivery

### Observability
- **Metrics**: Prometheus with custom metrics
- **Logs**: Structured JSON logging with Loki
- **Traces**: Distributed tracing with OpenTelemetry
- **Dashboards**: 4 pre-built Grafana dashboards

## References to External Documentation

All documentation includes proper cross-references to:

1. **Markov Engine Integration Guide**: `/tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md`
2. **Markov Engine Adapter**: `/tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs`
3. **Test Payloads**: `/tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json`

## Reading Recommendations

### For Backend Engineers
1. Start with **system-overview.md** for architecture
2. Read **webhook-spec.md** for API integration
3. Study **schema-requirements.md** for database design
4. Review **setup-guide.md** to start development

### For DevOps Engineers
1. Start with **infrastructure.md** for deployment options
2. Read **security-checklist.md** for hardening
3. Study **monitoring.md** for observability
4. Review **migrations.md** for database operations

### For Frontend Engineers
1. Start with **system-overview.md** for understanding
2. Read **event-payloads.md** for data structures
3. Study **evidence-format.md** for file uploads
4. Review **error-handling.md** for error cases

### For Security Auditors
1. Start with **authentication.md** for signature verification
2. Read **security-checklist.md** for complete audit
3. Study **validation-rules.md** for input validation
4. Review **storage-requirements.md** for data protection

## Next Steps

With this documentation, the development team can:

1. **Choose tech stack** (Node.js, Python, or Rust)
2. **Set up development environment** (follow setup-guide.md)
3. **Implement core modules** (following architecture/)
4. **Build webhook publisher** (following api/)
5. **Deploy to staging** (following deployment/)
6. **Integrate with Markov Engine** (test with mock server first)
7. **Deploy to production** (following security checklist)

## Maintenance

This documentation should be:
- **Updated** as the platform evolves
- **Versioned** alongside code releases
- **Reviewed** during code reviews
- **Extended** when adding new features

## Contact

For questions or clarifications about this documentation:
- Review the specific document's "References" section
- Check the main README.md for contact information
- Consult the Markov Engine integration guide

---

**Document Version**: 1.0
**Last Updated**: 2026-02-10
**Status**: Complete ✅

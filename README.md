# Gotong Royong

**Mutual Credit Platform with Proof of Reality (PoR) Evidence**

Gotong Royong is a native mutual credit platform for physical task verification and reputation tracking, integrated with the Markov Credential Engine. It enables transparent, community-driven contribution tracking through cryptographically secured evidence submission and decentralized verification.

## What This Is

A backend platform that:
- Tracks physical and digital task completion
- Validates contributions through Proof of Reality (PoR) evidence
- Manages contributor reputation via Markov Engine integration
- Publishes webhook events for real-time reputation updates
- Provides mutual credit accounting for community work

## Tech Stack

- **Language/Edition**: Rust 2024
- **Backend Framework**: Axum + Tokio + Tower/tower-http
- **Database**: SurrealDB v3 beta (current lock: `v3.0.0-beta-4`)
- **Realtime**: WebSocket (primary), SSE and polling fallbacks
- **Cache**: Redis (idempotency, rate controls, ephemeral fanout metadata)
- **Storage**: S3-compatible object storage (for evidence files)
- **Authentication**: JWT + HMAC-SHA256 webhook signatures
- **Integration**: REST API + webhooks to Markov Engine

## Getting Started

See [docs/development/setup-guide.md](docs/development/setup-guide.md) for installation and local development setup.

## Documentation

Complete technical specifications and design docs are in the [docs/](docs/) directory:

- [Architecture Overview](docs/architecture/system-overview.md)
- [API Specifications](docs/api/webhook-spec.md)
- [Database Schema](docs/database/schema-requirements.md)
- [PoR Evidence Format](docs/por-evidence/evidence-format.md)
- [Deployment Guide](docs/deployment/infrastructure.md)
- [Development Setup](docs/development/setup-guide.md)
- [Design Index](docs/DESIGN-INDEX.md)

## Markov Engine Integration

This platform integrates with the Markov Credential Engine for reputation calculation:

- **Integration Guide**: [tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md](../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md)
- **Adapter Implementation**: [tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs](../tandang/markov-engine/crates/infrastructure/src/adapters/gotong_royong.rs)
- **Integration Mode**: Native (direct database access)
- **Webhook Events**: contribution_created, vouch_submitted, por_evidence

## Quick Links

- [API Documentation](docs/api/webhook-spec.md)
- [Security Policy](docs/deployment/security-checklist.md)
- [Authentication Guide](docs/api/authentication.md)
- [Full Documentation Index](docs/README.md)

## License

See LICENSE file (to be added).

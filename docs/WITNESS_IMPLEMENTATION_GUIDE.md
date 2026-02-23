# Witness Graph Implementation Guide

**Status**: Frontend-first architecture with backend alignment
**Date**: 2026-02-23
**Scope**: SurrealDB schema + Rust service layer → Frontend types

---

## Architecture Overview

```
Frontend (Truth)                Backend (Implementation)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Witness {                      witness table {
  witness_id                     witness_id
  title, summary                 title, summary
  status: "open|draft|..."       status: "draft|open|active|resolved|closed"
  members: []                    witness_member table
  messages: []                   witness_message table (paginated)
  plan: PathPlan                 witness_plan table
}                              }

ChatMessage                    witness_message {
  type: "user|ai_card|..."      type: "user|ai_card|diff_card|vote_card|system|evidence|galang"
  content, attachments          content, attachments, author, blocks, data, etc.
}

UserProfile                    warga + ontology edges
  user_id                        warga node (from existing)
  tandang_signals               signal edges (bagus, vouch, skeptis, proof_of_resolve)
  tier, stats, skills            VOUCHES edges + measurement nodes
```

---

## 1. Database Setup

### Apply Migration

```bash
# Add to MIGRATION-STATE.md pending list:
# 14. `0014_witness_graph_schema.surql`

# Run migration
cd /path/to/gotong-royong
surreal import --conn ws://localhost:8000 --user root --pass mypass \
  --namespace gotong --database chat \
  database/migrations/0014_witness_graph_schema.surql
```

### Verify Tables Created

```bash
surreal query --conn ws://localhost:8000 --user root --pass mypass \
  --namespace gotong --database chat \
  <<EOF
SELECT * FROM information_schema::tables WHERE table LIKE 'witness%';
EOF
```

---

## 2. Rust Backend: Repository Layer

### Files Created

- `crates/infra/src/repositories/witness.rs` — Main implementation
  - `WitnessRepository` trait (async-trait interface)
  - `SurrealWitnessRepository` (production)
  - Types: `Witness`, `WitnessMember`, `ChatMessage`, `WitnessDetail`, `Paginated<T>`

### Register in Module

**File: `crates/infra/src/repositories/mod.rs`**

```rust
pub mod witness;

pub use witness::{
    WitnessRepository, SurrealWitnessRepository,
    Witness, WitnessDetail, ChatMessage, WitnessMember,
    CreateWitnessInput, ListOptions, Paginated,
};
```

### Add to AppState

**File: `crates/api/src/state.rs`**

```rust
use gotong_infra::repositories::{WitnessRepository, SurrealWitnessRepository};

pub struct AppState {
    // existing fields...
    pub witness_repo: Arc<dyn WitnessRepository>,
}

// In AppState::new():
let witness_repo = match config.data_backend.as_str() {
    "memory" => Arc::new(InMemoryWitnessRepository::new()),
    "surreal" => Arc::new(SurrealWitnessRepository::new(&db_config).await?),
    _ => bail!("unsupported data_backend"),
};
```

---

## 3. API Endpoints

### Routes to Add

**File: `crates/api/src/routes/mod.rs`**

```rust
pub mod witness;

// In router():
let protected = Router::new()
    // existing routes...
    .route("/v1/witnesses", post(witness::create_witness).get(witness::list_witnesses))
    .route("/v1/witnesses/:witness_id", get(witness::get_witness_detail))
    .route("/v1/witnesses/:witness_id/messages",
        post(witness::add_message).get(witness::get_messages))
    .route("/v1/witnesses/:witness_id/members",
        post(witness::add_member).get(witness::get_members))
    .route("/v1/witnesses/:witness_id/status", put(witness::update_status))
    // ... WebSocket route for real-time messages
    .route_layer(middleware::from_fn(app_middleware::require_auth_middleware));
```

### Handler Template

**File: `crates/api/src/routes/witness.rs`** (NEW)

```rust
use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::state::AppState;

pub async fn create_witness(
    State(state): State<AppState>,
    Json(payload): Json<CreateWitnessInput>,
) -> Result<(StatusCode, Json<WitnessDetail>), ApiError> {
    let detail = state.witness_repo.create(payload).await
        .map_err(|e| ApiError::Internal)?;
    Ok((StatusCode::CREATED, Json(detail)))
}

pub async fn get_witness_detail(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
) -> Result<Json<WitnessDetail>, ApiError> {
    let detail = state.witness_repo.get_detail(&witness_id).await
        .map_err(|_| ApiError::NotFound)?;
    Ok(Json(detail))
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path(witness_id): Path<String>,
    Query(opts): Query<MessageQueryOpts>,
) -> Result<Json<Paginated<ChatMessage>>, ApiError> {
    let msgs = state.witness_repo.get_messages(&witness_id, opts.limit.unwrap_or(20), opts.cursor).await
        .map_err(|e| ApiError::Internal)?;
    Ok(Json(msgs))
}

// ... add_message, add_member, update_status, etc.
```

---

## 4. Frontend Service Layer

### Replace Mocks with Real API Client

**File: `apps/web/src/lib/services/index.ts`**

```typescript
import { ApiWitnessService } from './api/witness-service';
import { ApiUserService } from './api/user-service';
import { type WitnessService, type UserService } from './types';

const USE_MOCK = false; // Toggle to false to use real API

export function getServices(): ServiceProvider {
  if (USE_MOCK) {
    return {
      witness: new MockWitnessService(),
      user: new MockUserService(),
      // ...
    };
  } else {
    return {
      witness: new ApiWitnessService(),
      user: new ApiUserService(),
      // ...
    };
  }
}
```

### Create API Client Service

**File: `apps/web/src/lib/services/api/witness-service.ts`** (NEW)

```typescript
import type {
  WitnessService,
  Paginated,
} from '../types';
import type {
  Witness,
  WitnessDetail,
  WitnessCreateInput,
  ChatMessage,
  PathPlan,
  DiffResponse,
} from '$lib/types';

export class ApiWitnessService implements WitnessService {
  private baseUrl = '/api';

  async create(input: WitnessCreateInput): Promise<WitnessDetail> {
    const response = await fetch(`${this.baseUrl}/v1/witnesses`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    });
    if (!response.ok) throw new Error('Failed to create witness');
    return response.json();
  }

  async list(opts?: {
    status?: string;
    cursor?: string;
    limit?: number;
  }): Promise<Paginated<Witness>> {
    const params = new URLSearchParams();
    if (opts?.status) params.append('status', opts.status);
    if (opts?.cursor) params.append('cursor', opts.cursor);
    if (opts?.limit) params.append('limit', String(opts.limit));

    const response = await fetch(`${this.baseUrl}/v1/witnesses?${params}`, {
      headers: { Authorization: `Bearer ${this.getToken()}` },
    });
    if (!response.ok) throw new Error('Failed to list witnesses');
    return response.json();
  }

  async get(witnessId: string): Promise<WitnessDetail> {
    const response = await fetch(`${this.baseUrl}/v1/witnesses/${witnessId}`, {
      headers: { Authorization: `Bearer ${this.getToken()}` },
    });
    if (!response.ok) throw new Error('Failed to get witness');
    return response.json();
  }

  async getMessages(
    witnessId: string,
    opts?: { cursor?: string; limit?: number }
  ): Promise<Paginated<ChatMessage>> {
    const params = new URLSearchParams();
    if (opts?.cursor) params.append('cursor', opts.cursor);
    if (opts?.limit) params.append('limit', String(opts.limit));

    const response = await fetch(
      `${this.baseUrl}/v1/witnesses/${witnessId}/messages?${params}`,
      {
        headers: { Authorization: `Bearer ${this.getToken()}` },
      }
    );
    if (!response.ok) throw new Error('Failed to get messages');
    return response.json();
  }

  async sendMessage(
    witnessId: string,
    content: string,
    attachments?: File[]
  ): Promise<ChatMessage> {
    const formData = new FormData();
    formData.append('content', content);
    if (attachments) {
      attachments.forEach((file) => formData.append('attachments', file));
    }

    const response = await fetch(
      `${this.baseUrl}/v1/witnesses/${witnessId}/messages`,
      {
        method: 'POST',
        headers: { Authorization: `Bearer ${this.getToken()}` },
        body: formData,
      }
    );
    if (!response.ok) throw new Error('Failed to send message');
    return response.json();
  }

  // ... other methods

  private getToken(): string {
    // Retrieve from session/auth store
    return localStorage.getItem('auth_token') || '';
  }
}
```

---

## 5. Real-Time Integration (WebSocket)

### Client-side Subscription

```typescript
// In a Svelte component
const unsubscribe = witnessStore.subscribe(async ($witness) => {
  if ($witness.witness_id) {
    const ws = new WebSocket(
      `wss://api.example.com/ws/witnesses/${$witness.witness_id}/messages`
    );

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      // Update store with new message
      witnessStore.update(w => ({
        ...w,
        messages: [message, ...w.messages],
        message_count: w.message_count + 1,
      }));
    };
  }
});
```

---

## 6. Implementation Checklist

### Phase 1: Database & Backend
- [ ] Apply migration `0014_witness_graph_schema.surql`
- [ ] Update `crates/infra/src/repositories/mod.rs` to export witness types
- [ ] Register `WitnessRepository` in `AppState`
- [ ] Create `crates/api/src/routes/witness.rs` with all handlers
- [ ] Add routes to `crates/api/src/routes/mod.rs`
- [ ] Test endpoints with Postman/curl

### Phase 2: Frontend Service Layer
- [ ] Create `apps/web/src/lib/services/api/witness-service.ts`
- [ ] Create `apps/web/src/lib/services/api/user-service.ts`
- [ ] Update `apps/web/src/lib/services/index.ts` to toggle USE_MOCK → false
- [ ] Test mock → real API transition

### Phase 3: Real-Time & Optimization
- [ ] Implement WebSocket subscription for messages
- [ ] Add cursor-based pagination in UI
- [ ] Cache witness detail in stores
- [ ] Test performance with 100+ messages

### Phase 4: Graph Queries (As-Needed)
- [ ] Implement signal queries (user → signals → witnesses)
- [ ] Implement vouch traversal (user → vouches → users → skills)
- [ ] Add signal counting queries

---

## 7. Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Separate message table** | Enables pagination, real-time streaming without reloading entire witness |
| **Denormalized counts** | Fast display (member_count, message_count); updated on insert via trigger |
| **Cursor-based pagination** | Better for real-time data (no offset jumping) |
| **Hybrid aggregate + graph** | Witnesses are aggregates; signals/vouches are graph edges |
| **Request/correlation IDs** | Idempotency, tracing, audit trail |

---

## 8. Future Optimizations

```rust
// Add SurrealDB triggers to keep denormalized counts accurate
DEFINE TRIGGER increment_message_count ON witness_message AFTER CREATE THEN
  UPDATE witness SET message_count += 1 WHERE witness_id = $after.witness_id;

// Add WebSocket subscription handler for real-time messages
pub async fn subscribe_witness_messages(
    ws: WebSocketUpgrade,
    Path(witness_id): Path<String>,
) -> impl IntoResponse {
    // Use SurrealDB LIVE QUERY or custom event stream
}

// Add caching layer (Redis)
let cached_witness = cache.get::<WitnessDetail>(&witness_id).await;
```

---

## Files & Paths Summary

| File | Purpose |
|------|---------|
| `database/migrations/0014_witness_graph_schema.surql` | SurrealDB schema |
| `crates/infra/src/repositories/witness.rs` | Repository trait + impl |
| `crates/api/src/routes/witness.rs` | HTTP handlers (NEW) |
| `apps/web/src/lib/services/api/witness-service.ts` | Frontend API client (NEW) |
| `apps/web/src/lib/services/index.ts` | Service factory (UPDATE) |

---

## Glossary

- **Witness**: Frontend domain concept; backend stores as `witness` table
- **ChatMessage**: Paginated messages in `witness_message` table (7 types)
- **WitnessMember**: Role-based membership (`witness_member` table)
- **PathPlan**: Adaptive path stored separately (`witness_plan` table)
- **Signal**: User → SignalType → Witness (graph edges)
- **Vouch**: User → User peer endorsement (graph edges)

---

## Next Steps

1. **Apply migration** to SurrealDB
2. **Register repository** in backend state
3. **Implement routes** for all CRUD operations
4. **Create frontend API client** to replace mocks
5. **Toggle `USE_MOCK = false`** to activate real API
6. **Test end-to-end** with sample data
7. **Optimize & monitor** with OpenTelemetry

Good luck! 🚀

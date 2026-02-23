> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 2. Three-Layer Architecture

All AI decision points follow a consistent three-layer pattern:

```
┌─────────────────────────────────────────┐
│   Backend Layer (Gotong Royong)         │
│   ├─ Data collection & validation       │
│   ├─ Pre-processing (redaction, QA)     │
│   └─ Triggers for AI decision points    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Tandang Integration Layer             │
│   ├─ Feature vector creation            │
│   ├─ Caching & retrieval                │
│   └─ Fallback data sources              │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   AI Decision Layer (LLM / ML)          │
│   ├─ Model inference                    │
│   ├─ Confidence scoring                 │
│   └─ Reasoning extraction               │
└─────────────────────────────────────────┘
```

Each AI touch point:
1. Receives structured input from Backend or Tandang
2. Queries Tandang for contextual data (if applicable)
3. Runs model inference with system prompt + few-shot examples
4. Returns structured output + confidence + reasoning
5. Falls back gracefully if model unavailable

---

### 2.1 Edge-Pod: AI Service Deployment Model

The AI Decision Layer runs as a **separate service** ("edge-pod") from the
Gotong Royong platform backend. This separation allows independent scaling,
model upgrades, and prompt iteration without redeploying the platform.

```
┌──────────────┐      ┌──────────────────┐      ┌──────────────────────┐
│   Frontend   │      │  Platform Backend │      │   Edge-Pod           │
│  (SvelteKit) │─────▶│  (Axum REST API)  │─────▶│  (AI Service)        │
│              │      │                  │      │                      │
│  • User input│      │  • Auth / session │      │  • Prompt assembly   │
│  • Renders   │      │  • DB context     │      │  • LLM calls         │
│    TriageResult     │  • Forwards delta  │      │  • Session / history │
│  • Renders   │      │    + context to   │      │  • State machine     │
│    PathPlan  │      │    edge-pod       │      │  • Structured output │
│              │      │  • Persists final │      │  • Confidence scoring│
│              │      │    outcome        │      │  • Fallback logic    │
└──────────────┘      └──────────────────┘      └──────────────────────┘
```

#### Responsibilities

| Layer | Owns | Does NOT Own |
|-------|------|--------------|
| **Frontend** | UI rendering, user interaction, TypeScript type contracts | Business logic, AI calls, DB access |
| **Platform Backend** | Authentication, DB reads/writes, context assembly from DB, persisting final outcomes (witness, triage result) | Prompt engineering, LLM API calls, conversation history management |
| **Edge-Pod** | Prompt templates, LLM API calls, conversation session state, state machine progression, structured output validation | User auth, DB access, persistence of business entities |

#### Why Separate?

1. **Independent scaling** — AI inference is bursty and GPU/API-bound; platform is I/O-bound
2. **Independent iteration** — Prompt changes, model swaps, and A/B tests don't require platform deploys
3. **Cost isolation** — LLM API costs are tracked and budgeted separately
4. **Security boundary** — Edge-pod never has direct DB access; receives only what the platform sends

### 2.2 Session Ownership & Delta Pattern

For multi-turn AI interactions (AI-00 triage, AI-06 criteria suggestions),
the edge-pod owns the conversation session:

```
Platform                         Edge-Pod
   │                                │
   │  POST /session/start           │
   │  { context, first_message }    │
   │───────────────────────────────▶│
   │                                │ ← creates session, stores history
   │  { session_id, result }        │
   │◀───────────────────────────────│
   │                                │
   │  POST /session/{id}/message    │
   │  { message }  ← delta only    │
   │───────────────────────────────▶│
   │                                │ ← appends to history, runs inference
   │  { result }                    │
   │◀───────────────────────────────│
   │                                │
   │  ... repeat ...                │
   │                                │
   │  POST /session/{id}/message    │
   │  { message }                   │
   │───────────────────────────────▶│
   │                                │ ← final inference, state = ready
   │  { result + proposed_plan }    │
   │◀───────────────────────────────│
   │                                │
   │  (Platform persists outcome)   │
```

**Key design decisions:**

| Decision | Rationale |
|----------|-----------|
| **Delta-only messages** | Sending only the new message per turn is lighter than re-sending the full conversation. Edge-pod manages its own history. |
| **Context sent at session start** | Platform sends full DB context (user profile, community, location) once at session creation. Edge-pod caches it for the session lifetime. |
| **Context refresh (optional)** | For long sessions, platform can send a `context_refresh` payload on any message turn if DB state has changed. |
| **Session TTL** | Edge-pod sessions expire after 30 minutes of inactivity. Platform must start a new session if expired. |
| **Platform persists outcome** | When triage reaches `ready`, platform saves the `TriageResult`, triage transcript, and proposed `PathPlan` to its own DB. The edge-pod session can then be discarded. |

### 2.3 Context Assembly

The platform backend is responsible for assembling the context payload from
its own data sources before sending to the edge-pod:

```typescript
interface EdgePodContext {
  // User identity
  user_id: string;
  user_name: string;
  user_tier: number;

  // Location (if available)
  location?: { lat: number; lng: number };

  // Community context
  community_id?: string;
  community_name?: string;

  // Tandang signals (pre-fetched by platform)
  reputation_score?: number;
  recent_witnesses?: Array<{ witness_id: string; title: string; track_hint: string }>;

  // Platform metadata
  platform_version: string;
  locale: string;
}
```

This context is opaque to the frontend — the frontend only sends user input
to the platform API, which enriches it with DB context before forwarding to
the edge-pod.

---

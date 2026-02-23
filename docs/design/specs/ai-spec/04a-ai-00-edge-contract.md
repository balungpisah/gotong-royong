> [← Back to AI Spec index](../AI-SPEC-v0.2.md)
> Design spec: [04-ai-00-triage.md](./04-ai-00-triage.md)

# AI-00 Triage — Edge-Pod Implementation Contract

> **Status:** Draft v0.1
> **Last updated:** 2026-02-22
>
> This document bridges the AI-00 design spec (Section 04) to wire-level
> API contracts. It defines the exact endpoints, request/response schemas,
> session lifecycle, and TypeScript type mappings that the edge-pod must
> implement and the platform backend must call.
>
> **This is also the template** for all future AI touchpoint contracts.
> Copy this structure when adding contracts for AI-01 through AI-09.

---

## 1. Contract Overview

| Property | Value |
|----------|-------|
| **Touch Point** | AI-00 Conversational Triage |
| **Edge-Pod Base** | `{EDGE_POD_URL}/api/v1/triage` |
| **Auth** | Platform service token (header: `X-Platform-Token`) |
| **Session Model** | Stateful, multi-turn, edge-pod-owned |
| **Message Pattern** | Delta-only (new message per turn) |
| **Context Pattern** | Full context at session start, optional refresh |
| **Frontend Types** | `TriageResult`, `PathPlan`, `TriageConfidence`, `ContextBarState`, `EntryRoute`, `SeedHint`, `TrackHint`, `TrajectoryType`, `CardEnrichment`, `TriageBudget` |

---

## 2. Endpoints

### 2.1 Start Session

Creates a new triage session. Platform sends the first user message along
with full context from its DB.

```
POST /api/v1/triage/start
```

**Request:**

```jsonc
{
  // The user's first message
  "message": "Jalan di depan rumah rusak parah sudah 3 bulan",

  // Platform-assembled context (see §2.3 in 02-three-layer-architecture.md)
  "context": {
    "user_id": "u-001",
    "user_name": "Ahmad Hidayat",
    "user_tier": 2,
    "location": { "lat": -6.175, "lng": 106.827 },
    "community_id": "comm-menteng-05",
    "community_name": "RT 05 Menteng",
    "reputation_score": 72.5,
    "recent_witnesses": [],
    "platform_version": "0.1.0",
    "locale": "id"
  },

  // Optional: media URLs uploaded by user
  "media_urls": []
}
```

**Response:**

```jsonc
{
  // Edge-pod session identifier — platform must store and re-send
  "session_id": "triage-sess-abc123",

  // Triage result — maps to TypeScript `TriageResult`
  "result": {
    "bar_state": "probing",         // ContextBarState
    "route": "komunitas",           // EntryRoute
    "track_hint": null,             // TrackHint | null
    "seed_hint": null,              // SeedHint | null
    "confidence": {                 // TriageConfidence | null
      "score": 0.4,
      "label": "Menganalisis..."
    },
    "proposed_plan": null,          // PathPlan | null (only when ready)
    "duplicate": null,              // duplicate detection (from AI-03)
    "trajectory_type": null,       // TrajectoryType | null (only when leaning+)
    "card_enrichment": null,       // CardEnrichment | null (only when ready)
    "budget": {                    // TriageBudget — platform-computed (see §4.5)
      "total_tokens": 6000,
      "used_tokens": 820,
      "remaining_tokens": 5180,
      "budget_pct": 0.14,
      "can_continue": true,
      "turn_count": 1,
      "max_turns": 8
    }
  },

  // Current turn number (1-based; start = turn 1)
  "turn_count": 1,

  // AI's response text to show the user
  "ai_message": "Bisa ceritakan lebih detail? Sudah berapa lama jalannya rusak dan apakah sudah dilaporkan ke RT?"
}
```

### 2.2 Send Message (Delta)

Continues an existing session. Platform sends **only the new user message**.
Edge-pod appends to its internal history and runs inference.

```
POST /api/v1/triage/session/{session_id}/message
```

**Request:**

```jsonc
{
  "message": "Sudah 3 bulan, sudah lapor ke RT tapi belum ada tindakan",

  // Optional: refresh context if DB state changed
  "context_refresh": null
}
```

**Response:** Same shape as start response (minus `session_id`):

```jsonc
{
  "result": {
    "bar_state": "leaning",
    "route": "komunitas",
    "track_hint": "tuntaskan",
    "seed_hint": "Keresahan",
    "confidence": { "score": 0.72, "label": "Tuntaskan · 72%" },
    "proposed_plan": null,
    "duplicate": null,
    "budget": {
      "total_tokens": 6000,
      "used_tokens": 1740,
      "remaining_tokens": 4260,
      "budget_pct": 0.29,
      "can_continue": true,
      "turn_count": 2,
      "max_turns": 8
    }
  },
  "turn_count": 2,
  "ai_message": "Saya mulai memahami. Sepertinya ini perlu tindakan penyelesaian. Ada informasi tambahan tentang dampaknya ke warga?"
}
```

### 2.3 Final Turn (Ready State)

When the edge-pod determines sufficient confidence, it returns
`bar_state: "ready"` (or `"vault-ready"` / `"siaga-ready"`) along with
the proposed `PathPlan`.

**Response example:**

```jsonc
{
  "result": {
    "bar_state": "ready",
    "route": "komunitas",
    "track_hint": "tuntaskan",
    "seed_hint": "Keresahan",
    "confidence": { "score": 0.92, "label": "Tuntaskan · 92%" },
    "proposed_plan": {
      "plan_id": "plan-generated-001",
      "version": 1,
      "title": "Rencana Perbaikan Jalan Jl. Mawar",
      "summary": "Rencana penyelesaian melalui koordinasi warga, penggalangan dana, dan pelaksanaan perbaikan.",
      "track_hint": "tuntaskan",
      "seed_hint": "Keresahan",
      "branches": [
        {
          "branch_id": "branch-main",
          "label": "Jalur Utama",
          "parent_checkpoint_id": null,
          "phases": [
            {
              "phase_id": "phase-1",
              "title": "Pengumpulan Bukti",
              "objective": "Dokumentasi dan kesaksian warga",
              "status": "planned",
              "source": "ai",
              "locked_fields": [],
              "checkpoints": []
            }
            // ... more phases
          ]
        }
      ]
    },
    "duplicate": null,
    "trajectory_type": "aksi",     // TrajectoryType
    "budget": {                    // TriageBudget — final turn
      "total_tokens": 6000,
      "used_tokens": 4920,
      "remaining_tokens": 1080,
      "budget_pct": 0.82,
      "can_continue": false,
      "turn_count": 5,
      "max_turns": 8
    },
    "card_enrichment": {           // CardEnrichment
      "icon": "construction",
      "trajectory_type": "aksi",
      "title": "Jalan Berlubang Jl. Mawar, 30 KK Terdampak",
      "hook_line": "3 bulan tanpa respons — warga turun tangan sendiri",
      "pull_quote": "Sudah lapor ke RT tapi belum ada tindakan",
      "body": "Warga Jl. Mawar melaporkan jalan rusak parah selama 3 bulan...",
      "sentiment": "hopeful",
      "intensity": 3,
      "entity_tags": [
        { "label": "Jl. Mawar", "entity_type": "lingkungan", "confidence": 0.95 }
      ]
    }
  },
  "ai_message": "Triase selesai! Saya mengusulkan jalur Tuntaskan dengan rencana perbaikan jalan. Silakan tinjau rencana dan buat saksi.",

  // Conversation summary for persistence
  "conversation_summary": "Warga melaporkan jalan rusak di Jl. Mawar selama 3 bulan tanpa tindakan dari RT. Dampak: kemacetan dan kecelakaan ringan.",

  // Full transcript for platform to persist
  "transcript": [
    { "role": "user", "content": "Jalan di depan rumah rusak parah sudah 3 bulan", "timestamp": "2026-02-21T10:00:00Z" },
    { "role": "assistant", "content": "Bisa ceritakan lebih detail?...", "timestamp": "2026-02-21T10:00:02Z" },
    { "role": "user", "content": "Sudah 3 bulan, sudah lapor ke RT...", "timestamp": "2026-02-21T10:00:30Z" },
    { "role": "assistant", "content": "Saya mulai memahami...", "timestamp": "2026-02-21T10:00:32Z" },
    { "role": "user", "content": "Banyak motor jatuh karena lubang besar", "timestamp": "2026-02-21T10:01:00Z" },
    { "role": "assistant", "content": "Triase selesai!...", "timestamp": "2026-02-21T10:01:02Z" }
  ]
}
```

### 2.4 End Session (Optional)

Platform can explicitly end a session. Otherwise sessions expire after TTL.

```
DELETE /api/v1/triage/session/{session_id}
```

---

## 3. TypeScript Type Mapping

The edge-pod response `result` field maps **directly** to the frontend
`TriageResult` type. The platform backend passes it through without
transformation.

| Edge-Pod Field | TypeScript Type | File |
|----------------|-----------------|------|
| `result` | `TriageResult` | `types/triage.ts` |
| `result.bar_state` | `ContextBarState` | `types/triage.ts` |
| `result.route` | `EntryRoute` | `types/triage.ts` |
| `result.track_hint` | `TrackHint` | `types/blocks.ts` |
| `result.seed_hint` | `SeedHint` | `types/path-plan.ts` |
| `result.confidence` | `TriageConfidence` | `types/triage.ts` |
| `result.proposed_plan` | `PathPlan` | `types/path-plan.ts` |
| `result.duplicate` | `TriageResult.duplicate` | `types/triage.ts` |
| `result.trajectory_type` | `TrajectoryType` | `types/card-enrichment.ts` |
| `result.card_enrichment` | `CardEnrichment` | `types/card-enrichment.ts` |
| `result.budget` | `TriageBudget` | `types/triage.ts` |

**Frontend `TriageService` interface** (in `services/types.ts`):

```typescript
export interface TriageService {
  startTriage(content: string): Promise<TriageResult>;
  updateTriage(sessionId: string, answer: string): Promise<TriageResult>;
}
```

The platform backend implements this interface by:
1. Calling edge-pod endpoints
2. Extracting `result` from the response
3. Returning it as `TriageResult` to the frontend

The `ai_message` field is used by the platform to construct the AI response
text shown in the triage chat UI. The frontend's `TriageStore` does not need
to know about the edge-pod — it only sees `TriageResult`.

---

## 4. Session Lifecycle

```
                    ┌──────────┐
                    │  (none)  │
                    └────┬─────┘
                         │ POST /start
                         ▼
                    ┌──────────┐
              ┌─────│ probing  │◀────┐
              │     └────┬─────┘     │
              │          │ POST /message
              │          ▼           │
              │     ┌──────────┐     │
              │     │ leaning  │─────┘  (may loop back to probing)
              │     └────┬─────┘
              │          │ POST /message (confidence ≥ threshold)
              │          ▼
              │     ┌────────────────┐
              │     │ ready          │  (or vault-ready, siaga-ready)
              │     │ + proposed_plan│
              │     └────┬───────────┘
              │          │
              │          ▼
              │     ┌──────────┐
              └────▶│ expired  │  (TTL 30min, or DELETE)
                    └──────────┘
```

**State transitions** (edge-pod internal):

| From | To | Trigger |
|------|----|---------|
| (none) | `probing` | Session start — always begins with a follow-up question |
| `probing` | `probing` | Low confidence after message — ask another question |
| `probing` | `leaning` | Moderate confidence (0.5–0.8) — show track hint |
| `leaning` | `probing` | User response contradicts or adds complexity |
| `leaning` | `ready` | High confidence (≥ 0.8) — propose path plan |
| `leaning` | `vault-ready` | Vault signals detected (see 04 §4.7a) |
| `leaning` | `siaga-ready` | Emergency signals detected (see 04 §4.7b) |
| `ready` | `expired` | TTL or explicit DELETE |
| any | `expired` | 30 min inactivity |

**Session Budget Model:** Instead of a flat turn count, each triage session
gets a **token budget** — the total input tokens the edge-pod may consume.
This is more equitable than counting turns: short focused messages get more
turns, verbose messages get fewer, and the real resource (LLM compute) is
what's actually capped.

### 4.1 Budget Allocation

Budget = `f(user_tier, trajectory_complexity)`. The platform computes the
budget at session start based on the user's Tandang tier and the trajectory
complexity class (determined after the first message or from the trajectory
grid primer).

**Trajectory complexity classes:**

| Class | Trajectories | Rationale |
|-------|-------------|-----------|
| **Simple** | bantuan, pencapaian, siaga, data | Quick intent — 2–3 turns typically enough |
| **Standard** | aksi, pantau, program | Moderate probing needed |
| **Complex** | advokasi, mufakat, mediasi, vault | Multi-party, evidence, or sensitive — needs depth |

**Budget matrix (input tokens):**

| | Shadow (0) | Novice (1) | Contributor (2) | Pillar (3) | Keystone (4) |
|---|---|---|---|---|---|
| **Simple** | 2,000 | 3,000 | 4,000 | 5,000 | 6,000 |
| **Standard** | 3,000 | 4,000 | 6,000 | 8,000 | 10,000 |
| **Complex** | 3,000 | 5,000 | 8,000 | 10,000 | 12,000 |

### 4.2 Safety Rails

Token budget alone isn't enough. Hard limits prevent edge cases:

| Rail | Value | Why |
|------|-------|-----|
| **Hard turn cap** | 8 turns (absolute max, all tiers) | Prevent infinite loops even with budget remaining |
| **Min turns** | 2 turns | AI must ask at least 1 follow-up (prevents insta-ready gaming) |
| **Per-message cap** | 2,000 characters | Prevent token-bomb messages |
| **Idle timeout** | 5 min between turns | Prevent slow-drip abuse |
| **Budget grace** | Last turn always allowed if >80% budget used | Don't cut off mid-conversation awkwardly |

### 4.3 AI Internal Strategy

The edge-pod sees its remaining budget and adjusts behavior:

| Budget Remaining | AI Behavior |
|-----------------|-------------|
| **>70%** | Explore freely, weave in 1 identity anchor question |
| **30–70%** | Start converging toward a plan, 1 more anchor if natural |
| **<30%** | Wrap up, produce `ready` state with best available info |
| **Exhausted** | MUST produce `ready` or `manual`, no more turns |

### 4.4 Budget in Responses

The response includes a `budget` object so the frontend can show an energy
indicator ("Sisa Energi AI") and the platform can enforce the limit.

| Turn | Expected Behavior |
|------|-------------------|
| 1 | Budget allocated, probing begins |
| 2–6 | Normal cycle — AI adjusts depth based on remaining budget |
| 7 | Edge-pod SHOULD aim for `ready` if budget < 30% |
| 8 | **Hard cap** — MUST return `ready` or `manual` regardless of budget |
| 9+ | Edge-pod MUST reject with `422` |

### 4.5 Token Accounting — Who Counts?

The **platform backend** counts tokens, not the AI. The edge-pod has no
built-in awareness of token consumption — it receives its remaining budget
as an external number.

**Flow:**

```
┌──────────┐         ┌──────────────┐         ┌───────────┐
│ Frontend │ ──msg──▶│   Platform   │ ──msg──▶│ Edge-Pod  │
│          │         │   Backend    │         │  (LLM)    │
│          │         │              │         │           │
│          │◀─result─│  counts via  │◀─result─│  behaves  │
│  energy  │         │  LLM API    │         │  per hint │
│   bar    │         │  usage stats │         │           │
└──────────┘         └──────────────┘         └───────────┘
```

**Step-by-step:**

1. **Session start** — Platform computes `total_tokens` from the budget
   matrix (§4.1) using `user_tier × trajectory_complexity`.

2. **Each turn** — Platform forwards the user message to the edge-pod
   LLM. The LLM API response includes `usage.prompt_tokens` and
   `usage.completion_tokens`. Platform adds both to a running
   `used_tokens` counter.

3. **Budget injection** — Before each turn, platform injects the
   remaining budget into the edge-pod's **system prompt context**:
   ```
   [Budget: 2,340 of 6,000 tokens remaining. Adjust depth accordingly.]
   ```
   The AI reads this as a behavioral hint — it doesn't measure tokens
   itself. It just follows the strategy guidelines (§4.3) based on the
   number it sees.

4. **Response enrichment** — Platform constructs the `budget` object
   from its own accounting and attaches it to the response:
   ```jsonc
   "budget": {
     "total_tokens": 6000,
     "used_tokens": 3660,
     "remaining_tokens": 2340,
     "budget_pct": 0.61,      // 3660 / 6000
     "can_continue": true,
     "turn_count": 3,
     "max_turns": 8
   }
   ```

5. **Frontend display** — The frontend reads `budget` to render the
   "Sisa Energi AI" energy bar. The bar shows `(1 - budget_pct) × 100`
   as remaining percentage, color-coded green → amber → red.

6. **Enforcement** — When `remaining_tokens ≤ 0` or `turn_count ≥ 8`,
   platform stops forwarding messages and expects the edge-pod's last
   response to be `ready` or `manual`.

**Why platform-side, not AI-side?**

| Reason | Detail |
|--------|--------|
| **Accuracy** | LLM API returns exact token counts; the AI can only estimate |
| **Trust boundary** | The AI is the resource being metered — it shouldn't self-report consumption |
| **Simplicity** | Edge-pod stays stateless about budgets; platform is already the accounting layer |
| **Consistency** | Same counting method across all AI touchpoints (AI-00 through AI-09) |

---

## 5. Error Handling

| HTTP Status | Meaning | Platform Action |
|-------------|---------|-----------------|
| `200` | Success | Pass `result` to frontend |
| `404` | Session expired/not found | Start new session |
| `408` | Edge-pod timeout (>5s) | Return fallback `TriageResult` with `bar_state: "manual"` |
| `422` | Invalid input | Log and return error to frontend |
| `429` | Rate limited | Retry with exponential backoff (max 3) |
| `500+` | Edge-pod error | Return fallback with `bar_state: "manual"` |

**Fallback `TriageResult`** (when edge-pod is unavailable):

```json
{
  "bar_state": "manual",
  "route": "komunitas",
  "confidence": null,
  "track_hint": null,
  "seed_hint": null,
  "proposed_plan": null
}
```

This shows the manual track selection grid so users can always proceed
even without AI.

---

## 6. Rate Limiting & Abuse Prevention

Triage sessions consume LLM tokens. Multiple layers prevent abuse:

### 6.1 Platform Backend (Enforced Before Edge-Pod Call)

| Control | Rule | Response |
|---------|------|----------|
| **Session rate limit** | Max **10 sessions per user per hour** | HTTP `429` to frontend, show "Coba lagi nanti" |
| **Turn limit** | Max **8 turns per session** + token budget (see §4.1–4.5) | When budget exhausted or turn 8 reached, auto-transition to `ready` or `manual` |
| **Concurrent sessions** | Max **1 active session per user** | Return existing `session_id` or reject |
| **Message length** | Max **2000 characters per message** | Truncate or reject with `422` |
| **Cooldown** | **30 seconds** between session completion and next session start | Frontend enforces; backend rejects if violated |

### 6.2 Tier-Based Quotas (Tandang Integration)

The `user_tier` from the context payload (already sent at session start)
determines daily AI interaction budgets:

| Tier | Daily Triage Sessions | Rationale |
|------|----------------------|-----------|
| **Keystone** (4) | 30 | Trusted community leaders |
| **Pillar** (3) | 20 | Established contributors |
| **Contributor** (2) | 10 | Active members |
| **Novice** (1) | 5 | New users, earning trust |
| **Shadow** (0) | 2 | Unverified / low-trust — heavily restricted |

The platform backend tracks `triage_session_count` per user per day and
rejects with `429` when the quota is exceeded. The edge-pod does not need
to know about quotas — the platform gatekeeps.

### 6.3 Bot / Adversarial Resistance

| Layer | Mechanism | Notes |
|-------|-----------|-------|
| **Proof of Humanity** | Platform requires phone-verified account to access triage | Blocks anonymous bot farms |
| **Behavioral fingerprint** | Platform tracks time-between-messages, typing cadence, session patterns | Flag accounts with bot-like patterns (e.g., <500ms between receiving AI response and sending next message) |
| **Content hash dedup** | Platform hashes first user message per session; reject exact duplicates within 1 hour | Prevents replay attacks |
| **Progressive challenge** | If flagged as suspicious (3+ rapid sessions, identical messages), require lightweight challenge before next session | Not a traditional CAPTCHA — a contextual question like "Kamu di RT berapa?" that bots can't trivially answer |
| **Edge-pod prompt hardening** | Edge-pod system prompt includes jailbreak resistance; refuses to act as general-purpose chatbot | Adversarial prompts get `bar_state: "manual"` fallback |

### 6.4 Identity Anchoring (Conversational Security)

The triage AI weaves identity-anchoring questions **naturally into the
conversation** — framed as rapport-building, not interrogation. This serves
two purposes:

1. **Anti-impersonation**: Answers become identity anchors that only the real
   user would know. If someone later tries to act as them, they'd fail these
   contextual checks.
2. **Integrity measurement**: Consistency of answers across sessions feeds
   into tandang's Integrity (I) score.

**Prompt strategy — the AI must:**

- **Mirror the user's style.** If they write `"jalan dpn rumah ancur parah bro"`,
  the AI responds in the same register — casual, abbreviated, relatable.
  If they write formally, the AI matches that too.
- **Frame anchoring as closeness, not verification.** The vibe is:
  *"Biar lebih deket, kasih tau hal yang cuma kamu yang tau — biar nanti
  ga ada yang bisa pura-pura jadi kamu di sini."*
- **Ask naturally within triage flow**, not as a separate interrogation step.
  The AI picks 1–2 anchoring questions per session that fit the conversation
  context.

**Anchoring question categories:**

| Category | Example Questions | What It Verifies |
|----------|-------------------|-----------------|
| **Location-specific** | "Di jalan apa tepatnya?", "Deket gang apa?" | User actually lives nearby |
| **Community context** | "Pak RT-nya siapa?", "Sudah berapa lama di sini?" | Membership in community |
| **Case-specific** | "Terakhir lihat kapan?", "Yang kena dampak siapa aja?" | Witness authenticity |
| **Personal anchor** | "Biasa dipanggil apa sama tetangga?" | Identity consistency |

**What the edge-pod returns:**

```jsonc
{
  "result": { /* ... normal triage result ... */ },

  // Extracted identity anchors — platform persists, NOT shown to other users
  "identity_anchors": [
    { "category": "location", "question": "Di jalan apa?", "answer": "Jl. Mawar gang 3", "confidence": 0.9 },
    { "category": "community", "question": "Pak RT siapa?", "answer": "Pak Harto", "confidence": 0.85 }
  ]
}
```

**Platform responsibilities:**
- Persist `identity_anchors` in user profile (encrypted, never exposed to other users)
- Cross-reference with previous sessions — consistent answers raise I score,
  contradictions trigger review
- On suspicious login / impersonation attempt, use stored anchors as challenge questions
- Never reveal what the "correct" answers are — the challenge is *having* the knowledge,
  not knowing the format

**Privacy note:** Anchors are PII-adjacent. They MUST be encrypted at rest,
excluded from analytics exports, and deletable on user request (GDPR-like).

### 6.5 Monitoring & Alerting

| Signal | Alert | Action |
|--------|-------|--------|
| User exceeds 3x normal session rate | Flag account | Review + possible `Shadow` tier demotion |
| Identical first-messages across 5+ users | Coordinated attack | IP-level rate limit + account review |
| Edge-pod returns errors for >10% of requests | System health | Page on-call, switch to fallback mode |
| Session completes in <2 seconds | Bot behavior | Flag for review |

---

## 7. What Platform Backend Persists

When triage reaches a `ready` state and the user creates a witness:

| Data | Stored In | Source |
|------|-----------|--------|
| `TriageResult` | `witness.triage` column | Edge-pod `result` |
| Triage transcript | `witness.triage_messages` | Edge-pod `transcript` |
| Proposed `PathPlan` | `witness.plan` | Edge-pod `result.proposed_plan` |
| Conversation summary | `witness.summary` | Edge-pod `conversation_summary` |
| User's first message | `witness.title` (truncated) | First user message from transcript |

The edge-pod session can be discarded after the platform persists. The
edge-pod is ephemeral; the platform is the system of record.

---

## 8. Template: Applying This Pattern to Other Touch Points

When creating a new AI touchpoint contract (AI-01 through AI-09), copy
this document structure and fill in:

### Required Sections

1. **Contract Overview** — Touch point ID, base URL, auth, session model,
   message pattern, frontend TypeScript types involved
2. **Endpoints** — Request/response schemas for each endpoint
3. **TypeScript Type Mapping** — How edge-pod response maps to frontend types
4. **Session Lifecycle** (if multi-turn) — State diagram, transitions, TTL
5. **Error Handling** — HTTP status codes, fallback behavior
6. **What Platform Persists** — What the platform saves after the AI call

### Optional Sections (as needed)

- **Prompt Strategy Reference** — Link to the design spec section
- **Context Assembly** — If the touch point needs special context beyond
  the standard `EdgePodContext`
- **Batch Mode** — If the touch point can process multiple items (e.g., AI-03
  duplicate detection across a batch of seeds)
- **Webhook Callback** — If the edge-pod needs to notify the platform
  asynchronously (e.g., AI-04 moderation hold resolution)

### Naming Convention

```
{NN}a-ai-{NN}-edge-contract.md
```

Examples:
- `04a-ai-00-edge-contract.md` (this file)
- `05a-ai-01-edge-contract.md`
- `10a-ai-06-edge-contract.md`

---

## 9. Open Questions

- [ ] **Session storage**: Redis? In-memory with persistence? Edge-pod team decides.
- [ ] **Streaming**: Should `ai_message` be streamed (SSE) for real-time typing effect?
- [ ] **Context refresh frequency**: Should platform send refreshed context every N turns?
- [ ] **Transcript format**: Should edge-pod return the full transcript on every turn, or only on the final `ready` turn?
- [ ] **Media handling**: How are `media_urls` forwarded — as URLs for the LLM to fetch, or as base64?

---

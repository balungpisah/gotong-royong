# AI-09a: Signal Completion Resolution

> Subordinate spec to AI-09 (Credit Accreditation).
> Defines how content-directed signals resolve when a witness reaches terminal state.

## 1. Problem

Content-directed signals (`saksi`, `perlu_dicek`) are cast while a witness is active, but their credit impact depends on the *outcome* of the witness. A "Saya Saksi" signal on a witness that turns out to be fabricated should not earn the same credit as one on a verified incident.

> **Note:** `bagus` (quality upvote) was removed from the signal pipeline and renamed to **"Dukung"** (support). It is now a non-Tandang social action handled by FeedStore. See §10.

**Key insight:** signals should **hang as pending** until the witness itself reaches a terminal state (`resolved` or `closed`). The witness lifecycle IS the verification mechanism — no separate community jury or moderator queue needed.

## 2. WitnessCloseReason

When a koordinator (or system automation) moves a witness to `resolved` or `closed`, they must supply a `close_reason`:

| Reason         | Meaning                              | Signal outcome   |
|----------------|--------------------------------------|------------------|
| `selesai`      | Goal achieved, resolved successfully | Positive signals rewarded |
| `tidak_valid`  | Content was false/invalid            | Doubt signals rewarded    |
| `duplikat`     | Merged into another witness          | Neutral — no credit swing |
| `kedaluwarsa`  | Expired / no longer relevant         | Neutral                   |
| `ditarik`      | Creator withdrew                     | Neutral                   |

## 3. Signal Resolution Matrix

Maps `(signal_type × close_reason)` → credit outcome:

| Signal          | `resolved`/`selesai` | `closed`/`tidak_valid` | `closed`/other |
|-----------------|---------------------|------------------------|----------------|
| saksi (witness) | I credit earned     | —                      | Neutral        |
| perlu_dicek     | Small J penalty     | I+J credit earned      | Neutral        |
| vouch (person→content) | I+C credit   | I penalty              | Neutral        |

**Neutral** means no credit change — the signal is marked `resolved_neutral` and archived.

## 4. Signal Lifecycle States

```
pending → resolved_positive | resolved_negative | resolved_neutral | expired
```

- `pending` — witness is still active, signal is recorded but not yet scored
- `resolved_positive` — signal aligned with the outcome (earns credit)
- `resolved_negative` — signal contradicted the outcome (loses credit or no gain)
- `resolved_neutral` — ambiguous outcome, no credit swing
- `expired` — witness closed without clear positive/negative mapping

## 5. Resolution Flow

Backend event triggered by witness status change:

```
koordinator marks witness → resolved/closed (with close_reason)
  → Backend event: resolve_pending_signals(witness_id, terminal_status, close_reason)
    → For each pending signal on this witness:
      → Look up (signal_type × close_reason) in resolution matrix
      → Calculate credit delta: base_points × tier_multiplier
      → Update tandang I/C/J scores for signal giver
      → Update tandang I/C/J scores for content creator (if applicable)
    → Push notification to all signal givers:
      "Sinyal kamu pada '{title}' telah diselesaikan"
```

### 5.1 Credit Calculation

```
credit_delta = BASE_POINTS[signal_type] × TIER_MULTIPLIER[giver_tier] × OUTCOME_SIGN[outcome]
```

| Signal       | Base Points | Affected Score |
|--------------|-------------|----------------|
| saksi        | 5           | I (Integrity)  |
| perlu_dicek  | 4           | I + J          |
| vouch        | 6           | I + C          |

Tier multiplier: `[1.0, 1.1, 1.25, 1.5, 2.0]` for tiers 0–4.

## 6. API Contracts

### 6.1 Cast Signal

```
POST /api/witnesses/{witness_id}/signals
Content-Type: application/json

{
  "signal_type": "saksi" | "perlu_dicek"
}

→ 201 Created
{
  "signal_id": "sig-abc123",
  "witness_id": "w-001",
  "user_id": "u-001",
  "signal_type": "saksi",
  "outcome": "pending",
  "created_at": "2026-02-22T10:00:00Z"
}
```

### 6.2 Remove Signal (Undo)

```
DELETE /api/witnesses/{witness_id}/signals/{signal_type}

→ 204 No Content
```

### 6.3 Get My Relation

```
GET /api/witnesses/{witness_id}/my-relation

→ 200 OK
{
  "vouched": false,
  "witnessed": true,
  "flagged": false,
  "supported": false
}
```

### 6.4 Get Signal Counts

```
GET /api/witnesses/{witness_id}/signal-counts

→ 200 OK
{
  "vouch_positive": 5,
  "vouch_skeptical": 1,
  "witness_count": 8,
  "dukung_count": 12,
  "flags": 2
}
```

### 6.5 Get Resolutions (after witness completion)

```
GET /api/witnesses/{witness_id}/resolutions

→ 200 OK
[
  {
    "signal_id": "sig-abc123",
    "witness_id": "w-001",
    "user_id": "u-001",
    "signal_type": "saksi",
    "outcome": "resolved_positive",
    "created_at": "2026-02-22T10:00:00Z",
    "resolved_at": "2026-02-23T15:00:00Z",
    "credit_delta": 5.5
  }
]
```

## 7. Frontend Behavior

### 7.1 Pending State

While a witness is active, signal chips show a subtle animated dot indicating "pending — awaiting outcome." This teaches users that their signal will resolve when the witness completes.

### 7.2 Resolved State

After witness completion, signal chips show:
- **Checkmark** for `resolved_positive`
- **Dash** for `resolved_neutral`
- **X** for `resolved_negative`

### 7.3 Feed Card Badge

On completed witnesses, the feed card shows a resolution summary:
- "3 sinyal diselesaikan" with outcome breakdown

## 8. Edge Cases

1. **User removes signal before resolution:** Signal is deleted, no resolution occurs.
2. **Witness reopened after resolution:** Signals remain resolved. New signals can be cast and will be pending again.
3. **Multiple signals by same user:** Each signal type resolves independently. A user can have both `saksi` (pending) and `perlu_dicek` (pending) on the same witness.
4. **Koordinator changes close_reason:** Re-runs resolution for all signals with new reason. Previous credit deltas are reversed first.

## 9. Signal Labels Storage Contract

Signal labels are LLM-generated contextual labels for the 2 content-directed signal chips. They are produced by AI-01 as part of `CardEnrichment` during triage and stored on the witness record.

### 9.1 Schema

```sql
-- SurrealDB: signal_labels stored as an object field on the witness record
-- Populated from card_enrichment.signal_labels during triage completion

DEFINE FIELD signal_labels ON TABLE witness TYPE option<object>;
DEFINE FIELD signal_labels.saksi ON TABLE witness TYPE object;
DEFINE FIELD signal_labels.saksi.label ON TABLE witness TYPE string ASSERT string::len($value) <= 15;
DEFINE FIELD signal_labels.saksi.desc ON TABLE witness TYPE string;
DEFINE FIELD signal_labels.saksi.icon ON TABLE witness TYPE option<string>;
DEFINE FIELD signal_labels.perlu_dicek ON TABLE witness TYPE object;
DEFINE FIELD signal_labels.perlu_dicek.label ON TABLE witness TYPE string ASSERT string::len($value) <= 15;
DEFINE FIELD signal_labels.perlu_dicek.desc ON TABLE witness TYPE string;
DEFINE FIELD signal_labels.perlu_dicek.icon ON TABLE witness TYPE option<string>;
```

### 9.2 Data Flow

1. **AI triage** (AI-00 → AI-01) generates `card_enrichment.signal_labels`
2. **Backend** stores `signal_labels` on the witness record at triage completion
3. **Feed API** inlines `signal_labels` into the `FeedItem` response
4. **Frontend** reads `signal_labels` from `FeedItem`; falls back to hardcoded defaults if absent (legacy witnesses, API failure)

### 9.3 Migration

Legacy witnesses created before signal labels will have `signal_labels = NONE`. The frontend handles this gracefully with default labels ("Saya Saksi" / "Perlu Dicek"). No backfill migration is required — labels are purely presentational.

## 10. Dukung — Non-Tandang Social Action

**"Dukung"** (support, formerly "Bagus") is a lightweight social action that does **not** flow through the Tandang signal resolution pipeline. It was removed from content-directed signals because it doesn't cleanly map to any Tandang score dimension (I/C/J).

### 10.1 Design Rationale

| Concern | Signal Pipeline (saksi, perlu_dicek) | Dukung |
|---------|--------------------------------------|--------|
| Tandang scoring | Yes — affects I, J scores | No — purely social |
| Resolution outcome | Resolves with witness completion | N/A — instant toggle |
| Auto-pantau trigger | Yes (witnessed, flagged) | No — too lightweight |
| Credit calculation | BASE_POINTS × TIER × OUTCOME | None |
| Store | SignalStore | FeedStore |

### 10.2 Frontend Implementation

- **UI**: Heart icon button in feed card footer (between member count and action buttons)
- **State**: `MyRelation.supported` (boolean) + `SignalCounts.dukung_count` (number)
- **Toggle**: `FeedStore.toggleDukung(witnessId)` — optimistic, no API call needed for mock
- **Visual**: Pill-shaped button, rose-colored when active, filled heart icon

### 10.3 Backend Contract (Future)

```
POST /api/witnesses/{witness_id}/dukung
→ 204 No Content (toggle)

GET response includes:
  my_relation.supported: boolean
  signal_counts.dukung_count: number
```

No resolution, no credit, no tandang score impact. Simple counter.

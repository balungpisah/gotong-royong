> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

# Identity Anchor Graph — Warga Identity Data Model

> **Status:** Draft v0.1
> **Last updated:** 2026-02-22
>
> This document defines how identity data gathered during AI-00 triage
> conversations is modeled, stored, and queried within the existing
> ontology graph. It extends the triple model from
> [ONTOLOGY-VOCAB-v0.1.md](../ONTOLOGY-VOCAB-v0.1.md) to cover **people**
> in addition to **content**.
>
> **Companion docs:**
> - [04a — AI-00 Edge Contract §6.4](./04a-ai-00-edge-contract.md) — how
>   the AI extracts anchors during conversation
> - [ONTOLOGY-VOCAB-v0.1.md](../ONTOLOGY-VOCAB-v0.1.md) — the triple
>   model this extends
> - [03 — Tandang](./03-tandang-handled-ai.md) — integrity scoring that
>   consumes anchor data

---

## 1. The Problem

The ontology spec (ONTOLOGY-VOCAB-v0.1.md) models **content** beautifully:
notes, plans, and alerts link to Wikidata concepts via `ABOUT`, `LOCATED_AT`,
and `HAS_ACTION` edges. But it treats **people** (`warga:*` nodes) as
second-class citizens — they only appear as authors, vouchers, or challengers.

We need to model **who people are** — where they live, who they know, what
roles they hold, what local knowledge they have — for three purposes:

1. **Anti-impersonation** — stored identity facts become challenge questions
   that only the real user can answer
2. **Integrity scoring** — anchor density and consistency feed the Tandang
   Integrity (I) score
3. **Cross-referencing** — identity data connects to content data through
   shared concept/place nodes, enabling powerful verification queries

---

## 2. The Elegant Insight

**Warga are subjects, just like notes are subjects.**

In the current graph, content nodes link to concepts:

```
note:abc  ──ABOUT──▶  concept:Q93189      (note is about eggs)
note:abc  ──LOCATED_AT──▶  place:pasar    (note located at market)
```

The extension: warga nodes ALSO link to concepts, places, and other warga:

```
warga:damar  ──ANCHOR──▶  place:rt05       (lives here)
warga:damar  ──ANCHOR──▶  concept:Q2461838 (member of posyandu)
warga:damar  ──ANCHOR──▶  warga:sari       (knows this person)
warga:damar  ──ANCHOR──▶  concept:Q8068    (knows about floods)
```

**One new edge type: `ANCHOR`.** Same Schema.org predicates on the edge,
same graph, same query patterns. Content and identity share the same
concept/place nodes.

---

## 3. The ANCHOR Edge

### 3.1 Schema

```sql
DEFINE TABLE ANCHOR TYPE RELATION
  FROM warga
  TO concept | place | warga
  ENFORCE;

DEFINE FIELD predicate   ON TABLE ANCHOR TYPE string  ASSERT $value != NONE;
DEFINE FIELD kind        ON TABLE ANCHOR TYPE string  ASSERT $value INSIDE ["location", "social", "role", "knowledge", "personal"];
DEFINE FIELD detail      ON TABLE ANCHOR TYPE option<string>;
DEFINE FIELD confidence  ON TABLE ANCHOR TYPE float   ASSERT $value >= 0.0 AND $value <= 1.0;
DEFINE FIELD visibility  ON TABLE ANCHOR TYPE string  ASSERT $value INSIDE ["system_only", "mutual", "public"];
DEFINE FIELD source      ON TABLE ANCHOR TYPE record<session>;
DEFINE FIELD gathered_at ON TABLE ANCHOR TYPE datetime;
```

### 3.2 Edge Fields

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `predicate` | string | Yes | Schema.org property (same convention as content triples) |
| `kind` | enum | Yes | Anchor category — for grouping, scoring, and privacy defaults |
| `detail` | string | No | Free-text context extracted from conversation |
| `confidence` | float | Yes | AI confidence in extraction accuracy (0.0–1.0) |
| `visibility` | enum | Yes | Privacy tier — who can query this anchor |
| `source` | record | Yes | Session ID that revealed this anchor (provenance) |
| `gathered_at` | datetime | Yes | When extracted |

### 3.3 Example

```sql
-- User said: "Saya tinggal di RT 05 RW 03, Gang Kenari"
RELATE warga:damar -> ANCHOR -> place:rt05_rw03
  SET predicate   = "schema:homeLocation",
      kind        = "location",
      detail      = "Gang Kenari",
      confidence  = 0.9,
      visibility  = "system_only",
      source      = session:abc123,
      gathered_at = time::now();

-- User said: "Saya kader posyandu"
RELATE warga:damar -> ANCHOR -> concept:Q2461838
  SET predicate   = "schema:memberOf",
      kind        = "role",
      detail      = "Kader posyandu RT 05",
      confidence  = 0.85,
      visibility  = "public",
      source      = session:abc123,
      gathered_at = time::now();

-- User said: "Bu Sari tetangga saya"
RELATE warga:damar -> ANCHOR -> warga:sari
  SET predicate   = "schema:knows",
      kind        = "social",
      detail      = "Tetangga di Gang Kenari",
      confidence  = 0.7,
      visibility  = "mutual",
      source      = session:def456,
      gathered_at = time::now();

-- User said: "Banjir bulan lalu rusak jembatan di Jl. Merdeka"
-- (Local knowledge — proves community membership)
RELATE warga:damar -> ANCHOR -> concept:Q8068
  SET predicate   = "schema:knowsAbout",
      kind        = "knowledge",
      detail      = "Banjir bulan lalu rusak jembatan Jl. Merdeka",
      confidence  = 0.8,
      visibility  = "system_only",
      source      = session:abc123,
      gathered_at = time::now();

-- User said: "Tetangga manggil saya Mas D"
RELATE warga:damar -> ANCHOR -> warga:damar
  SET predicate   = "schema:alternateName",
      kind        = "personal",
      detail      = "Mas D",
      confidence  = 0.95,
      visibility  = "system_only",
      source      = session:abc123,
      gathered_at = time::now();
```

---

## 4. Five Anchor Kinds

Each anchor kind has a default visibility, typical Schema.org predicates,
and a role in integrity scoring.

| Kind | Schema.org Predicates | Example | Default Visibility | Integrity Role |
|------|-----------------------|---------|--------------------|----------------|
| **location** | `schema:homeLocation`, `schema:workLocation` | "Tinggal di RT 05 RW 03" | `system_only` | Cross-ref with report locations |
| **social** | `schema:knows` | "Bu Sari tetangga saya" | `mutual` | Bidirectional validation |
| **role** | `schema:memberOf`, `schema:jobTitle` | "Saya kader posyandu" | `public` | Community membership proof |
| **knowledge** | `schema:knowsAbout` | "Banjir bulan lalu rusak jembatan" | `system_only` | Local knowledge verification |
| **personal** | `schema:alternateName`, `schema:identifier` | "Dipanggil Mas D" | `system_only` | Identity consistency |

### 4.1 Visibility Tiers

| Visibility | Who Can Query | Use Case |
|------------|---------------|----------|
| `system_only` | Platform backend only (encrypted at rest) | Verification challenges, integrity scoring |
| `mutual` | The warga AND the target warga | Social graph — "Bu Sari" can see that Damar claims to know her |
| `public` | Anyone in the community | Community roles and contributions |

---

## 5. How Content and Identity Connect

The power of this model: content and identity share the same concept/place
nodes. This enables cross-referencing queries that would be impossible if
identity were stored separately.

### 5.1 Graph Topology

```
                     concept:Q8068 (flood)
                      ▲            ▲
                      │            │
               ABOUT  │            │  ANCHOR (knowsAbout)
                      │            │
                note:flood_report  warga:damar
                      │
                      ▼
                LOCATED_AT
                      │
                      ▼
                place:rt05_rw03
                      ▲
                      │
               ANCHOR (homeLocation)
                      │
                warga:damar
```

Note and warga connect through the **same** concept and place nodes.
No explicit link between note and warga is needed — the graph connects them.

### 5.2 Cross-Referencing Queries

```sql
-- 1. "Who has local knowledge about flooding?"
SELECT <-ANCHOR<-warga AS knowledgeable_warga
FROM concept:Q8068
WHERE <-ANCHOR[kind = "knowledge"];

-- 2. "Does this reporter's claimed home match their report locations?"
LET $claimed = SELECT ->ANCHOR->place
  FROM warga:suspect
  WHERE predicate = "schema:homeLocation";

LET $reported = SELECT ->LOCATED_AT->*
  FROM note
  WHERE author = warga:suspect;

-- Compare: if $claimed ∩ $reported = ∅ → suspicious

-- 3. "Find all warga who claim to live in RT 05"
SELECT <-ANCHOR<-warga AS residents
FROM place:rt05_rw03
WHERE <-ANCHOR[predicate = "schema:homeLocation"];

-- 4. "Who knows who? (Social graph for RT 05)"
SELECT in AS person, out AS knows_person, detail
FROM ANCHOR
WHERE kind = "social"
  AND in IN (SELECT <-ANCHOR<-warga FROM place:rt05_rw03);

-- 5. "Find reports about topics the reporter claims knowledge of"
--    (Higher credibility — reporter has domain knowledge)
SELECT note.*, anchor.detail AS reporter_knowledge
FROM note
JOIN ANCHOR AS anchor ON anchor.in = note.author
WHERE note->ABOUT->concept IN anchor.out
  AND anchor.kind = "knowledge";
```

---

## 6. Conversational Extraction Flow

Identity anchors are extracted **during** AI-00 triage, not in a separate
step. The AI weaves anchoring questions into natural rapport.

See [04a §6.4](./04a-ai-00-edge-contract.md) for the full prompt strategy,
question categories, and edge-pod response format.

### 6.1 Summary of Extraction Pattern

```
Conversation Turn                           → ANCHOR Edge(s) Created
───────────────────────────────────────────────────────────────────────
AI:   "Kamu tinggal di sekitar mana?"
User: "RT 05 RW 03, Gang Kenari"           → (warga → place:rt05_rw03,
                                               homeLocation, location)

AI:   "Oh dekat Posyandu ya? Sering ke sana?"
User: "Iya, saya kader posyandu"            → (warga → concept:Q2461838,
                                               memberOf, role)

AI:   "Wah hebat! Siapa ketua RT di sana?"
User: "Pak Budi, udah 3 tahun"             → (warga → warga:pak_budi,
                                               knows, social)
                                            + VERIFICATION: if system knows
                                              RT 05 chair, check answer
```

### 6.2 Edge-Pod Response Format

The edge-pod returns extracted anchors alongside the normal triage result:

```jsonc
{
  "result": { /* ... normal TriageResult ... */ },
  "turn_count": 3,

  // Extracted identity anchors — platform persists these
  "identity_anchors": [
    {
      "category": "location",
      "predicate": "schema:homeLocation",
      "question": "Di jalan apa?",
      "answer": "Jl. Mawar gang 3",
      "target_ref": "place:rt05_rw03",    // resolved by AI
      "confidence": 0.9
    },
    {
      "category": "social",
      "predicate": "schema:knows",
      "question": "Pak RT siapa?",
      "answer": "Pak Harto",
      "target_ref": "warga:pak_harto",    // resolved if known
      "confidence": 0.85
    }
  ]
}
```

### 6.3 Platform Persistence

When the platform receives `identity_anchors`, it:

1. For each anchor, creates an `ANCHOR` edge in SurrealDB
2. Sets `visibility` based on `category` defaults (see §4 table)
3. Sets `source` to the current `session_id`
4. If `target_ref` is a known entity, links directly; otherwise creates a
   new `place:*` or `warga:*` node
5. Cross-references with existing anchors for consistency (see §7)

---

## 7. Integrity Scoring (Tandang Integration)

Anchor data feeds the Integrity (I) score in the Tandang trust system.

### 7.1 Scoring Signals

| Signal | Formula | What It Measures |
|--------|---------|-----------------|
| **Anchor density** | `COUNT(->ANCHOR) / sessions_count` | How much identity data per interaction |
| **Anchor consistency** | `consistent_anchors / total_anchors` | Do answers stay the same across sessions? |
| **Location match** | `claimed_locations ∩ report_locations / total_reports` | Do they report from where they claim to live? |
| **Social validation** | `COUNT(bidirectional ANCHOR[knows])` | Do mutual acquaintances confirm each other? |
| **Knowledge accuracy** | `anchors_verified / anchors_checkable` | Are local knowledge claims accurate? |

### 7.2 Consistency Detection

```sql
-- Find all location anchors for a warga, grouped by target
-- Consistent = same place across sessions
-- Inconsistent = different places → flag for review
SELECT
  out AS place,
  count() AS times_claimed,
  array::distinct(source) AS sessions,
  math::mean(confidence) AS avg_confidence
FROM ANCHOR
WHERE in = warga:damar
  AND kind = "location"
  AND predicate = "schema:homeLocation"
GROUP BY out;

-- Flag: user claimed different RT in different sessions
-- (could be legitimate move, or could be impersonation)
```

### 7.3 Cross-Validation

```sql
-- "Does Bu Sari also claim to know Damar?"
-- Bidirectional ANCHOR edges = strong social validation
SELECT *
FROM ANCHOR
WHERE in = warga:sari
  AND out = warga:damar
  AND kind = "social";
-- If exists → both claim to know each other → strong signal
-- If missing → one-sided claim → weaker signal
```

---

## 8. Privacy & Security

### 8.1 Data Classification

| Data | Classification | Storage |
|------|---------------|---------|
| `system_only` anchors | PII-adjacent | Encrypted at rest, excluded from analytics |
| `mutual` anchors | Semi-private | Visible only to the two parties |
| `public` anchors | Public | Visible to community members |
| All anchors | User-deletable | GDPR-like right to deletion on request |

### 8.2 Security Rules

1. **Anchors are never exposed in API responses** to other users
2. **Challenge questions** use anchors but never reveal the "correct" answer
3. **Anchor deletion** cascades — deleting a warga deletes all their anchors
4. **Export exclusion** — anchors are excluded from any data export or backup
   shared with third parties
5. **Encryption key** — per-community, rotated quarterly

### 8.3 Retention

| Anchor Kind | Retention | Rationale |
|-------------|-----------|-----------|
| `location` | Until user updates or deletes | People move |
| `social` | Until either party deletes | Relationships change |
| `role` | Until user updates | Roles rotate |
| `knowledge` | Permanent (unless deleted) | Historical knowledge doesn't expire |
| `personal` | Until user deletes | User controls personal data |

---

## 9. Updated Edge Type Table

This is the complete edge type inventory for the ontology graph,
including the new `ANCHOR` edge:

| Edge | From | To | Purpose |
|------|------|----|---------|
| `ABOUT` | note/plan/siaga | concept:QID | Content → concept (with predicate + object metadata) |
| `LOCATED_AT` | note/plan/siaga | concept:QID or place:* | Content → location (with OSM tags) |
| `HAS_ACTION` | note/plan/siaga | action:* | Content → intent (Schema.org Action type) |
| `BROADER` | concept:QID | concept:QID | Wikidata hierarchy (P279 subclass of) |
| `INSTANCE_OF` | place:* | concept:QID | Local entity → Wikidata type (P31) |
| `VOUCHES` | warga:* | note:* | Trust signal |
| `CHALLENGES` | warga:* | note:* | Dispute signal |
| **`ANCHOR`** | **warga:*** | **concept/place/warga** | **Identity fact (Schema.org predicate on edge)** |

---

## 10. Relationship to Other Specs

| Spec | Connection |
|------|------------|
| [ONTOLOGY-VOCAB-v0.1.md](../ONTOLOGY-VOCAB-v0.1.md) §4.1 | `ANCHOR` added to Edge Types table |
| [04a — Edge Contract §6.4](./04a-ai-00-edge-contract.md) | How AI extracts anchors during triage conversation |
| [03 — Tandang](./03-tandang-handled-ai.md) | Integrity (I) score consumes anchor density + consistency |
| [04b — Trajectory Map](./04b-trajectory-map.md) | Anchors are extracted during all trajectory triage flows |
| [04c — Operator/Skill Map](./04c-operator-skill-map.md) | Identity extraction is a cross-cutting skill |

---

## 11. Design Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | One edge type (`ANCHOR`) for all identity facts | Same pattern as content triples; simpler than 5 separate edge types |
| D2 | Schema.org predicates on ANCHOR edges | Reuses existing vocabulary; no custom predicates needed |
| D3 | `kind` field for categorization | Enables per-category privacy defaults and scoring weights |
| D4 | `visibility` on edge, not on node | Same anchor target can have different visibility from different warga |
| D5 | `source` tracks provenance | Every fact traces back to the conversation that revealed it |
| D6 | No separate identity database | Identity and content share the same graph; cross-referencing is free |
| D7 | Progressive enrichment | Each session adds 1–2 anchors; no upfront "profile setup" required |
| D8 | No OWL/SKOS/SPARQL dependency | SurrealDB v3 has no RDF support; LLM is our reasoner (see SurrealDB v3 research) |

---

## 12. Open Questions

| # | Question | Notes |
|---|----------|-------|
| OQ-1 | Minimum anchor density before I score is meaningful? | Need threshold — e.g., at least 3 anchors across 2+ sessions |
| OQ-2 | How to handle legitimate contradictions (user moved)? | Probably: new anchor supersedes old; old archived, not deleted |
| OQ-3 | Should social anchors require mutual confirmation? | Unilateral "I know Bu Sari" vs bilateral both claim to know each other |
| OQ-4 | Anchor extraction prompt — exact wording per trajectory? | Different trajectories may naturally invite different anchor questions |

---

*Document created: 2026-02-22*
*Companion to: ONTOLOGY-VOCAB-v0.1.md, 04a-ai-00-edge-contract.md*

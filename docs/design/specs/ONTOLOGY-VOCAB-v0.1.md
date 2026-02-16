# Gotong Royong — Ontology & Vocabulary Convention v0.1

## Status
Proposed: 2026-02-16 | Revised: 2026-02-16
Purpose: Define the vocabulary conventions that constrain LLM concept classification across all four modes, using pure RDF-style triples built on standards LLMs already know natively.

> **Key decision (S3-MD1):** No custom vocabulary. Every position in the triple is constrained by its source standard. The constraint IS the standard.

---

## 1. Design Principle: Don't Invent — Reuse What LLMs Already Know

Custom taxonomies are expensive: they must be loaded into every prompt, maintained by humans, and taught to the model. Instead, we express all knowledge as **RDF-style triples** using three open standards that LLMs are saturated with from pre-training:

| Position in Triple | Constrained By | Why LLMs Know It |
|---|---|---|
| **Subject** | Wikidata QID | Wikipedia/Wikidata is core training data. LLMs resolve "telur" → Q93189 natively. Language-agnostic. |
| **Predicate** | Schema.org property | Every website uses Schema.org for SEO structured data. Billions of pages. LLMs know `schema:price`, `schema:about`, `schema:potentialAction`. |
| **Object** | Wikidata QID or literal value | Same as subject. Literals for measurements (28000 IDR), dates, counts. |
| **Location** (optional) | OpenStreetMap tags | OSM is massively represented in training data. LLMs know `amenity=marketplace`. |

**The LLM does not need to read our vocabulary.** It already speaks these languages. We just tell it which languages to speak.

### 1.1 What We Don't Have (and Why)

| Removed Concept | Why | What Replaces It |
|---|---|---|
| Custom Ranah (8 domain categories) | Artificially constraining. LLM has to learn our mapping. | Wikidata hierarchy — domain is derived at display time via P279 (subclass of) traversal. |
| Custom Tujuan (intent categories) | Overlapped with track hints and mode routing. | Schema.org Action types — `RepairAction`, `InformAction`, etc. |
| Separate track hint classification | Duplicate work alongside ontology classification. | Schema.org Action type in the triple IS the routing signal. |
| Flat hashtags | No structure, no hierarchy, no deduplication. | Wikidata QIDs as structured concept tags. |

---

## 2. The Triple Model

Every piece of content that enters the system gets classified as **2–5 RDF-style triples**. Each triple is `(subject, predicate, object)`:

```
User says: "Telur Rp 28.000 di Pasar Minggu"

Triple 1: (Q93189 [egg],         schema:price,            "28000 IDR/kg")
Triple 2: (Q93189 [egg],         schema:contentLocation,  Q132510 [market])
Triple 3: (Q93189 [egg],         schema:potentialAction,   schema:InformAction)
```

Each triple is a fact. The LLM extracts facts from natural language and expresses them in this format. No custom vocabulary needed — every element comes from Schema.org or Wikidata.

### 2.1 Positions Explained

**Subject — Wikidata QID**
The primary concept. What is this about?

| User input | Subject QID | Label |
|---|---|---|
| "Telur 28k" | Q93189 | egg |
| "Jalan Merdeka berlubang" | Q34442 | road |
| "Ada kasus DBD di RT 03" | Q30953 | dengue fever |
| "Kerja bakti Minggu depan" | Q5586288 | gotong royong |
| "PDAM mati sejak pagi" | Q181937 | water supply |

**Predicate — Schema.org Property**
The relationship or attribute type. What aspect of the subject are we describing?

| Predicate | Meaning | Example |
|---|---|---|
| `schema:price` | Price observation | Egg costs 28k |
| `schema:about` | General topic relation | Note is about flooding |
| `schema:contentLocation` | Where something is | At Pasar Minggu |
| `schema:potentialAction` | Intent / what should happen | Repair, inform, celebrate |
| `schema:status` | Current state | Road closure, power outage |
| `schema:result` | Outcome | Achievement, completion |
| `schema:startDate` / `schema:endDate` | Temporal bounds | Event schedule |
| `schema:description` | Descriptive detail | Condition detail |

**Object — Wikidata QID or Literal**
The value. What is being said about the subject?

- Another Wikidata QID when referencing a concept: `Q132510` (market)
- A Schema.org Action type when expressing intent: `schema:RepairAction`
- A literal value when expressing a measurement: `"28000 IDR/kg"`
- A date/time for temporal facts: `"2026-02-23T08:00"`

### 2.2 Mode Routing via Schema.org Action Types

The `schema:potentialAction` predicate carries the intent signal. This replaces both track hints and mode routing:

| Action Type | Maps To | Old Track Hint |
|---|---|---|
| `schema:InformAction` | → Catatan Komunitas | (new — informational) |
| `schema:RepairAction` | → Komunitas | Tuntaskan |
| `schema:CreateAction` | → Komunitas | Wujudkan |
| `schema:SearchAction` | → Komunitas | Telusuri |
| `schema:AchieveAction` | → Komunitas | Rayakan |
| `schema:AssessAction` | → Komunitas | Musyawarah |
| `schema:AlertAction` | → Siaga | (emergency) |

Catatan Saksi routing is user-initiated ("Saya ingin mencatat untuk diri sendiri") — not derived from Action type.

AI-00 produces the triples including the Action type. Mode routing reads the Action type triple to determine destination. Track hint labels (Tuntaskan, Wujudkan, etc.) remain as **UI display labels** only — they are derived from the Action type, not separately classified.

### 2.3 Domain (Ranah) Derived from Wikidata Hierarchy

No separate domain classification. Domain is computed at display time by traversing Wikidata's own hierarchy:

```
Q93189 (egg) → P279 → Q2095 (food) → P279 → Q11004 (nutrition)
  → Display domain: "Ekonomi" (because food/prices = economic)

Q8068 (flood) → P279 → Q8065 (natural disaster) → P279 → Q3839081 (disaster)
  → Display domain: "Lingkungan" (because natural disaster = environmental)

Q34442 (road) → P279 → Q83620 (thoroughfare) → P279 → Q2063507 (infrastructure)
  → Display domain: "Infrastruktur"
```

The mapping from Wikidata high-level concepts to display domain labels is a thin UI-layer lookup table, not a classification step. It's used only for feed filters and dashboard grouping.

---

## 3. LLM Classification Output Schema

When content enters the system (via AI-00 triage), the LLM outputs triples plus metadata:

```json
{
  "triples": [
    {
      "subject": { "qid": "Q93189", "label": "telur" },
      "predicate": "schema:price",
      "object": { "value": 28000, "unit": "IDR", "per": "kg" }
    },
    {
      "subject": { "qid": "Q93189", "label": "telur" },
      "predicate": "schema:contentLocation",
      "object": { "qid": "Q132510", "label": "pasar" }
    },
    {
      "subject": { "qid": "Q93189", "label": "telur" },
      "predicate": "schema:potentialAction",
      "object": { "action": "schema:InformAction" }
    }
  ],
  "osm_tags": ["amenity=marketplace"],
  "temporal_class": "ephemeral",
  "confidence": 0.92
}
```

### 3.1 Field Rules

| Field | Required | Constraint | Notes |
|---|---|---|---|
| `triples` | Yes, 2–5 | Each position constrained by its source standard | At minimum: one `schema:about` or typed predicate + one `schema:potentialAction` |
| `triples[].subject.qid` | Yes | Valid Wikidata QID | LLM resolves from its training knowledge |
| `triples[].predicate` | Yes | Valid Schema.org property | From Schema.org vocabulary including extensions |
| `triples[].object` | Yes | Wikidata QID, Schema.org type, or literal | Depends on predicate semantics |
| `osm_tags` | If location present | Valid OSM tag strings | Compound tags allowed as array: `["amenity=marketplace", "wholesale=yes"]` |
| `temporal_class` | Yes | One of 4 values | Drives TTL default (see §3.2) |
| `confidence` | Yes | 0.0–1.0 | Overall classification confidence |

### 3.2 Temporal Classes

| Class | Meaning | Default TTL | Examples |
|---|---|---|---|
| `ephemeral` | Changes within hours/day | 24h | Prices, traffic, weather, power outage |
| `periodic` | Recurs on schedule | Until next occurrence | Posyandu schedule, weekly market, monthly meeting |
| `durable` | Lasts days to weeks | 7–30 days | Road closure, construction, policy change |
| `permanent` | Persistent fact | No expiry | Facility exists, achievement, infrastructure built |

`temporal_class` is derived from the triple content — `schema:price` implies ephemeral, `schema:startDate` with recurrence implies periodic. AI-00 can suggest an override. Author has final say within bounds (S3-A3).

---

## 4. SurrealDB Graph Storage

Triples map directly to SurrealDB's native graph edges:

```sql
-- Content node
CREATE note:abc SET
  content = "Telur Rp 28k di Pasar Minggu",
  author = warga:damar,
  created_at = time::now(),
  temporal_class = "ephemeral",
  ttl_expires = time::now() + 24h,
  ai_readable = true,
  rahasia_level = 0,
  confidence = 0.92;

-- Triple 1: subject-about relation
RELATE note:abc -> ABOUT -> concept:Q93189
  SET predicate = "schema:price",
      object_value = 28000,
      object_unit = "IDR",
      object_per = "kg",
      role = "subject";

-- Triple 2: location relation
RELATE note:abc -> LOCATED_AT -> concept:Q132510
  SET predicate = "schema:contentLocation",
      osm_tags = ["amenity=marketplace"];

-- Triple 3: action/intent relation
RELATE note:abc -> HAS_ACTION -> action:InformAction
  SET predicate = "schema:potentialAction";

-- Wikidata hierarchy (pre-seeded or lazily populated)
RELATE concept:Q93189 -> BROADER -> concept:Q2095;   -- egg → food
RELATE concept:Q2095 -> BROADER -> concept:Q11004;    -- food → nutrition
RELATE concept:Q132510 -> BROADER -> concept:Q3914;   -- market → economy

-- Numeric measurement (time-series capable)
CREATE measurement:abc_price SET
  note = note:abc,
  concept = concept:Q93189,
  location = concept:Q132510,
  predicate = "schema:price",
  value = 28000,
  unit = "IDR",
  per = "kg",
  observed_at = time::now();

-- Vouch edge
RELATE warga:sari -> VOUCHES -> note:abc
  SET vouched_at = time::now();
```

### 4.1 Edge Types

| Edge | From | To | Purpose |
|---|---|---|---|
| `ABOUT` | note/plan/siaga | concept:QID | Subject relation (with predicate + object metadata on edge) |
| `LOCATED_AT` | note/plan/siaga | concept:QID or place:* | Spatial relation (with OSM tags on edge) |
| `HAS_ACTION` | note/plan/siaga | action:* | Intent/routing (Schema.org Action type) |
| `BROADER` | concept:QID | concept:QID | Wikidata hierarchy (P279 subclass of) |
| `INSTANCE_OF` | place:* | concept:QID | Local entity → Wikidata type (P31) |
| `VOUCHES` | warga:* | note:* | Trust signal |
| `CHALLENGES` | warga:* | note:* | Dispute signal |

### 4.2 Cross-Mode Connectivity

The ontology is shared across ALL four modes. A Catatan Komunitas note about "banjir" and a Komunitas plan about "banjir" both link to `concept:Q8068`:

```sql
-- Community note
RELATE note:flood_report -> ABOUT -> concept:Q8068
  SET predicate = "schema:about";

-- Komunitas plan (adaptive path)
RELATE plan:fix_drainage -> ABOUT -> concept:Q8068
  SET predicate = "schema:about";

-- Siaga broadcast
RELATE siaga:flood_alert -> ABOUT -> concept:Q8068
  SET predicate = "schema:about";

-- Find everything about flooding across all modes
SELECT * FROM (
  SELECT <-ABOUT<-note AS notes,
         <-ABOUT<-plan AS plans,
         <-ABOUT<-siaga AS alerts
  FROM concept:Q8068
);
```

One concept node connects all four modes without any manual linking.

### 4.3 Hierarchy Queries

> **Probe finding (2026-02-16):** Forward filtering (`WHERE ->ABOUT->concept->BROADER CONTAINS ...`) does NOT work in SurrealDB v3 beta — graph traversal returns string representations, not record references. **Use reverse walk from parent concept instead.** See `docs/research/ontology-probe-report.md` P3c.

```sql
-- Find all notes about ANY type of food (not just eggs)
-- PATTERN: Walk backwards from parent concept through hierarchy
SELECT <-BROADER<-concept<-ABOUT<-note.content AS notes FROM concept:Q2095;
-- Returns: ["Telur Rp 28.000 di Pasar Minggu", "Beras naik jadi Rp 15.000/kg"]

-- Find all infrastructure-related content across all modes
SELECT
  <-BROADER<-concept<-ABOUT<-note AS notes,
  <-BROADER<-concept<-ABOUT<-plan AS plans,
  <-BROADER<-concept<-ABOUT<-siaga AS alerts
FROM concept:Q2063507;

-- Deeper hierarchy (2 levels): nutrition → food → [egg, rice, chili] → notes
SELECT <-BROADER<-concept<-BROADER<-concept<-ABOUT<-note.content AS notes
FROM concept:Q11004;
```

### 4.4 Time-Series Queries

```sql
-- Price of eggs over last 30 days, grouped by location
SELECT
  location,
  time::group(observed_at, 'day') AS day,
  math::mean(value) AS avg_price,
  count() AS report_count
FROM measurement
WHERE concept = concept:Q93189
  AND observed_at > time::now() - 30d
GROUP BY location, day
ORDER BY day DESC;

-- Detect price spike: current vs 7-day average
SELECT concept, location,
  math::mean(value) AS current_avg,
  (SELECT math::mean(value) FROM measurement
   WHERE concept = $parent.concept
     AND observed_at > time::now() - 7d) AS week_avg
FROM measurement
WHERE observed_at > time::now() - 1d
GROUP BY concept, location
HAVING current_avg > week_avg * 1.2;  -- 20% spike threshold
```

---

## 5. Concept Node Lifecycle

### 5.1 Pre-Seeded vs Lazily Created

**Pre-seeded concepts (~200):** Batch imported at deploy from Wikidata. Most common community concepts:

| Category | Example QIDs |
|---|---|
| Staple foods | Q93189 (egg), Q36465 (rice), Q165199 (chili), Q11030 (cooking oil) |
| Infrastructure | Q34442 (road), Q12277 (bridge), Q12323 (drainage), Q210932 (street light) |
| Facilities | Q2461838 (posyandu), Q32815 (mosque), Q9842 (school) |
| Hazards | Q8068 (flood), Q3196 (fire), Q7944 (earthquake), Q190977 (landslide) |
| Services | Q4830453 (electricity supply), Q181937 (water supply), Q7362 (waste management) |
| Health | Q30953 (dengue), Q84 (COVID-19), Q12135 (diarrhea) |
| Governance | Q7188 (election), Q1437459 (community meeting), Q182985 (budget) |

Pre-seeded with Wikidata hierarchy (3 levels of BROADER, per S3-D2) and both `label_id` + `label_en`.

**Lazily created:** When the LLM references a QID not yet in the graph:

```sql
INSERT INTO concept {
  id: concept:Q12345,
  qid: "Q12345",
  label_id: "sesuatu",      -- provided by LLM at classification time
  label_en: "something",    -- provided by LLM at classification time
  created_at: time::now(),
  source: "llm_classification",
  verified: false
} ON DUPLICATE KEY UPDATE last_referenced = time::now();
```

### 5.2 Hierarchy Population

Wikidata hierarchy (`BROADER` edges) populated via three methods:

1. **Pre-loaded** for seeded concepts (batch import from Wikidata at deploy, 3 levels up)
2. **LLM-inferred** for new concepts (LLM knows "egg is a type of food" — fast, inline)
3. **Background verification** via browsing-capable LLM tier (S3-MD7) — validates and enriches lazily created concepts against live Wikidata

Priority: method 2 for speed at classification time, method 3 for accuracy as background job.

### 5.3 Label Storage (S3-D5)

Labels stored locally, updated incrementally:
- Batch import ~200 common concepts at deploy time
- LLM provides `label_id` and `label_en` at classification time for new concepts
- Browsing-capable LLM tier periodically verifies and refreshes labels from Wikidata API
- Local-first, eventually consistent — no API call at display time

### 5.4 Concepts Never Deleted

Concept nodes are never removed from the graph. Inactive concepts:
- Their notes expire via TTL
- They have no incoming `ABOUT` edges from active content
- They remain discoverable for historical queries
- Zero maintenance burden (just a node with no active edges)

---

## 6. Synonym & Normalization

The LLM handles normalization at classification time. No synonym table needed.

| Challenge | How the LLM Handles It | Why It Works |
|---|---|---|
| **Bahasa ↔ English** | "egg" and "telur" both → Q93189 | LLMs are multilingual; Wikidata QIDs are language-agnostic |
| **Local dialect** | "endog" (Javanese for egg) → Q93189 | LLMs trained on Indonesian regional languages |
| **Abbreviation** | "PDAM" → Q181937 (water supply) | LLMs know Indonesian acronyms |
| **Typo** | "telor" → Q93189 | LLMs handle spelling variation |
| **Code-switching** | "Harga rice naik" → Q36465 (rice) | LLMs handle mixed-language input |

---

## 7. Hyper-Local Concepts (No Wikidata Entry)

Some things don't exist in Wikidata: "Posyandu RT 05", "Warung Bu Sari", "Gang Kenari".

These are **instances**, not concepts. Stored as local entities with `INSTANCE_OF` linking to a Wikidata parent concept:

```sql
CREATE place:posyandu_rt05 SET
  name = "Posyandu RT 05",
  osm_tags = ["amenity=clinic"],
  location = geo::point(-6.26, 106.85),
  source = "community";

RELATE place:posyandu_rt05 -> INSTANCE_OF -> concept:Q2461838;  -- posyandu

-- Notes can reference both the local place and the concept type
RELATE note:xyz -> LOCATED_AT -> place:posyandu_rt05
  SET predicate = "schema:contentLocation";
RELATE note:xyz -> ABOUT -> concept:Q2461838
  SET predicate = "schema:about";
```

The LLM outputs: a QID for the concept type + a local entity name. The system creates the local place node linked to the Wikidata concept type via `INSTANCE_OF`.

---

## 8. Robustness & Edge Cases

### 8.1 QID Hallucination

Wikidata QIDs are verifiable. The browsing-capable LLM tier (S3-MD7) validates:

```sql
-- Flag unverified concepts
SELECT * FROM concept WHERE verified = false AND created_at < time::now() - 1h;
```

Verification: browsing-capable LLM accesses `https://www.wikidata.org/wiki/Special:EntityData/Q{id}.json`. If invalid, concept flagged for review. Verification cadence is configurable (S3-D1) — default: hourly for new concepts, daily sweep for all.

**Mitigation in prompt:** "Hanya gunakan QID yang Anda yakin benar. Jika ragu, gunakan QID induk yang pasti benar dan set confidence rendah."

### 8.2 Concept Drift

Wikidata QIDs are stable (immutable identifiers). Schema.org properties are stable (backward compatible by design). OSM tags are stable (governed by community consensus). If Wikidata adds a more specific QID later, existing references to the parent QID remain valid. The system can optionally reclassify historical data via background job.

### 8.3 LLM Model Swap

Because we use standards the LLM learned from training data — not custom vocabulary — the classification works across different models (Claude, GPT, Llama, Mistral, Gemini). Any model trained on web data knows Schema.org, Wikidata, and OSM. The system is model-agnostic.

### 8.4 Schema.org Extensions (S3-D3)

We use Schema.org extensions (e.g., `GovernmentService`, `MedicalCondition`) — not just core types. The browsing-capable LLM tier can verify extensions exist programmatically.

### 8.5 OSM Compound Tags (S3-D4)

Compound tags are allowed: `["amenity=marketplace", "wholesale=yes"]`. SurrealDB indexes arrays natively, so compound tags are searchable without performance cost.

---

## 9. Two LLM Tiers (S3-MD7)

| Tier | Model Class | Purpose | Internet | Latency |
|---|---|---|---|---|
| **Fast** | Haiku-class | Real-time UX: triage, classification, inline suggestions, triple generation | No | Low (<2s) |
| **Capable** | Sonnet-class with tools | Background: QID verification, label refresh, enrichment, pattern detection, Wikidata validation | Yes (browsing) | Tolerant |

The fast tier produces triples during AI-00 triage. The capable tier verifies and enriches them afterward.

---

## 10. System Dynamics & Analytics

The triple-based graph + time-series data enables analytics without any custom analytics schema:

### 10.1 Topic Velocity

```sql
-- Which concepts are trending this week vs last week?
SELECT concept,
  count(SELECT * FROM note WHERE ->ABOUT CONTAINS $parent.concept
    AND created_at > time::now() - 7d) AS this_week,
  count(SELECT * FROM note WHERE ->ABOUT CONTAINS $parent.concept
    AND created_at > time::now() - 14d
    AND created_at < time::now() - 7d) AS last_week
FROM concept
HAVING this_week > last_week * 1.5
ORDER BY (this_week - last_week) DESC
LIMIT 10;
```

### 10.2 Note-to-Action Conversion Rate

```sql
-- How often do notes about a topic lead to Komunitas plans?
SELECT
  concept.label_id AS topic,
  count(<-ABOUT<-note) AS note_count,
  count(<-ABOUT<-plan) AS plan_count,
  (plan_count / note_count * 100) AS conversion_pct
FROM concept
WHERE count(<-ABOUT<-note) > 5
ORDER BY conversion_pct DESC;
```

### 10.3 Community Health Index

```sql
-- Per-community activity breakdown by concept hierarchy
SELECT
  community_id,
  concept->BROADER->concept.label_id AS domain,
  count() AS activity_count,
  count(WHERE ->HAS_ACTION->action = action:RepairAction) AS repair_items,
  count(WHERE temporal_class = "ephemeral") AS ephemeral_reports
FROM note
WHERE created_at > time::now() - 30d
GROUP BY community_id, domain;
```

### 10.4 Cross-Mode Discovery

```sql
-- "Show me everything the community knows about flooding"
SELECT
  "notes" AS source, content, created_at, author
FROM note WHERE ->ABOUT CONTAINS concept:Q8068
UNION ALL
SELECT
  "plans" AS source, title AS content, created_at, author
FROM plan WHERE ->ABOUT CONTAINS concept:Q8068
UNION ALL
SELECT
  "siaga" AS source, message AS content, created_at, author
FROM siaga WHERE ->ABOUT CONTAINS concept:Q8068
ORDER BY created_at DESC;
```

---

## 11. Integration with AI Touchpoints

| AI Touchpoint | How Triples Integrate |
|---|---|
| **AI-00 (Triage)** | Produces triples as part of triage. `schema:potentialAction` triple determines mode routing. |
| **AI-01 (Classification)** | Can refine/validate triples produced by AI-00. Adds additional triples if needed. |
| **AI-03 (Duplicate Detection)** | Vector similarity (existing) + QID overlap (new). Two notes with same subject QIDs are likely duplicates. |
| **AI-04 (Moderation)** | Reads `ai_readable` notes for pattern detection. AI aggregates anonymous reports — no public tagging (S3-MD5). |
| **AI-09 (Credit)** | Uses Action type and concept hierarchy for mode-specific credit rules. |
| **Catatan Komunitas** | Triples replace flat tags. Users see human-readable labels; system stores QIDs + predicates. |
| **TTL assignment** | `temporal_class` (derived from triple predicates) sets default TTL. Author can override (S3-A3). |
| **Promotion bridge** | QID overlap between notes and plans shows "this topic already has a plan." |

---

## 12. Prompt Integration (Minimal)

The full prompt addition for triple-based classification is tiny — the LLM already knows these standards:

```
When classifying user input, express the content as 2-5 RDF-style triples:
- Subject: Wikidata QID (resolve the main concept)
- Predicate: Schema.org property (schema:price, schema:about, schema:contentLocation, schema:potentialAction, etc.)
- Object: Wikidata QID, Schema.org Action type, or literal value

Also provide:
- osm_tags: OpenStreetMap tags for any physical locations
- temporal_class: ephemeral / periodic / durable / permanent
- confidence: 0.0-1.0

Always resolve to canonical Wikidata QIDs. Do not invent QIDs. If unsure, use the nearest parent QID with lower confidence.

For potentialAction, use Schema.org Action types: InformAction, RepairAction, CreateAction, SearchAction, AchieveAction, AssessAction, AlertAction.
```

~12 lines of prompt. The LLM does the rest from its training knowledge.

---

## 13. What This Replaces

| Old approach | New approach |
|---|---|
| 4-layer stack (OSM → Wikidata → Schema.org → Ranah) with custom vocabulary | Pure RDF triples. No custom vocabulary. Each position constrained by its source standard. |
| 8 custom Ranah values (infra, ekon, sosial, etc.) as classification output | Domain derived from Wikidata hierarchy at display time. Thin UI lookup, not LLM task. |
| Custom Tujuan values for mode routing | Mode routing from Schema.org Action types (`schema:potentialAction` triple). |
| Separate track hint classification | Track hint labels derived from Action type. Display-only. |
| AI-01 auto-tags with custom labels | AI-00 produces Wikidata QID triples. AI-01 refines. |
| Flat hashtags on community notes | Wikidata QIDs as structured concept tags (rendered as human-readable labels). |
| Custom topic clustering | Graph traversal via BROADER edges on Wikidata hierarchy. |
| TTL hardcoded per "topic type" | TTL derived from `temporal_class` (inferred from triple predicates). |
| No cross-mode discovery | Shared QID-based concept graph connects all four modes. |

---

## 14. Open Questions

All previous open questions (§13 in v0.1 original) have been resolved:

| # | Question | Resolution | Decision |
|---|---|---|---|
| 1 | QID verification cadence? | Configurable. Default: hourly new, daily all. | S3-D1 |
| 2 | Hierarchy depth? | 3 levels pre-loaded. Deeper via traversal. | S3-D2 |
| 3 | Schema.org extensions? | Use them. Verify programmatically. | S3-D3 |
| 4 | OSM tag granularity? | Allow compound tags as arrays. | S3-D4 |
| 5 | Wikidata label storage? | Store locally, update incrementally. | S3-D5 |

**Remaining open (tracked in DECISIONS-LOG.md):**
- OPEN-12: SurrealDB DDL schema and migration strategy

---

## 15. Relationship to Other Specs

| Spec | Connection |
|---|---|
| DECISIONS-LOG.md | Decisions S3-MD1, S3-MD2, S3-A1–A3, S3-D1–D5 drive this spec. |
| ENTRY-PATH-MATRIX-v0.1.md | Section 4.3 (tags) → Wikidata QID triples. TTL → temporal_class. Mode routing → Action types. |
| ADAPTIVE-PATH-SPEC-v0.1.md | `track_hint` derived from Action type in triple. `seed_hint` remains. |
| AI-SPEC-v0.2.md (AI-00, AI-01, AI-03) | Classification output schema = triples. AI-03 duplicate detection uses QID overlap. |
| ADAPTIVE-PATH-ORCHESTRATION-v0.1.md | Cross-case learning queries use QID-based graph traversal. |
| UI-UX-SPEC-v0.5.md | Feed filters use domain derived from Wikidata hierarchy. Progressive disclosure (S3-B4). |

---

*Document revised: 2026-02-16*
*Supersedes: Original 4-layer stack approach (see §13 "What This Replaces")*
*Companion to: DECISIONS-LOG.md, ENTRY-PATH-MATRIX-v0.1.md, AI-SPEC-v0.2.md*

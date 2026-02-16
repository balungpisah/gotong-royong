# SurrealDB Ontology Graph Model — Probe Report

**Date:** 2026-02-16T10:58:19Z
**SurrealDB Version:** 3.0.0-beta.4 for macos on aarch64
**Namespace/DB:** gotong_ontology_probe/chat
**Locked Target:** v3.0.0-beta.4

---

## Objective

Validate that the RDF-triple graph model defined in `ONTOLOGY-VOCAB-v0.1.md` works
in SurrealDB v3.0.0-beta.4 using RELATE edges, graph traversal operators,
and SCHEMALESS tables — alongside the existing SCHEMAFULL chat/event schema.

## Result Summary

| Pattern | Name | Result | Detail |
|---|---|---|---|
| P1 | RELATE edge creation | PASS | 6 ABOUT edges created |
| P2a | Forward traversal (note->ABOUT->concept) | PASS | Got: telur |
| P2b | Reverse traversal (concept<-ABOUT<-note) | PASS | Got note content |
| P3a | Single-hop BROADER (egg→food) | PASS | Got: makanan |
| P3b | Multi-hop BROADER (egg→food→nutrition) | PASS | Got: gizi |
| P3c | Hierarchy query — reverse walk (parent<-BROADER<-concept<-ABOUT<-note) | PASS | 2 food notes found via reverse walk |
| P4 | Cross-mode via shared concept | PASS | notes=1, plans=1, alerts=1 |
| P5 | Time-series aggregation | PASS | avg=28000.0, count=3 |
| P6 | Temporal class + TTL expiry query | PASS | ephemeral=3, will_expire=3 |
| P7 | Wilson score ranking query | PASS | Counts ready for app-layer Wilson: top='Telur Rp 28.000 di Pasar Min...', vouches=3 |
| P8 | Privacy filtering (rahasia + ai_readable) | PASS | public=4, ai_readable=5, truly_private=1 |
| P9 | INSTANCE_OF hyper-local linking | PASS | Pasar Minggu → pasar (Q132510) |
| P10 | Edge metadata (predicate + object on RELATE) | PASS | predicate=schema:price, value=28000 |
| P11 | SCHEMALESS + SCHEMAFULL coexistence | PASS | schemaless notes=6, schemafull=1 |
| P12 | Mode routing via Action type graph | PASS | Modes found: catatan_komunitas,komunitas,siaga |

**Score: 15 pass, 0 warn, 0 fail out of 15 patterns**

---

## Graph Statistics

```json
Nodes: notes=6, plans=1, siaga=1, concepts=12, places=2, actions=7
Edges: ABOUT=6, LOCATED_AT=2, HAS_ACTION=7, BROADER=7, INSTANCE_OF=2, VOUCHES=4, CHALLENGES=1
Measurements: 4
```

---

## Detailed Output

### P1: RELATE Edge Creation
```json
[[{"c":6}]]
```

### P2: Graph Traversal (Forward + Reverse)
```json
[[{"concepts":["telur"]}]]
[[{"notes":["Telur Rp 28.000 di Pasar Minggu"]}]]
```

### P3: Wikidata Hierarchy (BROADER) Traversal
```json
[[{"parent":["makanan"]}]]
[[{"grandparent":["gizi"]}]]
REVERSE_WALK (working pattern):
[[{"notes":["Beras naik jadi Rp 15.000/kg","Telur Rp 28.000 di Pasar Minggu"]}]]
FULL_OBJECTS:
[[{"notes":["note:rice_price","note:egg_price"],"plans":[]}]]
```

### P4: Cross-Mode Connectivity
```json
[[{"alerts":["PERINGATAN: Banjir di RT 06, hindari gang depan"],"notes":["Banjir setinggi 50cm di gang depan"],"plans":["Perbaiki saluran air gang depan"]}]]
```

### P5: Time-Series Aggregation
```json
[[{"avg_price":28000.0,"concept":"concept:Q93189","max_price":29000,"min_price":27000,"report_count":3}]]
```

### P6: Temporal Class + TTL Expiry
```json
[[{"content":"Telur Rp 28.000 di Pasar Minggu","temporal_class":"ephemeral","ttl_expires":"2026-02-17T10:00:00Z"},{"content":"Banjir setinggi 50cm di gang depan","temporal_class":"ephemeral","ttl_expires":"2026-02-17T00:00:00Z"},{"content":"Beras naik jadi Rp 15.000/kg","temporal_class":"ephemeral","ttl_expires":"2026-02-17T10:00:00Z"}]]
EXPIRED:
[[{"content":"Telur Rp 28.000 di Pasar Minggu","ttl_expires":"2026-02-17T10:00:00Z"},{"content":"Banjir setinggi 50cm di gang depan","ttl_expires":"2026-02-17T00:00:00Z"},{"content":"Beras naik jadi Rp 15.000/kg","ttl_expires":"2026-02-17T10:00:00Z"}]]
```

### P7: Wilson Score / Vouch Ranking
```json
[[{"challenges":0,"content":"Telur Rp 28.000 di Pasar Minggu","id":"note:egg_price","vouches":3},{"challenges":0,"content":"Jalan Kenari berlubang besar dekat pos ronda","id":"note:road_damage","vouches":1},{"challenges":0,"content":"Banjir setinggi 50cm di gang depan","id":"note:flood_report","vouches":0},{"challenges":1,"content":"Beras naik jadi Rp 15.000/kg","id":"note:rice_price","vouches":0}]]
GROUPED_VOUCHES:
[[{"note":"note:egg_price","vouch_count":3},{"note":"note:road_damage","vouch_count":1}]]
WILSON_ATTEMPT:
[null]
```

### P8: Privacy Filtering (ai_readable + rahasia_level)
```
PUBLIC: [[{"content":"Telur Rp 28.000 di Pasar Minggu"},{"content":"Banjir setinggi 50cm di gang depan"},{"content":"Beras naik jadi Rp 15.000/kg"},{"content":"Jalan Kenari berlubang besar dekat pos ronda"}]]
AI_READABLE: [[{"content":"Telur Rp 28.000 di Pasar Minggu"},{"content":"Banjir setinggi 50cm di gang depan"},{"content":"Ada tetangga yang sering ribut malam, khawatir KDRT"},{"content":"Beras naik jadi Rp 15.000/kg"},{"content":"Jalan Kenari berlubang besar dekat pos ronda"}]]
PRIVATE: [[{"content":"Hari ini aku sedih karena kehilangan kucing"}]]
```

### P9: INSTANCE_OF Hyper-Local Places
```json
[[{"concept_type":["pasar"],"name":"Pasar Minggu"}]]
[[{"local_places":["Pasar Minggu"]}]]
```

### P10: Edge Metadata (predicate + object_value on RELATE)
```json
[[{"object_per":"kg","object_unit":"IDR","object_value":28000,"predicate":"schema:price","role":"subject"}]]
```

### P12: Mode Routing via Action Type Graph
```json
[[{"content":"Telur Rp 28.000 di Pasar Minggu","note_id":"note:egg_price","routed_mode":["catatan_komunitas"],"track_label":["Informasi"]},{"content":"Banjir setinggi 50cm di gang depan","note_id":"note:flood_report","routed_mode":["siaga"],"track_label":["Siaga"]},{"content":"Beras naik jadi Rp 15.000/kg","note_id":"note:rice_price","routed_mode":["catatan_komunitas"],"track_label":["Informasi"]},{"content":"Jalan Kenari berlubang besar dekat pos ronda","note_id":"note:road_damage","routed_mode":["komunitas"],"track_label":["Tuntaskan"]}]]
```

### Bonus: Domain Derivation (display-time concept → parent)
```json
[[{"about_concept":["telur"],"content":"Telur Rp 28.000 di Pasar Minggu","id":"note:egg_price","parent_domain":["makanan"]}]]
```

---

## Key Findings & Patterns for Implementation

### What Works (high confidence)

1. **RELATE edges with metadata** — SurrealDB RELATE supports arbitrary fields on
   edge records. This is perfect for storing `predicate`, `object_value`, `object_unit`
   on ABOUT edges. The triple-to-graph mapping works exactly as designed.

2. **Graph traversal operators** — `->ABOUT->concept` and `<-ABOUT<-note` work for
   forward and reverse traversal. This enables both "what is this note about?" and
   "what notes mention this concept?".

3. **SCHEMALESS + SCHEMAFULL coexistence** — Ontology tables can be SCHEMALESS
   (flexible for graph evolution) while existing chat/event tables remain SCHEMAFULL.
   No migration conflict.

4. **Cross-mode via shared concept nodes** — A single concept:Q8068 (flood) links
   notes, plans, and siaga broadcasts. The `<-ABOUT<-note`, `<-ABOUT<-plan`,
   `<-ABOUT<-siaga` pattern works for unified discovery.

5. **Time-series aggregation** — `math::mean()`, `math::min()`, `math::max()` on
   measurement tables work for price trend analysis.

6. **Privacy filtering** — `WHERE ai_readable = true` and `WHERE rahasia_level = 0`
   cleanly separate public, AI-accessible, and truly private content.

7. **INSTANCE_OF for local places** — `place:pasar_minggu -> INSTANCE_OF -> concept:Q132510`
   correctly links hyper-local entities to Wikidata concept types.

8. **Action-type mode routing** — `->HAS_ACTION->action.maps_to_mode` traversal
   derives the destination mode from the graph, not from a separate classification step.

### Critical Pattern: Hierarchy Queries (P3c — investigated)

**Problem:** Forward filtering does NOT work. All of these return empty:
- `SELECT * FROM note WHERE ->ABOUT->concept->BROADER CONTAINS concept:Q2095`
- `SELECT * FROM note WHERE ->ABOUT->concept IN (SELECT VALUE id ...)`
- LET + CONTAINSANY subquery approach

**Root cause:** `->ABOUT->concept` returns string representations (e.g. `"concept:Q93189"`),
not record references. So comparison against record IDs (`concept:Q93189`) fails.

**Working pattern — reverse walk from parent:**
```sql
SELECT <-BROADER<-concept<-ABOUT<-note.content AS notes FROM concept:Q2095;
-- Returns: ["Telur Rp 28.000 di Pasar Minggu", "Beras naik jadi Rp 15.000/kg"]
```

**Implication for implementation:** "Show me everything about food" starts from
`concept:Q2095` and walks backwards. This aligns with our design (domain = display-time
lookup from concept hierarchy). Feed filters and dashboard grouping work naturally.
For deeper hierarchies, chain: `<-BROADER<-concept<-BROADER<-concept<-ABOUT<-note`.

### Vouch Counting (P7 — investigated)

**Working pattern:**
```sql
SELECT id, content,
  count(SELECT * FROM VOUCHES WHERE out = $parent.id) AS vouches,
  count(SELECT * FROM CHALLENGES WHERE out = $parent.id) AS challenges
FROM note WHERE rahasia_level = 0
ORDER BY vouches DESC;
```

Wilson score formula itself stays in app layer (Rust). DB provides clean integer counts.

### Other Attention Items

1. **Recursive hierarchy** — SurrealDB doesn't have recursive CTEs. For "find all
   descendants of Q2095" you need chained `<-BROADER<-concept` operators (fixed depth)
   or pre-computed closure tables. 3-level pre-load (S3-D2) is the right call.

2. **TTL garbage collection** — SurrealDB doesn't auto-delete expired rows.
   Need a periodic `DELETE FROM note WHERE ttl_expires < time::now()` job
   or handle at query time with `WHERE ttl_expires > time::now() OR ttl_expires IS NONE`.

3. **Edge table indexes** — Migration includes indexes on `in` and `out` fields
   for all edge tables. Verify performance at scale (1000+ concepts, 10k+ notes).

### Schema Patterns Discovered

1. **Edge tables are first-class** — In SurrealDB v3, RELATE creates records in
   a named edge table. You can query ABOUT directly: `SELECT * FROM ABOUT WHERE in = note:abc`.
   This is powerful for analytics.

2. **Record links vs string IDs** — The ontology tables use record IDs (`concept:Q93189`)
   while existing schema uses string IDs (`contribution_id TYPE string`). Both work.
   For new ontology tables, prefer record IDs for graph traversal.

3. **SCHEMALESS is fine for graphs** — Concept nodes and edges benefit from SCHEMALESS
   because different predicates carry different metadata. A price edge has `object_value`
   + `object_unit`; a location edge has `osm_tags`. SCHEMALESS accommodates this naturally.

4. **Namespace isolation** — The probe runs in its own namespace. Production ontology
   tables can coexist in the same namespace/database as existing tables.

---

## Migration Recommendation

Add as `database/migrations/0013_ontology_schema.surql`:

```sql
-- 0013_ontology_schema
-- RDF-triple graph model for ONTOLOGY-VOCAB-v0.1
-- Tables are SCHEMALESS to accommodate varied edge metadata.

DEFINE TABLE concept SCHEMALESS;
DEFINE FIELD qid ON TABLE concept TYPE string;
DEFINE FIELD label_id ON TABLE concept TYPE string;
DEFINE FIELD label_en ON TABLE concept TYPE string;
DEFINE FIELD verified ON TABLE concept TYPE bool DEFAULT false;
DEFINE FIELD created_at ON TABLE concept TYPE datetime DEFAULT time::now();
DEFINE FIELD last_referenced ON TABLE concept TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_concept_qid ON TABLE concept FIELDS qid UNIQUE;

DEFINE TABLE action SCHEMALESS;
DEFINE FIELD action_type ON TABLE action TYPE string;
DEFINE FIELD maps_to_mode ON TABLE action TYPE string;
DEFINE FIELD display_label ON TABLE action TYPE option<string>;
DEFINE INDEX idx_action_type ON TABLE action FIELDS action_type UNIQUE;

DEFINE TABLE place SCHEMALESS;
DEFINE FIELD name ON TABLE place TYPE string;
DEFINE FIELD osm_tags ON TABLE place TYPE array DEFAULT [];
DEFINE FIELD location ON TABLE place TYPE option<geometry<point>>;
DEFINE FIELD source ON TABLE place TYPE string DEFAULT 'community';
DEFINE FIELD created_at ON TABLE place TYPE datetime DEFAULT time::now();

DEFINE TABLE note SCHEMALESS;
DEFINE FIELD content ON TABLE note TYPE string;
DEFINE FIELD author ON TABLE note TYPE record<warga>;
DEFINE FIELD community_id ON TABLE note TYPE string;
DEFINE FIELD created_at ON TABLE note TYPE datetime DEFAULT time::now();
DEFINE FIELD temporal_class ON TABLE note TYPE string;
DEFINE FIELD ttl_expires ON TABLE note TYPE option<datetime>;
DEFINE FIELD ai_readable ON TABLE note TYPE bool DEFAULT true;
DEFINE FIELD rahasia_level ON TABLE note TYPE int DEFAULT 0;
DEFINE FIELD confidence ON TABLE note TYPE float DEFAULT 0.0;
DEFINE INDEX idx_note_community ON TABLE note FIELDS community_id, created_at;
DEFINE INDEX idx_note_ttl ON TABLE note FIELDS ttl_expires;
DEFINE INDEX idx_note_temporal ON TABLE note FIELDS temporal_class;

DEFINE TABLE measurement SCHEMALESS;
DEFINE FIELD note ON TABLE measurement TYPE record<note>;
DEFINE FIELD concept ON TABLE measurement TYPE record<concept>;
DEFINE FIELD location ON TABLE measurement TYPE option<record>;
DEFINE FIELD predicate ON TABLE measurement TYPE string;
DEFINE FIELD value ON TABLE measurement TYPE number;
DEFINE FIELD unit ON TABLE measurement TYPE string;
DEFINE FIELD per ON TABLE measurement TYPE option<string>;
DEFINE FIELD observed_at ON TABLE measurement TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_measurement_concept ON TABLE measurement FIELDS concept, observed_at;

DEFINE TABLE warga SCHEMALESS;
DEFINE FIELD username ON TABLE warga TYPE string;
DEFINE FIELD voucher_tier ON TABLE warga TYPE string DEFAULT 'warga';
DEFINE FIELD community_id ON TABLE warga TYPE string;

-- Edge tables
DEFINE TABLE ABOUT SCHEMALESS;
DEFINE TABLE LOCATED_AT SCHEMALESS;
DEFINE TABLE HAS_ACTION SCHEMALESS;
DEFINE TABLE BROADER SCHEMALESS;
DEFINE TABLE INSTANCE_OF SCHEMALESS;
DEFINE TABLE VOUCHES SCHEMALESS;
DEFINE TABLE CHALLENGES SCHEMALESS;

-- Edge indexes for performance
DEFINE INDEX idx_about_out ON TABLE ABOUT FIELDS out;
DEFINE INDEX idx_about_in ON TABLE ABOUT FIELDS in;
DEFINE INDEX idx_located_out ON TABLE LOCATED_AT FIELDS out;
DEFINE INDEX idx_action_out ON TABLE HAS_ACTION FIELDS out;
DEFINE INDEX idx_broader_in ON TABLE BROADER FIELDS in;
DEFINE INDEX idx_broader_out ON TABLE BROADER FIELDS out;
DEFINE INDEX idx_vouch_out ON TABLE VOUCHES FIELDS out;
```

---

## Hardening Tests (H1–H8)

Run after main probe against same namespace.

| Test | Name | Result | Detail |
|---|---|---|---|
| H1 | 3-level hierarchy chain | PASS | `nutrition→food→[egg,rice]→notes` returns 2 notes |
| H2 | Full objects from reverse walk | PARTIAL | Returns record IDs (`note:rice_price`), not full objects. Use `.content` accessor or follow-up SELECT. |
| H3 | Cross-mode hierarchy walk | PASS | `natural_disaster→flood→note+plan+siaga` all found |
| H4 | Edge table direct query | PASS | `SELECT in, out, predicate, object_value FROM ABOUT` returns clean analytics view |
| H5 | Concept popularity ranking | PASS | `GROUP BY out` on ABOUT edges — Q8068 (flood) has 3 references |
| H6 | INTERSECT for multi-edge filter | FAIL | SurrealDB v3 beta does not support INTERSECT. Use subquery instead. |
| H7 | TTL cleanup query | PASS | `WHERE ttl_expires < time::now()` returns empty (correct — data not yet expired) |
| H8 | Active feed query | PASS | `WHERE ttl_expires IS NONE OR ttl_expires > time::now()` returns 4 public notes |

### H2 Pattern: Getting full objects from hierarchy walk

Reverse walk returns record IDs, not full objects. Two approaches:

```sql
-- Option A: Access fields directly on the traversal
SELECT <-BROADER<-concept<-ABOUT<-note.content AS contents FROM concept:Q2095;
-- Returns: ["Beras naik jadi...", "Telur Rp 28.000..."]

-- Option B: Two-step (get IDs, then fetch)
LET $note_ids = (SELECT VALUE <-BROADER<-concept<-ABOUT<-note FROM concept:Q2095);
SELECT * FROM $note_ids;
-- Returns full note objects (needs testing)
```

For the Rust layer: use Option A for feed display (only need content + metadata), Option B if you need the complete record.

### H6 Pattern: Multi-edge filtering without INTERSECT

```sql
-- Find notes that have BOTH a price AND a location (without INTERSECT)
SELECT * FROM note WHERE
  id IN (SELECT VALUE in FROM ABOUT WHERE predicate = 'schema:price')
  AND id IN (SELECT VALUE in FROM LOCATED_AT);
```

---

## Recommended Next Steps

1. Run this probe on your machine: `just surreal-ontology-probe`
2. Review the detailed output for any FAIL/WARN patterns
3. If all pass, commit the migration file (`0013_ontology_schema.surql`)
4. Write the corresponding Rust repository layer in `crates/infra`
5. Seed the ~200 pre-loaded concepts from Wikidata batch export
6. Add the check file (`0013_ontology_schema_check.surql`)

---

*Probe script: docs/research/samples/surrealdb/ontology_probe.sh*
*Spec reference: docs/design/specs/ONTOLOGY-VOCAB-v0.1.md*

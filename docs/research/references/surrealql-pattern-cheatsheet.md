# SurrealQL Pattern Cheatsheet — Ontology Repository Layer

**Purpose:** Ready-to-use query patterns for `crates/infra` repository implementation.
Every pattern here is **probe-validated** against SurrealDB v3.0.0-beta.4.

**Source:** `ontology-probe-report.md` (15/15 pass) + hardening H1–H8 + cheatsheet validation C1–C10 (8 pass, 2 known-failing documented).

---

## Gotchas (Read First)

| # | Gotcha | Impact | Workaround |
|---|---|---|---|
| G1 | Graph traversal (`->TABLE->`) returns **string representations**, not record refs | `WHERE ->ABOUT->concept CONTAINS concept:Q2095` **always fails** | Use reverse walk from target node (see §2) |
| G2 | No `INTERSECT` keyword | Multi-edge filter with INTERSECT causes parse error | Use `WHERE id IN (subquery) AND id IN (subquery)` (see §6) |
| G3 | No recursive CTE | Can't do `WITH RECURSIVE` for unbounded hierarchy | Chain `<-BROADER<-concept` at fixed depth (3 levels per S3-D2) |
| G4 | Reverse walk without accessor returns **record IDs**, not full objects | `<-ABOUT<-note` returns `["note:abc"]` — but `<-ABOUT<-note.*` returns **full objects** (C3 validated) | Append `.*` for full objects, `.field` for specific fields (see §2) |
| G5 | No auto TTL expiry | Expired rows stay in table until explicitly deleted | Query-time filter + periodic cleanup job (see §7) |
| G6 | `array::intersect()` is a function, not query-level | Available for in-memory array ops, not for table joins | Use subquery pattern for table-level filtering |
| G7 | `BEGIN` / `COMMIT` in scripted CLI flow is unreliable in v3.0.0-beta.4 | `COMMIT` can return `Cannot COMMIT without starting a transaction` while writes still persist | Use multi-statement idempotent writes; gate true transaction semantics by runtime verification |

---

## Known Failing Forms (Validated on v3.0.0-beta.4)

### KF1: `BEGIN` / `COMMIT` transaction block in scripted CLI flow

Known failing form:

```sql
BEGIN TRANSACTION;
CREATE note:$id SET content = $content;
COMMIT TRANSACTION;
```

Observed error:

```text
Thrown error: Invalid statement: Cannot COMMIT without starting a transaction
```

Recommended form:

```sql
-- Use a single multi-statement write block without COMMIT in CLI scripts.
CREATE note:$id SET content = $content;
RELATE note:$id -> ABOUT -> concept:$qid SET predicate = $predicate;
```

### KF2: `math::mean(value)` without proper grouped aggregation context

Known failing form:

```sql
SELECT
  math::mean(value) AS avg_price,
  math::min(value) AS min_price,
  math::max(value) AS max_price,
  count() AS report_count
FROM measurement
WHERE concept = concept:$qid
  AND observed_at > $since;
```

Observed error:

```text
Thrown error: Incorrect arguments for function math::mean(). Argument 1 was the wrong type. Expected `array<number>` but found `28000`
```

Recommended form:

```sql
SELECT
  concept,
  math::mean(value) AS avg_price,
  math::min(value) AS min_price,
  math::max(value) AS max_price,
  count() AS report_count
FROM measurement
WHERE concept = concept:$qid
  AND observed_at > $since
GROUP BY concept;
```

---

## 1. Write Patterns

### 1.1 Create Note with Triples (multi-statement write)

`BEGIN` / `COMMIT` is currently unreliable in scripted CLI execution on v3.0.0-beta.4.
Use this as a single multi-statement write block. If your runtime validates transaction
semantics, wrap this block with `BEGIN TRANSACTION;` and `COMMIT TRANSACTION;`.

```sql
-- Content node
CREATE note:$id SET
  content = $content,
  author = warga:$author_id,
  community_id = $community_id,
  created_at = time::now(),
  temporal_class = $temporal_class,
  ttl_expires = $ttl_expires,
  ai_readable = $ai_readable,
  rahasia_level = $rahasia_level,
  confidence = $confidence;

-- Ensure concept exists before RELATE (avoid dangling target IDs)
UPSERT concept:$qid SET
  qid = $qid,
  last_referenced = time::now();

-- Subject triple (repeat per triple)
RELATE note:$id -> ABOUT -> concept:$qid
  SET predicate = $predicate,
      object_value = $object_value,
      object_unit = $object_unit,
      object_per = $object_per,
      role = 'subject';

-- Location triple (if present)
RELATE note:$id -> LOCATED_AT -> concept:$location_qid
  SET predicate = 'schema:contentLocation',
      osm_tags = $osm_tags;

-- Action/routing triple (always 1)
RELATE note:$id -> HAS_ACTION -> action:$action_type
  SET predicate = 'schema:potentialAction';

-- Measurement (if numeric predicate)
CREATE measurement:$m_id SET
  note = note:$id,
  concept = concept:$qid,
  location = concept:$location_qid,
  predicate = $predicate,
  value = $numeric_value,
  unit = $unit,
  per = $per,
  observed_at = time::now();
```

### 1.2 Create Concept (lazy creation during classification)

```sql
CREATE concept:$qid SET
  qid = $qid_string,
  label_id = $label_id,
  label_en = $label_en,
  verified = false,
  created_at = time::now(),
  last_referenced = time::now();
```

### 1.3 Create BROADER Hierarchy Link

```sql
RELATE concept:$child_qid -> BROADER -> concept:$parent_qid;
```

### 1.4 Vouch / Challenge

```sql
RELATE warga:$user_id -> VOUCHES -> note:$note_id
  SET vouched_at = time::now();

RELATE warga:$user_id -> CHALLENGES -> note:$note_id
  SET challenged_at = time::now(),
      reason = $reason;
```

### 1.5 Update Last Referenced (on concept mention)

```sql
UPDATE concept:$qid SET last_referenced = time::now();
```

---

## 2. Hierarchy Queries (Critical — G1 applies)

### 2.1 "Show all notes about food" (walk DOWN from parent)

```sql
-- Single level: food → [egg, rice, ...] → notes
SELECT <-BROADER<-concept<-ABOUT<-note.content AS notes
  FROM concept:Q2095;

-- 2 levels: nutrition → food → [items] → notes
SELECT <-BROADER<-concept<-BROADER<-concept<-ABOUT<-note.content AS notes
  FROM concept:Q11004;

-- 3 levels (max pre-loaded per S3-D2):
SELECT <-BROADER<-concept<-BROADER<-concept<-BROADER<-concept<-ABOUT<-note.content AS notes
  FROM concept:$root;
```

### 2.2 Get Full Note Objects (not just IDs)

```sql
-- Option A: Field accessor (preferred for feed display)
SELECT <-BROADER<-concept<-ABOUT<-note.* AS notes
  FROM concept:Q2095;

-- Option B: Specific fields
SELECT
  <-BROADER<-concept<-ABOUT<-note.id AS note_ids,
  <-BROADER<-concept<-ABOUT<-note.content AS contents,
  <-BROADER<-concept<-ABOUT<-note.created_at AS dates
  FROM concept:Q2095;
```

### 2.3 Navigate UP (note → concept → parent domain)

```sql
SELECT
  ->ABOUT->concept.label_id AS about_concept,
  ->ABOUT->concept->BROADER->concept.label_id AS parent_domain
  FROM note:$id;
```

---

## 3. Feed Queries

### 3.1 Active Feed (public notes, respecting TTL)

```sql
SELECT * FROM note
  WHERE rahasia_level = 0
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
  ORDER BY created_at DESC
  LIMIT $page_size
  START $offset;
```

### 3.2 AI-Readable Feed (for pattern detection)

```sql
SELECT * FROM note
  WHERE ai_readable = true
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
  ORDER BY created_at DESC;
```

### 3.3 Community-Scoped Feed

```sql
SELECT * FROM note
  WHERE community_id = $community_id
    AND rahasia_level = 0
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
  ORDER BY created_at DESC
  LIMIT $page_size;
```

### 3.4 Feed by Concept (tapped concept pill)

```sql
SELECT in AS note, in.content AS content, in.created_at AS created_at
  FROM ABOUT
  WHERE out = concept:$qid
  ORDER BY created_at DESC;
```

### 3.5 Cursor Pagination (safer under concurrent inserts)

```sql
-- First page: same as §3.1 with LIMIT only
SELECT * FROM note
  WHERE rahasia_level = 0
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
  ORDER BY created_at DESC, id DESC
  LIMIT $page_size;

-- Next page: pass last row from previous page as cursor
SELECT * FROM note
  WHERE rahasia_level = 0
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
    AND (
      created_at < $cursor_created_at
      OR (created_at = $cursor_created_at AND id < $cursor_id)
    )
  ORDER BY created_at DESC, id DESC
  LIMIT $page_size;
```

---

## 4. Vouch Ranking

### 4.1 Notes with Vouch/Challenge Counts (for Wilson score input)

```sql
SELECT
  id,
  content,
  created_at,
  count(SELECT * FROM VOUCHES WHERE out = $parent.id) AS vouches,
  count(SELECT * FROM CHALLENGES WHERE out = $parent.id) AS challenges
  FROM note
  WHERE rahasia_level = 0
    AND (ttl_expires IS NONE OR ttl_expires > time::now())
  ORDER BY vouches DESC
  LIMIT $page_size;
```

Wilson score computation stays in Rust (`crates/domain`). DB provides clean integer counts.

### 4.2 Concept Popularity (for trending/analytics)

```sql
SELECT out AS concept, count() AS ref_count
  FROM ABOUT
  GROUP BY out
  ORDER BY ref_count DESC
  LIMIT 20;
```

### 4.3 Grouped Vouches (analytics view)

```sql
SELECT out AS note, count() AS vouch_count
  FROM VOUCHES
  GROUP BY out
  ORDER BY vouch_count DESC;
```

---

## 5. Cross-Mode Discovery

### 5.1 Everything About a Concept (all modes)

```sql
SELECT
  <-ABOUT<-note.content AS notes,
  <-ABOUT<-plan.title AS plans,
  <-ABOUT<-siaga.message AS alerts
  FROM concept:$qid;
```

### 5.2 Mode Routing (note → action → mode)

```sql
SELECT
  id,
  content,
  ->HAS_ACTION->action.maps_to_mode AS routed_mode,
  ->HAS_ACTION->action.display_label AS track_label
  FROM note;
```

---

## 6. Multi-Edge Filtering (G2 — no INTERSECT)

### 6.1 Notes that have BOTH a specific predicate AND a location

```sql
SELECT * FROM note WHERE
  id IN (SELECT VALUE in FROM ABOUT WHERE predicate = 'schema:price')
  AND id IN (SELECT VALUE in FROM LOCATED_AT);
```

### 6.2 Notes about a concept AND in a specific location

```sql
SELECT * FROM note WHERE
  id IN (SELECT VALUE in FROM ABOUT WHERE out = concept:$qid)
  AND id IN (SELECT VALUE in FROM LOCATED_AT WHERE out = concept:$location_qid);
```

### 6.3 Array intersection (in-memory, for app-layer use)

```sql
-- Only for comparing two arrays, NOT for table joins
LET $a = [1, 2, 3];
LET $b = [2, 3, 4];
RETURN array::intersect($a, $b);
-- Returns: [2, 3]
```

---

## 7. TTL & Cleanup

### 7.1 Active Notes (query-time filter)

```sql
SELECT * FROM note
  WHERE ttl_expires IS NONE OR ttl_expires > time::now();
```

### 7.2 Expired Notes (cleanup job target)

```sql
DELETE FROM note
  WHERE ttl_expires IS NOT NONE
    AND ttl_expires < time::now();
```

### 7.3 Expiring Soon (for UI warning)

```sql
SELECT * FROM note
  WHERE ttl_expires IS NOT NONE
    AND ttl_expires > time::now()
    AND ttl_expires < time::now() + 1h;
```

---

## 8. Edge Table Direct Queries (Analytics)

### 8.1 All Relations for a Note

```sql
SELECT * FROM ABOUT WHERE in = note:$id;
SELECT * FROM LOCATED_AT WHERE in = note:$id;
SELECT * FROM HAS_ACTION WHERE in = note:$id;
```

### 8.2 Edge Metadata Inspection

```sql
SELECT in, out, predicate, object_value, object_unit
  FROM ABOUT
  WHERE predicate = 'schema:price';
```

### 8.3 INSTANCE_OF Lookup (local place → Wikidata type)

```sql
-- Place to concept type
SELECT ->INSTANCE_OF->concept.label_id AS concept_type, name
  FROM place:$id;

-- All local places of a type
SELECT <-INSTANCE_OF<-place.name AS local_places
  FROM concept:$qid;
```

---

## 9. Time-Series Aggregation

### 9.1 Price Trend for a Concept

```sql
-- Keep grouped field (`concept`) in SELECT to satisfy GROUP BY semantics.
SELECT
  concept,
  math::mean(value) AS avg_price,
  math::min(value) AS min_price,
  math::max(value) AS max_price,
  count() AS report_count
  FROM measurement
  WHERE concept = concept:$qid
    AND observed_at > $since
  GROUP BY concept;
```

### 9.2 Latest Measurement per Concept (for mini-charts)

```sql
SELECT * FROM measurement
  WHERE concept = concept:$qid
  ORDER BY observed_at DESC
  LIMIT 30;
```

---

## 10. Concept Management

### 10.1 Verify a Concept (background job)

```sql
UPDATE concept:$qid SET verified = true, last_referenced = time::now();
```

### 10.2 Upsert Concept (lazy creation — if exists, touch; if not, create)

```sql
-- SurrealDB v3 UPSERT
UPSERT concept:$qid SET
  qid = $qid_string,
  label_id = $label_id,
  label_en = $label_en,
  last_referenced = time::now();
```

### 10.3 Stale Concepts (for cleanup/re-verification)

```sql
SELECT * FROM concept
  WHERE verified = false
    OR last_referenced < time::now() - 30d
  ORDER BY last_referenced ASC;
```

---

## Rust Repository Method → SurrealQL Mapping

| Repository Method | Section | Primary Pattern |
|---|---|---|
| `create_note_with_triples()` | §1.1 | Multi-statement write: CREATE + RELATE × N |
| `upsert_concept()` | §10.2 | UPSERT |
| `link_hierarchy()` | §1.3 | RELATE BROADER |
| `vouch_note()` / `challenge_note()` | §1.4 | RELATE VOUCHES/CHALLENGES |
| `get_feed()` | §3.1 | SELECT + TTL filter + pagination |
| `get_feed_by_concept()` | §3.4 | SELECT from ABOUT edge table |
| `get_notes_by_hierarchy()` | §2.1 | Reverse walk from parent concept |
| `get_note_domain()` | §2.3 | Forward walk note→concept→BROADER |
| `get_ranked_notes()` | §4.1 | SELECT with count subquery |
| `get_concept_popularity()` | §4.2 | GROUP BY on ABOUT edge |
| `get_cross_mode()` | §5.1 | Reverse walk from concept |
| `get_mode_routing()` | §5.2 | Forward walk to action node |
| `get_price_trend()` | §9.1 | Aggregation on measurement |
| `cleanup_expired()` | §7.2 | DELETE with TTL filter |
| `get_stale_concepts()` | §10.3 | SELECT with age filter |
| `get_multi_edge_filter()` | §6.1 | `WHERE id IN (subquery) AND id IN (subquery)` |

---

*Generated from probe findings. Tested against SurrealDB v3.0.0-beta.4 in-memory; see gotchas for transaction caveat.*
*Companion to: ontology-probe-report.md, ONTOLOGY-VOCAB-v0.1.md §4*

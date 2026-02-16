# Gotong Royong — Design Decisions Log

> **Purpose:** Single source of truth for all design decisions, including rationale and what they supersede. Read alongside DESIGN-CONTEXT.md. Decisions are grouped by domain and numbered for cross-referencing.

---

## How to Read This Document

- **Status**: `CONFIRMED` = locked, implement as-is. `SUPERSEDED` = replaced by a later decision. `PARKED` = consciously deferred.
- **Session**: When the decision was made (for audit trail).
- **Affects**: Which spec files need updating to reflect this decision.
- Decisions build on each other — later decisions may refine or replace earlier ones. Always check the latest decision in a chain.

---

## Session 1 (2026-02-15): Adaptive Path Migration

These decisions migrated the platform from fixed track lifecycles to adaptive path guidance.

| # | Decision | Status |
|---|---|---|
| S1-01 | Fixed track lifecycles (Bahas→Rancang→Garap→Tuntas) replaced by Adaptive Path Guidance. LLM proposes case-specific phases and checkpoints. Track names persist only as optional `track_hint` metadata. | CONFIRMED |
| S1-02 | 5 track hints (Tuntaskan, Wujudkan, Telusuri, Rayakan, Musyawarah) remain as optional classification metadata on plans. They help discovery and LLM context but do NOT drive lifecycle. | CONFIRMED — but see S3-MD1 which subsumes track hints into RDF triples |
| S1-03 | All old terminology (Bahas, Rancang, Garap, Tuntas) removed from active specs. Historical references kept only in archive/research files. | CONFIRMED |

---

## Session 2 (2026-02-16, Part 1): Documentation Cleanup & Catatan Komunitas

### Documentation Cleanup

| # | Decision | Status |
|---|---|---|
| S2-01 | Micro-files (09-evidence-triad, 10-voting-power, 12-member-progression) consolidated into 07-governance-voting.md. Originals converted to one-line tombstone stubs pointing to merged location. | CONFIRMED |
| S2-02 | Files that cannot be deleted (virtiofs mount restriction) are replaced with one-line tombstone stubs: `[MERGED]`, `[REMOVED]`, or `[ARCHIVED]` with pointer to replacement. | CONFIRMED |
| S2-03 | UI-UX-SPEC-v0.5.md renumbered from 29 to 24 active sections. Tombstone table added at bottom. | CONFIRMED |
| S2-04 | DESIGN-SEQUENCE.md and TRACK-MAP.md archived (tombstone stubs). Replaced by ADAPTIVE-PATH-SPEC-v0.1.md and ADAPTIVE-PATH-MAP.md respectively. | CONFIRMED |

### Catatan Komunitas (4th Mode)

| # | Decision | Rationale | Status |
|---|---|---|---|
| S2-05 | Platform has **4 modes**, not 3: Komunitas, Catatan Saksi, Siaga, **Catatan Komunitas** (new). | Gap: no clean path for lightweight information sharing (prices, status updates, schedules). | CONFIRMED |
| S2-06 | No forced clustering of community notes. Individual posts with tags. | Clustering carries merge risk — wrong merges hide data, dilute attribution. Tags give same discoverability without write-side merge. | CONFIRMED |
| S2-07 | Community notes are globally visible (not scoped to one RT/RW). Location tags for sorting. | Facts like prices and road closures are useful beyond one RT. Author's community shown as source context. | CONFIRMED |
| S2-08 | Credit cap: max 5 notes/day earn Type A credit. | Prevents low-effort spam farming. Type A is binary, small — rewards sharing without creating farming vector. | CONFIRMED |
| S2-09 | Research→Notes bridge: author chooses at Telusuri completion. Option presented: "Publish as Catatan Komunitas?" Not automatic. | Not all research findings are worth public posting. Author decides. | CONFIRMED |
| S2-10 | SurrealDB's native graph, time-series, and advanced features leveraged for the ontology/data model. | SurrealDB is the confirmed database. Graph relations (RELATE), time-series values, and native graph traversal replace custom data structures. | CONFIRMED |

---

## Session 3 (2026-02-16, Part 2): Ontology, Architecture & System Design Decisions

### MD — Major Design Decisions

| # | Decision | Rationale | Status | Affects |
|---|---|---|---|---|
| S3-MD1 | **Ontology model: Pure RDF-style triples.** Every piece of content classified as `(subject, predicate, object)` using Schema.org properties and Wikidata QIDs. No custom vocabulary. Each position constrained by its source standard, not by artificial lists. LLM produces 2-5 triples per input. | Custom taxonomies are expensive (must load into every prompt). LLMs are already deeply trained on Schema.org and Wikidata. The constraint IS the standard. Zero maintenance. | CONFIRMED | ONTOLOGY-VOCAB-v0.1.md (rewrite), AI-00, AI-01 output schemas |
| S3-MD2 | **Track hints and mode routing collapse into one system.** Schema.org Action types (`InformAction`, `RepairAction`, `CreateAction`, `SearchAction`, `AchieveAction`, `AssessAction`, `AlertAction`) replace both track hints and mode routing. Domain (ranah) derived from Wikidata hierarchy at display time, not classified separately. | Track hints and ranah were two separate classification tasks doing overlapping work. Unifying into one triple eliminates the collision. | CONFIRMED — supersedes S1-02 partially (track hints remain as UI labels but derived from Action type) | ENTRY-PATH-MATRIX, ADAPTIVE-PATH-SPEC, AI-00, AI-01, DESIGN-CONTEXT |
| S3-MD3 | **Navigation rearranged: Catatan Komunitas gets its own tab.** New 5-tab layout: 🏠 Beranda \| 📝 Catatan \| 🤝 Bantu \| 🔔 Notifikasi \| ☰ Lainnya. Profile (CV Hidup), Terlibat, Template Saya, Pengaturan move to hamburger menu. | Catatan Komunitas is the most casually sticky content — lowest barrier to entry. Best for user acquisition. Terlibat is a power-user feature that doesn't need prime tab real estate. | CONFIRMED | UI-UX-SPEC (navigation section), DESIGN-CONTEXT, DESIGN-DNA |
| S3-MD4 | **Rahasia levels apply to Catatan Komunitas with AI-readable flag.** Notes support L0-L3 privacy. New `ai_readable` boolean per note. When true, AI can use note data for anonymized pattern detection and community warnings. L2+ai_readable = anonymous but AI learns from it. | Enables community safety warnings without public bad-actor tagging. AI aggregates anonymous reports and warns through patterns — no names, no accusations, just signal. | CONFIRMED | ENTRY-PATH-MATRIX (Section 4, 10), Rahasia spec, AI-04 |
| S3-MD5 | **Community warning via AI pattern aggregation, NOT public tagging.** No public bad-actor tagging system. Instead, AI detects patterns from multiple ai_readable anonymous reports and warns others: "Beberapa warga telah melaporkan masalah serupa di area ini." No names, no accusations. | Public tagging carries mob risk, false accusations, weaponization. AI aggregation is safer — individuals can't be targeted, but the community still gets warned. | CONFIRMED — resolves ENTRY-PATH-MATRIX Open Question #2 | AI-04, ENTRY-PATH-MATRIX |
| S3-MD6 | **Pre-screen for ALL modes, including Siaga.** AI-00 conversational triage always guides the user — even for emergencies. The triage conversation IS the pre-screen. Makes users calmer, makes reports more structured and useful. Siaga still broadcasts fast, but with better-quality initial information. | Even in emergencies, a few seconds of guided input produces dramatically better information for responders. LLM guidance calms panicked users. | CONFIRMED — modifies Siaga's "zero moderation hold" to "zero moderation hold AFTER triage" | AI-00, ENTRY-PATH-MATRIX (Siaga section), DESIGN-CONTEXT |
| S3-MD7 | **Two LLM tiers with different capabilities.** Fast/cheap (Haiku-class): real-time UX — classification, triage, inline suggestions. No internet needed. Capable+browsing (Sonnet-class with tools): background jobs — QID verification, label refresh, enrichment, pattern detection. Has internet access. | Clean separation of real-time UX (latency-sensitive) from background enrichment (accuracy-sensitive). LLMs with browsing can verify Wikidata QIDs, check prices, validate OSM tags without custom API integrations. | CONFIRMED | AI-SPEC (model selection), ONTOLOGY-VOCAB |
| S3-MD8 | **Cross-community moderation: ranking-based, not community-owned.** Notes ranked by: vouch count (Wilson score weighted by voucher tier), recency, location proximity, challenge count. No single community "owns" moderation of globally visible notes. Users can pin notes. >5 Lapor → AI-04 pre-screen → hidden pending review by any Pilar+ tier user from any community. | Global notes need global moderation. Community-specific moderators would create jurisdictional conflicts. Ranking is emergent and scales without governance overhead. | CONFIRMED — resolves ENTRY-PATH-MATRIX Open Question #3 | ENTRY-PATH-MATRIX, UI-UX-SPEC (feed ranking) |

### A — Architecture & Integration Decisions

| # | Decision | Rationale | Status |
|---|---|---|---|
| S3-A1 | Track hints and ranah are NOT separate dimensions — they're unified in the RDF triple model. Schema.org Action type replaces track hints for routing. Wikidata hierarchy replaces ranah for domain classification. Both derived, not separately classified. | See S3-MD1 and S3-MD2. Eliminates confusion about whether track_hint and ranah overlap or conflict. | CONFIRMED |
| S3-A2 | Ontology classification happens during AI-00 triage as part of triple generation. AI-00 produces triples. AI-01 can refine/validate. Single pipeline, not separate classification steps. | Triples are the classification. No need for separate ontology classification step. | CONFIRMED |
| S3-A3 | TTL: `temporal_class` (derived from triple properties) sets the default. AI-00 can suggest override based on context. Author has final say within bounds. | Temporal class is structural (from the triples — Schema.org properties like `schema:price` imply ephemeral). But human judgment should override. | CONFIRMED |
| S3-A4 | Conversation carryover: AI-00 produces a **summary** for the Percakapan tab. Full triage transcript preserved in graph as `triage_transcript` on the content node. Searchable, queryable, but not shown in conversation UI. | Summary is cleaner for UX. Full data preserved in graph — nothing is lost. Graph connectivity means the transcript is always findable. | CONFIRMED |
| S3-A5 | Manual override ("Pilih sendiri") available at any point, with **reputation threshold**. Pemula (new users) get full AI-00 guidance. Kontributor+ tier see "Pilih sendiri" earlier in the triage flow. | Rewards familiarity with the platform. New users benefit from guidance. Experienced users aren't slowed down. | CONFIRMED |

### B — Catatan Komunitas Decisions

| # | Decision | Rationale | Status |
|---|---|---|---|
| S3-B1 | Moderation: pre-screen for all modes. AI-00 triage IS the pre-screen. Even Siaga goes through conversational triage (but fast — focused on extracting structured info). | See S3-MD6. LLM guidance produces better reports across all modes. | CONFIRMED |
| S3-B2 | Rahasia + AI-readable flag on community notes. See S3-MD4 for full detail. | See S3-MD4. | CONFIRMED |
| S3-B3 | Catatan Komunitas gets dedicated tab (📝 Catatan, position 2). See S3-MD3 for full navigation redesign. | See S3-MD3. Catatan is the user acquisition magnet. | CONFIRMED |
| S3-B4 | Discovery UX: **progressive disclosure** powered by graph. (1) Feed: reverse-chronological notes with tappable concept pills. (2) Concept tap: all related notes via graph traversal, with time-series mini-chart for numeric data. (3) Search: free text → LLM resolves to QIDs → results as concept groups. (4) BROADER expansion: "See also" links up the Wikidata hierarchy. | Graph makes all discovery queries cheap — just traversal. No full-text search needed for concept discovery. Progressive disclosure keeps UI simple. | CONFIRMED |
| S3-B5 | Community warning: no public tagging. AI aggregation from anonymous reports. See S3-MD5. | See S3-MD5. | CONFIRMED |
| S3-B6 | Cross-community moderation: ranking-based. See S3-MD8. | See S3-MD8. | CONFIRMED |

### C — Adaptive Path Orchestration Decisions

| # | Decision | Rationale | Status |
|---|---|---|---|
| S3-C1 | Phase elaboration triggers at **50%** checkpoint completion (not waiting for phase to fully complete). Gives planning lead time. | LLM can draft the next phase while current work continues. Users can review before phase activates. | CONFIRMED |
| S3-C2 | Affinity weight scope: **global** for now. Both global and community-local usage patterns presented to AI for decision. AI decides which pattern to use. It's a suggestion, not a constraint. | Global is simpler to start. AI has flexibility to override. Local patterns can be added later when data volume justifies it. | CONFIRMED |
| S3-C3 | Template governance: **everyone** can save templates. Each user gets a personal "Template Saya" list (accessible via hamburger → Lainnya menu). | Spreads good practices organically. Templates as a user acquisition feature — "I used this template for kerja bakti, you can too." | CONFIRMED |
| S3-C4 | Recurring schedule limits: **daily minimum** frequency. Users can also manually recreate from saved template at any time. | Prevents abuse (hourly auto-spawn would be spam). Manual recreation from template is always available. | CONFIRMED |
| S3-C5 | Suggestion timeout: **standard timeout for everyone**. No per-community connectivity adjustments. Users can always edit after auto-accept — nothing is permanent. | Simplicity. The ability to edit post-accept removes the risk of bad auto-accepts. | CONFIRMED |

### D — Ontology & Data Decisions

| # | Decision | Rationale | Status |
|---|---|---|---|
| S3-D1 | QID verification cadence: **configurable** by system admin. Default proposal: hourly for new concepts, daily sweep for all. This also raises the broader principle: **operational parameters should be configurable, not hardcoded.** | Different deployments may have different needs. Rural deployments with limited bandwidth might verify less frequently. | CONFIRMED |
| S3-D2 | Hierarchy depth: **3 levels of BROADER pre-loaded**. But since the graph is fully connected, deeper queries still work via traversal — depth limit is about pre-loading performance, not query capability. | 3 levels covers most useful discovery (egg → food → nutrition). Deeper paths still reachable at query time. | CONFIRMED |
| S3-D3 | Schema.org extensions: **use them** (e.g., `GovernmentService`, `MedicalCondition`). Verify programmatically using browsing-capable LLM tier. | Extensions are richer for community context. Programmatic verification ensures correctness. | CONFIRMED |
| S3-D4 | OSM tag granularity: **allow compound tags** (e.g., `["amenity=marketplace", "wholesale=yes"]`). SurrealDB indexes arrays natively. | More specific = richer queries. No technical cost with array indexing. | CONFIRMED |
| S3-D5 | Wikidata labels: **store locally, update incrementally**. Batch import ~200 common concepts at deploy. Lazy creation for new QIDs (LLM provides label at classification time). Background job (browsing-capable LLM) periodically verifies and refreshes labels from Wikidata. Local-first, eventually consistent. | Fast reads (no API call at display time). Eventually consistent (background refresh). Leverages browsing-capable LLM for maintenance. | CONFIRMED |

### E — Reputation & Credit Decisions

| # | Decision | Rationale | Status |
|---|---|---|---|
| S3-E1 | **Entire reputation/credit system design PARKED for separate session.** Will study existing systems in other repos and design holistically. | Reputation is complex enough to deserve its own deep-dive. Rushing it risks gaming vectors and unintended incentives. | PARKED |

---

## Resolved Open Questions (Cross-Reference)

These open questions from various specs have been resolved by decisions above.

| Source | Question | Resolved By | Resolution |
|---|---|---|---|
| ENTRY-PATH-MATRIX §10 Q1 | Catatan Komunitas moderation: post-first or pre-screen? | S3-B1, S3-MD6 | Pre-screen. AI-00 triage IS the pre-screen. |
| ENTRY-PATH-MATRIX §10 Q2 | Community warning system: bad actor tagging? | S3-MD5 | No public tagging. AI aggregates anonymous reports for pattern-based warnings. |
| ENTRY-PATH-MATRIX §10 Q3 | Cross-community governance: who moderates globally visible notes? | S3-MD8 | Ranking-based moderation (Wilson score). Pilar+ from any community can review flagged notes. |
| ENTRY-PATH-MATRIX §10 Q4 | Anonymous notes: should Rahasia levels apply? | S3-MD4 | Yes. L0-L3 apply. New `ai_readable` flag enables AI pattern learning from anonymous notes. |
| ADAPTIVE-PATH-ORCH §10 Q1 | Elaboration timing: 50% or completion? | S3-C1 | 50% of checkpoints triggers next phase elaboration. |
| ADAPTIVE-PATH-ORCH §10 Q2 | Affinity weight scope? | S3-C2 | Global for now. AI decides using both global and local patterns. |
| ADAPTIVE-PATH-ORCH §10 Q3 | Template governance: who can save? | S3-C3 | Everyone. Personal template list in hamburger menu. |
| ADAPTIVE-PATH-ORCH §10 Q4 | Recurring schedule limits? | S3-C4 | Daily minimum frequency. |
| ADAPTIVE-PATH-ORCH §10 Q5 | Suggestion timeout for slow connectivity? | S3-C5 | Standard timeout. Users can edit after auto-accept. |
| ONTOLOGY-VOCAB §13 Q1 | QID verification cadence? | S3-D1 | Configurable. Default: hourly new, daily all. |
| ONTOLOGY-VOCAB §13 Q2 | Hierarchy depth? | S3-D2 | 3 levels pre-loaded. Deeper queries via traversal. |
| ONTOLOGY-VOCAB §13 Q3 | Schema.org extensions? | S3-D3 | Use them. Verify programmatically. |
| ONTOLOGY-VOCAB §13 Q4 | OSM tag granularity? | S3-D4 | Allow compound tags. |
| ONTOLOGY-VOCAB §13 Q5 | Wikidata label storage? | S3-D5 | Store locally, update incrementally. |

---

## Still Open (Unresolved)

| # | Question | Context | Notes |
|---|---|---|---|
| OPEN-01 | **Vault encryption algorithm** | Catatan Saksi says "encrypted" but doesn't specify. AES-256-GCM? TweetNaCl? | Affects key management, mobile performance, multi-device sync. |
| OPEN-02 | **L2 tier badge leakage** | L2 hides author but shows tier badge. In small communities, badge could de-anonymize. | Should L2 also hide tier? Or accept risk? |
| OPEN-03 | **Vault multi-device sync** | Cloud-stored encryption keys vs. explicit export/import. | Affects architecture significantly. |
| OPEN-04 | **Offline behavior** | What works without internet? Cached feed? Offline compose queue? Vault access? | Critical for Indonesian rural areas. |
| OPEN-05 | **AI touchpoints 02-08 full specs** | Currently listed but not detailed. AI-02 (redaction), AI-05 (gaming), AI-06 (criteria), AI-07 (summarization), AI-08 (media). | Needed before dev handoff. |
| OPEN-06 | **Accessibility beyond color contrast** | Keyboard nav, screen readers, focus management, ARIA labels. | WCAG 2.1 AA compliance audit needed. |
| OPEN-07 | **Onboarding flow** | App install → first contribution. No spec exists. | Critical for user acquisition. |
| OPEN-08 | **Reputation system deep design** | Vouch slashing cascade depth, cross-community transfer, decay mechanics, inflation prevention. | PARKED for dedicated session (S3-E1). |
| OPEN-09 | **Configurable operational parameters** | Which system parameters should be admin-configurable? (raised by S3-D1) | Needs a configuration spec or convention. |
| OPEN-10 | **Content moderation appeals process** | User gets content hidden — how do they appeal? Jury selection? Timeline? | Related to OPEN-08 (reputation). |
| OPEN-11 | **Catatan Komunitas discovery UX detail spec** | Progressive disclosure decided (S3-B4) but no wireframe/prototype yet. | Needs UI spec or prototype. |
| ~~OPEN-12~~ | **SurrealDB schema & migration strategy** | ✅ RESOLVED Session 4. Migration `0013_ontology_schema.surql` created. Probe validated 13/15 patterns pass. Key finding: hierarchy queries must use reverse walk (`<-BROADER<-concept<-ABOUT<-note` from parent), not forward filtering. Vouch counts via `count(SELECT * FROM VOUCHES WHERE out = $parent.id)`. Wilson score stays app-layer. See `docs/research/ontology-probe-report.md`. | S4-P1 |

---

## Session 4 (2026-02-16, Part 4): SurrealDB Ontology Probe

### Probe Results (S4-P1)

| # | Finding | Status |
|---|---|---|
| S4-P1 | SurrealDB v3.0.0-beta.4 graph model validated. **15/15 patterns pass** after fixes. Migration `0013_ontology_schema.surql` created with SCHEMALESS tables + RELATE edge tables. | CONFIRMED |
| S4-P2 | **Hierarchy queries must use reverse walk.** Forward filtering (`WHERE ->ABOUT->concept->BROADER CONTAINS ...`) fails — traversal returns strings, not record refs. Working pattern: `SELECT <-BROADER<-concept<-ABOUT<-note FROM concept:parent`. Aligns with display-time domain derivation design. | CONFIRMED |
| S4-P3 | **Vouch/challenge counting works.** `count(SELECT * FROM VOUCHES WHERE out = $parent.id)` returns clean integers. Wilson score formula stays in Rust app layer — DB provides counts only. | CONFIRMED |
| S4-P4 | **SCHEMALESS + SCHEMAFULL coexist.** Ontology tables (SCHEMALESS) work alongside existing chat/event tables (SCHEMAFULL) in the same database. No migration conflict. | CONFIRMED |
| S4-P5 | **Edge metadata on RELATE works.** `predicate`, `object_value`, `object_unit` stored as fields on ABOUT edge records. Enables rich triple-to-graph mapping. | CONFIRMED |
| S4-P6 | **Cross-mode connectivity confirmed.** Single concept node (e.g. `concept:Q8068` flood) links notes, plans, and siaga via `<-ABOUT<-` reverse traversal. Unified discovery works. | CONFIRMED |
| S4-H1 | **3-level hierarchy chain works.** `nutrition→food→[egg,rice]→notes` via chained `<-BROADER<-concept`. S3-D2 (3-level pre-load) validated. | CONFIRMED |
| S4-H2 | **Reverse walk returns record IDs, not full objects.** Use `.content` field accessor for display, or two-step query for full records. Pattern for Rust repository layer. | CONFIRMED |
| S4-H3 | **INTERSECT not supported in SurrealDB v3 beta.** Multi-edge filtering uses `WHERE id IN (subquery)` pattern instead. | CONFIRMED |
| S4-H4 | **TTL feed query works.** `WHERE ttl_expires IS NONE OR ttl_expires > time::now()` correctly handles both permanent and timed notes. | CONFIRMED |
| S4-H5 | **SurrealQL Pattern Cheatsheet created and validated (C1–C10).** 8/10 pass, 2 known-failing forms documented (KF1: BEGIN/COMMIT unreliable in CLI, KF2: math::mean needs GROUP BY). 7 gotchas documented (G1–G7). Key C3 finding: `.*` accessor on reverse walk returns **full objects** — G4 is weaker than initially thought. Cheatsheet is now implementation-ready. See `docs/research/surrealql-pattern-cheatsheet.md`. | CONFIRMED |

---

## Superseded Decisions

| Original | What Changed | Superseded By |
|---|---|---|
| S1-02: Track hints as separate classification | Track hints now derived from Schema.org Action types in the RDF triple model | S3-MD2 |
| Original ONTOLOGY-VOCAB-v0.1.md: 4-layer stack (OSM → Wikidata → Schema.org → Ranah) with custom vocabulary | Replaced by pure RDF triples. No custom vocabulary. Each position constrained by source standard. | S3-MD1 |
| Original ONTOLOGY-VOCAB-v0.1.md: 8 custom Ranah values as separate classification | Ranah derived from Wikidata hierarchy at display time, not classified separately | S3-MD1, S3-MD2 |
| Original ONTOLOGY-VOCAB-v0.1.md: 8 custom Tujuan values for mode routing | Mode routing derived from Schema.org Action types | S3-MD2 |
| Original 5-tab nav: Beranda / Terlibat / Bantu / Notifikasi / Profil | Rearranged: Beranda / Catatan / Bantu / Notifikasi / Lainnya(☰). Terlibat and Profil move to hamburger. | S3-MD3 |
| Siaga: "zero moderation hold" (original A+3 design) | Siaga still broadcasts fast but goes through AI-00 triage first. Triage IS the pre-screen. | S3-MD6 |
| ENTRY-PATH-MATRIX §4.3: flat hashtags for discovery | Replaced by ontology-based concept tags (Wikidata QIDs via RDF triples) | S3-MD1, updated in spec |

---

*Document created: 2026-02-16*
*Companion to: DESIGN-CONTEXT.md, all spec files*
*Update this document whenever design decisions are made in any session.*

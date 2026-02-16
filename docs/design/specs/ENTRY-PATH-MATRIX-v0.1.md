# Gotong Royong — Entry Path Matrix v0.1

## Status
Proposed: 2026-02-16 | Revised: 2026-02-16
Purpose: Eagle-view of every path a warga's input can take, from first tap to resolution.

> **Key updates (Session 3):** Pre-screen for all modes including Siaga (S3-MD6). Mode routing via Schema.org Action types (S3-MD2). RDF triples replace flat tags (S3-MD1). Rahasia + ai_readable flag (S3-MD4). Ranking-based moderation (S3-MD8). All Section 10 open questions resolved — see DECISIONS-LOG.md.

---

## 1. The Four Modes

Everything a warga shares enters through AI-00 (Conversational Triage) and is routed to one of four modes.

```
                          ┌─────────────────────────────────────────┐
                          │            AI-00 Triage Chat            │
                          │  "Ceritakan apa yang Anda saksikan..."  │
                          └──────────┬──────────────────────────────┘
                                     │
                    ┌────────────────┼────────────────┬──────────────┐
                    ▼                ▼                ▼              ▼
             ┌──────────┐    ┌──────────┐    ┌──────────┐   ┌──────────┐
             │ KOMUNITAS│    │ CATATAN   │    │  SIAGA   │   │ CATATAN  │
             │          │    │  SAKSI    │    │          │   │ KOMUNITAS│
             │ Adaptive │    │  Vault    │    │ Emergency│   │  Notes   │
             │   Path   │    │ (private) │    │ Broadcast│   │ (public) │
             └──────────┘    └──────────┘    └──────────┘   └──────────┘
                 │                │                │              │
            Phases &         Sealed &          Instant &     Aggregated &
           Checkpoints      Encrypted         One-tap         Vouchable
```

---

## 2. Mode Comparison Matrix

| Dimension | Komunitas | Catatan Saksi | Siaga | Catatan Komunitas (NEW) |
|---|---|---|---|---|
| **Nature** | Collaborative action | Private record | Emergency alert | Public information |
| **Lifecycle** | Adaptive path (phases, checkpoints) | Sealed → Surfaced | None (instant) | Topic-based (aggregate, no phases) |
| **LLM role** | Proposes & evolves plan | Classifies & encrypts | Detects urgency | Classifies (ontology) & detects duplicates |
| **Who sees it** | Community (per Rahasia level) | Author + Wali only | All (broadcast) | Community (public) |
| **Duration** | Days to months | Permanent until surfaced | Minutes to hours | Hours to weeks (TTL) |
| **Governance** | Votes, PIC, checkpoints | Author-only | None | Vouch / Challenge / Update |
| **Tandang credit** | Full (Types A–E) | Zero | Zero | Minimal (Type A only) |
| **Can promote?** | — | Can surface to Komunitas | Can spawn Komunitas | Can promote to Komunitas |
| **AI touch points** | AI-00 through AI-09 | AI-00, AI-02, AI-08 | AI-00 | AI-00, AI-01, AI-03 (duplicate detection) |
| **Track hint** | Derived from Action type (5 labels) | None | None | None (concept pills for filtering) |
| **Rahasia** | L0–L3 | L2–L3 (encrypted) | L0 only (broadcast) | L0–L3 + `ai_readable` flag (S3-MD4) |
| **Pre-screen** | AI-00 triage | AI-00 triage | AI-00 triage (fast, focused) | AI-00 triage |

---

## 3. Komunitas — Adaptive Path Detail

### 3.1 Entry

AI-00 detects collaborative intent → proposes initial adaptive path plan.

### 3.2 Track Hints × Phase Patterns

Track hints are **derived from the Schema.org Action type** in the content's RDF triples (S3-MD2, S3-A1). They are UI display labels only — not separately classified. The Action type is produced by AI-00 as part of triple generation.

| Track Hint | Action Type | Spirit | Typical Phase Patterns | Example Case |
|---|---|---|---|---|
| **Tuntaskan** | `schema:RepairAction` | Fix a problem | Diskusi → Perencanaan → Pelaksanaan → Verifikasi | "Jalan berlubang di Blok C" |
| **Wujudkan** | `schema:CreateAction` | Build something | Diskusi → Perencanaan → Pelaksanaan → Perayaan | "Bangun taman bermain" |
| **Telusuri** | `schema:SearchAction` | Research/investigate | Diskusi → Investigasi → Verifikasi → Penemuan | "Kenapa air sumur keruh?" |
| **Rayakan** | `schema:AchieveAction` | Honor achievement | Validasi → Apresiasi → Dampak | "Pak Joko juara nasional" |
| **Musyawarah** | `schema:AssessAction` | Decide together | Pembahasan → Keputusan → Pelaksanaan → Tinjauan | "Iuran RT naik atau tidak?" |

### 3.3 Cross-Cutting Features (can appear in any path)

| Feature | Trigger | Phase/Checkpoint |
|---|---|---|
| **Galang** | Resource pooling needed | Appears as phases within any adaptive path |
| **Siarkan** | Outreach needed | Added as checkpoint or phase for public comms |
| **Rutin** | Recurring activity | Template saved, auto-spawned on schedule |

### 3.4 Research Cases (Telusuri Deep Dive)

The investigation pattern handles collaborative research. Key behaviors:

| Aspect | How It Works |
|---|---|
| **Short research** | 1 investigation phase, evidence board, quick findings |
| **Long-running** | Multiple investigation phases (LLM elaborates new phases as hypotheses evolve) |
| **Multi-location** | Branch support — parallel sub-investigations that merge at Penemuan |
| **Evidence governance** | Any participant submits evidence; PIC curates; AI-03 clusters similar evidence |
| **Findings exit** | Findings can spawn new Komunitas plan (LLM suggests, human confirms) |

---

## 4. Catatan Komunitas — Community Notes (NEW)

### 4.1 What It Is

A lightweight, lifecycle-free mode for sharing useful public facts that don't need collaborative action. Think community bulletin board, not project tracker.

### 4.2 How AI-00 Routes Here

AI-00 produces RDF triples for every input (S3-MD1, S3-A2). The `schema:potentialAction` triple determines routing. When the Action type is `schema:InformAction`, it routes to Catatan Komunitas.

AI-00 detects **informational intent without action need**:
- Statement of fact: "Telur Rp 28k di Pasar Minggu"
- Status update: "Jalan Melati ditutup hari ini"
- Observation: "Listrik padam sejak jam 8"
- No question, no request for help, no call to action

**Diagnostic signals:**
- Declarative sentence (not interrogative, not imperative)
- Contains observable data (price, location, time, status)
- No "butuh", "tolong", "bagaimana", "siapa bisa"
- Short input (typically 1-3 sentences)

**All modes go through AI-00 triage** (S3-MD6), including Siaga. The triage conversation IS the pre-screen — it guides users to provide structured, useful information. For Siaga, triage is fast and focused on extracting critical details (what, where, how urgent).

**Context bar state:** `informational` → "Ini terlihat seperti informasi untuk komunitas. Bagikan sebagai Catatan Komunitas?"

### 4.3 Individual Notes with Tags (No Forced Clustering)

Community notes are **individual posts**, not merged topics. Each note stands on its own. The system does NOT cluster or merge notes — that carries risk of wrong merges, hidden data, and loss of attribution.

Instead, discoverability comes from **ontology-based concept tags** (see `ONTOLOGY-VOCAB-v0.1.md`). The LLM classifies each note using standards it already knows — Schema.org types, Wikidata QIDs, and OSM tags — so no custom vocabulary needs to be loaded or maintained.

```
┌─────────────────────────────────────────────────┐
│ 📝 Catatan Komunitas                             │
│                                                  │
│ Telur Rp 28.000/kg di Pasar Minggu              │
│ — Ibu Sari, 2j lalu · 📍 Pasar Minggu          │
│ 🏷 telur · harga · pasar                         │
│    (backed by Wikidata QIDs: Q93189, Q132510)   │
│                                                  │
│ ✓ 3 konfirmasi · ✗ 1 koreksi                    │
│ ⏱ TTL: 23j lagi                                 │
└─────────────────────────────────────────────────┘
```

**How it works:**

| Step | Actor | What Happens |
|---|---|---|
| 1. Note submitted | Warga + AI-00 | AI-00 produces 2–5 RDF triples: (subject QID, Schema.org predicate, object). Includes `schema:potentialAction → InformAction` for routing. |
| 2. Triple refinement | AI-01 (optional) | Can validate/refine triples. Adds additional triples if needed. |
| 3. Graph storage | SurrealDB | Note stored with ABOUT, LOCATED_AT, HAS_ACTION edges to concept nodes. Measurements stored as time-series. |
| 4. Feed display | System | Notes appear in global feed with human-readable concept pills. Users tap concept to traverse graph. Progressive disclosure (S3-B4). |
| 5. Vouch/challenge | Community | Other warga confirm, correct, or add context. Ranking via Wilson score weighted by voucher tier (S3-MD8). |

**Why no auto-clustering:** Merging carries risk — wrong merges hide data, correct data gets buried under a bad cluster, attribution is diluted. Ontology-based concept nodes give richer discoverability without the merge risk — notes sharing the same Wikidata QID are naturally related via graph edges, but each note remains an individual record. Users who want a "topic view" filter by concept (e.g., tap "telur" to see all egg-related notes across locations).

**Visibility:** Notes are visible globally (not scoped to one RT/RW). A note from Kelurahan X is visible to everyone. The author's community is shown as a source tag for context ("📍 Pasar Minggu"), and users filter by location tags to find relevant nearby data.

### 4.4 Privacy & AI-Readable Flag (S3-MD4)

Community notes support Rahasia levels L0–L3. New `ai_readable` boolean per note:

| Rahasia + ai_readable | Who Sees Content | AI Can Read? | Use Case |
|---|---|---|---|
| L0 (Terbuka) + ai_readable=true | Everyone | Yes | Default: public fact sharing |
| L1 (Terbatas) + ai_readable=true | Verified community members | Yes | Sensitive but useful data |
| L2 (Rahasia) + ai_readable=true | Author only | Yes (anonymized) | Anonymous report — AI learns patterns |
| L2 (Rahasia) + ai_readable=false | Author only | No | Truly private observation |
| L3 (Sangat Rahasia) | Author + Wali only | No | Use Catatan Saksi instead |

**AI pattern aggregation (S3-MD5):** When `ai_readable=true`, AI-04 can aggregate anonymous reports to detect patterns and warn the community: "Beberapa warga telah melaporkan masalah serupa di area ini." No names, no accusations — just signal. This enables community safety warnings without public bad-actor tagging.

### 4.5 Cross-Community Moderation (S3-MD8)

Notes are globally visible and moderated by ranking, not community ownership:

- **Ranking factors:** Vouch count (Wilson score weighted by voucher tier), recency, location proximity, challenge count
- **Flagging:** >5 Lapor → AI-04 pre-screen → hidden pending review by any Pilar+ tier user from any community
- **No jurisdictional moderators** — ranking is emergent and scales without governance overhead
- **Users can pin notes** to their personal view

### 4.6 Vouch & Challenge

Instead of governance votes, community notes use lightweight reactions:

| Reaction | Meaning | Effect |
|---|---|---|
| **✓ Konfirmasi** | "I can verify this" | Adds vouch to that data point. Type A credit (binary). |
| **✗ Koreksi** | "This is wrong, here's the correct data" | Creates competing data point within same topic. Shows both. |
| **📝 Tambah** | "I have more info" | Adds new data point to the topic. |
| **⚠ Lapor** | "This is false/harmful" | Standard moderation (AI-04). |

**No formal voting.** Trust is visible through vouch counts, author tier badges, and recency.

### 4.7 Time-to-Live (TTL)

Community notes aren't permanent. TTL is driven by `temporal_class` from the RDF triples (S3-A3):

| Temporal Class | Default TTL | Derived From | Examples |
|---|---|---|---|
| **ephemeral** | 24 hours | `schema:price`, traffic, weather predicates | Prices, power outage, traffic |
| **periodic** | Until next occurrence | `schema:startDate` with recurrence | Posyandu schedule, weekly market |
| **durable** | 7–30 days | Infrastructure/status predicates | Road closure, construction |
| **permanent** | No expiry | Achievement, facility predicates | Facility exists, person achieved |

AI-00 infers `temporal_class` from the triple predicates. Author can override within bounds (S3-A3).

After TTL expires:
- Topic moves to "Arsip" (archive) section of feed
- No longer appears in main feed
- Still searchable
- Data available for cross-case learning

### 4.8 Promotion to Komunitas

When a community note reveals something that needs action, it can be promoted:

| Trigger | Example | What Happens |
|---|---|---|
| **Manual** | User taps "Ini perlu ditindak" on a note | AI-00 opens with note context pre-filled. Routes to Komunitas. |
| **AI-suggested** | Multiple corrections on same topic, heated discussion | System suggests: "Topik ini tampaknya perlu koordinasi lebih. Buat rencana?" |
| **Threshold** | >5 challenges on same data point | System flags for community attention |

Promoted notes link back to the original topic (bidirectional reference).

### 4.9 What Community Notes Are NOT

- Not a chat room (no threaded conversation — use Komunitas for that)
- Not a wiki (no collaborative editing of the same text)
- Not social media (no likes, shares, follower counts)
- Not Siaga (not for emergencies)
- Not a replacement for Komunitas (if action is needed, promote it)

---

## 5. Decision Matrix: Where Does My Input Go?

This is the eagle-view decision tree AI-00 follows:

| User says... | Action Type (from triple) | Mode | Track Label |
|---|---|---|---|
| "Jalan berlubang di depan rumah saya" | `schema:RepairAction` | Komunitas | Tuntaskan |
| "Kita bangun pos ronda baru yuk" | `schema:CreateAction` | Komunitas | Wujudkan |
| "Kenapa air PDAM sering mati?" | `schema:SearchAction` | Komunitas | Telusuri |
| "Pak RT kita menang lomba!" | `schema:AchieveAction` | Komunitas | Rayakan |
| "Iuran sampah mau dinaikkan, setuju?" | `schema:AssessAction` | Komunitas | Musyawarah |
| "Saya lihat ada yang buang limbah ke sungai" | (user-initiated) | Catatan Saksi | — |
| "KEBAKARAN di Blok D!!" | `schema:AlertAction` | Siaga | — |
| "Telur Rp 28k di Pasar Minggu" | `schema:InformAction` | Catatan Komunitas | — |
| "Jalan Melati ditutup hari ini" | `schema:InformAction` | Catatan Komunitas | — |
| "Jadwal posyandu bulan ini Senin 10 Feb" | `schema:InformAction` | Catatan Komunitas | — |
| "Listrik padam sejak jam 8 tadi" | `schema:InformAction` | Catatan Komunitas | — |

### 5.1 Ambiguous Cases

| User says... | Could be... | AI-00 strategy |
|---|---|---|
| "Harga sembako naik terus" | Note (fact) OR Komunitas (let's do something about it) | AI-00 probes: "Apakah Anda ingin berbagi informasi ini, atau ingin komunitas mengambil tindakan?" |
| "Listrik padam lagi" | Note (info) OR Siaga (if dangerous) | AI-00 checks urgency markers. If calm → Note. If urgent → probes for Siaga. |
| "Ada tikus besar di selokan" | Note (observation) OR Komunitas (pest control needed) | AI-00 probes: "Apakah ini untuk informasi saja, atau perlu ditangani bersama?" |

---

## 6. Full System Graph

```
                            ┌──────────────┐
                            │    Warga     │
                            │  (Resident)  │
                            └──────┬───────┘
                                   │
                                   ▼
                            ┌──────────────┐
                            │   AI-00      │
                            │   Triage     │
                            └──────┬───────┘
                                   │
              ┌────────────┬───────┼───────┬────────────┐
              ▼            ▼       │       ▼            ▼
        ┌──────────┐ ┌─────────┐  │ ┌──────────┐ ┌──────────┐
        │Komunitas │ │Catatan  │  │ │  Siaga   │ │ Catatan  │
        │          │ │ Saksi   │  │ │          │ │Komunitas │
        └────┬─────┘ └────┬────┘  │ └────┬─────┘ └────┬─────┘
             │            │       │      │             │
             ▼            ▼       │      ▼             ▼
     ┌───────────────┐  Sealed   │   Broadcast   ┌──────────┐
     │ Adaptive Path │  Archive  │   (instant)   │Individual│
     │               │           │               │  Notes   │
     │ ┌───────────┐ │           │               │          │
     │ │  Phases   │ │           │               │ Tags     │
     │ │  ├─ Plan  │ │           │               │ Vouches  │
     │ │  ├─ Exec  │ │     ┌─────┘               │ TTL      │
     │ │  ├─ Verify│ │     │                     └────┬─────┘
     │ │  └─ Done  │ │     │                          │
     │ └───────────┘ │     │                          │
     │               │     │                     ┌────▼─────┐
     │ Cross-cutting:│     │                     │ Promote? │
     │ Galang        │     │                     │ → Komun. │
     │ Siarkan       │◄────┼─────────────────────┘          │
     │ Rutin         │     │                                │
     └───────┬───────┘     │                                │
             │             │                                │
             ▼             ▼                                │
     ┌───────────────────────┐                              │
     │    Tandang Credit     │◄─────────────────────────────┘
     │  (Type A–E scoring)   │   (Type A only for notes)
     └───────────────────────┘
```

---

## 7. Credit Mapping Across All Modes

| Mode | Credit Types Earned | Examples |
|---|---|---|
| **Komunitas** | A, B, C, D, E (all) | Task completion, sustained effort, peer review, quality rating, vouching |
| **Catatan Saksi** | None | Private vault — zero reputation impact |
| **Siaga** | None | Emergency — no gamification |
| **Catatan Komunitas** | A only | Binary: "shared useful info" (1 credit per note), "confirmed data" (1 credit per vouch) |

### 7.1 Why Minimal Credit for Notes

The design principle is "addiction to contribution, not consumption." Full credit for typing "telur 28k" would inflate reputation and incentivize low-effort spam. Type A (binary, small) rewards the act of sharing without creating a farming vector.

Promotion to Komunitas unlocks full credit earning for the follow-up action.

---

## 8. Relationship to Other Specs

| Spec | What It Covers | What This Matrix Adds |
|---|---|---|
| ADAPTIVE-PATH-SPEC-v0.1.md | Plan data model | Mode routing context (why a plan exists) |
| ADAPTIVE-PATH-ORCHESTRATION-v0.1.md | Plan lifecycle flow | The full entry picture (4 modes, not just Komunitas) |
| AI-SPEC-v0.2.md | AI touch point details | Decision matrix for AI-00 routing |
| UI-UX-SPEC-v0.5.md | Screen layouts | New Catatan Komunitas screens needed |
| DESIGN-CONTEXT.md | Terminology locks | Catatan Komunitas terminology to add |

---

## 9. Resolved Decisions

| # | Question | Decision | Rationale |
|---|---|---|---|
| D1 | **Topic clustering** | No forced clustering. Individual notes with tags. | Clustering carries merge risk. Tags give same discoverability without write-side merge. Users filter by tag for topic views. |
| D2 | **Cross-community visibility** | Notes are globally visible. Location tags for sorting. | Facts like prices and road closures are useful beyond one RT. Author's community shown as source context. |
| D3 | **Credit cap** | Yes. Max 5 notes/day earn Type A credit. | Prevents low-effort spam farming. |
| D4 | **Research → Notes bridge** | Author chooses. Option presented at Telusuri completion: "Publish as Catatan Komunitas?" | Not all research findings are worth public posting. Author decides. |

## 10. Resolved Open Questions

All previously open questions have been resolved in Session 3. See DECISIONS-LOG.md for full rationale.

| # | Question | Resolution | Decision Ref |
|---|---|---|---|
| 1 | Catatan Komunitas moderation: post-first or pre-screen? | Pre-screen. AI-00 triage IS the pre-screen for ALL modes, including Siaga. | S3-MD6, S3-B1 |
| 2 | Community warning system: bad-actor tagging? | No public tagging. AI aggregates anonymous `ai_readable` reports for pattern-based warnings. | S3-MD5, S3-B5 |
| 3 | Cross-community governance: who moderates? | Ranking-based (Wilson score). Pilar+ from any community can review flagged notes. | S3-MD8, S3-B6 |
| 4 | Anonymous notes: Rahasia levels? | Yes. L0–L3 apply. New `ai_readable` flag enables AI learning from anonymous notes. | S3-MD4, S3-B2 |

---

*Document prepared: 2026-02-16 | Revised: 2026-02-16*
*Companion to: ADAPTIVE-PATH-ORCHESTRATION-v0.1.md, ONTOLOGY-VOCAB-v0.1.md, DECISIONS-LOG.md, DESIGN-CONTEXT.md*

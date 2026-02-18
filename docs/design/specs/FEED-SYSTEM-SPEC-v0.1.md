# Gotong Royong — Feed System Spec v0.1

## Status
Draft: 2026-02-19 | Author: Design Session
Purpose: Formal specification of the Pulse feed system — event stream architecture, card anatomy, entity follow graph, and gamification mapping.

---

## §1 — Overview

The Pulse feed is an **event stream**, not a case list. It shows community activity from entities you follow and witnesses you participate in.

The feed merges 3 layers of activity into a single chronological stream. Each witness appears as **one card** showing its latest significant event — not a wall of individual posts. Active witnesses surface to the top; resolved or dormant ones sink.

This design reflects a core product principle: a witness is community property, not a personal broadcast. The feed surfaces *collective progress*, not individual posts.

---

## §2 — Feed Layers (3 Sources)

| Layer | Source | How Populated |
|-------|--------|---------------|
| **📌 Ikutan** | Entities you follow (places, topics, groups, people, institutions) | Manual follow |
| **🔔 Terlibat** | Witnesses you participate in | Auto when you join |
| **🌏 Sekitar** | Trending witnesses from nearby areas | Algorithm (proximity + popularity) |

**Filter tabs:**

```
[Semua]  [📌 Diikuti]  [🔔 Terlibat]  [🌏 Sekitar]
```

**Decision: No auto-subscribe to RT/RW.** Users are not automatically subscribed to their administrative area. Suggestions are shown during onboarding, but following is always an explicit action. This reduces notification fatigue and respects agency.

---

## §3 — Feed Items: Event-Based Cards

### 3.1 Card Model

- **One card per witness** — the latest significant event is the headline.
- Older events on the same witness collapse behind a "+N aktivitas" affordance.
- Cards are sorted by **latest event timestamp** — active witnesses rise, dormant ones sink.
- A witness card disappears from the feed only if it falls out of all 3 layers (unfollowed entity, no longer participating, no longer trending nearby).

### 3.2 Event Types

~8 canonical event types drive the entire feed. Each event has an emoji, a verb template, and a defined snippet.

| Event Type | Emoji | Verb Template | Snippet Content |
|------------|-------|---------------|-----------------|
| `created` | 📢 | "melaporkan" | First 2 lines of witness description |
| `joined` | 🙋 | "bergabung sebagai [role]" | Role name + current member count |
| `checkpoint` | 📍 | "mencapai fase [name]" | Phase name + progress summary |
| `vote_opened` | 🗳️ | "membuka pemungutan suara" | Vote question + deadline |
| `evidence` | 📎 | "menambah bukti" | Evidence title or media thumbnail label |
| `resolved` | ✅ | "diselesaikan" | Resolution summary (1 line) |
| `galang_milestone` | 💰 | "galang dana mencapai [X]%" | Amount raised vs. target |
| `community_note` | 📝 | "menambah catatan komunitas" | First line of the note |

### 3.3 Card Anatomy

```
┌──────────────────────────────────────────────────┐
│ [4px track color left-border]                     │
│                                                   │
│ [repost header — if repost frame present]         │
│ "👤 X melaporkan" · faded text                    │
│                                                   │
│ [event headline row]                              │
│ [emoji] [verb] · [timeAgo]     [urgency badge]   │
│                                                   │
│ [witness title — font-semibold]                   │
│ [snippet — 2-line clamp, text-muted]              │
│                                                   │
│ [meta row]                                        │
│ [AvatarGroup] [member count] [collapsed "+N"]     │
│                                                   │
│ [entity pills row — tappable Ikutan tags]         │
└──────────────────────────────────────────────────┘
```

**Left border color** maps to witness track (e.g., infrastructure = blue, health = red). This is the same track system defined in UI-UX-SPEC.

**Entity pills** are tappable — they navigate to the entity detail page and offer a [+ Ikuti] action, forming the primary discovery loop for the follow graph.

**Avatar group** shows the 3 most recent active participants. Tapping opens the full participant list with role badges.

---

## §4 — Post Ownership

**Nobody "owns" a witness post.** The witness is community property from the moment it is created.

- **Pelapor (reporter)** always receives permanent credit: displayed as "Dilaporkan oleh X" in a faded subheading on every card.
- **All roles** are visible via the avatar stack and role badges (Pelapor, Relawan, Koordinator, Saksi Ahli).
- **Contribution credit** is tracked per-action via the Tandang system (see Whitepaper). The feed displays *proof of contribution* (role badges, avatar position), not ownership.

This design prevents the social dynamic of a witness "belonging" to its reporter, which would discourage community takeover when the reporter is unavailable.

---

## §5 — Role Reposts (Brag Rights)

When you participate in a witness, your followers can see it framed through **your role**. This is a social proof mechanism, not a reshare of the raw report.

### 5.1 Repost Defaults

| Event | Repost Default | What Followers See |
|-------|----------------|---------------------|
| Report (pelapor) | **ON** | "X melaporkan: [title]" |
| Join as relawan | **ON** | "X bergabung sebagai Relawan" |
| Submit evidence | **ON** | "X menambah bukti" |
| Vote | **OFF** (privacy) | — |
| Galang contribution | **OFF** (financial privacy) | — |
| Witness resolved | **AUTO for all participants** | "X berkontribusi — SELESAI 🎉" |
| Become koordinator | **ON** | "X memimpin sebagai Koordinator" |

### 5.2 User Control

Each witness has a per-witness toggle in the participant settings:

```
📢 Tampilkan di feed pengikut saya  [ON / OFF]
```

**Hard rule:** Witnesses at Rahasia level L2+ (Rahasia) are **never reposted**, regardless of toggle state. The repost system only applies to L0 (public) and L1 (community-visible) witnesses.

---

## §6 — Followable Entities (Ikutan)

Anything that is a node in the community knowledge graph can be followed. The follow graph is the spine of the Ikutan layer.

| Entity Type | Icon | Example | Source |
|-------------|------|---------|--------|
| Lingkungan (Place) | 📍 | "RT 05 Menteng" | OSM / Wikidata from AI-00 RDF triples |
| Topik (Concept) | 🏷️ | "Infrastruktur", "Harga Sembako" | Wikidata QID from AI-00 triples |
| Kelompok (Group) | 👥 | "Karang Taruna RT 05" | Emerged from repeated mentions or user-created |
| Lembaga (Institution) | 🏢 | "SD Negeri 3 Menteng" | Wikidata / OSM match |
| Warga (Person) | 👤 | "Pak Budi" | User profiles |

### 6.1 Entity Discovery Loop

1. Feed card displays entity pills (place, topic, group) extracted by AI-00 from the witness.
2. User taps a pill → **entity detail page** showing recent activity and follower count.
3. Entity detail page shows [+ Ikuti] button.
4. After following, new witnesses tagged with that entity appear in the Ikutan layer.

### 6.2 Entity Emergence

Entities are **not pre-seeded** by an admin. They emerge organically from AI-00 RDF triple extraction as witnesses are created. A "Kelompok" node appears in the graph when an organization is mentioned enough times across separate witnesses, or when a user explicitly creates one.

Users **can** explicitly create a Kelompok entity (for a community group that does not yet appear in witnesses), but this is optional — organic emergence is the primary path.

---

## §7 — Urgency Badges

Urgency badges appear in the top-right of the card headline row. At most one badge is shown per card (highest priority wins).

| Badge | When | Intended Feel |
|-------|------|---------------|
| 🔴 **BARU** | Witness created < 1 hour ago | Something just happened nearby |
| 🟡 **VOTING** | A vote is open with an active deadline | Your voice matters right now |
| 🟢 **SELESAI** | Witness resolved in the last 24 hours | We did it together |
| 🔥 **RAMAI** | > 10 events on this witness in the past 24 hours | This is getting attention |

Badge priority (if multiple apply): VOTING > BARU > RAMAI > SELESAI.

---

## §8 — Onboarding Suggestions

New users with no Ikutan entities see a suggestion block at the top of the feed based on their registered location.

```
💡 Disarankan untuk Anda

📍 RT 05 Menteng          — 23 aktivitas aktif
📍 Kelurahan Menteng       — 87 aktivitas
🏷️ Infrastruktur           — 12 terkini

[+ Ikuti semua]   atau tap satu per satu
```

This block disappears once the user follows 3+ entities. Suggestions are re-surfaced in the Sekitar tab if the user's Ikutan layer is later pruned below 3 entities.

Suggestions are derived from the user's location data (set during registration) and the most active entities in that administrative area. No behavioral tracking is used at this stage.

---

## §9 — Octalysis Mapping

The feed is designed to activate all seven Core Drives from the Octalysis framework.

| Core Drive | How the Feed Activates It |
|-----------|---------------------------|
| **CD1: Epic Meaning & Calling** | Follow community-scale topics (infrastructure, health, education); see your local area's collective progress on a shared timeline |
| **CD2: Development & Accomplishment** | Progress badges per witness phase, resolution trophies, contribution wall showing your Tandang history |
| **CD3: Empowerment of Creativity** | Curate YOUR feed via Ikutan; every card has a contextual CTA that lets you choose your role |
| **CD4: Ownership & Possession** | "Ikutan Saya" — a personalized view that is uniquely yours; role reposts make your contribution identity visible |
| **CD5: Social Influence & Relatedness** | Avatar stacks showing familiar community members; social proof reposts when people you follow join a witness |
| **CD6: Scarcity & Impatience** | Voting deadlines, "sisa 2 hari" countdowns, galang dana targets with a progress bar |
| **CD7: Unpredictability & Curiosity** | Heterogeneous card types create a varied scroll experience; Sekitar algorithm surfaces witnesses you would not have found via Ikutan |

---

## §10 — Relationship to Other Specs

| Spec | Relationship to This Document |
|------|-------------------------------|
| **ENTRY-PATH-MATRIX-v0.1** | Feed is the display layer for activity from all 4 entry modes (Komunitas, Catatan Saksi surface events, Siaga, Catatan Komunitas) |
| **ADAPTIVE-PATH-ORCHESTRATION-v0.1** | Checkpoint and phase-transition events from the adaptive path flow into the feed as `checkpoint` event cards |
| **Whitepaper (Tandang)** | Tandang credit attribution is shown as role badges on repost frames; the contribution wall (§11) will surface Tandang totals |
| **UI-UX-SPEC-v0.5** | New components required: FeedCard, EntityPill, AvatarGroup, UrgencyBadge, EntityDetailPage, RepostHeader |
| **ONTOLOGY-VOCAB-v0.1** | Entity types in §6 map directly to the ontology node types; RDF triples from AI-00 populate the follow graph |

---

## §11 — Future Work (Out of Scope for v0.1)

The following are acknowledged design areas that are intentionally deferred:

- **Profile "contribution wall"** — visualizing a user's Tandang history across all witnesses
- **Entity detail page** — full page for a Lingkungan, Topik, Kelompok, or Lembaga node with activity timeline and follower list
- **Notification system** — push/in-app notifications tied to Ikutan and Terlibat events
- **"Sekitar" recommendation algorithm** — proximity weighting, popularity decay function, diversity injection
- **Feed pagination and infinite scroll strategy** — cursor-based pagination, prefetch window, stale card eviction
- **Muted entities** — the ability to follow an entity but suppress specific event types
- **Cross-witness search** — full-text search across the event stream

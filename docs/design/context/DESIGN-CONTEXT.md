# Gotong Royong — Design Context (Session Handoff)

> **READ THIS FIRST** in every new session. This is the single source of truth for all locked design decisions. Last updated after B5 lock. Phase B complete.

---

## What Is This Project?

**Gotong Royong** is a witness-first community coordination platform. Users ("saksi"/witnesses) share what they observe — problems, ideas, questions, good news, or proposals — and the community acts on them through structured tracks.

**Design mood: TANAH** — warm, earthy, Nunito font, rounded shapes, Indonesian token names.

---

## Terminology — USE THESE EXACT TERMS

### 5 Tracks (Komunitas)

| Track | Meaning | Seed | Lifecycle |
|---|---|---|---|
| **Tuntaskan** | Resolve a problem | Keresahan | Keresahan → Bahas → Rancang → Garap → Periksa → Tuntas (+ Dampak opsional) |
| **Wujudkan** | Create from idea | Gagasan | Gagasan → Bahas → Rancang → [Galang] → Garap → Rayakan → Tuntas (+ Dampak opsional) |
| **Telusuri** | Explore a question | Pertanyaan | Pertanyaan → Dugaan → Uji → Temuan → Tuntas |
| **Rayakan** | Celebrate achievement | Kabar Baik | Kabar Baik → Sahkan → Apresiasi → Tuntas (+ Dampak opsional) |
| **Musyawarah** | Deliberate a decision | Usul | Usul → Bahas → Putuskan → Jalankan → [Tinjau] → Tuntas |

**Tuntaskan = reactive** (fix a problem). **Wujudkan = proactive** (build something new).

### Cross-Cutting Features

| Feature | Meaning |
|---|---|
| **Galang** | Mobilize/pool resources. Sub-lifecycle: Sasaran → Kumpul → Salurkan → Lapor |
| **Siarkan** | Broadcast/share. No lifecycle — capability-based. |
| **Rutin** | Recurring/routine activity |
| **Rahasia** | Confidentiality overlay. L0 (Terbuka) → L1 (Terbatas) → L2 (Rahasia) → L3 (Sangat Rahasia) |

### Three Saksi Modes (decided in this project)

| Mode | Nature | Key Properties |
|---|---|---|
| **Komunitas** | Community / collaborative | 5 tracks, public, reputation impacts, lifecycle |
| **Catatan Saksi** | Private witness vault | Encrypted, timestamped, author-only, zero Tandang impact, optional Wali (trustee), can surface later |
| **Siaga** | Emergency broadcast | Instant, no lifecycle, auto-location, 112/BPBD link |

### AI Touch Points

| ID | Name | Function |
|---|---|---|
| **AI-00** | Conversational Triage | NEW. Conversational chat before classification. Routes to Komunitas / Catatan Saksi / Siaga. Detects multi-entity splits. |
| **AI-01** | Track & Seed Classification | Track & seed classification (≥0.80 auto, 0.50-0.79 suggest, <0.50 manual) |
| **AI-02** | Redaction LLM | |
| **AI-03** | Duplicate Detection | |
| **AI-04** | Content Moderation | |
| **AI-05** | Gaming Pattern Detection | |
| **AI-06** | Criteria & Task Suggestion | |
| **AI-07** | Discussion Summarization | Discussion summaries, especially Musyawarah |
| **AI-08** | Sensitive Media Detection & Redaction | |
| **AI-09** | Credit Accreditation | Tracks all GR actions, calculates Tandang credit per contribution type (A–E), proposes credit distributions at Tuntas, nudges for quality ratings (Type D) and vouch suggestions (Type E). Uses Suggest-Don't-Overwrite for proposed distributions. |

### Reputation Tiers (Tandang Markov Credential Engine)

| Tier | Name | Badge | Percentile | Description |
|---|---|---|---|---|
| 4 | **Kunci** (Keystone) | ◆◆◆◆ | ≥99th | Top 1%, community anchors |
| 3 | **Pilar** (Pillar) | ◆◆◆◇ | ≥90th | Top 10%, trusted leaders |
| 2 | **Kontributor** (Contributor) | ◆◆◇◇ | ≥60th | Active participants |
| 1 | **Pemula** (Novice) | ◆◇◇◇ | <60th | New/learning members |
| — | **Bayangan** (Shadow) | ○ | Post-slash | Reputation suspended |

### Three Reputation Axes (I / C / J)

| Axis | Name | Decay | Meaning |
|---|---|---|---|
| **I** | Integrity (Integritas) | None — permanent | Civic participation, trustworthiness, vouch staking |
| **C** | Competence (Kompetensi) | 90-day half-life | Skill demonstration, contribution quality |
| **J** | Judgment (Pertimbangan) | Slow decay | Validation/verification accuracy |

### 5 Contribution Types (A–E)

| Type | Name | Scoring | Example GR Actions |
|---|---|---|---|
| **A** | Binary Verification (Verifikasi Biner) | Did/didn't — pass/fail | Task completion (Garap), voting (Putuskan), seeding |
| **B** | Time-Weighted (Bobot Waktu) | Effort over duration | Building in Rancang, contributing to Galang |
| **C** | Peer Consensus (Konsensus Rekan) | Group validates quality | Validation (Sahkan), verification (Periksa/Tinjau) |
| **D** | Quality Spectrum (Spektrum Kualitas) | Rated on quality scale | Discussion quality (Bahas), proposal writing (Usul) |
| **E** | Stake-Weighted (Jaminan) | Reputation risked/staked | Vouching, guaranteeing. Only type where credit can be *subtracted* via slash cascade. |

### GR Action → Tandang Credit Mapping

| GR Action | Where | Score | Type | Detail |
|---|---|---|---|---|
| Submit a seed | AI-00 → any track | **I+** | A · Binary | Civic initiation — you started something |
| Discuss in Bahas | All tracks with Bahas | **C** | D · Quality Spectrum | Discussion quality rated by AI-08 Sensitive Media Detection & Redaction + peers |
| Contribute to Rancang | Tuntaskan, Wujudkan | **C** | B · Time-Weighted | Sustained planning effort |
| Complete task in Garap | Tuntaskan, Wujudkan | **C** | A · Binary | Task done or not done |
| Validate in Sahkan | Rayakan | **J** | C · Peer Consensus | Endorsement accuracy |
| Vote in Putuskan | Musyawarah | **I+** | A · Binary | Civic participation — you showed up |
| Verify in Periksa/Tinjau | Tuntaskan, Musyawarah | **J** | C · Peer Consensus | Verification accuracy |
| Contribute to Galang | Wujudkan (Galang sub-flow) | **C** | B · Time-Weighted | Resource contribution |
| Vouch for someone | Profile | **I** (stake) | E · Stake-Weighted | Risk proportional to vouchee behavior. Slash cascade if vouchee penalized. |
| Propose hypothesis | Telusuri (Dugaan) | **C** | D · Quality Spectrum | Hypothesis quality rated by outcome |
| Collect evidence | Telusuri (Uji) | **C** | B · Time-Weighted | Research effort over time |

### ESCO-ID Skill Taxonomy
**ESCO** (European Skills, Competences, Qualifications and Occupations) — Indonesian localization (**ESCO-ID**). ~13,000 skills in hierarchical taxonomy. Used for:
1. **Tagging seeds** — AI-00 auto-tags skill needs during triage
2. **User skill declaration** — users pick skills from ESCO-ID picker on their profile
3. **Bantu matching** — connecting seed skill needs to user skill declarations
4. **Tandang validation** — peer-validated contributions earn validated skill badges

Two display states:
- **Dinyatakan** (Declared) = ○ outline pill — self-reported
- **Tervalidasi** (Validated) = ● filled pill — Tandang-earned through contributions

### Weather System (Global Difficulty Floor / GDF)

Cerah (☀️ 0-5%) → Berawan (🌤️ 5-10%) → Hujan (🌧️ 10-15%) → Badai (⛈️ 15-20%)

Affects all Competence (C) scoring. Higher difficulty = bonus multiplier on contributions. Published in community feed as "Cuaca Komunitas" widget.

---

## Locked Design Tokens

### Colors (CSS Variables)

```css
/* Core */
--c-tanah-gelap: #3E2723;   /* Text, headings */
--c-tanah:       #5D4037;   /* Secondary text */
--c-kayu:        #8D6E63;   /* Captions, meta */
--c-pasir:       #A1887F;   /* Placeholder, disabled */
--c-batu:        #D7CCC8;   /* Borders, dividers */
--c-kapas:       #F5EDE3;   /* Input backgrounds */
--c-susu:        #FFFBF5;   /* Card backgrounds */

/* Action (Api) */
--c-api:         #C05621;   /* Primary action */
--c-api-terang:  #D2691E;   /* Hover */
--c-api-dalam:   #A0461A;   /* Pressed */
--c-bara:        #FFF3E0;   /* Soft bg, highlight */

/* Semantic */
--c-berhasil:    #2E7D32;   --c-berhasil-l: #E8F5E9;
--c-peringatan:  #E65100;   --c-peringatan-l: #FFF3E0;
--c-bahaya:      #C62828;   --c-bahaya-l: #FFEBEE;
--c-keterangan:  #4E342E;   --c-keterangan-l: #EFEBE9;

/* Track Accents */
--c-tuntaskan:   #C05621;   /* soft: #FFF3E0, muted: #FFECD2 */
--c-wujudkan:    #2E7D32;   /* soft: #E8F5E9, muted: #C8E6C9 */
--c-telusuri:    #6A1B9A;   /* soft: #F3E5F5, muted: #E1BEE7 */
--c-rayakan:     #F57F17;   /* soft: #FFF8E1, muted: #FFECB3 */
--c-musyawarah:  #4E342E;   /* soft: #EFEBE9, muted: #D7CCC8 */

/* Special: Vault */
--c-vault:         #263238;
--c-vault-surface: #37474F;
/* Full set: #263238, #37474F, #546E7A, #78909C, #B0BEC5, #ECEFF1 */

/* Special: Siaga */
--c-siaga:        #B71C1C;
--c-siaga-accent: #D32F2F;
/* Full set: #B71C1C, #D32F2F, #FF5252, #FFEBEE, #FFCDD2 */

/* Rahasia */
/* L3: #3E2723, L2: #5D4037, L1: #8D6E63, L0: none */
```

### Typography

```css
/* Nunito — Major Third scale (1.250) */
--fs-display: 32px;  --fs-h1: 26px;  --fs-h2: 20px;  --fs-h3: 16px;
--fs-body: 14px;  --fs-small: 12px;  --fs-caption: 11px;  --fs-micro: 9px;

--fw-regular: 400;  --fw-semi: 600;  --fw-bold: 700;  --fw-extra: 800;
--lh-tight: 1.2;  --lh-normal: 1.5;  --lh-relaxed: 1.7;
```

### Spacing (4px base grid)

```css
--sp-1: 4px;  --sp-2: 8px;  --sp-3: 12px;  --sp-4: 16px;  --sp-5: 20px;
--sp-6: 24px;  --sp-8: 32px;  --sp-10: 40px;  --sp-12: 48px;  --sp-16: 64px;
```

### Radii & Shadows

```css
--r-sm: 6px;  --r-md: 10px;  --r-lg: 14px;  --r-xl: 20px;  --r-full: 9999px;

--shadow-sm: 0 1px 4px rgba(62,39,35,0.06);
--shadow-md: 0 2px 12px rgba(62,39,35,0.08);
--shadow-lg: 0 4px 24px rgba(62,39,35,0.12);
--shadow-xl: 0 8px 40px rgba(62,39,35,0.16);
```

---

## Locked Design Decisions

### Track Color = Identity, Breadcrumb = State
- Track accent color stays constant throughout lifecycle (identity)
- Breadcrumb stepper shows current stage using actual Indonesian names
- States never change the track color

### Single Entry Point → Conversational AI-00 → Route
- ONE chat page ("Bagikan")
- AI-00 greets user conversationally, not a blank textarea
- AI probes with follow-up questions if needed
- Quick-reply chips speed up the conversation
- Triage result appears in **context bar** above keyboard (not in chat bubbles)

### Context Bar (Morphing Strip Above Keyboard)
AI response includes XML metadata parsed into the context bar:

| Bar State | XML | Visual |
|---|---|---|
| **listening** | `<triage status="listening"/>` | Pulsing dot + "Mendengarkan..." + "Pilih sendiri ›" |
| **probing** | `<triage status="probing" confidence="0.35" needs="..."/>` | Signal progress bar + "Perlu info lebih" + "Pilih sendiri ›" |
| **leaning** | `<triage status="leaning" track="tuntaskan" confidence="0.62"/>` | Tappable pill: "Mungkin: Tuntaskan 62% →" + "Lainnya" |
| **ready** | `<triage status="ready" track="tuntaskan" confidence="0.87" alt="..." seed="..."/>` | Full card with badge, title, confidence, alt route, Setuju/Ubah |
| **vault-ready** | `<triage status="ready" mode="vault" sensitivity="high"/>` | Steel-grey tinted bar |
| **siaga-ready** | `<triage status="ready" mode="siaga" urgency="critical" location="auto:..."/>` | Pulsing red bar, auto-location, 112 link |
| **split-ready** | `<triage status="ready" mode="split" entities="siaga,vault" linkability="warning"/>` | Split items + linkability warning |
| **manual** | `<triage status="uncertain" fallback="manual"/>` | 3×2 grid: 5 tracks + Catatan Saksi |

User can tap "Pilih sendiri" at any time → jumps to manual grid.

### Triage → Card Handoff
When user confirms track ("Setuju — Tuntaskan"), the AI-00 triage conversation **carries over** as the first messages in the card's Bahas Diskusi tab. The witness story isn't lost or repeated — it becomes the opening context for community discussion.

### Emergency (Siaga) Skips Follow-up
AI detects emergency language → immediately shows siaga-ready bar. No probing.

### Multi-Entity Split
AI can detect mixed content → splits into separate entities → warns about linkability between public and private content.

### Catatan Saksi Vault UI (A+2)
- **Dark vault palette**: `--v-deep:#263238 → --v-wash:#ECEFF1` (6-step blue-grey scale)
- **Seal bar**: bottom strip that mirrors the context bar concept from A+1. Morphs between:
  - **Unsealed**: edit icon + "Belum tersegel — masih bisa diedit" + "Simpan & Segel" button
  - **Sealed**: green check + timestamp + truncated SHA-256 hash + "Tambah Wali" / "Terbitkan" actions
- **5-state lifecycle**:
  1. **Menyimpan**: compose view with text pre-filled from AI-00 conversation, attachment tools (📷📎📍🎙️), privacy reminder
  2. **Tersegel**: sealed entry card with full timestamp (WIB), SHA-256 hash, encrypted badge, "Tandang tidak terpengaruh" note, attachment thumbnails
  3. **Wali**: trustee designation with search, explicit permission list (can: read, surface with consent; cannot: edit, share without permission), selected Wali shows avatar + tier badge
  4. **Terbitkan**: surface to community — orange warning card with 3 consequences (visibility, Tandang impact, irreversibility), track picker chips, Rahasia L2 toggle, deliberate publish button
  5. **Pola**: AI pattern detection — gentle notification counting similar entries, safety resource links (Telepon Sahabat 119, Komnas Perempuan, LPSK), non-pushy dismiss
- **Key principle**: Vault entries have ZERO Tandang impact while sealed. Only when published (Terbitkan) does reputation get affected.
- **Wali permissions are strict**: read and surface-with-consent only. No edit, no share without permission.

### Siaga Broadcast UI (A+3)
- **Siaga red palette**: `--s-deep:#B71C1C; --s-accent:#D32F2F; --s-bright:#FF5252; --s-wash:#FFEBEE; --s-soft:#FFCDD2`
- **Design principle**: Speed over ceremony. Minimal screens, one-tap actions, no lifecycle complexity.
- **Broadcast bar**: bottom strip following same morphing pattern as context bar (A+1) and seal bar (A+2):
  - **Composing**: red background, "Siarkan Sekarang" button
  - **Active**: pulsing red border, reach counter, "Pembaruan" + "Selesai" actions
  - **Resolved**: green background, "Kembali ke Beranda"
- **4-state lifecycle**:
  1. **Kirim**: compose with pre-filled AI-00 text, 5 emergency type chips (Bencana/Kecelakaan/Keamanan/Medis/Lainnya), auto-location card with map preview + "✓ Auto" badge, 112 link always visible in app bar
  2. **Aktif**: shimmer-animated live card, real-time stats (terjangkau/melihat/merespons), quick update text input, chronological event timeline
  3. **Respons**: responder cards with avatar, status message, distance, ETA; quick-respond buttons ("Saya datang" / "Bantu dari jauh" / "Hubungi layanan"); growing timeline from both author and responders
  4. **Selesai**: deliberate confirmation dialog ("Ya, situasi sudah aman" / "Belum"), summary card (duration, responders, location, services contacted), thank-you: "Komunitas lebih aman karena Anda"
- **Key principles**:
  - ZERO Tandang impact (same as vault)
  - 112 emergency link always accessible (app bar + dedicated card)
  - Auto-location is default (user can override)
  - Resolution requires deliberate confirmation — can't accidentally end a siaga
  - All responders get notified when resolved

### Bottom Bar Pattern (Cross-Cutting)
All three A+ screens share the same bottom-bar concept with different skins:
| Screen | Bar Name | States |
|---|---|---|
| A+1 Triage | Context Bar | listening → probing → leaning → ready (+ vault/siaga/split/manual) |
| A+2 Vault | Seal Bar | unsealed (editable) → sealed (locked + actions) |
| A+3 Siaga | Broadcast Bar | composing → active (pulsing) → resolved (green) |

### Dual-Tab Pattern (B1 — Cross-Cutting Component)
Every stage that has both structured content and conversation gets two swipeable tabs. Bookend stages (seeds + Tuntas) have no tabs.
- **Tab bar**: sits below app bar, track-colored underline on active tab, `‹ geser ›` swipe hint
- **Left tab**: primary action for that stage (varies)
- **Right tab**: secondary (usually conversation)
- **Tab mapping per stage**:
  - Bahas: 💬 Diskusi | 📋 Rangkuman
  - Rancang: 📋 Papan GR | 💬 Koordinasi
  - Garap: ✅ Progres | 💬 Koordinasi
  - Periksa: 📊 Laporan | 💬 Tanggapan
  - (Other tracks adapt naming but follow same dual-tab structure)
- **Notification dot**: red 6px circle on tab when unread content exists on the other tab
- **WhatsApp-style chat**: `.chat-bubble.other` (left, white) and `.chat-bubble.self` (right, track-soft). Names, timestamps, emoji reactions, date separators.
- **AI inline cards in chat**: `.ai-chat-card` — dashed border, centered, italic — appears mid-conversation when AI-07 surfaces summaries or cross-references the other tab.

### LLM ↔ UI Architecture (B1 — Canonical Pattern)

#### Core Invariants (universal, never broken)
1. **Two-layer data model**: every card has conversation layer (chat tab) + structured layer (summary/papan tab)
2. **AI never auto-applies**: always suggests via diff card, human decides (Suggest-Don't-Overwrite)
3. **Human edit = lock**: when human edits an AI item, source flips from "ai" to "human", AI stops touching it
4. **Additive-first**: AI can add items, suggest status changes, draft content. AI cannot remove, uncheck, delete, or overwrite human-modified content.

#### 7 Block Primitives
The structured layer is composed of **block primitives**. The LLM decides which blocks to use based on the specific problem — guided by stage context, not rigid per-stage schemas. The UI renders any combination generically by block type.

| Block | UI renders as | AI rules |
|---|---|---|
| **`list`** | Checklist, table, timeline, gallery (varies by item fields) | Additive. Source-tagged per item. Status-changeable with citation. Supports nesting (items with children). |
| **`document`** | Rich text with tracked-changes sections | AI drafts, human edits sections. Source-tagged per section. Diff card shows paragraph-level changes. |
| **`form`** | Labeled input fields | AI may suggest per field. `protected` fields (financial, identity) = AI completely hands-off. Source-tagged per field. |
| **`computed`** | Read-only display (progress bar, status indicator) | System-derived from other blocks. Nobody edits directly. |
| **`display`** | Presentational card (recognition, celebration) | One-way render from community/system actions. No edit. |
| **`vote`** | Voting interface with tallies | System-authoritative. Tallies computed by system, not AI. |
| **`reference`** | Linked card preview / cross-card pointer | Links to other cards, sub-flows, or track-change origins. |

`list` is the workhorse: checklists, hypotheses, experiments, evidence, contributions, attestors, budget items, media — anything "collection of similar things." UI rendering adapts to item fields: items with dates → timeline, with amounts → table, with checkboxes → checklist, with media → gallery.

#### LLM-Driven Composition
No rigid per-stage schemas. Instead:
- LLM receives: stage context + block catalog + current structured state + new conversation
- LLM decides: which blocks this specific problem needs, fills them appropriately
- Human refines: edit directly (✏️) or ask LLM to restructure (🤖 Bantu edit)
- Iteration: each cycle, LLM respects existing human edits (source locking)

#### Source Tags (3 types)
- **`"ai"`**: LLM-generated. Can be overwritten by next LLM pass or human edit.
- **`"human"`**: Human-created or human-edited. AI stops touching. Locked.
- **`"system"`**: System-computed (vote tallies, readiness status, progress %). Nobody edits.

#### 4 Trigger Modes
| Mode | When | AI touch points | Output destination |
|---|---|---|---|
| **Manual** | 🔄 Perbarui button (shows "12 pesan baru" badge) | AI-07 summarization | Diff card on structured tab |
| **Milestone** | Keyword/pattern detection at breakpoints | AI-05 Gaming Pattern Detection | Transition suggestion |
| **Time-triggered** | Scheduled interval | AI-04 Content Moderation (inactivity), AI-06 Criteria & Task Suggestion (30-day) | Alert in conversation tab (NOT direct structured edit) |
| **Passive** | Continuous monitoring | AI-08 Sensitive Media Detection & Redaction, AI-03 Duplicate Detection | Badge/indicator only |

#### Diff Card (Suggest-Don't-Overwrite)
When LLM produces updates, they appear as a diff card with citations from conversation:
- **For lists**: "Added 2 items, checked off 1" with evidence quotes
- **For documents**: Tracked-changes style with highlighted additions/edits
- **For forms**: Field-by-field comparison
- **Actions**: [Terapkan Semua] | [Tinjau Satu-satu] | dismiss
- **Edit flow**: Each section has ✏️ for inline edit + 🤖 Bantu edit for natural language instructions to AI

#### Cross-Card Operations
When a sub-flow (e.g., Musyawarah) produces output for a parent card, it arrives as a **suggestion diff card on the parent** — same Suggest-Don't-Overwrite. No automatic cross-card writes.

#### Protected Fields
Fields marked `protected: true` (financial amounts, identity data) are completely excluded from AI access. Only humans can fill, edit, or view protected fields. Applies especially to Galang financial data.

#### Credit Accreditation Pattern (C5 — Cross-Cutting Mechanic)

Every GR action earns Tandang credit. The accreditation mechanism varies by contribution type — no single "credit screen." Credit is woven into the existing card lifecycle.

**Accreditation by Type:**

| Type | Accreditor | AI-09 Role | Human Role | Timing |
|---|---|---|---|---|
| **A · Binary** | SYSTEM (auto) | Observes action completion | Action = proof. No confirmation needed. | Instant — task checked, vote cast, seed submitted |
| **B · Time-Weighted** | SYSTEM (auto) | Calculates duration from activity logs | Participation = proof | Accumulated — tallied over time |
| **C · Peer Consensus** | PEERS (human collective) | Tracks consensus progress, nudges when threshold nears | Humans vote/endorse — consensus IS the accreditation | When enough peers validate (e.g., Sahkan threshold) |
| **D · Quality Spectrum** | AI-PROPOSED → HUMAN-CONFIRMED | AI-08 Sensitive Media Detection & Redaction + AI-07 Discussion Summarization → proposes quality rating | PIC or peers accept/override AI rating | At milestone moments (end of Bahas, Tuntas) |
| **E · Stake-Weighted** | HUMAN (self-initiated) | Suggests vouch candidates based on collaboration history | Deliberate "Jaminkan" action with explicit risk warning | When human chooses to vouch |

**Credit Flow:**
1. **Silent tracking** — System logs every action (who, what, when, how long). No UI, no interruption. The activity log IS the credit ledger.
2. **Instant feedback** — For Type A/B, credit toast slides up immediately after action. For Type C, toast appears when consensus threshold is met. For Type D, toast appears after PIC confirms AI-proposed rating. For Type E, toast appears after vouch confirmation.
3. **AI nudges in chat** — AI-09 surfaces contextual messages as `.ai-chat-card` in the Diskusi tab (same dashed-border inline cards as AI-07):
   - After quality discussion: *"💡 Diskusi berkualitas — kontribusi Anda dicatat (Tipe D · Kompetensi)"*
   - Approaching validation threshold: *"📊 12 dari 15 validator sudah mengkonfirmasi"*
   - Vouch suggestion: *"🤝 Anda sudah 3× berkolaborasi dengan Bu Maya. Pertimbangkan menjaminkannya?"*
   - Stall + decay reminder: *"⏳ Belum ada aktivitas 14 hari. Kompetensi kontributor aktif akan luruh."*
4. **Tuntas credit summary** — When a card reaches Tuntas, AI-09 proposes the full Kontribusi distribution as a **diff card** (same Suggest-Don't-Overwrite pattern). PIC reviews: [Terapkan] · [Tinjau Satu-satu] · [Tolak]. Once confirmed, the Kontribusi panel locks as a `computed` block (source: system).
5. **Dispute mechanism** — For auto-awarded credits (Type A/B), any participant can flag "Kredit ini tidak tepat" → triggers peer review → AI-09 mediates resolution.

**Key Principles:**
- **No separate credit management screen** — credit is earned inside existing activities, not in a separate flow.
- **Same source-tag model** — credit accreditation follows `system` (auto Type A/B), `ai` (proposed Type D), `human` (confirmed Type C/D/E).
- **Suggest-Don't-Overwrite for credit** — AI-09 proposes, PIC confirms. Same pattern as AI-07 content summaries.
- **Vault & Siaga = ZERO credit** — sealed Catatan Saksi entries and Siaga broadcasts do not earn or affect Tandang scores. Only published community actions count.
- **GDF multiplier applied automatically** — system applies the current GDF weather multiplier to all Competence (C) credit awards. Higher community difficulty = bonus on contributions.

### Seed Card Universal Anatomy (B0)
- **Card structure** (top to bottom):
  1. **Track Strip**: 4px left border in track accent color — identity marker, never changes
  2. **Header**: seed badge (track icon + seed name) + optional Rahasia badge (L1/L2/L3) + title (max 2 lines) + author row (avatar + name + tier + timestamp)
  3. **Body**: content text (3-line clamp + "...selengkapnya") + media thumbnails + location tag + optional PIC row + optional Dampak row
  4. **Stepper**: horizontal breadcrumb with actual Indonesian stage names. Current stage = track-colored filled dot + bold label. Done = muted. Future = empty circle.
  5. **Footer**: 💬 comments · 👥 supporters · ⏱ time-in-stage · AI badge (rightmost)
- **AI badges** (footer, right-aligned):
  - 🤖 **Classified** (green): auto-classified with confidence, e.g., "🤖 Tuntaskan · 92%"
  - 🤖 **Suggested** (orange): below threshold, needs confirmation, e.g., "🤖 Wujudkan? · 74%"
  - ⚠ **Stalled** (red): AI-04 Content Moderation, e.g., "⚠ Macet 48j"
  - 🌱 **Dampak** (green): AI-06 Criteria & Task Suggestion measured impact
  - 📝 **AI-07**: discussion summary available
  - ⚠ **Duplicate** (orange): AI-03 Duplicate Detection detected
- **Rahasia overlay levels**:
  - **L0 (Terbuka)**: no badge, full visibility
  - **L1 (Terbatas)**: badge shown, full content visible, verified-members-only access
  - **L2 (Rahasia)**: author → "Anonim" (grey avatar, italic), content visible, tier still shown
  - **L3 (Sangat Rahasia)**: title → "Judul disembunyikan", author → "Tersembunyi", content → hatched redaction block
- **Track color = identity**: left strip + seed badge + stepper current dot all use the same accent color. Card shape never changes across tracks.

### App Navigation (5-Tab Bottom Bar)
Bottom navigation with 5 equal-width tabs. No floating action button — the entry point is the Beranda header.

| Tab | Icon | Label | Purpose |
|---|---|---|---|
| 🏠 | home | **Beranda** | Community feed — all seeds, Community Pulse, track filter tabs |
| 📋 | clipboard | **Terlibat** | Seeds the user is involved in (as author, PIC, contributor, voter) |
| 🤝 | handshake | **Bantu** | Skill-matched opportunities — seeds that need your declared/validated skills |
| 🔔 | bell | **Notifikasi** | Grouped notifications (mentions, stage changes, credit earned, stall alerts) |
| 👤 | person | **Profil** | CV Hidup — the Tandang profile IS the profile page |

**Key changes from earlier draft:**
- Search is NOT a nav tab — it's a 🔍 icon in the app header (opens full-screen overlay)
- "Butuh Bantuan Anda" promoted to its own nav tab as **Bantu** (shorter, action-oriented)
- Terlibat is a separate page, not a toggle on Beranda

### App Header (Sticky, All Pages)
Every page shares the same app header:
```
┌──────────────────────────────────────────┐
│  [scope ▼]    Gotong Royong    [🔍] [+]  │
└──────────────────────────────────────────┘
```
- **Scope selector** (left): shows current area, e.g., "RT 05 ▼". Tap opens area picker sheet.
- **Search icon** 🔍 (right): opens full-screen search overlay with track filters, skill tags, recency.
- **[+] Compose** (right): opens AI-00 triage conversation (Bagikan flow).

### Scope / Area Hierarchy
Users belong to an administrative area. Seeds are scoped to areas. The scope selector in the app header filters all feeds.

| Level | Indonesian | Example | Typical Size |
|---|---|---|---|
| 7 | **Nasional** | Indonesia | 275M |
| 6 | **Provinsi** | Jawa Barat | ~50M |
| 5 | **Kota / Kabupaten** | Kota Depok | ~2M |
| 4 | **Kecamatan** | Cimanggis | ~200K |
| 3 | **Kelurahan / Desa** | Tugu | ~15K |
| 2 | **RW** | RW 03 | ~1K |
| 1 | **RT** | RT 05 | ~150 |

- **Default scope**: user's registered RT (most local).
- **Upward browsing**: user can widen scope to see seeds from parent areas (RW → Kelurahan → etc.)
- **Seeds inherit area**: when created, seed gets author's current scope. Can be manually adjusted.
- **Reputation is area-aware**: Tandang scores can be viewed at different area levels (local hero vs. city-level contributor).
- **Area-level aggregation**: Community Pulse and GDF Weather can be shown per area level.

### ESCO Skill Tags on Seeds
Seeds are tagged with **skill needs** from the ESCO-ID (European Skills, Competences, Qualifications and Occupations — Indonesian localization) taxonomy. This creates a bridge between what communities need and what individuals can offer.

- **When**: AI-00 auto-tags skill needs during triage (e.g., a plumbing complaint gets tagged "perbaikan pipa", "pemeliharaan bangunan")
- **Source**: ESCO-ID taxonomy provides ~13,000 skills in standardized hierarchy. We use skill labels, not occupation labels.
- **Display**: skill tags appear as pills on seed cards (below body text, above stepper)
- **Editable**: author or PIC can add/remove skill tags after creation
- **AI confidence**: AI-suggested tags show confidence (same 3-band system from C2). Author confirms or dismisses.
- **Bridge to Bantu**: skill tags on seeds are what the Bantu tab matches against user skills.

### Two-Layer Skill System (Declared vs. Validated)
Users have skills from two sources. The distinction is visible everywhere skills appear.

| Layer | Source | Badge | Pill Style | How Earned |
|---|---|---|---|---|
| **Dinyatakan** (Declared) | Self-reported by user | ○ outline | `border: 1.5px solid; background: transparent` | User picks from ESCO-ID list + optional free text input |
| **Tervalidasi** (Validated) | Earned through Tandang | ● filled | `background: filled; color: white` | Contribute to seeds tagged with that skill → peers validate → Tandang records |

**Skill lifecycle:**
1. **Declare** → user picks skills from ESCO-ID picker (searchable, hierarchical) + can type free-text skills
2. **Match** → Bantu tab shows seeds that need your declared skills
3. **Contribute** → user helps on matched seeds (Garap tasks, Bahas discussions, etc.)
4. **Validate** → peers confirm quality of contribution (Type C/D accreditation)
5. **CV Hidup grows** → validated skill ● replaces declared ○ on profile. Tandang Competence (C) score increases.

**Rules:**
- Free-text skills remain ○ until ESCO-ID mapping is confirmed or peer-validated.
- Validated skills decay with Competence (C) axis — 90-day half-life. If user stops contributing in that skill area, ● fades back to ○.
- Both layers visible on profile. Validated always ranked above declared.
- Skill tags on seeds use the same ESCO-ID taxonomy, enabling exact matching.

### Bantu Tab (Skill-Matched Opportunities Feed)
The Bantu tab is a personalized feed of seeds that need the user's skills. It answers: "Where can I help right now?"

- **Matching algorithm**: compares user's declared + validated ESCO skills against seed skill tags
- **Ranking**: validated skills (●) weighted higher than declared (○). Closer area scope weighted higher. Seeds with fewer responders prioritized.
- **Card display**: same seed card anatomy (B0) but with a skill-match indicator: "🤝 Cocok: Perbaikan Pipa ●" showing which of your skills matched
- **Empty state**: if no matches, show ESCO skill picker prompt ("Tambah keahlian untuk melihat peluang")
- **Scope-filtered**: respects the current scope selector (app header)

### Terlibat Tab (Involvement Feed)
The Terlibat tab shows seeds where the user has a stake. It answers: "What needs my attention?"

- **Includes seeds where user is**: author, PIC, contributor (any Garap task), voter (Putuskan), validator (Sahkan/Periksa), discussant (Bahas with recent message)
- **Sorting**: by urgency — (1) seeds needing YOUR action as PIC, (2) seeds with unread activity, (3) seeds nearing completion, (4) recently active
- **Card enhancements**: progress ring overlay showing % completion, role badge ("PIC" / "Kontributor" / "Penulis"), unread indicator dot
- **Streak counter**: top of feed shows contribution streak ("🔥 12 hari berturut-turut")

### Community Pulse Bar (Beranda Header)
Merges GDF Weather widget (from C5) with live community stats into a single compact bar at the top of Beranda, below the app header.

```
┌────────────────────────────────────────┐
│  ☀️ Cerah · 14 aktif · 3 baru · 1 vote │
└────────────────────────────────────────┘
```

- **Weather emoji**: from GDF (Cerah ☀️ / Berawan 🌤️ / Hujan 🌧️ / Badai ⛈️)
- **Live stats**: count of active seeds, new seeds today, pending votes/reviews
- **Tap to expand**: opens full Community Pulse detail card (seed breakdown by track, top contributors this week, area health)
- **Area-scoped**: stats reflect current scope selector setting

### Action-Weighted Feed Ordering (Beranda)
Beranda feed is NOT chronological. It's ordered by community utility:

| Priority | What | Why |
|---|---|---|
| 1 | Seeds needing YOUR action (PIC tasks, pending votes) | Direct responsibility |
| 2 | Seeds nearing stage completion (≥80% progress) | Momentum — help finish |
| 3 | New seeds (< 24h old) | Fresh opportunities |
| 4 | Active seeds with recent discussion | Community energy |
| 5 | Completed/celebrated seeds | Positive reinforcement |

- **Track tabs**: horizontal scroll tabs (Semua / Tuntaskan / Wujudkan / Telusuri / Rayakan / Musyawarah) filter the weighted feed by track
- **Pull-to-refresh**: updates feed and Community Pulse stats
- **Infinite scroll**: loads more seeds as user scrolls down

### CV Hidup as Profile (Profil Tab)
The Profil tab IS the Tandang profile — the "CV Hidup" (living resume). No separate profile page.

- **Identity section**: avatar, name, area (RT/RW/Kelurahan), tier badge (◆◇ system), member-since date
- **I/C/J Radar**: triangular radar chart showing Integrity / Competence / Judgment scores
- **Skills section**: two groups — Tervalidasi ● (filled pills, sorted by recency) and Dinyatakan ○ (outline pills). "Tambah Keahlian" button opens ESCO picker.
- **Kontribusi timeline**: chronological list of completed seeds where user contributed, with track color strips and role badges
- **Vouch section**: who vouched for this user (vouch chain), who this user vouched for. "Jaminkan" button for stake-weighted vouching (Type E).
- **Impact metrics**: total seeds completed, total Bantu matches responded, community rank in area
- **QR Code**: shareable CV Hidup link — scannable for external sharing (job applications, community introductions)
- **Decay indicator**: for Competence (C) axis, shows time since last contribution per skill. Nudge: "Keahlian X luruh dalam 30 hari — bantu komunitas untuk mempertahankan"

### Positive Engagement Levers
Design patterns that encourage continued positive contribution (not attention-seeking):

| Lever | Location | Mechanic |
|---|---|---|
| **Streak counter** | Terlibat tab header | Consecutive days with any contribution. "🔥 12 hari" |
| **Progress rings** | Terlibat seed cards | Visual % completion overlay on card thumbnail |
| **Skill match notification** | Bantu tab + push | "Ada yang butuh keahlian Anda: Perbaikan Pipa" |
| **Community Pulse** | Beranda header | Live stats create sense of active community |
| **Dampak celebration** | Feed + Profil | AI-06 triggered impact measurement appears as celebratory card |
| **Weekly digest** | Notifikasi | "Minggu ini: 3 kontribusi, 2 skill tervalidasi, +15 Kompetensi" |
| **Tier progression** | Profil | Clear path from Pemula → Kontributor → Pilar → Kunci |

**Principle**: addiction to contribution, not consumption. Every engagement lever is tied to an ACTION the user takes, not content the user views.

---

## File Map

> **Directory structure**: All paths below are relative to `docs/`. Context docs live in `design/context/`, prototypes in `design/prototypes/`, specs in `design/specs/`, and archived files in `design/archive/`.

| File | Status | Contents |
|---|---|---|
| **Context & Tracking** |||
| `design/context/DESIGN-CONTEXT.md` | Living | **THIS FILE** — session handoff reference |
| `design/context/DESIGN-SEQUENCE.md` | Living | Design checklist (what's locked, what's next) |
| `design/context/TRACK-MAP.md` | Locked | ASCII flowcharts for all 5 track lifecycles |
| `design/context/REVIEW-FIXES.md` | Living | Review amendments (14 parts A–N): contradiction fixes, design decisions, GDF computation, Vault Security Contract, Rahasia×Tandang matrix, Wali consent, moderator dashboard, offline/error patterns, account management, block/mute, push notifications, Siaga abuse prevention, accessibility contract, deep link landing pages. |
| `design/context/REVIEW-PROMPT.md` | Reference | Completeness review prompt for external reviewers |
| `design/context/TANDANG-GAP-PROMPTS.md` | Reference | Implementation prompts for Tandang gap items |
| **HTML Prototypes** (`design/prototypes/`) |||
| `design/prototypes/A1-mood-vibe.html` | Locked | 3 moods × 3 phases. **Tanah selected.** |
| `design/prototypes/A2-color-palette.html` | Locked | Full Tanah token set, WCAG AA verified |
| `design/prototypes/A3-typography-spacing.html` | Locked | Type scale, spacing, radii, shadows, live card sample |
| `design/prototypes/A4-core-components.html` | Locked | Atoms: buttons, badges, inputs, avatars, pills, indicators, progress, toggles |
| `design/prototypes/A+1-triage-screen.html` | Locked (v4) | Conversational AI-00 + morphing context bar, 8 states |
| `design/prototypes/A+2-catatan-saksi.html` | Locked | Dark vault UI, 5-state lifecycle, seal bar |
| `design/prototypes/A+3-siaga-broadcast.html` | Locked | Siaga red UI, 4-state lifecycle, broadcast bar |
| `design/prototypes/B0-seed-card.html` | Locked | Universal card anatomy, 5 views |
| `design/prototypes/B1-tuntaskan-card.html` | Locked (v3) | Tuntaskan lifecycle: 6 states, dual-tab, Papan GR, Kontribusi |
| `design/prototypes/B2-wujudkan-card.html` | Locked (v1) | Wujudkan lifecycle: 7 states, milestones, Galang, Rayakan |
| `design/prototypes/B3-telusuri-card.html` | Locked (v1) | Telusuri lifecycle: 5 states, hypotheses, evidence board, Temuan |
| `design/prototypes/B4-rayakan-card.html` | Locked (v1) | Rayakan lifecycle: 4 states + post-Tuntas Dampak panel |
| `design/prototypes/B5-musyawarah-card.html` | Locked (v1) | Musyawarah lifecycle: 6 states, voting, Ketetapan |
| `design/prototypes/C1-rahasia-overlays.html` | Locked (v1) | Rahasia 4-level privacy (L0–L3), 5 views |
| `design/prototypes/C2-ai-surface-states.html` | Locked (v1) | AI badges, confidence bands, diff cards, moderation, duplicate detection |
| `design/prototypes/C3-navigation-feed.html` | Locked (v2) | 5-tab nav, feed, search, scope picker, CV Hidup, 7 views |
| `design/prototypes/C4-catatan-saksi-feed.html` | Locked (v1) | Vault timeline, sealed entries, Wali management, 5 views |
| `design/prototypes/C5-tandang-credit.html` | Locked (v1) | Tandang credit: tiers, toasts, vouch, GDF weather, 5 views |
| `design/prototypes/C6-share-sheet.html` | Locked (v1) | External sharing: share sheet, OG previews, Siaga broadcast, invite, 6 views |
| `design/prototypes/D2-style-guide.html` | Locked (v0.1) | Live component reference, 40/40 verified, all tokens + 12 component families |
| **Formal Specs** (`design/specs/`) |||
| `design/specs/DESIGN-DNA-v0.1.md` | Locked (v0.1) | Design DNA: 9 sections + appendix. Also split into `design/specs/design-dna/01–10` chapter files. |
| `design/specs/AI-SPEC-v0.2.md` | Locked (v0.2) | AI Layer: 10 touch points (AI-00–AI-09). Also split into `design/specs/ai-spec/01–21` chapter files. |
| `design/specs/UI-UX-SPEC-v0.5.md` | Locked (v0.5) | UI/UX: 29 sections. Also split into `design/specs/ui-ux-spec/01–30` chapter files. |
| **Archive** (`design/archive/`) |||
| `design/archive/AI-SPEC-v0.1.docx` | Superseded | Original AI spec (8 touch points). Replaced by AI-SPEC-v0.2.md. |
| `design/archive/UI-UX-SPEC-v0.4.docx` | Superseded | Original UI/UX spec (22 sections). Replaced by UI-UX-SPEC-v0.5.md. |
| `design/archive/UI-UX-CONCEPT.md` | Superseded | Early concept doc. |
| **Other Docs** |||
| `whitepaper/WHITEPAPER-v0.1-DRAFT.md` | Baseline | Platform spec (pre-dates AI-00, Catatan Saksi, Siaga decisions) |
| `api/` | Living | Webhook spec, authentication, error handling, event payloads |
| `architecture/` | Living | System overview, data flow, integration architecture |
| `database/` | Living | Schema requirements, migrations |
| `deployment/` | Living | Infrastructure, monitoring, security checklist |
| `development/` | Living | Setup guide, local development, testing |
| `por-evidence/` | Living | Proof of Reality evidence format, storage, validation rules |

---

## What's Next

**Phase D complete.** All steps locked: D1 (Design DNA) + D2 (Style Guide) + D3 (AI-SPEC v0.2 + UI-UX-SPEC v0.5).

**All phases locked**: A (4 steps) + A+ (3 steps) + B (6 steps) + C (6 steps) + D (3 steps) = **22 design steps total.** C6 (Share Sheet) added post-D3 as cross-cutting addition.

**Review amendments applied**: REVIEW-FIXES.md resolves 8 critical gaps, 8 important gaps, 10 contradictions, and 4 nice-to-haves from external completeness review. Score target: 7/10 → 9/10.

See `DESIGN-SEQUENCE.md` for full checklist.

---

*This file should be updated every time a new design step is locked. Last updated after REVIEW-FIXES.md.*

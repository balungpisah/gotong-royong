# Gotong Royong — Review Fixes & Amendments

> **Purpose**: This document resolves all findings from the external completeness review (7/10 score). It fixes contradictions, records design decisions, and adds missing operational specs. All items reference the original review findings.
>
> **Status**: Amends locked specs. Treat as authoritative where it contradicts earlier documents.
>
> **Date**: 2026-02-14

---

## Part A — Contradiction Fixes (applied directly to files)

These have been applied directly to the source files. Listed here for audit trail.

| # | Issue | Resolution | Files Changed |
|---|---|---|---|
| A1 | AI-02/03 ID confusion | AI-SPEC-v0.2.md is canonical: AI-02=Redaction LLM, AI-03=Duplicate Detection. Updated all tables/badges. | DESIGN-CONTEXT.md |
| A2 | Rayakan color #E65100 vs #F57F17 | #F57F17 is canonical (distinct from semantic --c-peringatan #E65100). | DESIGN-CONTEXT.md, A4-core-components.html, A+1-triage-screen.html, B0-seed-card.html |
| A3 | Vault palette collision in C6 | C6 now uses canonical vault palette (--v-deep:#263238). | C6-share-sheet.html |
| A4 | Missing prototype refs in AI-SPEC | A+3-duplicate-card.html → C2 (Duplikat view), A+4-moderation-hold.html → C2 (Moderasi view). | AI-SPEC-v0.2.md |
| A5 | GAP_ANALYSIS.md missing | File archived. Reference updated in file map. | DESIGN-CONTEXT.md |

---

## Part B — Color Token Namespace (Alias Map)

### Problem
Two naming conventions coexist: `--c-*` (A-phase, early files) and `--t-*` (B/C-phase, later files) for track colors. Also `-l` vs `-lembut` for soft variants.

### Decision
**Both are valid aliases.** The canonical names going forward are `--t-*` for tracks (used in B0+ files). Early `--c-*` files are NOT broken — they render correctly because the hex values are identical.

### Canonical Token Map

```css
/* Track tokens — CANONICAL (use --t-* in new code) */
--t-tuntaskan:    #C05621;  --t-tuntaskan-soft: #FFF3E0;  --t-tuntaskan-muted: #FFECD2;
--t-wujudkan:     #2E7D32;  --t-wujudkan-soft:  #E8F5E9;  --t-wujudkan-muted:  #C8E6C9;
--t-telusuri:     #6A1B9A;  --t-telusuri-soft:  #F3E5F5;  --t-telusuri-muted:  #E1BEE7;
--t-rayakan:      #F57F17;  --t-rayakan-soft:   #FFF8E1;  --t-rayakan-muted:   #FFECB3;
--t-musyawarah:   #4E342E;  --t-musyawarah-soft:#EFEBE9;  --t-musyawarah-muted:#D7CCC8;

/* Aliases — DEPRECATED but still functional in legacy files */
--c-tuntaskan → --t-tuntaskan    (same hex)
--c-wujudkan  → --t-wujudkan     (same hex)
--c-telusuri  → --t-telusuri     (same hex)
--c-rayakan   → --t-rayakan      (same hex, NOW FIXED to #F57F17)
--c-musyawarah→ --t-musyawarah   (same hex)

/* Semantic soft suffix — CANONICAL is -l (matches Tanah palette pattern) */
--c-berhasil-l  (correct)    not --c-berhasil-lembut
--c-peringatan-l (correct)   not --c-peringatan-lembut
```

### Implementation Rule
When building the CSS token file for production, define BOTH `--c-*` and `--t-*` pointing to the same values. This costs nothing and prevents refactoring churn.

---

## Part C — Design Decisions

### C1. Dampak — Post-Tuntas Measurement Panel, Not a Stage

**Decision**: Dampak is NOT a lifecycle stage. It is an **optional post-Tuntas measurement panel** triggered by AI-06 after a configurable period (default: 30 days).

**Why**: The card prototypes (B1, B2, B4) never show Dampak in the stepper breadcrumb. The TRACK-MAP marks it `[optional]`. Making it a real stage would add complexity with no user action required — it's purely an AI measurement display.

**What this means**:
- Stepper shows: `... → Periksa → Tuntas` (Tuntaskan), `... → Rayakan → Tuntas` (Wujudkan), etc.
- After reaching Tuntas, the card stays in Tuntas state. If AI-06 triggers Dampak measurement, a **Dampak panel** appears inside the Tuntas view (collapsible, below the completion summary).
- The Dampak panel uses the same before/after display designed in B4-rayakan-card.html.
- TRACK-MAP.md `[Dampak]` brackets = correct notation (optional, not a state transition).
- UI-UX-SPEC-v0.5.md transition table "Periksa → Dampak" should be read as "Periksa → Tuntas (with optional Dampak measurement)".

### C2. Manual Triage Grid — 5 Tracks + Catatan Saksi Only

**Decision**: The manual triage grid ("Pilih sendiri") shows **6 options**: 5 community tracks + Catatan Saksi. **Siaga is NOT included.**

**Why**: Siaga should only trigger through AI-00 detection of emergency language, or by tapping the 112 emergency link. Allowing casual manual selection of Siaga risks abuse (false alarms) and cheapens the emergency signal. If someone genuinely needs Siaga, they'll express urgency in their message and AI-00 will catch it.

**Grid layout**:
```
┌──────────┬──────────┬──────────┐
│ Tuntaskan│ Wujudkan │ Telusuri │
├──────────┼──────────┼──────────┤
│ Rayakan  │Musyawarah│Cat. Saksi│
└──────────┴──────────┴──────────┘
```

**Emergency escape hatch**: The 112 link is ALWAYS visible in the app header during triage. If someone needs emergency help, they can tap 112 directly — this bypasses triage entirely.

**Amends**: UI-UX-SPEC-v0.5.md line ~438 "5 tracks + vault + siaga" → "5 tracks + Catatan Saksi" (no siaga).

### C3. "Auto-Apply" vs "Never Auto-Apply" — Scope Clarification

**Decision**: These are not contradictory. They describe different scopes:

| Scope | Rule | Example |
|---|---|---|
| **Classification (AI-01)** | Auto-apply at ≥80% confidence | AI assigns track without asking if confidence is high |
| **Content changes (AI-07, etc.)** | Never auto-apply — suggest via diff card | AI proposes summary edits, human confirms |
| **Credit (AI-09 Type A/B)** | Auto-award (system action, no human gate) | Binary task completion → instant credit |

**The principle "AI never auto-applies" refers specifically to content/structured data changes** — the Suggest-Don't-Overwrite pattern. Classification and credit are system-level routing/scoring, not content changes.

**Amends**: No file changes needed. This clarification is sufficient for developers.

### C4. Community Classification Confirmation — Implicit Unless Challenged

**Decision**: Track assignment is **implicit** — the AI or author sets the track, and the community can challenge it during Bahas. There is no explicit "community vote to confirm track" step.

**Flow**:
1. AI-00 suggests track → user confirms ("Setuju — Tuntaskan") → seed enters Bahas with that track
2. During Bahas, any participant can tap "🏷 Salah jalur?" → opens a reclassification suggestion (same bottom sheet as manual grid)
3. If ≥3 participants agree on a different track → AI surfaces a reclassification diff card to the PIC
4. PIC decides (Suggest-Don't-Overwrite)
5. Pre-Bahas: author can freely change track (no community gate)

**Amends**: UI-UX-SPEC-v0.5.md line ~43 "the community confirms via vote" → "the community can challenge via reclassification during Bahas" (governance text update).

### C5. Dormant Kabar — Removed

**Decision**: There is no 6th seed type. **Everything routes to one of the 5 tracks.** "Kabar Baik" (good news) routes to Rayakan. General news/information with no action needed can be posted as a discussion in any relevant track's Bahas tab, or as a Rayakan seed if it's worth celebrating.

**Why**: A dormant seed type with no track creates orphaned content, complicates feed logic, and goes against the platform's action-oriented design ("addiction to contribution, not consumption").

**Amends**: UI-UX-SPEC-v0.5.md line ~358 "Dormant Kabar" section should be treated as deprecated. The concept does not ship.

---

## Part D — GDF Computation Model

### Problem
The GDF (Global Difficulty Floor) Weather system shows in the UI but the computation and multiplier behavior are undefined.

### Computation

**Input signals** (all measured at the scope level — RT, RW, etc.):

| Signal | Weight | Source |
|---|---|---|
| Active seed ratio | 40% | `active_seeds / total_members` — lower = harder community |
| Contribution frequency | 30% | `contributions_last_30d / active_members` — lower = less active |
| Stall rate | 20% | `stalled_seeds / active_seeds` — higher = more friction |
| Response time | 10% | `median_hours_to_first_response` — higher = slower community |

**Formula**:
```
difficulty_score = (1 - active_seed_ratio) × 0.4
                 + (1 - contribution_frequency_normalized) × 0.3
                 + stall_rate × 0.2
                 + response_time_normalized × 0.1

GDF% = clamp(difficulty_score × 20, 0, 20)
```

All component values are normalized to 0–1 range based on platform-wide percentiles.

### Multiplier — Discrete Bands (Canonical)

| Band | GDF% Range | Weather | Multiplier | Applied To |
|---|---|---|---|---|
| Cerah | 0–5% | ☀️ | 1.0× | Competence (C) axis only |
| Berawan | 5–10% | 🌤️ | 1.2× | Competence (C) axis only |
| Hujan | 10–15% | 🌧️ | 1.5× | Competence (C) axis only |
| Badai | 15–20% | ⛈️ | 2.0× | Competence (C) axis only |

**Discrete, not continuous.** The "bonus x1.35 at 17.4%" shown in C5-tandang-credit.html was a prototype illustration error — the actual multiplier at 17.4% GDF is 2.0× (Badai band).

**Update frequency**: Recalculated daily at 00:00 WIB. Weather changes are not retroactive — contributions earn the multiplier active at time of action.

**Scope**: Each administrative level (RT, RW, Kelurahan, etc.) has its own GDF%. The multiplier applied to a contribution uses the seed's scope level.

---

## Part E — Vault Security Contract

### Threat Model

The Catatan Saksi vault protects **witness records** that may be sensitive (domestic violence, corruption, safety concerns). The primary threats are:

1. **Unauthorized access** — someone other than the author reading sealed entries
2. **Tampering** — altering the content of a sealed entry after the fact
3. **Forced disclosure** — platform operator compelled to reveal vault contents
4. **Loss** — author loses access to their own records

### Encryption Model — Server-Side Encryption with Client-Side Integrity Seal

**Why not full E2EE**: The Wali (trustee) system requires server-mediated access sharing. Full E2EE would mean the Wali cannot read entries without the author's device being online for key exchange, which breaks the safety use case (author may be incapacitated).

| Layer | What | How |
|---|---|---|
| **Transport** | All data in transit | TLS 1.3 |
| **At-rest encryption** | All vault entries on server | AES-256-GCM with per-user derived key (server-managed KMS) |
| **Integrity seal** | Tamper evidence | Client-side SHA-256 hash of (content + attachments + timestamp + author_id). Hash stored both client-side and server-side. Author can verify integrity anytime by recomputing. |
| **Wali access** | Trustee reads sealed entries | Server grants read-only decryption to designated Wali. Author controls Wali list. Revocable. |

### What Gets Hashed (SHA-256 Seal)

```
seal = SHA-256(
  utf8(entry_text)
  + utf8(attachment_hashes[].join(","))
  + utf8(ISO8601_timestamp)
  + utf8(author_user_id)
)
```

- Displayed as truncated hex in UI: `SHA-256: a3f8c2...b91d`
- Full hash available on tap (copy to clipboard)
- Author can recompute locally to verify server hasn't tampered

### Wali Access Flow

1. **Author designates Wali**: searches by name → selects → confirms permissions
2. **Server grants Wali read key**: Wali's account is added to the entry's ACL
3. **Wali sees**: entry text + attachments + seal hash + timestamp. Cannot edit, delete, or share.
4. **"Surface with consent"**: Wali can send a request to the author: "May I share this with [reason]?" Author approves/denies in-app. If author is unreachable for 30 days, Wali can escalate to platform safety team (see below).
5. **Revocation**: Author can remove Wali at any time. Wali immediately loses access.

### Safety Escalation

If a Wali believes the author is in danger (based on Pola/pattern detection or direct knowledge):
1. Wali taps "Eskalasi Keamanan" (Safety Escalation)
2. Platform safety team reviews (human, not AI)
3. If warranted, safety team can surface the entry to appropriate authorities WITH documented legal basis
4. Author is notified unless notification itself poses risk (per safety team judgment)

### Data Export & Deletion

- **Export**: Author can export all vault entries as encrypted ZIP (password-protected) at any time
- **Account deletion**: Vault entries are permanently deleted after 30-day grace period. Wali access revoked immediately.
- **No platform backup of deleted vaults**: Once purged, data is unrecoverable

---

## Part F — Rahasia × Tandang Credit Visibility Matrix

### Problem
L2 is "anonymous" but the system still awards credits. What's visible to whom?

### Matrix

| Aspect | L0 Terbuka | L1 Terbatas | L2 Rahasia | L3 Sangat Rahasia |
|---|---|---|---|---|
| **Author name** | Visible | Visible (verified only) | "Anonim" (grey avatar) | "Tersembunyi" |
| **Author tier badge** | Visible | Visible | Shown (no name link) | Hidden |
| **Content** | Full | Full | Full | Hatched/redacted |
| **Credit earned?** | Yes | Yes | Yes — silently | Yes — silently |
| **Credit toast visible to author?** | Yes | Yes | Yes (private) | Yes (private) |
| **Credit visible on public profile?** | Yes, linked to seed | Yes, linked to seed | **No link** — credit aggregated without seed reference | **No link** — aggregated |
| **Credit diff card at Tuntas** | Full names | Full names | Pseudonymized ("Kontributor A, B...") | Pseudonymized |
| **Dispute evidence** | Full identity | Full identity | Pseudonymized to disputants, real identity visible only to PIC + moderator | Pseudonymized to all except moderator |
| **Share (C6)** | ↗ Active | ↗ Hidden 🔒 | ↗ Hidden 🔒 | ↗ Hidden 🔒 |
| **Feed visibility** | All | Verified members only | Verified members only | Hidden from feed |
| **Notifications to others** | Full | Full | "Seseorang berkontribusi pada..." | No notification |

### Key Rules

1. **Credit is always earned** — even at L3. The system tracks contributions regardless of privacy level. This prevents gaming where users lower privacy to avoid credit consequences.
2. **Credit visibility is privacy-gated** — the credit itself is real, but the public display respects the Rahasia level.
3. **PIC and moderators always see real identity** — they need it for governance. This is disclosed in the privacy settings ("PIC dan moderator selalu dapat melihat identitas Anda").
4. **Disputes at L2/L3 use pseudonyms** — "Kontributor A menyelesaikan tugas X" instead of real names. Only PIC + moderator can de-anonymize if needed for resolution.

---

## Part G — Wali Consent Mechanism

### "Surface with Consent" — Detailed Flow

```
┌─────────┐       ┌──────────┐       ┌─────────┐
│  Wali   │──1──▶│  Server  │──2──▶│  Author │
│ requests│       │ validates│       │ reviews │
└─────────┘       └──────────┘       └────┬────┘
                                          │
                                     3a. Approve
                                     3b. Deny
                                          │
                                    ┌─────▼─────┐
                                    │  Outcome   │
                                    └───────────┘
```

1. **Wali initiates**: Taps "Minta Persetujuan Publikasi" on a sealed entry they can read. Provides:
   - Reason (free text, required): "Saya khawatir tentang keselamatan Anda" / "Bukti ini relevan untuk kasus RT"
   - Destination: "Publikasi ke komunitas" / "Bagikan ke [nama]" / "Eskalasi keamanan"
   - Urgency: Normal / Mendesak

2. **Author receives notification**: "Wali Anda [nama] meminta persetujuan untuk [destination]. Alasan: [reason]"
   - Author sees: full request details + the entry that would be surfaced + preview of how it would appear
   - Actions: **Setuju** / **Tolak** / **Tanya Lebih Lanjut** (opens chat with Wali)

3. **Outcomes**:
   - **Approved**: entry is surfaced per destination. If "Publikasi ke komunitas", follows the normal Terbitkan flow (track picker, Rahasia toggle, etc.)
   - **Denied**: Wali is notified. Entry stays sealed. Wali cannot re-request for the same entry for 7 days.
   - **No response 30 days + Mendesak**: Wali sees option "Eskalasi Keamanan" (see Vault Security Contract above)

4. **Audit trail**: All consent requests, responses, and escalations are logged with timestamps. Visible to author in entry detail view under "Riwayat Wali" (Wali History).

---

## Part H — Moderator Dashboard

### Who Are Moderators?

At RT/RW scale, moderators are:
- **Ketua RT/RW** (head of RT/RW) — automatic moderator role upon community creation
- **Appointed by Ketua RT** — up to 3 additional moderators per RT, selected from Pilar or Kunci tier members
- **Platform safety team** — can intervene at any level (not visible as local moderators)

### Access Point

Moderators access their queue via **Lainnya (☰) → CV Hidup (Profil) → "Moderasi" section** (appears only for users with moderator role). This is NOT a separate app or tab — it's a section within the hamburger menu profile page, keeping the 5-tab nav clean. *(Updated per S3-MD3: Profil moved from tab bar to hamburger menu.)*

### Moderator Queue UI

```
┌────────────────────────────────────┐
│  Moderasi · RT 05                   │
│  3 menunggu · 1 eskalasi           │
├────────────────────────────────────┤
│  ┌─ Antrean ──────────────────┐    │
│  │ 🟡 Laporan konten (2)      │    │
│  │ 🟠 Moderasi AI-04 (1)      │    │
│  │ 🔴 Eskalasi Wali (1)       │    │
│  └────────────────────────────┘    │
│                                     │
│  [Riwayat Keputusan]  [Panduan]    │
└────────────────────────────────────┘
```

### Queue Item Types

| Type | Source | Actions Available |
|---|---|---|
| **Content report** | User taps "Laporkan" on any seed/comment | View content + reporter reason + author history → **Hapus** / **Peringatan** / **Abaikan** / **Eskalasi** |
| **AI-04 moderation hold** | AI-04 detects policy violation (≥60% confidence) | View content + AI reason + confidence → **Setuju (hapus)** / **Tolak (restore)** / **Eskalasi** |
| **Wali safety escalation** | Wali escalated after 30-day no-response | View vault entry (read-only) + Wali reason → **Teruskan ke Tim Keamanan** / **Kembalikan ke Wali** |
| **Reclassification dispute** | Community challenged track (≥3 users) | View seed + original track + proposed track → **Ubah Jalur** / **Pertahankan** |
| **Duplicate merge request** | AI-03 flagged ≥80% similarity | View both seeds side-by-side → **Gabung** / **Beda** / **Tunda** |

### Moderator Actions

All moderator actions follow these rules:
1. **Logged**: every action creates an audit entry (who, what, when, reason)
2. **Reversible within 24h**: moderator can undo their own decisions
3. **Appealable**: affected users see "Banding" (appeal) button → escalates to platform safety team
4. **Transparent to author**: author always knows their content was moderated and why (except in safety escalations where notification poses risk)

### Permissions

| Action | Moderator | Ketua RT | Platform Safety |
|---|---|---|---|
| Remove content | ✓ | ✓ | ✓ |
| Issue warning | ✓ | ✓ | ✓ |
| Temporary mute (24h) | ✓ | ✓ | ✓ |
| Permanent ban | ✗ | ✗ | ✓ (only) |
| View L2/L3 author identity | ✓ | ✓ | ✓ |
| Access vault entries | ✗ | ✗ | ✓ (with legal basis) |
| Modify Tandang scores | ✗ | ✗ | ✓ (dispute resolution) |

---

## Part I — Offline / Error State Patterns

### Design Principle
Indonesia's mobile connectivity is unreliable. The app must **never lose user work** and must **always be honest about state**.

### Connection States

| State | Indicator | Behavior |
|---|---|---|
| **Online** | No indicator (clean) | Normal operation |
| **Slow** | Yellow bar: "Koneksi lambat" | All features work, media uploads queued, AI responses may be slower |
| **Offline** | Red bar: "Tidak ada koneksi — perubahan akan disimpan" | Read cached content, queue writes, disable AI features |
| **Reconnecting** | Pulsing bar: "Menyambungkan kembali..." | Syncing queued actions, progress indicator |

### Per-Feature Offline Behavior

| Feature | Offline Behavior |
|---|---|
| **Feed (Beranda)** | Show cached cards with "Terakhir diperbarui: [time]" label. Pull-to-refresh shows "Tidak bisa memperbarui — offline" |
| **AI-00 Triage** | Unavailable. Show: "Triage memerlukan koneksi. Simpan sebagai draf?" → saves locally. Or "Pilih sendiri" manual grid (works offline for creating draft seeds) |
| **Chat (Bahas tab)** | Read cached messages. New messages queued with ⏳ clock icon. "Menunggu koneksi..." status. Sent automatically when online. |
| **Media upload** | Queued. Show progress: "📷 3 foto menunggu unggah". Retry automatically. |
| **Voting (Putuskan)** | Queued. Show: "Suara Anda tersimpan, akan dikirim saat online" |
| **Siaga** | **Critical path**: attempt to send via SMS fallback if data unavailable. Show: "⚠ Koneksi tidak stabil — mencoba SMS darurat ke 112" |
| **Catatan Saksi** | Full compose + seal works offline (SHA-256 computed client-side). Sync when online. |
| **Notifications** | Cached. New notifications arrive when reconnected. |

### Error States

| Error | UI Pattern |
|---|---|
| **API timeout** | Inline retry: "Gagal memuat. [Coba lagi]" — no full-screen error |
| **AI failure** | Graceful degradation: "AI sedang tidak tersedia. Anda bisa melanjutkan secara manual." + manual fallback |
| **Upload failure** | Per-file retry: "📷 Gagal mengunggah foto-1.jpg [↻ Ulangi] [✕ Batal]" |
| **Permission denied** | Clear explanation: "Anda perlu verifikasi untuk melihat konten L1" + action button |
| **Rate limit** | Soft block: "Terlalu banyak aktivitas. Coba lagi dalam [X] menit." |
| **Server error (500)** | Friendly: "Ada masalah di server kami. Tim sedang memperbaiki. [Coba lagi]" |

### Empty States

| Screen | Empty State |
|---|---|
| **Beranda (0 seeds)** | Illustration + "Komunitas ini baru dimulai! Jadilah yang pertama berbagi." + prominent [+ Bagikan] button |
| **Terlibat (no involvement)** | "Anda belum terlibat di kegiatan apa pun. Lihat [Bantu] untuk peluang." |
| **Bantu (no matches)** | "Belum ada yang cocok dengan keahlian Anda. [Tambah Keahlian] untuk melihat lebih banyak peluang." |
| **Notifikasi (empty)** | "Belum ada notifikasi. Mulai berkontribusi untuk menerima pembaruan!" |
| **Catatan Saksi (0 entries)** | Vault illustration + "Catatan Anda bersifat pribadi dan terenkripsi. Mulai mencatat." |
| **Search (no results)** | "Tidak ditemukan. Coba kata kunci lain atau perluas jangkauan." |
| **Single-person RT** | Special banner on Beranda: "Anda satu-satunya warga di RT ini. [Undang Tetangga] untuk memulai gotong royong." + direct link to C6 Undang flow |

---

## Part J — Account & Community Management

### Account Settings (Profil → ⚙️ Pengaturan)

| Setting | Details |
|---|---|
| **Edit profil** | Name, avatar, bio. Area change requires re-verification. |
| **Ubah nomor HP** | OTP verification on new number. Old number invalidated. |
| **Kelola komunitas** | List of joined RT/RW communities. Switch primary. Leave community (with warning about losing local reputation context). |
| **Gabung komunitas baru** | Search by area or invite code. Verification required (varies by community settings). |
| **Bahasa** | Indonesian (default). English annotations toggle (deferred — placeholder for future). |
| **Notifikasi** | Per-type toggles (see Part L). Quiet hours setting. |
| **Privasi** | Default Rahasia level for new seeds. Visibility of profile to non-members. |
| **Data & penyimpanan** | Cache size. Clear cache. Export data (JSON + media ZIP). |
| **Keluar** | Logout. Session cleared. |
| **Hapus akun** | Deliberate confirmation (type "HAPUS" to confirm). 30-day grace period. All data permanently deleted after grace period. Vault entries purged. Tandang scores removed from leaderboards. Contributions anonymized ("Mantan Anggota") on existing seeds. |

### Joining / Leaving Communities

**Join flow**:
1. Search by area name or enter invite code (from C6 Undang)
2. Request to join → community moderator approves (or auto-approve if community settings allow)
3. User starts as Pemula tier in new community
4. Tandang scores are per-community (not global)

**Leave flow**:
1. Profil → Pengaturan → Kelola Komunitas → [community] → "Tinggalkan"
2. Warning: "Anda akan kehilangan peran dan konteks reputasi lokal di [community]. Kontribusi Anda tetap ada sebagai 'Mantan Anggota'. Yakin?"
3. Confirm → user removed from community. Seeds they authored remain (attributed to "Mantan Anggota"). Active PIC roles reassigned by moderator.

### Multi-Community

Users can be in multiple RT/RW communities simultaneously. The scope selector (app header) switches between them. Tandang scores are **per-community** — being Kunci in RT 05 doesn't make you Kunci in RT 06.

---

## Part K — Block / Mute Flow

### Access Points
- **On any seed card**: overflow menu (⋮) → "Blokir [nama]" / "Senyapkan [nama]"
- **On any chat message**: long-press → "Blokir" / "Senyapkan"
- **On user profile**: overflow menu → "Blokir" / "Senyapkan"
- **In Pengaturan**: "Pengguna Diblokir" → manage list

### Mute vs Block

| Action | Effect | Duration | Reversible |
|---|---|---|---|
| **Senyapkan** (Mute) | Their content deprioritized in your feed. No notifications from them. You still see their content if you look for it. They don't know. | Until unmuted | Yes — Pengaturan → Pengguna Disenyapkan |
| **Blokir** (Block) | Their content hidden from you entirely. They can't see your profile or seeds. They can't @mention you. You can't interact with their content. They see "Konten tidak tersedia" on your seeds. They are NOT notified of the block. | Until unblocked | Yes — Pengaturan → Pengguna Diblokir |

### Interaction with Other Systems

| System | Muted User | Blocked User |
|---|---|---|
| **Feed** | Deprioritized (bottom of feed) | Hidden entirely |
| **Chat** | Messages visible but no notification | Messages hidden, can't send to you |
| **Voting** | Their vote still counts (system integrity) | Their vote still counts (anonymous, system integrity) |
| **Tandang** | Credit still earned/given | Credit still earned/given (system integrity) |
| **Moderation juries** | Can still serve | Excluded from juries involving you |
| **Wali** | Can still be Wali | Cannot be designated as your Wali |
| **Bantu** | Still matched to your seeds | Not matched to your seeds |

### Moderator Visibility
Moderators can see block/mute patterns in aggregate (not specific pairs) to identify potential harassment patterns. Example: "5 users have blocked [nama] this week" → triggers review.

---

## Part L — Push Notification Templates + Siaga Safety

### Push Notification Templates

| Type | Lock Screen Text | Rahasia L2+ Variant |
|---|---|---|
| **Skill match** | "🤝 Ada yang butuh keahlian Anda: [skill] di [seed title]" | "🤝 Ada yang butuh keahlian Anda di komunitas" |
| **Credit earned** | "⭐ +[X] [axis] — kontribusi Anda di [seed title]" | "⭐ Anda mendapat kredit baru" |
| **Mention** | "💬 [nama] menyebut Anda di [seed title]" | "💬 Seseorang menyebut Anda" |
| **Stage change** | "📋 [seed title] → [stage name]" | "📋 Sebuah kegiatan berubah tahap" |
| **Vote open** | "🗳 Saatnya memilih: [musyawarah title]" | "🗳 Ada pemilihan baru" |
| **Stall alert** | "⏳ [seed title] macet [X] hari — butuh bantuan?" | "⏳ Sebuah kegiatan membutuhkan perhatian" |
| **Weekly digest** | "📊 Minggu ini: [X] kontribusi, [Y] skill tervalidasi" | Same (aggregated, no seed refs) |
| **Siaga** | "🚨 DARURAT: [type] di [location]" | N/A (Siaga is always L0) |

### Permission Prompt
First-time: "Gotong Royong ingin mengirim notifikasi untuk kegiatan komunitas, darurat, dan kecocokan keahlian. [Izinkan] [Nanti]"

### Quiet Hours
Default: 22:00–06:00 WIB. Siaga notifications **always** break through quiet hours (emergency override).

### Siaga Abuse Prevention

| Mechanism | Rule |
|---|---|
| **Rate limit** | Max 1 Siaga per user per 24h. Max 3 per user per 7d. |
| **Tier gate** | Bayangan (Shadow) tier cannot create Siaga broadcasts. |
| **AI-00 validation** | AI must detect genuine emergency language before enabling Siaga bar. Random/test messages → manual grid (no Siaga option). |
| **Community flag** | Any 3 users can flag a Siaga as "Tidak darurat" → moderator queue → if confirmed false: Siaga ended + warning to author + rate limit doubled. |
| **Repeat offender** | 2 confirmed false Siaga → Siaga privilege suspended 30 days. 3 → permanent suspension (platform safety team can reinstate). |

### No-Responder Scenario

| Time Since Broadcast | UI Change |
|---|---|
| **5 minutes, 0 responders** | Banner: "Belum ada yang merespons. [Sebarkan ke WhatsApp] untuk jangkauan lebih luas" + prominent C6 share button |
| **15 minutes, 0 responders** | Auto-escalation: notification sent to all Pilar + Kunci tier members in scope (even if muted/quiet hours) |
| **30 minutes, 0 responders** | Suggestion: "Pertimbangkan menghubungi 112 langsung" + 112 call button made more prominent |
| **1 hour, 0 responders** | Auto-widen scope: if RT-scoped, expand to RW. If RW, expand to Kelurahan. Notification: "Siaga diperluas ke [wider scope]" |

---

## Part M — Accessibility Contract

### Minimum Standards

| Aspect | Standard | Current Status |
|---|---|---|
| **Color contrast** | WCAG AA (4.5:1 body text, 3:1 large text) | ✓ Verified in A2-color-palette.html |
| **Touch targets** | Minimum 44×44px (iOS) / 48×48dp (Android) | ⚠ Some caption-level tappables need audit |
| **Font sizes** | Minimum readable: 12px (--fs-small). 9px (--fs-micro) used ONLY for non-essential decorative labels, never for actionable content | ⚠ Micro (9px) should not be used for interactive elements |
| **Reduced motion** | Respect `prefers-reduced-motion`: disable shimmer (Siaga), pulsing (context bar), auto-scroll. Keep all functionality. | To implement |
| **Screen readers** | Semantic HTML: proper heading hierarchy, ARIA labels on icons, alt text on avatars, role attributes on interactive elements | To implement |
| **Color-blind safe** | All status indicators use icon + color (never color alone). Track identity uses icon + color + label. | ✓ Designed with redundant cues |
| **RTL support** | Not required for Indonesian. Defer. | N/A |
| **Older users** | System font size scaling respected. UI density adapts. "Teks Besar" option in Pengaturan doubles --fs-micro and --fs-caption. | To implement |

### Key Rules for Implementation

1. **Never use color alone** to convey meaning. Always pair with icon, label, or pattern.
2. **--fs-micro (9px)** is for decorative/non-essential labels only (SHA hash snippets, build version, annotation copy). Never for buttons, links, or required content.
3. **All icons must have ARIA labels** in Indonesian.
4. **Stepper breadcrumbs** must announce state changes to screen readers: "Tahap saat ini: Bahas. 2 dari 6 tahap."
5. **Siaga shimmer animation** must stop under `prefers-reduced-motion`. Replace with static red border.

---

## Part N — Deep Link Landing Pages

### When someone opens a shared link without the app

**URL pattern**: `gotongroyong.id/s/{short_id}`

**Web landing page structure**:
```
┌──────────────────────────────────┐
│  [GR logo]  Gotong Royong        │
├──────────────────────────────────┤
│                                   │
│  [OG preview card — same as C6   │
│   Pratinjau Tautan design]        │
│                                   │
│  ┌──────────────────────────┐    │
│  │  Buka di Gotong Royong   │    │ ← deep link attempt (app installed)
│  └──────────────────────────┘    │
│  ┌──────────────────────────┐    │
│  │  Pasang Aplikasi         │    │ ← app store link (not installed)
│  └──────────────────────────┘    │
│                                   │
│  ── atau lihat ringkasan ──      │
│                                   │
│  [Abbreviated content preview]    │ ← title + first 3 lines + author
│  [Track badge + stage + stats]    │   (no full content — must install)
│                                   │
│  Rahasia L1+: "Konten ini        │
│  hanya tersedia untuk anggota    │
│  terverifikasi di Gotong Royong" │
│                                   │
└──────────────────────────────────┘
```

**Behavior by Rahasia level**:
- **L0**: Show abbreviated preview (title, 3 lines, author, track, stage)
- **L1+**: Show only: "Konten ini bersifat [level]. Bergabung di Gotong Royong untuk melihat."
- **Vault**: Link never generated (C6 blocks this)
- **Siaga**: Full preview always shown (emergency = public interest)

**Invite links** (`gotongroyong.id/gabung/{community_code}`): Show community name, member count, area, and "Bergabung" button → app store or app deep link.

---

## Changelog

| Date | Author | Changes |
|---|---|---|
| 2026-02-14 | Design Team + AI | Initial version. Resolves all 8 critical gaps, 8 important gaps, 10 contradictions, 4 nice-to-haves from external review. |

# Gotong Royong — Design Sequence Guide

> Work through each step in order. Nothing gets assumed — each checkpoint needs approval before moving on.

---

## Phase A — Design DNA Foundation

- [x] **A1 · Mood & Vibe** ✓ TANAH
  Warm community, earthy tones, Nunito, rounded shapes. Locked.

- [x] **A2 · Color Palette** ✓ LOCKED
  Full Tanah palette with Indonesian token names. Core (7), Action (4), Semantic (8), 5 Track Accents (3 each), Vault (6), Siaga (5), Rahasia (4 levels). All pass WCAG AA.

- [x] **A3 · Typography & Spacing** ✓ LOCKED
  Nunito Major Third scale (8 steps), 4px spacing grid (10 tokens), 5 radii, 4 warm shadows. All as CSS variables.

- [x] **A4 · Core Components** ✓ LOCKED
  Buttons (6 variants × 3 sizes + track + vault/siaga), badges (track/seed/semantic/stepper/special), inputs (4 states + compose), avatars (5 sizes + group + tier), pills (filter/track/removable), status indicators (6 states), progress bars, toggles, tooltips.

---

## Phase A+ — Entry Flow (new: AI-00 Triage)

- [x] **A+1 · Conversational Triage + Context Bar** ✓ LOCKED (v4)
  Chat-based AI-00 conversation (not a compose box). AI greets → user tells story → AI probes if needed → routes. Morphing context bar above keyboard fed by AI XML metadata. Bar states: listening → probing (signal bar) → leaning (tappable pill) → ready (full card). "Pilih sendiri" skip to manual grid. Vault-tinted, siaga-pulsing, split, and manual variants. Three outcomes: Komunitas (→ AI-01), Catatan Saksi (→ vault), Siaga (→ broadcast). Split detection with linkability warning.

- [x] **A+2 · Catatan Saksi Vault UI** ✓ LOCKED
  Dark vault-tinted UI (--v-deep → --v-wash palette). 5-state lifecycle: Menyimpan (compose with pre-filled AI-00 text, attachment tools) → Tersegel (SHA-256 hash, timestamp, encrypted badge, "Tandang tidak terpengaruh") → Wali (trustee search, permission list, tier badge) → Terbitkan (orange warning with 3 consequences, track picker, Rahasia L2 toggle, deliberate confirmation) → Pola (AI pattern detection, gentle safety alert, resource links). Seal bar at bottom mirrors context bar concept — morphs between unsealed (editable) and sealed (locked + actions).

- [x] **A+3 · Siaga Broadcast UI** ✓ LOCKED
  Siaga red palette (--s-deep → --s-soft). Speed-first, no ceremony. 4-state lifecycle: Kirim (compose with pre-filled AI-00 text, emergency type chips, auto-location card with map, 112 link always visible, one-tap "Siarkan Sekarang") → Aktif (shimmer-animated live card, real-time stats: terjangkau/melihat/merespons, quick update input, event timeline, pulsing broadcast bar) → Respons (responder cards with distance/ETA/status message, quick-respond buttons, growing timeline) → Selesai (deliberate confirmation, summary card with duration/responders/services, thank-you message, green resolved bar). Broadcast bar at bottom mirrors context bar (A+1) and seal bar (A+2) — morphs between composing/active/resolved.

---

## Phase B — Card Interfaces (one track at a time)

- [x] **B0 · Seed Card (universal)** ✓ LOCKED
  Universal card anatomy: 4px track-colored left strip (identity), header (seed badge + Rahasia level + title + author row with avatar/name/tier/time), body (3-line text + "selengkapnya" + media thumbnails + location tag + PIC row + Dampak row), stepper breadcrumb (track-colored current dot, done/future states, actual Indonesian stage names), footer (💬 comments · 👥 supporters · ⏱ time-in-stage · AI badge). 5 views: Anatomi (annotated reference), Baru (fresh seed with AI-01 badge), Aktif (mid-lifecycle with PIC + stall detection), Rahasia (L1 badge-only → L2 anonim → L3 full redaction with hatched pattern), Lima Warna (all 5 track colors on same shape + full Rayakan/Musyawarah examples).

- [x] **B1 · Tuntaskan Card** ✓ LOCKED (v3)
  6-state lifecycle (Keresahan → Bahas → Rancang → Garap → Periksa → Tuntas). Introduces dual-tab pattern (cross-cutting): tab bar with track-colored underline + swipe hint, WhatsApp-style chat with AI-07 inline cards, date separators, reactions. Stage tabs: Bahas (Diskusi|Rangkuman), Rancang (Papan GR|Koordinasi), Garap (Progres|Koordinasi), Periksa (Laporan|Tanggapan). Papan Gotong Royong with task list, Kontribusi system (anonymous slots), PIC-controlled readiness checklist. Also locks the **LLM ↔ UI Architecture**: 7 block primitives (list, document, form, computed, display, vote, reference), LLM-driven composition (no rigid schemas — LLM decides blocks per problem, human iterates), 3 source tags (ai/human/system), 4 trigger modes (manual/milestone/time/passive), Suggest-Don't-Overwrite diff card, cross-card write-back via suggestion, protected fields for financial data.

- [x] **B2 · Wujudkan Card** ✓ LOCKED (v1)
  7-state lifecycle (Gagasan → Bahas → Rancang → Galang → Garap → Rayakan → Tuntas). Introduces **milestone tracker** (E4 hierarchy): milestone cards with sub-tasks, assignees, and phase progress counts. Introduces **Galang sub-lifecycle panel** (E6 protected fields): target/collected/remaining with 🔒 DILINDUNGI badge, progress bar, contributor list, AI-exclusion notice for financial data. Introduces **Rayakan celebration display**: emoji, contributor avatars, community stats, reaction buttons (replaces Periksa from Tuntaskan). Dual-tab pattern and all cross-cutting components re-themed to Wujudkan green (#2E7D32).

- [x] **B3 · Telusuri Card** ✓ LOCKED (v1)
  5-state lifecycle (Pertanyaan → Dugaan → Uji → Temuan → Tuntas). Introduces **hypothesis cards** (E2 multi-state): 5 states (Diajukan → Diuji → Terbukti/Ditolak/Belum Jelas), evidence counts per hypothesis, source tags (ai/human). Introduces **evidence board**: evidence items grouped by hypothesis with support/refute/neutral indicators, typed badges (Observasi/Kesaksian/Data/Dokumen). Introduces **Temuan document** (E1 document block): rich text sections with per-section ✏️/🤖 edit, confidence meter with visual bar (low/medium/high), hypothesis result summary. Introduces **track-change suggestion card** (E5 cross-card): "Buat Tuntaskan" / "Buat Wujudkan" spawning from findings, track-change badge on archived card. Dual-tab pattern and all cross-cutting components re-themed to Telusuri purple (#6A1B9A).

- [x] **B4 · Rayakan Card** ✓ LOCKED (v1)
  4-state lifecycle (Kabar Baik → Sahkan → Apresiasi → Tuntas). **Dampak is not a stage**: optional post-Tuntas measurement panel (E3 time-trigger, AI-06) that appears inside the Tuntas view with before/after comparison and impact indicators. Introduces **validation panel**: community endorsement with threshold counter, progress bar, individual validator comments + checkmarks. Introduces **recognition card**: hero display for the celebrated person with avatar, achievement badge ("Pahlawan Literasi"), stats row. Introduces **appreciation wall**: community messages with emoji reactions, message count. Dual-tab pattern and all cross-cutting components re-themed to Rayakan amber (#F57F17).

- [x] **B5 · Musyawarah Card** ✓ LOCKED (v1)
  6-state lifecycle (Usul → Bahas → Putuskan → Jalankan → Tinjau → Tuntas, Tinjau optional). Introduces **position board** (E2 position tracking): community options with supporter counts, visual support bars, supporter chips, "Dukung Opsi Ini" stance buttons. Introduces **vote panel** (`vote` block primitive): system-authoritative with SISTEM tag, radio-button options, quorum progress bar, deadline timer, anonymous & immutable vote notice. Introduces **vote results**: bar chart with winner highlight, percentage labels, participation rate. Introduces **Ketetapan document** (E1 document + E5): formal numbered decision document (KMW/RT05/2026/003) with sections (Keputusan/Dasar/Tindak Lanjut), per-section edit, implementation tasks with assignees and done/active states. Introduces **Tinjau panel** (E3 time-trigger): AI-06 auto-triggered review, effectiveness metrics with percentage changes. Dual-tab pattern and all cross-cutting components re-themed to Musyawarah brown (#4E342E).

---

## Phase C — Cross-cutting UI

- [x] **C5 · Tandang Credit Integration** ✓ LOCKED (v1)
  **UI Components:** Credit toast (3 variants: +I amber, +C teal, +J purple), formalized tier badge (◆◇ system: Kunci/Pilar/Kontributor/Pemula/Bayangan on all avatars), contribution summary on Tuntas cards (computed block, source: system), Tandang Profile card (I/C/J radar, tier, ESCO-ID skills, vouch graph, CV Hidup QR), GDF Weather widget ("Cuaca Komunitas" with Cerah/Berawan/Hujan/Badai + compact feed variant), competence decay nudge, vouch mechanics ("Jaminkan" button, vouch chain visualization, slash cascade warning). Tandang teal palette (--c-tandang:#00695C), I/C/J axis colors (amber/teal/purple), tier colors (gold/blue/teal/brown/red).
  **Mechanics (Credit Accreditation Pattern):** Locks AI-09 (Credit Accreditation). Type A/B = system auto-awards (action/duration = proof). Type C = peers validate (consensus = accreditation). Type D = AI-proposed → human-confirmed (quality rating). Type E = human-initiated (vouch with stake risk). 5-step credit flow: silent tracking → instant toast → AI nudges in chat → Tuntas diff card (PIC confirms via Suggest-Don't-Overwrite) → dispute mechanism. Vault & Siaga = ZERO credit. GDF multiplier auto-applied to Competence scoring. Also locks Type E · Stake-Weighted contribution type and complete GR→Tandang credit mapping (11 actions mapped).

- [x] **C1 · Rahasia Overlays** ✓ LOCKED (v1)
  5 views (Empat Tingkat / Gerbang Akses / Dalam Peninjauan / Media Rahasia / Pengaturan). 4-level privacy system (L0 Terbuka → L1 Terbatas → L2 Rahasia → L3 Sangat Rahasia) on same card. L1 verified-only gate, L2 anonymous + blurred media + request-access gate, L3 fully redacted hatched + invisible in feed. Dalam Peninjauan: 3 perspectives (author/moderator/public). Media redaction per level. Privacy settings with upgrade (reversible) vs downgrade (IRREVERSIBLE) warning, change audit log. Rahasia level colors: L1=#8D6E63, L2=#5D4037, L3=#3E2723.

- [x] **C2 · AI Surface States** ✓ LOCKED (v1)
  5 views (Lencana Keyakinan / Chip Saran / Ubah & Diff / Moderasi / Duplikat). 3-band confidence system (Tinggi ≥80% auto-apply, Sedang 50-79% suggest, Rendah <50% show-uncertainty), reusable confidence meter, full AI badge catalog (8 badges). 4 chip types (action/edit/analysis/credit) × 4 states (idle/hover/applied/dismissed). Universal AI inline card with confidence. Suggest-Don't-Overwrite diff card (old/new/protected, evidence quotes, field-by-field review). Moderation hold: 3 perspectives (author/moderator/public), auto-release rule. AI-03 duplicate detection: comparison card, similarity thresholds (≥80% blocking, <50% non-blocking), merge/different actions. "AI is Furniture" principle — no AI brand color.

- [x] **C3 · Navigation & Feed** ✓ LOCKED (v2)
  5-tab bottom nav (Beranda/Terlibat/Bantu/Notifikasi/Profil), app header with scope selector (RT▼) + search (🔍) + compose (+), Community Pulse bar (GDF Weather + live stats), action-weighted feed (5 priority levels: your-action/nearing/new/active/completed), ESCO skill tags on seeds, 7-level scope hierarchy (RT→RW→Kel→Kec→Kota→Prov→Nasional) with bottom-sheet picker, Bantu skill-matched opportunities (validated ● vs declared ○ pills), Terlibat involvement feed (role badges + progress rings + streak counter), time-grouped notifications (7 types: skill-match/credit/mention/stage/vote/stall/digest), CV Hidup profile (I/C/J scores + dual-layer skills + kontribusi timeline + vouch + impact + QR + decay nudge), full-screen search overlay (track/skill/time filters). 7 views total.

- [x] **C4 · Catatan Saksi Feed (private)** ✓ LOCKED (v1)
  Vault-dark timeline (--v-deep → --v-wash palette, no bottom nav). 5 views: Kronologi (vault-bar + stats bar + filter tabs + pattern alert banner + compact feed cards with status badges/SHA-snippet/attach-icons/Wali-avatars), Detail Tersegel (full sealed entry + complete SHA-256 hash + attachment gallery + compact Wali section + seal bar with Ganti Wali/Terbitkan), Kelola Wali (two-perspective hub: Wali Anda + Anda Menjadi Wali, fixed permissions ✓read ✓surface ✕edit ✕share, entry counts, permission legend), Pratinjau Terbitkan (A+2 publish-warning with 3 consequences + track picker + Rahasia L2 toggle + split visual vault→community transformation), Pola & Dukungan (pattern detection alert + flagged entry timeline + 4 Indonesian crisis resources: Telepon Sahabat 119/Komnas Perempuan/LPSK/LBH + gentle dismissible tone).

- [x] **C6 · Share Sheet** ✓ LOCKED (v1)
  Cross-cutting external sharing system. 6 views (Tombol Bagikan / Lembar Bagikan / Pratinjau Tautan / Siaga Sebarkan / Gerbang Rahasia / Undang Komunitas). ↗ share icon in seed card footer (rightmost, after AI badge). OS-style bottom sheet with link preview, 5 platforms (WhatsApp/Telegram/Instagram/Twitter/Facebook) + Salin Tautan + Salin Teks + Lainnya (native OS share). Track-colored Open Graph preview cards with per-track CTA (Bantu Selesaikan/Dukung Sekarang/Lihat Temuan/Ikut Bersuara/Lihat & Bantu). Siaga gets prominent "SEBARKAN SEKARANG" button (not footer icon) — one-tap → WhatsApp with pre-formatted emergency message (location + needs + link). Rahasia gating: L0 = share active, L1+ = ↗ button hidden (replaced by 🔒), no override. Catatan Saksi = never shareable. Web fallback for non-app recipients (preview page + install prompt). Undang Komunitas: invite link + QR code per RT/RW scope, share to WA/Telegram, no Tandang credit for invites (anti-spam).

---

## Phase D — Lock & Package

- [x] **D1 · Design DNA Document** ✓ LOCKED (v0.1)
DESIGN-DNA-v0.1.md — comprehensive formal spec (9 sections + appendix). Covers: 13 core principles, 46+ color tokens (8 palettes: Tanah/Api/Semantik/Track/Vault/Siaga/Rahasia/Tandang), Nunito Major Third type scale (8 steps), 4px spacing grid (10 tokens), 5 radii, 4 brown-tinted shadows, 12 component families (buttons/badges/inputs/avatars/pills/status/progress/toggles/tooltips/dividers), seed card 6-part anatomy + dual-tab pattern, all 5 track lifecycles with unique components, AI-00 triage (8 context bar states) + 3 entry flows (Komunitas/Vault/Siaga), 5-tab navigation + 7-level scope hierarchy + Community Pulse, Tandang credit system (types A–E, 11 GR→Tandang mappings, 5-step flow, 5 tiers, GDF multiplier), Rahasia privacy (L0–L3), AI surface states (3 bands, 8 badges, diff card, moderation), LLM ↔ UI architecture (7 block primitives, 3 source tags, 4 triggers), prototype map updated for 24 HTML references (20 locked), CSS variable quick reference appendix.

- [x] **D2 · HTML Style Guide** ✓ LOCKED (v0.1)
  D2-style-guide.html — interactive browser-viewable reference. 40/40 verification. All 46+ color tokens (8 palettes inc. Rahasia L1–L3), Nunito 8-step type scale, 10 spacing tokens, 5 radii, 4 shadows rendered as live swatches. 12 component families: buttons (6 variants × 3 sizes + text + track + vault/siaga), badges (track/seed/semantic/stepper/AI catalog 8 types/tier full + compact pip), inputs (4 states + textarea), avatars (5 sizes + group + tier pip), pills (state/track/ESCO validated vs declared), status indicators (6), progress bars, confidence bars (3 bands), toggles, tooltips, dividers. Patterns: seed card 6-part anatomy with sample data, 5 track mini-cards with lifecycles, context bar 8 states (listening, probing, leaning, ready, vault-ready, siaga-ready, split-ready, manual), seal bar (unsealed/sealed). Tier system: full (◆◇ + label) and compact pip (18px colored circle with number) variants for different density contexts.

- [x] **D3 · Update AI-SPEC + UI-UX-SPEC** ✓ LOCKED (v0.2 / v0.5)
  AI-SPEC-v0.2.md — expanded from 8 to 10 touch points. Added AI-00 Conversational Triage (8-state context bar, vault/siaga detection, multi-turn conversation flow, prompt strategy) and AI-09 Credit Accreditation (5 credit types A–E, 5-step flow, 11 GR→Tandang mappings, Tuntas diff card, protected fields). Updated AI-01 (now invoked by AI-00 internally), AI-03 (dual-threshold ≥80%/50%, context bar pill, comparison card), AI-04 (3-perspective moderation hold, auto-release). Updated model selection (10 rows), cross-references to UI-UX v0.5, decision log (D11–D14), guardrail metrics for AI-00/AI-09. 20/20 gap coverage verified.
  UI-UX-SPEC-v0.5.md — expanded from 22 to 29 sections. Rewritten Section 19 (Bagikan → AI-00 triage with morphing context bar). Updated Section 13 (Tandang tier ◆◇ + compact pip, I/C/J axes, GDF Weather, CV Hidup). Updated Section 14 (Rahasia 4-level overlay L0–L3 from C1). Updated Section 22 (10 AI touch points). New sections: 23 Catatan Saksi vault (5 states, seal bar, wali, publish, pola), 24 Siaga Broadcast (4 states, broadcast bar), 25 Navigation & Feed (5-tab nav, header, scope 7 levels, Community Pulse, feed priority 5 levels), 26 Design Tokens (cross-ref DESIGN-DNA), 27 Dual-Tab Pattern, 28 LLM ↔ UI Architecture (7 blocks, source tags, 4 triggers, diff card), 29 ESCO Skill System (validated vs declared). 20/20 gap coverage verified.

---

*Phase D complete. All steps locked: D1 (Design DNA) + D2 (Style Guide) + D3 (AI-SPEC v0.2 + UI-UX-SPEC v0.5). C6 (Share Sheet) added post-D3 lock as cross-cutting addition.*

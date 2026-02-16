> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

### 8.2 Sistem Privasi Rahasia

4 tingkat privasi diterapkan pada seed card apa pun. Level overlay yang meningkatkan penyembunyian identitas dan konten.

| Level | Nama | Author | Konten | Media | Reversibilitas |
|---|---|---|---|---|---|
| **L0** | Terbuka | Nama lengkap | Publik | Penuh | — |
| **L1** | Terbatas | Nama lengkap | Hanya terverifikasi | Penuh | Reversibel |
| **L2** | Rahasia | Anonim (abu) | Gate request-access | Blur | Naik: reversibel, Turun: IRREVERSIBEL |
| **L3** | Sangat Rahasia | Tersembunyi | Redaksi hatched | Hatched | IRREVERSIBEL identitas |

**Dalam Peninjauan:** Author melihat konten + timer. Moderator melihat semua + approve/reject/escalate. Publik melihat placeholder netral. Auto-release: <60% confidence, 15 menit timeout.

### 8.3 AI Surface States

Prinsip **"AI is Furniture"** — AI ambient, tanpa warna brand khusus.

#### 8.3.1 Band Keyakinan

| Band | Range | Warna | Perilaku | UI |
|---|---|---|---|---|
| **Tinggi** | ≥80% | Hijau | Auto-apply | Badge: 🤖 + "Tuntaskan · 92%" |
| **Sedang** | 50-79% | Amber | Suggest + confirm | Chip: 🤖 + "Wujudkan? · 74%" |
| **Rendah** | <50% | Abu | Show uncertainty | Indikator: ⚠ + fallback manual |

#### 8.3.2 Katalog Badge AI (8 Tipe)

| Badge | Makna | Lokasi | Warna |
|---|---|---|---|
| 🤖 Classified | Auto-classified + confidence | Footer kartu | Hijau |
| 🤖 Suggested | Di bawah threshold | Footer kartu | Amber |
| ⚠ Stalled | AI-04 stall detection | Footer kartu | Merah |
| 🌱 Dampak | AI-06 impact measured | Footer kartu | Hijau |
| 📝 Summary | AI-07 ringkasan tersedia | Footer kartu | Biru |
| ⚠ Duplicate | AI-03 duplikat terdeteksi | Footer kartu | Oranye |
| ⚠ Resolve | AI-05 resolution check | Footer kartu | Oranye |
| 📊 Sentiment | AI-08 analisis sentimen | Inline chat | Biru |

#### 8.3.3 Diff Card (Suggest-Don't-Overwrite)

Saat LLM menghasilkan update: untuk **list** = "Ditambah 2 item, dicentang 1" + kutipan bukti. Untuk **dokumen** = tracked-changes. Untuk **form** = perbandingan per field. Aksi: **[Terapkan Semua] | [Tinjau Satu-satu] | [Abaikan]**. Protected fields (finansial, identitas) DIKECUALIKAN dari akses AI.

#### 8.3.4 Moderasi & Duplikat

**Moderation hold (3 perspektif):** Author melihat status + timer countdown. Moderator melihat queue + flags + confidence score + approve/reject/escalate. Publik melihat placeholder netral. Auto-release: <60% confidence setelah 15 menit timeout.

**AI-03 Duplicate detection:** Context bar pill (pra-submission), comparison card (side-by-side, similarity bar), high (≥80% blocking) vs low (<50% non-blocking) thresholds. Aksi: merge / different / view existing.

### 8.4 Arsitektur LLM ↔ UI

Model dua lapis: conversation layer (tab chat) + structured layer (tab rangkuman/papan). AI tidak pernah auto-apply; selalu suggest via diff card.

#### 8.4.1 Invarian Inti

1. **Dua lapis data**: percakapan + terstruktur.
2. **AI tidak auto-apply**: selalu suggest via diff card, manusia memutuskan.
3. **Human edit = lock**: saat manusia edit item AI, source flip `"ai"` → `"human"`, AI berhenti menyentuh.
4. **Additive-first**: AI menambah, menyarankan, mendraft. AI tidak bisa menghapus atau overwrite konten human.

#### 8.4.2 7 Block Primitives

| Block | Render Sebagai | Aturan AI | Source Tag |
|---|---|---|---|
| `list` | Checklist, tabel, timeline, gallery | Additive. Bisa nested. Status-changeable. | Per-item |
| `document` | Rich text + tracked changes | AI draft, human edit sections. | Per-section |
| `form` | Input fields berlabel | AI suggest per field. Protected = hands-off. | Per-field |
| `computed` | Read-only (progress bar, status) | System-derived. Nobody edits. | `system` |
| `display` | Kartu presentasi (recognition) | One-way render. No edit. | `system` |
| `vote` | Interface voting + tally | System tallies. Not AI. | `system` |
| `reference` | Preview kartu linked | Links to other cards, cross-card. | `reference` |

#### 8.4.3 Source Tags

| Tag | Arti | Aturan |
|---|---|---|
| `"ai"` | LLM-generated | Bisa di-overwrite pass berikutnya atau human edit |
| `"human"` | Human-created/edited | AI berhenti menyentuh. Locked. |
| `"system"` | System-computed (vote, progress) | Nobody edits. |

#### 8.4.4 4 Trigger Modes

| Mode | Kapan | AI Touch Points | Output |
|---|---|---|---|
| **Manual** | Tombol 🔄 Perbarui (badge "12 pesan baru") | AI-07 summarization | Diff card di tab terstruktur |
| **Milestone** | Deteksi keyword/pattern di breakpoints | AI-05 resolution check | Saran transisi stage |
| **Time-Triggered** | Interval terjadwal | AI-04 stall (inaktif), AI-06 Dampak (30 hari) | Alert di tab chat |
| **Passive** | Monitoring kontinu | AI-08 sentimen, AI-03 anomali Galang | Badge/indikator saja |

---

## Section 9: Peta File

20 file HTML referensi inti (locked) + 4 referensi tambahan.

| Fase | File | Konten Utama |
|---|---|---|
| A | `A1-mood-vibe.html` | 3 mood × 3 fase. **Tanah terpilih.** |
| A | `A2-color-palette.html` | Token warna lengkap, WCAG AA verified |
| A | `A3-typography-spacing.html` | Skala tipe, spasi, radii, shadow |
| A | `A4-core-components.html` | Atom: buttons, badges, inputs, avatars, pills, indicators |
| A+ | `A+1-triage-screen.html` | AI-00 conversational + morphing context bar, 8-state |
| A+ | `A+2-catatan-saksi.html` | Vault UI gelap, 5-state lifecycle, seal bar |
| A+ | `A+3-siaga-broadcast.html` | Siaga UI merah, 4-state lifecycle, broadcast bar |
| B | `B0-seed-card.html` | Anatomi kartu universal, 5 views |
| B | `B1-tuntaskan-card.html` | Tuntaskan: 6 state, dual-tab, Papan GR, LLM architecture |
| B | `B2-wujudkan-card.html` | Wujudkan: 7 state, milestone tracker, Galang, Rayakan |
| B | `B3-telusuri-card.html` | Telusuri: 5 state, hypothesis cards, evidence board, Temuan |
| B | `B4-rayakan-card.html` | Rayakan: 4 state (+ panel Dampak opsional post-completion), validation, recognition, appreciation wall |
| B | `B5-musyawarah-card.html` | Musyawarah: 6 state, position board, vote, Ketetapan |
| C | `C5-tandang-credit.html` | Tandang: kredit, tier, vouch, GDF Weather, CV Hidup |
| C | `C1-rahasia-overlays.html` | Rahasia: 4 level, gerbang akses, media redaction |
| C | `C2-ai-surface-states.html` | AI Surface: confidence, badges, diff card, moderasi |
| C | `C3-navigation-feed.html` | Navigasi: 5 tab, scope, feed, profil CV Hidup, search |
| C | `C4-catatan-saksi-feed.html` | Vault feed: Kronologi, Detail, Wali, Publish, Pola |
| C | `C6-share-sheet.html` | Berbagi eksternal, OG preview, undangan komunitas |
| D | `D2-style-guide.html` | Kumpulan komponen final + 40/40 verifikasi |

## Referensi Tambahan

| Fase | File | Konten Utama |
|---|---|---|
| R&D | `card-faces.html` | Referensi cepat untuk varian komponen kartu |
| R&D | `entity-evolution.html` | Evolusi entitas dan overlay transisi |
| R&D | `prototype.html` | Skeleton scaffold aplikasi prototipe |
| R&D | `wireframe.html` | Baseline low-fidelity layout |

---

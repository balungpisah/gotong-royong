> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 5: Sistem Kartu

### 5.1 Anatomi Seed Card (Universal)

Semua seed card mengikuti struktur 6-bagian yang sama, terlepas dari track:

| Bagian | Komponen | Keterangan |
|---|---|---|
| **1. Track Strip** | Border kiri 4px, warna track | Identitas visual, tidak pernah berubah |
| **2. Header** | Seed badge + Rahasia badge + judul (maks 2 baris) + author row (avatar, nama, tier, waktu) | Author row: avatar-sm + nama + badge tier + timestamp |
| **3. Body** | Teks 3 baris (clamp) + "...selengkapnya" + thumbnail media + tag lokasi + PIC row + Dampak row | PIC row dan Dampak row opsional per state |
| **4. Skill Tags** | ESCO-ID pill (auto-tagged AI-00, editable PIC) | Dinyatakan ○ (outlined) vs Tervalidasi ● (filled) |
| **5. Stepper** | Breadcrumb horizontal: dot current (track-colored) + nama stage Indonesia | Done = muted, Future = empty circle |
| **6. Footer** | 💬 komentar · 👥 pendukung · ⏱ waktu-di-stage · 🤖 AI badge | AI badge di kanan (right-aligned) |

**AI Badge Variants (Footer, Right-Aligned):**

- 🤖 **Classified** (green): "🤖 Tuntaskan · 92%"
- 🤖 **Suggested** (orange): "🤖 Wujudkan? · 74%"
- ⚠ **Stalled** (red): "⚠ Macet 48j"
- 🌱 **Dampak** (green): "🌱 Dampak"
- 📝 **Summary** (blue): "📝 Ringkasan"
- ⚠ **Duplicate** (orange): "⚠ Duplikat"

### 5.2 Pola Dual-Tab (Cross-cutting)

Setiap stage non-bookend memiliki 2 tab yang bisa diswipe: tab konten terstruktur + tab percakapan. Bar tab di bawah app bar, underline warna track pada tab aktif, hint "‹ geser ›". Notification dot merah 6px saat konten belum dibaca di tab lain.

Chat menggunakan gaya WhatsApp: `.chat-bubble.other` (kiri, putih) dan `.chat-bubble.self` (kanan, track-soft). AI inline card muncul sebagai kartu khusus dalam alur chat.

**Contoh pemetaan tab (Tuntaskan):**

| Stage | Tab 1 | Tab 2 |
|---|---|---|
| Bahas | 💬 Diskusi | 📋 Rangkuman |
| Rancang | 📋 Papan GR | 💬 Koordinasi |
| Garap | ✅ Progres | 💬 Koordinasi |
| Periksa | 📊 Laporan | 💬 Tanggapan |

### 5.3 Ringkasan Track

5 track dengan lifecycle dan komponen unik masing-masing:

#### Tuntaskan (Selesaikan Masalah)

Warna: `#C05621` (oranye-coklat). Lifecycle 6 state: **Keresahan → Bahas → Rancang → Garap → Periksa → Tuntas.**

Komponen unik: Papan Gotong Royong (task list), sistem Kontribusi (slot anonim), PIC-controlled readiness checklist, Periksa (peer verification). Juga mengunci **LLM ↔ UI Architecture**: 7 block primitives, LLM-driven composition, 3 source tags, 4 trigger modes, Suggest-Don't-Overwrite diff card.

#### Wujudkan (Wujudkan Ide)

Warna: `#2E7D32` (hijau). Lifecycle 7 state: **Gagasan → Bahas → Rancang → Galang → Garap → Rayakan → Tuntas.**

Komponen unik: Milestone tracker (hierarki E4), Galang sub-lifecycle (E6 financial fields terlindungi: target/terkumpul/sisa dengan badge 🔒 DILINDUNGI), Rayakan celebration display.

#### Telusuri (Teliti Pertanyaan)

Warna: `#6A1B9A` (ungu). Lifecycle 5 state: **Pertanyaan → Dugaan → Uji → Temuan → Tuntas.**

Komponen unik: Hypothesis cards (5 state: Diajukan/Diuji/Terbukti/Ditolak/Belum Jelas), evidence board dengan indikator dukungan/sanggahan/netral, Temuan document dengan confidence meter, track-change suggestion card (E5 cross-card).

#### Rayakan (Rayakan Pencapaian)

Warna: `#F57F17` (amber). Lifecycle 4 state: **Kabar Baik → Sahkan → Apresiasi → Tuntas**.

Komponen unik: Validation panel (endorsement threshold), recognition card (badge pencapaian + statistik), appreciation wall (pesan komunitas + reaksi emoji), **Dampak panel** (opsional, post-Tuntas; E3 time-trigger, AI-06, before/after comparison).

#### Musyawarah (Musyawarah Keputusan)

Warna: `#4E342E` (coklat). Lifecycle 6 state: **Usul → Bahas → Putuskan → Jalankan → Tinjau → Tuntas** (Tinjau opsional).

Komponen unik: Position board (opsi + bar dukungan + chip pendukung), vote panel (vote block system-authoritative, quorum bar, timer deadline, notice anonim & immutable), Ketetapan document (keputusan formal bernomor, E1+E5), Tinjau review panel (E3 time-trigger).

---


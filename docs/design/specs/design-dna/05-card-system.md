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
| **5. Stepper** | Breadcrumb horizontal: dot current (track-colored) + nama phase/checkpoint | Completed = muted, Planned = empty circle |
| **6. Footer** | 💬 komentar · 👥 pendukung · ⏱ waktu-di-stage · 🤖 AI badge | AI badge di kanan (right-aligned) |

**AI Badge Variants (Footer, Right-Aligned):**

- 🤖 **Classified** (green): "🤖 Tuntaskan · 92%"
- 🤖 **Suggested** (orange): "🤖 Wujudkan? · 74%"
- ⚠ **Stalled** (red): "⚠ Macet 48j"
- 🌱 **Dampak** (green): "🌱 Dampak"
- 📝 **Summary** (blue): "📝 Ringkasan"
- ⚠ **Duplicate** (orange): "⚠ Duplikat"

### 5.2 Pola Dual-Tab (Cross-cutting)

Setiap phase memiliki 2 tab yang bisa diswipe: tab konten terstruktur + tab percakapan. Bar tab di bawah app bar, underline warna track pada tab aktif, hint "‹ geser ›". Notification dot merah 6px saat konten belum dibaca di tab lain.

Chat menggunakan gaya WhatsApp: `.chat-bubble.other` (kiri, putih) dan `.chat-bubble.self` (kanan, track-soft). AI inline card muncul sebagai kartu khusus dalam alur chat.

**Contoh pemetaan tab (Adaptive Path):**

| Phase | Tab 1 | Tab 2 |
|---|---|---|
| Pembahasan | 💬 Diskusi | 📋 Rangkuman |
| Perencanaan | 📋 Papan GR | 💬 Koordinasi |
| Pelaksanaan | ✅ Progres | 💬 Koordinasi |
| Verifikasi | 📊 Laporan | 💬 Tanggapan |

### 5.3 Adaptive Path & Track Hints

> **Updated 2026-02-16.** Fixed track lifecycles are superseded by Adaptive Path Guidance. Tracks remain as optional color-coded hints for visual identity.

The LLM proposes case-specific phases and checkpoints. Track hints provide visual theming only:

| Track Hint | Warna | Spirit |
|---|---|---|
| tuntaskan | `#C05621` (oranye-coklat) | Selesaikan masalah |
| wujudkan | `#2E7D32` (hijau) | Wujudkan ide |
| telusuri | `#6A1B9A` (ungu) | Teliti pertanyaan |
| rayakan | `#F57F17` (amber) | Rayakan pencapaian |
| musyawarah | `#4E342E` (coklat) | Musyawarah keputusan |

**Reusable UI components** (available in any adaptive phase):

- **Papan Gotong Royong** (task board) — execution phases
- **Galang** sub-lifecycle — resource-pooling phases (6 financial fields protected 🔒)
- **Hypothesis cards & evidence board** — investigation phases
- **Validation panel & appreciation wall** — celebration phases
- **Position board & vote panel** — governance phases
- **Ketetapan** — formal decision document output
- **LLM ↔ UI Architecture** — 7 block primitives, LLM-driven composition, 3 source tags (`ai`, `human`, `system`), 4 trigger modes, Suggest-Don't-Overwrite diff card

---


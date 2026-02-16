> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

# Gotong Royong — Design DNA v0.1

> Panduan Sistem Desain · Referensi Formal untuk Tim Desain & Pengembangan
>
> Februari 2026 · Versi 0.1 · DOKUMEN INTERNAL

---

## Daftar Isi

1. [Filosofi & Prinsip](#section-1-filosofi--prinsip)
2. [Sistem Warna](#section-2-sistem-warna)
3. [Tipografi & Spasi](#section-3-tipografi--spasi)
4. [Komponen Inti](#section-4-komponen-inti)
5. [Sistem Kartu](#section-5-sistem-kartu)
6. [Alur Entry](#section-6-alur-entry)
7. [Navigasi & Jangkauan](#section-7-navigasi--jangkauan)
8. [Pola Lintas-Fitur](#section-8-pola-lintas-fitur)
9. [Peta File](#section-9-peta-file)
10. [Lampiran: Referensi Cepat Token](#lampiran-referensi-cepat-token)

---

## Section 1: Filosofi & Prinsip

### 1.1 Mood Desain: Tanah

Gotong Royong menggunakan identitas visual "Tanah" — hangat, earthy, community-first. Palet warna terinspirasi tanah Indonesia: coklat gelap untuk teks, oranye tanah untuk aksi, krem lembut untuk latar. Setiap elemen visual dirancang untuk mengundang partisipasi, bukan mengintimidasi.

Font utama: **Nunito** (Major Third scale). Bentuk membulat (border-radius tinggi). Shadow hangat (brown-tinted). Semua token menggunakan nama Bahasa Indonesia.

### 1.2 Prinsip Inti (13 Prinsip)

| Prinsip | Deskripsi |
|---|---|
| **AI is Furniture** | AI bersifat ambient, tanpa branding khusus. Menggunakan palet semantik Tanah, bukan warna AI tersendiri. Terintegrasi seamless ke alur kerja. |
| **Addiction to Contribution** | Semua lever engagement (streak, skill match, progress ring, Dampak) terikat pada AKSI pengguna, bukan konten yang dilihat. Mendorong partisipasi aktif. |
| **Zero Tandang (Vault/Siaga)** | Catatan Saksi tersegel dan siaran Siaga menghasilkan NOL kredit reputasi selama tersegel/aktif. Reputasi hanya terakumulasi jika dipublikasi. |
| **Suggest-Don't-Overwrite** | LLM tidak pernah auto-apply. Semua saran muncul sebagai diff card dengan kutipan bukti. Manusia selalu memutuskan. |
| **Source-Tagged Data** | Setiap konten terstruktur ditag origin: `ai` / `human` / `system`. AI menghormati edit manusia (berhenti menyentuh item human-sourced). |
| **Track Color = Identity** | Warna aksen track tetap konstan sepanjang lifecycle. Tidak berubah dengan state atau Rahasia level. |
| **Speed Over Ceremony (Siaga)** | Siaga broadcast dirancang untuk aksi instan. Layar minimal, satu ketuk, auto-lokasi. Dibangun untuk respons krisis. |
| **Context Carries Over** | Percakapan AI-00 triage menjadi pesan pertama di tab Percakapan. Cerita saksi tidak hilang atau diulang. |
| **Single Entry Point** | Semua entry via halaman Bagikan dengan AI-00 conversational triage. Tidak ada textarea kosong. AI menyapa, menyelidiki, context bar morfing. |
| **Reputation is Area-Aware** | Skor Tandang dapat dilihat per tingkat scope (RT→Nasional). Pengguna bisa menjadi pahlawan lokal di RT tapi pemula di provinsi. |
| **GDF Weather = Difficulty Floor** | Cuaca Komunitas: Cerah/Berawan/Hujan/Badai. Semakin sulit → bonus multiplier pada kredit Kompetensi (C). Mendorong kontribusi saat sulit. |
| **No Separate Credit Screen** | Kredit diperoleh di dalam aktivitas yang ada (pelaksanaan, pembahasan, dll), bukan alur terpisah. Visibilitas via toast + nudge AI + diff card. |
| **Strict Wali Permissions** | Wali hanya bisa: baca, surface-with-consent. Tidak bisa: edit, bagikan tanpa izin. Menjamin privasi vault. |

---


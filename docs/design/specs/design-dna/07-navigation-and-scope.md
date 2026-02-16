> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 7: Navigasi & Jangkauan

### 7.1 Bottom Navigation (5 Tab)

| Tab | Ikon | Label | Fungsi |
|---|---|---|---|
| 1 | 🏠 | **Beranda** | Feed komunitas: semua seed, Community Pulse, filter track horizontal |
| 2 | 📝 | **Catatan** | Catatan Komunitas: catatan publik ringan (harga, status, jadwal), concept pills, progressive disclosure (S3-B4) |
| 3 | 🤝 | **Bantu** | Peluang sesuai keahlian ESCO pengguna. Pill validated ● vs declared ○. Jumlah sukarelawan. |
| 4 | 🔔 | **Notifikasi** | Dikelompokkan waktu (Hari Ini/Kemarin/Minggu Ini). 7 tipe: skill-match, credit, mention, stage, vote, stall, digest. |
| 5 | ☰ | **Lainnya** | Menu hamburger: CV Hidup (Profil), Terlibat, Template Saya (S3-C3), Pengaturan |

> **Perubahan S3-MD3:** Catatan Komunitas menggantikan Terlibat sebagai tab utama (penghalang masuk terendah, terbaik untuk akuisisi pengguna). Terlibat dan Profil pindah ke menu hamburger.

### 7.2 App Header

Header sticky di semua halaman:

```
[scope ▼]    Gotong Royong    [🔍] [+]
```

**Scope selector** (kiri): area saat ini, misal "RT 05 ▼" → buka area picker sheet. **Search 🔍** (kanan): overlay pencarian layar penuh dengan filter track + ESCO skill + waktu. **Compose [+]** (kanan): buka AI-00 triage (alur Bagikan).

### 7.3 Hierarki Jangkauan (7 Level)

| Level | Nama | Contoh | Perkiraan Ukuran |
|---|---|---|---|
| 7 | Nasional | Indonesia | 275 juta |
| 6 | Provinsi | Jawa Barat | ~50 juta |
| 5 | Kota / Kabupaten | Kota Depok | ~2 juta |
| 4 | Kecamatan | Cimanggis | ~200 ribu |
| 3 | Kelurahan / Desa | Tugu | ~15 ribu |
| 2 | RW | RW 03 | ~1.000 |
| 1 | RT | RT 05 | ~150 |

Default scope: RT terdaftar pengguna. Bisa perluas ke atas. Seed mewarisi scope author saat pembuatan. Reputasi area-aware: skor Tandang terlihat per level.

### 7.4 Community Pulse Bar

Di header Beranda: `☀️ Cerah · 14 aktif · 3 baru · 1 vote`. Menampilkan GDF Weather emoji + statistik live. Bisa diketuk untuk detail.

4 status cuaca: Cerah ☀️ (mudah), Berawan 🌤️ (sedang), Hujan 🌧️ (sulit), Badai ⛈️ (sangat sulit). Semakin sulit = bonus multiplier pada kredit Kompetensi.

### 7.5 Urutan Feed (Action-Weighted)

5 level prioritas di Beranda:

| Prioritas | Kondisi | Contoh |
|---|---|---|
| 1 — Your Action | Seed butuh aksi dari pengguna | PIC menugaskan Anda, vote terbuka |
| 2 — Nearing | Deadline/milestone dekat | Pelaksanaan H-3, vote 2 jam lagi |
| 3 — New | Baru dibuat (24 jam) | Seed baru di RT Anda |
| 4 — Active | Aktivitas terkini | Diskusi berlangsung |
| 5 — Completed | Selesai | Seed yang sudah selesai |

---


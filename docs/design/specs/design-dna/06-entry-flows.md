> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 6: Alur Entry

Semua entry dimulai dari AI-00 conversational triage di halaman **Bagikan**. AI-00 proposes an adaptive path plan, then directs to one of 3 flows.

### 6.1 AI-00 Conversational Triage

Pengguna tidak melihat textarea kosong. AI-00 menyapa, mendengarkan cerita, menyelidiki jika perlu, lalu menghasilkan adaptive path plan. Context bar di atas keyboard berubah bentuk (morph) sesuai state percakapan.

| State Bar | Visual | Penjelasan |
|---|---|---|
| **Listening** | Bar kosong, indikator gelombang | AI sedang mendengarkan, belum ada klasifikasi |
| **Probing** | Bar + signal indicator | AI mengirim follow-up untuk klarifikasi |
| **Leaning** | Pill track bisa diketuk | AI punya kecenderungan awal (tappable preview) |
| **Ready** | Kartu penuh: track + confidence | Path plan dihasilkan, siap submit ke Komunitas |
| **Vault-ready** | Kartu vault (gelap) | Cerita diarahkan ke Catatan Saksi |
| **Siaga-ready** | Kartu siaga (merah, pulse) | Darurat terdeteksi, arahkan ke Siaga |
| **Split-ready** | Kartu terbelah | Cerita bisa dipecah ke 2 alur (dengan peringatan linkability) |
| **Manual** | Grid 5 tracks + vault | Pengguna tekan "Pilih sendiri" untuk pilih track hint manual |

### 6.2 Alur Komunitas

Context bar `status="ready"` + track terpilih. Percakapan AI-00 menjadi pesan pertama di tab Percakapan. Tahapan tab menampilkan path plan. Plan card terbuat dengan tag ESCO dari AI-00, confidence badge, dan track hint sesuai klasifikasi.

**Dampak reputasi: PENUH** — semua kontribusi di lifecycle menghasilkan kredit Tandang sesuai tipe (A–E).

### 6.3 Alur Catatan Saksi (Vault)

Context bar `mode="vault"`. Masuk ke palet gelap vault.

| State | UI | Tujuan | Seal |
|---|---|---|---|
| **Menyimpan** | Compose: teks pra-isi dari AI-00, tool lampiran | Tulis catatan saksi | Unsealed |
| **Tersegel** | Hash SHA-256, timestamp, badge terenkripsi | Bukti tamper-proof | Sealed 🔒 |
| **Wali** | Cari trustee, daftar izin, badge tier | Tunjuk orang tepercaya | Sealed 🔒 |
| **Terbitkan** | Peringatan oranye, 3 konsekuensi, track picker | Publish ke komunitas (IRREVERSIBLE) | Sealed → Published |
| **Pola** | AI pattern detection, alert lembut, resource links | Deteksi pola kekerasan/eksploitasi | Sealed 🔒 |

**Dampak reputasi: NOL selama tersegel.** Hanya jika diterbitkan (Terbitkan) ke Komunitas, kredit mulai terakumulasi.

**Seal bar:** bar bawah yang berubah antara unsealed (bisa edit) dan sealed (terkunci + tombol aksi: Ganti Wali / Terbitkan).

### 6.4 Alur Siaga (Darurat)

Context bar `mode="siaga"`. Speed over ceremony — layar minimal.

| State | UI | Elemen Real-time |
|---|---|---|
| **Kirim** | Compose: teks pra-isi, chip jenis darurat, kartu lokasi-otomatis, link 112 | Satu ketuk: Siarkan Sekarang |
| **Aktif** | Kartu live dengan shimmer, stats real-time, input update cepat, timeline event | terjangkau / melihat / merespons |
| **Respons** | Kartu responder (jarak/ETA/status), tombol quick-respond, timeline | Responder cards update real-time |
| **Selesai** | Konfirmasi deliberate, summary (durasi/responder/layanan), pesan terima kasih | Bar resolved hijau |

**Dampak reputasi: NOL.** Broadcast bar di bawah morph antara composing/active/resolved. Link 112 selalu terlihat.

---


> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 2: Sistem Warna

Sistem warna Gotong Royong terdiri dari 46+ token warna yang dikelompokkan dalam 8 kategori. Setiap token memiliki nama Indonesia dan fungsi spesifik.

### 2.1 Palet Tanah (Inti)

7 warna inti yang membentuk identitas visual utama.

| Token | Hex | Usage |
|---|---|---|
| `--c-tanah-gelap` | `#3E2723` | Teks utama, heading |
| `--c-tanah` | `#5D4037` | Teks sekunder |
| `--c-kayu` | `#8D6E63` | Caption, meta, label |
| `--c-pasir` | `#A1887F` | Placeholder, disabled |
| `--c-batu` | `#D7CCC8` | Border, divider |
| `--c-kapas` | `#F5EDE3` | Latar input |
| `--c-susu` | `#FFFBF5` | Latar kartu |

### 2.2 Palet Aksi (Api)

4 warna untuk elemen interaktif utama.

| Token | Hex | Usage |
|---|---|---|
| `--c-api` | `#C05621` | Tombol primer, aksi utama |
| `--c-api-terang` | `#D2691E` | Hover state |
| `--c-api-dalam` | `#A0461A` | Pressed/active state |
| `--c-bara` | `#FFF3E0` | Latar sorotan, highlight |

### 2.3 Warna Semantik

8 warna untuk umpan balik sistem (4 pasang: warna + latar lembut).

| Token | Hex | Usage |
|---|---|---|
| `--c-berhasil` | `#2E7D32` | Sukses (ikon, teks) |
| `--c-berhasil-lembut` | `#E8F5E9` | Sukses (latar) |
| `--c-peringatan` | `#E65100` | Peringatan (ikon, teks) |
| `--c-peringatan-lembut` | `#FFF3E0` | Peringatan (latar) |
| `--c-bahaya` | `#C62828` | Error/bahaya (ikon, teks) |
| `--c-bahaya-lembut` | `#FFEBEE` | Error/bahaya (latar) |
| `--c-keterangan` | `#4E342E` | Info (ikon, teks) |
| `--c-keterangan-lembut` | `#EFEBE9` | Info (latar) |

### 2.4 Aksen Track (5 Jalur)

Setiap track memiliki 3 varian: Strong (aksen utama), Soft (latar), Muted (latar sekunder).

| Track | Strong | Soft | Muted | Fungsi |
|---|---|---|---|---|
| **Tuntaskan** | `#C05621` | `#FFF3E0` | `#FFECD2` | Selesaikan masalah |
| **Wujudkan** | `#2E7D32` | `#E8F5E9` | `#C8E6C9` | Wujudkan ide |
| **Telusuri** | `#6A1B9A` | `#F3E5F5` | `#E1BEE7` | Teliti pertanyaan |
| **Rayakan** | `#F57F17` | `#FFF8E1` | `#FFECB3` | Rayakan pencapaian |
| **Musyawarah** | `#4E342E` | `#EFEBE9` | `#D7CCC8` | Musyawarah keputusan |

### 2.5 Palet Vault (Catatan Saksi)

6 warna skala biru-abu gelap untuk ruang pribadi vault.

| Token | Hex | Usage |
|---|---|---|
| `--v-deep` | `#263238` | Header vault |
| `--v-surface` | `#37474F` | Permukaan vault |
| `--v-mid` | `#546E7A` | Teks vault |
| `--v-soft` | `#78909C` | Meta vault |
| `--v-light` | `#B0BEC5` | Ikon gembok |
| `--v-wash` | `#ECEFF1` | Latar kartu vault |

### 2.6 Palet Siaga (Darurat)

5 warna merah untuk mode darurat.

| Token | Hex | Usage |
|---|---|---|
| `--s-deep` | `#B71C1C` | Header siaga |
| `--s-pulse` | `#D32F2F` | Ikon, animasi pulse |
| `--s-accent` | `#FF5252` | Aksen siaga |
| `--s-soft` | `#FFEBEE` | Latar kartu siaga |
| `--s-border` | `#FFCDD2` | Border siaga |

### 2.7 Level Rahasia

4 tingkat privasi dengan warna progresif.

| Level | Nama | Hex | Deskripsi |
|---|---|---|---|
| L0 | Terbuka | — | Tanpa overlay, konten publik |
| L1 | Terbatas | `#8D6E63` | Badge saja, akses terverifikasi |
| L2 | Rahasia | `#5D4037` | Anonim, media blur, gerbang akses |
| L3 | Sangat Rahasia | `#3E2723` | Redaksi penuh, hatched pattern, tak terlihat di feed |

### 2.8 Palet Tandang (Reputasi)

Warna teal untuk sistem reputasi dan 3 sumbu skor I/C/J.

| Token | Hex | Usage |
|---|---|---|
| `--c-tandang` | `#00695C` | Warna utama Tandang (teal) |
| I (Inisiatif) | `#F57F17` | Sumbu Inisiatif (amber) |
| C (Kompetensi) | `#00695C` | Sumbu Kompetensi (teal) |
| J (Penilaian) | `#7B1FA2` | Sumbu Penilaian (purple) |

---


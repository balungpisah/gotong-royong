> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 4: Komponen Inti

12 keluarga komponen atom yang menjadi dasar seluruh antarmuka. Semua berukuran sesuai grid spasi dan skala tipografi.

### 4.1 Tombol (Button)

6 varian gaya × 3 ukuran + varian track (5 warna) + mode khusus (vault, siaga).

| Varian | Latar | Teks | Border | Interaksi |
|---|---|---|---|---|
| `btn-primary` | Api (#C05621) | Putih | — | Hover: Api Terang, Active: Api Dalam |
| `btn-secondary` | Kapas (#F5EDE3) | Tanah | — | Hover: Batu bg |
| `btn-outline` | Transparan | Api | Api 1.5px | Hover: Bara bg |
| `btn-ghost` | Transparan | Kayu | — | Hover: Kapas bg |
| `btn-danger` | Bahaya (#C62828) | Putih | — | Hover: #B71C1C |
| `btn-disabled` | Batu (#D7CCC8) | Pasir | — | cursor: not-allowed |

**Ukuran:** btn-lg (32px tinggi, 12px/24px padding), btn-md (14px, 8px/18px), btn-sm (11px, 6px/14px).

**Track buttons:** btn-track-tuntaskan (#C05621), btn-track-wujudkan (#2E7D32), btn-track-telusuri (#6A1B9A), btn-track-rayakan (#F57F17), btn-track-musyawarah (#4E342E). **Mode:** btn-vault (#263238), btn-siaga (#B71C1C).

### 4.2 Badge

Badge untuk track, semantik, stepper, dan mode khusus. Ukuran: 9px (fs-micro), uppercase, 800 weight.

| Kategori | Varian | Warna |
|---|---|---|
| Track | badge-tuntaskan / wujudkan / telusuri / rayakan / musyawarah | Masing-masing warna strong track |
| Semantik | badge-berhasil / peringatan / bahaya / keterangan | Hijau / oranye / merah / coklat |
| Stepper | badge-step (aktif) / badge-step-done / badge-step-future | Api / Berhasil Lembut / Kapas |
| Khusus | badge-rahasia / badge-vault / badge-siaga | Gelap / Vault / Merah (pulse) |
| Keyakinan | badge-confidence | Kapas bg, Kayu teks, 9px |

### 4.3 Input

4 state: default, focus, error, disabled. Plus compose textarea.

| State | Latar | Border | Efek |
|---|---|---|---|
| Default | Kapas | Batu 1.5px | Placeholder: Pasir |
| Focus | Susu | Api 1.5px | Shadow ring Api, teks Tanah Gelap |
| Error | Bahaya Lembut | Bahaya 1.5px | Pesan error di bawah |
| Disabled | Batu | — | Teks Pasir, cursor not-allowed |

Elemen terkait: input-label (11px, bold), input-hint (11px, gray), input-error (11px, Bahaya, semibold), textarea-field (min-height 80px).

### 4.4 Avatar

| Ukuran | Dimensi | Font | Keterangan |
|---|---|---|---|
| `avatar-xs` | 20px | 8px | Inline, notifikasi |
| `avatar-sm` | 28px | 10px | List item, komentar |
| `avatar-md` | 36px | 13px | Kartu, header |
| `avatar-lg` | 48px | 17px | Profil, detail |
| `avatar-xl` | 64px | 22px | Hero, profil utama |

Varian: avatar-group (tumpuk, -8px overlap, border Susu), avatar-tier (badge tier di kanan-bawah). Default bg: Api Terang. Border-radius: r-full.

### 4.5 Pill / Tag

| State | Latar | Teks | Keterangan |
|---|---|---|---|
| `pill-active` | Bara | Api | Filter aktif |
| `pill-default` | Kapas | Tanah | Filter tidak aktif |
| `pill-success` | Berhasil Lembut | Berhasil | Status sukses |
| `pill-removable` | Kapas + × | Tanah | Bisa dihapus |
| `pill-track` | Warna track | Putih | 5 varian track |

11px (fs-caption), bold, padding 4px/12px, border-radius r-full.

### 4.6 Indikator Status

| Status | Warna Dot | Animasi | Keterangan |
|---|---|---|---|
| `status-active` | Berhasil | — | Seed aktif |
| `status-stalled` | Peringatan | — | Macet/stall |
| `status-tuntas` | Batu | — | Selesai |
| `status-review` | Api | Blink 1.5s | Dalam peninjauan |
| `status-moderation` | Bahaya | Blink 1.5s | Ditahan moderasi |
| `status-sealed` | Vault | — | Tersegel (vault) |

Dot: 8px round. Font: 11px, bold. Gap: 6px antara dot dan label.

### 4.7 Progress & Confidence Bar

Progress bar: 6px tinggi, bg Kapas, fill Api (default) atau Berhasil (task). Confidence bar: 4px tinggi, bg Batu, fill berdasarkan skor (Api tinggi, Peringatan rendah). Label persentase 11px.

### 4.8 Toggle, Tooltip, Divider

**Toggle:** 40×22px, off=Batu bg, on=Api bg, lingkaran putih 16px, transisi 0.2s.

**Tooltip:** bg Tanah Gelap, teks putih 11px semibold, padding 5px/10px, radius r-sm, shadow-md, arrow 4px.

**Divider:** 1px Batu, margin vertikal sp-3.

---


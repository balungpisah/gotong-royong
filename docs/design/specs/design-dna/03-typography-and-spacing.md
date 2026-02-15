> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 3: Tipografi & Spasi

### 3.1 Skala Tipe (Nunito, Major Third 1.250)

8 langkah dari Micro (9px) sampai Display (32px). Rasio Major Third memberikan hierarki yang jelas tanpa lompatan drastis.

| Token | Ukuran | Rem | Weight | Fungsi |
|---|---|---|---|---|
| `--fs-display` | 32px | 2.000rem | 800 (Extra) | Hero number, splash |
| `--fs-h1` | 26px | 1.625rem | 800 (Extra) | Judul halaman |
| `--fs-h2` | 20px | 1.250rem | 700 (Bold) | Judul seksi |
| `--fs-h3` | 16px | 1.000rem | 700 (Bold) | Judul kartu, sub-heading |
| `--fs-body` | 14px | 0.875rem | 400 (Regular) | Teks isi utama |
| `--fs-small` | 12px | 0.750rem | 600 (Semi) | Teks kecil, meta |
| `--fs-caption` | 11px | 0.688rem | 600 (Semi) | Caption, label input |
| `--fs-micro` | 9px | 0.563rem | 700 (Bold) | Badge, uppercase label |

### 3.2 Line Height & Font Weight

| Token | Nilai | Fungsi |
|---|---|---|
| `--lh-tight` | 1.2 | Heading, badge |
| `--lh-normal` | 1.5 | Teks isi |
| `--lh-relaxed` | 1.7 | Teks panjang, paragraf |
| `--fw-regular` | 400 | Teks biasa |
| `--fw-semi` | 600 | Teks tebal ringan, caption |
| `--fw-bold` | 700 | Heading, aksen |
| `--fw-extra` | 800 | Display, H1 |

### 3.3 Grid Spasi (Basis 4px)

10 token spasi berdasarkan kelipatan 4px. Digunakan untuk margin, padding, gap.

| Token | Nilai | Fungsi Umum |
|---|---|---|
| `--sp-1` | 4px | Gap ikon-teks terkecil |
| `--sp-2` | 8px | Padding badge, gap internal |
| `--sp-3` | 12px | Padding input, gap kartu internal |
| `--sp-4` | 16px | Padding kartu, margin antar elemen |
| `--sp-5` | 20px | Margin seksi kecil |
| `--sp-6` | 24px | Padding halaman mobile |
| `--sp-8` | 32px | Margin seksi besar |
| `--sp-10` | 40px | Padding hero, header |
| `--sp-12` | 48px | Spacing antar grup major |
| `--sp-16` | 64px | Spacing halaman, header besar |

### 3.4 Border Radius

| Token | Nilai | Fungsi |
|---|---|---|
| `--r-sm` | 6px | Input, badge, chip |
| `--r-md` | 10px | Kartu inner, container |
| `--r-lg` | 14px | Kartu outer, section |
| `--r-xl` | 20px | Modal, overlay |
| `--r-full` | 9999px | Avatar, pill, toggle |

### 3.5 Elevasi / Shadow (Brown-Tinted)

Shadow menggunakan `rgba(62,39,35)` — coklat Tanah Gelap — untuk konsistensi warm tone.

| Token | Nilai | Fungsi |
|---|---|---|
| `--shadow-sm` | `0 1px 4px rgba(62,39,35,0.06)` | Badge, pill |
| `--shadow-md` | `0 2px 12px rgba(62,39,35,0.08)` | Kartu, input |
| `--shadow-lg` | `0 4px 24px rgba(62,39,35,0.12)` | Dropdown, popover |
| `--shadow-xl` | `0 8px 40px rgba(62,39,35,0.16)` | Modal, overlay |

---


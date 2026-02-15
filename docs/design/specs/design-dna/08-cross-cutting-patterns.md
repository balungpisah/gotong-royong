> [← Kembali ke indeks spesifikasi](../DESIGN-DNA-v0.1.md)

## Section 8: Pola Lintas-Fitur

### 8.1 Sistem Kredit Tandang

Tandang adalah mesin kredensial Markov yang mengukur reputasi lewat 3 sumbu: **Inisiatif (I, amber)**, **Kompetensi (C, teal)**, dan **Penilaian (J, purple)**. Kredit terakumulasi dari aksi di Gotong Royong, diproses oleh AI-09.

#### 8.1.1 Tipe Kredit (A–E)

| Tipe | Nama | Model Skor | Contoh Aksi GR | Peran AI-09 |
|---|---|---|---|---|
| **A** | Verifikasi Biner | Did/didn't — pass/fail | Task completion (Garap), voting, seeding | SYSTEM (auto) |
| **B** | Bobot Waktu | Effort over duration | Building di Rancang, kontribusi Galang | SYSTEM (auto) |
| **C** | Konsensus Rekan | Grup validasi kualitas | Validasi (Sahkan), verifikasi (Periksa) | PEERS (human) |
| **D** | Spektrum Kualitas | Rating kualitas | Kualitas diskusi (Bahas), proposal (Usul) | AI-PROPOSED → HUMAN |
| **E** | Jaminan (Stake) | Reputasi dipertaruhkan | Vouching, menjamin | HUMAN (self-initiated) |

#### 8.1.2 Pemetaan Aksi GR → Tandang

| Aksi GR | Sumbu | Tipe | Detail |
|---|---|---|---|
| Submit seed | I+ | A | Inisiatif sipil |
| Diskusi di Bahas | C | D | Kualitas diskusi |
| Kontribusi Rancang | C | B | Effort perencanaan |
| Selesaikan task Garap | C | A | Task completion |
| Validasi di Sahkan | J | C | Akurasi endorsement |
| Vote di Putuskan | I+ | A | Partisipasi sipil |
| Verifikasi Periksa/Tinjau | J | C | Akurasi verifikasi |
| Kontribusi Galang | C | B | Kontribusi sumber daya |
| Vouch (Jaminkan) | I (stake) | E | Risiko proporsional; slash cascade |
| Ajukan hipotesis | C | D | Kualitas dinilai dari outcome |
| Kumpulkan bukti | C | B | Effort riset |

#### 8.1.3 Alur Kredit 5 Langkah

| Langkah | Apa yang Terjadi |
|---|---|
| **1. Silent tracking** | Sistem mencatat setiap aksi (siapa, apa, kapan, durasi). Log aktivitas = buku besar kredit. |
| **2. Instant feedback** | Tipe A/B: toast langsung. Tipe C: toast saat threshold tercapai. Tipe D: setelah PIC konfirmasi. Tipe E: setelah vouch. |
| **3. AI nudge di chat** | AI-09 mengirim pesan inline di tab Diskusi: "💡 Diskusi berkualitas — kontribusi Anda dicatat (Tipe D · Kompetensi)" |
| **4. Ringkasan Tuntas** | Saat kartu Tuntas, AI-09 mengusulkan distribusi Kontribusi sebagai diff card. PIC review: Terapkan / Tinjau / Tolak. |
| **5. Mekanisme sengketa** | Peserta bisa flag kredit otomatis → peer review → AI-09 mediasi. |

#### 8.1.4 Sistem Tier

| Tier | Nama | Badge | Persentil |
|---|---|---|---|
| 4 | Kunci | ◆◆◆◆ | Top 2% |
| 3 | Pilar | ◆◆◆◇ | Top 10% |
| 2 | Kontributor | ◆◆◇◇ | Top 40% |
| 1 | Pemula | ◆◇◇◇ | Bawah 60% |
| 0 | Bayangan | ◇◇◇◇ | Tidak aktif / baru |

#### 8.1.5 GDF Weather Multiplier

Cuaca Komunitas (Global Difficulty Floor) mempengaruhi kredit Kompetensi (C):

| Cuaca | Emoji | Multiplier | Arti |
|---|---|---|---|
| Cerah | ☀️ | 1.0× | Komunitas aktif, banyak kontributor |
| Berawan | 🌤️ | 1.2× | Aktivitas sedang |
| Hujan | 🌧️ | 1.5× | Aktivitas rendah, butuh kontribusi |
| Badai | ⛈️ | 2.0× | Krisis, kontribusi sangat dibutuhkan |


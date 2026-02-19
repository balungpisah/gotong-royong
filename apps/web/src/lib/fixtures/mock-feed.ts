/**
 * Mock feed fixtures for the Pulse event-based feed.
 * 10 feed items covering all event types, urgency badges, sources, and reposts,
 * plus followable entity suggestions for onboarding.
 */

import type {
	FeedItem,
	FeedEvent,
	FeedMemberPreview,
	EntityTag,
	FollowableEntity,
	RepostFrame,
	SystemCardData,
	MyRelation,
	SignalCounts
} from '$lib/types';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();

/** Returns an ISO timestamp N minutes in the past. */
const ts = (minutesAgo: number): string => new Date(now - minutesAgo * 60 * 1000).toISOString();

/** Returns an ISO timestamp N days in the past. */
const tsDay = (daysAgo: number): string => ts(daysAgo * 24 * 60);

// ---------------------------------------------------------------------------
// Reusable member previews
// ---------------------------------------------------------------------------

const memberAhmad: FeedMemberPreview = {
	user_id: 'u-001',
	name: 'Ahmad Hidayat',
	role: 'pelapor'
};

const memberSari: FeedMemberPreview = {
	user_id: 'u-002',
	name: 'Sari Dewi',
	avatar_url: 'https://placehold.co/40x40/2E7D32/white?text=SD',
	role: 'relawan'
};

const memberBudi: FeedMemberPreview = {
	user_id: 'u-003',
	name: 'Budi Santoso',
	role: 'saksi'
};

const memberRina: FeedMemberPreview = {
	user_id: 'u-004',
	name: 'Rina Kartika',
	avatar_url: 'https://placehold.co/40x40/C05621/white?text=RK',
	role: 'koordinator'
};

const memberDewi: FeedMemberPreview = {
	user_id: 'u-005',
	name: 'Dewi Lestari',
	avatar_url: 'https://placehold.co/40x40/6A1B9A/white?text=DL',
	role: 'relawan'
};

// ---------------------------------------------------------------------------
// Reusable entity tags
// ---------------------------------------------------------------------------

const entityRT05: EntityTag = {
	entity_id: 'ent-001',
	entity_type: 'lingkungan',
	label: 'RT 05 Menteng',
	followed: true
};

const entityInfrastruktur: EntityTag = {
	entity_id: 'ent-002',
	entity_type: 'topik',
	label: 'Infrastruktur',
	followed: true
};

const entityKarangTaruna: EntityTag = {
	entity_id: 'ent-003',
	entity_type: 'kelompok',
	label: 'Karang Taruna RT 05',
	followed: false
};

const entitySDN3: EntityTag = {
	entity_id: 'ent-004',
	entity_type: 'lembaga',
	label: 'SD Negeri 3 Menteng',
	followed: false
};

const entityLingkungan: EntityTag = {
	entity_id: 'ent-005',
	entity_type: 'topik',
	label: 'Lingkungan Hidup',
	followed: true
};

const entityRW03: EntityTag = {
	entity_id: 'ent-006',
	entity_type: 'lingkungan',
	label: 'RW 03 Menteng',
	followed: false
};

const entitySembako: EntityTag = {
	entity_id: 'ent-007',
	entity_type: 'topik',
	label: 'Harga Sembako',
	followed: false
};

// ---------------------------------------------------------------------------
// Individual feed items
// ---------------------------------------------------------------------------

/** Returns an ISO timestamp N minutes in the FUTURE. */
const tsFuture = (minutesAhead: number): string =>
	new Date(now + minutesAhead * 60 * 1000).toISOString();

/** 1. Created — PHOTO + LONG BODY — angry, with evidence photo */
export const mockFeedItem1: FeedItem = {
	witness_id: 'witness-feed-001',
	title: 'Lampu Jalan Mati di Gang Melati',
	track_hint: 'tuntaskan',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-001',
		event_type: 'created',
		actor_name: 'Ahmad Hidayat',
		actor_role: 'pelapor',
		timestamp: ts(25),
		verb: 'melaporkan masalah baru',
		snippet: 'Sudah 2 minggu lampu jalan di Gang Melati padam total. Warga merasa tidak aman saat malam.'
	},
	collapsed_count: 0,
	member_count: 1,
	members_preview: [memberAhmad],
	entity_tags: [entityRT05, entityInfrastruktur],
	urgency: 'baru',
	source: 'ikutan',
	hook_line: 'Sudah 2 minggu, gelap total.',
	sentiment: 'angry',
	intensity: 4,
	cover_url: 'https://images.unsplash.com/photo-1504472478235-9bc48ba4d60f?w=600&h=400&fit=crop',
	body: 'Warga Gang Melati mengeluh lampu jalan padam total sejak dua minggu lalu. Ibu-ibu takut pulang malam dari pasar. Pak Ahmad sudah lapor ke kelurahan tapi belum ada respon. "Anak saya harus lewat gang gelap setiap pulang les," katanya.',
	active_now: 3,
	// Auto-pantau: user is saksi (witnessed: true)
	monitored: true,
	// Tandang signals: user witnessed this problem, 2 vouches, 1 skeptic
	my_relation: {
		vouched: false,
		witnessed: true,
		flagged: false,
		quality_voted: false
	},
	signal_counts: {
		vouch_positive: 2,
		vouch_skeptical: 1,
		witness_count: 5,
		quality_avg: 0,
		quality_votes: 0,
		flags: 0
	},
	peek_messages: [
		{ author: 'Sari', text: 'Sudah foto buktinya tadi pagi, gelap banget. Saya kirim ke grup RT tapi belum ada yang respon sampai sekarang.' },
		{ author: 'Ahmad', text: 'Saya coba hubungi kelurahan lagi besok pagi. Kemarin Pak Lurah bilang anggaran penerangan habis, tapi saya mau coba minta alokasi darurat karena ini sudah dua minggu lebih.' },
		{ author: 'Budi', text: 'Gang sebelah juga mati, mungkin satu trafo.' },
		{ author: 'Dewi', text: 'Ibu-ibu pengajian sudah kumpul tanda tangan 47 orang yang terdampak. Mau kita serahkan ke kelurahan bareng?' }
	]
};

/** 2. Joined — BARE CARD — short, punchy, no photo no body */
export const mockFeedItem2: FeedItem = {
	witness_id: 'witness-feed-002',
	title: 'Jalan Rusak Jl. Mawar RT 05',
	track_hint: 'tuntaskan',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-002',
		event_type: 'joined',
		actor_name: 'Dewi Lestari',
		actor_avatar: 'https://placehold.co/40x40/6A1B9A/white?text=DL',
		actor_role: 'relawan',
		timestamp: ts(45),
		verb: 'bergabung sebagai Relawan'
	},
	collapsed_count: 5,
	member_count: 12,
	members_preview: [memberAhmad, memberSari, memberBudi, memberRina, memberDewi],
	entity_tags: [entityRT05, entityInfrastruktur],
	source: 'terlibat',
	hook_line: '12 orang turun tangan.',
	sentiment: 'hopeful',
	intensity: 3,
	active_now: 5,
	// Auto-pantau: user vouched (positive)
	monitored: true,
	// Tandang signals: user vouched positive, many vouches, good quality
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: false,
		flagged: false,
		quality_voted: true
	},
	signal_counts: {
		vouch_positive: 8,
		vouch_skeptical: 0,
		witness_count: 3,
		quality_avg: 4.2,
		quality_votes: 6,
		flags: 0
	},
	peek_messages: [
		{ author: 'Dewi', text: 'Saya bisa bantu survei akhir pekan ini kalau cuaca bagus, tapi kalau hujan mungkin kita tunda ke minggu depan saja ya?' },
		{ author: 'Rina', text: 'Pak RT sudah setuju koordinasi. Beliau minta kita siapkan proposal sederhana dulu sebelum turun ke lapangan biar ada dokumentasinya.' },
		{ author: 'Ahmad', text: 'Siapa yang punya cangkul? Kita butuh minimal 3 buat kerja bakti Sabtu.' },
		{ author: 'Sari', text: 'Saya bawa dari rumah.' },
		{ author: 'Budi', text: 'Kemarin saya lewat situ, lubangnya makin dalam. Motor saya hampir masuk. Tolong cepat ya sebelum ada korban.' }
	]
};

/** 3. Checkpoint — BODY ONLY — longer story, no photo */
export const mockFeedItem3: FeedItem = {
	witness_id: 'witness-feed-003',
	title: 'Pembangunan Taman Warga RW 03',
	track_hint: 'wujudkan',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-003',
		event_type: 'checkpoint',
		actor_name: 'Rina Kartika',
		actor_avatar: 'https://placehold.co/40x40/C05621/white?text=RK',
		actor_role: 'koordinator',
		timestamp: ts(90),
		verb: 'menyelesaikan langkah',
		snippet: 'Survei lokasi selesai — lahan di Jl. Kenari cocok untuk taman bermain anak.'
	},
	collapsed_count: 8,
	member_count: 15,
	members_preview: [memberRina, memberSari, memberAhmad, memberBudi],
	entity_tags: [entityRW03, entityKarangTaruna],
	source: 'terlibat',
	hook_line: 'Lahan di Jl. Kenari cocok — survei selesai.',
	sentiment: 'hopeful',
	intensity: 3,
	body: 'Setelah 3 bulan negosiasi dengan pemilik lahan, akhirnya disepakati pinjam pakai selama 5 tahun. Tim survei Karang Taruna turun langsung mengukur dan memetakan. Rencananya ada area bermain anak, bangku lansia, dan kebun kecil yang dikelola bersama. Bu Rina bilang, "Ini mimpi warga sejak 2019."',
	active_now: 2,
	// Tandang signals: user hasn't interacted, moderate community signals
	my_relation: {
		vouched: false,
		witnessed: false,
		flagged: false,
		quality_voted: false
	},
	signal_counts: {
		vouch_positive: 5,
		vouch_skeptical: 2,
		witness_count: 7,
		quality_avg: 3.8,
		quality_votes: 4,
		flags: 0
	},
	peek_messages: [
		{ author: 'Rina', text: 'Ukuran lahan 12x20 meter, cukup luas! Kita bisa bagi jadi 3 zona: bermain anak, bangku lansia, dan kebun komunitas.' },
		{ author: 'Budi', text: 'Kalau ada kebun kecil, saya siap rawat setiap pagi sebelum kerja. Sudah pengalaman nanam sayur di belakang rumah.' },
		{ author: 'Sari', text: 'Anak-anak SD sini butuh banget tempat main. Selama ini mereka main di jalan, bahaya.' }
	]
};

/** 4. Vote opened — BARE CARD — no photo, no body, just the vote hook */
export const mockFeedItem4: FeedItem = {
	witness_id: 'witness-feed-004',
	title: 'Musyawarah Anggaran RT 05 2025',
	track_hint: 'musyawarah',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-004',
		event_type: 'vote_opened',
		actor_name: 'Sistem',
		timestamp: ts(120),
		verb: 'membuka voting',
		snippet: 'Setuju anggaran naik 15% untuk perbaikan fasilitas? Sisa 2 hari.'
	},
	collapsed_count: 3,
	member_count: 25,
	members_preview: [memberAhmad, memberRina, memberBudi, memberSari],
	entity_tags: [entityRT05],
	urgency: 'voting',
	source: 'ikutan',
	hook_line: 'Anggaran naik 15% — setuju atau tidak?',
	sentiment: 'curious',
	intensity: 4,
	active_now: 8,
	// Auto-pantau: user vouched + voted
	monitored: true,
	// Tandang signals: user voted yes, high community engagement
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: false,
		flagged: false,
		quality_voted: true,
		vote_cast: 'yes'
	},
	signal_counts: {
		vouch_positive: 15,
		vouch_skeptical: 3,
		witness_count: 0,
		quality_avg: 4.5,
		quality_votes: 12,
		flags: 1
	},
	deadline: tsFuture(47 * 60),       // ~47 hours from now (under 2 days)
	deadline_label: 'Voting ditutup',
	quorum_target: 40,
	quorum_current: 25,
	peek_messages: [
		{ author: 'Ahmad', text: 'Kenaikan 15% itu untuk apa saja? Minta rinciannya dong, jangan cuma angka total.' },
		{ author: 'Rina', text: 'Perbaikan got dan penerangan gang. Detailnya ada di dokumen yang Pak Sekretaris upload kemarin.' },
		{ author: 'Dewi', text: 'Saya setuju selama transparan. Tahun lalu iuran naik tapi laporannya tidak jelas, jadi kali ini saya minta ada laporan bulanan yang bisa dilihat semua warga.' }
	]
};

/** 5. Evidence — PHOTO + BODY — damning river pollution evidence */
export const mockFeedItem5: FeedItem = {
	witness_id: 'witness-feed-005',
	title: 'Penyelidikan Limbah Pabrik Sungai Ciliwung',
	track_hint: 'telusuri',
	status: 'active',
	rahasia_level: 'L1',
	latest_event: {
		event_id: 'evt-005',
		event_type: 'evidence',
		actor_name: 'Budi Santoso',
		actor_role: 'saksi',
		timestamp: ts(180),
		verb: 'menambah bukti',
		snippet: 'Foto sampel air sungai menunjukkan perubahan warna signifikan di titik pembuangan.'
	},
	collapsed_count: 12,
	member_count: 7,
	members_preview: [memberBudi, memberAhmad, memberDewi],
	entity_tags: [entityLingkungan],
	source: 'terlibat',
	hook_line: 'Air sungai berubah warna di titik buangan.',
	sentiment: 'angry',
	intensity: 5,
	cover_url: 'https://images.unsplash.com/photo-1611273426858-450d8e3c9fce?w=600&h=400&fit=crop',
	body: 'Pak Budi mengambil sampel air di tiga titik berbeda sepanjang sungai. Di dekat pipa pembuangan pabrik, air berubah kecoklatan dengan bau menyengat. Warga nelayan hilir melaporkan ikan mati mengambang sejak bulan lalu. Data ini sudah dikirim ke Dinas Lingkungan Hidup.',
	active_now: 4,
	// Auto-pantau: user vouched (skeptical)
	monitored: true,
	// Tandang signals: user is skeptical, contentious evidence with flags
	my_relation: {
		vouched: true,
		vouch_type: 'skeptical',
		witnessed: false,
		flagged: false,
		quality_voted: true
	},
	signal_counts: {
		vouch_positive: 3,
		vouch_skeptical: 4,
		witness_count: 2,
		quality_avg: 3.1,
		quality_votes: 8,
		flags: 3
	},
	peek_messages: [
		{ author: 'Budi', text: 'Sampel ketiga paling parah, baunya menyengat sampai 50 meter dari sungai. Warga sekitar sudah mulai pakai masker kalau lewat situ.' },
		{ author: 'Ahmad', text: 'Nelayan hilir bilang ikan mati mengambang sejak bulan lalu. Pendapatan mereka turun drastis, ada yang sudah pindah profesi.' },
		{ author: 'Dewi', text: 'Sudah kirim ke Dinas LH, tunggu respons.' },
		{ author: 'Rina', text: 'Saya punya kontak wartawan Kompas regional. Mau saya hubungi? Kalau masuk media biasanya pemerintah lebih cepat bergerak.' }
	]
};

/** 6. Resolved — PHOTO ONLY — celebratory community moment, no body */
export const mockFeedItem6: FeedItem = {
	witness_id: 'witness-feed-006',
	title: 'Perbaikan Pipa Air PDAM Blok C',
	track_hint: 'tuntaskan',
	status: 'resolved',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-006',
		event_type: 'resolved',
		actor_name: 'Sistem',
		timestamp: ts(240),
		verb: 'kasus diselesaikan',
		snippet: 'Perbaikan pipa selesai. Air mengalir normal kembali. 18 warga berkontribusi.'
	},
	collapsed_count: 15,
	member_count: 18,
	members_preview: [memberRina, memberAhmad, memberSari, memberBudi, memberDewi],
	entity_tags: [entityRT05, entityInfrastruktur],
	urgency: 'selesai',
	source: 'ikutan',
	hook_line: 'Air mengalir lagi. 18 warga berkontribusi.',
	sentiment: 'celebratory',
	intensity: 2,
	cover_url: 'https://images.unsplash.com/photo-1585704032915-c3400ca199e7?w=600&h=300&fit=crop',
	// Auto-pantau: user vouched + saksi
	monitored: true,
	// Tandang signals: user confirmed resolution, high trust, lots of witnesses
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: true,
		flagged: false,
		quality_voted: true
	},
	signal_counts: {
		vouch_positive: 14,
		vouch_skeptical: 0,
		witness_count: 11,
		quality_avg: 4.8,
		quality_votes: 15,
		flags: 0
	}
};

/** 7. Galang milestone — BODY ONLY — fundraising story */
export const mockFeedItem7: FeedItem = {
	witness_id: 'witness-feed-007',
	title: 'Galang Dana Bedah Rumah Pak Surya',
	track_hint: 'wujudkan',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-007',
		event_type: 'galang_milestone',
		actor_name: 'Sistem',
		timestamp: ts(300),
		verb: 'target galang tercapai 75%',
		snippet: 'Rp 7.500.000 dari Rp 10.000.000 terkumpul. 32 donatur.'
	},
	collapsed_count: 20,
	member_count: 32,
	members_preview: [memberSari, memberRina, memberDewi],
	entity_tags: [entityRW03],
	urgency: 'ramai',
	source: 'sekitar',
	hook_line: 'Rp 7,5 juta terkumpul — tinggal 25% lagi.',
	sentiment: 'hopeful',
	intensity: 4,
	body: 'Pak Surya, 72 tahun, tinggal sendirian di rumah yang atapnya sudah bocor di mana-mana. Musim hujan kemarin plafon kamar tidurnya runtuh. Tetangga mulai galang dana setelah melihat kondisinya. Dalam 3 minggu, 32 orang sudah menyumbang. Sisa Rp 2,5 juta lagi untuk beli material atap baru.',
	active_now: 1,
	// Auto-pantau: user vouched (positive)
	monitored: true,
	// Tandang signals: user vouched, moderate engagement
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: false,
		flagged: false,
		quality_voted: false
	},
	signal_counts: {
		vouch_positive: 20,
		vouch_skeptical: 1,
		witness_count: 4,
		quality_avg: 4.0,
		quality_votes: 9,
		flags: 0
	},
	deadline: tsFuture(5 * 24 * 60),   // 5 days from now
	deadline_label: 'Galang dana berakhir',
	peek_messages: [
		{ author: 'Sari', text: 'Pak Surya sudah bisa tidur di ruang tamu sementara, tapi kalau hujan deras tetap bocor juga. Kita harus cepat sebelum musim hujan puncak.' },
		{ author: 'Rina', text: 'Tinggal 2,5 juta lagi, ayo semangat! Kalau masing-masing donasi 50 ribu, cuma butuh 50 orang lagi.' },
		{ author: 'Ahmad', text: 'Tukang bangunan langganan saya bersedia kasih diskon material. Hubungi Pak Joko di 0812-xxx.' }
	]
};

/** 8. Community note — BARE CARD — quick curiosity question */
export const mockFeedItem8: FeedItem = {
	witness_id: 'witness-feed-008',
	title: 'Harga Beras Naik di Pasar Menteng',
	track_hint: 'telusuri',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-008',
		event_type: 'community_note',
		actor_name: 'Sari Dewi',
		actor_avatar: 'https://placehold.co/40x40/2E7D32/white?text=SD',
		actor_role: 'relawan',
		timestamp: ts(360),
		verb: 'menambah catatan komunitas',
		snippet: 'Harga beras medium naik Rp 2.000/kg sejak minggu lalu. Perlu verifikasi di pasar lain.'
	},
	collapsed_count: 2,
	member_count: 4,
	members_preview: [memberSari, memberAhmad],
	entity_tags: [entitySembako],
	source: 'sekitar',
	hook_line: 'Beras naik Rp 2.000/kg — siapa lagi yang lihat?',
	sentiment: 'curious',
	intensity: 2,
	// Tandang signals: fresh report, no user interaction, minimal signals
	signal_counts: {
		vouch_positive: 1,
		vouch_skeptical: 0,
		witness_count: 1,
		quality_avg: 0,
		quality_votes: 0,
		flags: 0
	}
};

/** 9. Repost — PHOTO + BODY — community building work in progress */
export const mockFeedItem9: FeedItem = {
	witness_id: 'witness-feed-009',
	title: 'Renovasi Pos Ronda RT 07',
	track_hint: 'wujudkan',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-009',
		event_type: 'checkpoint',
		actor_name: 'Dewi Lestari',
		actor_avatar: 'https://placehold.co/40x40/6A1B9A/white?text=DL',
		actor_role: 'relawan',
		timestamp: ts(420),
		verb: 'menyelesaikan langkah',
		snippet: 'Material bangunan sudah tiba di lokasi. Persiapan renovasi dimulai akhir pekan.'
	},
	collapsed_count: 6,
	member_count: 9,
	members_preview: [memberDewi, memberRina, memberBudi],
	entity_tags: [entityInfrastruktur],
	source: 'ikutan',
	repost: {
		reposter_name: 'Sari Dewi',
		reposter_avatar: 'https://placehold.co/40x40/2E7D32/white?text=SD',
		reposter_role: 'relawan',
		action_verb: 'bergabung sebagai Relawan'
	},
	hook_line: 'Material sudah tiba — renovasi akhir pekan ini.',
	sentiment: 'hopeful',
	intensity: 3,
	cover_url: 'https://images.unsplash.com/photo-1504307651254-35680f356dfd?w=600&h=400&fit=crop',
	body: 'Warga RT 07 gotong royong mengumpulkan material bekas untuk pos ronda baru. Semen, pasir, dan bata sudah ditumpuk di lokasi. Akhir pekan ini mulai kerja bakti — sudah ada 9 relawan yang siap turun.',
	// Tandang signals: user quality voted, healthy community trust
	my_relation: {
		vouched: false,
		witnessed: false,
		flagged: false,
		quality_voted: true
	},
	signal_counts: {
		vouch_positive: 6,
		vouch_skeptical: 0,
		witness_count: 3,
		quality_avg: 4.3,
		quality_votes: 7,
		flags: 0
	}
};

/** 10. Created with repost — PHOTO ONLY — visual evidence of trash, no body */
export const mockFeedItem10: FeedItem = {
	witness_id: 'witness-feed-010',
	title: 'Sampah Menumpuk di Pinggir Kali Baru',
	track_hint: 'tuntaskan',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-010',
		event_type: 'created',
		actor_name: 'Budi Santoso',
		actor_role: 'pelapor',
		timestamp: ts(480),
		verb: 'melaporkan masalah baru',
		snippet: 'Tumpukan sampah di pinggir Kali Baru makin parah setelah hujan deras kemarin.'
	},
	collapsed_count: 0,
	member_count: 2,
	members_preview: [memberBudi],
	entity_tags: [entityLingkungan, entityRW03],
	source: 'ikutan',
	repost: {
		reposter_name: 'Budi Santoso',
		reposter_role: 'pelapor',
		action_verb: 'melaporkan'
	},
	hook_line: 'Makin parah setelah hujan — sampah meluap.',
	sentiment: 'angry',
	intensity: 4,
	cover_url: 'https://images.unsplash.com/photo-1530587191325-3db32d826c18?w=600&h=350&fit=crop',
	// Auto-pantau: user is saksi + flagged
	monitored: true,
	// Tandang signals: user flagged this, some skepticism in community
	my_relation: {
		vouched: false,
		witnessed: true,
		flagged: true,
		quality_voted: false
	},
	signal_counts: {
		vouch_positive: 2,
		vouch_skeptical: 2,
		witness_count: 4,
		quality_avg: 2.5,
		quality_votes: 3,
		flags: 2
	}
};

// ---------------------------------------------------------------------------
// All feed items array (sorted by latest event, most recent first)
// ---------------------------------------------------------------------------

export const mockFeedItems: FeedItem[] = [
	mockFeedItem1,
	mockFeedItem2,
	mockFeedItem3,
	mockFeedItem4,
	mockFeedItem5,
	mockFeedItem6,
	mockFeedItem7,
	mockFeedItem8,
	mockFeedItem9,
	mockFeedItem10
];

// ---------------------------------------------------------------------------
// Followable entity suggestions (for onboarding)
// ---------------------------------------------------------------------------

export const mockSuggestedEntity1: FollowableEntity = {
	entity_id: 'ent-001',
	entity_type: 'lingkungan',
	label: 'RT 05 Menteng',
	followed: false,
	description: 'Komunitas warga RT 05 Kelurahan Menteng',
	witness_count: 23,
	follower_count: 45
};

export const mockSuggestedEntity2: FollowableEntity = {
	entity_id: 'ent-006',
	entity_type: 'lingkungan',
	label: 'RW 03 Menteng',
	followed: false,
	description: 'Wilayah RW 03 Kelurahan Menteng',
	witness_count: 87,
	follower_count: 120
};

export const mockSuggestedEntity3: FollowableEntity = {
	entity_id: 'ent-002',
	entity_type: 'topik',
	label: 'Infrastruktur',
	followed: false,
	description: 'Isu infrastruktur: jalan, jembatan, saluran air',
	witness_count: 12,
	follower_count: 78
};

export const mockSuggestedEntity4: FollowableEntity = {
	entity_id: 'ent-005',
	entity_type: 'topik',
	label: 'Lingkungan Hidup',
	followed: false,
	description: 'Isu lingkungan: kebersihan, polusi, penghijauan',
	witness_count: 8,
	follower_count: 56
};

export const mockSuggestedEntities: FollowableEntity[] = [
	mockSuggestedEntity1,
	mockSuggestedEntity2,
	mockSuggestedEntity3,
	mockSuggestedEntity4
];

// ---------------------------------------------------------------------------
// System cards (inline platform cards for the polymorphic feed)
// ---------------------------------------------------------------------------

export const mockSystemCardSuggestion: SystemCardData = {
	variant: 'suggestion',
	icon: '💡',
	title: 'Ikuti topik yang relevan',
	description: 'Dapatkan update tentang isu yang Anda pedulikan.',
	dismissible: true,
	payload: {
		variant: 'suggestion',
		entities: [mockSuggestedEntity1, mockSuggestedEntity3]
	}
};

export const mockSystemCardTip: SystemCardData = {
	variant: 'tip',
	icon: '📸',
	title: 'Tahukah Anda?',
	description: 'Anda bisa melampirkan foto dan video sebagai bukti untuk memperkuat laporan.',
	dismissible: true,
	payload: {
		variant: 'tip',
		tip_id: 'tip-evidence-upload'
	}
};

export const mockSystemCardMilestone: SystemCardData = {
	variant: 'milestone',
	icon: '🎉',
	title: 'Komunitas makin aktif!',
	description: '10 saksi berhasil diselesaikan bulan ini oleh warga sekitar.',
	dismissible: true,
	payload: {
		variant: 'milestone',
		metric_label: 'Saksi selesai bulan ini',
		metric_value: '10'
	}
};

export const mockSystemCards: SystemCardData[] = [
	mockSystemCardSuggestion,
	mockSystemCardTip,
	mockSystemCardMilestone
];

/**
 * Mock feed fixtures for the Pulse event-based feed.
 * 10 feed items covering all 10 trajectory types + varied event types,
 * urgency badges, sources, reposts, and visual treatments.
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
	SignalCounts,
	SignalLabels
} from '$lib/types';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();

/** Returns an ISO timestamp N minutes in the past. */
const ts = (minutesAgo: number): string => new Date(now - minutesAgo * 60 * 1000).toISOString();

/** Returns an ISO timestamp N days in the past. */
const tsDay = (daysAgo: number): string => ts(daysAgo * 24 * 60);

/** Returns an ISO timestamp N minutes in the FUTURE. */
const tsFuture = (minutesAhead: number): string =>
	new Date(now + minutesAhead * 60 * 1000).toISOString();

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

const entityKeamanan: EntityTag = {
	entity_id: 'ent-008',
	entity_type: 'topik',
	label: 'Keamanan',
	followed: false
};

const entityHukum: EntityTag = {
	entity_id: 'ent-009',
	entity_type: 'topik',
	label: 'Bantuan Hukum',
	followed: false
};

const entityBencana: EntityTag = {
	entity_id: 'ent-010',
	entity_type: 'topik',
	label: 'Kesiagaan Bencana',
	followed: true
};

// ---------------------------------------------------------------------------
// 1. AKSI — Jalan Berlubang (created, photo+body, angry)
// ---------------------------------------------------------------------------

export const mockFeedItem1: FeedItem = {
	witness_id: 'witness-feed-001',
	title: 'Jalan Berlubang Jl. Mawar, 30 KK Terdampak',
	trajectory_type: 'aksi',
	icon: 'construction',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-001',
		event_type: 'created',
		actor_name: 'Ahmad Hidayat',
		actor_role: 'pelapor',
		timestamp: ts(25),
		verb: 'melaporkan masalah baru',
		snippet: 'Jalan rusak parah di Jl. Mawar selama 3 bulan tanpa perbaikan. Motor sering jatuh.'
	},
	collapsed_count: 0,
	member_count: 1,
	members_preview: [memberAhmad],
	entity_tags: [entityRT05, entityInfrastruktur],
	urgency: 'baru',
	source: 'ikutan',
	hook_line: '3 bulan tanpa respons — warga turun tangan sendiri.',
	sentiment: 'angry',
	intensity: 4,
	cover_url: 'https://images.unsplash.com/photo-1515162816999-a0c47dc192f7?w=600&h=400&fit=crop',
	body: 'Warga Jl. Mawar melaporkan jalan rusak parah selama 3 bulan tanpa tindakan dari RT. Sekitar 30 KK terdampak, termasuk motor yang sering jatuh. Warga siap gotong royong perbaiki sendiri jika ada dana.',
	active_now: 3,
	monitored: true,
	my_relation: {
		vouched: false,
		witnessed: true,
		flagged: false,
		supported: false
	},
	signal_counts: {
		vouch_positive: 2,
		vouch_skeptical: 1,
		witness_count: 5,
		dukung_count: 0,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Saya Saksi', desc: 'Kamu melihat atau mengalami sendiri' },
		perlu_dicek: { label: 'Perlu Dicek', desc: 'Informasi perlu diverifikasi kebenarannya' }
	},
	peek_messages: [
		{ author: 'Ahmad', text: 'Sudah lapor ke RT tapi belum ada tindakan sama sekali. Motor anak saya jatuh minggu lalu gara-gara lubang ini.' },
		{ author: 'Sari', text: 'Saya foto buktinya tadi pagi. Lubangnya makin lebar setelah hujan kemarin.' },
		{ author: 'Budi', text: 'Gang sebelah juga rusak, mungkin satu area yang perlu diperbaiki sekaligus.' }
	]
};

// ---------------------------------------------------------------------------
// 2. MUFAKAT — Taman Bermain (vote_opened, bare card, curious)
// ---------------------------------------------------------------------------

export const mockFeedItem2: FeedItem = {
	witness_id: 'witness-feed-002',
	title: 'Usulan Taman Bermain Anak di Lahan Kosong RT 03',
	trajectory_type: 'mufakat',
	icon: 'users',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-002',
		event_type: 'vote_opened',
		actor_name: 'Sistem',
		timestamp: ts(60),
		verb: 'membuka voting',
		snippet: 'Setuju lahan kosong dipakai untuk taman bermain anak? Sisa 3 hari.'
	},
	collapsed_count: 3,
	member_count: 25,
	members_preview: [memberRina, memberSari, memberAhmad, memberBudi],
	entity_tags: [entityRW03, entityKarangTaruna],
	urgency: 'voting',
	source: 'ikutan',
	hook_line: 'Lahan kosong jadi taman bermain — setuju?',
	sentiment: 'curious',
	intensity: 3,
	active_now: 8,
	monitored: true,
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: false,
		flagged: false,
		supported: true,
		vote_cast: 'yes'
	},
	signal_counts: {
		vouch_positive: 12,
		vouch_skeptical: 3,
		witness_count: 0,
		dukung_count: 8,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Hadir', desc: 'Kamu hadir atau mengikuti musyawarah ini' },
		perlu_dicek: { label: 'Perlu Revisi', desc: 'Usulan perlu direvisi sebelum disetujui' }
	},
	deadline: tsFuture(72 * 60),
	deadline_label: 'Voting ditutup',
	quorum_target: 40,
	quorum_current: 25,
	peek_messages: [
		{ author: 'Rina', text: 'Anak-anak SD butuh banget tempat main. Selama ini mereka main di jalan, bahaya.' },
		{ author: 'Ahmad', text: 'Kalau jadi taman, siapa yang rawat? Perlu rencana pemeliharaan yang jelas.' },
		{ author: 'Dewi', text: 'Saya usul ada area kebun kecil juga supaya anak-anak belajar menanam.' }
	]
};

// ---------------------------------------------------------------------------
// 3. PANTAU — Dana Desa (checkpoint, body only, curious)
// ---------------------------------------------------------------------------

export const mockFeedItem3: FeedItem = {
	witness_id: 'witness-feed-003',
	title: 'Pemantauan Dana Desa Tahap II — Rp 120 Juta',
	trajectory_type: 'pantau',
	icon: 'eye',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-003',
		event_type: 'checkpoint',
		actor_name: 'Budi Santoso',
		actor_role: 'saksi',
		timestamp: ts(120),
		verb: 'menyelesaikan langkah',
		snippet: 'Progress fisik baru 30% padahal dana sudah cair 2 bulan lalu.'
	},
	collapsed_count: 8,
	member_count: 7,
	members_preview: [memberBudi, memberAhmad, memberDewi],
	entity_tags: [entityRT05, entityInfrastruktur],
	source: 'terlibat',
	hook_line: 'Dana cair Rp 120 juta tapi progress baru 30%.',
	sentiment: 'curious',
	intensity: 3,
	body: 'Pemantauan realisasi Dana Desa tahap II sebesar Rp 120 juta untuk perbaikan jalan RT 05-07. Dana sudah cair sejak 2 bulan lalu tapi progress fisik baru 30%. Warga meminta transparansi laporan penggunaan dana.',
	active_now: 2,
	monitored: true,
	my_relation: {
		vouched: false,
		witnessed: false,
		flagged: false,
		supported: false
	},
	signal_counts: {
		vouch_positive: 5,
		vouch_skeptical: 4,
		witness_count: 3,
		dukung_count: 2,
		flags: 1
	},
	signal_labels: {
		saksi: { label: 'Saya Pantau', desc: 'Kamu ikut memantau penggunaan dana desa' },
		perlu_dicek: { label: 'Perlu Audit', desc: 'Penggunaan dana perlu diaudit lebih lanjut' }
	},
	peek_messages: [
		{ author: 'Budi', text: 'Saya cek langsung ke lokasi. Material yang terpasang tidak sesuai spesifikasi RAB.' },
		{ author: 'Ahmad', text: 'Kita minta laporan resmi dari kepala desa. Harus ada pertanggungjawaban.' },
		{ author: 'Dewi', text: 'Saya bisa bantu dokumentasi foto setiap minggu supaya ada bukti progress.' }
	]
};

// ---------------------------------------------------------------------------
// 4. DATA — Harga Cabai (community_note, bare card, curious)
// ---------------------------------------------------------------------------

export const mockFeedItem4: FeedItem = {
	witness_id: 'witness-feed-004',
	title: 'Harga Cabai Rawit di Pasar Minggu — Rp 85.000/kg',
	trajectory_type: 'data',
	icon: 'file-text',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-004',
		event_type: 'community_note',
		actor_name: 'Sari Dewi',
		actor_avatar: 'https://placehold.co/40x40/2E7D32/white?text=SD',
		actor_role: 'relawan',
		timestamp: ts(180),
		verb: 'mencatat data komunitas',
		snippet: 'Harga cabai rawit naik 40% dari minggu lalu — Rp 85.000/kg.'
	},
	collapsed_count: 1,
	member_count: 3,
	members_preview: [memberSari, memberAhmad],
	entity_tags: [entitySembako],
	source: 'sekitar',
	hook_line: 'Cabai rawit Rp 85.000/kg — naik 40% dalam seminggu.',
	sentiment: 'curious',
	intensity: 2,
	signal_counts: {
		vouch_positive: 2,
		vouch_skeptical: 0,
		witness_count: 3,
		dukung_count: 0,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Konfirmasi', desc: 'Kamu konfirmasi kenaikan harga di pasar' },
		perlu_dicek: { label: 'Perlu Koreksi', desc: 'Harga yang dilaporkan perlu dicek ulang' }
	},
	peek_messages: [
		{ author: 'Sari', text: 'Di Pasar Minggu tadi pagi Rp 85.000. Ada yang bisa cek pasar lain?' },
		{ author: 'Ahmad', text: 'Pasar Tebet juga naik, tapi cuma Rp 75.000. Mungkin beda kualitas.' }
	]
};

// ---------------------------------------------------------------------------
// 5. BANTUAN — Bantuan Hukum (created, body only, sad)
// ---------------------------------------------------------------------------

export const mockFeedItem5: FeedItem = {
	witness_id: 'witness-feed-005',
	title: 'Butuh Bantuan Hukum — Sengketa Tanah Warisan',
	trajectory_type: 'bantuan',
	icon: 'heart',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-005',
		event_type: 'created',
		actor_name: 'Rina Kartika',
		actor_avatar: 'https://placehold.co/40x40/C05621/white?text=RK',
		actor_role: 'pelapor',
		timestamp: ts(240),
		verb: 'meminta bantuan warga',
		snippet: 'Butuh pendamping hukum pro-bono untuk sengketa tanah warisan yang sudah berjalan 6 bulan.'
	},
	collapsed_count: 0,
	member_count: 1,
	members_preview: [memberRina],
	entity_tags: [entityHukum, entityRW03],
	urgency: 'baru',
	source: 'sekitar',
	hook_line: 'Hak waris terancam — butuh pendamping hukum pro-bono.',
	sentiment: 'sad',
	intensity: 3,
	body: 'Seorang warga membutuhkan bantuan hukum pro-bono untuk menyelesaikan sengketa tanah warisan keluarga. Kasus sudah berjalan 6 bulan tanpa kemajuan. Tidak mampu bayar pengacara tapi hak waris terancam hilang.',
	signal_counts: {
		vouch_positive: 1,
		vouch_skeptical: 0,
		witness_count: 0,
		dukung_count: 0,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Bisa Bantu', desc: 'Kamu memiliki keahlian hukum yang relevan' },
		perlu_dicek: { label: 'Perlu Verifikasi', desc: 'Detail kasus perlu diverifikasi lebih lanjut' }
	},
	peek_messages: [
		{ author: 'Rina', text: 'Sudah 6 bulan tanpa kemajuan. Kalau ada yang kenal pengacara atau LBH, tolong bantu hubungkan.' }
	]
};

// ---------------------------------------------------------------------------
// 6. PENCAPAIAN — Jalan Diperbaiki (resolved, photo+body, celebratory)
// ---------------------------------------------------------------------------

export const mockFeedItem6: FeedItem = {
	witness_id: 'witness-feed-006',
	title: 'Jalan Berhasil Diperbaiki! Gotong Royong 2 Hari',
	trajectory_type: 'pencapaian',
	icon: 'trophy',
	status: 'resolved',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-006',
		event_type: 'resolved',
		actor_name: 'Sistem',
		timestamp: ts(300),
		verb: 'kasus diselesaikan',
		snippet: 'Perbaikan jalan selesai dalam 2 hari gotong royong. 45 warga berkontribusi.'
	},
	collapsed_count: 15,
	member_count: 45,
	members_preview: [memberAhmad, memberSari, memberBudi, memberRina, memberDewi],
	entity_tags: [entityRT05, entityInfrastruktur],
	urgency: 'selesai',
	source: 'ikutan',
	hook_line: 'Kalau bersama, semua bisa! 45 warga turun tangan.',
	sentiment: 'celebratory',
	intensity: 5,
	cover_url: 'https://images.unsplash.com/photo-1585704032915-c3400ca199e7?w=600&h=300&fit=crop',
	body: 'Setelah 2 hari gotong royong, jalan di Jl. Mawar berhasil diperbaiki oleh 45 warga. Total biaya Rp 3.2 juta dari iuran sukarela. Bukti nyata kekuatan kolaborasi warga.',
	monitored: true,
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: true,
		flagged: false,
		supported: true
	},
	signal_counts: {
		vouch_positive: 30,
		vouch_skeptical: 0,
		witness_count: 20,
		dukung_count: 25,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Ikut Serta', desc: 'Kamu ikut dalam gotong royong perbaikan jalan' },
		perlu_dicek: { label: 'Sudah Beres', desc: 'Perbaikan sudah selesai dan berfungsi baik' }
	},
	peek_messages: [
		{ author: 'Ahmad', text: 'Alhamdulillah akhirnya selesai! Terima kasih semua yang sudah turun tangan.' },
		{ author: 'Sari', text: 'Ibu-ibu nyiapin makanan 2 hari penuh. Luar biasa semangat warga sini!' },
		{ author: 'Budi', text: 'Motor sudah bisa lewat tanpa was-was. Terima kasih pak-pak yang kerja bakti!' }
	]
};

// ---------------------------------------------------------------------------
// 7. SIAGA — Banjir Ciliwung (created, photo only, urgent)
// ---------------------------------------------------------------------------

export const mockFeedItem7: FeedItem = {
	witness_id: 'witness-feed-007',
	title: 'Peringatan Banjir — Sungai Ciliwung Siaga Merah',
	trajectory_type: 'siaga',
	icon: 'siren',
	status: 'open',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-007',
		event_type: 'created',
		actor_name: 'Budi Santoso',
		actor_role: 'pelapor',
		timestamp: ts(15),
		verb: 'melaporkan keadaan darurat',
		snippet: 'Ketinggian air Sungai Ciliwung naik 2 meter dalam 6 jam terakhir!'
	},
	collapsed_count: 0,
	member_count: 3,
	members_preview: [memberBudi, memberAhmad, memberDewi],
	entity_tags: [entityBencana, entityLingkungan],
	urgency: 'baru',
	source: 'sekitar',
	hook_line: 'Siaga merah! Segera evakuasi barang ke lantai atas.',
	sentiment: 'urgent',
	intensity: 5,
	cover_url: 'https://images.unsplash.com/photo-1547683905-f686c993aae5?w=600&h=400&fit=crop',
	active_now: 12,
	monitored: true,
	my_relation: {
		vouched: false,
		witnessed: true,
		flagged: false,
		supported: false
	},
	signal_counts: {
		vouch_positive: 5,
		vouch_skeptical: 0,
		witness_count: 8,
		dukung_count: 3,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Konfirmasi', desc: 'Kamu melihat langsung kondisi sungai/banjir' },
		perlu_dicek: { label: 'Sudah Aman', desc: 'Kondisi sudah membaik di lokasi kamu' }
	},
	peek_messages: [
		{ author: 'Budi', text: 'Air naik cepat banget! Warga bantaran sudah mulai evakuasi. Tolong yang punya perahu karet bantu!' },
		{ author: 'Ahmad', text: 'Posko darurat sudah dibuka di Balai RT 05. Bawa selimut dan makanan siap saji.' },
		{ author: 'Dewi', text: 'BMKG prediksi hujan deras lagi malam ini. Semua warga bantaran segera naik!' }
	]
};

// ---------------------------------------------------------------------------
// 8. PROGRAM — Jadwal Ronda (checkpoint, bare card, hopeful)
// ---------------------------------------------------------------------------

export const mockFeedItem8: FeedItem = {
	witness_id: 'witness-feed-008',
	title: 'Jadwal Ronda RT 05 — Periode Februari 2026',
	trajectory_type: 'program',
	icon: 'calendar',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-008',
		event_type: 'checkpoint',
		actor_name: 'Rina Kartika',
		actor_avatar: 'https://placehold.co/40x40/C05621/white?text=RK',
		actor_role: 'koordinator',
		timestamp: ts(360),
		verb: 'memperbarui jadwal',
		snippet: 'Rotasi minggu ke-4 sudah ditentukan. 4 shift, masing-masing 3 orang.'
	},
	collapsed_count: 4,
	member_count: 12,
	members_preview: [memberRina, memberBudi, memberAhmad],
	entity_tags: [entityRT05, entityKeamanan],
	source: 'terlibat',
	hook_line: 'Rotasi jaga malam minggu ke-4 sudah ditetapkan.',
	sentiment: 'hopeful',
	intensity: 1,
	my_relation: {
		vouched: false,
		witnessed: false,
		flagged: false,
		supported: true
	},
	signal_counts: {
		vouch_positive: 6,
		vouch_skeptical: 0,
		witness_count: 0,
		dukung_count: 4,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Hadir', desc: 'Kamu hadir pada shift ronda yang ditentukan' },
		perlu_dicek: { label: 'Jadwal Berubah', desc: 'Ada perubahan jadwal yang perlu dikoordinasikan' }
	},
	peek_messages: [
		{ author: 'Rina', text: 'Minggu ini shift A: Pak Budi, Pak Ahmad, Mas Doni. Mulai jam 22:00.' },
		{ author: 'Budi', text: 'Siap. Saya bawa senter dan HT seperti biasa.' }
	]
};

// ---------------------------------------------------------------------------
// 9. ADVOKASI — Hak Air Bersih (evidence, photo+body, hopeful)
// ---------------------------------------------------------------------------

export const mockFeedItem9: FeedItem = {
	witness_id: 'witness-feed-009',
	title: 'Advokasi Hak Air Bersih Kampung Melayu',
	trajectory_type: 'advokasi',
	icon: 'megaphone',
	status: 'active',
	rahasia_level: 'L0',
	latest_event: {
		event_id: 'evt-009',
		event_type: 'evidence',
		actor_name: 'Dewi Lestari',
		actor_avatar: 'https://placehold.co/40x40/6A1B9A/white?text=DL',
		actor_role: 'relawan',
		timestamp: ts(420),
		verb: 'menambah bukti',
		snippet: 'Hasil lab menunjukkan air sumur tercemar bakteri E.coli di atas ambang batas aman.'
	},
	collapsed_count: 10,
	member_count: 15,
	members_preview: [memberDewi, memberRina, memberSari],
	entity_tags: [entityLingkungan, entityRW03],
	source: 'terlibat',
	repost: {
		reposter_name: 'Sari Dewi',
		reposter_avatar: 'https://placehold.co/40x40/2E7D32/white?text=SD',
		reposter_role: 'relawan',
		action_verb: 'membagikan bukti baru'
	},
	hook_line: 'Bukti lab: air sumur tidak layak minum.',
	sentiment: 'hopeful',
	intensity: 4,
	cover_url: 'https://images.unsplash.com/photo-1611273426858-450d8e3c9fce?w=600&h=400&fit=crop',
	body: 'Komunitas Kampung Melayu mendesak pemerintah daerah untuk menyediakan akses air bersih. Hasil uji lab dari 5 sumur warga menunjukkan kadar bakteri E.coli di atas ambang batas aman. Petisi sudah ditandatangani 200+ warga.',
	active_now: 4,
	monitored: true,
	my_relation: {
		vouched: true,
		vouch_type: 'positive',
		witnessed: false,
		flagged: false,
		supported: true
	},
	signal_counts: {
		vouch_positive: 12,
		vouch_skeptical: 1,
		witness_count: 5,
		dukung_count: 15,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Saya Terdampak', desc: 'Kamu mengalami masalah air bersih ini' },
		perlu_dicek: { label: 'Perlu Verifikasi', desc: 'Hasil lab perlu diverifikasi pihak independen' }
	},
	peek_messages: [
		{ author: 'Dewi', text: 'Lab resmi UGM sudah konfirmasi. Ini bukti kuat untuk petisi kita ke DPRD.' },
		{ author: 'Rina', text: 'Petisi sudah 200+ tanda tangan. Target kita 500 sebelum kirim ke dewan.' },
		{ author: 'Sari', text: 'Saya hubungi wartawan Kompas, mereka tertarik liputan minggu depan.' }
	]
};

// ---------------------------------------------------------------------------
// 10. MEDIASI — Sengketa Lahan (joined, body only, curious)
// ---------------------------------------------------------------------------

export const mockFeedItem10: FeedItem = {
	witness_id: 'witness-feed-010',
	title: 'Mediasi Sengketa Batas Lahan Antar Warga RT 07',
	trajectory_type: 'mediasi',
	icon: 'scale',
	status: 'active',
	rahasia_level: 'L1',
	latest_event: {
		event_id: 'evt-010',
		event_type: 'joined',
		actor_name: 'Rina Kartika',
		actor_avatar: 'https://placehold.co/40x40/C05621/white?text=RK',
		actor_role: 'koordinator',
		timestamp: ts(480),
		verb: 'bergabung sebagai Mediator'
	},
	collapsed_count: 4,
	member_count: 5,
	members_preview: [memberRina, memberAhmad, memberBudi],
	entity_tags: [entityRW03],
	source: 'terlibat',
	hook_line: 'Mediator bergabung — dua warga cari jalan tengah.',
	sentiment: 'curious',
	intensity: 2,
	body: 'Dua warga RT 07 berselisih soal batas tanah warisan. Bu Rina sebagai mediator akan memfasilitasi pertemuan pertama akhir pekan ini. Tujuannya mencapai kesepakatan tanpa harus ke jalur hukum formal.',
	my_relation: {
		vouched: false,
		witnessed: false,
		flagged: false,
		supported: false
	},
	signal_counts: {
		vouch_positive: 2,
		vouch_skeptical: 1,
		witness_count: 2,
		dukung_count: 1,
		flags: 0
	},
	signal_labels: {
		saksi: { label: 'Saya Tahu', desc: 'Kamu mengetahui detail sengketa batas lahan ini' },
		perlu_dicek: { label: 'Perlu Klarifikasi', desc: 'Batas lahan perlu diukur ulang secara resmi' }
	},
	peek_messages: [
		{ author: 'Rina', text: 'Saya sudah bicara dengan kedua pihak. Ada titik temu yang bisa diusahakan.' },
		{ author: 'Ahmad', text: 'Kalau butuh surat ukur BPN, saya bisa bantu uruskan. Prosesnya sekitar 2 minggu.' }
	]
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

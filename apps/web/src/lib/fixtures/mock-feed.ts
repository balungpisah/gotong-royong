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
	SystemCardData
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
	body: 'Warga Gang Melati mengeluh lampu jalan padam total sejak dua minggu lalu. Ibu-ibu takut pulang malam dari pasar. Pak Ahmad sudah lapor ke kelurahan tapi belum ada respon. "Anak saya harus lewat gang gelap setiap pulang les," katanya.'
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
	intensity: 3
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
	body: 'Setelah 3 bulan negosiasi dengan pemilik lahan, akhirnya disepakati pinjam pakai selama 5 tahun. Tim survei Karang Taruna turun langsung mengukur dan memetakan. Rencananya ada area bermain anak, bangku lansia, dan kebun kecil yang dikelola bersama. Bu Rina bilang, "Ini mimpi warga sejak 2019."'
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
	intensity: 4
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
	body: 'Pak Budi mengambil sampel air di tiga titik berbeda sepanjang sungai. Di dekat pipa pembuangan pabrik, air berubah kecoklatan dengan bau menyengat. Warga nelayan hilir melaporkan ikan mati mengambang sejak bulan lalu. Data ini sudah dikirim ke Dinas Lingkungan Hidup.'
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
	cover_url: 'https://images.unsplash.com/photo-1585704032915-c3400ca199e7?w=600&h=300&fit=crop'
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
	body: 'Pak Surya, 72 tahun, tinggal sendirian di rumah yang atapnya sudah bocor di mana-mana. Musim hujan kemarin plafon kamar tidurnya runtuh. Tetangga mulai galang dana setelah melihat kondisinya. Dalam 3 minggu, 32 orang sudah menyumbang. Sisa Rp 2,5 juta lagi untuk beli material atap baru.'
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
	intensity: 2
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
	body: 'Warga RT 07 gotong royong mengumpulkan material bekas untuk pos ronda baru. Semen, pasir, dan bata sudah ditumpuk di lokasi. Akhir pekan ini mulai kerja bakti — sudah ada 9 relawan yang siap turun.'
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
	cover_url: 'https://images.unsplash.com/photo-1530587191325-3db32d826c18?w=600&h=350&fit=crop'
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

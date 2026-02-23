/**
 * Mock TandangProfile fixtures for the dev gallery.
 * 5 profiles matching mockUser1-5, covering all tier levels 0-4.
 */

import type { TandangProfile, PersonRelation } from '$lib/types';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();

/** Returns an ISO timestamp N days in the past. */
const tsDay = (daysAgo: number): string =>
	new Date(now - daysAgo * 24 * 60 * 60 * 1000).toISOString();

/** Returns an ISO timestamp N hours in the past. */
const tsHour = (hoursAgo: number): string =>
	new Date(now - hoursAgo * 60 * 60 * 1000).toISOString();

// ---------------------------------------------------------------------------
// mockTandangProfile1 — Ahmad Hidayat, tier 2 Kontributor
// ---------------------------------------------------------------------------

export const mockTandangProfile1: TandangProfile = {
	user_id: 'u-001',
	name: 'Ahmad Hidayat',
	tier: {
		level: 2,
		name: 'Kontributor',
		pips: '◆◆◇◇',
		color: '#00695C',
		percentile: 55
	},
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	joined_at: tsDay(180),
	location: 'RT 05',
	last_active_at: tsHour(2),
	scores: {
		integrity: { value: 0.65 },
		competence: {
			aggregate: 0.55,
			domains: [
				{
					skill_id: 'ESCO-ID-GR-001',
					skill_name: 'Koordinasi Komunitas',
					score: 0.62,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(3),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-004',
					skill_name: 'Verifikasi Lapangan',
					score: 0.48,
					decaying: true,
					days_until_decay: 12,
					last_activity: tsDay(18),
					validated: true
				}
			]
		},
		judgment: {
			value: 0.48,
			vouch_outcomes_count: 4,
			dukung_success_rate: 0.75
		}
	},
	consistency: {
		multiplier: 1.06,
		streak_days: 14,
		streak_weeks: 2,
		contributions_30d: 8,
		quality_avg: 0.72,
		gap_days: 1
	},
	genesis: {
		weight: 72.9,
		meaningful_interactions_this_month: 5,
		threshold: 3
	},
	skills: [
		{
			skill_id: 'ESCO-ID-GR-001',
			skill_name: 'Koordinasi Komunitas',
			validated: true,
			score: 0.62,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-004',
			skill_name: 'Verifikasi Lapangan',
			validated: true,
			score: 0.48,
			decaying: true,
			days_until_decay: 12
		},
		{
			skill_id: 'declared-001',
			skill_name: 'Pengelolaan Data',
			validated: false
		}
	],
	vouched_by: [
		{
			vouch_id: 'vc-101',
			user_id: 'u-002',
			user_name: 'Sari Dewi',
			user_tier: 3,
			vouch_type: 'positive',
			created_at: tsDay(20),
			context_label: 'Koordinasi banjir RT 05'
		},
		{
			vouch_id: 'vc-102',
			user_id: 'u-004',
			user_name: 'Rina Kartika',
			user_tier: 4,
			vouch_type: 'collective',
			created_at: tsDay(45),
			context_label: 'Penggalangan dana jalan rusak'
		},
		{
			vouch_id: 'vc-103',
			user_id: 'u-003',
			user_name: 'Budi Santoso',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(60)
		},
		{
			vouch_id: 'vc-104',
			user_id: 'u-005',
			user_name: 'Hendra Wijaya',
			user_tier: 0,
			vouch_type: 'collective',
			created_at: tsDay(10)
		}
	],
	vouching_for: [
		{
			vouch_id: 'vc-201',
			user_id: 'u-003',
			user_name: 'Budi Santoso',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(15),
			context_label: 'Pelaporan jalan berlubang RT 08'
		},
		{
			vouch_id: 'vc-202',
			user_id: 'u-005',
			user_name: 'Hendra Wijaya',
			user_tier: 0,
			vouch_type: 'collective',
			created_at: tsDay(8)
		},
		{
			vouch_id: 'vc-203',
			user_id: 'u-006',
			user_name: 'Dewi Lestari',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(5)
		}
	],
	vouch_budget: {
		max_vouches: 20,
		active_vouches: 3,
		remaining: 17
	},
	dukung_given: [
		{
			dukung_id: 'dk-001',
			witness_id: 'w-010',
			witness_title: 'Perbaikan Drainase RW 03',
			supporter_id: 'u-001',
			supporter_name: 'Ahmad Hidayat',
			created_at: tsDay(12),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-002',
			witness_id: 'w-015',
			witness_title: 'Pengadaan Lampu Jalan RT 05',
			supporter_id: 'u-001',
			supporter_name: 'Ahmad Hidayat',
			created_at: tsDay(5),
			outcome: 'pending'
		}
	],
	dukung_received: [
		{
			dukung_id: 'dk-101',
			witness_id: 'w-008',
			witness_title: 'Koordinasi Banjir RT 05',
			supporter_id: 'u-002',
			supporter_name: 'Sari Dewi',
			created_at: tsDay(30),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-102',
			witness_id: 'w-009',
			witness_title: 'Verifikasi Jalan Rusak RT 05',
			supporter_id: 'u-004',
			supporter_name: 'Rina Kartika',
			created_at: tsDay(22),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-103',
			witness_id: 'w-012',
			witness_title: 'Pendataan Warga Terdampak',
			supporter_id: 'u-003',
			supporter_name: 'Budi Santoso',
			created_at: tsDay(10),
			outcome: 'pending'
		}
	],
	timeline: [
		{
			activity_id: 'tl-001',
			type: 'vouch_given',
			text: 'Memberi vouch pada Hendra Wijaya sebagai warga baru',
			timestamp: tsDay(1)
		},
		{
			activity_id: 'tl-002',
			type: 'evidence_submitted',
			text: 'Mengirim foto bukti genangan RT 05',
			timestamp: tsDay(3),
			witness_id: 'w-008'
		},
		{
			activity_id: 'tl-003',
			type: 'vote_cast',
			text: 'Memilih prioritas perbaikan drainase RW 03',
			timestamp: tsDay(5)
		},
		{
			activity_id: 'tl-004',
			type: 'dukung_given',
			text: 'Mendukung pengadaan lampu jalan RT 05',
			timestamp: tsDay(5),
			witness_id: 'w-015'
		},
		{
			activity_id: 'tl-005',
			type: 'vouch_received',
			text: 'Menerima vouch dari Hendra Wijaya',
			timestamp: tsDay(8)
		},
		{
			activity_id: 'tl-006',
			type: 'witness_joined',
			text: 'Bergabung sebagai saksi banjir RT 05',
			timestamp: tsDay(10),
			witness_id: 'w-008'
		},
		{
			activity_id: 'tl-007',
			type: 'dukung_given',
			text: 'Mendukung perbaikan drainase RW 03',
			timestamp: tsDay(12),
			witness_id: 'w-010'
		},
		{
			activity_id: 'tl-008',
			type: 'vote_cast',
			text: 'Berpartisipasi dalam voting penggalangan dana',
			timestamp: tsDay(14)
		},
		{
			activity_id: 'tl-009',
			type: 'resolution_completed',
			text: 'Laporan jalan rusak RT 05 berhasil diselesaikan',
			timestamp: tsDay(18),
			witness_id: 'w-009'
		},
		{
			activity_id: 'tl-010',
			type: 'vouch_received',
			text: 'Menerima vouch kolektif dari Rina Kartika',
			timestamp: tsDay(20)
		},
		{
			activity_id: 'tl-011',
			type: 'skill_validated',
			text: 'Kompetensi Koordinasi Komunitas divalidasi',
			timestamp: tsDay(25)
		},
		{
			activity_id: 'tl-012',
			type: 'tier_change',
			text: 'Naik ke tier Kontributor (◆◆◇◇)',
			timestamp: tsDay(30)
		}
	],
	impact: {
		witnesses_resolved: 5,
		people_helped: 28,
		total_dukung_given: 8,
		total_dukung_received: 12,
		evidence_validated: 10,
		votes_participated: 15
	},
	decay_warnings: [
		{ domain: 'Verifikasi Lapangan', days_until_decay: 12 }
	]
};

// ---------------------------------------------------------------------------
// mockTandangProfile2 — Sari Dewi, tier 3 Pilar
// ---------------------------------------------------------------------------

export const mockTandangProfile2: TandangProfile = {
	user_id: 'u-002',
	name: 'Sari Dewi',
	tier: {
		level: 3,
		name: 'Pilar',
		pips: '◆◆◆◇',
		color: '#1E88E5',
		percentile: 82
	},
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	joined_at: tsDay(365),
	location: 'RW 03',
	last_active_at: tsHour(1),
	scores: {
		integrity: { value: 0.78 },
		competence: {
			aggregate: 0.72,
			domains: [
				{
					skill_id: 'ESCO-ID-GR-001',
					skill_name: 'Koordinasi Komunitas',
					score: 0.75,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(1),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-002',
					skill_name: 'Investigasi Warga',
					score: 0.70,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(3),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-003',
					skill_name: 'Mediasi Konflik',
					score: 0.68,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(5),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-005',
					skill_name: 'Pendampingan Warga',
					score: 0.65,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(7),
					validated: true
				}
			]
		},
		judgment: {
			value: 0.68,
			vouch_outcomes_count: 8,
			dukung_success_rate: 0.88
		}
	},
	consistency: {
		multiplier: 1.14,
		streak_days: 42,
		streak_weeks: 6,
		contributions_30d: 15,
		quality_avg: 0.85,
		gap_days: 0
	},
	genesis: {
		weight: null,
		meaningful_interactions_this_month: 12,
		threshold: 3
	},
	skills: [
		{
			skill_id: 'ESCO-ID-GR-001',
			skill_name: 'Koordinasi Komunitas',
			validated: true,
			score: 0.75,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-002',
			skill_name: 'Investigasi Warga',
			validated: true,
			score: 0.70,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-003',
			skill_name: 'Mediasi Konflik',
			validated: true,
			score: 0.68,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-005',
			skill_name: 'Pendampingan Warga',
			validated: true,
			score: 0.65,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'declared-002',
			skill_name: 'Fasilitasi Musyawarah',
			validated: false
		}
	],
	vouched_by: [
		{
			vouch_id: 'vc-301',
			user_id: 'u-004',
			user_name: 'Rina Kartika',
			user_tier: 4,
			vouch_type: 'positive',
			created_at: tsDay(10),
			context_label: 'Moderasi diskusi taman RW 03'
		},
		{
			vouch_id: 'vc-302',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'collective',
			created_at: tsDay(20)
		},
		{
			vouch_id: 'vc-303',
			user_id: 'u-007',
			user_name: 'Rudi Prasetyo',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(30),
			context_label: 'Penyelesaian konflik lahan'
		},
		{
			vouch_id: 'vc-304',
			user_id: 'u-008',
			user_name: 'Fitri Handayani',
			user_tier: 3,
			vouch_type: 'mentorship',
			created_at: tsDay(45)
		},
		{
			vouch_id: 'vc-305',
			user_id: 'u-009',
			user_name: 'Agus Salim',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(60)
		},
		{
			vouch_id: 'vc-306',
			user_id: 'u-010',
			user_name: 'Yuni Astuti',
			user_tier: 1,
			vouch_type: 'collective',
			created_at: tsDay(90)
		},
		{
			vouch_id: 'vc-307',
			user_id: 'u-011',
			user_name: 'Bambang Hermawan',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(120)
		},
		{
			vouch_id: 'vc-308',
			user_id: 'u-012',
			user_name: 'Endang Susilo',
			user_tier: 3,
			vouch_type: 'project_scoped',
			created_at: tsDay(150),
			context_label: 'Proyek RPJM Kelurahan 2024'
		}
	],
	vouching_for: [
		{
			vouch_id: 'vc-401',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(20)
		},
		{
			vouch_id: 'vc-402',
			user_id: 'u-003',
			user_name: 'Budi Santoso',
			user_tier: 1,
			vouch_type: 'collective',
			created_at: tsDay(35)
		},
		{
			vouch_id: 'vc-403',
			user_id: 'u-005',
			user_name: 'Hendra Wijaya',
			user_tier: 0,
			vouch_type: 'mentorship',
			created_at: tsDay(7)
		},
		{
			vouch_id: 'vc-404',
			user_id: 'u-013',
			user_name: 'Eko Nugroho',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(50)
		},
		{
			vouch_id: 'vc-405',
			user_id: 'u-014',
			user_name: 'Sri Wahyuni',
			user_tier: 0,
			vouch_type: 'collective',
			created_at: tsDay(14)
		},
		{
			vouch_id: 'vc-406',
			user_id: 'u-015',
			user_name: 'Toni Kusuma',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(28)
		}
	],
	vouch_budget: {
		max_vouches: 35,
		active_vouches: 6,
		remaining: 29
	},
	dukung_given: [
		{
			dukung_id: 'dk-201',
			witness_id: 'w-020',
			witness_title: 'Perbaikan Taman Bermain RW 03',
			supporter_id: 'u-002',
			supporter_name: 'Sari Dewi',
			created_at: tsDay(8),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-202',
			witness_id: 'w-021',
			witness_title: 'Pengadaan Tempat Sampah RT 07',
			supporter_id: 'u-002',
			supporter_name: 'Sari Dewi',
			created_at: tsDay(15),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-203',
			witness_id: 'w-022',
			witness_title: 'Mediasi Sengketa Warga RT 02',
			supporter_id: 'u-002',
			supporter_name: 'Sari Dewi',
			created_at: tsDay(3),
			outcome: 'pending'
		}
	],
	dukung_received: [
		{
			dukung_id: 'dk-301',
			witness_id: 'w-018',
			witness_title: 'Investigasi Sampah Ilegal RW 03',
			supporter_id: 'u-004',
			supporter_name: 'Rina Kartika',
			created_at: tsDay(12),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-302',
			witness_id: 'w-019',
			witness_title: 'Pendampingan Warga Lansia RT 04',
			supporter_id: 'u-001',
			supporter_name: 'Ahmad Hidayat',
			created_at: tsDay(25),
			outcome: 'success'
		}
	],
	timeline: [
		{
			activity_id: 'tl-101',
			type: 'vouch_given',
			text: 'Memberi vouch mentor pada Hendra Wijaya',
			timestamp: tsDay(1)
		},
		{
			activity_id: 'tl-102',
			type: 'witness_created',
			text: 'Membuka laporan investigasi sampah ilegal RW 03',
			timestamp: tsDay(2),
			witness_id: 'w-018'
		},
		{
			activity_id: 'tl-103',
			type: 'vote_cast',
			text: 'Memimpin voting pembangunan taman RW 03',
			timestamp: tsDay(3)
		},
		{
			activity_id: 'tl-104',
			type: 'evidence_submitted',
			text: 'Mengunggah dokumentasi mediasi konflik lahan',
			timestamp: tsDay(5),
			witness_id: 'w-022'
		},
		{
			activity_id: 'tl-105',
			type: 'resolution_completed',
			text: 'Laporan sampah ilegal RW 03 berhasil diselesaikan',
			timestamp: tsDay(8),
			witness_id: 'w-018'
		},
		{
			activity_id: 'tl-106',
			type: 'vouch_received',
			text: 'Menerima vouch positif dari Rina Kartika',
			timestamp: tsDay(10)
		},
		{
			activity_id: 'tl-107',
			type: 'dukung_given',
			text: 'Mendukung perbaikan taman bermain RW 03',
			timestamp: tsDay(12),
			witness_id: 'w-020'
		},
		{
			activity_id: 'tl-108',
			type: 'witness_joined',
			text: 'Bergabung sebagai koordinator saksi banjir RW 03',
			timestamp: tsDay(15),
			witness_id: 'w-025'
		},
		{
			activity_id: 'tl-109',
			type: 'skill_validated',
			text: 'Kompetensi Investigasi Warga divalidasi',
			timestamp: tsDay(18)
		},
		{
			activity_id: 'tl-110',
			type: 'vote_cast',
			text: 'Berpartisipasi dalam voting prioritas RPJM',
			timestamp: tsDay(20)
		},
		{
			activity_id: 'tl-111',
			type: 'vouch_given',
			text: 'Memberi vouch pada Ahmad Hidayat atas koordinasi banjir',
			timestamp: tsDay(22)
		},
		{
			activity_id: 'tl-112',
			type: 'resolution_completed',
			text: 'Pendampingan warga lansia RT 04 selesai',
			timestamp: tsDay(25),
			witness_id: 'w-019'
		},
		{
			activity_id: 'tl-113',
			type: 'evidence_submitted',
			text: 'Mengirim laporan kondisi drainase RW 03',
			timestamp: tsDay(28)
		},
		{
			activity_id: 'tl-114',
			type: 'tier_change',
			text: 'Naik ke tier Pilar (◆◆◆◇)',
			timestamp: tsDay(90)
		},
		{
			activity_id: 'tl-115',
			type: 'skill_validated',
			text: 'Kompetensi Mediasi Konflik divalidasi',
			timestamp: tsDay(120)
		}
	],
	impact: {
		witnesses_resolved: 9,
		people_helped: 52,
		total_dukung_given: 14,
		total_dukung_received: 18,
		evidence_validated: 23,
		votes_participated: 31
	},
	decay_warnings: []
};

// ---------------------------------------------------------------------------
// mockTandangProfile3 — Budi Santoso, tier 1 Pemula
// ---------------------------------------------------------------------------

export const mockTandangProfile3: TandangProfile = {
	user_id: 'u-003',
	name: 'Budi Santoso',
	tier: {
		level: 1,
		name: 'Pemula',
		pips: '◆◇◇◇',
		color: '#795548',
		percentile: 28
	},
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	joined_at: tsDay(90),
	location: 'RT 08',
	last_active_at: tsHour(5),
	scores: {
		integrity: { value: 0.42 },
		competence: {
			aggregate: 0.30,
			domains: [
				{
					skill_id: 'ESCO-ID-GR-004',
					skill_name: 'Verifikasi Lapangan',
					score: 0.30,
					decaying: true,
					days_until_decay: 5,
					last_activity: tsDay(25),
					validated: true
				}
			]
		},
		judgment: {
			value: 0.25,
			vouch_outcomes_count: 1,
			dukung_success_rate: null
		}
	},
	consistency: {
		multiplier: 1.02,
		streak_days: 5,
		streak_weeks: 0,
		contributions_30d: 3,
		quality_avg: 0.55,
		gap_days: 3
	},
	genesis: {
		weight: 90.0,
		meaningful_interactions_this_month: 2,
		threshold: 3
	},
	skills: [
		{
			skill_id: 'ESCO-ID-GR-004',
			skill_name: 'Verifikasi Lapangan',
			validated: true,
			score: 0.30,
			decaying: true,
			days_until_decay: 5
		},
		{
			skill_id: 'declared-003',
			skill_name: 'Fotografi Dokumentasi',
			validated: false
		}
	],
	vouched_by: [
		{
			vouch_id: 'vc-501',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(15)
		},
		{
			vouch_id: 'vc-502',
			user_id: 'u-002',
			user_name: 'Sari Dewi',
			user_tier: 3,
			vouch_type: 'collective',
			created_at: tsDay(30)
		},
		{
			vouch_id: 'vc-503',
			user_id: 'u-016',
			user_name: 'Pak RT 08',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(60),
			context_label: 'Warga aktif RT 08'
		}
	],
	vouching_for: [
		{
			vouch_id: 'vc-601',
			user_id: 'u-005',
			user_name: 'Hendra Wijaya',
			user_tier: 0,
			vouch_type: 'positive',
			created_at: tsDay(20)
		}
	],
	vouch_budget: {
		max_vouches: 10,
		active_vouches: 1,
		remaining: 9
	},
	dukung_given: [
		{
			dukung_id: 'dk-401',
			witness_id: 'w-030',
			witness_title: 'Perbaikan Jalan Berlubang RT 08',
			supporter_id: 'u-003',
			supporter_name: 'Budi Santoso',
			created_at: tsDay(10),
			outcome: 'pending'
		}
	],
	dukung_received: [
		{
			dukung_id: 'dk-501',
			witness_id: 'w-028',
			witness_title: 'Verifikasi Jalan Rusak RT 08',
			supporter_id: 'u-001',
			supporter_name: 'Ahmad Hidayat',
			created_at: tsDay(20),
			outcome: 'success'
		}
	],
	timeline: [
		{
			activity_id: 'tl-201',
			type: 'evidence_submitted',
			text: 'Mengirim foto bukti jalan berlubang RT 08',
			timestamp: tsDay(3),
			witness_id: 'w-028'
		},
		{
			activity_id: 'tl-202',
			type: 'vote_cast',
			text: 'Berpartisipasi dalam voting perbaikan jalan',
			timestamp: tsDay(7)
		},
		{
			activity_id: 'tl-203',
			type: 'vouch_given',
			text: 'Memberi vouch pada Hendra Wijaya',
			timestamp: tsDay(12)
		},
		{
			activity_id: 'tl-204',
			type: 'witness_joined',
			text: 'Bergabung sebagai saksi banjir RT 08',
			timestamp: tsDay(20),
			witness_id: 'w-030'
		},
		{
			activity_id: 'tl-205',
			type: 'vouch_received',
			text: 'Menerima vouch dari Ahmad Hidayat',
			timestamp: tsDay(25)
		},
		{
			activity_id: 'tl-206',
			type: 'skill_validated',
			text: 'Kompetensi Verifikasi Lapangan divalidasi',
			timestamp: tsDay(50)
		},
		{
			activity_id: 'tl-207',
			type: 'vouch_received',
			text: 'Menerima vouch kolektif dari Sari Dewi',
			timestamp: tsDay(60)
		},
		{
			activity_id: 'tl-208',
			type: 'tier_change',
			text: 'Naik ke tier Pemula (◆◇◇◇)',
			timestamp: tsDay(75)
		}
	],
	impact: {
		witnesses_resolved: 2,
		people_helped: 8,
		total_dukung_given: 2,
		total_dukung_received: 3,
		evidence_validated: 5,
		votes_participated: 8
	},
	decay_warnings: [
		{ domain: 'Verifikasi Lapangan', days_until_decay: 5 }
	]
};

// ---------------------------------------------------------------------------
// mockTandangProfile4 — Rina Kartika, tier 4 Kunci
// ---------------------------------------------------------------------------

export const mockTandangProfile4: TandangProfile = {
	user_id: 'u-004',
	name: 'Rina Kartika',
	avatar_url: 'https://placehold.co/40x40/C05621/white?text=RK',
	tier: {
		level: 4,
		name: 'Kunci',
		pips: '◆◆◆◆',
		color: '#FFD700',
		percentile: 97
	},
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	joined_at: tsDay(730),
	location: 'RW 01',
	last_active_at: tsHour(3),
	scores: {
		integrity: { value: 0.92 },
		competence: {
			aggregate: 0.88,
			domains: [
				{
					skill_id: 'ESCO-ID-GR-001',
					skill_name: 'Koordinasi Komunitas',
					score: 0.95,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(1),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-002',
					skill_name: 'Investigasi Warga',
					score: 0.90,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(2),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-003',
					skill_name: 'Mediasi Konflik',
					score: 0.88,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(3),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-004',
					skill_name: 'Verifikasi Lapangan',
					score: 0.85,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(4),
					validated: true
				},
				{
					skill_id: 'ESCO-ID-GR-005',
					skill_name: 'Pendampingan Warga',
					score: 0.82,
					decaying: false,
					days_until_decay: null,
					last_activity: tsDay(5),
					validated: true
				}
			]
		},
		judgment: {
			value: 0.85,
			vouch_outcomes_count: 12,
			dukung_success_rate: 0.95
		}
	},
	consistency: {
		multiplier: 1.20,
		streak_days: 180,
		streak_weeks: 25,
		contributions_30d: 22,
		quality_avg: 0.94,
		gap_days: 0
	},
	genesis: {
		weight: null,
		meaningful_interactions_this_month: 18,
		threshold: 3
	},
	skills: [
		{
			skill_id: 'ESCO-ID-GR-001',
			skill_name: 'Koordinasi Komunitas',
			validated: true,
			score: 0.95,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-002',
			skill_name: 'Investigasi Warga',
			validated: true,
			score: 0.90,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-003',
			skill_name: 'Mediasi Konflik',
			validated: true,
			score: 0.88,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-004',
			skill_name: 'Verifikasi Lapangan',
			validated: true,
			score: 0.85,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'ESCO-ID-GR-005',
			skill_name: 'Pendampingan Warga',
			validated: true,
			score: 0.82,
			decaying: false,
			days_until_decay: null
		},
		{
			skill_id: 'declared-004',
			skill_name: 'Manajemen Konflik Skala Besar',
			validated: false
		}
	],
	vouched_by: [
		{
			vouch_id: 'vc-701',
			user_id: 'u-017',
			user_name: 'Pak Lurah',
			user_tier: 4,
			vouch_type: 'positive',
			created_at: tsDay(30),
			context_label: 'Kontribusi RPJM Kelurahan'
		},
		{
			vouch_id: 'vc-702',
			user_id: 'u-002',
			user_name: 'Sari Dewi',
			user_tier: 3,
			vouch_type: 'collective',
			created_at: tsDay(45)
		},
		{
			vouch_id: 'vc-703',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(60)
		},
		{
			vouch_id: 'vc-704',
			user_id: 'u-018',
			user_name: 'Dra. Hartini',
			user_tier: 4,
			vouch_type: 'project_scoped',
			created_at: tsDay(90),
			context_label: 'Proyek Air Bersih RW 01'
		},
		{
			vouch_id: 'vc-705',
			user_id: 'u-019',
			user_name: 'Mulyadi',
			user_tier: 3,
			vouch_type: 'mentorship',
			created_at: tsDay(120)
		},
		{
			vouch_id: 'vc-706',
			user_id: 'u-020',
			user_name: 'Lestari Wulandari',
			user_tier: 3,
			vouch_type: 'positive',
			created_at: tsDay(150)
		},
		{
			vouch_id: 'vc-707',
			user_id: 'u-021',
			user_name: 'Joko Widodo',
			user_tier: 2,
			vouch_type: 'collective',
			created_at: tsDay(200)
		},
		{
			vouch_id: 'vc-708',
			user_id: 'u-022',
			user_name: 'Ratih Purnama',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(250)
		},
		{
			vouch_id: 'vc-709',
			user_id: 'u-023',
			user_name: 'Supriyadi',
			user_tier: 3,
			vouch_type: 'conditional',
			created_at: tsDay(300)
		},
		{
			vouch_id: 'vc-710',
			user_id: 'u-024',
			user_name: 'Nurul Aini',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(350)
		},
		{
			vouch_id: 'vc-711',
			user_id: 'u-025',
			user_name: 'Wahyudi',
			user_tier: 1,
			vouch_type: 'collective',
			created_at: tsDay(400)
		},
		{
			vouch_id: 'vc-712',
			user_id: 'u-026',
			user_name: 'Mariani',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(500)
		}
	],
	vouching_for: [
		{
			vouch_id: 'vc-801',
			user_id: 'u-002',
			user_name: 'Sari Dewi',
			user_tier: 3,
			vouch_type: 'positive',
			created_at: tsDay(10)
		},
		{
			vouch_id: 'vc-802',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'collective',
			created_at: tsDay(45)
		},
		{
			vouch_id: 'vc-803',
			user_id: 'u-003',
			user_name: 'Budi Santoso',
			user_tier: 1,
			vouch_type: 'mentorship',
			created_at: tsDay(30)
		},
		{
			vouch_id: 'vc-804',
			user_id: 'u-005',
			user_name: 'Hendra Wijaya',
			user_tier: 0,
			vouch_type: 'collective',
			created_at: tsDay(7)
		},
		{
			vouch_id: 'vc-805',
			user_id: 'u-027',
			user_name: 'Citra Dewi',
			user_tier: 2,
			vouch_type: 'positive',
			created_at: tsDay(20)
		},
		{
			vouch_id: 'vc-806',
			user_id: 'u-028',
			user_name: 'Prasetyo',
			user_tier: 1,
			vouch_type: 'positive',
			created_at: tsDay(55)
		},
		{
			vouch_id: 'vc-807',
			user_id: 'u-029',
			user_name: 'Widiawati',
			user_tier: 2,
			vouch_type: 'collective',
			created_at: tsDay(80)
		},
		{
			vouch_id: 'vc-808',
			user_id: 'u-030',
			user_name: 'Hendro Susanto',
			user_tier: 1,
			vouch_type: 'project_scoped',
			created_at: tsDay(100),
			context_label: 'Proyek Posyandu RW 01'
		}
	],
	vouch_budget: {
		max_vouches: 50,
		active_vouches: 8,
		remaining: 42
	},
	dukung_given: [
		{
			dukung_id: 'dk-601',
			witness_id: 'w-040',
			witness_title: 'Revitalisasi Posyandu RW 01',
			supporter_id: 'u-004',
			supporter_name: 'Rina Kartika',
			created_at: tsDay(5),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-602',
			witness_id: 'w-041',
			witness_title: 'Program Air Bersih RW 01',
			supporter_id: 'u-004',
			supporter_name: 'Rina Kartika',
			created_at: tsDay(15),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-603',
			witness_id: 'w-042',
			witness_title: 'Pemetaan Risiko Banjir Jakarta Selatan',
			supporter_id: 'u-004',
			supporter_name: 'Rina Kartika',
			created_at: tsDay(22),
			outcome: 'success'
		}
	],
	dukung_received: [
		{
			dukung_id: 'dk-701',
			witness_id: 'w-038',
			witness_title: 'Koordinasi Darurat Banjir RW 01',
			supporter_id: 'u-002',
			supporter_name: 'Sari Dewi',
			created_at: tsDay(10),
			outcome: 'success'
		},
		{
			dukung_id: 'dk-702',
			witness_id: 'w-039',
			witness_title: 'Mediasi Sengketa Lahan RW 01',
			supporter_id: 'u-017',
			supporter_name: 'Pak Lurah',
			created_at: tsDay(30),
			outcome: 'success'
		}
	],
	timeline: [
		{
			activity_id: 'tl-301',
			type: 'vouch_given',
			text: 'Memberi vouch mentor pada Budi Santoso',
			timestamp: tsDay(1)
		},
		{
			activity_id: 'tl-302',
			type: 'witness_created',
			text: 'Membuka laporan pemetaan risiko banjir Jakarta Selatan',
			timestamp: tsDay(2),
			witness_id: 'w-042'
		},
		{
			activity_id: 'tl-303',
			type: 'vote_cast',
			text: 'Memimpin musyawarah prioritas anggaran RW 01',
			timestamp: tsDay(3)
		},
		{
			activity_id: 'tl-304',
			type: 'resolution_completed',
			text: 'Eskalasi laporan RT 05 berhasil diselesaikan',
			timestamp: tsDay(4),
			witness_id: 'w-041'
		},
		{
			activity_id: 'tl-305',
			type: 'evidence_submitted',
			text: 'Mendokumentasikan kondisi drainase RW 01',
			timestamp: tsDay(5)
		},
		{
			activity_id: 'tl-306',
			type: 'vouch_received',
			text: 'Menerima vouch proyek dari Dra. Hartini',
			timestamp: tsDay(7)
		},
		{
			activity_id: 'tl-307',
			type: 'dukung_given',
			text: 'Mendukung revitalisasi Posyandu RW 01',
			timestamp: tsDay(8),
			witness_id: 'w-040'
		},
		{
			activity_id: 'tl-308',
			type: 'witness_joined',
			text: 'Menjadi koordinator utama saksi banjir RW 01',
			timestamp: tsDay(10),
			witness_id: 'w-038'
		},
		{
			activity_id: 'tl-309',
			type: 'skill_validated',
			text: 'Semua kompetensi GR diperbarui dan divalidasi',
			timestamp: tsDay(12)
		},
		{
			activity_id: 'tl-310',
			type: 'vote_cast',
			text: 'Berpartisipasi dalam voting RPJM Kelurahan 2024',
			timestamp: tsDay(14)
		},
		{
			activity_id: 'tl-311',
			type: 'resolution_completed',
			text: 'Program air bersih RW 01 berhasil diselesaikan',
			timestamp: tsDay(15),
			witness_id: 'w-041'
		},
		{
			activity_id: 'tl-312',
			type: 'vouch_given',
			text: 'Memberi vouch kolektif pada Hendra Wijaya',
			timestamp: tsDay(17)
		},
		{
			activity_id: 'tl-313',
			type: 'dukung_given',
			text: 'Mendukung program air bersih RW 01',
			timestamp: tsDay(18),
			witness_id: 'w-041'
		},
		{
			activity_id: 'tl-314',
			type: 'evidence_submitted',
			text: 'Mengirim laporan hasil mediasi sengketa lahan',
			timestamp: tsDay(20)
		},
		{
			activity_id: 'tl-315',
			type: 'vouch_given',
			text: 'Memberi vouch positif pada Ahmad Hidayat',
			timestamp: tsDay(22)
		},
		{
			activity_id: 'tl-316',
			type: 'witness_created',
			text: 'Membuka investigasi kondisi jalan RW 01',
			timestamp: tsDay(25),
			witness_id: 'w-043'
		},
		{
			activity_id: 'tl-317',
			type: 'vote_cast',
			text: 'Memilih koordinator saksi baru RW 01',
			timestamp: tsDay(27)
		},
		{
			activity_id: 'tl-318',
			type: 'resolution_completed',
			text: 'Mediasi sengketa lahan RW 01 berhasil',
			timestamp: tsDay(30),
			witness_id: 'w-039'
		},
		{
			activity_id: 'tl-319',
			type: 'tier_change',
			text: 'Mencapai tier Kunci (◆◆◆◆) — status veteran',
			timestamp: tsDay(365)
		},
		{
			activity_id: 'tl-320',
			type: 'skill_validated',
			text: 'Kompetensi Koordinasi Komunitas pertama kali divalidasi',
			timestamp: tsDay(400)
		}
	],
	impact: {
		witnesses_resolved: 15,
		people_helped: 120,
		total_dukung_given: 28,
		total_dukung_received: 35,
		evidence_validated: 45,
		votes_participated: 52
	},
	decay_warnings: []
};

// ---------------------------------------------------------------------------
// mockTandangProfile5 — Hendra Wijaya, tier 0 Bayangan
// ---------------------------------------------------------------------------

export const mockTandangProfile5: TandangProfile = {
	user_id: 'u-005',
	name: 'Hendra Wijaya',
	tier: {
		level: 0,
		name: 'Bayangan',
		pips: '◇◇◇◇',
		color: '#9E9E9E',
		percentile: 5
	},
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	joined_at: tsDay(7),
	location: 'RT 12',
	last_active_at: tsHour(24),
	scores: {
		integrity: { value: 0.15 },
		competence: {
			aggregate: 0.08,
			domains: []
		},
		judgment: {
			value: 0.05,
			vouch_outcomes_count: 0,
			dukung_success_rate: null
		}
	},
	consistency: {
		multiplier: 1.0,
		streak_days: 1,
		streak_weeks: 0,
		contributions_30d: 1,
		quality_avg: 0.30,
		gap_days: 5
	},
	genesis: {
		weight: 100.0,
		meaningful_interactions_this_month: 0,
		threshold: 3
	},
	skills: [
		{
			skill_id: 'declared-005',
			skill_name: 'Pelaporan Masalah Lingkungan',
			validated: false
		}
	],
	vouched_by: [
		{
			vouch_id: 'vc-901',
			user_id: 'u-001',
			user_name: 'Ahmad Hidayat',
			user_tier: 2,
			vouch_type: 'collective',
			created_at: tsDay(5),
			context_label: 'Warga baru RT 12'
		}
	],
	vouching_for: [],
	vouch_budget: {
		max_vouches: 5,
		active_vouches: 0,
		remaining: 5
	},
	dukung_given: [],
	dukung_received: [],
	timeline: [
		{
			activity_id: 'tl-401',
			type: 'tier_change',
			text: 'Bergabung sebagai warga baru Bayangan (◇◇◇◇)',
			timestamp: tsDay(7)
		},
		{
			activity_id: 'tl-402',
			type: 'vouch_received',
			text: 'Menerima vouch pertama dari Ahmad Hidayat',
			timestamp: tsDay(5)
		},
		{
			activity_id: 'tl-403',
			type: 'witness_joined',
			text: 'Bergabung sebagai pengamat laporan banjir RT 12',
			timestamp: tsDay(2),
			witness_id: 'w-050'
		}
	],
	impact: {
		witnesses_resolved: 0,
		people_helped: 1,
		total_dukung_given: 0,
		total_dukung_received: 0,
		evidence_validated: 0,
		votes_participated: 2
	},
	decay_warnings: []
};

// ---------------------------------------------------------------------------
// All profiles array
// ---------------------------------------------------------------------------

export const mockTandangProfiles: TandangProfile[] = [
	mockTandangProfile1,
	mockTandangProfile2,
	mockTandangProfile3,
	mockTandangProfile4,
	mockTandangProfile5
];

// ---------------------------------------------------------------------------
// Current user profile (default logged-in user for dev/testing)
// ---------------------------------------------------------------------------

export const mockCurrentTandangProfile: TandangProfile = mockTandangProfile1;

// ---------------------------------------------------------------------------
// Mock PersonRelation data (keyed by user_id)
// ---------------------------------------------------------------------------

export const mockPersonRelations: Record<string, PersonRelation> = {
	'u-002': { vouched: true, vouch_type: 'positive', vouched_back: true, skeptical: false },
	'u-003': { vouched: false, vouched_back: false, skeptical: true },
	'u-004': { vouched: true, vouch_type: 'mentorship', vouched_back: false, skeptical: false },
	'u-005': { vouched: true, vouch_type: 'collective', vouched_back: false, skeptical: false }
};

/**
 * Mock user profile fixtures for the dev gallery.
 * 5 user profiles covering all roles (user, moderator, admin) and tier levels 0-4.
 */

import type { UserProfile } from '$lib/types';

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
// Individual user profiles (exported for component-level testing)
// ---------------------------------------------------------------------------

export const mockUser1: UserProfile = {
	user_id: 'u-001',
	name: 'Ahmad Hidayat',
	role: 'user',
	tier: 2,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(180),
	location: 'RT 05',
	stats: {
		witnesses_created: 3,
		witnesses_participated: 8,
		evidence_submitted: 12,
		votes_cast: 15,
		resolutions_completed: 5
	},
	tandang_signals: {
		vouch: 18,
		dukung: 14,
		proof_of_resolve: 7,
		skeptis: 2
	},
	octalysis: {
		epic_meaning: 85,
		accomplishment: 72,
		empowerment: 60,
		social_influence: 78,
		unpredictability: 45
	},
	recent_activity: [
		{ text: 'Memberi Vouch pada laporan banjir', timestamp: tsHour(2) },
		{ text: 'Menjadi saksi untuk jalan rusak RT 05', timestamp: tsHour(26) },
		{ text: 'Menyelesaikan penggalangan dana', timestamp: tsHour(72) }
	]
};

export const mockUser2: UserProfile = {
	user_id: 'u-002',
	name: 'Sari Dewi',
	role: 'moderator',
	tier: 3,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(365),
	location: 'RW 03',
	stats: {
		witnesses_created: 7,
		witnesses_participated: 15,
		evidence_submitted: 23,
		votes_cast: 31,
		resolutions_completed: 9
	},
	tandang_signals: {
		vouch: 34,
		dukung: 28,
		proof_of_resolve: 15,
		skeptis: 4
	},
	octalysis: {
		epic_meaning: 90,
		accomplishment: 85,
		empowerment: 75,
		social_influence: 88,
		unpredictability: 55
	},
	recent_activity: [
		{ text: 'Moderasi diskusi pembangunan taman', timestamp: tsHour(1) },
		{ text: 'Verifikasi bukti laporan sampah', timestamp: tsHour(8) },
		{ text: 'Memberi Vouch pada 3 saksi baru', timestamp: tsHour(48) }
	]
};

export const mockUser3: UserProfile = {
	user_id: 'u-003',
	name: 'Budi Santoso',
	role: 'user',
	tier: 1,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(90),
	location: 'RT 08',
	stats: {
		witnesses_created: 1,
		witnesses_participated: 4,
		evidence_submitted: 5,
		votes_cast: 8,
		resolutions_completed: 2
	},
	tandang_signals: {
		vouch: 6,
		dukung: 4,
		proof_of_resolve: 2,
		skeptis: 1
	},
	octalysis: {
		epic_meaning: 55,
		accomplishment: 40,
		empowerment: 35,
		social_influence: 50,
		unpredictability: 30
	},
	recent_activity: [
		{ text: 'Mengirim bukti foto jalan berlubang', timestamp: tsHour(5) },
		{ text: 'Bergabung sebagai saksi banjir', timestamp: tsHour(72) }
	]
};

export const mockUser4: UserProfile = {
	user_id: 'u-004',
	name: 'Rina Kartika',
	avatar_url: 'https://placehold.co/40x40/C05621/white?text=RK',
	role: 'admin',
	tier: 4,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(730),
	location: 'RW 01',
	stats: {
		witnesses_created: 12,
		witnesses_participated: 28,
		evidence_submitted: 45,
		votes_cast: 52,
		resolutions_completed: 15
	},
	tandang_signals: {
		vouch: 56,
		dukung: 48,
		proof_of_resolve: 25,
		skeptis: 6
	},
	octalysis: {
		epic_meaning: 95,
		accomplishment: 92,
		empowerment: 88,
		social_influence: 94,
		unpredictability: 60
	},
	recent_activity: [
		{ text: 'Menetapkan koordinator saksi baru', timestamp: tsHour(3) },
		{ text: 'Menyelesaikan eskalasi laporan RT 05', timestamp: tsHour(12) },
		{ text: 'Membuka voting penggalangan dana', timestamp: tsHour(36) }
	]
};

export const mockUser5: UserProfile = {
	user_id: 'u-005',
	name: 'Hendra Wijaya',
	role: 'user',
	tier: 0,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(7),
	location: 'RT 12',
	stats: {
		witnesses_created: 0,
		witnesses_participated: 1,
		evidence_submitted: 0,
		votes_cast: 2,
		resolutions_completed: 0
	},
	tandang_signals: {
		vouch: 1,
		dukung: 0,
		proof_of_resolve: 0,
		skeptis: 0
	},
	octalysis: {
		epic_meaning: 30,
		accomplishment: 15,
		empowerment: 10,
		social_influence: 20,
		unpredictability: 25
	},
	recent_activity: [{ text: 'Bergabung sebagai warga baru', timestamp: tsHour(24) }]
};

// ---------------------------------------------------------------------------
// All users array
// ---------------------------------------------------------------------------

export const mockUsers: UserProfile[] = [mockUser1, mockUser2, mockUser3, mockUser4, mockUser5];

// ---------------------------------------------------------------------------
// Current user (default logged-in user for dev/testing)
// ---------------------------------------------------------------------------

export const mockCurrentUser: UserProfile = mockUser1;

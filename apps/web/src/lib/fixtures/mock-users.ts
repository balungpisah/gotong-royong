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
	stats: {
		witnesses_created: 3,
		witnesses_participated: 8,
		evidence_submitted: 12,
		votes_cast: 15
	}
};

export const mockUser2: UserProfile = {
	user_id: 'u-002',
	name: 'Sari Dewi',
	role: 'moderator',
	tier: 3,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(365),
	stats: {
		witnesses_created: 7,
		witnesses_participated: 15,
		evidence_submitted: 23,
		votes_cast: 31
	}
};

export const mockUser3: UserProfile = {
	user_id: 'u-003',
	name: 'Budi Santoso',
	role: 'user',
	tier: 1,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(90),
	stats: {
		witnesses_created: 1,
		witnesses_participated: 4,
		evidence_submitted: 5,
		votes_cast: 8
	}
};

export const mockUser4: UserProfile = {
	user_id: 'u-004',
	name: 'Rina Kartika',
	avatar_url: 'https://placehold.co/40x40/C05621/white?text=RK',
	role: 'admin',
	tier: 4,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(730),
	stats: {
		witnesses_created: 12,
		witnesses_participated: 28,
		evidence_submitted: 45,
		votes_cast: 52
	}
};

export const mockUser5: UserProfile = {
	user_id: 'u-005',
	name: 'Hendra Wijaya',
	role: 'user',
	tier: 0,
	community_id: 'comm-jakarta-selatan',
	joined_at: tsDay(7),
	stats: {
		witnesses_created: 0,
		witnesses_participated: 1,
		evidence_submitted: 0,
		votes_cast: 2
	}
};

// ---------------------------------------------------------------------------
// All users array
// ---------------------------------------------------------------------------

export const mockUsers: UserProfile[] = [mockUser1, mockUser2, mockUser3, mockUser4, mockUser5];

// ---------------------------------------------------------------------------
// Current user (default logged-in user for dev/testing)
// ---------------------------------------------------------------------------

export const mockCurrentUser: UserProfile = mockUser1;

/**
 * Mock CommunityDashboard fixture for the dev gallery.
 * Represents Jakarta Selatan community with 847 members.
 */

import type { CommunityDashboard } from '$lib/types';

export const mockCommunityDashboard: CommunityDashboard = {
	community_id: 'comm-jakarta-selatan',
	community_name: 'Jakarta Selatan',
	member_count: 847,
	weather: {
		weather: 'cerah',
		emoji: '☀️',
		multiplier: 1.0,
		label: 'Komunitas aktif dan sehat'
	},
	icj_summary: {
		avg_integrity: 0.58,
		avg_competence: 0.51,
		avg_judgment: 0.45
	},
	tier_distribution: [
		{
			tier: 0,
			tier_name: 'Bayangan',
			count: 42,
			percentage: 5,
			color: '#9E9E9E'
		},
		{
			tier: 1,
			tier_name: 'Pemula',
			count: 285,
			percentage: 34,
			color: '#795548'
		},
		{
			tier: 2,
			tier_name: 'Kontributor',
			count: 312,
			percentage: 37,
			color: '#00695C'
		},
		{
			tier: 3,
			tier_name: 'Pilar',
			count: 168,
			percentage: 20,
			color: '#1E88E5'
		},
		{
			tier: 4,
			tier_name: 'Kunci',
			count: 40,
			percentage: 5,
			color: '#FFD700'
		}
	],
	avg_tier: 1.85,
	active_highlights: [
		{
			user_id: 'u-002',
			name: 'Sari Dewi',
			tier: 3,
			highlight_reason: 'Top kontributor minggu ini',
			contributions_this_week: 15,
			streak_days: 42
		},
		{
			user_id: 'u-004',
			name: 'Rina Kartika',
			tier: 4,
			highlight_reason: 'Resolusi tercepat bulan ini',
			contributions_this_week: 22,
			streak_days: 180
		},
		{
			user_id: 'u-001',
			name: 'Ahmad Hidayat',
			tier: 2,
			highlight_reason: 'Saksi paling aktif minggu ini',
			contributions_this_week: 8,
			streak_days: 14
		},
		{
			user_id: 'u-007',
			name: 'Rudi Prasetyo',
			tier: 2,
			highlight_reason: 'Vouch terbanyak diberikan',
			contributions_this_week: 6,
			streak_days: 21
		},
		{
			user_id: 'u-005',
			name: 'Hendra Wijaya',
			tier: 0,
			highlight_reason: 'Pendatang baru teraktif',
			contributions_this_week: 1,
			streak_days: 1
		}
	],
	signal_flow: [
		{
			week_label: 'Mgg 1',
			vouch: 48,
			skeptis: 9,
			dukung: 35,
			proof_of_resolve: 14,
			perlu_dicek: 7
		},
		{
			week_label: 'Mgg 2',
			vouch: 55,
			skeptis: 12,
			dukung: 42,
			proof_of_resolve: 18,
			perlu_dicek: 9
		},
		{
			week_label: 'Mgg 3',
			vouch: 45,
			skeptis: 8,
			dukung: 38,
			proof_of_resolve: 15,
			perlu_dicek: 6
		},
		{
			week_label: 'Mgg 4',
			vouch: 60,
			skeptis: 14,
			dukung: 47,
			proof_of_resolve: 20,
			perlu_dicek: 10
		}
	]
};

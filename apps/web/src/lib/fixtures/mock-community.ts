/**
 * Mock community data fixtures.
 * Matches the hardcoded values previously in community-pulse.svelte.
 */

import type {
	CommunityStats,
	ParticipationDataPoint,
	CommunitySignalSummary,
	CommunityActivityItem
} from '$lib/types';

// ---------------------------------------------------------------------------
// Community stats
// ---------------------------------------------------------------------------

export const mockCommunityStats: CommunityStats = {
	active_witness_count: 24,
	active_witness_delta: 3,
	messages_today: 47,
	conversations_today: 12,
	resolution_rate: 73,
	tandang_signals_this_week: 156
};

// ---------------------------------------------------------------------------
// 7-day participation chart data
// ---------------------------------------------------------------------------

export const mockParticipation: ParticipationDataPoint[] = [
	{ day: 'Sen', value: 40 },
	{ day: 'Sel', value: 65 },
	{ day: 'Rab', value: 55 },
	{ day: 'Kam', value: 80 },
	{ day: 'Jum', value: 70 },
	{ day: 'Sab', value: 90 },
	{ day: 'Min', value: 60 }
];

// ---------------------------------------------------------------------------
// Tandang signal summary
// ---------------------------------------------------------------------------

export const mockCommunitySignals: CommunitySignalSummary = {
	vouch: 45,
	skeptis: 12,
	proof_of_resolve: 38,
	dukung: 42,
	perlu_dicek: 19
};

// ---------------------------------------------------------------------------
// Recent community activity
// ---------------------------------------------------------------------------

export const mockCommunityActivity: CommunityActivityItem[] = [
	{
		icon_type: 'vouch',
		text: 'Pak Ahmad memberi Vouch pada laporan jalan rusak',
		time_label: '2m'
	},
	{
		icon_type: 'contribute',
		text: 'Ibu Sari menyumbang Rp 500.000 untuk perbaikan',
		time_label: '15m'
	},
	{
		icon_type: 'verify',
		text: '3 saksi baru bergabung untuk verifikasi banjir',
		time_label: '1j'
	},
	{
		icon_type: 'resolve',
		text: 'Laporan jalan rusak RT 05 berhasil diselesaikan',
		time_label: '3j'
	}
];

/**
 * Community domain types — aggregate stats, signals, and activity.
 */

/**
 * Top-level community health statistics.
 */
export interface CommunityStats {
	active_witness_count: number;
	active_witness_delta: number;
	messages_today: number;
	conversations_today: number;
	/** Resolution success rate 0-100. */
	resolution_rate: number;
	tandang_signals_this_week: number;
}

/**
 * A single data point for the 7-day participation chart.
 */
export interface ParticipationDataPoint {
	day: string;
	value: number;
}

/**
 * Aggregate tandang signal counts for the community.
 */
export interface CommunitySignalSummary {
	vouch: number;
	skeptis: number;
	proof_of_resolve: number;
	bagus: number;
	perlu_dicek: number;
}

/**
 * A recent community activity feed item.
 */
export interface CommunityActivityItem {
	icon_type: 'vouch' | 'contribute' | 'verify' | 'resolve';
	text: string;
	time_label: string;
}

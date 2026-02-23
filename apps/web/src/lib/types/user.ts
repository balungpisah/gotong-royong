/**
 * User domain types — profile and reputation.
 */

import type { AuthRole } from '$lib/auth';

/**
 * Tandang reputation signals received from community.
 */
export interface TandangSignals {
	vouch: number;
	dukung: number;
	proof_of_resolve: number;
	skeptis: number;
}

/**
 * Octalysis gamification engagement scores (0-100).
 * Subset of 5 core drives tracked in this app (of 8 total).
 * Omitted: ownership, scarcity, avoidance — not yet relevant to civic engagement model.
 */
export interface OctalysisScores {
	epic_meaning: number;
	accomplishment: number;
	empowerment: number;
	social_influence: number;
	unpredictability: number;
}

/**
 * A single recent activity entry.
 */
export interface ActivityItem {
	text: string;
	timestamp: string;
}

/**
 * User profile aggregate.
 */
export interface UserProfile {
	user_id: string;
	name: string;
	avatar_url?: string;
	role: AuthRole;
	/** Reputation tier 0-4. */
	tier: number;
	community_id?: string;
	joined_at: string;
	stats: UserStats;
	location?: string;
	tandang_signals?: TandangSignals;
	octalysis?: OctalysisScores;
	recent_activity?: ActivityItem[];
}

/**
 * User activity statistics.
 */
export interface UserStats {
	witnesses_created: number;
	witnesses_participated: number;
	evidence_submitted: number;
	votes_cast: number;
	resolutions_completed: number;
}

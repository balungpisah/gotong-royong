/**
 * User domain types — profile and reputation.
 */

import type { AuthRole } from '$lib/auth';

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
}

/**
 * User activity statistics.
 */
export interface UserStats {
	witnesses_created: number;
	witnesses_participated: number;
	evidence_submitted: number;
	votes_cast: number;
}

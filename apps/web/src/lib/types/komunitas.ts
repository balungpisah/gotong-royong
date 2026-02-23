/**
 * Community dashboard types — full page model.
 * The compact sidebar continues using CommunityStats from community.ts.
 */

import type { TandangTierLevel, GdfWeather } from './tandang';

export interface TierDistribution {
	tier: TandangTierLevel;
	tier_name: string;
	count: number;
	percentage: number;
	color: string;
}

export interface CommunityIcjSummary {
	avg_integrity: number;
	avg_competence: number;
	avg_judgment: number;
}

export interface ActiveMemberHighlight {
	user_id: string;
	name: string;
	avatar_url?: string;
	tier: TandangTierLevel;
	highlight_reason: string;
	contributions_this_week: number;
	streak_days: number;
}

export interface SignalFlowDataPoint {
	week_label: string;
	vouch: number;
	skeptis: number;
	dukung: number;
	proof_of_resolve: number;
	perlu_dicek: number;
}

export interface CommunityDashboard {
	community_id: string;
	community_name: string;
	member_count: number;
	weather: GdfWeather;
	icj_summary: CommunityIcjSummary;
	tier_distribution: TierDistribution[];
	avg_tier: number;
	active_highlights: ActiveMemberHighlight[];
	signal_flow: SignalFlowDataPoint[];
}

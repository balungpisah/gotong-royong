/**
 * Tandang reputation engine types — full I/C/J model.
 * Used by the CV Hidup (Aku) full page. The compact sidebar continues
 * using the simpler TandangSignals from user.ts.
 */

// ---------------------------------------------------------------------------
// Tier System
// ---------------------------------------------------------------------------

/** Tandang tier 0-4. */
export type TandangTierLevel = 0 | 1 | 2 | 3 | 4;

export type TandangTierName = 'Bayangan' | 'Pemula' | 'Kontributor' | 'Pilar' | 'Kunci';

export interface TandangTier {
	level: TandangTierLevel;
	name: TandangTierName;
	/** Diamond pip string e.g. "◆◆◆◇" */
	pips: string;
	/** Tier color hex from spec */
	color: string;
	/** Percentile rank within community (0-100). */
	percentile: number;
}

// ---------------------------------------------------------------------------
// I/C/J Score Model
// ---------------------------------------------------------------------------

export interface IntegrityScore {
	value: number; // 0.0 - 1.0
}

export interface CompetenceDomain {
	skill_id: string;
	skill_name: string;
	score: number; // 0.0 - 1.0
	decaying: boolean;
	days_until_decay: number | null;
	last_activity: string; // ISO date
	validated: boolean;
}

export interface CompetenceScore {
	aggregate: number; // 0.0 - 1.0
	domains: CompetenceDomain[];
}

export interface JudgmentScore {
	value: number; // 0.0 - 1.0
	vouch_outcomes_count: number;
	dukung_success_rate: number | null;
}

export interface TandangScores {
	integrity: IntegrityScore;
	competence: CompetenceScore;
	judgment: JudgmentScore;
}

// ---------------------------------------------------------------------------
// Consistency & Activity
// ---------------------------------------------------------------------------

export interface ConsistencyInfo {
	multiplier: number; // 1.0 - 1.2
	streak_days: number;
	streak_weeks: number;
	contributions_30d: number;
	quality_avg: number;
	gap_days: number;
}

export interface GenesisInfo {
	weight: number | null;
	meaningful_interactions_this_month: number;
	threshold: number;
}

// ---------------------------------------------------------------------------
// Vouch Network
// ---------------------------------------------------------------------------

export type VouchType =
	| 'positive'
	| 'collective'
	| 'skeptical'
	| 'conditional'
	| 'mentorship'
	| 'project_scoped';

export interface VouchRelation {
	vouch_id: string;
	user_id: string;
	user_name: string;
	user_avatar_url?: string;
	user_tier: TandangTierLevel;
	vouch_type: VouchType;
	created_at: string;
	context_entity_id?: string;
	context_label?: string;
}

// ---------------------------------------------------------------------------
// Person Relation (my trust relationship with another person)
// ---------------------------------------------------------------------------

export interface PersonRelation {
	vouched: boolean;
	vouch_type?: VouchType;
	vouched_back: boolean;
	skeptical: boolean;
}

export interface TandangAvatarPerson {
	user_id: string;
	name: string;
	avatar_url?: string;
	tier?: TandangTierLevel;
	role?: string;
}

export interface VouchBudget {
	max_vouches: number;
	active_vouches: number;
	remaining: number;
}

// ---------------------------------------------------------------------------
// Dukung (GR-only, not tandang trust graph)
// ---------------------------------------------------------------------------

export interface DukungRecord {
	dukung_id: string;
	witness_id: string;
	witness_title: string;
	supporter_id: string;
	supporter_name: string;
	supporter_avatar_url?: string;
	created_at: string;
	outcome?: 'success' | 'slashed' | 'pending';
}

// ---------------------------------------------------------------------------
// Skills (ESCO-ID GR Domains)
// ---------------------------------------------------------------------------

export type GrSkillDomain =
	| 'ESCO-ID-GR-001'
	| 'ESCO-ID-GR-002'
	| 'ESCO-ID-GR-003'
	| 'ESCO-ID-GR-004'
	| 'ESCO-ID-GR-005';

export interface UserSkill {
	skill_id: string;
	skill_name: string;
	validated: boolean;
	score?: number;
	decaying?: boolean;
	days_until_decay?: number | null;
}

// ---------------------------------------------------------------------------
// GDF Weather Widget
// ---------------------------------------------------------------------------

export type WeatherType = 'cerah' | 'berawan' | 'hujan' | 'badai';

export interface GdfWeather {
	weather: WeatherType;
	emoji: string;
	multiplier: number;
	label: string;
}

// ---------------------------------------------------------------------------
// Impact Metrics
// ---------------------------------------------------------------------------

export interface ImpactMetrics {
	witnesses_resolved: number;
	people_helped: number;
	total_dukung_given: number;
	total_dukung_received: number;
	evidence_validated: number;
	votes_participated: number;
}

// ---------------------------------------------------------------------------
// Activity Timeline
// ---------------------------------------------------------------------------

export interface ActivityTimelineItem {
	activity_id: string;
	type:
		| 'witness_created'
		| 'witness_joined'
		| 'evidence_submitted'
		| 'vouch_given'
		| 'vouch_received'
		| 'vote_cast'
		| 'resolution_completed'
		| 'skill_validated'
		| 'dukung_given'
		| 'dukung_received'
		| 'tier_change';
	text: string;
	timestamp: string;
	witness_id?: string;
	icon_hint?: string;
}

// ---------------------------------------------------------------------------
// Full Tandang Profile
// ---------------------------------------------------------------------------

export interface TandangProfile {
	user_id: string;
	name: string;
	avatar_url?: string;
	tier: TandangTier;
	community_id: string;
	community_name: string;
	joined_at: string;
	location?: string;
	last_active_at: string;
	scores: TandangScores;
	consistency: ConsistencyInfo;
	genesis: GenesisInfo;
	skills: UserSkill[];
	vouched_by: VouchRelation[];
	vouching_for: VouchRelation[];
	vouch_budget: VouchBudget;
	dukung_given: DukungRecord[];
	dukung_received: DukungRecord[];
	timeline: ActivityTimelineItem[];
	impact: ImpactMetrics;
	decay_warnings: Array<{ domain: string; days_until_decay: number }>;
}

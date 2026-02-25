import type { AuthRole } from '$lib/auth';
import type { ApiClient } from '$lib/api';
import type {
	ActivityItem,
	TandangProfile,
	TandangTierLevel,
	UserProfile,
	UserStats
} from '$lib/types';
import type { UserService } from '../types';

type JsonRecord = Record<string, unknown>;

interface ApiAuthMe {
	user_id?: string;
	username?: string;
	role?: string;
}

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const nowIso = () => new Date().toISOString();

const toAuthRole = (value: string | undefined, fallback: AuthRole): AuthRole => {
	const normalized = value?.trim().toLowerCase();
	if (normalized === 'admin' || normalized === 'moderator' || normalized === 'system') {
		return normalized;
	}
	if (normalized === 'user') {
		return 'user';
	}
	return fallback;
};

const defaultStats = (): UserStats => ({
	witnesses_created: 0,
	witnesses_participated: 0,
	evidence_submitted: 0,
	votes_cast: 0,
	resolutions_completed: 0
});

const defaultUserProfile = (userId: string): UserProfile => ({
	user_id: userId,
	name: userId,
	role: 'user',
	tier: 0,
	joined_at: nowIso(),
	stats: defaultStats()
});

const defaultTandangProfile = (userId: string): TandangProfile => ({
	user_id: userId,
	name: userId,
	community_id: 'unknown',
	community_name: 'Unknown',
	joined_at: nowIso(),
	last_active_at: nowIso(),
	tier: {
		level: 0,
		name: 'Bayangan',
		pips: '◇◇◇◇',
		color: '#9E9E9E',
		percentile: 0
	},
	scores: {
		integrity: { value: 0 },
		competence: { aggregate: 0, domains: [] },
		judgment: { value: 0, vouch_outcomes_count: 0, dukung_success_rate: null }
	},
	consistency: {
		multiplier: 1,
		streak_days: 0,
		streak_weeks: 0,
		contributions_30d: 0,
		quality_avg: 0,
		gap_days: 0
	},
	genesis: {
		weight: null,
		meaningful_interactions_this_month: 0,
		threshold: 3
	},
	skills: [],
	vouched_by: [],
	vouching_for: [],
	vouch_budget: {
		max_vouches: 20,
		active_vouches: 0,
		remaining: 20
	},
	dukung_given: [],
	dukung_received: [],
	timeline: [],
	impact: {
		witnesses_resolved: 0,
		people_helped: 0,
		total_dukung_given: 0,
		total_dukung_received: 0,
		evidence_validated: 0,
		votes_participated: 0
	},
	decay_warnings: []
});

const normalizeEnvelopeData = (value: unknown): JsonRecord => {
	if (!isRecord(value)) {
		return {};
	}
	if (isRecord(value.data)) {
		return value.data;
	}
	return value;
};

const platformUserIdFromIdentity = (value: string | undefined): string | undefined => {
	if (!value) return undefined;
	const [platform, ...rest] = value.split(':');
	if (!platform || rest.length === 0) return value;
	return rest.join(':');
};

const readPlatformUserId = (profileData: JsonRecord): string | undefined =>
	asString(profileData.platform_user_id) ??
	platformUserIdFromIdentity(asString(profileData.identity));

const mapUserProfile = (
	profileEnvelope: unknown,
	fallbackProfile: UserProfile,
	authMe?: ApiAuthMe
): UserProfile => {
	const data = normalizeEnvelopeData(profileEnvelope);
	const reputation = isRecord(data.reputation) ? data.reputation : undefined;
	const activity = isRecord(data.activity) ? data.activity : undefined;
	const cvHidup = isRecord(data.cv_hidup) ? data.cv_hidup : undefined;
	const stats: UserStats = {
		witnesses_created:
			asNumber(activity?.solutions_submitted) ??
			asNumber(activity?.witnesses_created) ??
			fallbackProfile.stats.witnesses_created,
		witnesses_participated:
			asNumber(activity?.witnesses_participated) ?? fallbackProfile.stats.witnesses_participated,
		evidence_submitted:
			asNumber(activity?.evidence_submitted) ?? fallbackProfile.stats.evidence_submitted,
		votes_cast: asNumber(activity?.votes_cast) ?? fallbackProfile.stats.votes_cast,
		resolutions_completed:
			asNumber(activity?.resolutions_completed) ?? fallbackProfile.stats.resolutions_completed
	};
	const tier = readTierLevel(data, (fallbackProfile.tier ?? 0) as TandangTierLevel);
	return {
		...fallbackProfile,
		user_id:
			asString(authMe?.user_id) ??
			readPlatformUserId(data) ??
			asString(cvHidup?.platform_user_id) ??
			fallbackProfile.user_id,
		name:
			asString(authMe?.username) ??
			asString(cvHidup?.username) ??
			asString(reputation?.username) ??
			fallbackProfile.name,
		role: authMe ? toAuthRole(asString(authMe.role), fallbackProfile.role) : fallbackProfile.role,
		tier,
		community_id:
			asString(cvHidup?.community_id) ?? asString(data.identity) ?? fallbackProfile.community_id,
		joined_at: asString(cvHidup?.joined_at) ?? fallbackProfile.joined_at ?? nowIso(),
		recent_activity: mapActivityItems(activity),
		stats
	};
};

const mapTandangProfile = (
	profileEnvelope: unknown,
	fallbackProfile: TandangProfile,
	authMe?: ApiAuthMe,
	budgetEnvelope?: unknown,
	decayEnvelope?: unknown
): TandangProfile => {
	const profileData = normalizeEnvelopeData(profileEnvelope);
	const reputation = isRecord(profileData.reputation) ? profileData.reputation : undefined;
	const activity = isRecord(profileData.activity) ? profileData.activity : undefined;
	const cvHidup = isRecord(profileData.cv_hidup) ? profileData.cv_hidup : undefined;
	const budgetData = normalizeEnvelopeData(budgetEnvelope);
	const decayData = normalizeEnvelopeData(decayEnvelope);

	const userId =
		asString(authMe?.user_id) ??
		readPlatformUserId(profileData) ??
		asString(cvHidup?.platform_user_id) ??
		fallbackProfile.user_id;

	const tierLevel = readTierLevel(profileData, fallbackProfile.tier.level);
	const tierNameByLevel = ['Bayangan', 'Pemula', 'Kontributor', 'Pilar', 'Kunci'] as const;
	const tierPipsByLevel = ['◇◇◇◇', '◆◇◇◇', '◆◆◇◇', '◆◆◆◇', '◆◆◆◆'] as const;
	const tierColorByLevel = ['#9E9E9E', '#2E7D32', '#00695C', '#1E88E5', '#5E35B1'] as const;

	const warningsSource = Array.isArray(decayData.warnings)
		? decayData.warnings
		: Array.isArray(decayData.items)
			? decayData.items
			: [];
	const decayWarnings = warningsSource
		.map((item) => {
			if (!isRecord(item)) return undefined;
			const domain = asString(item.domain) ?? asString(item.skill_name);
			const days =
				asNumber(item.days_until_decay) ??
				asNumber(item.days_remaining) ??
				asNumber(item.days_left);
			if (!domain || days === undefined) return undefined;
			return { domain, days_until_decay: Math.max(0, Math.round(days)) };
		})
		.filter((item): item is { domain: string; days_until_decay: number } => Boolean(item));

	return {
		...fallbackProfile,
		user_id: userId,
		name:
			asString(authMe?.username) ??
			asString(cvHidup?.username) ??
			asString(reputation?.username) ??
			fallbackProfile.name,
		community_id: asString(cvHidup?.community_id) ?? fallbackProfile.community_id,
		community_name:
			asString(cvHidup?.community_name) ??
			asString(cvHidup?.community_id) ??
			fallbackProfile.community_name,
		joined_at: asString(cvHidup?.joined_at) ?? fallbackProfile.joined_at,
		last_active_at:
			asString(activity?.last_active_at) ??
			asString(activity?.updated_at) ??
			fallbackProfile.last_active_at,
		tier: {
			level: tierLevel,
			name: tierNameByLevel[tierLevel],
			pips: tierPipsByLevel[tierLevel],
			color: tierColorByLevel[tierLevel],
			percentile:
				asNumber(reputation?.percentile) ??
				asNumber(profileData.percentile) ??
				fallbackProfile.tier.percentile
		},
		vouch_budget: {
			max_vouches: asNumber(budgetData.max_vouches) ?? fallbackProfile.vouch_budget.max_vouches,
			active_vouches:
				asNumber(budgetData.active_vouches) ?? fallbackProfile.vouch_budget.active_vouches,
			remaining: asNumber(budgetData.remaining) ?? fallbackProfile.vouch_budget.remaining
		},
		decay_warnings: decayWarnings.length ? decayWarnings : fallbackProfile.decay_warnings
	};
};

const tierFromSymbol = (symbol: string | undefined): TandangTierLevel | undefined => {
	if (!symbol) return undefined;
	const filled = [...symbol].filter((ch) => ch === '◆').length;
	const normalized = Math.max(0, Math.min(4, filled));
	return normalized as TandangTierLevel;
};

const tierFromName = (name: string | undefined): TandangTierLevel | undefined => {
	const normalized = name?.trim().toLowerCase();
	if (!normalized) return undefined;
	if (['keystone', 'kunci'].includes(normalized)) return 4;
	if (['pillar', 'pilar'].includes(normalized)) return 3;
	if (['contributor', 'kontributor'].includes(normalized)) return 2;
	if (['novice', 'pemula'].includes(normalized)) return 1;
	if (['shadow', 'bayangan'].includes(normalized)) return 0;
	return undefined;
};

const tierFromValue = (value: unknown): TandangTierLevel | undefined => {
	if (typeof value === 'number' && Number.isFinite(value)) {
		return Math.max(0, Math.min(4, Math.round(value))) as TandangTierLevel;
	}
	if (typeof value === 'string') {
		const parsed = Number(value);
		if (!Number.isNaN(parsed)) {
			return Math.max(0, Math.min(4, Math.round(parsed))) as TandangTierLevel;
		}
		return tierFromName(value);
	}
	return undefined;
};

const readTierLevel = (
	profileData: JsonRecord,
	fallbackTier: TandangTierLevel
): TandangTierLevel => {
	const tier = isRecord(profileData.tier) ? profileData.tier : undefined;
	const reputation = isRecord(profileData.reputation) ? profileData.reputation : undefined;
	const byNumeric =
		tierFromValue(tier?.level) ??
		tierFromValue(reputation?.tier_level) ??
		tierFromValue(reputation?.tier_numeric);
	if (byNumeric !== undefined) {
		return byNumeric;
	}
	const bySymbol = tierFromSymbol(asString(tier?.tier_symbol) ?? asString(reputation?.tier_symbol));
	if (bySymbol !== undefined) {
		return bySymbol;
	}
	const byName = tierFromName(asString(tier?.tier) ?? asString(reputation?.tier));
	if (byName !== undefined) {
		return byName;
	}
	return fallbackTier;
};

const mapActivityItems = (activityValue: unknown): ActivityItem[] | undefined => {
	if (!isRecord(activityValue)) return undefined;
	const timeline = activityValue.timeline;
	if (!Array.isArray(timeline)) return undefined;

	const items = timeline
		.map((item) => {
			if (!isRecord(item)) return undefined;
			const text = asString(item.text) ?? asString(item.activity) ?? asString(item.summary);
			if (!text) return undefined;
			const timestamp = asString(item.timestamp) ?? asString(item.created_at) ?? nowIso();
			return { text, timestamp };
		})
		.filter((item): item is ActivityItem => Boolean(item));

	return items.length > 0 ? items : undefined;
};

interface ApiUserServiceOptions {
	allowMockFallback?: boolean;
}

export class ApiUserService implements UserService {
	private readonly client: ApiClient;
	private readonly fallback: UserService;
	private readonly allowMockFallback: boolean;

	constructor(client: ApiClient, fallback: UserService, options: ApiUserServiceOptions = {}) {
		this.client = client;
		this.fallback = fallback;
		this.allowMockFallback = options.allowMockFallback ?? true;
	}

	private fallbackOrThrow<T>(fallback: () => Promise<T>, error?: unknown): Promise<T> {
		if (this.allowMockFallback) {
			return fallback();
		}
		if (error instanceof Error) {
			throw error;
		}
		throw new Error('Mock fallback disabled for user service');
	}

	private normalizeUserId(userId: string): string {
		const normalized = userId.trim();
		if (!normalized) {
			throw new Error('userId is required');
		}
		return normalized;
	}

	private async currentUserId(): Promise<string | undefined> {
		try {
			const me = await this.client.get<ApiAuthMe>('/auth/me');
			return asString(me.user_id);
		} catch {
			return undefined;
		}
	}

	async getProfile(userId: string): Promise<UserProfile> {
		const normalizedUserId = this.normalizeUserId(userId);
		const currentUserId = await this.currentUserId();
		if (currentUserId === normalizedUserId) {
			return this.getCurrentUser();
		}
		const profileEnvelope = await this.client.get<unknown>(
			`/tandang/users/${encodeURIComponent(normalizedUserId)}/profile`
		);
		return mapUserProfile(profileEnvelope, defaultUserProfile(normalizedUserId));
	}

	async getCurrentUser(): Promise<UserProfile> {
		try {
			const [authMe, tandangProfile] = await Promise.all([
				this.client.get<ApiAuthMe>('/auth/me'),
				this.client.get<unknown>('/tandang/me/profile')
			]);
			const userId = asString(authMe.user_id) ?? 'unknown';
			return mapUserProfile(tandangProfile, defaultUserProfile(userId), authMe);
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.getCurrentUser(), error);
		}
	}

	async getTandangProfile(userId: string): Promise<TandangProfile> {
		const normalizedUserId = this.normalizeUserId(userId);
		const currentUserId = await this.currentUserId();
		if (currentUserId === normalizedUserId) {
			return this.getCurrentTandangProfile();
		}
		const profileEnvelope = await this.client.get<unknown>(
			`/tandang/users/${encodeURIComponent(normalizedUserId)}/profile`
		);
		return mapTandangProfile(profileEnvelope, defaultTandangProfile(normalizedUserId));
	}

	async getCurrentTandangProfile(): Promise<TandangProfile> {
		try {
			const me = await this.client.get<ApiAuthMe>('/auth/me');
			const profileEnvelope = await this.client.get<unknown>('/tandang/me/profile');
			const profileData = normalizeEnvelopeData(profileEnvelope);
			const userId = asString(me.user_id) ?? readPlatformUserId(profileData) ?? 'unknown';

			const [budgetEnvelope, decayEnvelope] = await Promise.all([
				this.client.get<unknown>(`/tandang/users/${userId}/vouch-budget`).catch(() => undefined),
				this.client.get<unknown>(`/tandang/decay/warnings/${userId}`).catch(() => undefined)
			]);
			return mapTandangProfile(
				profileEnvelope,
				defaultTandangProfile(userId),
				me,
				budgetEnvelope,
				decayEnvelope
			);
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.getCurrentTandangProfile(), error);
		}
	}
}

import type { ApiClient } from '$lib/api';
import type { CommunityDashboard, GdfWeather, TandangTierLevel, WeatherType } from '$lib/types';
import type { CommunityService } from '../types';

type JsonRecord = Record<string, unknown>;

const WEATHER_BY_STATUS: Record<string, WeatherType> = {
	cerah: 'cerah',
	berawan: 'berawan',
	hujan: 'hujan',
	badai: 'badai'
};

const WEATHER_EMOJI: Record<WeatherType, string> = {
	cerah: '☀️',
	berawan: '☁️',
	hujan: '🌧️',
	badai: '⛈️'
};

const TIER_NAME_BY_LEVEL: Record<TandangTierLevel, string> = {
	0: 'Bayangan',
	1: 'Pemula',
	2: 'Kontributor',
	3: 'Pilar',
	4: 'Kunci'
};

const TIER_COLOR_BY_LEVEL: Record<TandangTierLevel, string> = {
	0: '#9E9E9E',
	1: '#795548',
	2: '#00695C',
	3: '#1E88E5',
	4: '#FFD700'
};

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value)
		? value
		: typeof value === 'string' && value.trim().length > 0 && Number.isFinite(Number(value))
			? Number(value)
			: undefined;

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const normalizeEnvelopeData = (value: unknown): JsonRecord => {
	if (!isRecord(value)) {
		return {};
	}
	if (isRecord(value.data)) {
		return value.data;
	}
	return value;
};

const readTierLevel = (value: unknown): TandangTierLevel | undefined => {
	const numeric = asNumber(value);
	if (numeric !== undefined) {
		return clamp(Math.round(numeric), 0, 4) as TandangTierLevel;
	}
	const normalized = asString(value)?.toLowerCase();
	if (!normalized) return undefined;
	if (normalized === 'bayangan' || normalized === 'shadow') return 0;
	if (normalized === 'pemula' || normalized === 'novice') return 1;
	if (normalized === 'kontributor' || normalized === 'contributor') return 2;
	if (normalized === 'pilar' || normalized === 'pillar') return 3;
	if (normalized === 'kunci' || normalized === 'keystone') return 4;
	return undefined;
};

const normalizeScore = (value: number | undefined, fallback = 0): number => {
	if (value === undefined) return fallback;
	if (value > 1) return clamp(value / 100, 0, 1);
	return clamp(value, 0, 1);
};

const formatTrendLabel = (rawDate: string | undefined) => {
	if (!rawDate) return '-';
	const date = new Date(rawDate);
	if (Number.isNaN(date.getTime())) {
		return rawDate;
	}
	return new Intl.DateTimeFormat('id-ID', { day: '2-digit', month: '2-digit' }).format(date);
};

const mapWeather = (value: unknown): GdfWeather => {
	const weatherRecord = isRecord(value) ? value : {};
	const status = asString(weatherRecord.status)?.toLowerCase() ?? 'berawan';
	const weather = WEATHER_BY_STATUS[status] ?? 'berawan';
	const reduction = asNumber(weatherRecord.earning_rate_reduction);
	const multiplierRaw = reduction !== undefined ? 1 - reduction : 1;
	const multiplier = clamp(Math.round(multiplierRaw * 100) / 100, 0, 1);

	return {
		weather,
		emoji: asString(weatherRecord.icon) ?? WEATHER_EMOJI[weather],
		multiplier,
		label: asString(weatherRecord.label) ?? weather
	};
};

const mapTierDistribution = (
	pulseData: JsonRecord,
	totalUsers: number
): CommunityDashboard['tier_distribution'] => {
	const charts = isRecord(pulseData.charts) ? pulseData.charts : undefined;
	const chartRows = Array.isArray(charts?.tier_distribution) ? charts.tier_distribution : [];
	const mapped = chartRows
		.map((row) => {
			if (!isRecord(row)) return undefined;
			const tier = readTierLevel(row.label);
			if (tier === undefined) return undefined;
			const count = Math.max(0, Math.round(asNumber(row.value) ?? 0));
			let percentage = asNumber(row.percentage);
			if (percentage === undefined) {
				percentage = totalUsers > 0 ? (count / totalUsers) * 100 : 0;
			}
			if (percentage <= 1) {
				percentage *= 100;
			}
			return {
				tier,
				tier_name: TIER_NAME_BY_LEVEL[tier],
				count,
				percentage: Math.round(clamp(percentage, 0, 100)),
				color: TIER_COLOR_BY_LEVEL[tier]
			};
		})
		.filter((row): row is CommunityDashboard['tier_distribution'][number] => Boolean(row))
		.sort((left, right) => left.tier - right.tier);

	if (mapped.length > 0) {
		return mapped;
	}

	return [
		{
			tier: 0,
			tier_name: TIER_NAME_BY_LEVEL[0],
			count: totalUsers,
			percentage: totalUsers > 0 ? 100 : 0,
			color: TIER_COLOR_BY_LEVEL[0]
		}
	];
};

const mapActiveHighlights = (insightsData: JsonRecord): CommunityDashboard['active_highlights'] => {
	const leaderboard = isRecord(insightsData.leaderboard) ? insightsData.leaderboard : undefined;
	const entries = Array.isArray(leaderboard?.entries) ? leaderboard.entries : [];
	const metricLabel = asString(leaderboard?.metric) ?? 'reputation';

	return entries
		.map((entry) => {
			if (!isRecord(entry)) return undefined;
			const rank = Math.max(1, Math.round(asNumber(entry.rank) ?? 0));
			const name = asString(entry.username) ?? `Warga ${rank}`;
			const tier = readTierLevel(entry.tier) ?? 0;
			const metricValue = asNumber(entry.metric_value);
			const contributions = Math.max(
				0,
				Math.round(metricValue ?? asNumber(entry.verified_solutions) ?? 0)
			);
			const idSeed = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
			return {
				user_id: `lb-${rank}-${idSeed || 'user'}`,
				name,
				tier,
				highlight_reason: `Top ${metricLabel} #${rank}`,
				contributions_this_week: contributions,
				streak_days: 0
			};
		})
		.filter(
			(row): row is CommunityDashboard['active_highlights'][number] => Boolean(row)
		)
		.slice(0, 8);
};

const mapSignalFlowFromTrends = (trendsData: JsonRecord): CommunityDashboard['signal_flow'] => {
	const points = Array.isArray(trendsData.trend_points) ? trendsData.trend_points : [];
	return points
		.slice(-8)
		.map((point) => {
			if (!isRecord(point)) return undefined;
			const activeUsers = Math.max(0, Math.round(asNumber(point.active_users) ?? 0));
			return {
				week_label: formatTrendLabel(asString(point.date)),
				vouch: activeUsers,
				skeptis: 0,
				dukung: 0,
				proof_of_resolve: 0,
				perlu_dicek: 0
			};
		})
		.filter((row): row is CommunityDashboard['signal_flow'][number] => Boolean(row));
};

const mapSignalFlowFromActivityTrend = (
	pulseData: JsonRecord
): CommunityDashboard['signal_flow'] => {
	const charts = isRecord(pulseData.charts) ? pulseData.charts : undefined;
	const points = Array.isArray(charts?.activity_trend) ? charts.activity_trend : [];
	return points
		.map((point) => {
			if (!isRecord(point)) return undefined;
			const label = asString(point.label);
			if (!label) return undefined;
			const value = Math.max(0, Math.round(asNumber(point.value) ?? 0));
			return {
				week_label: label,
				vouch: value,
				skeptis: 0,
				dukung: 0,
				proof_of_resolve: 0,
				perlu_dicek: 0
			};
		})
		.filter((row): row is CommunityDashboard['signal_flow'][number] => Boolean(row));
};

const calculateAverageTier = (tiers: CommunityDashboard['tier_distribution']) => {
	const total = tiers.reduce((sum, tier) => sum + tier.count, 0);
	if (total <= 0) return 0;
	const weighted = tiers.reduce((sum, tier) => sum + tier.tier * tier.count, 0);
	return weighted / total;
};

const mapDashboard = (
	overviewData: JsonRecord,
	insightsData: JsonRecord,
	trendsData: JsonRecord
): CommunityDashboard => {
	const pulseData = Object.keys(insightsData).length > 0 ? insightsData : overviewData;
	const totalUsers = Math.max(
		0,
		Math.round(
			asNumber(pulseData.total_users) ?? asNumber(overviewData.total_users) ?? 0
		)
	);
	const tierDistribution = mapTierDistribution(pulseData, totalUsers);
	const competence = normalizeScore(asNumber(pulseData.average_competence), 0);
	const signalFlow =
		mapSignalFlowFromTrends(trendsData).length > 0
			? mapSignalFlowFromTrends(trendsData)
			: mapSignalFlowFromActivityTrend(pulseData);

	return {
		community_id: asString(pulseData.community_id) ?? 'gotong-royong',
		community_name: asString(pulseData.community_name) ?? 'Komunitas Gotong Royong',
		member_count: totalUsers,
		weather: mapWeather(pulseData.weather ?? overviewData.weather),
		icj_summary: {
			avg_integrity: normalizeScore(
				asNumber(pulseData.avg_integrity) ?? asNumber(pulseData.integrity_avg),
				competence
			),
			avg_competence: competence,
			avg_judgment: normalizeScore(
				asNumber(pulseData.avg_judgment) ?? asNumber(pulseData.judgment_avg),
				competence
			)
		},
		tier_distribution: tierDistribution,
		avg_tier: calculateAverageTier(tierDistribution),
		active_highlights: mapActiveHighlights(insightsData),
		signal_flow: signalFlow
	};
};

export class ApiCommunityService implements CommunityService {
	private readonly client: ApiClient;

	constructor(client: ApiClient) {
		this.client = client;
	}

	async getDashboard(opts?: { period?: string }): Promise<CommunityDashboard> {
		const [overviewEnvelope, insightsEnvelope, trendsEnvelope] = await Promise.all([
			this.client.get<unknown>('/tandang/community/pulse/overview'),
			this.client.get<unknown>('/tandang/community/pulse/insights'),
			this.client.get<unknown>('/tandang/community/pulse/trends', {
				query: {
					period: opts?.period
				}
			})
		]);

		return mapDashboard(
			normalizeEnvelopeData(overviewEnvelope),
			normalizeEnvelopeData(insightsEnvelope),
			normalizeEnvelopeData(trendsEnvelope)
		);
	}
}

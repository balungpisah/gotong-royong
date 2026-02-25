import { describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type { TandangProfile, UserProfile } from '$lib/types';
import type { UserService } from '$lib/services/types';
import { ApiUserService } from '../user-service';

const makeApiClient = () => {
	const get = vi.fn();
	const client = {
		request: vi.fn(),
		get,
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, get };
};

const makeFallbackUserProfile = (): UserProfile => ({
	user_id: 'fallback-user',
	name: 'Fallback User',
	role: 'user',
	tier: 1,
	joined_at: '2025-01-01T00:00:00.000Z',
	stats: {
		witnesses_created: 1,
		witnesses_participated: 2,
		evidence_submitted: 3,
		votes_cast: 4,
		resolutions_completed: 5
	}
});

const makeFallbackTandangProfile = (): TandangProfile => ({
	user_id: 'fallback-user',
	name: 'Fallback User',
	community_id: 'comm-fallback',
	community_name: 'Fallback Community',
	joined_at: '2025-01-01T00:00:00.000Z',
	last_active_at: '2025-01-02T00:00:00.000Z',
	tier: {
		level: 1,
		name: 'Pemula',
		pips: '◆◇◇◇',
		color: '#2E7D32',
		percentile: 10
	},
	scores: {
		integrity: { value: 0.4 },
		competence: { aggregate: 0.4, domains: [] },
		judgment: { value: 0.4, vouch_outcomes_count: 0, dukung_success_rate: null }
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

const makeFallbackService = () => {
	const fallbackUser = makeFallbackUserProfile();
	const fallbackTandang = makeFallbackTandangProfile();
	const service: UserService = {
		getProfile: vi.fn(async () => fallbackUser),
		getCurrentUser: vi.fn(async () => fallbackUser),
		getTandangProfile: vi.fn(async () => fallbackTandang),
		getCurrentTandangProfile: vi.fn(async () => fallbackTandang)
	};
	return { service, fallbackUser, fallbackTandang };
};

describe('ApiUserService', () => {
	it('maps current user from auth + tandang profile envelope', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();

		get.mockImplementation(async (path: string) => {
			if (path === '/auth/me') {
				return { user_id: 'user-123', username: 'Rina', role: 'admin' };
			}
			if (path === '/tandang/me/profile') {
				return {
					data: {
						reputation: {
							user_id: 'markov-user-123',
							tier: 'Contributor'
						},
						tier: {
							tier_symbol: '◆◆◇◇'
						},
						activity: {
							solutions_submitted: 7,
							witnesses_participated: 5,
							evidence_submitted: 3
						},
						cv_hidup: {
							community_id: 'rt-05'
						}
					}
				};
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiUserService(client, fallback);
		const profile = await service.getCurrentUser();

		expect(profile.user_id).toBe('user-123');
		expect(profile.name).toBe('Rina');
		expect(profile.role).toBe('admin');
		expect(profile.tier).toBe(2);
		expect(profile.community_id).toBe('rt-05');
		expect(profile.stats.witnesses_created).toBe(7);
		expect(profile.stats.witnesses_participated).toBe(5);
		expect(profile.stats.evidence_submitted).toBe(3);
	});

	it('merges vouch budget and decay warnings into current tandang profile', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();

		get.mockImplementation(async (path: string) => {
			if (path === '/auth/me') {
				return { user_id: 'user-123', username: 'Rina', role: 'user' };
			}
			if (path === '/tandang/me/profile') {
				return {
					data: {
						reputation: { user_id: 'markov-user-123' },
						tier: { tier: 'Pillar', tier_symbol: '◆◆◆◇' },
						activity: { last_active_at: '2026-02-25T00:00:00.000Z' },
						cv_hidup: { community_id: 'rt-05', community_name: 'RT 05' }
					}
				};
			}
			if (path === '/tandang/users/user-123/vouch-budget') {
				return {
					data: {
						max_vouches: 15,
						active_vouches: 12,
						remaining: 3
					}
				};
			}
			if (path === '/tandang/decay/warnings/user-123') {
				return {
					data: {
						warnings: [{ domain: 'Verifikasi', days_until_decay: 5 }]
					}
				};
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiUserService(client, fallback);
		const profile = await service.getCurrentTandangProfile();

		expect(profile.user_id).toBe('user-123');
		expect(profile.name).toBe('Rina');
		expect(profile.tier.level).toBe(3);
		expect(profile.community_id).toBe('rt-05');
		expect(profile.vouch_budget).toEqual({
			max_vouches: 15,
			active_vouches: 12,
			remaining: 3
		});
		expect(profile.decay_warnings).toEqual([{ domain: 'Verifikasi', days_until_decay: 5 }]);
	});

	it('loads non-self user profile from backend endpoint', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		get.mockImplementation(async (path: string) => {
			if (path === '/auth/me') {
				return { user_id: 'user-123', username: 'Rina', role: 'user' };
			}
			if (path === '/tandang/users/user-999/profile') {
				return {
					data: {
						platform_user_id: 'user-999',
						identity: 'gotong_royong:user-999',
						reputation: {
							username: 'Budi'
						},
						tier: {
							tier_symbol: '◆◇◇◇'
						},
						activity: {
							witnesses_created: 3
						},
						cv_hidup: {
							community_id: 'rt-09',
							joined_at: '2026-02-01T00:00:00.000Z'
						}
					}
				};
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiUserService(client, fallback);
		const profile = await service.getProfile('user-999');

		expect(profile.user_id).toBe('user-999');
		expect(profile.name).toBe('Budi');
		expect(profile.role).toBe('user');
		expect(profile.tier).toBe(1);
		expect(profile.community_id).toBe('rt-09');
		expect(profile.stats.witnesses_created).toBe(3);
		expect(fallback.getProfile).not.toHaveBeenCalled();
	});

	it('loads non-self tandang profile from backend endpoint without mock fallback', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		get.mockImplementation(async (path: string) => {
			if (path === '/auth/me') {
				return { user_id: 'user-123', username: 'Rina', role: 'user' };
			}
			if (path === '/tandang/users/user-999/profile') {
				return {
					data: {
						platform_user_id: 'user-999',
						identity: 'gotong_royong:user-999',
						reputation: { username: 'Budi', percentile: 44 },
						tier: { tier_symbol: '◆◆◇◇' },
						activity: { last_active_at: '2026-02-25T00:00:00.000Z' },
						cv_hidup: { community_id: 'rw-09', community_name: 'RW 09' }
					}
				};
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiUserService(client, fallback);
		const profile = await service.getTandangProfile('user-999');

		expect(profile.user_id).toBe('user-999');
		expect(profile.name).toBe('Budi');
		expect(profile.tier.level).toBe(2);
		expect(profile.tier.percentile).toBe(44);
		expect(profile.community_id).toBe('rw-09');
		expect(profile.community_name).toBe('RW 09');
		expect(fallback.getTandangProfile).not.toHaveBeenCalled();
	});

	it('falls back when backend calls fail', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback, fallbackUser, fallbackTandang } = makeFallbackService();
		get.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiUserService(client, fallback);
		await expect(service.getCurrentUser()).resolves.toEqual(fallbackUser);
		await expect(service.getCurrentTandangProfile()).resolves.toEqual(fallbackTandang);
	});

	it('does not use mock fallback when disabled', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		get.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiUserService(client, fallback, { allowMockFallback: false });
		await expect(service.getCurrentUser()).rejects.toThrow('backend unavailable');
		expect(fallback.getCurrentUser).not.toHaveBeenCalled();
	});
});

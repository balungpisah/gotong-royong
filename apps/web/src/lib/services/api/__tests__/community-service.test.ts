import { describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import { ApiCommunityService } from '../community-service';

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

describe('ApiCommunityService', () => {
	it('maps overview+insights+trends into CommunityDashboard', async () => {
		const { client, get } = makeApiClient();
		get.mockImplementation(async (path: string) => {
			if (path === '/tandang/community/pulse/overview') {
				return {
					data: {
						total_users: 120,
						average_competence: 0.62,
						charts: {
							tier_distribution: [
								{ label: 'Novice', value: 40, percentage: 33.3 },
								{ label: 'Contributor', value: 50, percentage: 41.7 },
								{ label: 'Pillar', value: 20, percentage: 16.7 },
								{ label: 'Keystone', value: 10, percentage: 8.3 }
							]
						},
						weather: {
							status: 'cerah',
							icon: '☀️',
							label: 'Cerah',
							earning_rate_reduction: 0.05
						}
					}
				};
			}
			if (path === '/tandang/community/pulse/insights') {
				return {
					data: {
						community_id: 'comm-rw-01',
						community_name: 'RW 01 Melati',
						total_users: 120,
						average_competence: 0.62,
						charts: {
							tier_distribution: [
								{ label: 'Novice', value: 40, percentage: 33.3 },
								{ label: 'Contributor', value: 50, percentage: 41.7 },
								{ label: 'Pillar', value: 20, percentage: 16.7 },
								{ label: 'Keystone', value: 10, percentage: 8.3 }
							]
						},
						leaderboard: {
							metric: 'reputation',
							entries: [
								{
									rank: 1,
									username: 'Sari Dewi',
									tier: 'Pillar',
									metric_value: 12.4,
									verified_solutions: 7
								}
							]
						},
						weather: {
							status: 'cerah',
							icon: '☀️',
							label: 'Cerah',
							earning_rate_reduction: 0.05
						}
					}
				};
			}
			if (path === '/tandang/community/pulse/trends') {
				return {
					data: {
						trend_points: [
							{ date: '2026-02-20', active_users: 18 },
							{ date: '2026-02-21', active_users: 22 }
						]
					}
				};
			}
			throw new Error(`unexpected path ${path}`);
		});

		const service = new ApiCommunityService(client);
		const dashboard = await service.getDashboard({ period: '30d' });

		expect(get).toHaveBeenCalledWith('/tandang/community/pulse/trends', {
			query: { period: '30d' }
		});
		expect(dashboard.community_id).toBe('comm-rw-01');
		expect(dashboard.community_name).toBe('RW 01 Melati');
		expect(dashboard.member_count).toBe(120);
		expect(dashboard.weather).toMatchObject({
			weather: 'cerah',
			emoji: '☀️',
			label: 'Cerah',
			multiplier: 0.95
		});
		expect(dashboard.tier_distribution).toHaveLength(4);
		expect(dashboard.active_highlights[0]).toMatchObject({
			name: 'Sari Dewi',
			tier: 3
		});
		expect(dashboard.signal_flow).toHaveLength(2);
		expect(dashboard.signal_flow[1]).toMatchObject({
			vouch: 22,
			skeptis: 0,
			dukung: 0
		});
	});

	it('returns safe defaults when pulse payload is sparse', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValue({});

		const service = new ApiCommunityService(client);
		const dashboard = await service.getDashboard();

		expect(dashboard.community_id).toBe('gotong-royong');
		expect(dashboard.community_name).toBe('Komunitas Gotong Royong');
		expect(dashboard.member_count).toBe(0);
		expect(dashboard.tier_distribution[0]).toMatchObject({
			tier: 0,
			count: 0
		});
		expect(dashboard.active_highlights).toEqual([]);
	});
});

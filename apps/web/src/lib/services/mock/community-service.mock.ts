import { mockCommunityDashboard } from '$lib/fixtures';
import type { CommunityDashboard } from '$lib/types';
import type { CommunityService } from '../types';

const cloneDashboard = (value: CommunityDashboard): CommunityDashboard => ({
	...value,
	weather: { ...value.weather },
	icj_summary: { ...value.icj_summary },
	tier_distribution: value.tier_distribution.map((tier) => ({ ...tier })),
	active_highlights: value.active_highlights.map((member) => ({ ...member })),
	signal_flow: value.signal_flow.map((point) => ({ ...point }))
});

export class MockCommunityService implements CommunityService {
	async getDashboard(): Promise<CommunityDashboard> {
		return cloneDashboard(mockCommunityDashboard);
	}
}

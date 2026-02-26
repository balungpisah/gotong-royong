import type { FeedService, Paginated } from '../types';
import type { FeedStreamItem, FollowableEntity } from '$lib/types';
import { mockFeedItems, mockSuggestedEntities } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockFeedService implements FeedService {
	private readonly feedItems: FeedStreamItem[] = mockFeedItems.map((item) => ({
		stream_id: `w-${item.witness_id}`,
		sort_timestamp: item.latest_event.timestamp,
		kind: 'witness',
		data: { ...item }
	}));
	private readonly suggestions = mockSuggestedEntities.map((item) => ({ ...item }));

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<FeedStreamItem>> {
		await delay();
		const limit = opts?.limit ?? 20;
		return {
			items: this.feedItems.slice(0, limit),
			total: this.feedItems.length
		};
	}

	async listSuggestions(): Promise<FollowableEntity[]> {
		await delay(120);
		return this.suggestions.map((item) => ({ ...item }));
	}

	async setMonitorPreference(witnessId: string, monitored: boolean): Promise<void> {
		await delay(80);
		for (const item of this.feedItems) {
			if (item.kind === 'witness' && item.data.witness_id === witnessId) {
				item.data.monitored = monitored;
			}
		}
	}

	async setEntityFollowPreference(entityId: string, followed: boolean): Promise<void> {
		await delay(80);
		for (const suggestion of this.suggestions) {
			if (suggestion.entity_id === entityId) {
				suggestion.followed = followed;
			}
		}
		for (const item of this.feedItems) {
			if (item.kind !== 'witness') continue;
			item.data.entity_tags = item.data.entity_tags.map((tag) =>
				tag.entity_id === entityId ? { ...tag, followed } : tag
			);
		}
	}
}

import type { FeedService, Paginated } from '../types';
import type { FeedItem, FollowableEntity } from '$lib/types';
import { mockFeedItems, mockSuggestedEntities } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockFeedService implements FeedService {
	private readonly feedItems = mockFeedItems.map((item) => ({ ...item }));
	private readonly suggestions = mockSuggestedEntities.map((item) => ({ ...item }));

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<FeedItem>> {
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
			if (item.witness_id === witnessId) {
				item.monitored = monitored;
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
			item.entity_tags = item.entity_tags.map((tag) =>
				tag.entity_id === entityId ? { ...tag, followed } : tag
			);
		}
	}
}

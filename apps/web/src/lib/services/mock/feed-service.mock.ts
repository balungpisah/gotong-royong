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
}

/**
 * Feed Store — manages the Pulse event-based feed state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Currently mock-backed — will be swapped to a FeedService when backend is ready.
 */

import type { FeedItem, FeedFilter, FollowableEntity } from '$lib/types';
import { mockFeedItems, mockSuggestedEntities } from '$lib/fixtures';

export class FeedStore {
	// ---------------------------------------------------------------------------
	// Feed state
	// ---------------------------------------------------------------------------

	items = $state<FeedItem[]>([]);
	filter = $state<FeedFilter>('semua');
	loading = $state(false);
	error = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Suggestion state (onboarding)
	// ---------------------------------------------------------------------------

	suggestedEntities = $state<FollowableEntity[]>([]);
	suggestionsLoading = $state(false);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	filteredItems = $derived(
		this.filter === 'semua'
			? this.items
			: this.items.filter((i) => i.source === this.filter)
	);

	/** Total number of items across all filters. */
	totalCount = $derived(this.items.length);

	/** Whether there are suggested entities to show (onboarding). */
	hasSuggestions = $derived(this.suggestedEntities.length > 0);

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/**
	 * Load the feed. Currently returns mock data.
	 * TODO: Replace with FeedService.list() when backend is ready.
	 */
	async loadFeed() {
		this.loading = true;
		this.error = null;
		try {
			// Simulate network delay
			await new Promise((resolve) => setTimeout(resolve, 400));
			this.items = [...mockFeedItems];
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat feed';
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Load suggested entities for onboarding.
	 * TODO: Replace with FeedService.getSuggestions() when backend is ready.
	 */
	async loadSuggestions() {
		this.suggestionsLoading = true;
		try {
			await new Promise((resolve) => setTimeout(resolve, 200));
			this.suggestedEntities = [...mockSuggestedEntities];
		} catch {
			// Suggestions are non-critical, silently fail
			this.suggestedEntities = [];
		} finally {
			this.suggestionsLoading = false;
		}
	}

	/** Set the active feed filter tab. */
	setFilter(f: FeedFilter) {
		this.filter = f;
	}

	/**
	 * Toggle follow state for an entity.
	 * Updates both feed item entity_tags and suggested entities.
	 * TODO: Replace with API call when backend is ready.
	 */
	async toggleFollow(entityId: string) {
		// Toggle in suggested entities
		this.suggestedEntities = this.suggestedEntities.map((e) =>
			e.entity_id === entityId ? { ...e, followed: !e.followed } : e
		);

		// Toggle in feed item entity tags
		this.items = this.items.map((item) => ({
			...item,
			entity_tags: item.entity_tags.map((tag) =>
				tag.entity_id === entityId ? { ...tag, followed: !tag.followed } : tag
			)
		}));
	}

	/**
	 * Follow all suggested entities at once (onboarding bulk action).
	 */
	async followAllSuggested() {
		this.suggestedEntities = this.suggestedEntities.map((e) => ({
			...e,
			followed: true
		}));

		// Also update entity tags in feed items
		const suggestedIds = new Set(this.suggestedEntities.map((e) => e.entity_id));
		this.items = this.items.map((item) => ({
			...item,
			entity_tags: item.entity_tags.map((tag) =>
				suggestedIds.has(tag.entity_id) ? { ...tag, followed: true } : tag
			)
		}));
	}
}

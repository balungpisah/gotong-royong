/**
 * Feed Store — manages the Pulse event-based feed state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Feed list/suggestions are backed by FeedService (API-first in service factory).
 *
 * The feed is now a **polymorphic stream** — it can contain both witness activity
 * cards and inline system cards (suggestions, tips, milestones, prompts).
 */

import type {
	FeedItem,
	FeedFilter,
	FeedStreamItem,
	FeedWitnessItem,
	SystemCardData,
	FollowableEntity
} from '$lib/types';
import type { FeedService } from '$lib/services/types';
import { shouldAutoMonitor } from '$lib/types/feed';
import { mockSystemCards } from '$lib/fixtures';

/** How often to inject a system card into the stream (every Nth witness item). */
const SYSTEM_CARD_INTERVAL = 3;

export class FeedStore {
	private readonly service: FeedService;

	constructor(service: FeedService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Feed state
	// ---------------------------------------------------------------------------

	items = $state<FeedItem[]>([]);
	systemCards = $state<SystemCardData[]>([]);
	filter = $state<FeedFilter>('semua');
	loading = $state(false);
	error = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Dismiss state
	// ---------------------------------------------------------------------------

	dismissed = $state<Set<string>>(new Set());

	// ---------------------------------------------------------------------------
	// Suggestion state (onboarding)
	// ---------------------------------------------------------------------------

	suggestedEntities = $state<FollowableEntity[]>([]);
	suggestionsLoading = $state(false);

	// ---------------------------------------------------------------------------
	// Derived — filtered witness items
	// ---------------------------------------------------------------------------

	filteredItems = $derived(
		this.filter === 'semua' || this.filter === 'discover'
			? this.items
			: this.filter === 'pantauan'
				? this.items.filter((i) => i.monitored)
				: this.items.filter((i) => i.source === this.filter)
	);

	/** Number of monitored witnesses. */
	monitoredCount = $derived(this.items.filter((i) => i.monitored).length);

	// ---------------------------------------------------------------------------
	// Derived — polymorphic stream (witness cards + system cards interleaved)
	// ---------------------------------------------------------------------------

	/**
	 * Assembles the polymorphic feed stream:
	 * - Wraps each FeedItem as FeedWitnessItem
	 * - Injects system cards at intervals (only in 'semua' filter)
	 * - Filters out dismissed system cards
	 */
	filteredStream = $derived.by((): FeedStreamItem[] => {
		const witnessItems: FeedWitnessItem[] = this.filteredItems.map((item) => ({
			stream_id: `w-${item.witness_id}`,
			sort_timestamp: item.latest_event.timestamp,
			kind: 'witness' as const,
			data: item
		}));

		// System cards only appear in 'semua' tab
		if (this.filter !== 'semua') {
			return witnessItems;
		}

		// Interleave system cards (non-dismissed) into the stream
		const availableCards = this.systemCards.filter(
			(_, i) => !this.dismissed.has(`sys-${i}`)
		);

		const stream: FeedStreamItem[] = [];
		let cardIndex = 0;

		for (let i = 0; i < witnessItems.length; i++) {
			stream.push(witnessItems[i]);

			// Inject a system card after every Nth witness item
			if (
				(i + 1) % SYSTEM_CARD_INTERVAL === 0 &&
				cardIndex < availableCards.length
			) {
				const card = availableCards[cardIndex];
				const originalIndex = this.systemCards.indexOf(card);
				stream.push({
					stream_id: `sys-${originalIndex}`,
					sort_timestamp: witnessItems[i].sort_timestamp,
					kind: 'system' as const,
					data: card
				});
				cardIndex++;
			}
		}

		return stream;
	});

	/** Total number of items across all filters. */
	totalCount = $derived(this.items.length);

	/** Whether there are suggested entities to show (onboarding). */
	hasSuggestions = $derived(this.suggestedEntities.length > 0);

	/** Whether the discover tab is active. */
	isDiscoverActive = $derived(this.filter === 'discover');

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/**
	 * Load the feed via FeedService.
	 */
	async loadFeed() {
		this.loading = true;
		this.error = null;
		try {
			const result = await this.service.list({ limit: 50 });
			this.items = result.items;
			this.systemCards = [...mockSystemCards];
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat feed';
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Load suggested entities for onboarding.
	 */
	async loadSuggestions() {
		this.suggestionsLoading = true;
		try {
			this.suggestedEntities = await this.service.listSuggestions();
		} catch {
			this.suggestedEntities = [];
		} finally {
			this.suggestionsLoading = false;
		}
	}

	/** Prepend a new witness feed item (e.g. after witness creation). */
	prependWitnessItem(feedItem: FeedItem) {
		this.items = [feedItem, ...this.items];
	}

	/** Set the active feed filter tab. */
	setFilter(f: FeedFilter) {
		this.filter = f;
	}

	/**
	 * Toggle monitor (pantau) state for a witness.
	 */
	async toggleMonitor(witnessId: string) {
		this.error = null;
		const previousItems = this.items;
		let nextValue = false;
		this.items = this.items.map((item) => {
			if (item.witness_id !== witnessId) return item;
			nextValue = !item.monitored;
			return { ...item, monitored: nextValue };
		});
		try {
			await this.service.setMonitorPreference(witnessId, nextValue);
		} catch (err) {
			this.items = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status pantau';
		}
	}

	/**
	 * Auto-monitor a witness after an engagement action.
	 *
	 * Call this after vouch, witness, flag, vote, or evidence actions.
	 * Uses the shouldAutoMonitor() contract to determine eligibility.
	 * Only sets monitored=true (never removes — that's the user's choice).
	 *
	 */
	autoMonitorOnAction(witnessId: string, updatedRelation: Partial<import('$lib/types').MyRelation>) {
		let shouldPersistMonitor = false;
		this.items = this.items.map((item) => {
			if (item.witness_id !== witnessId) return item;

			// Merge the new relation fields with existing
			const baseRelation: import('$lib/types').MyRelation = item.my_relation ?? {
				vouched: false,
				witnessed: false,
				flagged: false,
				supported: false
			};
			const merged = { ...baseRelation, ...updatedRelation } as import('$lib/types').MyRelation;

			// Check if this action qualifies for auto-pantau
			const shouldMonitor = shouldAutoMonitor(merged);

			return {
				...item,
				my_relation: merged,
				// Only auto-set to true, never auto-remove
				monitored: (() => {
					const nextMonitored = item.monitored || shouldMonitor;
					if (!item.monitored && nextMonitored) {
						shouldPersistMonitor = true;
					}
					return nextMonitored;
				})()
			};
		});
		if (shouldPersistMonitor) {
			void this.service.setMonitorPreference(witnessId, true).catch(() => undefined);
		}
	}

	/**
	 * Toggle "dukung" (support) state for a witness.
	 * Optimistic client-local toggle outside Tandang signal pipeline.
	 */
	toggleDukung(witnessId: string) {
		this.items = this.items.map((item) => {
			if (item.witness_id !== witnessId) return item;

			const wasSupported = item.my_relation?.supported ?? false;
			const currentCount = item.signal_counts?.dukung_count ?? 0;

			return {
				...item,
				my_relation: {
					...item.my_relation,
					vouched: item.my_relation?.vouched ?? false,
					witnessed: item.my_relation?.witnessed ?? false,
					flagged: item.my_relation?.flagged ?? false,
					supported: !wasSupported
				},
				signal_counts: item.signal_counts
					? {
							...item.signal_counts,
							dukung_count: wasSupported
								? Math.max(0, currentCount - 1)
								: currentCount + 1
						}
					: undefined
			};
		});
	}

	/** Dismiss a system card so it doesn't appear again. */
	dismissCard(streamId: string) {
		this.dismissed = new Set([...this.dismissed, streamId]);
	}

	/**
	 * Toggle follow state for an entity.
	 * Updates both feed item entity_tags and suggested entities.
	 */
	async toggleFollow(entityId: string) {
		this.error = null;
		const previousSuggestions = this.suggestedEntities;
		const previousItems = this.items;
		const currentFollowed =
			this.suggestedEntities.find((item) => item.entity_id === entityId)?.followed ??
			this.items
				.flatMap((item) => item.entity_tags)
				.find((tag) => tag.entity_id === entityId)
				?.followed ??
			false;
		const nextFollowed = !currentFollowed;

		// Toggle in suggested entities
		this.suggestedEntities = this.suggestedEntities.map((e) => {
			if (e.entity_id !== entityId) return e;
			return { ...e, followed: nextFollowed };
		});

		// Toggle in feed item entity tags
		this.items = this.items.map((item) => ({
			...item,
			entity_tags: item.entity_tags.map((tag) =>
				tag.entity_id === entityId ? { ...tag, followed: !tag.followed } : tag
			)
		}));
		try {
			await this.service.setEntityFollowPreference(entityId, nextFollowed);
		} catch (err) {
			this.suggestedEntities = previousSuggestions;
			this.items = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status ikuti';
		}
	}

	/**
	 * Follow all suggested entities at once (onboarding bulk action).
	 */
	async followAllSuggested() {
		this.error = null;
		const previousSuggestions = this.suggestedEntities;
		const previousItems = this.items;

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
		try {
			await Promise.all(
				[...suggestedIds].map((entityId) =>
					this.service.setEntityFollowPreference(entityId, true)
				)
			);
		} catch (err) {
			this.suggestedEntities = previousSuggestions;
			this.items = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status ikuti';
		}
	}
}

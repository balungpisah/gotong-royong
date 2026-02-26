/**
 * Feed Store — manages the Pulse event-based feed state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Feed list/suggestions are backed by FeedService (API-first in service factory).
 */

import type {
	FeedFilter,
	FeedItem,
	FeedStreamItem,
	FeedSystemItem,
	FeedWitnessItem,
	FollowableEntity,
	MyRelation,
	SystemCardData
} from '$lib/types';
import type { FeedService } from '$lib/services/types';
import { shouldAutoMonitor } from '$lib/types/feed';

const isSuggestionSystemItem = (
	item: FeedStreamItem
): item is FeedSystemItem & {
	data: SystemCardData & {
		payload: {
			variant: 'suggestion';
			entities: FollowableEntity[];
		};
	};
} => item.kind === 'system' && item.data.payload.variant === 'suggestion';

export class FeedStore {
	private readonly service: FeedService;

	constructor(service: FeedService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Feed state
	// ---------------------------------------------------------------------------

	streamItems = $state<FeedStreamItem[]>([]);
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
	// Derived — witness items and filtered witness items
	// ---------------------------------------------------------------------------

	items = $derived.by<FeedItem[]>(() =>
		this.streamItems
			.filter((item): item is FeedWitnessItem => item.kind === 'witness')
			.map((item) => item.data)
	);

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
	// Derived — polymorphic stream (server-provided stream + client filtering)
	// ---------------------------------------------------------------------------

	filteredStream = $derived.by((): FeedStreamItem[] => {
		if (this.filter === 'semua') {
			return this.streamItems.filter(
				(item) => item.kind !== 'system' || !this.dismissed.has(item.stream_id)
			);
		}

		const allowedWitnessIds = new Set(this.filteredItems.map((item) => item.witness_id));
		return this.streamItems.filter(
			(item): item is FeedWitnessItem =>
				item.kind === 'witness' && allowedWitnessIds.has(item.data.witness_id)
		);
	});

	/** Total number of witness items across all filters. */
	totalCount = $derived(this.items.length);

	/** Whether there are suggested entities to show (onboarding). */
	hasSuggestions = $derived(this.suggestedEntities.length > 0);

	/** Whether the discover tab is active. */
	isDiscoverActive = $derived(this.filter === 'discover');

	private updateWitnesses(updater: (item: FeedItem) => FeedItem) {
		this.streamItems = this.streamItems.map((item) => {
			if (item.kind !== 'witness') return item;
			return {
				...item,
				data: updater(item.data)
			};
		});
	}

	private updateEntityFollowAcrossStream(entityId: string, followed: boolean) {
		this.streamItems = this.streamItems.map((item) => {
			if (item.kind === 'witness') {
				return {
					...item,
					data: {
						...item.data,
						entity_tags: item.data.entity_tags.map((tag) =>
							tag.entity_id === entityId ? { ...tag, followed } : tag
						)
					}
				};
			}

			if (isSuggestionSystemItem(item)) {
				return {
					...item,
					data: {
						...item.data,
						payload: {
							...item.data.payload,
							entities: item.data.payload.entities.map((entity) =>
								entity.entity_id === entityId ? { ...entity, followed } : entity
							)
						}
					}
				};
			}

			return item;
		});
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/** Load the feed via FeedService. */
	async loadFeed() {
		this.loading = true;
		this.error = null;
		try {
			const result = await this.service.list({ limit: 50 });
			this.streamItems = result.items;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat feed';
		} finally {
			this.loading = false;
		}
	}

	/** Load suggested entities for onboarding. */
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
		this.streamItems = [
			{
				stream_id: `w-${feedItem.witness_id}-${Date.now()}`,
				sort_timestamp: feedItem.latest_event.timestamp,
				kind: 'witness',
				data: feedItem
			},
			...this.streamItems
		];
	}

	/** Set the active feed filter tab. */
	setFilter(f: FeedFilter) {
		this.filter = f;
	}

	/** Toggle monitor (pantau) state for a witness. */
	async toggleMonitor(witnessId: string) {
		this.error = null;
		const previousItems = this.streamItems;
		let nextValue: boolean | null = null;
		this.updateWitnesses((item) => {
			if (item.witness_id !== witnessId) return item;
			nextValue = !item.monitored;
			return { ...item, monitored: nextValue };
		});
		if (nextValue === null) {
			return;
		}
		try {
			await this.service.setMonitorPreference(witnessId, nextValue);
		} catch (err) {
			this.streamItems = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status pantau';
		}
	}

	/**
	 * Auto-monitor a witness after an engagement action.
	 *
	 * Call this after vouch, witness, flag, vote, or evidence actions.
	 * Uses the shouldAutoMonitor() contract to determine eligibility.
	 * Only sets monitored=true (never removes — that's the user's choice).
	 */
	autoMonitorOnAction(witnessId: string, updatedRelation: Partial<MyRelation>) {
		let shouldPersistMonitor = false;
		this.updateWitnesses((item) => {
			if (item.witness_id !== witnessId) return item;

			const baseRelation: MyRelation = item.my_relation ?? {
				vouched: false,
				witnessed: false,
				flagged: false,
				supported: false
			};
			const merged = { ...baseRelation, ...updatedRelation } as MyRelation;
			const shouldMonitor = shouldAutoMonitor(merged);
			const nextMonitored = item.monitored || shouldMonitor;
			if (!item.monitored && nextMonitored) {
				shouldPersistMonitor = true;
			}
			return {
				...item,
				my_relation: merged,
				monitored: nextMonitored
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
		this.updateWitnesses((item) => {
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
	 * Updates feed witness tags, feed suggestion cards, and onboarding suggestions.
	 */
	async toggleFollow(entityId: string) {
		this.error = null;
		const previousSuggestions = this.suggestedEntities;
		const previousItems = this.streamItems;
		const currentFollowed =
			this.suggestedEntities.find((item) => item.entity_id === entityId)?.followed ??
			this.items
				.flatMap((item) => item.entity_tags)
				.find((tag) => tag.entity_id === entityId)
				?.followed ??
			this.streamItems
				.filter((item) => isSuggestionSystemItem(item))
				.flatMap((item) => item.data.payload.entities)
				.find((entity) => entity.entity_id === entityId)
				?.followed ??
			false;
		const nextFollowed = !currentFollowed;

		this.suggestedEntities = this.suggestedEntities.map((e) =>
			e.entity_id === entityId ? { ...e, followed: nextFollowed } : e
		);
		this.updateEntityFollowAcrossStream(entityId, nextFollowed);

		try {
			await this.service.setEntityFollowPreference(entityId, nextFollowed);
		} catch (err) {
			this.suggestedEntities = previousSuggestions;
			this.streamItems = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status ikuti';
		}
	}

	/** Follow all suggested entities at once (onboarding bulk action). */
	async followAllSuggested() {
		this.error = null;
		const previousSuggestions = this.suggestedEntities;
		const previousItems = this.streamItems;

		this.suggestedEntities = this.suggestedEntities.map((e) => ({
			...e,
			followed: true
		}));

		const suggestedIds = new Set(this.suggestedEntities.map((e) => e.entity_id));
		for (const entityId of suggestedIds) {
			this.updateEntityFollowAcrossStream(entityId, true);
		}

		try {
			await Promise.all(
				[...suggestedIds].map((entityId) =>
					this.service.setEntityFollowPreference(entityId, true)
				)
			);
		} catch (err) {
			this.suggestedEntities = previousSuggestions;
			this.streamItems = previousItems;
			this.error = err instanceof Error ? err.message : 'Gagal menyimpan status ikuti';
		}
	}
}

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getWitnessStore, getNotificationStore, getFeedStore } from '$lib/stores';
	import { ChatInput } from '$lib/components/shell';
	import {
		PulseActivityCard,
		WitnessDetailPanel,
		FeedEventCard,
		FeedSystemCard,
		DiscoverView,
		ContextBox
	} from '$lib/components/pulse';
	import Activity from '@lucide/svelte/icons/activity';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import Masonry from 'svelte-bricks';
	import { shareFeedItem } from '$lib/utils/share';
	import { Button } from '$lib/components/ui/button';

	const witnessStore = getWitnessStore();
	const notificationStore = getNotificationStore();
	const feedStore = getFeedStore();

	// Load data on mount
	$effect(() => {
		witnessStore.loadList();
		notificationStore.loadNotifications();
		feedStore.loadFeed();
		feedStore.loadSuggestions();
	});

	async function retryFeedLoad() {
		await feedStore.loadFeed();
		await feedStore.loadSuggestions();
	}

	async function retryWitnessListLoad() {
		await witnessStore.loadList();
	}

	async function retryWitnessDetailLoad() {
		if (!selectedWitnessId) return;
		await witnessStore.loadDetail(selectedWitnessId);
	}

	const sortedWitnesses = $derived(
		[...witnessStore.witnesses].sort(
			(a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
		)
	);

	// ---------------------------------------------------------------------------
	// Master-detail state
	// ---------------------------------------------------------------------------

	let selectedWitnessId = $state<string | null>(null);
	let messageSending = $state(false);

	// Future: profile state for polymorphic context box
	let selectedUserId = $state<string | null>(null);

	const isDetailOpen = $derived(selectedWitnessId !== null && witnessStore.current !== null);
	const isDetailLoading = $derived(selectedWitnessId !== null && witnessStore.detailLoading);
	const showDetail = $derived(isDetailOpen || isDetailLoading);

	// Context box active signal: external triggers that should open it
	const contextActive = $derived(showDetail || selectedUserId !== null);

	// Suppress masonry FLIP animation when context box opens/closes
	// to prevent card size flash during column reflow.
	// On subsequent clicks (context already open), FLIP stays enabled for smooth reorder.
	let masonryAnimate = $state(true);
	$effect(() => {
		const _ = contextActive; // track open/close transitions
		masonryAnimate = false;
		const timer = setTimeout(() => { masonryAnimate = true; }, 50);
		return () => clearTimeout(timer);
	});

	// Scroll selected card to top of viewport after masonry reflow settles.
	// Uses getBoundingClientRect for accurate position inside masonry's
	// absolute/transformed layout, with manual header offset.
	const HEADER_OFFSET = 80; // px — clears sticky header + breathing room
	$effect(() => {
		if (!selectedWitnessId) return;
		const timer = setTimeout(() => {
			const card = document.querySelector(`[data-witness-id="${selectedWitnessId}"]`);
			if (!card) return;
			const rect = card.getBoundingClientRect();
			const targetY = window.scrollY + rect.top - HEADER_OFFSET;
			window.scrollTo({ top: Math.max(0, targetY), behavior: 'smooth' });
		}, 500);
		return () => clearTimeout(timer);
	});

	// The selected feed item — used for pinned card header in detail panel
	const selectedFeedItem = $derived.by(() => {
		if (!selectedWitnessId) return null;
		const match = feedStore.filteredStream
			.filter((s): s is import('$lib/types').FeedStreamItem & { kind: 'witness' } => s.kind === 'witness')
			.find((s) => s.data.witness_id === selectedWitnessId);
		return match?.data ?? null;
	});

	function selectWitness(witnessId: string) {
		if (selectedWitnessId === witnessId) {
			closeDetail();
			return;
		}
		selectedWitnessId = witnessId;
		witnessStore.loadDetail(witnessId);
	}

	function closeDetail() {
		selectedWitnessId = null;
		witnessStore.current = null;
		selectedUserId = null;
	}

	async function handleSendMessage(content: string, attachments?: File[]) {
		messageSending = true;
		try {
			await witnessStore.sendMessage(content, attachments);
		} finally {
			messageSending = false;
		}
	}

	function handleWitnessCreated(witnessId: string) {
		selectedWitnessId = witnessId;
		// witnessStore.current is already set by createWitness()
		// selectedFeedItem derived will auto-find the new feed item
	}

	// ---------------------------------------------------------------------------
	// Masonry skeleton items (variable heights for visual preview)
	// ---------------------------------------------------------------------------
	// ---------------------------------------------------------------------------
	// Masonry stream with triage card prepended
	// ---------------------------------------------------------------------------

	type TriageItem = { stream_id: string; kind: 'triage'; sort_timestamp: string };
	type MasonryItem = import('$lib/types').FeedStreamItem | TriageItem;

	const triageEntry: TriageItem = {
		stream_id: '__triage__',
		kind: 'triage',
		sort_timestamp: '9999-12-31T00:00:00Z' // always sorts first
	};

	// Keep natural feed order — the dark border on the selected card
	// identifies it without reordering and disrupting scroll position.
	const masonryStream = $derived.by<MasonryItem[]>(() => {
		if (feedStore.filter !== 'semua') {
			return feedStore.filteredStream;
		}
		return [triageEntry, ...feedStore.filteredStream];
	});

	const skeletonItems = [
		{ id: 1, h: 260 },
		{ id: 2, h: 320 },
		{ id: 3, h: 240 },
		{ id: 4, h: 340 },
		{ id: 5, h: 280 },
		{ id: 6, h: 250 }
	];
</script>

<!--
	3-Column Workspace Layout (desktop ≥lg):
	  SIDEBAR (60px, in layout) — collapsible app nav
	  FEED    (flex-1)          — masonry feed, scrolls with page
	  CONTEXT (50%)             — ContextBox, sticky, tabbed

	ContextBox tabs:
	  📋 Laporan   — witness detail (WitnessDetailPanel)
	  👤 Profil    — person profile (SelfProfile)
	  🏘 Komunitas — community pulse dashboard (CommunityPulse)

	On mobile (<lg): sidebar hidden, bottom TabBar used.
	ContextBox opens as full-screen overlay with backdrop.
-->

<!-- ── Mobile overlay — full-screen panel, <lg only ──────────────── -->
{#if showDetail}
	<div class="fixed inset-x-0 top-[3.5rem] bottom-0 z-40 lg:hidden">
		<!-- Backdrop -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute inset-0 bg-black/20 transition-opacity duration-200"
			onclick={closeDetail}
			onkeydown={(e) => { if (e.key === 'Escape') closeDetail(); }}
			role="button"
			tabindex="-1"
			aria-label="Close detail panel"
		></div>

		<!-- Panel -->
		<div
			class="absolute inset-y-0 right-0 w-full max-w-lg overflow-hidden border-l border-border/60 bg-card shadow-lg
				transition-transform duration-300 ease-[var(--ease-spring)]"
			style="transform: translateX(0);"
		>
			{#if isDetailOpen && witnessStore.current}
				<WitnessDetailPanel
					detail={witnessStore.current}
					feedItem={selectedFeedItem}
					onClose={closeDetail}
					onSendMessage={handleSendMessage}
					sending={messageSending}
				/>
			{:else if isDetailLoading}
				<div class="flex h-full items-center justify-center">
					<div class="flex flex-col items-center gap-3 text-muted-foreground">
						<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
						<p class="text-small">{m.pulse_loading_detail()}</p>
					</div>
				</div>
			{:else if selectedWitnessId && witnessStore.detailError}
				<div class="flex h-full items-center justify-center p-6">
					<div class="w-full max-w-sm rounded-xl border border-destructive/30 bg-destructive/5 p-4 text-center">
						<p class="text-body font-semibold text-foreground">Gagal memuat detail tandang</p>
						<p class="mt-1 text-small text-muted-foreground">{witnessStore.detailError}</p>
						<div class="mt-3 flex justify-center gap-2">
							<Button
								variant="outline"
								size="sm"
								onclick={retryWitnessDetailLoad}
							>
								Coba lagi
							</Button>
							<Button
								variant="ghost"
								size="sm"
								onclick={closeDetail}
							>
								Tutup
							</Button>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<!-- ── Desktop workspace — sidebar + feed + context ───────────────── -->
<div class="mx-auto w-full px-4 lg:px-4">

	<!-- Two-column layout — feed + context, tops aligned -->
	<div class="flex w-full items-start gap-4">

	<!-- ── LEFT: Feed column ──────────────────────────────────────── -->
	<div
		class="min-w-0 flex-1 flex flex-col"
		role="region"
		aria-label="Activity feed"
	>
		<!-- Feed content -->
		<section class="flex flex-1 flex-col gap-3">
			{#if feedStore.error && feedStore.filteredStream.length > 0}
				<div
					class="flex items-center justify-between gap-3 rounded-xl border border-destructive/30 bg-destructive/5 px-3 py-2"
					role="status"
					aria-live="polite"
				>
					<p class="text-small text-destructive">{feedStore.error}</p>
					<Button
						variant="outline"
						size="sm"
						onclick={retryFeedLoad}
					>
						Coba lagi
					</Button>
				</div>
			{/if}

			{#if witnessStore.listError}
				<div
					class="flex items-center justify-between gap-3 rounded-xl border border-destructive/30 bg-destructive/5 px-3 py-2"
					role="status"
					aria-live="polite"
				>
					<p class="text-small text-destructive">{witnessStore.listError}</p>
					<Button
						variant="outline"
						size="sm"
						onclick={retryWitnessListLoad}
					>
						Coba lagi
					</Button>
				</div>
			{/if}

			{#if selectedWitnessId && witnessStore.detailError}
				<div
					class="flex items-center justify-between gap-3 rounded-xl border border-destructive/30 bg-destructive/5 px-3 py-2"
					role="status"
					aria-live="polite"
				>
					<p class="text-small text-destructive">{witnessStore.detailError}</p>
					<Button
						variant="outline"
						size="sm"
						onclick={retryWitnessDetailLoad}
					>
						Coba lagi
					</Button>
				</div>
			{/if}

			{#if feedStore.isDiscoverActive}
				<div role="group" aria-label="Discover">
					<DiscoverView />
				</div>
			{:else if feedStore.loading}
				<Masonry
					items={skeletonItems}
					getId={(item) => item.id}
					minColWidth={260}
					maxColWidth={340}
					gap={16}
					animate={false}
				>
					{#snippet children({ item })}
						<div
							class="animate-pulse rounded-xl border border-border/50 bg-card p-4"
							style="min-height: {item.h}px"
						>
							<div class="h-3 w-1/3 rounded bg-muted/60"></div>
							<div class="mt-2 h-4 w-3/4 rounded bg-muted"></div>
							<div class="mt-2 h-3 w-full rounded bg-muted/60"></div>
							<div class="mt-1 h-3 w-2/3 rounded bg-muted/60"></div>
							<div class="mt-3 flex gap-2">
								<div class="h-5 w-5 rounded-full bg-muted/40"></div>
								<div class="h-5 w-5 rounded-full bg-muted/40"></div>
								<div class="h-5 w-10 rounded bg-muted/40"></div>
							</div>
						</div>
					{/snippet}
				</Masonry>
			{:else if feedStore.error && feedStore.filteredStream.length === 0}
				<div
					class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-destructive/30 bg-destructive/5 py-12 text-center"
					role="status"
					aria-live="polite"
				>
					<div
						class="flex size-12 items-center justify-center rounded-full bg-destructive/10 text-destructive"
					>
						<Activity class="size-6" />
					</div>
					<p class="max-w-xs text-body text-muted-foreground">{feedStore.error}</p>
					<Button
						variant="outline"
						size="sm"
						onclick={retryFeedLoad}
					>
						Coba lagi
					</Button>
				</div>
				{:else if masonryStream.length === 0}
					<div
						class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center"
					>
					<div
						class="flex size-12 items-center justify-center rounded-full bg-muted/50 text-muted-foreground"
					>
						{#if feedStore.filter === 'pantauan'}
							<EyeIcon class="size-6" />
						{:else}
							<Activity class="size-6" />
						{/if}
					</div>
					<p class="max-w-xs text-body text-muted-foreground">
						{feedStore.filter === 'semua'
							? m.pulse_empty_state()
							: feedStore.filter === 'pantauan'
								? m.pulse_feed_empty_pantauan()
								: m.pulse_feed_empty_filter()}
					</p>
				</div>
			{:else}
				<div role="list" aria-label="Feed">
					<Masonry
						items={masonryStream}
						getId={(item) => item.stream_id}
						minColWidth={260}
						maxColWidth={340}
						gap={16}
						animate={masonryAnimate}
						duration={300}
						columnClass="masonry-col-constrain"
					>
						{#snippet children({ item: streamItem })}
							{#if streamItem.kind === 'triage'}
								<ChatInput onWitnessCreated={handleWitnessCreated} />
							{:else if streamItem.kind === 'witness'}
								<FeedEventCard
									item={streamItem.data}
									selected={selectedWitnessId === streamItem.data.witness_id}
									onclick={() => selectWitness(streamItem.data.witness_id)}
									onToggleMonitor={() => feedStore.toggleMonitor(streamItem.data.witness_id)}
									onShare={() => shareFeedItem(streamItem.data)}
								/>
							{:else if streamItem.kind === 'system'}
								<FeedSystemCard
									card={streamItem.data}
									onDismiss={() => feedStore.dismissCard(streamItem.stream_id)}
								/>
							{/if}
						{/snippet}
					</Masonry>
				</div>
			{/if}
		</section>
	</div>

	<!-- ── RIGHT: Context box — tabbed sticky workspace ────────────── -->
	<!-- ContextBox handles its own visibility via pin state + active prop.
	     AnimatePresence inside ContextBox handles smooth enter/exit.
	     The component renders nothing when invisible (no DOM footprint). -->
	<ContextBox
		witnessDetail={witnessStore.current}
		feedItem={selectedFeedItem}
		detailLoading={witnessStore.detailLoading}
		messageSending={messageSending}
		active={contextActive}
		selectedUserId={selectedUserId}
		onClose={closeDetail}
		onSendMessage={handleSendMessage}
	/>
	</div><!-- /flex row -->
</div><!-- /outer wrapper -->

<style>
	/*
	 * Masonry grid-item containment — prevents cards with wide intrinsic
	 * content (chip bar, story peek -mx-5) from expanding beyond their
	 * column. contain:inline-size constrains layout width while preserving
	 * visual overflow (pulse-glow box-shadows).
	 */
	:global(.masonry-col-constrain > *) {
		contain: inline-size;
		min-width: 0;
	}

</style>

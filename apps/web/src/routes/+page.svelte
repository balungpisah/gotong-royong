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

	async function handleSendMessage(content: string) {
		messageSending = true;
		try {
			await witnessStore.sendMessage(content);
		} finally {
			messageSending = false;
		}
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

	const masonryStream = $derived<MasonryItem[]>([triageEntry, ...feedStore.filteredStream]);

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
					onClose={closeDetail}
					onSendMessage={handleSendMessage}
					sending={messageSending}
				/>
			{:else if isDetailLoading}
				<div class="flex h-full items-center justify-center">
					<div class="flex flex-col items-center gap-3 text-muted-foreground">
						<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
						<p class="text-xs">{m.pulse_loading_detail()}</p>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<!-- ── Desktop workspace — sidebar + feed + context ───────────────── -->
<div class="mx-auto flex w-full items-start gap-4 px-4 lg:px-4">

	<!-- ── LEFT: Feed column ──────────────────────────────────────── -->
	<div
		class="min-w-0 flex-1 flex flex-col gap-6"
		role="region"
		aria-label="Activity feed"
	>
		<!-- Title row -->
		<div class="flex items-center gap-3">
			<div
				class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary"
			>
				<Activity class="size-5" />
			</div>
			<div>
				<h1 class="text-[var(--fs-h1)] font-bold leading-tight text-foreground">
					{m.pulse_title()}
				</h1>
			</div>
		</div>

		<!-- Feed content -->
		<section class="flex flex-1 flex-col gap-3">
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
			{:else if feedStore.filteredStream.length === 0}
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
					<p class="max-w-xs text-sm text-muted-foreground">
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
						animate={true}
						duration={300}
						columnClass="masonry-col-constrain"
					>
						{#snippet children({ item: streamItem })}
							{#if streamItem.kind === 'triage'}
								<ChatInput />
							{:else if streamItem.kind === 'witness'}
								<FeedEventCard
									item={streamItem.data}
									selected={selectedWitnessId === streamItem.data.witness_id}
									onclick={() => selectWitness(streamItem.data.witness_id)}
									onToggleMonitor={() => feedStore.toggleMonitor(streamItem.data.witness_id)}
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
		detailLoading={witnessStore.detailLoading}
		messageSending={messageSending}
		active={contextActive}
		selectedUserId={selectedUserId}
		onClose={closeDetail}
		onSendMessage={handleSendMessage}
	/>
</div>

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

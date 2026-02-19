<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getWitnessStore, getNotificationStore, getFeedStore } from '$lib/stores';
	import { ChatInput } from '$lib/components/shell';
	import {
		PulseActivityCard,
		WitnessDetailPanel,
		FeedEventCard,
		FeedSystemCard,
		DiscoverView
	} from '$lib/components/pulse';
	import Activity from '@lucide/svelte/icons/activity';
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

	const isDetailOpen = $derived(selectedWitnessId !== null && witnessStore.current !== null);
	const isDetailLoading = $derived(selectedWitnessId !== null && witnessStore.detailLoading);
	const showDetail = $derived(isDetailOpen || isDetailLoading);

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
	Master-detail with GPU-composited transitions (desktop ≥lg).

	Layout: feed column (flex-1) + fixed-position detail panel on the right.
	The detail panel slides in via transform: translateX (GPU-composited, no
	layout reflow). Feed column doesn't move — it just has a right margin
	that's always present on lg+ so the detail has a reserved landing zone.

	On mobile (<lg): detail opens as a fixed overlay with backdrop.
-->

<!-- Mobile overlay — fixed full-screen panel, shown only on <lg when detail is open -->
{#if showDetail}
	<div class="fixed inset-x-0 top-[3.5rem] bottom-0 z-40 lg:hidden">
		<!-- Backdrop — closes panel when clicked -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute inset-0 bg-black/20 transition-opacity duration-200"
			onclick={closeDetail}
			onkeydown={(e) => { if (e.key === 'Escape') closeDetail(); }}
			role="button"
			tabindex="-1"
			aria-label="Close detail panel"
		></div>

		<!-- Panel — slides in from right on mobile -->
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
						<div
							class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"
						></div>
						<p class="text-xs">{m.pulse_loading_detail()}</p>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<div
	class="mx-auto flex w-full px-4 lg:px-0 transition-[margin] duration-300 ease-[var(--ease-spring)]
		{showDetail ? 'lg:mr-[43rem]' : 'lg:mr-0'}"
>
	<!-- Feed column — click anywhere to close detail panel (event delegation) -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="min-w-0 flex-1 flex flex-col gap-6"
		onclick={() => { if (showDetail) closeDetail(); }}
		onkeydown={(e) => { if (e.key === 'Escape' && showDetail) closeDetail(); }}
		role="region"
		tabindex="-1"
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

		<!-- AI-00 triage entry — stop click from closing detail -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="group"
			aria-label="Chat input"
		>
			<ChatInput />
		</div>

		<!-- Feed content -->
		<section class="flex flex-1 flex-col gap-3">
			{#if feedStore.isDiscoverActive}
				<!-- Discover view — dedicated entity discovery tab -->
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<div
					onclick={(e) => e.stopPropagation()}
					onkeydown={(e) => e.stopPropagation()}
					role="group"
					aria-label="Discover"
				>
					<DiscoverView />
				</div>
			{:else if feedStore.loading}
				<!-- Masonry loading skeletons with variable heights -->
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
						<Activity class="size-6" />
					</div>
					<p class="max-w-xs text-sm text-muted-foreground">
						{feedStore.filter === 'semua'
							? m.pulse_empty_state()
							: m.pulse_feed_empty_filter()}
					</p>
				</div>
			{:else}
				<!-- Masonry feed stream — witness cards + inline system cards -->
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<div
					onclick={(e) => e.stopPropagation()}
					onkeydown={(e) => e.stopPropagation()}
					role="list"
					aria-label="Feed"
				>
					<Masonry
						items={feedStore.filteredStream}
						getId={(item) => item.stream_id}
						minColWidth={260}
						maxColWidth={340}
						gap={16}
						animate={true}
						columnClass="masonry-col-constrain"
					>
						{#snippet children({ item: streamItem })}
							{#if streamItem.kind === 'witness'}
								<FeedEventCard
									item={streamItem.data}
									selected={selectedWitnessId === streamItem.data.witness_id}
									onclick={() => selectWitness(streamItem.data.witness_id)}
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
</div>

<!--
	Detail panel — desktop only. Fixed position on the right side of the viewport.
	Slides in via GPU-composited transform: translateX. Zero layout reflow.
	Opacity fades in parallel for a polished entrance.
-->
<div
	class="hidden lg:block fixed top-[3.5rem] right-0 bottom-0 z-30
		transition-[transform,opacity] duration-300 ease-[var(--ease-spring)]
		will-change-[transform,opacity]"
	style="width: 42rem; transform: translateX({showDetail ? '0' : '100%'}); opacity: {showDetail ? '1' : '0'}; pointer-events: {showDetail ? 'auto' : 'none'};"
>
	<div
		class="flex h-full w-full flex-col overflow-hidden border-l border-border/60 bg-card shadow-lg"
	>
		{#if isDetailOpen && witnessStore.current}
			<WitnessDetailPanel
				detail={witnessStore.current}
				onClose={closeDetail}
				onSendMessage={handleSendMessage}
				sending={messageSending}
			/>
		{:else if isDetailLoading}
			<div class="flex flex-1 items-center justify-center">
				<div class="flex flex-col items-center gap-3 text-muted-foreground">
					<div
						class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"
					></div>
					<p class="text-xs">{m.pulse_loading_detail()}</p>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	/*
	 * Masonry grid-item containment — prevents cards with wide intrinsic
	 * content (chip bar, story peek -mx-5) from expanding beyond their
	 * column. The grid items are direct children of .col created by
	 * svelte-bricks; overflow:hidden clips them to column width.
	 */
	:global(.masonry-col-constrain > *) {
		contain: inline-size;
		min-width: 0;
	}
</style>

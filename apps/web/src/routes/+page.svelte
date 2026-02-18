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
</script>

<!--
	Sliding dual-column with fixed-width columns (desktop ≥lg).

	Both feed and detail are always max-w-2xl (672px). They sit inside a
	flex wrapper that is centered via mx-auto. The detail column transitions
	its width from 0 → 672px. Because the wrapper is centered, the feed
	naturally slides left as the wrapper grows to accommodate the detail.

	Feed never changes size — only its position changes.

	On mobile (<lg): feed is full-width, detail opens as a fixed overlay
	with a semi-transparent backdrop.
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

		<!-- Panel — slides up from bottom on mobile -->
		<div
			class="relative h-full overflow-hidden bg-card shadow-xl transition-transform duration-300 ease-out"
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
	class="mx-auto flex px-4 transition-all duration-[var(--dur-slow)] ease-[var(--ease-spring)] lg:px-0"
	style="width: {showDetail ? 'calc(42rem + 42rem + 1.5rem)' : '42rem'}; max-width: 100%;"
>
	<!-- Feed column — click anywhere to close detail panel (event delegation) -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="w-full lg:w-[42rem] shrink-0 flex flex-col gap-6"
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
				<div class="flex flex-col gap-3">
					{#each { length: 3 } as _}
						<div class="animate-pulse rounded-xl border border-border/40 bg-muted/30 p-4">
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
					{/each}
				</div>
			{:else if feedStore.filteredStream.length === 0}
				<div
					class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/60 py-12 text-center"
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
				<!-- Polymorphic feed stream — witness cards + inline system cards -->
				<div class="flex flex-col gap-3" role="list">
					{#each feedStore.filteredStream as streamItem (streamItem.stream_id)}
						<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
						<div
							onclick={(e) => e.stopPropagation()}
							onkeydown={(e) => e.stopPropagation()}
							role="listitem"
						>
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
						</div>
					{/each}
				</div>
			{/if}
		</section>
	</div>

	<!--
		Detail column — desktop only (hidden on mobile, which uses the fixed overlay above).
		Transitions width from 0 → 42rem and opacity from 0 → 1.
		overflow:hidden clips content when collapsed.
		The gap (margin-left) also transitions.
	-->
	<div
		class="hidden lg:block shrink-0 overflow-hidden transition-all duration-[var(--dur-slow)] ease-[var(--ease-spring)]"
		style="width: {showDetail ? '42rem' : '0px'}; margin-left: {showDetail ? '1.5rem' : '0px'}; opacity: {showDetail ? '1' : '0'};"
	>
		<div
			class="sticky top-[5.5rem] flex w-[42rem] flex-col overflow-hidden rounded-xl border border-border/60 bg-card shadow-lg"
			style="height: calc(100dvh - 7rem);"
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
</div>

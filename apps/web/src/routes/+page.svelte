<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getWitnessStore, getNotificationStore } from '$lib/stores';
	import { ChatInput } from '$lib/components/shell';
	import { PulseActivityCard, WitnessDetailPanel } from '$lib/components/pulse';
	import Activity from '@lucide/svelte/icons/activity';

	const witnessStore = getWitnessStore();
	const notificationStore = getNotificationStore();

	// Load data on mount
	$effect(() => {
		witnessStore.loadList();
		notificationStore.loadNotifications();
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
	Sliding dual-column with fixed-width columns.

	Both feed and detail are always max-w-2xl (672px). They sit inside a
	flex wrapper that is centered via mx-auto. The detail column transitions
	its width from 0 → 672px. Because the wrapper is centered, the feed
	naturally slides left as the wrapper grows to accommodate the detail.

	Feed never changes size — only its position changes.
-->
<div
	class="mx-auto flex transition-all duration-[var(--dur-slow)] ease-[var(--ease-spring)]"
	style="width: {showDetail ? 'calc(42rem + 42rem + 1.5rem)' : '42rem'};"
>
	<!-- Feed column — always 672px, never changes -->
	<div class="w-[42rem] shrink-0 flex flex-col gap-6">
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

		<!-- AI-00 triage entry -->
		<div>
			<ChatInput />
		</div>

		<!-- Recent activity -->
		<section class="flex flex-1 flex-col gap-3">
			<h2 class="text-[var(--fs-h3)] font-semibold text-foreground">
				{m.pulse_recent_activity()}
			</h2>

			{#if witnessStore.listLoading}
				<div class="flex flex-col gap-3">
					{#each { length: 3 } as _}
						<div class="animate-pulse rounded-xl border border-border/40 bg-muted/30 p-4">
							<div class="h-4 w-3/4 rounded bg-muted"></div>
							<div class="mt-2 h-3 w-full rounded bg-muted/60"></div>
							<div class="mt-1 h-3 w-2/3 rounded bg-muted/60"></div>
							<div class="mt-3 flex gap-2">
								<div class="h-5 w-16 rounded-full bg-muted/40"></div>
								<div class="h-5 w-10 rounded bg-muted/40"></div>
								<div class="h-5 w-10 rounded bg-muted/40"></div>
							</div>
						</div>
					{/each}
				</div>
			{:else if sortedWitnesses.length === 0}
				<div
					class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/60 py-12 text-center"
				>
					<div
						class="flex size-12 items-center justify-center rounded-full bg-muted/50 text-muted-foreground"
					>
						<Activity class="size-6" />
					</div>
					<p class="max-w-xs text-sm text-muted-foreground">
						{m.pulse_empty_state()}
					</p>
				</div>
			{:else}
				<div class="flex flex-col gap-3">
					{#each sortedWitnesses as witness (witness.witness_id)}
						<PulseActivityCard
							{witness}
							selected={selectedWitnessId === witness.witness_id}
							onclick={() => selectWitness(witness.witness_id)}
						/>
					{/each}
				</div>
			{/if}
		</section>
	</div>

	<!--
		Detail column — always in the DOM. Transitions width from 0 → 42rem
		and opacity from 0 → 1. overflow:hidden clips content when collapsed.
		The gap (margin-left) also transitions.
	-->
	<div
		class="shrink-0 overflow-hidden transition-all duration-[var(--dur-slow)] ease-[var(--ease-spring)]"
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
						<p class="text-xs">Memuat detail...</p>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

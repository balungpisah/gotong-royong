<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getWitnessStore, getNotificationStore } from '$lib/stores';
	import { ChatInput, PulseActivityCard } from '$lib/components/shell';
	import Activity from '@lucide/svelte/icons/activity';
	import Flame from '@lucide/svelte/icons/flame';
	import UsersIcon from '@lucide/svelte/icons/users';
	import BellRing from '@lucide/svelte/icons/bell-ring';

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
</script>

<div class="flex flex-1 flex-col gap-6">
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

	<!-- Stats row -->
	<div class="grid grid-cols-3 gap-3">
		<div class="rounded-xl border border-border/60 bg-card p-3 text-center">
			<div class="flex items-center justify-center gap-1.5 text-muted-foreground">
				<Flame class="size-3.5" />
				<span class="text-[var(--fs-caption)] font-medium uppercase tracking-wider">
					{m.pulse_stats_active()}
				</span>
			</div>
			<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">
				{witnessStore.activeWitnesses.length}
			</p>
		</div>

		<div class="rounded-xl border border-border/60 bg-card p-3 text-center">
			<div class="flex items-center justify-center gap-1.5 text-muted-foreground">
				<UsersIcon class="size-3.5" />
				<span class="text-[var(--fs-caption)] font-medium uppercase tracking-wider">
					{m.pulse_stats_total()}
				</span>
			</div>
			<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">
				{witnessStore.witnesses.length}
			</p>
		</div>

		<div class="rounded-xl border border-border/60 bg-card p-3 text-center">
			<div class="flex items-center justify-center gap-1.5 text-muted-foreground">
				<BellRing class="size-3.5" />
				<span class="text-[var(--fs-caption)] font-medium uppercase tracking-wider">
					{m.pulse_stats_unread()}
				</span>
			</div>
			<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">
				{notificationStore.unreadCount}
			</p>
		</div>
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
					<PulseActivityCard {witness} />
				{/each}
			</div>
		{/if}
	</section>

	<!-- Chat input -->
	<ChatInput />
</div>

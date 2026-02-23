<script lang="ts">
	import { untrack } from 'svelte';
	import { getCommunityStore, getGroupStore } from '$lib/stores';
	import {
		CommunityHeader,
		CommunityIcjSummary,
		TierDistribution,
		ActiveHighlights,
		SignalFlowChart
	} from '$lib/components/komunitas';
	import { m } from '$lib/paraglide/messages';
	import UsersIcon from '@lucide/svelte/icons/users';

	const store = getCommunityStore();
	const groupStore = getGroupStore();

	$effect(() => {
		untrack(() => {
			store.loadDashboard();
			groupStore.loadMyGroups();
		});
	});

	const dashboard = $derived(store.dashboard);
	const myGroupCount = $derived(groupStore.myGroupCount);
</script>

{#if store.dashboardLoading && !dashboard}
	<div class="flex h-64 items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-muted-foreground">
			<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
			<p class="text-xs">{m.loading_community()}</p>
		</div>
	</div>
{:else if dashboard}
	<div class="mx-auto w-full max-w-3xl py-6">
		<!-- Quick navigation -->
		<div class="mb-6 px-4">
			<a
				href="/komunitas/kelompok"
				class="block rounded-xl border border-border/40 bg-card p-4 transition hover:border-border"
			>
				<div class="flex items-start justify-between gap-3">
					<div>
						<p class="text-sm font-bold text-foreground">{m.group_nav_title()}</p>
						<p class="mt-0.5 text-xs text-muted-foreground/80">{m.group_nav_subtitle()}</p>
						<p class="mt-2 text-[11px] text-muted-foreground/70">{m.group_nav_count({ count: myGroupCount })}</p>
					</div>
					<div class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
						<UsersIcon class="size-5" />
					</div>
				</div>
			</a>
		</div>

		<!-- Header: community info with surface treatment -->
		<div class="page-header rounded-xl border border-border/30 px-4 py-4 mb-6">
			<CommunityHeader {dashboard} />
		</div>

		<div class="space-y-6 px-4">
			{#if dashboard.avg_tier !== undefined}
				<div class="flex items-center gap-2 rounded-lg bg-muted/10 px-4 py-2">
					<span class="text-caption text-muted-foreground">{m.komunitas_avg_tier()}</span>
					<span class="text-sm font-bold text-foreground">{dashboard.avg_tier.toFixed(1)}</span>
				</div>
			{/if}
			<CommunityIcjSummary summary={dashboard.icj_summary} />
			<TierDistribution tiers={dashboard.tier_distribution} />
			<ActiveHighlights members={dashboard.active_highlights} />
			<SignalFlowChart data={dashboard.signal_flow} />
		</div>
	</div>
{:else if store.dashboardError}
	<div class="flex h-64 flex-col items-center justify-center gap-3 text-center">
		<p class="text-xs text-red-500">{store.dashboardError}</p>
	</div>
{/if}

<style>
	.page-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
	}
</style>

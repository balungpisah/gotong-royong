<script lang="ts">
	import { getCommunityStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import {
		CommunityHeader,
		CommunityIcjSummary,
		TierDistribution,
		ActiveHighlights,
		SignalFlowChart
	} from '$lib/components/komunitas';

	const store = getCommunityStore();

	// Load dashboard on mount if not already loaded
	$effect(() => {
		if (!store.hasDashboard) {
			store.loadDashboard();
		}
	});

	const dashboard = $derived(store.dashboard);
</script>

{#if store.dashboardLoading && !dashboard}
	<div class="flex h-full items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-muted-foreground">
			<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
			<p class="text-xs">{m.loading_community_dashboard()}</p>
		</div>
	</div>
{:else if dashboard}
	<div class="flex h-full flex-col overflow-hidden">
		<!-- Header: community info with surface treatment matching tandang -->
		<div class="panel-header shrink-0 border-b border-border/60">
			<div class="px-4 py-3">
				<CommunityHeader {dashboard} />
			</div>
		</div>

		<!-- Scrollable content -->
		<div class="flex-1 overflow-y-auto px-4 py-4 space-y-4">
			<CommunityIcjSummary summary={dashboard.icj_summary} />
			<TierDistribution tiers={dashboard.tier_distribution} />
			<ActiveHighlights members={dashboard.active_highlights} />
			<SignalFlowChart data={dashboard.signal_flow} />
		</div>
	</div>
{:else if store.dashboardError}
	<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
		<p class="text-xs text-red-500">{store.dashboardError}</p>
	</div>
{/if}

<style>
	.panel-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
	}
</style>

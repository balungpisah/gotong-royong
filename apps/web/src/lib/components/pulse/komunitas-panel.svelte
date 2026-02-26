<script lang="ts">
	import { getCommunityStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import {
		GdfWeatherWidget,
		CommunityIcjSummary,
		TierDistribution,
		ActiveHighlights,
		SignalFlowChart
	} from '$lib/components/komunitas';
	import PanelPinnedCard from './panel-pinned-card.svelte';
	import Activity from '@lucide/svelte/icons/activity';

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
			<p class="text-small">{m.loading_community_dashboard()}</p>
		</div>
	</div>
{:else if dashboard}
	<div class="flex h-full flex-col overflow-hidden">
		<!-- Fixed header — PanelPinnedCard shared two-column template -->
		<PanelPinnedCard>
			{#snippet left()}
				<div class="flex items-center gap-2.5">
					<div
						class="flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary"
					>
						<Activity class="size-3.5" />
					</div>
					<div class="min-w-0 flex-1">
						<p class="truncate text-body font-bold leading-tight text-foreground">
							{dashboard.community_name}
						</p>
						<p class="truncate text-caption text-muted-foreground">
							{m.komunitas_member_count({ count: String(dashboard.member_count) })}
						</p>
					</div>
				</div>
			{/snippet}
			{#snippet right()}
				<div class="icj-col">
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-i)">I</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									dashboard.icj_summary.avg_integrity * 100
								)}%; background: var(--c-tandang-i)"
							></div>
						</div>
						<span class="icj-value">{Math.round(dashboard.icj_summary.avg_integrity * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-c)">C</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									dashboard.icj_summary.avg_competence * 100
								)}%; background: var(--c-tandang-c)"
							></div>
						</div>
						<span class="icj-value">{Math.round(dashboard.icj_summary.avg_competence * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-j)">J</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									dashboard.icj_summary.avg_judgment * 100
								)}%; background: var(--c-tandang-j)"
							></div>
						</div>
						<span class="icj-value">{Math.round(dashboard.icj_summary.avg_judgment * 100)}</span>
					</div>
				</div>
			{/snippet}
		</PanelPinnedCard>

		<!-- Scrollable content -->
		<div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2">
			<GdfWeatherWidget weather={dashboard.weather} size="full" />
			<CommunityIcjSummary summary={dashboard.icj_summary} />
			<TierDistribution tiers={dashboard.tier_distribution} />
			<ActiveHighlights members={dashboard.active_highlights} />
			<SignalFlowChart data={dashboard.signal_flow} />
		</div>
	</div>
{:else if store.dashboardError}
	<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
		<p class="text-small text-red-500">{store.dashboardError}</p>
	</div>
{/if}

<style>
	/* Right col content — rendered inside PanelPinnedCard's right slot */
	.icj-col {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 0.45rem;
		height: 100%;
	}

	.icj-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.icj-label {
		font-size: 0.75rem;
		font-weight: 800;
		letter-spacing: 0.05em;
		width: 0.75rem;
		flex-shrink: 0;
	}

	.icj-bar-track {
		flex: 1;
		height: 5px;
		border-radius: 99px;
		background: color-mix(in srgb, var(--color-muted) 40%, transparent);
		overflow: hidden;
	}

	.icj-bar-fill {
		height: 100%;
		border-radius: 99px;
		transition: width 500ms ease;
	}

	.icj-value {
		font-size: 11px;
		line-height: 16px;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: var(--color-muted-foreground);
		width: 1.5rem;
		text-align: right;
		flex-shrink: 0;
	}
</style>

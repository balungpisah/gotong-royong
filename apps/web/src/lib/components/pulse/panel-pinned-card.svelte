<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * PanelPinnedCard — shared two-column header shell used across context-box tabs.
	 *
	 * Mirrors the .detail-header / .pinned-card / .pinned-columns pattern from
	 * witness-detail-panel.svelte so all panels share identical proportions:
	 *   - header background + border
	 *   - inner card margin/padding (0.5rem 0.75rem / 0.625rem 0.75rem)
	 *   - 60% left col / divider / 40% right col
	 *
	 * Usage (Svelte 5 snippets):
	 *   <PanelPinnedCard>
	 *     {#snippet left()}  … identity content …  {/snippet}
	 *     {#snippet right()} … panel data …        {/snippet}
	 *   </PanelPinnedCard>
	 */

	interface Props {
		left: Snippet;
		right: Snippet;
	}

	const { left, right }: Props = $props();
</script>

<div class="panel-header shrink-0">
	<div class="pinned-card">
		<div class="pinned-columns">
			<!-- LEFT slot — 60% -->
			<div class="left-col">
				{@render left()}
			</div>

			<!-- DIVIDER — matches witness-detail-panel accent divider -->
			<div class="mx-2 w-px self-stretch bg-border/40"></div>

			<!-- RIGHT slot — 40% -->
			<div class="right-col">
				{@render right()}
			</div>
		</div>
	</div>
</div>

<style>
	/* Matches .detail-header in witness-detail-panel */
	.panel-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
		border-bottom: 1px solid color-mix(in srgb, var(--color-border) 30%, transparent);
	}

	/* Matches .pinned-card in witness-detail-panel */
	.pinned-card {
		margin: 0.5rem 0.75rem;
		padding: 0.625rem 0.75rem;
		border-radius: var(--r-lg, 0.5rem);
		background: transparent;
	}

	/* Matches .pinned-columns — 60/40 split */
	.pinned-columns {
		display: flex;
		align-items: stretch;
	}

	.left-col {
		flex: 0 1 60%;
		min-width: 0;
	}

	/* Matches .phase-list-col */
	.right-col {
		flex: 0 0 calc(40% - 1rem);
		min-width: 0;
	}
</style>

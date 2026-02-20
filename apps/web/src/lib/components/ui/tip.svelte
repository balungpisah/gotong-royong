<script lang="ts">
	/**
	 * <Tip> — global-aware tooltip wrapper.
	 *
	 * Wraps any element with a bits-ui tooltip that respects the global
	 * showTooltips preference from PreferencesStore.
	 *
	 * Usage:
	 *   <Tip text="Simpan">
	 *     <button><BookmarkIcon /></button>
	 *   </Tip>
	 *
	 * When tooltips are disabled globally, renders children directly (no wrapper).
	 *
	 * Uses bits-ui TooltipPrimitive.Trigger directly with the `child` snippet
	 * pattern to avoid nested button issues. The `child` snippet receives trigger
	 * props and spreads them onto a thin <span class="inline-flex"> wrapper
	 * that provides a proper anchor box for Floating UI tooltip positioning
	 * without affecting layout (inline-flex shrinks to fit its content).
	 */

	import type { Snippet } from 'svelte';
	import { Tooltip as TooltipPrimitive } from 'bits-ui';
	import { Tooltip, TooltipContent } from '$lib/components/ui/tooltip';
	import { getPreferencesStore } from '$lib/stores';

	interface Props {
		/** Tooltip text to display. */
		text: string;
		/** Which side to show the tooltip. */
		side?: 'top' | 'bottom' | 'left' | 'right';
		/** Content to wrap. */
		children: Snippet;
	}

	let { text, side = 'top', children }: Props = $props();

	const prefs = getPreferencesStore();
</script>

{#if prefs.showTooltips}
	<Tooltip>
		<TooltipPrimitive.Trigger>
			{#snippet child({ props })}
				<span {...props} class="inline-flex">
					{@render children()}
				</span>
			{/snippet}
		</TooltipPrimitive.Trigger>
		<TooltipContent {side}>
			{text}
		</TooltipContent>
	</Tooltip>
{:else}
	{@render children()}
{/if}

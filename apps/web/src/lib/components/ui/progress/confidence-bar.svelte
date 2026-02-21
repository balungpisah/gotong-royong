<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		value = 0,
		max = 100,
		showLabel = true,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		value?: number;
		max?: number;
		showLabel?: boolean;
	} = $props();

	const percentage = $derived(Math.round((value / max) * 100));
	const fillColor = $derived(percentage >= 70 ? 'bg-api' : percentage >= 40 ? 'bg-peringatan' : 'bg-bahaya');
</script>

<div
	bind:this={ref}
	data-slot="confidence-bar"
	class={cn('flex items-center gap-2', className)}
	{...restProps}
>
	<div class="relative h-1 flex-1 overflow-hidden rounded-full bg-batu">
		<div
			class={cn('h-full rounded-full transition-all', fillColor)}
			style="width: {percentage}%"
		></div>
	</div>
	{#if showLabel}
		<span class="text-xs font-semibold text-kayu">{percentage}%</span>
	{/if}
</div>

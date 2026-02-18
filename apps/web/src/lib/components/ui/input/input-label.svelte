<script lang="ts">
	import type { HTMLLabelAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLLabelAttributes> = $props();
</script>

<label
	bind:this={ref}
	data-slot="input-label"
	class={cn('text-[11px] font-bold uppercase tracking-wide text-tanah', className)}
	{...restProps}
>
	{@render children?.()}
</label>

<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="avatar-group"
	class={cn(
		'flex items-center -space-x-2 [&>[data-slot=avatar]]:border-2 [&>[data-slot=avatar]]:border-susu',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>

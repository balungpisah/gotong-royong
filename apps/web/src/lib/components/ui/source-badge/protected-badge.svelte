<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';
	import Lock from '@lucide/svelte/icons/lock';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> = $props();
</script>

<span
	bind:this={ref}
	data-slot="protected-badge"
	class={cn(
		'inline-flex items-center gap-1 rounded-full bg-peringatan-lembut px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-peringatan',
		className
	)}
	{...restProps}
>
	<Lock class="size-2.5" />
	<span>Dilindungi</span>
</span>

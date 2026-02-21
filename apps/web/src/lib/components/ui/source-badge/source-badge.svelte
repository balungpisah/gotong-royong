<script lang="ts" module>
	import { tv } from 'tailwind-variants';

	export const sourceBadgeVariants = tv({
		base: 'inline-flex items-center gap-1 rounded-full px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide',
		variants: {
			source: {
				ai: 'bg-api/10 text-api',
				human: 'bg-berhasil-lembut text-berhasil',
				system: 'bg-batu/20 text-kayu'
			}
		},
		defaultVariants: {
			source: 'ai'
		}
	});
</script>

<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';
	import type { SourceTag } from '$lib/types';

	let {
		ref = $bindable(null),
		source = 'ai' as SourceTag,
		class: className,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> & {
		source?: SourceTag;
	} = $props();

	const icons: Record<SourceTag, string> = {
		ai: '🤖',
		human: '👤',
		system: '⚙'
	};
</script>

<span
	bind:this={ref}
	data-slot="source-badge"
	class={cn(sourceBadgeVariants({ source }), className)}
	{...restProps}
>
	<span class="text-xs">{icons[source]}</span>
	<span>{source === 'ai' ? 'AI' : source === 'human' ? 'Manual' : 'Sistem'}</span>
</span>

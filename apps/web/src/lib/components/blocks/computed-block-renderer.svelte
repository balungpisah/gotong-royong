<script lang="ts">
	import type { ComputedBlock } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Progress } from '$lib/components/ui/progress';
	import { Badge } from '$lib/components/ui/badge';

	let { block }: { block: ComputedBlock } = $props();

	const percentage = $derived(block.max ? Math.round((block.value / block.max) * 100) : block.value);
</script>

<div class="flex flex-col gap-2" data-slot="computed-block">
	{#if block.display === 'progress'}
		<div class="flex items-center justify-between">
			<span class="text-sm font-medium text-foreground">{block.label}</span>
			<span class="text-sm font-bold text-api">{block.value}{block.unit || ''}</span>
		</div>
		<Progress value={percentage} max={100} class="h-1.5" />
	{:else if block.display === 'confidence'}
		<div class="flex items-center justify-between">
			<span class="text-sm font-medium text-foreground">{block.label}</span>
			<Badge variant="confidence">{block.value}{block.unit || '%'}</Badge>
		</div>
		<div class="h-1 w-full overflow-hidden rounded-full bg-batu">
			<div
				class={cn('h-full rounded-full transition-all', percentage >= 70 ? 'bg-api' : 'bg-peringatan')}
				style="width: {percentage}%"
			></div>
		</div>
	{:else if block.display === 'status'}
		<div class="flex items-center gap-3">
			<div class={cn('size-2.5 rounded-full', percentage >= 70 ? 'bg-berhasil' : percentage >= 40 ? 'bg-peringatan' : 'bg-bahaya')}></div>
			<span class="text-sm font-medium">{block.label}</span>
			<span class="text-sm text-muted-foreground">{block.value}{block.unit || ''}</span>
		</div>
	{:else if block.display === 'score' || block.display === 'counter'}
		<div class="flex flex-col items-center gap-1 rounded-lg border border-border bg-card p-4">
			<span class="text-3xl font-extrabold text-api">{block.value}</span>
			{#if block.unit}
				<span class="text-xs font-medium uppercase tracking-wide text-muted-foreground">{block.unit}</span>
			{/if}
			<span class="text-sm text-foreground">{block.label}</span>
		</div>
	{/if}
</div>

<script lang="ts">
	import type { ReferenceBlock } from '$lib/types';
	import { cn } from '$lib/utils';
	import { resolveTrajectoryColor } from '$lib/utils/trajectory-colors';
	import { Badge } from '$lib/components/ui/badge';
	import ExternalLink from '@lucide/svelte/icons/external-link';

	let {
		block,
		onnavigate
	}: { block: ReferenceBlock; onnavigate?: (refId: string, refType: string) => void } = $props();

	const refTypeLabels: Record<string, string> = {
		seed: 'Saksi',
		plan: 'Rencana',
		checkpoint: 'Titik Periksa',
		document: 'Dokumen'
	};

	const colors = $derived(resolveTrajectoryColor(block.track_hint));
</script>

<button
	class={cn(
		'flex w-full items-start gap-3 rounded-lg border bg-card p-3 text-left transition hover:bg-muted/50',
		colors ? colors.border : 'border-border',
		colors && 'border-l-4'
	)}
	onclick={() => onnavigate?.(block.ref_id, block.ref_type)}
	data-slot="reference-block"
>
	<div class="flex-1">
		<div class="mb-1 flex items-center gap-2">
			<Badge
				variant={block.track_hint ? (`track-${block.track_hint}` as any) : 'secondary'}
				class="text-[10px]"
			>
				{refTypeLabels[block.ref_type] || block.ref_type}
			</Badge>
		</div>
		<p class="text-body font-medium text-foreground">{block.title}</p>
		{#if block.snippet}
			<p class="mt-1 text-small text-muted-foreground line-clamp-2">{block.snippet}</p>
		{/if}
	</div>
	<ExternalLink class="mt-1 size-4 shrink-0 text-muted-foreground" />
</button>

<script lang="ts">
	import type { AiCardMessage } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import { BlockRenderer } from '$lib/components/blocks';

	let { message }: { message: AiCardMessage } = $props();

	const badgeLabels: Record<string, string> = {
		classified: '🤖 Klasifikasi',
		suggested: '🤖 Saran',
		stalled: '⚠ Macet',
		dampak: '🌱 Dampak',
		ringkasan: '📝 Ringkasan',
		duplikat: '⚠ Duplikat'
	};

	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));
</script>

<div class="flex flex-col gap-2 rounded-lg border border-dashed border-api/30 bg-api/5 p-3" data-slot="ai-inline-card">
	<div class="flex items-center justify-between">
		{#if message.badge}
			<Badge variant="confidence" class="text-[10px]">{badgeLabels[message.badge] || message.badge}</Badge>
		{/if}
		<span class="text-[9px] text-muted-foreground">{timeStr}</span>
	</div>
	{#if message.title}
		<p class="text-xs font-bold text-foreground">{message.title}</p>
	{/if}
	{#each message.blocks as block (block.id)}
		<BlockRenderer {block} />
	{/each}
</div>

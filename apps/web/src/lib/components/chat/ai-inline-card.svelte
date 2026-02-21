<script lang="ts">
	import type { AiCardMessage } from '$lib/types';
	import { BlockRenderer } from '$lib/components/blocks';

	let { message }: { message: AiCardMessage } = $props();
</script>

<div class="flex flex-col gap-2" data-slot="ai-inline-card">
	{#if message.title}
		<p class="text-xs font-bold text-foreground">{message.title}</p>
	{/if}
	{#each message.blocks as block (block.id)}
		<BlockRenderer {block} />
	{/each}
</div>

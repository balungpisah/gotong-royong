<script lang="ts">
	import type { DisplayBlock } from '$lib/types';
	import { renderMarkdown } from '$lib/utils';

	let { block }: { block: DisplayBlock } = $props();
</script>

<div class="rounded-lg border border-border bg-card p-4" data-slot="display-block">
	<h3 class="mb-2 text-sm font-bold text-foreground">{block.title}</h3>

	<div class="prose prose-sm max-w-none text-foreground prose-headings:text-foreground prose-p:text-foreground prose-a:text-api prose-strong:text-foreground">
		{@html renderMarkdown(block.content)}
	</div>

	{#if block.media?.length}
		<div class="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-3">
			{#each block.media as media}
				{#if media.type === 'image'}
					<img
						src={media.url}
						alt={media.alt || ''}
						class="rounded-md border border-border object-cover"
						loading="lazy"
					/>
				{:else if media.type === 'video'}
					<video
						src={media.url}
						controls
						class="rounded-md border border-border"
						preload="metadata"
					>
						<track kind="captions" />
					</video>
				{/if}
			{/each}
		</div>
	{/if}

	{#if block.meta && Object.keys(block.meta).length > 0}
		<div class="mt-3 flex flex-wrap gap-x-4 gap-y-1 border-t border-border/50 pt-2">
			{#each Object.entries(block.meta) as [key, value]}
				<span class="text-xs text-muted-foreground">
					<span class="font-medium capitalize">{key.replace(/_/g, ' ')}:</span> {String(value)}
				</span>
			{/each}
		</div>
	{/if}
</div>

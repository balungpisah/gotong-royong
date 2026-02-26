<script lang="ts">
	import type { DocumentBlock } from '$lib/types';
	import { renderMarkdown } from '$lib/utils';
	import { SourceBadge } from '$lib/components/ui/source-badge';
	import Lock from '@lucide/svelte/icons/lock';

	let { block }: { block: DocumentBlock } = $props();
</script>

{#if block.title}
	<h3 class="mb-3 text-body font-bold text-foreground">{block.title}</h3>
{/if}

<div class="flex flex-col gap-4" data-slot="document-block">
	{#each block.sections as section (section.id)}
		<div class="relative rounded-md border border-border/50 bg-card p-3">
			<div class="mb-2 flex items-center gap-2">
				{#if section.heading}
					<h4 class="text-small font-bold uppercase tracking-wide text-foreground">{section.heading}</h4>
				{/if}
				<SourceBadge source={section.source} />
				{#if section.locked_fields.length > 0}
					<Lock class="size-3 text-peringatan" />
				{/if}
			</div>
			<div class="prose prose-sm max-w-none text-foreground prose-headings:text-foreground prose-p:text-foreground prose-a:text-api prose-strong:text-foreground">
				{@html renderMarkdown(section.content)}
			</div>
		</div>
	{/each}
</div>

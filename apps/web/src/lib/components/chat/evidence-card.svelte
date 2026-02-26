<script lang="ts">
	import type { EvidenceMessage } from '$lib/types';

	let { message }: { message: EvidenceMessage } = $props();
</script>

<div class="flex flex-col gap-2" data-slot="evidence-card">
	<p class="text-body text-foreground">{message.content}</p>
	{#if message.attachments?.length}
		<div class="flex flex-wrap gap-1.5">
			{#each message.attachments as att, attIdx (attIdx)}
				{#if att.type === 'image' || att.type === 'receipt'}
					<img
						src={att.url}
						alt={att.alt || ''}
						class="h-16 w-auto rounded border border-border object-cover"
						loading="lazy"
					/>
				{/if}
			{/each}
		</div>
	{/if}
</div>

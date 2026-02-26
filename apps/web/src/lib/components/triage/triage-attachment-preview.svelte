<script lang="ts">
	import type { TriageAttachment } from '$lib/types';
	import X from '@lucide/svelte/icons/x';
	import Mic from '@lucide/svelte/icons/mic';
	import Video from '@lucide/svelte/icons/video';

	interface Props {
		attachments: TriageAttachment[];
		onRemove: (id: string) => void;
	}

	let { attachments, onRemove }: Props = $props();
</script>

{#if attachments.length > 0}
	<div class="flex gap-2 overflow-x-auto px-3 pb-2">
		{#each attachments as att (att.id)}
			<div class="group relative shrink-0">
				{#if att.type === 'image'}
					<img
						src={att.preview_url}
						alt=""
						class="size-12 rounded-lg object-cover ring-1 ring-border/40"
					/>
				{:else if att.type === 'video'}
					<div
						class="flex size-12 items-center justify-center rounded-lg bg-muted ring-1 ring-border/40"
					>
						<Video class="size-5 text-muted-foreground" />
					</div>
				{:else}
					<div
						class="flex size-12 items-center justify-center rounded-lg bg-muted ring-1 ring-border/40"
					>
						<Mic class="size-5 text-muted-foreground" />
					</div>
				{/if}

				<button
					type="button"
					onclick={() => onRemove(att.id)}
					class="absolute -right-1.5 -top-1.5 flex size-5 items-center justify-center rounded-full bg-destructive text-destructive-foreground opacity-0 shadow-sm transition group-hover:opacity-100"
					aria-label="Remove"
				>
					<X class="size-3" />
				</button>
			</div>
		{/each}
	</div>
{/if}

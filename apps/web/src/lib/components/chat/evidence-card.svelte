<script lang="ts">
	import type { EvidenceMessage } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge';
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import FileCheck from '@lucide/svelte/icons/file-check';

	let { message }: { message: EvidenceMessage } = $props();

	const typeLabels: Record<string, string> = {
		testimony: 'Kesaksian',
		corroboration: 'Korroborasi',
		document: 'Dokumen'
	};

	const initials = $derived(message.author.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase());
	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));
</script>

<div class="flex gap-2" data-slot="evidence-card">
	<Avatar class="size-8 shrink-0">
		<AvatarFallback class="text-[10px]">{initials}</AvatarFallback>
	</Avatar>
	<div class="max-w-[80%] rounded-lg border-l-4 border-berhasil bg-berhasil-lembut/30 p-3">
		<div class="mb-1.5 flex items-center gap-2">
			<FileCheck class="size-3.5 text-berhasil" />
			<Badge variant="success" class="text-[9px]">{typeLabels[message.evidence_type] || message.evidence_type}</Badge>
			<span class="text-[10px] text-muted-foreground">{message.author.name}</span>
		</div>
		<p class="text-sm text-foreground">{message.content}</p>
		{#if message.attachments?.length}
			<div class="mt-2 flex flex-wrap gap-1.5">
				{#each message.attachments as att}
					{#if att.type === 'image' || att.type === 'receipt'}
						<img src={att.url} alt={att.alt || ''} class="h-16 w-auto rounded border border-border object-cover" loading="lazy" />
					{/if}
				{/each}
			</div>
		{/if}
		<span class="mt-1.5 block text-[9px] text-muted-foreground">{timeStr}</span>
	</div>
</div>

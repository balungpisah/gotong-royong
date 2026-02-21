<script lang="ts">
	import type { DiffCard, DiffAction } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { ProtectedBadge } from '$lib/components/ui/source-badge';
	import Plus from '@lucide/svelte/icons/plus';
	import Minus from '@lucide/svelte/icons/minus';
	import Pencil from '@lucide/svelte/icons/pencil';
	import ArrowUpDown from '@lucide/svelte/icons/arrow-up-down';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	let { diff, ondiffaction, bare = false }: { diff: DiffCard; ondiffaction?: (action: DiffAction) => void; bare?: boolean } = $props();

	let showEvidence = $state(false);

	const opIcons = { add: Plus, remove: Minus, modify: Pencil, reorder: ArrowUpDown };
	const opColors = {
		add: 'bg-berhasil-lembut text-berhasil',
		remove: 'bg-bahaya-lembut text-bahaya',
		modify: 'bg-peringatan-lembut text-peringatan',
		reorder: 'bg-keterangan-lembut text-keterangan'
	};
</script>

<div class={cn('flex flex-col gap-3', !bare && 'rounded-lg border border-dashed border-api/40 bg-api/5 p-4')} data-slot="diff-card">
	<!-- Header -->
	<div class="flex items-start justify-between gap-2">
		<div>
			<p class="text-sm font-bold text-foreground">{diff.summary}</p>
			<p class="mt-0.5 text-xs text-muted-foreground">
				{diff.items.length} perubahan · {diff.target_type}
			</p>
		</div>
		<Badge variant="default" class="shrink-0 text-[10px]">🤖 AI</Badge>
	</div>

	<!-- Evidence (collapsible) -->
	{#if diff.evidence?.length}
		<button
			class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
			onclick={() => showEvidence = !showEvidence}
		>
			<ChevronDown class={cn('size-3 transition-transform', showEvidence && 'rotate-180')} />
			<span>Berdasarkan {diff.evidence.length} bukti</span>
		</button>
		{#if showEvidence}
			<div class="flex flex-col gap-1 rounded-md bg-card p-2">
				{#each diff.evidence as ev}
					<p class="text-xs italic text-muted-foreground">"{ev}"</p>
				{/each}
			</div>
		{/if}
	{/if}

	<!-- Items -->
	<div class="flex flex-col gap-2">
		{#each diff.items as item, i}
			{@const OpIcon = opIcons[item.operation]}
			<div class={cn('flex items-start gap-2 rounded-md border border-border/50 bg-card p-2', item.protected && 'opacity-60')}>
				<div class={cn('flex size-5 shrink-0 items-center justify-center rounded', opColors[item.operation])}>
					<OpIcon class="size-3" />
				</div>
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<span class="text-xs font-medium">{item.label}</span>
						{#if item.protected}
							<ProtectedBadge />
						{/if}
					</div>
					<p class="text-xs text-muted-foreground">{item.path}</p>
					{#if item.operation === 'modify' && item.old_value !== undefined}
						<div class="mt-1 flex items-center gap-2 text-xs">
							<span class="rounded bg-bahaya-lembut px-1 text-bahaya line-through">{String(item.old_value)}</span>
							<span>→</span>
							<span class="rounded bg-berhasil-lembut px-1 text-berhasil">{String(item.new_value)}</span>
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>

	<!-- Actions -->
	<div class="flex flex-wrap gap-2 border-t border-border/50 pt-3">
		<Button size="sm" variant="default" onclick={() => ondiffaction?.('apply_all')}>
			Terapkan Semua
		</Button>
		<Button size="sm" variant="outline" onclick={() => ondiffaction?.('review')}>
			Tinjau Satu-satu
		</Button>
		<Button size="sm" variant="ghost" onclick={() => ondiffaction?.('dismiss')}>
			Abaikan
		</Button>
	</div>
</div>

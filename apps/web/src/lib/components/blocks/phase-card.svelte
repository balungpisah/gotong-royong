<script lang="ts">
	import type { Phase } from '$lib/types';
	import { cn } from '$lib/utils';
	import { renderMarkdown } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge';
	import { SourceBadge } from '$lib/components/ui/source-badge';
	import Circle from '@lucide/svelte/icons/circle';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleX from '@lucide/svelte/icons/circle-x';
	import CircleMinus from '@lucide/svelte/icons/circle-minus';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';

	let { phase }: { phase: Phase } = $props();

	const statusVariant = (status: string) => {
		switch (status) {
			case 'completed': return 'success' as const;
			case 'active': return 'step' as const;
			case 'blocked': return 'danger' as const;
			case 'skipped': return 'secondary' as const;
			default: return 'step-future' as const;
		}
	};

	const statusIcons = {
		open: Circle,
		completed: CircleCheck,
		blocked: CircleX,
		skipped: CircleMinus,
		planned: Circle,
		active: Circle
	};

	const statusColors = {
		open: 'text-kayu',
		completed: 'text-berhasil',
		blocked: 'text-bahaya',
		skipped: 'text-batu',
		planned: 'text-batu',
		active: 'text-api'
	};
</script>

<div class="rounded-lg border border-border bg-card" data-slot="phase-card">
	<!-- Header -->
	<div class="flex items-center justify-between gap-2 border-b border-border/50 px-4 py-3">
		<div class="flex items-center gap-2">
			<h4 class="text-body font-bold text-foreground">{phase.title}</h4>
			<Badge variant={statusVariant(phase.status)} class="text-[10px]">{phase.status}</Badge>
		</div>
		<SourceBadge source={phase.source} />
	</div>

	<!-- Objective -->
	<div class="border-b border-border/30 px-4 py-2">
		<div class="prose prose-sm max-w-none text-small text-muted-foreground">
			{@html renderMarkdown(phase.objective)}
		</div>
	</div>

	<!-- Checkpoints -->
	<div class="flex flex-col gap-0 px-4 py-2">
		{#each phase.checkpoints as cp (cp.checkpoint_id)}
			{@const CpIcon = statusIcons[cp.status] || Circle}
			<div class="flex items-start gap-2 py-1.5">
				<CpIcon class={cn('mt-0.5 size-4 shrink-0', statusColors[cp.status] || 'text-batu')} />
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<span class={cn('text-body', cp.status === 'completed' && 'line-through text-muted-foreground')}>
							{cp.title}
						</span>
						{#if cp.evidence_required}
							<ShieldAlert class="size-3 text-peringatan" />
						{/if}
					</div>
					{#if cp.description}
						<p class="text-small text-muted-foreground">{cp.description}</p>
					{/if}
				</div>
				<SourceBadge source={cp.source} />
			</div>
		{/each}
	</div>
</div>

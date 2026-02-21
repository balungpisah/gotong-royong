<script lang="ts">
	import type { Phase } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Tooltip, TooltipContent, TooltipTrigger } from '$lib/components/ui/tooltip';

	let { phases, onphaseselect }: { phases: Phase[]; onphaseselect?: (phaseId: string) => void } = $props();

	const statusDotClass = (status: string) => {
		switch (status) {
			case 'completed': return 'bg-berhasil border-berhasil';
			case 'active': return 'bg-api border-api';
			case 'blocked': return 'bg-bahaya border-bahaya';
			case 'skipped': return 'bg-batu border-batu';
			default: return 'bg-transparent border-batu'; // planned, open
		}
	};

	const statusLineClass = (status: string) => {
		switch (status) {
			case 'completed': return 'bg-berhasil';
			default: return 'bg-batu/30';
		}
	};

</script>

<div class="flex items-center gap-0" data-slot="phase-breadcrumb" role="navigation" aria-label="Fase rencana">
	{#each phases as phase, i (phase.phase_id)}
		<!-- Connecting line (not before first) -->
		{#if i > 0}
			<div class={cn('h-0.5 flex-1 min-w-[20px] max-w-[60px]', statusLineClass(phases[i - 1].status))}></div>
		{/if}

		<!-- Phase dot -->
		<Tooltip>
			<TooltipTrigger>
				<button
					class={cn(
						'flex flex-col items-center gap-1.5',
						onphaseselect && 'cursor-pointer'
					)}
					onclick={() => onphaseselect?.(phase.phase_id)}
					aria-label={`${phase.title}: ${phase.status}`}
				>
					<div
						class={cn(
							'size-3.5 rounded-full border-2 transition-all',
							statusDotClass(phase.status),
							phase.status === 'active' && 'ring-2 ring-api/30 ring-offset-1 ring-offset-background'
						)}
					></div>
					<span class={cn(
						'max-w-[60px] truncate text-[11px] font-medium',
						phase.status === 'active' ? 'text-api' : phase.status === 'completed' ? 'text-berhasil' : 'text-muted-foreground'
					)}>
						{phase.title}
					</span>
				</button>
			</TooltipTrigger>
			<TooltipContent>
				<p class="font-semibold">{phase.title}</p>
				<p class="text-xs opacity-80">{phase.objective}</p>
				<p class="mt-1 text-xs capitalize">{phase.status} · {phase.checkpoints.length} titik periksa</p>
			</TooltipContent>
		</Tooltip>
	{/each}
</div>

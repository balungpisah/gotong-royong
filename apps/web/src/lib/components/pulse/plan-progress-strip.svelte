<script lang="ts">
	import type { Phase } from '$lib/types';

	interface Props {
		phases: Phase[];
		activePhaseIndex?: number;
		onPhaseClick?: (index: number) => void;
	}

	let { phases, activePhaseIndex = $bindable(0), onPhaseClick }: Props = $props();

	const totalCheckpoints = $derived(
		phases.reduce((sum, phase) => sum + phase.checkpoints.length, 0)
	);

	const completedCheckpoints = $derived(
		phases.reduce(
			(sum, phase) =>
				sum + phase.checkpoints.filter((cp) => cp.status === 'completed').length,
			0
		)
	);

	const progressPercent = $derived(
		totalCheckpoints > 0 ? Math.round((completedCheckpoints / totalCheckpoints) * 100) : 0
	);

	function getDotClass(phase: Phase, index: number): string {
		if (index === activePhaseIndex) {
			return 'bg-primary animate-pulse ring-2 ring-primary/20';
		}
		if (phase.status === 'completed') {
			return 'bg-berhasil';
		}
		if (phase.status === 'blocked') {
			return 'bg-bahaya';
		}
		return 'bg-muted-foreground/30';
	}

	function handleDotClick(index: number) {
		activePhaseIndex = index;
		onPhaseClick?.(index);
	}
</script>

{#if phases.length > 0}
	<div class="flex flex-col gap-1.5 px-3 py-2">
		<!-- Checkpoint progress bar -->
		<div class="flex items-center gap-2">
			<div class="bg-muted relative h-1.5 flex-1 overflow-hidden rounded-full">
				<div
					class="bg-primary h-full rounded-full transition-all duration-300"
					style="width: {progressPercent}%"
				></div>
			</div>
			<span class="text-muted-foreground shrink-0 text-[10px]">
				{completedCheckpoints}/{totalCheckpoints}
			</span>
		</div>

		<!-- Phase dots row -->
		<div class="flex items-start gap-3 overflow-x-auto">
			{#each phases as phase, index}
				<button
					type="button"
					class="flex flex-col items-center gap-0.5 focus:outline-none"
					onclick={() => handleDotClick(index)}
				>
					<div class="h-2.5 w-2.5 rounded-full transition-all duration-200 {getDotClass(phase, index)}"></div>
					<span class="text-muted-foreground max-w-[8ch] truncate text-[9px] leading-tight">
						{phase.title}
					</span>
				</button>
			{/each}
		</div>
	</div>
{/if}

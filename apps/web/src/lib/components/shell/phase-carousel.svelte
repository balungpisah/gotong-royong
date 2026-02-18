<script lang="ts">
	import type { Phase } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';
	import CheckCircle from '@lucide/svelte/icons/check-circle-2';
	import Circle from '@lucide/svelte/icons/circle';
	import Lock from '@lucide/svelte/icons/lock';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';

	interface Props {
		phases: Phase[];
		activeIndex?: number;
	}

	let { phases, activeIndex = $bindable(0) }: Props = $props();

	const currentPhase = $derived(phases[activeIndex] ?? null);

	const canGoPrev = $derived(activeIndex > 0);
	const canGoNext = $derived(activeIndex < phases.length - 1);

	function prev() {
		if (canGoPrev) activeIndex--;
	}
	function next() {
		if (canGoNext) activeIndex++;
	}

	// Map phase status to StatusIndicator status
	const phaseStatusMap: Record<string, StatusIndicatorStatus> = {
		planned: 'done',
		active: 'active',
		open: 'stalled',
		completed: 'done',
		blocked: 'moderation',
		skipped: 'sealed'
	};

	// Checkpoint icon helper
	function checkpointStatusIcon(status: string) {
		switch (status) {
			case 'completed':
				return CheckCircle;
			case 'blocked':
				return Lock;
			case 'active':
			case 'open':
				return AlertTriangle;
			default:
				return Circle;
		}
	}

	// Touch / swipe support
	let touchStartX = $state(0);
	let touchDelta = $state(0);
	let isSwiping = $state(false);

	function onTouchStart(e: TouchEvent) {
		touchStartX = e.touches[0].clientX;
		touchDelta = 0;
		isSwiping = true;
	}

	function onTouchMove(e: TouchEvent) {
		if (!isSwiping) return;
		touchDelta = e.touches[0].clientX - touchStartX;
	}

	function onTouchEnd() {
		if (!isSwiping) return;
		isSwiping = false;
		const threshold = 60;
		if (touchDelta > threshold && canGoPrev) {
			prev();
		} else if (touchDelta < -threshold && canGoNext) {
			next();
		}
		touchDelta = 0;
	}
</script>

<div class="flex flex-col gap-3">
	<!-- Phase navigation header -->
	<div class="flex items-center gap-2">
		<button
			onclick={prev}
			disabled={!canGoPrev}
			class="flex size-7 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted disabled:opacity-30"
			aria-label="Fase sebelumnya"
		>
			<ChevronLeft class="size-4" />
		</button>

		<!-- Phase dots -->
		<div class="flex flex-1 items-center justify-center gap-1.5">
			{#each phases as phase, i (phase.phase_id)}
				<button
					onclick={() => (activeIndex = i)}
					class="group flex items-center gap-1 rounded-full px-1.5 py-0.5 transition {i === activeIndex
						? 'bg-primary/10'
						: 'hover:bg-muted/60'}"
					aria-label="Fase {i + 1}: {phase.title}"
					aria-current={i === activeIndex ? 'step' : undefined}
				>
					<span
						class="size-2 rounded-full transition {i === activeIndex
							? 'bg-primary'
							: phase.status === 'completed'
								? 'bg-berhasil/60'
								: 'bg-muted-foreground/30'}"
					></span>
					{#if i === activeIndex}
						<span class="text-[10px] font-medium text-primary">
							{i + 1}/{phases.length}
						</span>
					{/if}
				</button>
			{/each}
		</div>

		<button
			onclick={next}
			disabled={!canGoNext}
			class="flex size-7 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted disabled:opacity-30"
			aria-label="Fase berikutnya"
		>
			<ChevronRight class="size-4" />
		</button>
	</div>

	<!-- Phase content -->
	{#if currentPhase}
		<div
			class="rounded-xl border border-border/60 bg-card p-4"
			ontouchstart={onTouchStart}
			ontouchmove={onTouchMove}
			ontouchend={onTouchEnd}
			role="tabpanel"
			tabindex="0"
			aria-label="Fase: {currentPhase.title}"
		>
			<!-- Phase title & status -->
			<div class="flex items-start justify-between gap-2">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<StatusIndicator status={phaseStatusMap[currentPhase.status] ?? 'active'} />
						<h3 class="truncate text-sm font-semibold text-foreground">
							{currentPhase.title}
						</h3>
					</div>
					<p class="mt-1 text-xs leading-relaxed text-muted-foreground">
						{currentPhase.objective}
					</p>
				</div>
				<Badge
					variant={currentPhase.status === 'completed'
						? 'success'
						: currentPhase.status === 'active'
							? 'default'
							: currentPhase.status === 'blocked'
								? 'danger'
								: 'secondary'}
					class="shrink-0 text-[10px]"
				>
					{currentPhase.status}
				</Badge>
			</div>

			<!-- Checkpoints -->
			{#if currentPhase.checkpoints.length > 0}
				<div class="mt-3 border-t border-border/40 pt-3">
					<ul class="flex flex-col gap-2">
						{#each currentPhase.checkpoints as cp (cp.checkpoint_id)}
							{@const Icon = checkpointStatusIcon(cp.status)}
							<li class="flex items-start gap-2 text-xs">
								<Icon
									class="mt-0.5 size-3.5 shrink-0 {cp.status === 'completed'
										? 'text-berhasil'
										: cp.status === 'blocked'
											? 'text-bahaya'
											: cp.status === 'active' || cp.status === 'open'
												? 'text-peringatan'
												: 'text-muted-foreground/50'}"
								/>
								<div class="min-w-0 flex-1">
									<span
										class={cp.status === 'completed'
											? 'text-muted-foreground line-through'
											: 'text-foreground'}
									>
										{cp.title}
									</span>
									{#if cp.description}
										<p class="mt-0.5 text-[10px] text-muted-foreground">
											{cp.description}
										</p>
									{/if}
								</div>
								{#if cp.evidence_required}
									<Badge variant="warning" class="shrink-0 text-[9px]">bukti</Badge>
								{/if}
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</div>
	{/if}
</div>

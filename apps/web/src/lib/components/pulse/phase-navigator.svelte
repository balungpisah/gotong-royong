<script lang="ts">
	import { slide, fly } from 'svelte/transition';
	import type { Phase } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import CheckCircle from '@lucide/svelte/icons/check-circle-2';
	import Circle from '@lucide/svelte/icons/circle';
	import Lock from '@lucide/svelte/icons/lock';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import ClipboardCheck from '@lucide/svelte/icons/clipboard-check';
	import MapPin from '@lucide/svelte/icons/map-pin';

	interface Props {
		phases: Phase[];
		/** Initially focused phase index. Defaults to the active phase. */
		initialIndex?: number;
	}

	let { phases, initialIndex }: Props = $props();

	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	let expanded = $state(false);
	/** 1 = moving right (next), -1 = moving left (prev) */
	let direction = $state(1);

	// Find the active phase or default to 0
	const defaultIndex = $derived.by(() => {
		if (initialIndex != null) return initialIndex;
		const idx = phases.findIndex((p) => p.status === 'active');
		return idx >= 0 ? idx : 0;
	});

	let focusedIndex = $state(0);

	// Sync focusedIndex when phases/detail changes
	$effect(() => {
		focusedIndex = defaultIndex;
	});

	function toggle() {
		expanded = !expanded;
	}

	// ---------------------------------------------------------------------------
	// Derived: focused phase + stats
	// ---------------------------------------------------------------------------

	const focusedPhase = $derived(phases[focusedIndex] ?? null);

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

	// Focused phase checkpoint stats
	const focusedCompleted = $derived(
		focusedPhase?.checkpoints.filter((cp) => cp.status === 'completed').length ?? 0
	);
	const focusedTotal = $derived(focusedPhase?.checkpoints.length ?? 0);

	// Action counts for the handlebar hint
	const blockedCount = $derived(
		phases.flatMap((p) => p.checkpoints ?? []).filter((c) => c.status === 'blocked').length
	);
	const evidenceCount = $derived(
		phases
			.flatMap((p) => p.checkpoints ?? [])
			.filter((c) => c.evidence_required === true && c.status !== 'completed').length
	);

	// ---------------------------------------------------------------------------
	// Handlebar text
	// ---------------------------------------------------------------------------

	const handlebarTitle = $derived.by(() => {
		if (!focusedPhase) return m.pulse_nav_no_plan();
		return focusedPhase.title;
	});

	const handlebarMeta = $derived.by(() => {
		if (!focusedPhase) return '';
		const parts: string[] = [];
		parts.push(`${focusedCompleted}/${focusedTotal}`);
		if (blockedCount > 0) parts.push(`${blockedCount} ${m.pulse_nav_blocked()}`);
		if (evidenceCount > 0) parts.push(`${evidenceCount} ${m.pulse_nav_evidence()}`);
		return parts.join(' · ');
	});

	const handlebarBadgeVariant = $derived.by(() => {
		if (!focusedPhase) return 'secondary' as const;
		switch (focusedPhase.status) {
			case 'completed':
				return 'success' as const;
			case 'active':
				return 'default' as const;
			case 'blocked':
				return 'danger' as const;
			default:
				return 'secondary' as const;
		}
	});

	// ---------------------------------------------------------------------------
	// Navigation
	// ---------------------------------------------------------------------------

	function goLeft() {
		if (focusedIndex > 0) {
			direction = -1;
			focusedIndex--;
		}
	}

	function goRight() {
		if (focusedIndex < phases.length - 1) {
			direction = 1;
			focusedIndex++;
		}
	}

	// Swipe support
	let touchStartX = $state(0);

	function handleTouchStart(e: TouchEvent) {
		touchStartX = e.touches[0].clientX;
	}

	function handleTouchEnd(e: TouchEvent) {
		const dx = e.changedTouches[0].clientX - touchStartX;
		if (Math.abs(dx) > 50) {
			// Swipe right → go to previous (left), swipe left → go to next (right)
			if (dx > 0) goLeft();
			else goRight();
		}
	}

	// ---------------------------------------------------------------------------
	// Checkpoint rendering helpers
	// ---------------------------------------------------------------------------

	function getCheckpointIconAndColor(status: string): {
		icon: typeof Circle;
		colorClass: string;
	} {
		switch (status) {
			case 'completed':
				return { icon: CheckCircle, colorClass: 'text-berhasil' };
			case 'blocked':
				return { icon: Lock, colorClass: 'text-bahaya' };
			case 'active':
			case 'open':
				return { icon: AlertTriangle, colorClass: 'text-peringatan' };
			default:
				return { icon: Circle, colorClass: 'text-muted-foreground/50' };
		}
	}

	function getDotClass(phase: Phase, index: number): string {
		if (index === focusedIndex) {
			return 'bg-primary ring-2 ring-primary/30 scale-125';
		}
		if (phase.status === 'completed') {
			return 'bg-berhasil';
		}
		if (phase.status === 'blocked') {
			return 'bg-bahaya';
		}
		return 'bg-muted-foreground/30';
	}
</script>

{#if phases.length > 0}
	<div class="flex flex-col" data-slot="phase-navigator">
		<!-- Handlebar — always visible -->
		<button
			type="button"
			onclick={toggle}
			class="flex w-full items-center gap-2 rounded-lg border border-border/40 bg-muted/20 px-3 py-1.5 text-left transition hover:bg-muted/40"
		>
			<MapPin class="size-3.5 shrink-0 text-muted-foreground" />
			<span class="min-w-0 flex-1 truncate text-xs font-medium text-foreground">
				{handlebarTitle}
			</span>
			{#if handlebarMeta}
				<span class="shrink-0 text-[10px] text-muted-foreground">{handlebarMeta}</span>
			{/if}
			<Badge variant={handlebarBadgeVariant} class="shrink-0 text-[9px]">
				{m.pulse_nav_phase({ num: focusedIndex + 1 })} {m.pulse_nav_of_total({ total: phases.length })}
			</Badge>
			<ChevronDown
				class="size-3.5 shrink-0 text-muted-foreground transition-transform duration-150 {expanded
					? 'rotate-180'
					: ''}"
			/>
		</button>

		<!-- Expanded: horizontal phase carousel -->
		{#if expanded}
			<div class="pt-1" transition:slide={{ duration: 150 }}>
				<div class="rounded-lg border border-border/40 bg-muted/10">
					<!-- Overall progress bar -->
					<div class="flex items-center gap-2 px-3 pt-2">
						<div class="relative h-1 flex-1 overflow-hidden rounded-full bg-muted">
							<div
								class="h-full rounded-full bg-primary transition-all duration-300"
								style="width: {progressPercent}%"
							></div>
						</div>
						<span class="shrink-0 text-[10px] text-muted-foreground">
							{completedCheckpoints}/{totalCheckpoints}
						</span>
					</div>

					<!-- Phase content area with swipe -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="relative overflow-hidden px-3 py-2"
						ontouchstart={handleTouchStart}
						ontouchend={handleTouchEnd}
					>
						<!-- Navigation arrows -->
						{#if focusedIndex > 0}
							<button
								type="button"
								onclick={goLeft}
								class="absolute left-0 top-1/2 z-10 flex size-6 -translate-y-1/2 items-center justify-center rounded-full bg-muted/60 text-muted-foreground transition hover:bg-muted"
							>
								<ChevronLeft class="size-3.5" />
							</button>
						{/if}

						{#if focusedIndex < phases.length - 1}
							<button
								type="button"
								onclick={goRight}
								class="absolute right-0 top-1/2 z-10 flex size-6 -translate-y-1/2 items-center justify-center rounded-full bg-muted/60 text-muted-foreground transition hover:bg-muted"
							>
								<ChevronRight class="size-3.5" />
							</button>
						{/if}

						<!-- Current phase card — slides in horizontally, height adapts naturally -->
						{#if focusedPhase}
							{#key focusedIndex}
								<div
									class="mx-4 transition-[min-height] duration-200 ease-out"
									in:fly={{ x: direction * 200, duration: 200 }}
								>
									<!-- Phase title + status -->
									<div class="mb-1.5 flex items-center gap-2">
										<span class="text-xs font-semibold text-foreground">
											{focusedPhase.title}
										</span>
										<Badge variant={handlebarBadgeVariant} class="text-[9px]">
											{focusedPhase.status}
										</Badge>
									</div>

									<!-- Objective -->
									{#if focusedPhase.objective}
										<p class="mb-2 text-[11px] italic text-muted-foreground">
											{focusedPhase.objective}
										</p>
									{/if}

									<!-- Checkpoints list -->
									{#if focusedPhase.checkpoints.length > 0}
										<ul class="flex flex-col gap-1.5">
											{#each focusedPhase.checkpoints as cp}
												{@const { icon: Icon, colorClass } = getCheckpointIconAndColor(cp.status)}
												{@const completed = cp.status === 'completed'}
												<li class="flex items-start gap-2 text-xs">
													<Icon class="mt-0.5 size-3.5 shrink-0 {colorClass}" />
													<div class="min-w-0 flex-1">
														<span
															class={completed
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
													{#if cp.evidence_required && cp.status !== 'completed'}
														<Badge variant="warning" class="shrink-0 text-[9px]">
															<ClipboardCheck class="mr-0.5 size-2.5" />
															{m.pulse_nav_evidence()}
														</Badge>
													{/if}
												</li>
											{/each}
										</ul>
									{/if}
								</div>
							{/key}
						{/if}
					</div>

					<!-- Phase dot indicators -->
					<div class="flex items-center justify-center gap-2 px-3 pb-2">
						{#each phases as phase, index}
							<button
								type="button"
								class="flex size-5 items-center justify-center focus:outline-none"
								onclick={() => {
								if (index !== focusedIndex) {
									direction = index > focusedIndex ? 1 : -1;
									focusedIndex = index;
								}
							}}
								aria-label={phase.title}
							>
								<div
									class="h-2 w-2 rounded-full transition-all duration-200 {getDotClass(phase, index)}"
								></div>
							</button>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>
{/if}

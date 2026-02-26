<script lang="ts">
	import { safeFly as fly } from '$lib/utils/safe-slide';
	import { safeSlide as slide } from '$lib/utils/safe-slide';
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
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Trophy from '@lucide/svelte/icons/trophy';
	import Eye from '@lucide/svelte/icons/eye';
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import User from '@lucide/svelte/icons/user';
	import Bot from '@lucide/svelte/icons/bot';
	import Settings from '@lucide/svelte/icons/settings';
	import { Button } from '$lib/components/ui/button';

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
			(sum, phase) => sum + phase.checkpoints.filter((cp) => cp.status === 'completed').length,
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
	const focusedPercent = $derived(
		focusedTotal > 0 ? Math.round((focusedCompleted / focusedTotal) * 100) : 0
	);

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
	// CD1: Epic Meaning — chapter-style narrative
	// ---------------------------------------------------------------------------

	/** Story chapter label — "Bab 1 dari 3" */
	const chapterLabel = $derived(`Bab ${focusedIndex + 1} dari ${phases.length}`);

	/** Narrative status line — gives emotional weight to the current phase state */
	const narrativeStatus = $derived.by(() => {
		if (!focusedPhase) return '';
		switch (focusedPhase.status) {
			case 'completed':
				return '✨ Bab ini sudah dituntaskan';
			case 'active':
				return '📖 Sedang berjalan…';
			case 'blocked':
				return '🚧 Butuh aksi untuk melanjutkan';
			case 'planned':
				return '🔮 Menanti giliran';
			default:
				return '';
		}
	});

	// ---------------------------------------------------------------------------
	// CD7: Curiosity — is the focused phase a future/locked one?
	// ---------------------------------------------------------------------------

	const isFuturePhase = $derived(
		focusedPhase?.status === 'planned' || focusedPhase?.status === 'skipped'
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

	/** CD4: Ownership — source tag icon */
	function getSourceIcon(source: string): typeof User {
		switch (source) {
			case 'human':
				return User;
			case 'ai':
				return Bot;
			case 'system':
				return Settings;
			default:
				return Circle;
		}
	}

	function getSourceLabel(source: string): string {
		switch (source) {
			case 'human':
				return 'Dari warga';
			case 'ai':
				return 'Saran AI';
			case 'system':
				return 'Sistem';
			default:
				return source;
		}
	}

	/** Phase dot label for story dots — shows chapter number or check mark */
	function getDotContent(phase: Phase, index: number): string {
		if (phase.status === 'completed') return '✓';
		return `${index + 1}`;
	}
</script>

{#if phases.length > 0}
	<div class="flex flex-col" data-slot="phase-navigator">
		<!-- Handlebar — always visible -->
		<Button
			variant="ghost"
			onclick={toggle}
			class="flex w-full items-center gap-2 rounded-lg border border-border/40 bg-muted/20 px-3 py-1.5 text-left transition hover:bg-muted/40 h-auto"
		>
			<MapPin class="size-3.5 shrink-0 text-muted-foreground" />
			<span class="min-w-0 flex-1 truncate text-small font-medium text-foreground">
				{handlebarTitle}
			</span>
			{#if handlebarMeta}
				<span class="shrink-0 text-small text-muted-foreground">{handlebarMeta}</span>
			{/if}
			<Badge variant={handlebarBadgeVariant} class="shrink-0 text-[10px]">
				{m.pulse_nav_phase({ num: focusedIndex + 1 })}
				{m.pulse_nav_of_total({ total: phases.length })}
			</Badge>
			<ChevronDown
				class="size-3.5 shrink-0 text-muted-foreground transition-transform duration-150 {expanded
					? 'rotate-180'
					: ''}"
			/>
		</Button>

		<!-- Expanded: story-style phase carousel -->
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
						<span class="shrink-0 text-small text-muted-foreground">
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
							<Button
								variant="ghost"
								size="icon-sm"
								onclick={goLeft}
								class="absolute left-0 top-1/2 z-10 -translate-y-1/2 rounded-full bg-muted/60 text-muted-foreground hover:bg-muted size-6"
							>
								<ChevronLeft class="size-3.5" />
							</Button>
						{/if}

						{#if focusedIndex < phases.length - 1}
							<Button
								variant="ghost"
								size="icon-sm"
								onclick={goRight}
								class="absolute right-0 top-1/2 z-10 -translate-y-1/2 rounded-full bg-muted/60 text-muted-foreground hover:bg-muted size-6"
							>
								<ChevronRight class="size-3.5" />
							</Button>
						{/if}

						<!-- Current phase card — slides in horizontally -->
						{#if focusedPhase}
							{#key focusedIndex}
								<div
									class="mx-4 transition-[min-height] duration-200 ease-out"
									in:fly={{ x: direction * 200, duration: 200 }}
								>
									<!-- CD1: Chapter header with narrative framing -->
									<div class="mb-1 flex items-center gap-1.5">
										<span class="font-['Caveat'] text-small text-muted-foreground/60">
											{chapterLabel}
										</span>
									</div>

									<!-- Phase title + status -->
									<div class="mb-1 flex items-center gap-2">
										{#if focusedPhase.status === 'completed'}
											<Trophy class="size-3.5 text-berhasil" />
										{:else if isFuturePhase}
											<EyeOff class="size-3.5 text-muted-foreground/40" />
										{:else}
											<Eye class="size-3.5 text-primary" />
										{/if}
										<span class="text-small font-semibold text-foreground">
											{focusedPhase.title}
										</span>
										<Badge variant={handlebarBadgeVariant} class="text-[10px]">
											{focusedPhase.status}
										</Badge>
									</div>

									<!-- Narrative status line -->
									{#if narrativeStatus}
										<p class="mb-1.5 text-small text-muted-foreground/70">
											{narrativeStatus}
										</p>
									{/if}

									<!-- Objective -->
									{#if focusedPhase.objective}
										<p
											class="mb-2 text-small italic text-muted-foreground {isFuturePhase
												? 'blur-[2px] select-none'
												: ''}"
										>
											{focusedPhase.objective}
										</p>
									{/if}

									<!-- CD2: Per-phase mini progress bar -->
									{#if focusedTotal > 0 && !isFuturePhase}
										<div class="mb-2 flex items-center gap-2">
											<div class="relative h-1 flex-1 overflow-hidden rounded-full bg-muted">
												<div
													class="h-full rounded-full transition-all duration-300 {focusedPhase.status ===
													'completed'
														? 'bg-berhasil'
														: 'bg-primary'}"
													style="width: {focusedPercent}%"
												></div>
											</div>
											<span class="text-[10px] tabular-nums text-muted-foreground">
												{focusedCompleted}/{focusedTotal}
											</span>
										</div>
									{/if}

									<!-- CD2: Celebration banner for completed phases -->
									{#if focusedPhase.status === 'completed'}
										<div class="mb-2 flex items-center gap-1.5 rounded-md bg-berhasil/10 px-2 py-1">
											<Sparkles class="size-3 text-berhasil animate-pulse" />
											<span class="text-small font-medium text-berhasil">
												Bab ini berhasil dituntaskan oleh warga!
											</span>
										</div>
									{/if}

									<!-- Checkpoints list — with CD4 source ownership tags -->
									{#if focusedPhase.checkpoints.length > 0 && !isFuturePhase}
										<ul class="flex flex-col gap-1.5">
											{#each focusedPhase.checkpoints as cp}
												{@const { icon: Icon, colorClass } = getCheckpointIconAndColor(cp.status)}
												{@const SourceIcon = getSourceIcon(cp.source)}
												{@const completed = cp.status === 'completed'}
												<li class="flex items-start gap-2 text-small">
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
															<p class="mt-0.5 text-small text-muted-foreground">
																{cp.description}
															</p>
														{/if}
													</div>
													<!-- CD4: Source ownership tag -->
													<span
														class="mt-0.5 inline-flex shrink-0 items-center gap-0.5 text-[10px] text-muted-foreground/50"
														title={getSourceLabel(cp.source)}
													>
														<SourceIcon class="size-2.5" />
													</span>
													{#if cp.evidence_required && cp.status !== 'completed'}
														<Badge variant="warning" class="shrink-0 text-[10px]">
															<ClipboardCheck class="mr-0.5 size-2.5" />
															{m.pulse_nav_evidence()}
														</Badge>
													{/if}
												</li>
											{/each}
										</ul>
									{/if}

									<!-- CD7: Future phase mystery view -->
									{#if isFuturePhase}
										<div class="flex flex-col items-center gap-1 py-3 text-center">
											<Lock class="size-5 text-muted-foreground/30" />
											<span class="text-small text-muted-foreground/50">
												Selesaikan bab sebelumnya untuk membuka
											</span>
											{#if focusedTotal > 0}
												<span class="text-[10px] text-muted-foreground/30">
													{focusedTotal} langkah menanti
												</span>
											{/if}
										</div>
									{/if}
								</div>
							{/key}
						{/if}
					</div>

					<!-- Story dot indicators — chapter-style numbered dots -->
					<div class="flex items-center justify-center gap-1 px-3 pb-2">
						{#each phases as phase, index}
							<button
								type="button"
								class="flex items-center justify-center focus:outline-none"
								onclick={() => {
									if (index !== focusedIndex) {
										direction = index > focusedIndex ? 1 : -1;
										focusedIndex = index;
									}
								}}
								aria-label={phase.title}
							>
								<div
									class="flex size-5 items-center justify-center rounded-full text-[10px] font-bold transition-all duration-200 {index ===
									focusedIndex
										? 'bg-primary text-primary-foreground ring-2 ring-primary/30 scale-110'
										: phase.status === 'completed'
											? 'bg-berhasil/20 text-berhasil'
											: phase.status === 'blocked'
												? 'bg-bahaya/20 text-bahaya'
												: 'bg-muted text-muted-foreground/50'}"
								>
									{getDotContent(phase, index)}
								</div>
								<!-- Connector line between dots -->
								{#if index < phases.length - 1}
									<div
										class="mx-0.5 h-px w-3 {phase.status === 'completed'
											? 'bg-berhasil/40'
											: 'bg-border/40'}"
									></div>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>
{/if}

<script lang="ts">
	import type { WitnessDetail, FeedItem } from '$lib/types';
	import type { Phase } from '$lib/types/path-plan';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';
	import WitnessChatPanel from './witness-chat-panel.svelte';
	import EntityPill from './entity-pill.svelte';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import type { TandangAvatarPerson } from '$lib/types';
	import X from '@lucide/svelte/icons/x';
	import UsersIcon from '@lucide/svelte/icons/users';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import Clock from '@lucide/svelte/icons/clock';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import Check from '@lucide/svelte/icons/check';
	import Circle from '@lucide/svelte/icons/circle';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Lock from '@lucide/svelte/icons/lock';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import { safeSlide as slide } from '$lib/utils/safe-slide';

	interface Props {
		detail: WitnessDetail;
		feedItem?: FeedItem | null;
		onClose?: () => void;
		onSendMessage?: (content: string, attachments?: File[]) => void;
		onStempel?: () => void;
		sending?: boolean;
		stempeling?: boolean;
	}

	let { detail, feedItem = null, onClose, onSendMessage, onStempel, sending = false, stempeling = false }: Props = $props();

	// ---------------------------------------------------------------------------
	// Maps
	// ---------------------------------------------------------------------------

	const statusMap: Record<string, StatusIndicatorStatus> = {
		draft: 'done',
		open: 'stalled',
		active: 'active',
		resolved: 'done',
		closed: 'sealed'
	};

	const trackVariantMap: Record<string, BadgeVariant> = {
		tuntaskan: 'track-tuntaskan',
		wujudkan: 'track-wujudkan',
		telusuri: 'track-telusuri',
		rayakan: 'track-rayakan',
		musyawarah: 'track-musyawarah'
	};

	// ---------------------------------------------------------------------------
	// Pinned card — sentiment color
	// ---------------------------------------------------------------------------

	const sentimentColorMap: Record<string, string> = {
		angry:       'var(--c-bahaya)',
		hopeful:     'var(--c-berhasil)',
		urgent:      'var(--c-peringatan)',
		celebratory: 'var(--t-rayakan)',
		sad:         'var(--v-mid)',
		curious:     'var(--t-telusuri)',
		fun:         'var(--c-api-terang)'
	};

	const trackColorMap: Record<string, string> = {
		tuntaskan:  'var(--t-tuntaskan)',
		wujudkan:   'var(--t-wujudkan)',
		telusuri:   'var(--t-telusuri)',
		rayakan:    'var(--t-rayakan)',
		musyawarah: 'var(--t-musyawarah)'
	};

	const accentColor = $derived(
		feedItem?.sentiment
			? (sentimentColorMap[feedItem.sentiment] ?? 'var(--c-batu)')
			: feedItem?.track_hint
				? (trackColorMap[feedItem.track_hint] ?? 'var(--c-batu)')
				: 'var(--c-batu)'
	);

	const eventEmojiMap: Record<string, string> = {
		created:          '📢',
		joined:           '🙋',
		checkpoint:       '📍',
		vote_opened:      '🗳️',
		evidence:         '📎',
		resolved:         '✅',
		galang_milestone: '💰',
		community_note:   '📝'
	};

	// ---------------------------------------------------------------------------
	// Derived: phases from main branch
	// ---------------------------------------------------------------------------

	const phases = $derived(detail.plan?.branches?.[0]?.phases ?? []);

	const currentPhaseIndex = $derived(
		Math.max(0, phases.findIndex((p) =>
			p.checkpoints.some((c) => c.status !== 'completed')
		))
	);

	// ---------------------------------------------------------------------------
	// Expansion drawer — which item is expanded below the card
	// 'overview' | phase index | null (collapsed)
	// ---------------------------------------------------------------------------

	let expandedItem = $state<'overview' | number | null>(null);
	let phaseScrollEl: HTMLDivElement | undefined = $state();

	// Auto-scroll to center the active phase row on mount
	$effect(() => {
		if (phaseScrollEl && currentPhaseIndex >= 0) {
			requestAnimationFrame(() => {
				const activeRow = phaseScrollEl?.querySelector('[data-phase-current]') as HTMLElement | null;
				if (activeRow && phaseScrollEl) {
					const top = activeRow.offsetTop - phaseScrollEl.offsetTop;
					const center = top - phaseScrollEl.clientHeight / 2 + activeRow.clientHeight / 2;
					phaseScrollEl.scrollTo({ top: Math.max(0, center), behavior: 'smooth' });
				}
			});
		}
	});

	function toggleItem(item: 'overview' | number) {
		expandedItem = expandedItem === item ? null : item;
	}

	// Phase status helpers
	function phaseIcon(phase: Phase, index: number) {
		if (phase.status === 'completed') return 'completed';
		if (phase.status === 'blocked') return 'blocked';
		if (index === currentPhaseIndex) return 'active';
		return 'pending';
	}

	function checkpointCounts(phase: Phase) {
		const total = phase.checkpoints.length;
		const done = phase.checkpoints.filter((c) => c.status === 'completed').length;
		const blocked = phase.checkpoints.filter((c) => c.status === 'blocked').length;
		return { total, done, blocked };
	}

	// ---------------------------------------------------------------------------
	// Member count / message count for overview
	// ---------------------------------------------------------------------------

	const memberCount = $derived((detail.members ?? []).length);

	/** The pelapor (initiator) — shown in the header as TandangAvatar */
	const initiator = $derived.by((): TandangAvatarPerson | null => {
		const pelapor = detail.members?.find((m) => m.role === 'pelapor');
		if (!pelapor) return null;
		return {
			user_id: pelapor.user_id,
			name: pelapor.name,
			avatar_url: pelapor.avatar_url,
			tier: pelapor.tier as TandangAvatarPerson['tier'],
			role: pelapor.role
		};
	});

	// ---------------------------------------------------------------------------
	// Phase momentum — completed count + active phase checkpoint progress
	// ---------------------------------------------------------------------------

	const completedPhases = $derived(phases.filter((p) => p.status === 'completed').length);
	const activePhase = $derived(phases[currentPhaseIndex] ?? null);
	const activeCheckpointsDone = $derived(
		activePhase ? activePhase.checkpoints.filter((c) => c.status === 'completed').length : 0
	);
	const activeCheckpointsTotal = $derived(activePhase ? activePhase.checkpoints.length : 0);

	// ---------------------------------------------------------------------------
	// Time since update
	// ---------------------------------------------------------------------------

	const timeSince = $derived.by(() => {
		const diff = Date.now() - new Date(detail.updated_at).getTime();
		const mins = Math.floor(diff / 60_000);
		if (mins < 1) return 'baru saja';
		if (mins < 60) return `${mins}m lalu`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}j lalu`;
		const days = Math.floor(hours / 24);
		return `${days}h lalu`;
	});

	// ---------------------------------------------------------------------------
	// Rahasia level
	// ---------------------------------------------------------------------------

	const rahasiaDisplay = $derived.by(() => {
		switch (detail.rahasia_level) {
			case 'L1':
				return { show: true, label: 'Rahasia', variant: 'danger' as BadgeVariant };
			case 'L2':
				return { show: true, label: 'Sangat Rahasia', variant: 'danger' as BadgeVariant };
			default:
				return { show: false, label: '', variant: 'secondary' as BadgeVariant };
		}
	});
</script>

{#key detail.witness_id}
<div class="flex h-full flex-col overflow-hidden" style="--accent: {accentColor};">
	<!-- ================================================================== -->
	<!-- PINNED CARD — two-column: identity left, phase nav right           -->
	<!-- ================================================================== -->
	<div class="detail-header relative z-10 shrink-0 border-y border-border/60">

		{#if feedItem}
			<div class="pinned-card">
				<!-- Close button — top right -->
				{#if onClose}
					<button
						onclick={onClose}
						class="absolute top-1.5 right-1.5 z-10 flex size-5 items-center justify-center rounded-md text-muted-foreground/70 transition hover:bg-muted hover:text-foreground"
						aria-label="Tutup panel"
					>
						<X class="size-3" />
					</button>
				{/if}

				<div class="pinned-columns">
					<!-- LEFT: identity — flex-col so initiator pins to bottom, aligned with last phase row -->
					<div class="min-w-0 flex-1 flex flex-col gap-1">
						<div class="flex items-start gap-1.5 pr-4">
							<span class="mt-0.5 text-sm select-none opacity-60">
								{eventEmojiMap[feedItem.latest_event.event_type] ?? '📌'}
							</span>
							<div class="min-w-0 flex-1">
								{#if feedItem.hook_line}
									<p class="text-sm font-bold leading-snug text-foreground line-clamp-2">
										{feedItem.hook_line}
									</p>
									<p class="mt-0.5 text-xs leading-snug text-muted-foreground/70 line-clamp-1">
										{feedItem.title}
									</p>
								{:else}
									<p class="text-sm font-bold leading-snug text-foreground line-clamp-2">
										{feedItem.title}
									</p>
								{/if}
							</div>
						</div>

						{#if feedItem.entity_tags.length > 0}
							<div class="flex flex-wrap items-center gap-1 opacity-80">
								{#each feedItem.entity_tags as tag (tag.entity_id)}
									<EntityPill {tag} />
								{/each}
							</div>
						{/if}

						<!-- Initiator signature — mt-auto pins to bottom, aligning with last phase row on right -->
						<div class="flex items-center gap-1.5 mt-auto pt-1">
							<StatusIndicator status={statusMap[detail.status] ?? 'active'} />
							{#if initiator}
								<a
									href="/profil/{initiator.user_id}"
									class="inline-flex items-center gap-1.5 rounded-md hover:bg-muted/30 px-1 -mx-1 transition-colors"
									aria-label="Profil {initiator.name}"
									data-initiator-profile-link
									data-profile-user-id={initiator.user_id}
								>
									<TandangAvatar person={initiator} size="xs" showTierDot={false} interactive={false} />
									<span class="text-[13px] text-muted-foreground/60 truncate max-w-[160px]" style="font-family: 'Caveat', cursive;">— {initiator.name}</span>
								</a>
							{/if}
							{#if rahasiaDisplay.show}
								<Badge variant={rahasiaDisplay.variant} class="text-caption">
									<ShieldAlert class="mr-0.5 size-2.5" />
									{rahasiaDisplay.label}
								</Badge>
							{/if}
						</div>
					</div>

					<!-- DIVIDER -->
					<div class="mx-2 w-px self-stretch" style="background: color-mix(in srgb, var(--accent) 25%, var(--color-border));"></div>

					<!-- RIGHT: phase list nav -->
					<div class="phase-list-col flex flex-col">
						<!-- Sticky overview row -->
						<button
							class="phase-row phase-row-sticky sticky top-0 z-10"
							class:phase-row-active={expandedItem === 'overview'}
							onclick={() => toggleItem('overview')}
						>
							<span class="flex items-center gap-1.5">
								<UsersIcon class="size-3 text-muted-foreground" />
								<span class="text-xs font-semibold text-foreground">Ikhtisar</span>
							</span>
							<span class="flex items-center gap-1.5 text-xs text-muted-foreground/60">
								<span>👥 {memberCount}</span>
								<span>💬 {detail.message_count}</span>
								<ChevronDown
									class="size-3 transition-transform {expandedItem === 'overview' ? 'rotate-180' : ''}"
								/>
							</span>
						</button>

						<!-- Phase items — scrollable -->
						<div class="phase-scroll flex-1 overflow-y-auto" bind:this={phaseScrollEl}>
							{#each phases as phase, i (phase.phase_id)}
								{@const icon = phaseIcon(phase, i)}
								{@const counts = checkpointCounts(phase)}
								<button
									class="phase-row"
									class:phase-row-active={expandedItem === i}
									class:phase-row-current={i === currentPhaseIndex}
									data-phase-current={i === currentPhaseIndex ? '' : undefined}
									onclick={() => toggleItem(i)}
								>
									<span class="flex items-center gap-1.5">
										{#if icon === 'completed'}
											<Check class="size-3 text-berhasil" />
										{:else if icon === 'blocked'}
											<Lock class="size-3 text-bahaya" />
										{:else if icon === 'active'}
											<CircleDot class="size-3 text-primary" />
										{:else}
											<Circle class="size-3 text-muted-foreground/40" />
										{/if}
										<span class="truncate text-xs {i === currentPhaseIndex ? 'font-semibold text-foreground' : 'text-muted-foreground'}">
											{phase.title}
										</span>
									</span>
									<span class="shrink-0 text-caption tabular-nums text-muted-foreground/50">
										{counts.done}/{counts.total}
									</span>
								</button>
							{/each}
						</div>
					</div>
				</div>
			</div>
		{:else}
			<!-- Fallback header when no feed item available (e.g. deep link) -->
			<div class="flex items-start gap-3 px-4 pt-3 pb-2">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<StatusIndicator status={statusMap[detail.status] ?? 'active'} />
						<h2 class="truncate text-sm font-semibold text-foreground">
							{detail.title}
						</h2>
					</div>
				</div>
				{#if onClose}
					<button
						onclick={onClose}
						class="flex size-7 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground"
						aria-label="Tutup panel"
					>
						<X class="size-4" />
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<!-- ================================================================== -->
	<!-- EXPANSION DRAWER — detail of selected phase or overview            -->
	<!-- ================================================================== -->
	{#if expandedItem !== null}
		<div class="shrink-0 border-b border-border/40" transition:slide={{ duration: 200 }}>
			<div class="expansion-drawer px-4 py-2.5">
				{#if expandedItem === 'overview'}
					<!-- Overview: aggregate stats -->
					<div class="space-y-1.5">
						<p class="drawer-title text-xs font-semibold text-muted-foreground uppercase tracking-wide">Ikhtisar</p>
						<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
							<span class="inline-flex items-center gap-1">
								<UsersIcon class="size-3" />
								{memberCount} anggota
							</span>
							<span class="inline-flex items-center gap-1">
								<MessageCircle class="size-3" />
								{detail.message_count} pesan
							</span>
							<span class="inline-flex items-center gap-1">
								<Clock class="size-3" />
								{timeSince}
							</span>
						</div>
						{#if detail.unread_count > 0}
							<Badge variant="danger" class="text-caption">
								{detail.unread_count} pesan baru
							</Badge>
						{/if}
					</div>
				{:else if typeof expandedItem === 'number' && phases[expandedItem]}
					<!-- Phase detail: checkpoints -->
					{@const phase = phases[expandedItem]}
					{@const counts = checkpointCounts(phase)}
					<div class="space-y-2">
						<div class="drawer-title flex items-center justify-between">
							<p class="text-xs font-semibold text-foreground">
								{phase.title}
							</p>
							<span class="text-caption tabular-nums text-muted-foreground">
								{counts.done}/{counts.total}{counts.blocked > 0 ? ` · ${counts.blocked} terblokir` : ''}
							</span>
						</div>
						{#if phase.objective}
							<p class="text-caption leading-relaxed text-muted-foreground/70">{phase.objective}</p>
						{/if}
						<ul class="space-y-1">
							{#each phase.checkpoints as cp (cp.checkpoint_id)}
								<li class="flex items-start gap-1.5 text-xs">
									{#if cp.status === 'completed'}
										<Check class="mt-0.5 size-3 shrink-0 text-berhasil" />
										<span class="text-muted-foreground line-through">{cp.title}</span>
									{:else if cp.status === 'blocked'}
										<Lock class="mt-0.5 size-3 shrink-0 text-bahaya" />
										<span class="text-bahaya">{cp.title}</span>
									{:else}
										<Circle class="mt-0.5 size-3 shrink-0 text-muted-foreground/30" />
										<span class="text-foreground">{cp.title}</span>
									{/if}
								</li>
							{/each}
						</ul>
					</div>
				{/if}
			</div>
		</div>
	{/if}

	<!-- ================================================================== -->
	<!-- MOMENTUM TRAIL — phase dots, no finish line                        -->
	<!-- ================================================================== -->
	{#if phases.length > 0}
		<div class="momentum-strip shrink-0 flex items-center gap-1.5 px-4 py-1.5 border-b border-border/30">
			<span class="text-caption font-medium text-muted-foreground/60">Fase {currentPhaseIndex + 1} aktif</span>
			<span class="text-muted-foreground/30">·</span>
			<div class="flex items-center gap-1">
				{#each phases as phase, i (phase.phase_id)}
					{#if phase.status === 'completed'}
						<!-- Completed — filled accent dot -->
						<div class="size-2 rounded-full" style="background: color-mix(in srgb, var(--accent) 70%, var(--color-muted-foreground));"></div>
					{:else if i === currentPhaseIndex}
						<!-- Active — pulsing accent dot with partial fill ring -->
						<div class="relative flex items-center justify-center">
							<div class="size-2.5 rounded-full border-[1.5px] animate-pulse" style="border-color: color-mix(in srgb, var(--accent) 60%, var(--color-primary));">
								{#if activeCheckpointsTotal > 0}
									<div
										class="absolute inset-0 rounded-full"
										style="background: conic-gradient(
											color-mix(in srgb, var(--accent) 50%, var(--color-primary)) {activeCheckpointsDone / activeCheckpointsTotal * 360}deg,
											transparent {activeCheckpointsDone / activeCheckpointsTotal * 360}deg
										); opacity: 0.4;"
									></div>
								{/if}
							</div>
						</div>
					{:else}
						<!-- Future — hollow muted dot -->
						<div class="size-2 rounded-full border border-muted-foreground/25"></div>
					{/if}
				{/each}
			</div>
			{#if completedPhases > 0}
				<span class="text-caption text-muted-foreground/50">{completedPhases} selesai</span>
			{/if}
		</div>
	{/if}

	<!-- ================================================================== -->
	<!-- CHAT — sunken conversation well                                    -->
	<!-- ================================================================== -->
	<div class="chat-reveal">
		<WitnessChatPanel messages={detail.messages} onSend={onSendMessage} {onStempel} {sending} {stempeling} />
	</div>
</div>
{/key}

<style>
	/*
	 * Staggered top-to-bottom reveal — matches the motion.div cascade
	 * in SelfProfile and CommunityPulse so all three tabs feel consistent.
	 */
	@keyframes unwrap-in {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	/*
	 * Unified dark surface — header, card, and drawer share
	 * one tone so only the chat well stands out as lighter.
	 */
	.detail-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
		border-bottom: 1px solid color-mix(in srgb, var(--accent, transparent) 20%, var(--color-border));
		animation: unwrap-in 0.25s ease-out both;
	}

	.pinned-card {
		position: relative;
		margin: 0.5rem 0.75rem;
		padding: 0.625rem 0.75rem;
		border-radius: var(--r-lg, 0.5rem);
		background: transparent; /* inherits from .detail-header */
	}

	/* Two-column layout: 60 left / 40 right */
	.pinned-columns {
		display: flex;
		align-items: stretch;
	}
	.pinned-columns > :first-child {
		flex: 0 1 60%;
		min-width: 0;
	}
	/* Right col: divider + phase list take the rest */
	.phase-list-col {
		flex: 0 0 calc(40% - 1rem); /* minus divider margins */
		min-width: 0;
		max-height: 120px; /* cap so it doesn't push chat down too much */
	}

	/* Phase list scroll area — fade masks hint at overflow */
	.phase-scroll {
		scrollbar-width: thin;
		scrollbar-color: var(--color-border) transparent;
		mask-image: linear-gradient(to bottom, transparent, black 8px, black calc(100% - 8px), transparent);
		-webkit-mask-image: linear-gradient(to bottom, transparent, black 8px, black calc(100% - 8px), transparent);
	}

	/* Each phase row — clickable, compact */
	.phase-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		width: 100%;
		padding: 0.25rem 0.375rem;
		border-radius: var(--r-md, 0.375rem);
		cursor: pointer;
		transition: background-color 0.15s;
		border: none;
		background: transparent;
		text-align: left;
	}
	.phase-row:hover {
		background: color-mix(in srgb, var(--color-foreground) 4%, transparent);
	}
	.phase-row-active {
		background: color-mix(in srgb, var(--color-primary) 8%, transparent);
	}
	.phase-row-current {
		background: color-mix(in srgb, var(--color-primary) 4%, transparent);
	}
	/* Sticky row needs solid bg so scrolled items don't bleed through */
	.phase-row-sticky {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
	}
	.phase-row-sticky:hover {
		background: color-mix(in srgb, var(--color-foreground) 8%, var(--color-card));
	}

	/* Expansion drawer — slightly lighter than header for separation */
	.expansion-drawer {
		background: color-mix(in srgb, var(--color-foreground) 3%, var(--color-card));
	}

	/* Momentum trail — subtle transitional strip between header and chat */
	.momentum-strip {
		background: color-mix(in srgb, var(--color-foreground) 2%, var(--color-background));
		animation: unwrap-in 0.2s ease-out 0.06s both;
	}

	/* Chat entrance — fills remaining space with staggered reveal */
	.chat-reveal {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		animation: unwrap-in 0.3s ease-out 0.12s both;
	}

	/* Drawer title — matches active phase-row highlight for visual connection */
	.drawer-title {
		margin: -0.25rem -0.5rem 0;
		padding: 0.25rem 0.5rem;
		border-radius: var(--r-md, 0.375rem);
		background: color-mix(in srgb, var(--color-primary) 8%, transparent);
	}
</style>

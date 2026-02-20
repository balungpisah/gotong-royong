<script lang="ts">
	import type { WitnessDetail } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';
	import PhaseNavigator from './phase-navigator.svelte';
	import WitnessChatPanel from './witness-chat-panel.svelte';
	import Tip from '$lib/components/ui/tip.svelte';
	import X from '@lucide/svelte/icons/x';
	import UsersIcon from '@lucide/svelte/icons/users';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import Clock from '@lucide/svelte/icons/clock';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import Flame from '@lucide/svelte/icons/flame';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import FileCheck from '@lucide/svelte/icons/file-check';
	import Lock from '@lucide/svelte/icons/lock';
	import Eye from '@lucide/svelte/icons/eye';
	import Sparkles from '@lucide/svelte/icons/sparkles';

	interface Props {
		detail: WitnessDetail;
		onClose?: () => void;
		onSendMessage?: (content: string) => void;
		sending?: boolean;
	}

	let { detail, onClose, onSendMessage, sending = false }: Props = $props();

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

	const roleLabels: Record<string, string> = {
		pelapor: 'Pelapor',
		relawan: 'Relawan',
		koordinator: 'Koordinator',
		saksi: 'Saksi'
	};

	const roleColors: Record<string, string> = {
		pelapor: 'bg-peringatan/20 text-peringatan',
		relawan: 'bg-berhasil/20 text-berhasil',
		koordinator: 'bg-primary/20 text-primary',
		saksi: 'bg-bahaya/20 text-bahaya'
	};

	// ---------------------------------------------------------------------------
	// Derived: phases from main branch
	// ---------------------------------------------------------------------------

	const phases = $derived(detail.plan?.branches?.[0]?.phases ?? []);

	// ---------------------------------------------------------------------------
	// CD2: Development & Accomplishment — progress metrics
	// ---------------------------------------------------------------------------

	const totalCheckpoints = $derived(
		phases.reduce((sum, p) => sum + p.checkpoints.length, 0)
	);

	const completedCheckpoints = $derived(
		phases.reduce(
			(sum, p) => sum + p.checkpoints.filter((c) => c.status === 'completed').length,
			0
		)
	);

	const progressPercent = $derived(
		totalCheckpoints > 0 ? Math.round((completedCheckpoints / totalCheckpoints) * 100) : 0
	);

	/** Momentum label — gives emotional feedback on progress */
	const momentumLabel = $derived.by(() => {
		if (totalCheckpoints === 0) return '';
		if (progressPercent === 100) return 'Selesai! 🎉';
		if (progressPercent >= 80) return 'Hampir sampai!';
		if (progressPercent >= 50) return 'Momentum bagus';
		if (progressPercent >= 25) return 'Mulai jalan';
		return 'Baru dimulai';
	});

	const progressBarColor = $derived.by(() => {
		if (progressPercent >= 80) return 'bg-berhasil';
		if (progressPercent >= 25) return 'bg-primary';
		return 'bg-peringatan';
	});

	// ---------------------------------------------------------------------------
	// CD5: Social Influence — member presence strip
	// ---------------------------------------------------------------------------

	const visibleMembers = $derived((detail.members ?? []).slice(0, 5));
	const overflowCount = $derived(Math.max(0, (detail.members ?? []).length - 5));

	/** Initials from a name string */
	function getInitials(name: string): string {
		return name
			.split(' ')
			.map((w) => w[0])
			.join('')
			.toUpperCase()
			.slice(0, 2);
	}

	/** Deterministic hue from string for avatar backgrounds */
	function nameHue(name: string): number {
		let hash = 0;
		for (let i = 0; i < name.length; i++) {
			hash = name.charCodeAt(i) + ((hash << 5) - hash);
		}
		return Math.abs(hash) % 360;
	}

	// ---------------------------------------------------------------------------
	// CD6: Scarcity — evidence needed + blocked count
	// ---------------------------------------------------------------------------

	const evidenceNeeded = $derived(
		phases
			.flatMap((p) => p.checkpoints)
			.filter((c) => c.evidence_required === true && c.status !== 'completed').length
	);

	const blockedCount = $derived(
		phases.flatMap((p) => p.checkpoints).filter((c) => c.status === 'blocked').length
	);

	// ---------------------------------------------------------------------------
	// CD8: Loss & Avoidance — unread / time since update
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
	// CD1: Epic Meaning — rahasia level display
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

<div class="flex h-full flex-col overflow-hidden">
	<!-- ================================================================== -->
	<!-- STATUS ZONE — enriched header + navigator, pinned top              -->
	<!-- ================================================================== -->
	<div class="relative z-10 shrink-0 border-b border-border/40 bg-card">
		<!-- Panel header -->
		<div class="flex items-start gap-3 px-4 pt-3 pb-1">
			<div class="min-w-0 flex-1">
				<!-- Title row -->
				<div class="flex items-center gap-2">
					<StatusIndicator status={statusMap[detail.status] ?? 'active'} />
					<h2 class="truncate text-sm font-semibold text-foreground">
						{detail.title}
					</h2>
				</div>

				<!-- Track badge + meta row -->
				<div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1">
					{#if detail.track_hint}
						<Badge
							variant={trackVariantMap[detail.track_hint] ?? 'secondary'}
							class="text-[10px]"
						>
							{detail.track_hint}
						</Badge>
					{/if}
					{#if rahasiaDisplay.show}
						<Badge variant={rahasiaDisplay.variant} class="text-[10px]">
							<ShieldAlert class="mr-0.5 size-2.5" />
							{rahasiaDisplay.label}
						</Badge>
					{/if}
					<span class="inline-flex items-center gap-0.5 text-[10px] text-muted-foreground">
						<Clock class="size-2.5" />
						{timeSince}
					</span>
					<span class="inline-flex items-center gap-0.5 text-[10px] text-muted-foreground">
						<MessageCircle class="size-2.5" />
						{detail.message_count}
					</span>
					{#if detail.unread_count > 0}
						<Badge variant="danger" class="text-[9px] animate-pulse">
							{detail.unread_count} baru
						</Badge>
					{/if}
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

		<!-- ============================================================== -->
		<!-- CD2: Progress bar with momentum label                         -->
		<!-- ============================================================== -->
		{#if totalCheckpoints > 0}
			<div class="px-4 pt-1.5 pb-1">
				<div class="flex items-center gap-2">
					<div class="relative h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
						<div
							class="h-full rounded-full transition-all duration-500 ease-out {progressBarColor}"
							style="width: {progressPercent}%"
						></div>
					</div>
					<span class="shrink-0 text-[10px] font-medium tabular-nums text-muted-foreground">
						{completedCheckpoints}/{totalCheckpoints}
					</span>
				</div>
				{#if momentumLabel}
					<div class="mt-0.5 flex items-center gap-1">
						{#if progressPercent >= 50}
							<Flame class="size-3 text-peringatan" />
						{:else}
							<TrendingUp class="size-3 text-muted-foreground" />
						{/if}
						<span class="text-[10px] font-medium {progressPercent >= 50 ? 'text-peringatan' : 'text-muted-foreground'}">
							{momentumLabel}
						</span>
						{#if progressPercent === 100}
							<Sparkles class="size-3 text-berhasil animate-bounce" />
						{/if}
					</div>
				{/if}
			</div>
		{/if}

		<!-- ============================================================== -->
		<!-- CD5: Member presence strip — avatar stack with role tooltips   -->
		<!-- ============================================================== -->
		{#if visibleMembers.length > 0}
			<div class="flex items-center gap-2 px-4 pb-1.5">
				<div class="flex -space-x-1.5">
					{#each visibleMembers as member, i}
						<Tip text="{member.name} · {roleLabels[member.role] ?? member.role}" side="bottom">
							<div
								class="relative flex size-6 items-center justify-center rounded-full border-2 border-card text-[8px] font-bold shadow-sm transition-transform hover:z-10 hover:scale-125"
								style="z-index: {visibleMembers.length - i};
									{member.avatar_url
									? `background-image: url(${member.avatar_url}); background-size: cover;`
									: `background-color: hsl(${nameHue(member.name)}, 55%, 45%); color: white;`}"
							>
								{#if !member.avatar_url}
									{getInitials(member.name)}
								{/if}
								<!-- Role dot indicator -->
								<div
									class="absolute -bottom-0.5 -right-0.5 size-2 rounded-full border border-card {roleColors[member.role]?.split(' ')[0] ?? 'bg-muted'}"
								></div>
							</div>
						</Tip>
					{/each}
					{#if overflowCount > 0}
						<div
							class="flex size-6 items-center justify-center rounded-full border-2 border-card bg-muted text-[8px] font-bold text-muted-foreground"
						>
							+{overflowCount}
						</div>
					{/if}
				</div>

				<!-- Compact role legend -->
				<div class="flex flex-wrap gap-1">
					{#each [...new Set(visibleMembers.map((m) => m.role))] as role}
						<span class="rounded px-1 py-0.5 text-[8px] font-medium {roleColors[role] ?? 'bg-muted text-muted-foreground'}">
							{roleLabels[role] ?? role}
						</span>
					{/each}
				</div>
			</div>
		{/if}

		<!-- ============================================================== -->
		<!-- CD6: Scarcity nudges — evidence needed / blocked               -->
		<!-- ============================================================== -->
		{#if evidenceNeeded > 0 || blockedCount > 0}
			<div class="flex items-center gap-1.5 px-4 pb-2">
				{#if evidenceNeeded > 0}
					<Badge variant="warning" class="text-[9px]">
						<FileCheck class="mr-0.5 size-2.5" />
						{evidenceNeeded} bukti dibutuhkan
					</Badge>
				{/if}
				{#if blockedCount > 0}
					<Badge variant="danger" class="text-[9px]">
						<Lock class="mr-0.5 size-2.5" />
						{blockedCount} terblokir
					</Badge>
				{/if}
			</div>
		{/if}

		<!-- Phase navigator — collapsible handlebar + horizontal phase carousel -->
		{#if phases.length > 0}
			<div class="px-3 pb-2">
				<PhaseNavigator {phases} />
			</div>
		{/if}
	</div>

	<!-- Chat zone — sunken conversation well -->
	<WitnessChatPanel messages={detail.messages} onSend={onSendMessage} {sending} />
</div>

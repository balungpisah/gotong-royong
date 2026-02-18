<script lang="ts">
	import type { WitnessDetail } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';
	import PhaseCarousel from './phase-carousel.svelte';
	import ChatThread from './chat-thread.svelte';
	import X from '@lucide/svelte/icons/x';
	import UsersIcon from '@lucide/svelte/icons/users';

	interface Props {
		detail: WitnessDetail;
		onClose: () => void;
		onSendMessage?: (content: string) => void;
		sending?: boolean;
	}

	let { detail, onClose, onSendMessage, sending = false }: Props = $props();

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

	// Get phases from the main branch (first branch)
	const phases = $derived(detail.plan?.branches?.[0]?.phases ?? []);

	// Find the active phase index or default to 0
	const initialPhaseIndex = $derived.by(() => {
		const idx = phases.findIndex((p) => p.status === 'active');
		return idx >= 0 ? idx : 0;
	});

	let activePhaseIndex = $state(0);

	// Set initial phase when detail changes
	$effect(() => {
		activePhaseIndex = initialPhaseIndex;
	});
</script>

<div class="flex h-full flex-col overflow-hidden">
	<!-- Panel header -->
	<div class="flex items-center gap-3 border-b border-border/40 px-4 py-3">
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2">
				<StatusIndicator status={statusMap[detail.status] ?? 'active'} />
				<h2 class="truncate text-sm font-semibold text-foreground">
					{detail.title}
				</h2>
			</div>
			<div class="mt-1 flex items-center gap-2">
				{#if detail.track_hint}
					<Badge
						variant={trackVariantMap[detail.track_hint] ?? 'secondary'}
						class="text-[10px]"
					>
						{detail.track_hint}
					</Badge>
				{/if}
				<span class="inline-flex items-center gap-1 text-[10px] text-muted-foreground">
					<UsersIcon class="size-3" />
					{detail.member_count}
				</span>
			</div>
		</div>
		<button
			onclick={onClose}
			class="flex size-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground"
			aria-label="Tutup panel"
		>
			<X class="size-4" />
		</button>
	</div>

	<!-- Phase carousel (top section) -->
	{#if phases.length > 0}
		<div class="border-b border-border/40 px-4 py-3">
			<PhaseCarousel {phases} bind:activeIndex={activePhaseIndex} />
		</div>
	{/if}

	<!-- Chat thread (bottom section, fills remaining space) -->
	<ChatThread messages={detail.messages} onSend={onSendMessage} {sending} />
</div>

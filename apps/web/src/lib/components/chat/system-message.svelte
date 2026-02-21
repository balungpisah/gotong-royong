<script lang="ts">
	import type { SystemMessage } from '$lib/types';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import Shield from '@lucide/svelte/icons/shield';
	import FileText from '@lucide/svelte/icons/file-text';
	import Coins from '@lucide/svelte/icons/coins';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Trophy from '@lucide/svelte/icons/trophy';
	import PartyPopper from '@lucide/svelte/icons/party-popper';
	import type { Component } from 'svelte';

	let { message }: { message: SystemMessage } = $props();

	const subtypeIcons: Record<string, Component<{ class?: string }>> = {
		checkpoint_completed: CircleCheck,
		phase_activated: ArrowRight,
		phase_completed: Trophy,
		vote_result: BarChart3,
		member_joined: UserPlus,
		role_assigned: Shield,
		plan_updated: FileText,
		galang_transaction: Coins
	};

	const Icon = $derived(subtypeIcons[message.subtype] || ArrowRight);
	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));

	// ---------------------------------------------------------------------------
	// CD2: Celebration moments — special styling for milestone events
	// ---------------------------------------------------------------------------

	const isCelebration = $derived(
		message.subtype === 'phase_completed' || message.subtype === 'checkpoint_completed'
	);

	const isWelcome = $derived(
		message.subtype === 'member_joined'
	);

	const pillClass = $derived.by(() => {
		if (isCelebration) {
			return 'bg-berhasil/10 border border-berhasil/20';
		}
		if (isWelcome) {
			return 'bg-primary/10 border border-primary/20';
		}
		return 'bg-muted';
	});

	const iconClass = $derived.by(() => {
		if (isCelebration) return 'text-berhasil';
		if (isWelcome) return 'text-primary';
		return 'text-muted-foreground';
	});

	const textClass = $derived.by(() => {
		if (isCelebration) return 'text-berhasil font-semibold';
		if (isWelcome) return 'text-primary font-medium';
		return 'text-muted-foreground font-medium';
	});

	const lineClass = $derived.by(() => {
		if (isCelebration) return 'bg-berhasil/20';
		return 'bg-border/50';
	});
</script>

<div class="flex items-center justify-center gap-2 py-2" data-slot="system-message">
	<div class="h-px flex-1 {lineClass}"></div>
	<div class="flex items-center gap-1.5 rounded-full px-3 py-1 {pillClass}">
		<!-- CD2: Celebration sparkle before icon -->
		{#if isCelebration}
			<Sparkles class="size-3 text-berhasil animate-pulse" />
		{/if}
		<Icon class="size-3 {iconClass}" />
		<span class="text-[11px] {textClass}">{message.content}</span>
		<span class="text-[10px] text-muted-foreground/60">{timeStr}</span>
		<!-- CD2: Party icon after celebration text -->
		{#if message.subtype === 'phase_completed'}
			<PartyPopper class="size-3 text-berhasil" />
		{/if}
	</div>
	<div class="h-px flex-1 {lineClass}"></div>
</div>

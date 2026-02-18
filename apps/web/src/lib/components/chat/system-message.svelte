<script lang="ts">
	import type { SystemMessage } from '$lib/types';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import Shield from '@lucide/svelte/icons/shield';
	import FileText from '@lucide/svelte/icons/file-text';
	import type { Component } from 'svelte';

	let { message }: { message: SystemMessage } = $props();

	const subtypeIcons: Record<string, Component<{ class?: string }>> = {
		checkpoint_completed: CircleCheck,
		phase_activated: ArrowRight,
		phase_completed: CircleCheck,
		vote_result: BarChart3,
		member_joined: UserPlus,
		role_assigned: Shield,
		plan_updated: FileText
	};

	const Icon = $derived(subtypeIcons[message.subtype] || ArrowRight);
	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));
</script>

<div class="flex items-center justify-center gap-2 py-2" data-slot="system-message">
	<div class="h-px flex-1 bg-border/50"></div>
	<div class="flex items-center gap-1.5 rounded-full bg-muted px-3 py-1">
		<Icon class="size-3 text-muted-foreground" />
		<span class="text-[10px] font-medium text-muted-foreground">{message.content}</span>
		<span class="text-[9px] text-muted-foreground/60">{timeStr}</span>
	</div>
	<div class="h-px flex-1 bg-border/50"></div>
</div>

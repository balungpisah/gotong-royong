<script lang="ts">
	import type { PathPlan } from '$lib/types';
	import { cn, trackColors } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge';
	import PhaseBreadcrumb from './phase-breadcrumb.svelte';
	import PhaseCard from './phase-card.svelte';
	import GitBranch from '@lucide/svelte/icons/git-branch';

	let { plan }: { plan: PathPlan } = $props();

	let activeBranchId = $state('');

	$effect(() => {
		activeBranchId = plan.branches[0]?.branch_id || '';
	});

	const activeBranch = $derived(plan.branches.find(b => b.branch_id === activeBranchId) || plan.branches[0]);

	const colors = $derived(plan.track_hint ? trackColors(plan.track_hint) : null);

	const seedHintLabels: Record<string, string> = {
		Keresahan: '😟 Keresahan',
		Aspirasi: '✨ Aspirasi',
		Kejadian: '📰 Kejadian',
		Rencana: '📋 Rencana',
		Pertanyaan: '❓ Pertanyaan'
	};
</script>

<div class={cn('flex flex-col gap-4', colors && colors.border, colors && 'border-l-4 pl-4')} data-slot="path-plan-view">
	<!-- Header -->
	<div class="flex flex-col gap-2">
		<div class="flex items-start justify-between gap-2">
			<h2 class="text-base font-bold text-foreground">{plan.title}</h2>
			<Badge variant="secondary" class="shrink-0 text-[9px]">v{plan.version}</Badge>
		</div>
		<p class="text-sm text-muted-foreground">{plan.summary}</p>
		<div class="flex flex-wrap gap-2">
			{#if plan.track_hint}
				<Badge variant={`track-${plan.track_hint}` as any} class="text-[9px]">{plan.track_hint}</Badge>
			{/if}
			{#if plan.seed_hint}
				<Badge variant="confidence" class="text-[9px]">{seedHintLabels[plan.seed_hint] || plan.seed_hint}</Badge>
			{/if}
		</div>
	</div>

	<!-- Branch selector (if multiple) -->
	{#if plan.branches.length > 1}
		<div class="flex items-center gap-2">
			<GitBranch class="size-4 text-muted-foreground" />
			{#each plan.branches as branch (branch.branch_id)}
				<button
					class={cn(
						'rounded-full px-3 py-1 text-xs font-medium transition',
						branch.branch_id === activeBranchId
							? 'bg-primary text-primary-foreground'
							: 'bg-muted text-muted-foreground hover:text-foreground'
					)}
					onclick={() => activeBranchId = branch.branch_id}
				>
					{branch.label}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Breadcrumb -->
	{#if activeBranch}
		<PhaseBreadcrumb phases={activeBranch.phases} />

		<!-- Phase Cards -->
		<div class="flex flex-col gap-3">
			{#each activeBranch.phases as phase (phase.phase_id)}
				<PhaseCard {phase} />
			{/each}
		</div>
	{/if}
</div>

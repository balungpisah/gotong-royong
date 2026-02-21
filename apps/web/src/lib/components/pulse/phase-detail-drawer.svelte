<script lang="ts">
	import { safeSlide as slide } from '$lib/utils/safe-slide';
	import type { Phase } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import CheckCircle from '@lucide/svelte/icons/check-circle-2';
	import Circle from '@lucide/svelte/icons/circle';
	import Lock from '@lucide/svelte/icons/lock';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	interface Props {
		phase: Phase | null;
		open?: boolean;
		onToggle?: () => void;
	}

	let { phase, open = $bindable(false), onToggle }: Props = $props();

	const phaseBadgeVariant = $derived.by(() => {
		if (!phase) return 'secondary' as const;
		switch (phase.status) {
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

	function handleToggle() {
		open = !open;
		onToggle?.();
	}

	function getCheckpointIconAndColor(status: string): { icon: typeof Circle; colorClass: string } {
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
</script>

{#if phase !== null}
	<div class="flex flex-col gap-1">
		<!-- Toggle Bar -->
		<div
			class="flex cursor-pointer items-center justify-between rounded-xl border border-border/40 bg-muted/30 px-3 py-2"
			onclick={handleToggle}
			role="button"
			tabindex="0"
			onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? handleToggle() : null}
		>
			<div class="flex items-center gap-2">
				<span class="text-xs font-medium text-foreground">{phase.title}</span>
				<Badge variant={phaseBadgeVariant} class="text-[9px]">
					{phase.status}
				</Badge>
			</div>
			<ChevronDown
				class="size-4 text-muted-foreground transition-transform duration-200 {open ? 'rotate-180' : ''}"
			/>
		</div>

		<!-- Expanded Content -->
		{#if open}
			<div
				class="rounded-xl border border-border/40 bg-muted/20 px-3 py-2"
				transition:slide={{ duration: 200 }}
			>
				{#if phase.objective}
					<p class="mb-2 text-xs italic text-muted-foreground">{phase.objective}</p>
				{/if}

				{#if phase.checkpoints && phase.checkpoints.length > 0}
					<ul class="flex flex-col gap-1.5">
						{#each phase.checkpoints as cp}
							{@const { icon: Icon, colorClass } = getCheckpointIconAndColor(cp.status)}
							{@const completed = cp.status === 'completed'}
							<li class="flex items-start gap-2 text-xs">
								<Icon class="mt-0.5 size-3.5 shrink-0 {colorClass}" />
								<div class="min-w-0 flex-1">
									<span class={completed ? 'text-muted-foreground line-through' : 'text-foreground'}>
										{cp.title}
									</span>
									{#if cp.description}
										<p class="mt-0.5 text-[10px] text-muted-foreground">{cp.description}</p>
									{/if}
								</div>
								{#if cp.evidence_required}
									<Badge variant="warning" class="shrink-0 text-[9px]">bukti</Badge>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { CommunityIcjSummary } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		summary: CommunityIcjSummary;
	}

	const { summary }: Props = $props();

	const axes = $derived([
		{ label: m.icj_integrity_label(), value: summary.avg_integrity, color: 'var(--c-tandang-i)' },
		{ label: m.icj_competence_label(), value: summary.avg_competence, color: 'var(--c-tandang-c)' },
		{ label: m.icj_judgment_label(), value: summary.avg_judgment, color: 'var(--c-tandang-j)' }
	]);
</script>

<motion.div
	initial={{ opacity: 0, y: 12 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.1 }}
	class="rounded-xl border border-border/30 bg-muted/10 p-4"
>
	<h3 class="text-small font-semibold text-foreground">{m.komunitas_icj_title()}</h3>
	<div class="mt-3 space-y-3">
		{#each axes as axis}
			<div class="flex items-center gap-3">
				<span class="w-28 shrink-0 text-caption font-medium" style="color: {axis.color}">{axis.label}</span>
				<div class="h-2.5 flex-1 rounded-full bg-muted/30">
					<div
						class="h-full rounded-full transition-all duration-700"
						style="width: {axis.value * 100}%; background: {axis.color}"
					></div>
				</div>
				<span class="w-10 shrink-0 text-right text-caption font-bold" style="color: {axis.color}">{(axis.value * 100).toFixed(0)}</span>
			</div>
		{/each}
	</div>
</motion.div>

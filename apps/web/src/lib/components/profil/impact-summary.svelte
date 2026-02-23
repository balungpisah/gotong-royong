<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import Target from '@lucide/svelte/icons/target';
	import HeartHandshake from '@lucide/svelte/icons/heart-handshake';
	import ThumbsUp from '@lucide/svelte/icons/thumbs-up';
	import FileCheck from '@lucide/svelte/icons/file-check';
	import Vote from '@lucide/svelte/icons/vote';
	import type { ImpactMetrics } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		impact: ImpactMetrics;
	}

	const { impact }: Props = $props();

	const metrics = $derived([
		{ label: m.profil_impact_resolved(), value: impact.witnesses_resolved, icon: Target, color: 'text-berhasil bg-berhasil-lembut' },
		{ label: m.profil_impact_helped(), value: impact.people_helped, icon: HeartHandshake, color: 'text-signal-proof bg-signal-proof/10' },
		{ label: m.profil_impact_dukung_given(), value: impact.total_dukung_given, icon: ThumbsUp, color: 'text-waspada bg-waspada-lembut' },
		{ label: m.profil_impact_evidence(), value: impact.evidence_validated, icon: FileCheck, color: 'text-signal-dukung bg-signal-dukung/10' },
		{ label: m.profil_impact_votes(), value: impact.votes_participated, icon: Vote, color: 'text-tandang bg-tandang/10' }
	]);
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.15 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4">
		<h3 class="text-xs font-semibold text-foreground">{m.profil_impact_title()}</h3>
		<p class="mt-0.5 text-caption text-muted-foreground">{m.profil_impact_subtitle()}</p>

		<div class="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-3">
			{#each metrics as metric, i}
				{@const Icon = metric.icon}
				<motion.div
					class="flex flex-col items-center gap-1.5 rounded-lg bg-muted/20 p-3 text-center"
					initial={{ opacity: 0, scale: 0.95 }}
					animate={{ opacity: 1, scale: 1 }}
					transition={{ duration: 0.25, delay: 0.05 * i }}
				>
					<div class="flex size-8 items-center justify-center rounded-lg {metric.color}">
						<Icon class="size-4" />
					</div>
					<p class="text-xl font-bold text-foreground">{metric.value}</p>
					<p class="text-caption text-muted-foreground leading-tight">{metric.label}</p>
				</motion.div>
			{/each}
		</div>
	</div>
</motion.div>

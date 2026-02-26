<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import Flame from '@lucide/svelte/icons/flame';
	import Zap from '@lucide/svelte/icons/zap';
	import CalendarDays from '@lucide/svelte/icons/calendar-days';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import type { ConsistencyInfo, GenesisInfo } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		consistency: ConsistencyInfo;
		genesis: GenesisInfo;
	}

	const { consistency, genesis }: Props = $props();

	const multiplierPct = $derived(Math.round((consistency.multiplier - 1) * 100));
	const qualityPct = $derived(Math.round(consistency.quality_avg * 100));
	const genesisPct = $derived(
		genesis.threshold > 0
			? Math.min(
					Math.round((genesis.meaningful_interactions_this_month / genesis.threshold) * 100),
					100
				)
			: 0
	);
	const hasGap = $derived(consistency.gap_days > 7);
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.2 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4">
		<h3 class="text-small font-semibold text-foreground">{m.profil_consistency_title()}</h3>

		<!-- Multiplier badge -->
		<div class="mt-3 flex items-center gap-3">
			<div class="flex items-center gap-1.5 rounded-full bg-waspada-lembut px-3 py-1">
				<Zap class="size-3.5 text-waspada" />
				<span class="text-caption font-bold text-waspada">{consistency.multiplier.toFixed(2)}×</span
				>
			</div>
			{#if multiplierPct > 0}
				<span class="text-caption text-muted-foreground"
					>{m.profil_consistency_bonus({ pct: String(multiplierPct) })}</span
				>
			{:else}
				<span class="text-caption text-muted-foreground">{m.profil_consistency_no_bonus()}</span>
			{/if}
		</div>

		<!-- Stats grid -->
		<div class="mt-3 grid grid-cols-2 gap-2">
			<div class="rounded-lg bg-muted/20 p-2.5">
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<Flame class="size-3" />
					<span class="text-caption">{m.profil_streak_label()}</span>
				</div>
				<p class="mt-1 text-body font-bold text-foreground">
					{m.profil_streak_weeks({ weeks: String(consistency.streak_weeks) })}
				</p>
			</div>
			<div class="rounded-lg bg-muted/20 p-2.5">
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<CalendarDays class="size-3" />
					<span class="text-caption">{m.profil_thirty_days()}</span>
				</div>
				<p class="mt-1 text-body font-bold text-foreground">
					{m.profil_contributions_count({ count: String(consistency.contributions_30d) })}
				</p>
			</div>
			<div class="rounded-lg bg-muted/20 p-2.5">
				<span class="text-caption text-muted-foreground">{m.profil_quality_avg()}</span>
				<div class="mt-1 flex items-center gap-2">
					<div class="h-1.5 flex-1 rounded-full bg-muted/30">
						<div
							class="h-full rounded-full bg-primary transition-all duration-500"
							style="width: {qualityPct}%"
						></div>
					</div>
					<span class="text-caption font-bold text-foreground">{qualityPct}%</span>
				</div>
			</div>
			<div class="rounded-lg bg-muted/20 p-2.5">
				{#if hasGap}
					<div class="flex items-center gap-1.5 text-waspada">
						<AlertTriangle class="size-3" />
						<span class="text-caption">{m.profil_gap_label()}</span>
					</div>
					<p class="mt-1 text-body font-bold text-waspada">
						{m.profil_gap_days({ days: String(consistency.gap_days) })}
					</p>
				{:else}
					<span class="text-caption text-muted-foreground">{m.profil_gap_label()}</span>
					<p class="mt-1 text-body font-bold text-berhasil">
						{m.profil_gap_days({ days: String(consistency.gap_days) })} ✓
					</p>
				{/if}
			</div>
		</div>

		<!-- Genesis section -->
		<div class="mt-4 rounded-lg border border-border/20 bg-muted/5 p-3">
			<div class="flex items-center justify-between">
				<span class="text-caption font-medium text-foreground">{m.profil_genesis_weight()}</span>
				<span class="text-caption font-bold text-foreground">
					{genesis.weight !== null ? genesis.weight.toFixed(3) : '—'}
				</span>
			</div>
			<div class="mt-2">
				<div class="flex items-center justify-between text-caption text-muted-foreground">
					<span>{m.profil_genesis_interactions()}</span>
					<span>{genesis.meaningful_interactions_this_month}/{genesis.threshold}</span>
				</div>
				<div class="mt-1 h-1.5 w-full rounded-full bg-muted/30">
					<div
						class="h-full rounded-full transition-all duration-500 {genesisPct >= 100
							? 'bg-berhasil'
							: 'bg-primary/60'}"
						style="width: {genesisPct}%"
					></div>
				</div>
				{#if genesisPct >= 100}
					<p class="mt-1 text-caption text-berhasil">{m.profil_genesis_paused()}</p>
				{:else}
					<p class="mt-1 text-caption text-muted-foreground">
						{m.profil_genesis_remaining({
							count: String(genesis.threshold - genesis.meaningful_interactions_this_month)
						})}
					</p>
				{/if}
			</div>
		</div>
	</div>
</motion.div>

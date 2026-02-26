<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { TierDistribution } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		tiers: TierDistribution[];
	}

	const { tiers }: Props = $props();
</script>

<motion.div
	initial={{ opacity: 0, y: 12 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.2 }}
	class="rounded-xl border border-border/30 bg-muted/10 p-4"
>
	<h3 class="text-small font-semibold text-foreground">{m.komunitas_tier_title()}</h3>

	<!-- Stacked bar -->
	<div class="mt-3 flex h-8 overflow-hidden rounded-full">
		{#each tiers as tier}
			<div
				class="transition-all duration-700"
				style="width: {tier.percentage}%; background: {tier.color}"
				title="{tier.tier_name}: {tier.count} ({tier.percentage}%)"
			></div>
		{/each}
	</div>

	<!-- Legend -->
	<div class="mt-3 flex flex-wrap gap-3">
		{#each tiers as tier}
			<div class="flex items-center gap-1.5">
				<div class="size-2.5 rounded-full" style="background: {tier.color}"></div>
				<span class="text-caption text-muted-foreground">
					{tier.tier_name} <span class="font-medium text-foreground">{tier.count}</span>
				</span>
			</div>
		{/each}
	</div>
</motion.div>

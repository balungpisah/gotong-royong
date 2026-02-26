<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import FileText from '@lucide/svelte/icons/file-text';
	import Users from '@lucide/svelte/icons/users';
	import Camera from '@lucide/svelte/icons/camera';
	import HandHeart from '@lucide/svelte/icons/hand-heart';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Vote from '@lucide/svelte/icons/vote';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import Award from '@lucide/svelte/icons/award';
	import ThumbsUp from '@lucide/svelte/icons/thumbs-up';
	import Heart from '@lucide/svelte/icons/heart';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import type { ActivityTimelineItem } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		items: ActivityTimelineItem[];
	}

	const { items }: Props = $props();

	const SHOW_LIMIT = 15;
	let showAll = $state(false);

	const visibleItems = $derived(showAll ? items : items.slice(0, SHOW_LIMIT));

	type ActivityType = ActivityTimelineItem['type'];

	const iconMap: Record<ActivityType, typeof FileText> = {
		witness_created: FileText,
		witness_joined: Users,
		evidence_submitted: Camera,
		vouch_given: HandHeart,
		vouch_received: ShieldCheck,
		vote_cast: Vote,
		resolution_completed: CheckCircle,
		skill_validated: Award,
		dukung_given: ThumbsUp,
		dukung_received: Heart,
		tier_change: TrendingUp
	};

	function relativeDate(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		const days = Math.floor(diff / 86400000);
		if (days === 0) return m.time_today();
		if (days === 1) return m.time_yesterday();
		if (days < 30) return m.time_days_ago({ days: String(days) });
		const months = Math.floor(days / 30);
		if (months < 12) return m.time_months_ago({ months: String(months) });
		return m.time_years_ago({ years: String(Math.floor(months / 12)) });
	}
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.25 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4">
		<h3 class="mb-4 text-small font-semibold text-foreground">{m.profil_contribution_trail()}</h3>

		<div class="relative border-l-2 border-primary/20 pl-4 space-y-4">
			{#each visibleItems as item, i (item.timestamp)}
				{@const Icon = iconMap[item.type] ?? FileText}
				<motion.div
					initial={{ opacity: 0, x: -4 }}
					animate={{ opacity: 1, x: 0 }}
					transition={{ duration: 0.22, delay: 0.03 * i }}
				>
					<div class="relative flex items-start gap-2.5">
						<!-- Timeline dot -->
						<span
							class="absolute -left-[1.375rem] top-1 size-2.5 rounded-full bg-primary/40 ring-2 ring-background shrink-0"
						></span>

						<!-- Icon -->
						<div class="mt-0.5 shrink-0 rounded-md bg-muted/20 p-1">
							<Icon class="size-3 text-foreground/60" />
						</div>

						<!-- Content -->
						<div class="flex min-w-0 flex-1 flex-col gap-0.5">
							<span class="text-caption text-foreground/80 leading-snug">
								{item.text}
							</span>
							<span class="text-caption text-muted-foreground">
								{relativeDate(item.timestamp)}
							</span>
						</div>
					</div>
				</motion.div>
			{/each}
		</div>

		{#if items.length > SHOW_LIMIT}
			<button
				onclick={() => (showAll = !showAll)}
				class="mt-4 text-caption text-muted-foreground hover:text-foreground transition-colors"
			>
				{showAll ? m.common_collapse() : m.common_view_all({ count: String(items.length) })}
			</button>
		{/if}
	</div>
</motion.div>

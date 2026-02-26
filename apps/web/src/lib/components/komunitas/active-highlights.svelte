<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { ActiveMemberHighlight } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';

	interface Props {
		members: ActiveMemberHighlight[];
	}

	const { members }: Props = $props();
</script>

<div>
	<h3 class="text-small font-semibold text-foreground">{m.komunitas_active_title()}</h3>
	<div class="mt-3 flex gap-3 overflow-x-auto pb-2" style="scrollbar-width: none;">
		{#each members as member, i (member.user_id)}
			<motion.div
				class="min-w-[160px] flex-shrink-0 rounded-xl border border-border/30 bg-card p-3 shadow-sm"
				initial={{ opacity: 0, y: 12 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.3, delay: i * 0.08 }}
			>
				<!-- Avatar + name -->
				<div class="flex items-center gap-2">
					<TandangAvatar
						person={{
							user_id: member.user_id,
							name: member.name,
							avatar_url: member.avatar_url,
							tier: member.tier
						}}
						size="sm"
						showTierDot
					/>
					<div class="min-w-0">
						<p class="truncate text-caption font-bold text-foreground">{member.name}</p>
						<span class="text-caption text-muted-foreground"
							>{m.komunitas_contributions({ count: String(member.contributions_this_week) })}</span
						>
					</div>
				</div>

				<!-- Highlight reason -->
				<p class="mt-2 text-caption leading-relaxed text-muted-foreground">
					{member.highlight_reason}
				</p>

				{#if member.streak_days > 7}
					<p class="mt-1 text-caption text-amber-600">
						🔥 {m.komunitas_streak_days({ days: String(member.streak_days) })}
					</p>
				{/if}
			</motion.div>
		{/each}
	</div>
</div>

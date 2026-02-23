<script lang="ts">
	import type { FollowableEntity } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		entities: FollowableEntity[];
		onFollow: (entityId: string) => void;
		onFollowAll: () => void;
	}

	let { entities, onFollow, onFollowAll }: Props = $props();

	const iconMap: Record<string, string> = {
		lingkungan: '📍',
		topik: '🏷️',
		kelompok: '👥',
		lembaga: '🏢',
		warga: '👤'
	};
</script>

<div class="rounded-xl border border-primary/20 bg-primary/5 p-4">
	<!-- Header -->
	<h3 class="text-sm font-semibold text-foreground">
		💡 {m.pulse_feed_suggestion_title()}
	</h3>

	<!-- Entity list -->
	<div class="mt-3 space-y-2">
		{#each entities as entity (entity.entity_id)}
			<div class="flex items-center justify-between gap-2">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-1.5">
						<span class="text-xs">{iconMap[entity.entity_type] ?? '📌'}</span>
						<span class="truncate text-sm font-medium text-foreground">{entity.label}</span>
					</div>
					{#if entity.description}
						<p class="mt-0.5 truncate text-xs text-muted-foreground">
							{entity.description}
						</p>
					{/if}
					<p class="text-xs text-muted-foreground/70">
						{m.pulse_feed_suggestion_activities({ count: entity.witness_count })} · {m.discover_followers({ count: String(entity.follower_count) })}
					</p>
				</div>

				<button
					onclick={() => onFollow(entity.entity_id)}
					class="shrink-0 rounded-full px-3 py-1 text-xs font-semibold transition-colors
						{entity.followed
						? 'bg-primary/10 text-primary'
						: 'bg-primary text-primary-foreground hover:bg-primary/90'}"
				>
					{entity.followed ? m.pulse_feed_entity_following() : m.pulse_feed_entity_follow()}
				</button>
			</div>
		{/each}
	</div>

	<!-- Follow all button -->
	{#if entities.some((e) => !e.followed)}
		<button
			onclick={onFollowAll}
			class="mt-3 w-full rounded-lg border border-primary/30 py-2 text-xs font-semibold text-primary transition-colors hover:bg-primary/10"
		>
			{m.pulse_feed_suggestion_follow_all()}
		</button>
	{/if}
</div>

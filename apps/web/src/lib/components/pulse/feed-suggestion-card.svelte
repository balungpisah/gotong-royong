<script lang="ts">
	import type { FollowableEntity } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';

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
	<h3 class="text-body font-semibold text-foreground">
		💡 {m.pulse_feed_suggestion_title()}
	</h3>

	<!-- Entity list -->
	<div class="mt-3 space-y-2">
		{#each entities as entity (entity.entity_id)}
			<div class="flex items-center justify-between gap-2">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-1.5">
						<span class="text-small">{iconMap[entity.entity_type] ?? '📌'}</span>
						<span class="truncate text-body font-medium text-foreground">{entity.label}</span>
					</div>
					{#if entity.description}
						<p class="mt-0.5 truncate text-small text-muted-foreground">
							{entity.description}
						</p>
					{/if}
					<p class="text-small text-muted-foreground/70">
						{m.pulse_feed_suggestion_activities({ count: entity.witness_count })} · {m.discover_followers(
							{ count: String(entity.follower_count) }
						)}
					</p>
				</div>

				<Button
					variant={entity.followed ? 'outline' : 'default'}
					size="pill"
					class={entity.followed ? 'bg-primary/10 text-primary' : ''}
					onclick={() => onFollow(entity.entity_id)}
				>
					{entity.followed ? m.pulse_feed_entity_following() : m.pulse_feed_entity_follow()}
				</Button>
			</div>
		{/each}
	</div>

	<!-- Follow all button -->
	{#if entities.some((e) => !e.followed)}
		<Button
			variant="outline"
			class="mt-3 w-full border-primary/30 text-primary hover:bg-primary/10"
			onclick={onFollowAll}
		>
			{m.pulse_feed_suggestion_follow_all()}
		</Button>
	{/if}
</div>

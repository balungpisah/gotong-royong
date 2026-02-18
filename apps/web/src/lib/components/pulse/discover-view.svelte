<script lang="ts">
	import type { EntityType, FollowableEntity } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getFeedStore } from '$lib/stores';
	import Compass from '@lucide/svelte/icons/compass';

	const feedStore = getFeedStore();

	const iconMap: Record<EntityType, string> = {
		lingkungan: '📍',
		topik: '🏷️',
		kelompok: '👥',
		lembaga: '🏢',
		warga: '👤'
	};

	const labelMap: Record<EntityType, string> = {
		lingkungan: 'Lingkungan',
		topik: 'Topik',
		kelompok: 'Kelompok',
		lembaga: 'Lembaga',
		warga: 'Warga'
	};

	/** Group entities by type. */
	const groupedEntities = $derived(() => {
		const groups = new Map<EntityType, FollowableEntity[]>();
		for (const entity of feedStore.suggestedEntities) {
			const list = groups.get(entity.entity_type) ?? [];
			list.push(entity);
			groups.set(entity.entity_type, list);
		}
		return groups;
	});

	function followAllInGroup(entities: FollowableEntity[]) {
		for (const e of entities) {
			if (!e.followed) {
				feedStore.toggleFollow(e.entity_id);
			}
		}
	}
</script>

<div class="flex flex-col gap-6">
	<!-- Discover header -->
	<div class="flex items-center gap-3">
		<div
			class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary"
		>
			<Compass class="size-5" />
		</div>
		<div>
			<h2 class="text-base font-bold text-foreground">{m.pulse_discover_title()}</h2>
			<p class="text-[11px] text-muted-foreground">{m.pulse_discover_suggested()}</p>
		</div>
	</div>

	{#if feedStore.suggestedEntities.length === 0}
		<!-- Empty state -->
		<div
			class="flex flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/60 py-12 text-center"
		>
			<div
				class="flex size-12 items-center justify-center rounded-full bg-muted/50 text-muted-foreground"
			>
				<Compass class="size-6" />
			</div>
			<p class="max-w-xs text-sm text-muted-foreground">
				{m.pulse_discover_empty()}
			</p>
		</div>
	{:else}
		<!-- Entity groups -->
		{#each [...groupedEntities()] as [entityType, entities] (entityType)}
			<section class="rounded-xl border border-border/60 bg-card p-4">
				<!-- Group header -->
				<div class="flex items-center justify-between gap-2">
					<h3 class="flex items-center gap-1.5 text-sm font-semibold text-foreground">
						<span>{iconMap[entityType]}</span>
						<span>{labelMap[entityType]}</span>
					</h3>
					{#if entities.some((e) => !e.followed)}
						<button
							onclick={() => followAllInGroup(entities)}
							class="text-[10px] font-semibold text-primary hover:underline"
						>
							{m.pulse_discover_follow_all_group()}
						</button>
					{/if}
				</div>

				<!-- Entity list -->
				<div class="mt-3 space-y-2.5">
					{#each entities as entity (entity.entity_id)}
						<div class="flex items-center justify-between gap-3">
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm font-medium text-foreground">
									{entity.label}
								</p>
								{#if entity.description}
									<p class="mt-0.5 truncate text-[10px] text-muted-foreground">
										{entity.description}
									</p>
								{/if}
								<p class="text-[10px] text-muted-foreground/70">
									{m.pulse_feed_suggestion_activities({ count: entity.witness_count })} · {entity.follower_count} pengikut
								</p>
							</div>

							<button
								onclick={() => feedStore.toggleFollow(entity.entity_id)}
								class="shrink-0 rounded-full px-3 py-1 text-[10px] font-semibold transition-colors
									{entity.followed
									? 'bg-primary/10 text-primary'
									: 'bg-primary text-primary-foreground hover:bg-primary/90'}"
							>
								{entity.followed ? m.pulse_feed_entity_following() : m.pulse_feed_entity_follow()}
							</button>
						</div>
					{/each}
				</div>
			</section>
		{/each}

		<!-- Future sections (placeholder) -->
		<section class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-4 text-center">
			<p class="text-[11px] text-muted-foreground/60">
				🔥 {m.pulse_discover_trending()} — segera hadir
			</p>
		</section>

		<section class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-4 text-center">
			<p class="text-[11px] text-muted-foreground/60">
				📍 {m.pulse_discover_nearby()} — segera hadir
			</p>
		</section>
	{/if}
</div>

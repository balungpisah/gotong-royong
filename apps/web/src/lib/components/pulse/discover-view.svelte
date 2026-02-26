<script lang="ts">
	import type { EntityType, FollowableEntity } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getFeedStore, getGroupStore } from '$lib/stores';
	import { goto } from '$app/navigation';
	import { GroupCard } from '$lib/components/kelompok';
	import * as Card from '$lib/components/ui/card';
	import Compass from '@lucide/svelte/icons/compass';
	import Masonry from 'svelte-bricks';
	import { Button } from '$lib/components/ui/button';
	const feedStore = getFeedStore();
	const groupStore = getGroupStore();

	const iconMap: Record<EntityType, string> = {
		lingkungan: '📍',
		topik: '🏷️',
		kelompok: '👥',
		lembaga: '🏢',
		warga: '👤'
	};

	const labelMap = $derived({
		lingkungan: m.entity_type_lingkungan(),
		topik: m.entity_type_topik(),
		kelompok: m.entity_type_kelompok(),
		lembaga: m.entity_type_lembaga(),
		warga: m.entity_type_warga()
	} as Record<EntityType, string>);

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

	/** Flattened array for Masonry keying. */
	const masonryItems = $derived(
		[...groupedEntities()].map(([entityType, entities]) => ({
			id: entityType,
			entityType,
			entities
		}))
	);

	/** Placeholder cards for the masonry grid. */
	const placeholderItems = [
		{ id: 'trending', emoji: '🔥', label: () => m.pulse_discover_trending() },
		{ id: 'nearby', emoji: '📍', label: () => m.pulse_discover_nearby() }
	];

	function followAllInGroup(entities: FollowableEntity[]) {
		for (const e of entities) {
			if (!e.followed) {
				feedStore.toggleFollow(e.entity_id);
			}
		}
	}

	$effect(() => {
		if (groupStore.groups.length === 0 && !groupStore.listLoading) {
			groupStore.loadGroups({ limit: 4 });
		}
	});

	const previewGroups = $derived(groupStore.groups.slice(0, 4));
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
			<h2 class="text-h3 font-bold text-foreground">{m.pulse_discover_title()}</h2>
			<p class="text-small text-muted-foreground">{m.pulse_discover_suggested()}</p>
		</div>
	</div>

	{#if feedStore.suggestedEntities.length === 0}
		<!-- Empty state -->
		<div
			class="flex flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/40 bg-muted/10 py-12 text-center"
		>
			<div
				class="flex size-12 items-center justify-center rounded-full bg-muted/50 text-muted-foreground"
			>
				<Compass class="size-6" />
			</div>
			<p class="max-w-xs text-body text-muted-foreground">
				{m.pulse_discover_empty()}
			</p>
		</div>
	{:else}
		<!-- Masonry entity groups -->
		<Masonry
			items={[...masonryItems, ...placeholderItems]}
			getId={(item) => item.id}
			minColWidth={260}
			maxColWidth={340}
			gap={16}
			animate={true}
		>
			{#snippet children({ item })}
				{#if 'entityType' in item}
					<Card.Root padding="compact">
						<!-- Group header -->
						<div class="flex items-center justify-between gap-2">
							<h3 class="flex items-center gap-1.5 text-body font-semibold text-foreground">
								<span>{iconMap[item.entityType]}</span>
								<span>{labelMap[item.entityType]}</span>
							</h3>
							{#if item.entities.some((e) => !e.followed)}
								<Button
									variant="link"
									class="h-auto p-0"
									onclick={() => followAllInGroup(item.entities)}
								>
									{m.pulse_discover_follow_all_group()}
								</Button>
							{/if}
						</div>

						<!-- Entity list -->
						<div class="mt-3 space-y-2.5">
							{#each item.entities as entity (entity.entity_id)}
								<div class="flex items-center justify-between gap-3">
									<div class="min-w-0 flex-1">
										<p class="truncate text-body font-medium text-foreground">
											{entity.label}
										</p>
										{#if entity.description}
											<p class="mt-0.5 truncate text-small text-muted-foreground">
												{entity.description}
											</p>
										{/if}
										<p class="text-small text-muted-foreground/70">
											{m.pulse_feed_suggestion_activities({ count: entity.witness_count })} · {m.discover_followers({ count: String(entity.follower_count) })}
										</p>
									</div>

									<Button
										variant={entity.followed ? 'outline' : 'default'}
										size="pill"
										class={entity.followed ? 'bg-primary/10 text-primary' : ''}
										onclick={() => feedStore.toggleFollow(entity.entity_id)}
									>
										{entity.followed ? m.pulse_feed_entity_following() : m.pulse_feed_entity_follow()}
									</Button>
								</div>
							{/each}
						</div>
					</Card.Root>
				{:else}
					<!-- Placeholder cards (trending, nearby) -->
					<section class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-4 text-center">
						<p class="text-small text-muted-foreground/60">
							{item.emoji} {item.label()} — {m.common_coming_soon()}
						</p>
					</section>
				{/if}
			{/snippet}
		</Masonry>
	{/if}

	<!-- Groups (Kelompok & Lembaga) — always visible, even for new users -->
	<Card.Root padding="compact">
		<div class="flex items-center justify-between gap-3">
			<div>
				<h3 class="text-body font-bold text-foreground">{m.group_discover_section_title()}</h3>
				<p class="mt-0.5 text-small text-muted-foreground/80">{m.group_discover_section_subtitle()}</p>
			</div>
			<a href="/komunitas/kelompok" class="text-small font-semibold text-primary hover:underline">
				{m.common_view_all({ count: previewGroups.length })}
			</a>
		</div>

		{#if previewGroups.length === 0}
			<p class="mt-3 text-small text-muted-foreground/80">{m.group_empty_discover()}</p>
		{:else}
			<div class="mt-3 grid gap-3 sm:grid-cols-2">
				{#each previewGroups as group (group.group_id)}
					<GroupCard
						{group}
						onJoin={(groupId) => groupStore.joinGroup(groupId)}
						onRequestJoin={(groupId) => goto(`/komunitas/kelompok/${groupId}`)}
					/>
				{/each}
			</div>
		{/if}
	</Card.Root>
</div>

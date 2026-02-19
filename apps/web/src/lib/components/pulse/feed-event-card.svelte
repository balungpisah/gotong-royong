<script lang="ts">
	import type { FeedItem, UrgencyBadge } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import EntityPill from './entity-pill.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	interface Props {
		item: FeedItem;
		selected?: boolean;
		onclick?: () => void;
	}

	let { item, selected = false, onclick }: Props = $props();

	// ── Track badge variant (same pattern as PulseActivityCard) ───
	const trackVariantMap: Record<string, BadgeVariant> = {
		tuntaskan: 'track-tuntaskan',
		wujudkan: 'track-wujudkan',
		telusuri: 'track-telusuri',
		rayakan: 'track-rayakan',
		musyawarah: 'track-musyawarah'
	};

	// ── Urgency badge mapping ─────────────────────────────────────
	const urgencyVariantMap: Record<UrgencyBadge, BadgeVariant> = {
		baru: 'destructive',
		voting: 'warning',
		selesai: 'success',
		ramai: 'info'
	};

	const urgencyLabelMap: Record<UrgencyBadge, () => string> = {
		baru: () => m.pulse_feed_badge_baru(),
		voting: () => m.pulse_feed_badge_voting(),
		selesai: () => m.pulse_feed_badge_selesai(),
		ramai: () => m.pulse_feed_badge_ramai()
	};

	// ── Event type emoji mapping ──────────────────────────────────
	const eventEmojiMap: Record<string, string> = {
		created: '📢',
		joined: '🙋',
		checkpoint: '📍',
		vote_opened: '🗳️',
		evidence: '📎',
		resolved: '✅',
		galang_milestone: '💰',
		community_note: '📝'
	};

	// ── Time formatting ───────────────────────────────────────────
	function timeAgo(dateStr: string): string {
		const now = Date.now();
		const then = new Date(dateStr).getTime();
		const diff = Math.max(0, now - then);
		const minutes = Math.floor(diff / 60000);
		if (minutes < 1) return 'Baru saja';
		if (minutes < 60) return `${minutes}m lalu`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}j lalu`;
		const days = Math.floor(hours / 24);
		if (days < 7) return `${days}h lalu`;
		return new Date(dateStr).toLocaleDateString('id-ID', { day: 'numeric', month: 'short' });
	}

	function handleKeydown(e: KeyboardEvent) {
		if (onclick && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onclick();
		}
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	role={onclick ? 'button' : 'article'}
	tabindex={onclick ? 0 : undefined}
	onclick={onclick}
	onkeydown={onclick ? handleKeydown : undefined}
	class="group rounded-xl p-4 transition
		{selected
			? 'border border-primary/50 bg-primary/5 shadow-sm'
			: 'border border-border/50 bg-card hover:border-border hover:shadow-sm'}
		{onclick ? 'cursor-pointer' : ''}"
>
	<!-- Repost header -->
	{#if item.repost}
		<p class="mb-1.5 text-[11px] text-muted-foreground">
			{#if item.repost.reposter_avatar}
				<img
					src={item.repost.reposter_avatar}
					alt=""
					class="mr-1 inline-block size-4 rounded-full"
				/>
			{/if}
			<span class="font-medium">{item.repost.reposter_name}</span>
			{item.repost.action_verb}
		</p>
	{/if}

	<!-- Event headline row -->
	<div class="flex items-center gap-1.5">
		<span class="text-xs">
			{eventEmojiMap[item.latest_event.event_type] ?? '📌'}
		</span>
		<span class="text-xs text-muted-foreground">
			{item.latest_event.verb}
		</span>
		<span class="text-[10px] text-muted-foreground/60">·</span>
		<span class="text-[10px] text-muted-foreground/60">
			{timeAgo(item.latest_event.timestamp)}
		</span>

		{#if item.urgency}
			<span class="ml-auto">
				<Badge variant={urgencyVariantMap[item.urgency]} class="text-[9px] px-1.5 py-0">
					{urgencyLabelMap[item.urgency]()}
				</Badge>
			</span>
		{/if}
	</div>

	<!-- Witness title -->
	<h3 class="mt-1.5 text-sm font-semibold text-foreground line-clamp-2">
		{item.title}
	</h3>

	<!-- Snippet -->
	{#if item.latest_event.snippet}
		<p class="mt-1 text-xs leading-relaxed text-muted-foreground line-clamp-3">
			{item.latest_event.snippet}
		</p>
	{/if}

	<!-- Meta row: track badge + avatar stack + member count + collapsed count -->
	<div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
		{#if item.track_hint}
			<Badge
				variant={trackVariantMap[item.track_hint] ?? 'secondary'}
				class="text-[10px]"
			>
				{item.track_hint}
			</Badge>
		{/if}

		<!-- Avatar stack -->
		{#if item.members_preview.length > 0}
			<div class="flex -space-x-1.5">
				{#each item.members_preview.slice(0, 4) as member (member.user_id)}
					{#if member.avatar_url}
						<img
							src={member.avatar_url}
							alt={member.name}
							class="size-5 rounded-full border-2 border-card"
						/>
					{:else}
						<span
							class="flex size-5 items-center justify-center rounded-full border-2 border-card bg-muted text-[8px] font-bold text-muted-foreground"
						>
							{member.name.charAt(0)}
						</span>
					{/if}
				{/each}
			</div>
		{/if}

		<span class="inline-flex items-center gap-1">
			<UsersIcon class="size-3" />
			{item.member_count}
		</span>

		{#if item.collapsed_count > 0}
			<span class="text-[10px] text-muted-foreground/70">
				+{item.collapsed_count} aktivitas
			</span>
		{/if}
	</div>

	<!-- Entity pills row -->
	{#if item.entity_tags.length > 0}
		<div class="mt-2 flex flex-wrap gap-1">
			{#each item.entity_tags as tag (tag.entity_id)}
				<EntityPill {tag} />
			{/each}
		</div>
	{/if}
</div>

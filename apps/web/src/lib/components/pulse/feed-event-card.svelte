<script lang="ts">
	import type { FeedItem, UrgencyBadge } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import EntityPill from './entity-pill.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import FlameIcon from '@lucide/svelte/icons/flame';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import ZapIcon from '@lucide/svelte/icons/zap';

	interface Props {
		item: FeedItem;
		selected?: boolean;
		onclick?: () => void;
	}

	let { item, selected = false, onclick }: Props = $props();

	// ── Sentiment → shadow color (design-system tokens only) ────────
	// Maps the LLM-extracted mood to a CSS custom property. The color
	// shows as a soft glow shadow around the card — felt before seen.
	const sentimentColorMap: Record<string, string> = {
		angry:       'var(--c-bahaya)',         // #c62828 danger red
		hopeful:     'var(--c-berhasil)',        // #2e7d32 success green
		urgent:      'var(--c-peringatan)',      // #e65100 warning orange
		celebratory: 'var(--t-rayakan)',         // #f57f17 celebration gold
		sad:         'var(--v-mid)',             // #546e7a muted slate
		curious:     'var(--t-telusuri)',        // #6a1b9a explore purple
		fun:         'var(--c-api-terang)'       // #d2691e warm amber
	};

	// Fallback: track-hint based color (legacy, kept for cards without sentiment)
	const trackColorMap: Record<string, string> = {
		tuntaskan:  'var(--t-tuntaskan)',
		wujudkan:   'var(--t-wujudkan)',
		telusuri:   'var(--t-telusuri)',
		rayakan:    'var(--t-rayakan)',
		musyawarah: 'var(--t-musyawarah)'
	};

	// Resolve: sentiment-first → track-fallback → neutral
	const moodColor = $derived(
		item.sentiment
			? (sentimentColorMap[item.sentiment] ?? 'var(--c-batu)')
			: item.track_hint
				? (trackColorMap[item.track_hint] ?? 'var(--c-batu)')
				: 'var(--c-batu)'
	);

	// ── Shadow styles — colored glow from mood ──────────────────────
	// Resting: very soft 20% opacity glow. Hover: lifts + 30% glow.
	const restShadow = $derived(
		`0 1px 3px 0 color-mix(in srgb, ${moodColor} 12%, transparent), 0 4px 12px -2px color-mix(in srgb, ${moodColor} 18%, transparent)`
	);
	const hoverShadow = $derived(
		`0 2px 6px 0 color-mix(in srgb, ${moodColor} 18%, transparent), 0 8px 20px -4px color-mix(in srgb, ${moodColor} 28%, transparent)`
	);
	const selectedShadow = $derived(
		`0 2px 8px 0 color-mix(in srgb, ${moodColor} 25%, transparent), 0 8px 24px -4px color-mix(in srgb, ${moodColor} 35%, transparent)`
	);

	// ── Urgency badge mapping ───────────────────────────────────────
	const urgencyVariantMap: Record<UrgencyBadge, BadgeVariant> = {
		baru:    'destructive',
		voting:  'warning',
		selesai: 'success',
		ramai:   'info'
	};

	const urgencyLabelMap: Record<UrgencyBadge, () => string> = {
		baru:    () => m.pulse_feed_badge_baru(),
		voting:  () => m.pulse_feed_badge_voting(),
		selesai: () => m.pulse_feed_badge_selesai(),
		ramai:   () => m.pulse_feed_badge_ramai()
	};

	// ── Event type emoji ────────────────────────────────────────────
	const eventEmojiMap: Record<string, string> = {
		created:          '📢',
		joined:           '🙋',
		checkpoint:       '📍',
		vote_opened:      '🗳️',
		evidence:         '📎',
		resolved:         '✅',
		galang_milestone: '💰',
		community_note:   '📝'
	};

	// ── Source label ────────────────────────────────────────────────
	const sourceLabel: Record<string, string> = {
		ikutan:   'Ikutan',
		terlibat: 'Terlibat',
		sekitar:  'Sekitar'
	};

	// ── Activity heat ──────────────────────────────────────────────
	const activityHeat = $derived(
		item.collapsed_count > 5 ? 'hot'
		: item.collapsed_count > 2 ? 'warm'
		: 'cool'
	);

	// ── Time formatting ─────────────────────────────────────────────
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
	{onclick}
	onkeydown={onclick ? handleKeydown : undefined}
	class="group relative overflow-hidden rounded-xl transition-all duration-200
		{selected
			? 'border border-border/40 ring-1 ring-primary/10'
			: 'border border-border/20 hover:border-border/40'}
		{onclick ? 'cursor-pointer' : ''}
		bg-card"
	style="box-shadow: {selected ? selectedShadow : restShadow};"
	onmouseenter={(e) => { if (!selected) e.currentTarget.style.boxShadow = hoverShadow; }}
	onmouseleave={(e) => { if (!selected) e.currentTarget.style.boxShadow = restShadow; }}
>
	<!-- ── Cover image — edge-to-edge, optional ──────────────────── -->
	{#if item.cover_url}
		<img
			src={item.cover_url}
			alt=""
			class="w-full object-cover"
			style="max-height: 200px;"
			loading="lazy"
		/>
	{/if}

	<!-- ── Card body ─────────────────────────────────────────────── -->
	<div class="px-5 pt-4 pb-5">

		<!-- Top row: urgency badge + event emoji -->
		<div class="mb-3 flex items-center justify-between">
			{#if item.urgency}
				<Badge variant={urgencyVariantMap[item.urgency]} class="text-[9px] px-1.5 py-0">
					{urgencyLabelMap[item.urgency]()}
				</Badge>
			{:else}
				<span></span>
			{/if}
			<span class="text-base select-none opacity-50">
				{eventEmojiMap[item.latest_event.event_type] ?? '📌'}
			</span>
		</div>

		<!-- Repost attribution -->
		{#if item.repost}
			<p class="mb-2.5 flex items-center gap-1.5 text-[11px] text-muted-foreground/70">
				{#if item.repost.reposter_avatar}
					<img src={item.repost.reposter_avatar} alt="" class="inline-block size-4 rounded-full" />
				{/if}
				<span class="font-medium">{item.repost.reposter_name}</span>
				<span>{item.repost.action_verb}</span>
			</p>
		{/if}

		<!-- Hook line — the bold opening, the curiosity driver -->
		{#if item.hook_line}
			<p class="text-[15px] font-bold leading-snug text-foreground line-clamp-3">
				{item.hook_line}
			</p>
			<!-- Title — demoted to muted subtitle when hook is present -->
			<p class="mt-1 text-[12.5px] font-medium leading-snug text-muted-foreground/70 line-clamp-2">
				{item.title}
			</p>
		{:else}
			<!-- No hook — title takes the lead -->
			<h3 class="text-[15px] font-bold leading-snug text-foreground line-clamp-2">
				{item.title}
			</h3>
		{/if}

		<!-- Event meta — verb + time -->
		<div class="mt-2 flex items-center gap-1.5">
			<span class="text-[11px] text-muted-foreground/60">{item.latest_event.verb}</span>
			<span class="text-[10px] text-muted-foreground/25">·</span>
			<span class="text-[10px] text-muted-foreground/45">{timeAgo(item.latest_event.timestamp)}</span>
		</div>

		<!-- Body — AI-summarized narrative from the saksi conversation -->
		{#if item.body}
			<p class="mt-2.5 text-[13px] leading-[1.7] text-muted-foreground/80 line-clamp-4">
				{item.body}
			</p>
		{:else if !item.hook_line && item.latest_event.snippet}
			<!-- Snippet fallback — only if no hook_line AND no body -->
			<p class="mt-2.5 text-[12.5px] leading-[1.7] text-muted-foreground/70 line-clamp-3">
				{item.latest_event.snippet}
			</p>
		{/if}

		<!-- Entity pills -->
		{#if item.entity_tags.length > 0}
			<div class="mt-3 flex flex-wrap gap-1.5">
				{#each item.entity_tags as tag (tag.entity_id)}
					<EntityPill {tag} />
				{/each}
			</div>
		{/if}

		<!-- ── Footer ────────────────────────────────────────────── -->
		<div class="mt-4 flex items-center gap-2">
			<!-- Avatar stack -->
			{#if item.members_preview.length > 0}
				<div class="flex -space-x-1.5">
					{#each item.members_preview.slice(0, 4) as member (member.user_id)}
						{#if member.avatar_url}
							<img
								src={member.avatar_url}
								alt={member.name}
								class="size-5 rounded-full border-[1.5px] border-card object-cover"
							/>
						{:else}
							<span
								class="flex size-5 items-center justify-center rounded-full border-[1.5px] border-card bg-muted text-[8px] font-medium text-muted-foreground"
							>
								{member.name.charAt(0)}
							</span>
						{/if}
					{/each}
				</div>
			{/if}

			<!-- Member count -->
			<span class="inline-flex items-center gap-0.5 text-[10px] text-muted-foreground/60">
				<UsersIcon class="size-2.5" />
				{item.member_count}
			</span>

			<!-- Activity heat -->
			{#if activityHeat !== 'cool'}
				<span
					class="inline-flex items-center gap-0.5 text-[10px] font-medium
						{activityHeat === 'hot' ? 'text-destructive/80' : 'text-amber-500/80'}"
				>
					<FlameIcon class="size-2.5" />
					{activityHeat === 'hot' ? 'Ramai' : 'Aktif'}
				</span>
			{/if}

			<!-- Collapsed count -->
			{#if item.collapsed_count > 0}
				<span class="inline-flex items-center gap-0.5 text-[10px] text-muted-foreground/40">
					<ZapIcon class="size-2.5" />
					+{item.collapsed_count}
				</span>
			{/if}

			<div class="flex-1"></div>

			<!-- Source -->
			{#if item.source}
				<span class="text-[9px] font-medium uppercase tracking-wider text-muted-foreground/40">
					{sourceLabel[item.source] ?? item.source}
				</span>
			{/if}

			<!-- Hover actions -->
			<div class="flex items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
				<button
					class="inline-flex items-center justify-center rounded-md p-1 text-muted-foreground/50 transition hover:bg-muted/60 hover:text-foreground"
					onclick={(e) => e.stopPropagation()}
					aria-label="Pantau"
				>
					<EyeIcon class="size-3" />
				</button>
				<button
					class="inline-flex items-center justify-center rounded-md p-1 text-muted-foreground/50 transition hover:bg-muted/60 hover:text-foreground"
					onclick={(e) => e.stopPropagation()}
					aria-label="Simpan"
				>
					<BookmarkIcon class="size-3" />
				</button>
			</div>
		</div>
	</div>
</div>

<script lang="ts">
	import type { FeedItem, UrgencyBadge } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import EntityPill from './entity-pill.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import MessageCircleIcon from '@lucide/svelte/icons/message-circle';
	import SignalChipBar from './signal-chip-bar.svelte';
	import Tip from '$lib/components/ui/tip.svelte';

	interface Props {
		item: FeedItem;
		selected?: boolean;
		onclick?: () => void;
		onToggleMonitor?: () => void;
		onShare?: () => void;
	}

	let { item, selected = false, onclick, onToggleMonitor, onShare }: Props = $props();

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

	// ── Pulse glow — card "breathes" when people are active ────────
	const isAlive = $derived((item.active_now ?? 0) > 0);

	// ── Countdown — real deadline urgency ───────────────────────────
	function getCountdown(deadline: string | undefined): { label: string; urgency: 'chill' | 'warm' | 'hot' } | null {
		if (!deadline) return null;
		const remaining = new Date(deadline).getTime() - Date.now();
		if (remaining <= 0) return { label: 'Berakhir!', urgency: 'hot' };
		const hours = Math.floor(remaining / 3600000);
		const minutes = Math.floor((remaining % 3600000) / 60000);
		if (hours >= 48) {
			const days = Math.floor(hours / 24);
			return { label: `${days} hari lagi`, urgency: 'chill' };
		}
		if (hours >= 24) return { label: `${hours}j ${minutes}m`, urgency: 'warm' };
		if (hours >= 1) return { label: `${hours}j ${minutes}m`, urgency: 'hot' };
		return { label: `${minutes}m lagi`, urgency: 'hot' };
	}

	const countdown = $derived(getCountdown(item.deadline));

	// ── Quorum progress ────────────────────────────────────────────
	const quorumPercent = $derived(
		item.quorum_target && item.quorum_current
			? Math.min(100, Math.round((item.quorum_current / item.quorum_target) * 100))
			: null
	);
	const quorumRemaining = $derived(
		item.quorum_target && item.quorum_current
			? Math.max(0, item.quorum_target - item.quorum_current)
			: null
	);

	// ── Story Peek — continuous smooth scroll ───────────────────────
	// Pure CSS animation scrolls a vertical strip of messages upward at
	// a constant pace. The strip is duplicated for seamless loop wrap.
	// 3-line window (54px) — long messages naturally get more reading time.
	// Duration scales with message count: more messages = longer cycle.
	// contain:strict isolates inner animation from masonry ResizeObserver.
	const peekMessages = $derived(item.peek_messages ?? []);
	const hasPeek = $derived(peekMessages.length >= 2);
	// ~4s per message feels natural — long messages scroll through slowly
	const peekDuration = $derived(peekMessages.length * 4);
	// Click-to-expand: toggle between compact (54px / ~3 lines) and tall (140px / ~8 lines)
	let peekExpanded = $state(false);

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
		{isAlive && !selected ? 'animate-pulse-glow' : ''}
		bg-card"
	style="--pulse-color: {moodColor}; box-shadow: {selected ? selectedShadow : isAlive ? 'none' : restShadow};"
	onmouseenter={(e) => { if (!selected && !isAlive) e.currentTarget.style.boxShadow = hoverShadow; }}
	onmouseleave={(e) => { if (!selected && !isAlive) e.currentTarget.style.boxShadow = restShadow; }}
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
	<div class="px-5 pt-4 pb-5 min-w-0 overflow-hidden">

		<!-- Top row: urgency badge (with live count merged) + event emoji -->
		<div class="mb-3 flex items-center gap-1.5">
			{#if item.urgency}
				<Badge variant={urgencyVariantMap[item.urgency]} class="text-[9px] px-1.5 py-0">
					{urgencyLabelMap[item.urgency]()}
				</Badge>
			{/if}

			{#if isAlive}
				<span class="inline-flex items-center gap-1 text-[9px] font-medium text-emerald-600">
					<span class="relative flex size-1.5">
						<span class="absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-75"></span>
						<span class="relative inline-flex size-1.5 rounded-full bg-emerald-500"></span>
					</span>
					{item.active_now} aktif
				</span>
			{/if}

			<span class="flex-1"></span>

			<span class="text-base select-none opacity-50">
				{eventEmojiMap[item.latest_event.event_type] ?? '📌'}
			</span>
		</div>

		<!-- Countdown + quorum strip — real deadline urgency -->
		{#if countdown || quorumPercent !== null}
			<div class="mb-3 flex flex-col gap-1.5">
				{#if countdown}
					<div class="flex items-center gap-1.5">
						<ClockIcon class="size-3 {countdown.urgency === 'hot' ? 'text-destructive' : countdown.urgency === 'warm' ? 'text-amber-500' : 'text-muted-foreground/60'}" />
						<span class="text-[10px] font-semibold
							{countdown.urgency === 'hot' ? 'text-destructive' : countdown.urgency === 'warm' ? 'text-amber-600' : 'text-muted-foreground/70'}">
							{item.deadline_label ?? 'Berakhir'}: {countdown.label}
						</span>
					</div>
				{/if}
				{#if quorumPercent !== null}
					<div class="flex items-center gap-2">
						<div class="h-1.5 flex-1 overflow-hidden rounded-full bg-muted/60">
							<div
								class="h-full rounded-full transition-all duration-500
									{quorumPercent >= 75 ? 'bg-emerald-500' : quorumPercent >= 50 ? 'bg-amber-500' : 'bg-primary'}"
								style="width: {quorumPercent}%"
							></div>
						</div>
						<span class="shrink-0 text-[9px] font-medium text-muted-foreground/60">
							{quorumRemaining} orang lagi
						</span>
					</div>
				{/if}
			</div>
		{/if}

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

		<!-- ── Story Peek — carved inset, continuous smooth scroll ── -->
		<!-- Click to toggle between compact (54px) and expanded (140px).
		     Scroll keeps running in both states. -->
		{#if hasPeek}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="mt-3 -mx-5 cursor-pointer border-y border-border/10 px-5 py-2 transition-colors hover:bg-muted/30"
				style="background: color-mix(in srgb, {moodColor} 3%, transparent);
					box-shadow: inset 0 2px 4px -1px color-mix(in srgb, {moodColor} 8%, transparent),
						inset 0 -1px 2px 0 color-mix(in srgb, {moodColor} 5%, transparent);"
				onclick={(e) => { e.stopPropagation(); peekExpanded = !peekExpanded; }}
				onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); peekExpanded = !peekExpanded; } }}
			>
				<div class="flex items-start gap-2">
					<MessageCircleIcon class="mt-0.5 size-3 shrink-0 text-muted-foreground/40" />
					<!-- Clipping window. Height transitions smoothly between states.
					     contain:strict isolates inner animation from masonry ResizeObserver. -->
					<div
						class="flex-1 overflow-hidden transition-[height] duration-300 ease-out"
						style="height: {peekExpanded ? 140 : 54}px; contain: strict;"
					>
						<!-- Vertical strip: messages × 2 for seamless loop.
						     CSS animation scrolls -50% at constant speed. -->
						<div class="peek-scroll" style="animation-duration: {peekDuration}s;">
							{#each peekMessages as msg, i (i)}
								<p class="text-[11.5px] leading-[1.5]" style="padding-bottom: 4px;">
									<span class="font-semibold text-foreground/70">{msg.author}:</span>
									<span class="text-foreground/55">{' '}{msg.text}</span>
								</p>
							{/each}
							{#each peekMessages as msg, i (`dup-${i}`)}
								<p class="text-[11.5px] leading-[1.5]" style="padding-bottom: 4px;">
									<span class="font-semibold text-foreground/70">{msg.author}:</span>
									<span class="text-foreground/55">{' '}{msg.text}</span>
								</p>
							{/each}
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- ── Row 1: Entity pills (followable context) ────────────── -->
		{#if item.entity_tags.length > 0}
			<div class="mt-3 flex flex-wrap items-center gap-1.5">
				{#each item.entity_tags as tag (tag.entity_id)}
					<EntityPill {tag} />
				{/each}
			</div>
		{/if}

		<!-- ── Row 2: Signal chips (icon-only, full-row expandable) ── -->
		{#if item.signal_counts || item.my_relation}
			<div class="mt-2">
				<SignalChipBar
					eventType={item.latest_event.event_type}
					myRelation={item.my_relation}
					signalCounts={item.signal_counts}
					{moodColor}
				/>
			</div>
		{/if}

		<!-- ── Footer — avatar stack + member count + actions ─── -->
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

			<div class="flex-1"></div>

			<!-- Actions (pantau: faint when idle, full on hover/active) -->
			<div class="flex items-center gap-0.5 {item.monitored ? '' : 'opacity-30'} transition-opacity duration-150 group-hover:opacity-100">
				<Tip text={item.monitored ? 'Berhenti pantau' : 'Pantau'}>
					<button
						class="inline-flex items-center justify-center rounded-md p-1 transition
							{item.monitored
								? 'text-primary bg-primary/10 hover:bg-primary/20'
								: 'text-muted-foreground/50 hover:bg-muted/60 hover:text-foreground'}"
						onclick={(e) => { e.stopPropagation(); onToggleMonitor?.(); }}
						aria-label={item.monitored ? 'Berhenti pantau' : 'Pantau'}
					>
						<EyeIcon class="size-3" />
					</button>
				</Tip>
				<Tip text="Simpan">
					<button
						class="inline-flex items-center justify-center rounded-md p-1 text-muted-foreground/50 transition hover:bg-muted/60 hover:text-foreground"
						onclick={(e) => e.stopPropagation()}
						aria-label="Simpan"
					>
						<BookmarkIcon class="size-3" />
					</button>
				</Tip>
				<Tip text="Bagikan">
					<button
						class="inline-flex items-center justify-center rounded-md p-1 text-muted-foreground/50 transition hover:bg-primary/10 hover:text-primary"
						onclick={(e) => { e.stopPropagation(); onShare?.(); }}
						aria-label="Bagikan"
					>
						<Share2Icon class="size-3" />
					</button>
				</Tip>
			</div>
		</div>
	</div>
</div>

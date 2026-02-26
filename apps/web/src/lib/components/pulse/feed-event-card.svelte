<script lang="ts">
	import type {
		FeedItem,
		UrgencyBadge,
		ContentSignalType,
		SignalResolutionOutcome,
		MyRelation,
		SignalCounts
	} from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import { dev } from '$app/environment';
	import EntityPill from './entity-pill.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import HeartIcon from '@lucide/svelte/icons/heart';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import MessageCircleIcon from '@lucide/svelte/icons/message-circle';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import SignalChipBar from './signal-chip-bar.svelte';
	import Tip from '$lib/components/ui/tip.svelte';
	import { getMoodColor, moodShadow } from '$lib/utils/mood-color';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import { getSignalStore, getFeedStore } from '$lib/stores';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		item: FeedItem;
		selected?: boolean;
		onclick?: () => void;
		onToggleMonitor?: () => void;
		onShare?: () => void;
	}

	let { item, selected = false, onclick, onToggleMonitor, onShare }: Props = $props();

	const signalStore = getSignalStore();
	const feedStore = getFeedStore();

	const signalRelation = $derived(signalStore.getRelation(item.witness_id));
	const signalCounts = $derived(signalStore.getCounts(item.witness_id));

	const effectiveRelation = $derived.by<MyRelation | undefined>(() => {
		if (!item.my_relation && !signalRelation) return undefined;
		const base: MyRelation = item.my_relation ?? {
			vouched: false,
			witnessed: false,
			flagged: false,
			supported: false
		};
		if (!signalRelation) return base;
		return {
			...base,
			witnessed: signalRelation.witnessed,
			flagged: signalRelation.flagged
		};
	});

	const effectiveSignalCounts = $derived.by<SignalCounts | undefined>(() => {
		if (!item.signal_counts && !signalCounts) return undefined;
		const base: SignalCounts = item.signal_counts ?? {
			vouch_positive: 0,
			vouch_skeptical: 0,
			witness_count: 0,
			dukung_count: 0,
			flags: 0
		};
		if (!signalCounts) return base;
		return {
			...base,
			witness_count: signalCounts.witness_count,
			flags: signalCounts.flags
		};
	});

	// ── Dukung (support) state — non-Tandang social action ──────────
	const isSupported = $derived(item.my_relation?.supported ?? false);
	const dukungCount = $derived(item.signal_counts?.dukung_count ?? 0);

	// ── Signal resolution state ─────────────────────────────────────
	const isTerminalWitness = $derived(item.status === 'resolved' || item.status === 'closed');
	const resolutions = $derived(signalStore.getResolutionsForWitness(item.witness_id));
	const resolutionCount = $derived(resolutions.length);

	function handleSignalChipClick(chip: ContentSignalType, value: boolean) {
		if (chip === 'saksi') {
			feedStore.autoMonitorOnAction(item.witness_id, { witnessed: value });
		} else if (chip === 'perlu_dicek') {
			feedStore.autoMonitorOnAction(item.witness_id, { flagged: value });
		}
	}

	/** Build outcome map for signal-chip-bar from resolved signals. */
	const signalOutcomes = $derived.by(() => {
		const map: Map<ContentSignalType, SignalResolutionOutcome> = new Map();
		for (const sig of resolutions) {
			map.set(sig.signal_type, sig.outcome);
		}
		return map;
	});

	// ── Mood color (shared utility) ─────────────────────────────────
	const moodColor = $derived(getMoodColor(item.sentiment, item.track_hint));

	// ── Shadow styles — colored glow from mood ──────────────────────
	// Resting: very soft 20% opacity glow. Hover: lifts + 30% glow.
	const restShadow = $derived(
		`0 1px 3px 0 color-mix(in srgb, ${moodColor} 12%, transparent), 0 4px 12px -2px color-mix(in srgb, ${moodColor} 18%, transparent)`
	);
	const hoverShadow = $derived(
		`0 2px 6px 0 color-mix(in srgb, ${moodColor} 18%, transparent), 0 8px 20px -4px color-mix(in srgb, ${moodColor} 28%, transparent)`
	);
	const selectedShadow = $derived(moodShadow(moodColor));

	// ── Urgency badge mapping ───────────────────────────────────────
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

	// ── Event type emoji ────────────────────────────────────────────
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

	// ── Pulse glow — card "breathes" when people are active ────────
	const isAlive = $derived((item.active_now ?? 0) > 0);

	// ── Countdown — real deadline urgency ───────────────────────────
	function getCountdown(
		deadline: string | undefined
	): { label: string; urgency: 'chill' | 'warm' | 'hot' } | null {
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
	const showSeedBadge = $derived(dev && item.dev_meta?.is_seed === true);

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
	data-witness-id={item.witness_id}
	class="group relative overflow-hidden rounded-xl transition-all duration-200
		{selected ? 'border-2 border-foreground/70' : 'border border-border/20 hover:border-border/40'}
		{onclick ? 'cursor-pointer' : ''}
		{isAlive && !selected ? 'animate-pulse-glow' : ''}
"
	style="--pulse-color: {moodColor};
		background: var(--color-card);
		box-shadow: {selected ? selectedShadow : isAlive ? 'none' : restShadow};
		scroll-margin-top: 5rem;"
	onmouseenter={(e) => {
		if (!selected && !isAlive) e.currentTarget.style.boxShadow = hoverShadow;
	}}
	onmouseleave={(e) => {
		if (!selected && !isAlive) e.currentTarget.style.boxShadow = restShadow;
	}}
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
			{#if showSeedBadge}
				<Badge variant="outline" class="text-[10px] px-1.5 py-0 border-dashed border-primary/50">
					SEED
				</Badge>
			{/if}

			{#if item.urgency}
				<Badge variant={urgencyVariantMap[item.urgency]} class="text-[10px] px-1.5 py-0">
					{urgencyLabelMap[item.urgency]()}
				</Badge>
			{/if}

			{#if isAlive}
				<span class="inline-flex items-center gap-1 text-[10px] font-medium text-berhasil">
					<span class="relative flex size-1.5">
						<span
							class="absolute inline-flex size-full animate-ping rounded-full bg-berhasil/60 opacity-75"
						></span>
						<span class="relative inline-flex size-1.5 rounded-full bg-berhasil"></span>
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
						<ClockIcon
							class="size-3 {countdown.urgency === 'hot'
								? 'text-destructive'
								: countdown.urgency === 'warm'
									? 'text-waspada'
									: 'text-muted-foreground/60'}"
						/>
						<span
							class="text-small font-semibold
							{countdown.urgency === 'hot'
								? 'text-destructive'
								: countdown.urgency === 'warm'
									? 'text-waspada'
									: 'text-muted-foreground/70'}"
						>
							{item.deadline_label ?? 'Berakhir'}: {countdown.label}
						</span>
					</div>
				{/if}
				{#if quorumPercent !== null}
					<div class="flex items-center gap-2">
						<div class="h-1.5 flex-1 overflow-hidden rounded-full bg-muted/60">
							<div
								class="h-full rounded-full transition-all duration-500
									{quorumPercent >= 75 ? 'bg-berhasil' : quorumPercent >= 50 ? 'bg-waspada' : 'bg-primary'}"
								style="width: {quorumPercent}%"
							></div>
						</div>
						<span class="shrink-0 text-[10px] font-medium text-muted-foreground/60">
							{quorumRemaining} orang lagi
						</span>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Repost attribution -->
		{#if item.repost}
			<p class="mb-2.5 flex items-center gap-1.5 text-small text-muted-foreground/70">
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
			<span class="text-small text-muted-foreground/60">{item.latest_event.verb}</span>
			<span class="text-small text-muted-foreground/25">·</span>
			<span class="text-small text-muted-foreground/45">{timeAgo(item.latest_event.timestamp)}</span
			>
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
				class="mt-3 -mx-5 cursor-pointer border-y border-border/10 bg-foreground/[0.04] px-5 py-2 transition-colors hover:bg-foreground/[0.07]"
				style="box-shadow: inset 0 2px 4px -1px color-mix(in srgb, {moodColor} 10%, transparent),
						inset 0 -1px 2px 0 color-mix(in srgb, {moodColor} 6%, transparent);"
				onclick={(e) => {
					e.stopPropagation();
					peekExpanded = !peekExpanded;
				}}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						e.stopPropagation();
						peekExpanded = !peekExpanded;
					}
				}}
			>
				<div class="flex items-start gap-2">
					<MessageCircleIcon class="mt-0.5 size-3 shrink-0 text-foreground/50" />
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
									<span class="font-semibold text-foreground/80">{msg.author}:</span>
									<span class="text-foreground/60"> {msg.text}</span>
								</p>
							{/each}
							{#each peekMessages as msg, i (`dup-${i}`)}
								<p class="text-[11.5px] leading-[1.5]" style="padding-bottom: 4px;">
									<span class="font-semibold text-foreground/80">{msg.author}:</span>
									<span class="text-foreground/60"> {msg.text}</span>
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
		{#if effectiveSignalCounts || effectiveRelation}
			<div class="mt-2">
				<SignalChipBar
					witnessId={item.witness_id}
					signalLabels={item.signal_labels}
					myRelation={effectiveRelation}
					signalCounts={effectiveSignalCounts}
					{signalOutcomes}
					{moodColor}
					onchipclick={handleSignalChipClick}
				/>
			</div>
		{/if}

		<!-- ── Resolution badge (completed witnesses only) ──────────── -->
		{#if isTerminalWitness && resolutionCount > 0}
			<div
				class="mt-2 flex items-center gap-1.5 rounded-md bg-berhasil/8 px-2.5 py-1.5 text-small text-berhasil"
			>
				<CheckCircle2Icon class="size-3.5 shrink-0" />
				<span class="font-medium">{m.signal_resolution_count({ count: resolutionCount })}</span>
			</div>
		{/if}

		<!-- ── Footer — avatar stack + member count + actions ─── -->
		<div class="mt-4 flex items-center gap-2">
			<!-- Avatar stack -->
			{#if item.members_preview.length > 0}
				<div class="flex -space-x-1.5">
					{#each item.members_preview.slice(0, 4) as member (member.user_id)}
						<a
							href="/profil/{member.user_id}"
							aria-label="Profil {member.name}"
							class="inline-flex rounded-full"
							data-profile-link
							data-profile-user-id={member.user_id}
							onclick={(e) => e.stopPropagation()}
						>
							<TandangAvatar
								person={{
									user_id: member.user_id,
									name: member.name,
									avatar_url: member.avatar_url
								}}
								size="xs"
								interactive={false}
								class="border-[1.5px] border-card"
							/>
						</a>
					{/each}
				</div>
			{/if}

			<!-- Member count -->
			<span class="inline-flex items-center gap-0.5 text-small text-muted-foreground/60">
				<UsersIcon class="size-2.5" />
				{item.member_count}
			</span>

			<!-- Dukung (support) button — non-Tandang social action -->
			<Button
				variant="ghost"
				size="pill"
				class={isSupported
					? 'bg-rose-500/12 text-rose-500 border border-rose-500/25 hover:bg-rose-500/18'
					: 'text-muted-foreground/50 hover:text-rose-400 hover:bg-rose-500/8 border border-transparent'}
				onclick={(e) => {
					e.stopPropagation();
					feedStore.toggleDukung(item.witness_id);
				}}
				aria-label={isSupported ? 'Batal dukung' : 'Dukung'}
				aria-pressed={isSupported}
			>
				<HeartIcon class="size-3 {isSupported ? 'fill-current' : ''}" />
				{#if dukungCount > 0}
					<span>{dukungCount}</span>
				{/if}
			</Button>

			<div class="flex-1"></div>

			<!-- Actions (pantau: faint when idle, full on hover/active) -->
			<div
				class="flex items-center gap-0.5 {item.monitored
					? ''
					: 'opacity-30'} transition-opacity duration-150 group-hover:opacity-100"
			>
				<Tip text={item.monitored ? 'Berhenti pantau' : 'Pantau'}>
					<Button
						variant="ghost"
						size="icon-sm"
						class={item.monitored
							? 'text-primary bg-primary/10 hover:bg-primary/20'
							: 'text-muted-foreground/50 hover:bg-muted/60 hover:text-foreground'}
						onclick={(e) => {
							e.stopPropagation();
							onToggleMonitor?.();
						}}
						aria-label={item.monitored ? 'Berhenti pantau' : 'Pantau'}
					>
						<EyeIcon class="size-3" />
					</Button>
				</Tip>
				<Tip text="Simpan">
					<Button
						variant="ghost"
						size="icon-sm"
						class="text-muted-foreground/50 hover:text-foreground"
						onclick={(e) => e.stopPropagation()}
						aria-label="Simpan"
					>
						<BookmarkIcon class="size-3" />
					</Button>
				</Tip>
				<Tip text="Bagikan">
					<Button
						variant="ghost"
						size="icon-sm"
						class="text-muted-foreground/50 hover:bg-primary/10 hover:text-primary"
						onclick={(e) => {
							e.stopPropagation();
							onShare?.();
						}}
						aria-label="Bagikan"
					>
						<Share2Icon class="size-3" />
					</Button>
				</Tip>
			</div>
		</div>
	</div>
</div>

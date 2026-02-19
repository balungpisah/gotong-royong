<script lang="ts">
	/**
	 * Signal Chip Bar — tandang-backed action chips for feed cards.
	 *
	 * 5 explicit chips (vouch, skeptis, PoR, bagus, perlu_dicek) + inline vote.
	 * Golden rule: every chip changes reputation (I/C/J), so every tap is explicit.
	 *
	 * PoR chip wording is contextual based on card event type:
	 *   - created / community_note → "Saya Saksi" (I witnessed this problem)
	 *   - resolved / checkpoint    → "Sudah Beres" (I confirm it's fixed)
	 *   - evidence                 → "Bukti Valid" (I validate this evidence)
	 *   - vote_opened              → hidden (use inline vote instead)
	 *   - others                   → "Saya Saksi" (default)
	 */

	import type { FeedEventType, MyRelation, SignalCounts } from '$lib/types';
	import HandshakeIcon from '@lucide/svelte/icons/handshake';
	import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import ThumbsUpIcon from '@lucide/svelte/icons/thumbs-up';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';

	interface Props {
		eventType: FeedEventType;
		myRelation?: MyRelation;
		signalCounts?: SignalCounts;
		/** Mood color CSS variable for active chip tinting. */
		moodColor?: string;
		onchipclick?: (chip: string, value: boolean) => void;
	}

	let {
		eventType,
		myRelation,
		signalCounts,
		moodColor = 'var(--c-batu)',
		onchipclick
	}: Props = $props();

	// ── PoR contextual wording ──────────────────────────────────────
	const porConfig = $derived.by(() => {
		switch (eventType) {
			case 'resolved':
			case 'checkpoint':
				return { label: 'Sudah Beres', icon: CheckCircleIcon };
			case 'evidence':
				return { label: 'Bukti Valid', icon: ShieldCheckIcon };
			case 'vote_opened':
				return null; // no PoR on vote cards — inline vote instead
			default:
				return { label: 'Saya Saksi', icon: EyeIcon };
		}
	});

	// ── Chip state derivations ──────────────────────────────────────
	const isVouched = $derived(myRelation?.vouched ?? false);
	const isWitnessed = $derived(myRelation?.witnessed ?? false);
	const isFlagged = $derived(myRelation?.flagged ?? false);
	const isQualityVoted = $derived(myRelation?.quality_voted ?? false);

	// Skeptis is: vouched with type 'skeptical'
	const isSkeptis = $derived(
		myRelation?.vouched && myRelation?.vouch_type === 'skeptical'
	);

	// ── Signal count helpers ────────────────────────────────────────
	const vouchCount = $derived(signalCounts?.vouch_positive ?? 0);
	const skeptisCount = $derived(signalCounts?.vouch_skeptical ?? 0);
	const witnessCount = $derived(signalCounts?.witness_count ?? 0);
	const qualityVotes = $derived(signalCounts?.quality_votes ?? 0);
	const flagCount = $derived(signalCounts?.flags ?? 0);

	// ── Chip click handlers ─────────────────────────────────────────
	function handleChip(chip: string, currentState: boolean) {
		onchipclick?.(chip, !currentState);
	}
</script>

<!-- Chip bar: horizontal scrollable strip.
     max-width:100% + min-width:0 prevent intrinsic chip width from
     expanding the card beyond its masonry column. -->
<div
	class="flex items-center gap-1.5 overflow-x-auto scrollbar-none"
	style="max-width: 100%; min-width: 0;"
	role="toolbar"
	aria-label="Sinyal tandang"
>
	<!-- 🤝 Vouch -->
	<button
		class="signal-chip group/chip"
		class:signal-chip--active={isVouched && !isSkeptis}
		style="--chip-active-color: var(--c-berhasil)"
		onclick={(e) => { e.stopPropagation(); handleChip('vouch', isVouched); }}
		aria-label="Vouch"
		aria-pressed={isVouched && !isSkeptis}
	>
		<HandshakeIcon class="size-3" />
		<span class="signal-chip-label">Vouch</span>
		{#if vouchCount > 0}
			<span class="signal-chip-count">{vouchCount}</span>
		{/if}
	</button>

	<!-- 🤔 Skeptis -->
	<button
		class="signal-chip group/chip"
		class:signal-chip--active={isSkeptis}
		style="--chip-active-color: var(--t-telusuri)"
		onclick={(e) => { e.stopPropagation(); handleChip('skeptis', !!isSkeptis); }}
		aria-label="Skeptis"
		aria-pressed={!!isSkeptis}
	>
		<CircleHelpIcon class="size-3" />
		<span class="signal-chip-label">Skeptis</span>
		{#if skeptisCount > 0}
			<span class="signal-chip-count">{skeptisCount}</span>
		{/if}
	</button>

	<!-- 👁️ PoR — contextual wording -->
	{#if porConfig}
		<button
			class="signal-chip group/chip"
			class:signal-chip--active={isWitnessed}
			style="--chip-active-color: var(--c-api)"
			onclick={(e) => { e.stopPropagation(); handleChip('saksi', isWitnessed); }}
			aria-label={porConfig.label}
			aria-pressed={isWitnessed}
		>
			<porConfig.icon class="size-3" />
			<span class="signal-chip-label">{porConfig.label}</span>
			{#if witnessCount > 0}
				<span class="signal-chip-count">{witnessCount}</span>
			{/if}
		</button>
	{/if}

	<!-- 👍 Bagus -->
	<button
		class="signal-chip group/chip"
		class:signal-chip--active={isQualityVoted}
		style="--chip-active-color: var(--t-wujudkan)"
		onclick={(e) => { e.stopPropagation(); handleChip('bagus', isQualityVoted); }}
		aria-label="Bagus"
		aria-pressed={isQualityVoted}
	>
		<ThumbsUpIcon class="size-3" />
		<span class="signal-chip-label">Bagus</span>
		{#if qualityVotes > 0}
			<span class="signal-chip-count">{qualityVotes}</span>
		{/if}
	</button>

	<!-- ⚠️ Perlu Dicek -->
	<button
		class="signal-chip group/chip"
		class:signal-chip--active={isFlagged}
		style="--chip-active-color: var(--c-bahaya)"
		onclick={(e) => { e.stopPropagation(); handleChip('perlu_dicek', isFlagged); }}
		aria-label="Perlu Dicek"
		aria-pressed={isFlagged}
	>
		<TriangleAlertIcon class="size-3" />
		<span class="signal-chip-label">Perlu Dicek</span>
		{#if flagCount > 0}
			<span class="signal-chip-count">{flagCount}</span>
		{/if}
	</button>
</div>

<style>
	/* ── Base chip ─────────────────────────────────────────────────── */
	.signal-chip {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		white-space: nowrap;
		border-radius: 9999px;
		padding: 0.25rem 0.5rem;
		font-size: 10px;
		font-weight: 500;
		line-height: 1;
		color: var(--v-mid);
		background: color-mix(in srgb, var(--v-wash) 60%, transparent);
		border: 1px solid color-mix(in srgb, var(--v-light) 30%, transparent);
		cursor: pointer;
		transition: all 150ms ease;
		user-select: none;
		flex-shrink: 0;
	}

	.signal-chip:hover {
		background: color-mix(in srgb, var(--v-wash) 90%, transparent);
		border-color: color-mix(in srgb, var(--v-light) 60%, transparent);
		color: var(--v-deep);
	}

	.signal-chip:active {
		transform: scale(0.96);
	}

	/* ── Active state — uses per-chip color ────────────────────────── */
	.signal-chip--active {
		background: color-mix(in srgb, var(--chip-active-color) 12%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 30%, transparent);
		color: var(--chip-active-color);
	}

	.signal-chip--active:hover {
		background: color-mix(in srgb, var(--chip-active-color) 18%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 45%, transparent);
		color: var(--chip-active-color);
	}

	/* ── Sub-elements ─────────────────────────────────────────────── */
	.signal-chip-label {
		letter-spacing: 0.01em;
	}

	.signal-chip-count {
		font-size: 9px;
		font-weight: 600;
		opacity: 0.7;
		padding-left: 0.125rem;
	}

	.signal-chip--active .signal-chip-count {
		opacity: 1;
	}

	/* ── Hide scrollbar for horizontal scroll ─────────────────────── */
	.scrollbar-none {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
	.scrollbar-none::-webkit-scrollbar {
		display: none;
	}
</style>

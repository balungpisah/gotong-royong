<script lang="ts">
	/**
	 * Signal Chip Bar — tandang-backed action chips for feed cards.
	 *
	 * Compact mode: icon-only chips + count in a dedicated row.
	 * The ENTIRE row surface is tappable to expand/collapse.
	 * Individual chip taps still fire actions (vouch/skeptis/etc.)
	 *
	 * Expanded mode: vertical list with label + description (teaching moment).
	 */

	import type { FeedEventType, MyRelation, SignalCounts } from '$lib/types';
	import { slide } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import HandshakeIcon from '@lucide/svelte/icons/handshake';
	import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import ThumbsUpIcon from '@lucide/svelte/icons/thumbs-up';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import Tip from '$lib/components/ui/tip.svelte';

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

	// ── Expand/collapse state ──────────────────────────────────────
	let expanded = $state(false);

	// ── PoR contextual wording ──────────────────────────────────────
	const porConfig = $derived.by(() => {
		switch (eventType) {
			case 'resolved':
			case 'checkpoint':
				return { label: 'Sudah Beres', desc: 'Kamu konfirmasi masalah sudah selesai', icon: CheckCircleIcon };
			case 'evidence':
				return { label: 'Bukti Valid', desc: 'Kamu validasi bukti yang diberikan', icon: ShieldCheckIcon };
			case 'vote_opened':
				return null;
			default:
				return { label: 'Saya Saksi', desc: 'Kamu melihat atau mengalami sendiri', icon: EyeIcon };
		}
	});

	// ── Chip state derivations ──────────────────────────────────────
	const isVouched = $derived(myRelation?.vouched ?? false);
	const isWitnessed = $derived(myRelation?.witnessed ?? false);
	const isFlagged = $derived(myRelation?.flagged ?? false);
	const isQualityVoted = $derived(myRelation?.quality_voted ?? false);
	const isSkeptis = $derived(
		myRelation?.vouched && myRelation?.vouch_type === 'skeptical'
	);

	// ── Signal count helpers ────────────────────────────────────────
	const vouchCount = $derived(signalCounts?.vouch_positive ?? 0);
	const skeptisCount = $derived(signalCounts?.vouch_skeptical ?? 0);
	const witnessCount = $derived(signalCounts?.witness_count ?? 0);
	const qualityVotes = $derived(signalCounts?.quality_votes ?? 0);
	const flagCount = $derived(signalCounts?.flags ?? 0);

	// ── Chip definitions (data-driven for both compact + expanded) ──
	const chips = $derived.by(() => {
		const list: Array<{
			id: string;
			icon: typeof HandshakeIcon;
			label: string;
			desc: string;
			count: number;
			active: boolean;
			activeColor: string;
		}> = [
			{
				id: 'vouch',
				icon: HandshakeIcon,
				label: 'Vouch',
				desc: 'Kamu percaya laporan ini akurat',
				count: vouchCount,
				active: isVouched && !isSkeptis,
				activeColor: 'var(--c-berhasil)'
			},
			{
				id: 'skeptis',
				icon: CircleHelpIcon,
				label: 'Skeptis',
				desc: 'Kamu ragu, perlu bukti lebih lanjut',
				count: skeptisCount,
				active: !!isSkeptis,
				activeColor: 'var(--t-telusuri)'
			}
		];

		if (porConfig) {
			list.push({
				id: 'saksi',
				icon: porConfig.icon,
				label: porConfig.label,
				desc: porConfig.desc,
				count: witnessCount,
				active: isWitnessed,
				activeColor: 'var(--c-api)'
			});
		}

		list.push(
			{
				id: 'bagus',
				icon: ThumbsUpIcon,
				label: 'Bagus',
				desc: 'Laporan berkualitas, layak diperhatikan',
				count: qualityVotes,
				active: isQualityVoted,
				activeColor: 'var(--t-wujudkan)'
			},
			{
				id: 'perlu_dicek',
				icon: TriangleAlertIcon,
				label: 'Perlu Dicek',
				desc: 'Informasi perlu diverifikasi kebenarannya',
				count: flagCount,
				active: isFlagged,
				activeColor: 'var(--c-bahaya)'
			}
		);

		return list;
	});

	// ── Handlers ────────────────────────────────────────────────────
	function handleChip(e: MouseEvent, chipId: string, currentState: boolean) {
		e.stopPropagation();
		onchipclick?.(chipId, !currentState);
	}

	function handleRowClick(e: MouseEvent) {
		e.stopPropagation();
		// Only expand if the click wasn't on a chip button
		const target = e.target as HTMLElement;
		if (!target.closest('.signal-chip')) {
			expanded = !expanded;
		}
	}
</script>

<!-- ── Compact row: entire surface is tappable to expand ──────── -->
<div
	class="signal-bar"
	class:signal-bar--expanded={expanded}
	onclick={handleRowClick}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); expanded = !expanded; } }}
	role="toolbar"
	tabindex="0"
	aria-label="Sinyal tandang — ketuk untuk penjelasan"
>
	{#each chips as chip (chip.id)}
		<Tip text={chip.label} side="bottom">
			<button
				class="signal-chip"
				class:signal-chip--active={chip.active}
				style="--chip-active-color: {chip.activeColor}"
				onclick={(e) => handleChip(e, chip.id, chip.active)}
				aria-label={chip.label}
				aria-pressed={chip.active}
			>
				<chip.icon class="size-3" />
				{#if chip.count > 0}
					<span class="signal-chip-count">{chip.count}</span>
				{/if}
			</button>
		</Tip>
	{/each}

	<!-- Chevron hint (visual only, entire row is the trigger) -->
	<span class="signal-bar-chevron" aria-hidden="true">
		<ChevronDownIcon
			class="size-3 transition-transform duration-200"
			style="transform: rotate({expanded ? 180 : 0}deg)"
		/>
	</span>
</div>

<!-- ── Expanded panel: vertical list with descriptions ────────── -->
{#if expanded}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="signal-expanded"
		role="group"
		aria-label="Penjelasan sinyal"
		transition:slide={{ duration: 250, easing: quintOut }}
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => { if (e.key === 'Escape') { e.stopPropagation(); expanded = false; } }}
	>
		{#each chips as chip (chip.id)}
			<button
				class="signal-row"
				class:signal-row--active={chip.active}
				style="--chip-active-color: {chip.activeColor}"
				onclick={(e) => { e.stopPropagation(); onchipclick?.(chip.id, !chip.active); }}
				aria-label={chip.label}
				aria-pressed={chip.active}
			>
				<div class="signal-row-icon">
					<chip.icon class="size-3.5" />
				</div>
				<div class="signal-row-text">
					<span class="signal-row-label">{chip.label}</span>
					<span class="signal-row-desc">{chip.desc}</span>
				</div>
				{#if chip.count > 0}
					<span class="signal-row-count">{chip.count}</span>
				{/if}
			</button>
		{/each}
	</div>
{/if}

<style>
	/* ── Bar row — full-width tappable surface ────────────────────── */
	.signal-bar {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		padding: 0.3rem 0.5rem;
		border-radius: 0.5rem;
		background: color-mix(in srgb, var(--v-wash) 35%, transparent);
		border: 1px solid color-mix(in srgb, var(--v-light) 15%, transparent);
		cursor: pointer;
		transition: all 150ms ease;
		user-select: none;
	}

	.signal-bar:hover {
		background: color-mix(in srgb, var(--v-wash) 55%, transparent);
		border-color: color-mix(in srgb, var(--v-light) 30%, transparent);
	}

	.signal-bar--expanded {
		background: color-mix(in srgb, var(--v-wash) 55%, transparent);
		border-color: color-mix(in srgb, var(--v-light) 30%, transparent);
	}

	/* ── Chevron hint at the end ──────────────────────────────────── */
	.signal-bar-chevron {
		display: inline-flex;
		align-items: center;
		margin-left: auto;
		color: var(--v-mid);
		opacity: 0.35;
		transition: opacity 150ms ease;
	}

	.signal-bar:hover .signal-bar-chevron {
		opacity: 0.7;
	}

	/* ── Compact chip (icon-only) ─────────────────────────────────── */
	.signal-chip {
		display: inline-flex;
		align-items: center;
		gap: 0.2rem;
		white-space: nowrap;
		border-radius: 9999px;
		padding: 0.2rem 0.35rem;
		font-size: 10px;
		font-weight: 500;
		line-height: 1;
		color: var(--v-mid);
		background: transparent;
		border: 1px solid transparent;
		cursor: pointer;
		transition: all 150ms ease;
		user-select: none;
		flex-shrink: 0;
	}

	.signal-chip:hover {
		background: color-mix(in srgb, var(--v-wash) 90%, transparent);
		border-color: color-mix(in srgb, var(--v-light) 40%, transparent);
		color: var(--v-deep);
	}

	.signal-chip:active {
		transform: scale(0.96);
	}

	/* ── Active state ─────────────────────────────────────────────── */
	.signal-chip--active {
		background: color-mix(in srgb, var(--chip-active-color) 12%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 25%, transparent);
		color: var(--chip-active-color);
	}

	.signal-chip--active:hover {
		background: color-mix(in srgb, var(--chip-active-color) 18%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 40%, transparent);
	}

	.signal-chip-count {
		font-size: 9px;
		font-weight: 600;
		opacity: 0.7;
	}

	.signal-chip--active .signal-chip-count {
		opacity: 1;
	}

	/* ── Expanded panel ───────────────────────────────────────────── */
	.signal-expanded {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		margin-top: 0.375rem;
		padding: 0.375rem;
		border-radius: 0.5rem;
		background: color-mix(in srgb, var(--v-wash) 40%, transparent);
		border: 1px solid color-mix(in srgb, var(--v-light) 20%, transparent);
	}

	/* ── Expanded row ─────────────────────────────────────────────── */
	.signal-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.375rem 0.5rem;
		border-radius: 0.375rem;
		border: none;
		background: transparent;
		cursor: pointer;
		transition: all 150ms ease;
		text-align: left;
		width: 100%;
	}

	.signal-row:hover {
		background: color-mix(in srgb, var(--v-wash) 80%, transparent);
	}

	.signal-row:active {
		transform: scale(0.99);
	}

	.signal-row--active {
		background: color-mix(in srgb, var(--chip-active-color) 8%, transparent);
	}

	.signal-row--active:hover {
		background: color-mix(in srgb, var(--chip-active-color) 14%, transparent);
	}

	.signal-row-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		border-radius: 9999px;
		flex-shrink: 0;
		color: var(--v-mid);
		background: color-mix(in srgb, var(--v-wash) 60%, transparent);
	}

	.signal-row--active .signal-row-icon {
		color: var(--chip-active-color);
		background: color-mix(in srgb, var(--chip-active-color) 15%, transparent);
	}

	.signal-row-text {
		display: flex;
		flex-direction: column;
		gap: 0.0625rem;
		flex: 1;
		min-width: 0;
	}

	.signal-row-label {
		font-size: 11px;
		font-weight: 600;
		line-height: 1.2;
		color: var(--v-deep);
	}

	.signal-row--active .signal-row-label {
		color: var(--chip-active-color);
	}

	.signal-row-desc {
		font-size: 10px;
		line-height: 1.3;
		color: var(--v-mid);
		opacity: 0.7;
	}

	.signal-row-count {
		font-size: 11px;
		font-weight: 600;
		color: var(--v-mid);
		opacity: 0.6;
		flex-shrink: 0;
	}

	.signal-row--active .signal-row-count {
		color: var(--chip-active-color);
		opacity: 1;
	}
</style>

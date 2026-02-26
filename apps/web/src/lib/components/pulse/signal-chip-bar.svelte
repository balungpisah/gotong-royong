<script lang="ts">
	/**
	 * Signal Chip Bar — content-directed signal chips for feed cards.
	 *
	 * Content signals only: saksi, perlu_dicek (2 consequential Tandang signals).
	 * "Bagus" was renamed to "Dukung" and moved to feed card footer as a social action.
	 * Vouch/skeptis moved to TandangAvatar popover (person-directed).
	 *
	 * Compact mode: icon-only chips + count in a dedicated row.
	 * The ENTIRE row surface is tappable to expand/collapse.
	 * Individual chip taps fire actions via SignalStore.
	 *
	 * Expanded mode: vertical list with label + description (teaching moment).
	 *
	 * Pending signals show a subtle animated dot.
	 * Resolved signals show outcome indicator (check/dash/x).
	 */

	import type {
		MyRelation,
		SignalCounts,
		ContentSignalType,
		SignalResolutionOutcome,
		SignalLabels
	} from '$lib/types';
	import { getSignalStore } from '$lib/stores';
	import { safeSlide as slide } from '$lib/utils/safe-slide';
	import { quintOut } from 'svelte/easing';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import CheckIcon from '@lucide/svelte/icons/check';
	import MinusIcon from '@lucide/svelte/icons/minus';
	import XIcon from '@lucide/svelte/icons/x';
	import Tip from '$lib/components/ui/tip.svelte';

	/** Hardcoded fallback labels when signal_labels is absent (legacy data). */
	const DEFAULT_LABELS: SignalLabels = {
		saksi: { label: 'Saya Saksi', desc: 'Kamu melihat atau mengalami sendiri' },
		perlu_dicek: { label: 'Perlu Dicek', desc: 'Informasi perlu diverifikasi kebenarannya' }
	};

	interface Props {
		witnessId: string;
		/** LLM-generated contextual labels for the 3 signal chips. */
		signalLabels?: SignalLabels;
		myRelation?: MyRelation;
		signalCounts?: SignalCounts;
		/** Per-signal resolution outcomes (from SignalStore). */
		signalOutcomes?: Map<ContentSignalType, SignalResolutionOutcome>;
		/** Mood color CSS variable for active chip tinting. */
		moodColor?: string;
		onchipclick?: (chip: ContentSignalType, value: boolean) => void;
	}

	let {
		witnessId,
		signalLabels,
		myRelation,
		signalCounts,
		signalOutcomes,
		moodColor: _moodColor = 'var(--c-batu)',
		onchipclick
	}: Props = $props();

	const signalStore = getSignalStore();
	let lastFailedAction = $state<{ chipId: ContentSignalType; currentState: boolean } | null>(null);
	const signalError = $derived(signalStore.getError(witnessId));

	// ── Expand/collapse state ──────────────────────────────────────
	let expanded = $state(false);

	// ── Resolve labels: LLM-generated or fallback defaults ─────────
	const labels = $derived(signalLabels ?? DEFAULT_LABELS);

	// ── Chip state derivations ──────────────────────────────────────
	const isWitnessed = $derived(myRelation?.witnessed ?? false);
	const isFlagged = $derived(myRelation?.flagged ?? false);

	// ── Signal count helpers ────────────────────────────────────────
	const witnessCount = $derived(signalCounts?.witness_count ?? 0);
	const flagCount = $derived(signalCounts?.flags ?? 0);

	// ── Resolution outcome helpers ─────────────────────────────────
	function getOutcome(signalType: ContentSignalType): SignalResolutionOutcome | undefined {
		return signalOutcomes?.get(signalType);
	}

	function isTerminal(outcome: SignalResolutionOutcome | undefined): boolean {
		return outcome !== undefined && outcome !== 'pending';
	}

	// ── Chip definitions — 2 consequential Tandang chips ──────────────
	const chips = $derived.by(() => {
		return [
			{
				id: 'saksi' as ContentSignalType,
				icon: EyeIcon,
				label: labels.saksi.label,
				desc: labels.saksi.desc,
				count: witnessCount,
				active: isWitnessed,
				activeColor: 'var(--c-api)',
				outcome: getOutcome('saksi')
			},
			{
				id: 'perlu_dicek' as ContentSignalType,
				icon: TriangleAlertIcon,
				label: labels.perlu_dicek.label,
				desc: labels.perlu_dicek.desc,
				count: flagCount,
				active: isFlagged,
				activeColor: 'var(--c-bahaya)',
				outcome: getOutcome('perlu_dicek')
			}
		];
	});

	// ── Handlers ────────────────────────────────────────────────────
	async function handleChip(e: MouseEvent, chipId: ContentSignalType, currentState: boolean) {
		e.stopPropagation();
		try {
			await signalStore.toggleSignal(witnessId, chipId);
			lastFailedAction = null;
			onchipclick?.(chipId, !currentState);
		} catch {
			lastFailedAction = { chipId, currentState };
		}
	}

	function handleRowClick(e: MouseEvent) {
		e.stopPropagation();
		const target = e.target as HTMLElement;
		if (!target.closest('.signal-chip')) {
			expanded = !expanded;
		}
	}

	async function handleRetry(e: MouseEvent) {
		e.stopPropagation();
		if (signalStore.sending) return;

		if (lastFailedAction) {
			try {
				await signalStore.toggleSignal(witnessId, lastFailedAction.chipId);
				onchipclick?.(lastFailedAction.chipId, !lastFailedAction.currentState);
				lastFailedAction = null;
				return;
			} catch {
				return;
			}
		}

		try {
			await signalStore.refreshWitness(witnessId);
		} catch {
			// keep store error state for retry feedback
		}
	}
</script>

<!-- ── Compact row: entire surface is tappable to expand ──────── -->
<div
	class="signal-bar"
	class:signal-bar--expanded={expanded}
	onclick={handleRowClick}
	onkeydown={(e) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			e.stopPropagation();
			expanded = !expanded;
		}
	}}
	role="toolbar"
	tabindex="0"
	aria-label="Sinyal tandang — ketuk untuk penjelasan"
>
	{#each chips as chip (chip.id)}
		<Tip text={chip.label} side="bottom">
			<button
				class="signal-chip"
				class:signal-chip--active={chip.active}
				class:signal-chip--resolved={isTerminal(chip.outcome)}
				style="--chip-active-color: {chip.activeColor}"
				onclick={(e) => handleChip(e, chip.id, chip.active)}
				aria-label={chip.label}
				aria-pressed={chip.active}
				disabled={isTerminal(chip.outcome) || signalStore.sending}
			>
				<chip.icon class="size-3" />
				{#if chip.count > 0}
					<span class="signal-chip-count">{chip.count}</span>
				{/if}
				<!-- Pending indicator: subtle animated dot -->
				{#if chip.active && chip.outcome === 'pending'}
					<span class="signal-pending-dot" aria-label="Menunggu hasil"></span>
				{/if}
				<!-- Resolved indicator -->
				{#if chip.outcome === 'resolved_positive'}
					<CheckIcon class="size-2.5 text-berhasil" />
				{:else if chip.outcome === 'resolved_neutral' || chip.outcome === 'expired'}
					<MinusIcon class="size-2.5 text-muted-foreground/50" />
				{:else if chip.outcome === 'resolved_negative'}
					<XIcon class="size-2.5 text-destructive" />
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
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				e.stopPropagation();
				expanded = false;
			}
		}}
	>
		{#each chips as chip (chip.id)}
			<button
				class="signal-row"
				class:signal-row--active={chip.active}
				class:signal-row--resolved={isTerminal(chip.outcome)}
				style="--chip-active-color: {chip.activeColor}"
				onclick={(e) => {
					e.stopPropagation();
					handleChip(e, chip.id, chip.active);
				}}
				aria-label={chip.label}
				aria-pressed={chip.active}
				disabled={isTerminal(chip.outcome) || signalStore.sending}
			>
				<div class="signal-row-icon">
					<chip.icon class="size-3.5" />
				</div>
				<div class="signal-row-text">
					<span class="signal-row-label">{chip.label}</span>
					<span class="signal-row-desc">{chip.desc}</span>
				</div>
				{#if chip.active && chip.outcome === 'pending'}
					<span class="signal-pending-dot signal-pending-dot--row" aria-label="Menunggu hasil"
					></span>
				{/if}
				{#if chip.outcome === 'resolved_positive'}
					<CheckIcon class="size-3.5 shrink-0 text-berhasil" />
				{:else if chip.outcome === 'resolved_neutral' || chip.outcome === 'expired'}
					<MinusIcon class="size-3.5 shrink-0 text-muted-foreground/50" />
				{:else if chip.outcome === 'resolved_negative'}
					<XIcon class="size-3.5 shrink-0 text-destructive" />
				{:else if chip.count > 0}
					<span class="signal-row-count">{chip.count}</span>
				{/if}
			</button>
		{/each}
	</div>
{/if}

{#if signalError}
	<div class="signal-error" role="status" aria-live="polite">
		<span class="signal-error-text">{signalError}</span>
		<button
			type="button"
			class="signal-error-retry"
			onclick={handleRetry}
			disabled={signalStore.sending}
		>
			Coba lagi
		</button>
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

	.signal-chip:hover:not(:disabled) {
		background: color-mix(in srgb, var(--v-wash) 90%, transparent);
		border-color: color-mix(in srgb, var(--v-light) 40%, transparent);
		color: var(--v-deep);
	}

	.signal-chip:active:not(:disabled) {
		transform: scale(0.96);
	}

	.signal-chip:disabled {
		cursor: default;
	}

	/* ── Active state ─────────────────────────────────────────────── */
	.signal-chip--active {
		background: color-mix(in srgb, var(--chip-active-color) 12%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 25%, transparent);
		color: var(--chip-active-color);
	}

	.signal-chip--active:hover:not(:disabled) {
		background: color-mix(in srgb, var(--chip-active-color) 18%, transparent);
		border-color: color-mix(in srgb, var(--chip-active-color) 40%, transparent);
	}

	/* ── Resolved state — muted, non-interactive ─────────────────── */
	.signal-chip--resolved {
		opacity: 0.6;
	}

	.signal-chip-count {
		font-size: 9px;
		font-weight: 600;
		opacity: 0.7;
	}

	.signal-chip--active .signal-chip-count {
		opacity: 1;
	}

	/* ── Pending dot — subtle animated indicator ─────────────────── */
	.signal-pending-dot {
		width: 5px;
		height: 5px;
		border-radius: 9999px;
		background: var(--chip-active-color);
		animation: pending-pulse 2s ease-in-out infinite;
		flex-shrink: 0;
	}

	.signal-pending-dot--row {
		width: 6px;
		height: 6px;
		margin-left: auto;
	}

	@keyframes pending-pulse {
		0%,
		100% {
			opacity: 0.3;
			transform: scale(0.8);
		}
		50% {
			opacity: 1;
			transform: scale(1.2);
		}
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

	.signal-error {
		margin-top: 0.3rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		border-radius: 0.5rem;
		border: 1px solid oklch(from var(--c-bahaya) l c h / 0.25);
		background: oklch(from var(--c-bahaya) l c h / 0.06);
		padding: 0.3rem 0.45rem;
	}

	.signal-error-text {
		font-size: 10px;
		line-height: 1.2;
		color: oklch(from var(--c-bahaya) l c h / 0.92);
	}

	.signal-error-retry {
		border-radius: 999px;
		border: 1px solid oklch(from var(--c-batu) l c h / 0.25);
		padding: 0.15rem 0.45rem;
		font-size: 10px;
		font-weight: 600;
		color: var(--v-deep);
		transition: background-color 120ms ease;
	}

	.signal-error-retry:hover:not(:disabled) {
		background: color-mix(in srgb, var(--v-wash) 80%, transparent);
	}

	.signal-error-retry:disabled {
		opacity: 0.5;
		cursor: not-allowed;
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

	.signal-row:hover:not(:disabled) {
		background: color-mix(in srgb, var(--v-wash) 80%, transparent);
	}

	.signal-row:active:not(:disabled) {
		transform: scale(0.99);
	}

	.signal-row:disabled {
		cursor: default;
	}

	.signal-row--active {
		background: color-mix(in srgb, var(--chip-active-color) 8%, transparent);
	}

	.signal-row--active:hover:not(:disabled) {
		background: color-mix(in srgb, var(--chip-active-color) 14%, transparent);
	}

	.signal-row--resolved {
		opacity: 0.6;
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

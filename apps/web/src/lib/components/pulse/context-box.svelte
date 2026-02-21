<script lang="ts">
	import { safeFade as fade, safeFly as fly } from '$lib/utils/safe-slide';
	import { cubicOut, cubicIn } from 'svelte/easing';

	/** Slide-fade: translateX + opacity only, no layout changes.
	 *  The box keeps its full flex:1 width throughout. */
	function slideFade(
		_node: Element,
		{ duration = 300, easing = cubicOut, x = 60 }: { duration?: number; easing?: (t: number) => number; x?: number } = {}
	) {
		return {
			duration,
			easing,
			css: (t: number) => `transform: translateX(${(1 - t) * x}px); opacity: ${t}`
		};
	}
	import ClipboardList from '@lucide/svelte/icons/clipboard-list';
	import User from '@lucide/svelte/icons/user';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import X from '@lucide/svelte/icons/x';
	import WitnessDetailPanel from './witness-detail-panel.svelte';
	import CommunityPulse from './community-pulse.svelte';
	import SelfProfile from './self-profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import Tip from '$lib/components/ui/tip.svelte';
	import { getMoodColor, moodShadow } from '$lib/utils/mood-color';

	// ---------------------------------------------------------------------------
	// Types
	// ---------------------------------------------------------------------------

	type ContextTab = 'project' | 'self' | 'community';

	// ---------------------------------------------------------------------------
	// Props
	// ---------------------------------------------------------------------------

	interface Props {
		/** Witness detail data (when project tab is active) */
		witnessDetail: import('$lib/types').WitnessDetail | null;
		/** The selected feed item (for pinned card header in detail panel) */
		feedItem?: import('$lib/types').FeedItem | null;
		/** Whether witness detail is loading */
		detailLoading?: boolean;
		/** Whether a message is being sent */
		messageSending?: boolean;
		/** Callback to close the context box */
		onClose: () => void;
		/** Callback to send a message in witness chat */
		onSendMessage: (content: string) => void;
		/** Callback to invoke AI Stempel evaluation */
		onStempel?: () => void;
		/** Whether Stempel is currently processing */
		stempeling?: boolean;
		/** Whether the context box should be visible (external trigger) */
		active?: boolean;
		/** Selected user ID for self-profile tab */
		selectedUserId?: string | null;
	}

	let {
		witnessDetail = null,
		feedItem = null,
		detailLoading = false,
		messageSending = false,
		onClose,
		onSendMessage,
		onStempel,
		stempeling = false,
		active = false,
		selectedUserId = null,
	}: Props = $props();

	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	/** Which tab is currently active */
	let activeTab = $state<ContextTab>('project');

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	/** Whether we have witness detail content to show */
	const hasWitnessDetail = $derived(witnessDetail !== null);

	// ---------------------------------------------------------------------------
	// Mood color — visual link to selected card
	// ---------------------------------------------------------------------------

	/** Derive mood color from the selected feed item (null when on self/community tabs) */
	const moodColor = $derived(
		feedItem ? getMoodColor(feedItem.sentiment, feedItem.track_hint) : null
	);

	/** Raised shadow matching the selected card's glow */
	const panelShadow = $derived(
		moodColor ? moodShadow(moodColor) : undefined
	);


	// ---------------------------------------------------------------------------
	// Rolling sentence tab system
	// ---------------------------------------------------------------------------
	// The tab bar reads as a sentence: AKU [conn1] TANDANG [conn2] KOMUNITAS
	// where conn1 and conn2 rotate every 15s with a rolling (slide-up) animation.
	// Each tab word (AKU, TANDANG, KOMUNITAS) is clickable.
	// Connector words are decorative italic text between tabs.

	/** Slot 1 connectors — "AKU ___ TANDANG" (attitude / willingness / feeling)
	 *  Every word must make "AKU ___ TANDANG" read as a coherent phrase.
	 *  Mix of formal, casual, and Gen Z bahasa gaul for unpredictability.
	 *  50 words × 24 slot2 words = 1200 unique combinations.
	 */
	const slot1Words = [
		// -- readiness --
		'berani', 'siap', 'mampu', 'mau', 'bisa',
		// -- emotion --
		'ikhlas', 'bangga', 'semangat', 'rela', 'senang',
		// -- determination --
		'nekad', 'mantap', 'yakin', 'teguh', 'bulat',
		// -- gen Z energy 🔥 --
		'gas', 'hajar', 'gaskeun', 'gaspol', 'hayuk',
		'cusss', 'otw', 'lesgo', 'santuy', 'sabi',
		'gercep', 'auto', 'fix', 'pasti', 'langsung',
		// -- wholesome gaul --
		'ayo', 'yuk', 'mari', 'kuy', 'joss',
		'asik', 'sip', 'oke', 'done', 'bet',
		// -- humble / gentle --
		'coba', 'ikut', 'turut', 'ingin', 'niat',
		// -- pride / duty --
		'wajib', 'harus', 'perlu', 'kudu', 'musti',
		// -- communal spirit --
		'kompak', 'bareng', 'gotong', 'solid', 'satu',
	] as const;

	/** Slot 2 connectors — "TANDANG ___ KOMUNITAS" (relation to community)
	 *  Every word must make "TANDANG ___ KOMUNITAS" read as a coherent phrase.
	 *  Mix of formal prepositions, casual connectors, and gaul.
	 */
	const slot2Words = [
		// -- directional --
		'di', 'ke', 'dari',
		// -- purposive --
		'untuk', 'demi', 'bagi', 'guna',
		// -- togetherness --
		'bersama', 'bareng', 'sama',
		// -- relational --
		'dalam', 'lewat', 'via', 'melalui',
		// -- belonging --
		'milik', 'punya', 'oleh',
		// -- benefactive --
		'buat', 'dukung', 'bantu',
		// -- gaul connectors --
		'ala', 'rame', 'satu',
	] as const;

	/** Pick a random index, avoiding the current one */
	function randomIndex(pool: readonly string[], current: number): number {
		let next: number;
		do {
			next = Math.floor(Math.random() * pool.length);
		} while (next === current && pool.length > 1);
		return next;
	}

	/** Current connector indices */
	let slot1Index = $state(Math.floor(Math.random() * slot1Words.length));
	let slot2Index = $state(Math.floor(Math.random() * slot2Words.length));

	/** Animation state for rolling transition */
	let slot1Rolling = $state(false);
	let slot2Rolling = $state(false);

	/** Next word (staged during roll animation — starts same as current) */
	let slot1Next = $state(Math.floor(Math.random() * slot1Words.length));
	let slot2Next = $state(Math.floor(Math.random() * slot2Words.length));

	/** Trigger a roll for a slot: stage next word, animate, then commit */
	function rollSlot(slot: 1 | 2) {
		if (slot === 1) {
			slot1Next = randomIndex(slot1Words, slot1Index);
			slot1Rolling = true;
			setTimeout(() => {
				slot1Index = slot1Next;
				slot1Rolling = false;
			}, 350); // match CSS transition duration
		} else {
			slot2Next = randomIndex(slot2Words, slot2Index);
			slot2Rolling = true;
			setTimeout(() => {
				slot2Index = slot2Next;
				slot2Rolling = false;
			}, 400);
		}
	}

	/** Interval timer id */
	let rollTimer: ReturnType<typeof setInterval> | undefined;

	// Start/stop rolling when panel is active
	$effect(() => {
		if (active) {
			// Stagger: slot1 rolls at 0s, slot2 rolls at ~7.5s offset
			// so the sentence never changes both words at once
			let tick = 0;
			rollTimer = setInterval(() => {
				tick++;
				if (tick % 2 === 1) {
					rollSlot(1);
				} else {
					rollSlot(2);
				}
			}, 7500); // each slot changes every 15s (7.5s × 2 ticks)

			return () => {
				clearInterval(rollTimer);
				rollTimer = undefined;
			};
		}
	});

	// Also roll both on tab switch for variety
	function switchTab(tab: ContextTab) {
		if (tab === activeTab) return;
		activeTab = tab;
		rollSlot(1);
		setTimeout(() => rollSlot(2), 150); // slight stagger
	}

	// Tab metadata (icon + tooltip)
	const tabMeta: Record<ContextTab, { icon: typeof ClipboardList; tip: string }> = {
		project: { icon: ClipboardList, tip: 'Tandang — lihat detail inisiatif' },
		self: { icon: User, tip: 'Aku — profil dan reputasi' },
		community: { icon: BarChart3, tip: 'Komunitas — pulse dan statistik' },
	};

	// ---------------------------------------------------------------------------
	// Effects
	// ---------------------------------------------------------------------------

	// When witness detail arrives, auto-switch to project tab
	$effect(() => {
		if (witnessDetail) {
			activeTab = 'project';
		}
	});

	// When user profile selected, auto-switch to self tab
	$effect(() => {
		if (selectedUserId) {
			activeTab = 'self';
		}
	});
</script>

<!--
	ContextBox — polymorphic workspace panel (right half of 50/50 layout).

	Three tabs rendered as a rolling sentence:
	  AKU [connector] TANDANG [connector] KOMUNITAS
	  "I dare to take initiative in the community"

	Each capitalized word is a clickable tab.
	Connector words rotate every 15s with a slide-up rolling animation.
	Tab switching also triggers a roll for variety.

	Appears when `active` prop is true (card click / profile select).
	X button fully closes the panel.
-->

{#if active}
	<div
		class="context-box hidden lg:flex flex-col
			rounded-xl bg-card
			{moodColor && activeTab === 'project' ? 'border border-foreground/90' : 'border border-border/20 shadow-sm'}"
		style:box-shadow={panelShadow}
		in:slideFade={{ duration: 300, easing: cubicOut, x: 60 }}
		out:slideFade={{ duration: 200, easing: cubicIn, x: 60 }}
	>
		<!-- Sentence tab bar — civic affirmation
		     Grid anchors AKU / TANDANG / KOMUNITAS in fixed columns.
		     Connectors fill the flex gaps and can change without shifting anchors.
		     Close button sits in a fixed-width end column. -->
		<div class="sentence-bar">
			<!-- AKU -->
			<Tip text={tabMeta.self.tip}>
				<button
					class="sentence-tab"
					class:sentence-tab-active={activeTab === 'self'}
					onclick={() => switchTab('self')}
					aria-label="Tab Aku — profil dan reputasi"
				>
					AKU
					{#if activeTab === 'self'}
						<div class="tab-indicator" transition:fade={{ duration: 150 }}></div>
					{/if}
				</button>
			</Tip>

			<!-- Connector 1 (rolling) -->
			<span class="sentence-connector" aria-hidden="true">
				<span class="connector-roller" class:rolling={slot1Rolling}>
					<span class="connector-word">{slot1Words[slot1Index]}</span>
					<span class="connector-word">{slot1Words[slot1Next]}</span>
				</span>
			</span>

			<!-- TANDANG -->
			<Tip text={tabMeta.project.tip}>
				<button
					class="sentence-tab"
					class:sentence-tab-active={activeTab === 'project'}
					onclick={() => switchTab('project')}
					aria-label="Tab Tandang — lihat detail inisiatif"
				>
					TANDANG
					{#if activeTab === 'project'}
						<div class="tab-indicator" transition:fade={{ duration: 150 }}></div>
					{/if}
				</button>
			</Tip>

			<!-- Connector 2 (rolling) -->
			<span class="sentence-connector" aria-hidden="true">
				<span class="connector-roller" class:rolling={slot2Rolling}>
					<span class="connector-word">{slot2Words[slot2Index]}</span>
					<span class="connector-word">{slot2Words[slot2Next]}</span>
				</span>
			</span>

			<!-- KOMUNITAS -->
			<Tip text={tabMeta.community.tip}>
				<button
					class="sentence-tab"
					class:sentence-tab-active={activeTab === 'community'}
					onclick={() => switchTab('community')}
					aria-label="Tab Komunitas — pulse dan statistik"
				>
					KOMUNITAS
					{#if activeTab === 'community'}
						<div class="tab-indicator" transition:fade={{ duration: 150 }}></div>
					{/if}
				</button>
			</Tip>

			<!-- Close button -->
			<Tip text="Tutup panel">
				<button
					class="sentence-close"
					onclick={onClose}
					aria-label="Tutup panel"
				>
					<X class="size-3.5" />
				</button>
			</Tip>
		</div>

		<!-- Tab content — each tab absolutely positioned for crossfade.
		     `will-change` promotes the layer for GPU-composited fly animation
		     so heavy child mounts (WitnessDetailPanel) can't block the transition. -->
		<div class="relative min-h-0 flex-1">
			{#if activeTab === 'project'}
				<div
					class="tab-pane absolute inset-0 overflow-y-auto overflow-x-hidden"
					transition:fly={{ x: 12, duration: 200 }}
				>
					{#if hasWitnessDetail && witnessDetail}
						<WitnessDetailPanel
							detail={witnessDetail}
							{feedItem}
							onSendMessage={onSendMessage}
							{onStempel}
							sending={messageSending}
							{stempeling}
						/>
					{:else if detailLoading}
						<div class="flex h-full items-center justify-center">
							<div class="flex flex-col items-center gap-3 text-muted-foreground">
								<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
								<p class="text-xs">{m.pulse_loading_detail()}</p>
							</div>
						</div>
					{:else}
						<!-- Empty state: no tandang selected -->
						<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
							<div class="flex size-12 items-center justify-center rounded-xl bg-muted/30 text-muted-foreground">
								<ClipboardList class="size-6" />
							</div>
							<p class="text-[var(--fs-small)] text-muted-foreground">
								Pilih tandang dari feed untuk melihat detail
							</p>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'self'}
				<div
					class="tab-pane absolute inset-0 overflow-y-auto overflow-x-hidden"
					transition:fly={{ x: 12, duration: 200 }}
				>
					<SelfProfile userId={selectedUserId} />
				</div>
			{:else if activeTab === 'community'}
				<div
					class="tab-pane absolute inset-0 overflow-y-auto overflow-x-hidden"
					transition:fly={{ x: 12, duration: 200 }}
				>
					<CommunityPulse />
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	/*
	 * Context box — right-side workspace region.
	 * Sticky: stays in viewport while masonry scrolls.
	 * 50% width of parent flex container — clean half-split with feed.
	 * Enables symmetrical layout transitions (e.g. history mode swap).
	 * Internal scroll per tab content (absolute positioned).
	 */
	.context-box {
		position: sticky;
		top: 4.5rem;
		flex: 1;
		height: calc(100vh - 5.5rem);
		overflow: hidden;
		transition: box-shadow 300ms ease, border-color 300ms ease;
	}

	/* ── Sentence bar: grid anchors AKU / TANDANG / KOMUNITAS ── */
	/*
	 * Layout: [AKU] [conn1] [TANDANG] [conn2] [KOMUNITAS] [close]
	 * Tab columns use auto (shrink-wrap), connectors use 1fr (absorb slack),
	 * close button is fixed-width. Tabs never shift position.
	 */
	.sentence-bar {
		display: grid;
		grid-template-columns: auto 1fr auto 1fr auto auto;
		align-items: center;
		padding: 0.35rem 0.5rem 0;
		border-bottom: 1px solid oklch(from var(--c-batu) l c h / 0.1);
		background: oklch(from var(--c-susu) l c h / 0.35);
	}

	/* ── Sentence tab: clickable anchor word ── */
	.sentence-tab {
		position: relative;
		padding: 0.5rem 0.65rem;
		padding-bottom: 0.65rem;
		border-radius: var(--r-sm) var(--r-sm) 0 0;
		font-size: 13px;
		font-weight: 800;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--color-muted-foreground);
		cursor: pointer;
		white-space: nowrap;
		transition: color 150ms ease, background-color 150ms ease;
	}

	.sentence-tab:hover {
		color: var(--color-foreground);
		background: oklch(from var(--c-batu) l c h / 0.1);
	}

	.sentence-tab-active {
		color: var(--color-primary);
	}

	/* ── Close button (end column) ── */
	.sentence-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		border-radius: var(--r-sm);
		color: var(--color-muted-foreground);
		cursor: pointer;
		transition: background-color 150ms ease, color 150ms ease;
		margin-left: 0.25rem;
	}

	.sentence-close:hover {
		background: oklch(from var(--c-batu) l c h / 0.12);
		color: var(--color-foreground);
	}

	/* ── Connector: italic rolling word between anchored tabs ── */
	.sentence-connector {
		display: flex;
		align-items: flex-start;
		justify-content: center;
		height: 18px;
		overflow: hidden;
	}

	.connector-roller {
		display: flex;
		flex-direction: column;
		align-items: center;
		transition: transform 380ms cubic-bezier(0.22, 1, 0.36, 1);
	}

	.connector-roller.rolling {
		transform: translateY(-18px);
	}

	.connector-word {
		display: block;
		height: 18px;
		line-height: 18px;
		font-family: 'Caveat', cursive;
		font-size: 15px;
		font-weight: 500;
		color: var(--color-muted-foreground);
		opacity: 0.7;
		white-space: nowrap;
		text-align: center;
	}

	/* ── Tab pane: GPU-promoted layer for smooth fly transitions ── */
	.tab-pane {
		will-change: transform, opacity;
		contain: layout style paint;
	}

	/* ── Animated underline indicator ── */
	.tab-indicator {
		position: absolute;
		bottom: 0;
		left: 0.4rem;
		right: 0.4rem;
		height: 2.5px;
		border-radius: 1.5px;
		background: var(--color-primary);
	}
</style>

<script lang="ts">
	import { browser } from '$app/environment';
	import { motion, AnimatePresence } from '@humanspeak/svelte-motion';
	import { fly, fade } from 'svelte/transition';
	import Pin from '@lucide/svelte/icons/pin';
	import PinOff from '@lucide/svelte/icons/pin-off';
	import ClipboardList from '@lucide/svelte/icons/clipboard-list';
	import User from '@lucide/svelte/icons/user';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import X from '@lucide/svelte/icons/x';
	import WitnessDetailPanel from './witness-detail-panel.svelte';
	import CommunityPulse from './community-pulse.svelte';
	import SelfProfile from './self-profile.svelte';
	import { m } from '$lib/paraglide/messages';

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
		/** Whether witness detail is loading */
		detailLoading?: boolean;
		/** Whether a message is being sent */
		messageSending?: boolean;
		/** Callback to close the context box */
		onClose: () => void;
		/** Callback to send a message in witness chat */
		onSendMessage: (content: string) => void;
		/** Whether the context box should be visible (external trigger) */
		active?: boolean;
		/** Selected user ID for self-profile tab */
		selectedUserId?: string | null;
	}

	let {
		witnessDetail = null,
		detailLoading = false,
		messageSending = false,
		onClose,
		onSendMessage,
		active = false,
		selectedUserId = null,
	}: Props = $props();

	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	const STORAGE_KEY = 'gr-context-pinned';

	/** Which tab is currently active */
	let activeTab = $state<ContextTab>('community');

	/** Whether the context box is pinned (stays open when detail closes) */
	let pinned = $state(loadPinState());

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	/** Context box is visible when: pinned, OR external trigger is active */
	const isVisible = $derived(pinned || active);

	/** Whether we have witness detail content to show */
	const hasWitnessDetail = $derived(witnessDetail !== null);

	// ---------------------------------------------------------------------------
	// Tab definitions
	// ---------------------------------------------------------------------------

	const tabs: { id: ContextTab; label: string; icon: typeof ClipboardList }[] = [
		{ id: 'project', label: 'Proyek', icon: ClipboardList },
		{ id: 'self', label: 'Profil', icon: User },
		{ id: 'community', label: 'Komunitas', icon: BarChart3 },
	];

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

	// When detail closes while pinned, fall back to community
	$effect(() => {
		if (pinned && !witnessDetail && !selectedUserId && (activeTab === 'project' || activeTab === 'self')) {
			activeTab = 'community';
		}
	});

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	function switchTab(tab: ContextTab) {
		if (tab === activeTab) return;
		activeTab = tab;
	}

	function togglePin() {
		pinned = !pinned;
		savePinState(pinned);
		// If unpinning, always close detail so parent can reset state
		if (!pinned) {
			onClose();
		}
	}

	function handleClose() {
		if (pinned) {
			// Pinned: close detail, fall back to community
			activeTab = 'community';
			onClose();
		} else {
			onClose();
		}
	}

	// ---------------------------------------------------------------------------
	// localStorage persistence
	// ---------------------------------------------------------------------------

	function loadPinState(): boolean {
		if (!browser) return false;
		try {
			return localStorage.getItem(STORAGE_KEY) === 'true';
		} catch {
			return false;
		}
	}

	function savePinState(value: boolean) {
		if (!browser) return;
		try {
			localStorage.setItem(STORAGE_KEY, String(value));
		} catch {
			// Ignore storage errors
		}
	}
</script>

<!--
	ContextBox — polymorphic workspace panel (right side of 50/50 layout).

	Three tabs:
	  📋 Project  — witness detail (WitnessDetailPanel)
	  👤 Self     — person profile (SelfProfile)
	  🏘 Community — community pulse dashboard (CommunityPulse)

	Pin toggle: when pinned, context box stays visible even when no detail is selected.
	When pinned and nothing selected, falls back to Community tab.

	Outer container uses Svelte built-in transition (fly) for enter/exit.
	Tab content uses svelte-motion AnimatePresence for crossfade switching.
-->

{#if isVisible}
	<div
		class="context-box hidden lg:flex flex-col
			rounded-xl border border-border/20 bg-card shadow-sm"
		transition:fly={{ x: 24, duration: 250 }}
	>
		<!-- Tab bar + pin -->
		<div class="flex items-center border-b border-border/20 px-2 pt-2">
			<!-- Tabs -->
			<div class="flex flex-1 gap-0.5">
				{#each tabs as tab}
					<button
						class="tab-button"
						class:tab-active={activeTab === tab.id}
						onclick={() => switchTab(tab.id)}
						aria-label="Tab {tab.label}"
					>
						<tab.icon class="size-3.5" />
						<span class="text-[var(--fs-caption)]">{tab.label}</span>
						{#if activeTab === tab.id}
							<div
								class="tab-indicator"
								transition:fade={{ duration: 150 }}
							></div>
						{/if}
					</button>
				{/each}
			</div>

			<!-- Pin + Close buttons -->
			<div class="flex items-center gap-0.5 pb-1">
				<motion.button
					class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors
						{pinned ? 'bg-primary/10 text-primary' : 'hover:bg-muted hover:text-foreground'}"
					onclick={togglePin}
					aria-label={pinned ? 'Lepas pin' : 'Pin panel'}
					whileTap={{ scale: 0.9 }}
					whileHover={{ scale: 1.05 }}
				>
					{#if pinned}
						<Pin class="size-3.5" />
					{:else}
						<PinOff class="size-3.5" />
					{/if}
				</motion.button>

				<button
					class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors
						hover:bg-muted hover:text-foreground"
					onclick={handleClose}
					aria-label="Tutup panel"
				>
					<X class="size-3.5" />
				</button>
			</div>
		</div>

		<!-- Tab content — each tab absolutely positioned for crossfade -->
		<div class="relative min-h-0 flex-1">
			{#if activeTab === 'project'}
				<div
					class="absolute inset-0 overflow-y-auto overflow-x-hidden"
					transition:fly={{ x: 12, duration: 200 }}
				>
					{#if hasWitnessDetail && witnessDetail}
						<WitnessDetailPanel
							detail={witnessDetail}
							onClose={handleClose}
							onSendMessage={onSendMessage}
							sending={messageSending}
						/>
					{:else if detailLoading}
						<div class="flex h-full items-center justify-center">
							<div class="flex flex-col items-center gap-3 text-muted-foreground">
								<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
								<p class="text-xs">{m.pulse_loading_detail()}</p>
							</div>
						</div>
					{:else}
						<!-- Empty state: no project selected -->
						<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
							<div class="flex size-12 items-center justify-center rounded-xl bg-muted/30 text-muted-foreground">
								<ClipboardList class="size-6" />
							</div>
							<p class="text-[var(--fs-small)] text-muted-foreground">
								Pilih laporan dari feed untuk melihat detail
							</p>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'self'}
				<div
					class="absolute inset-0 overflow-y-auto overflow-x-hidden"
					transition:fly={{ x: 12, duration: 200 }}
				>
					<SelfProfile userId={selectedUserId} />
				</div>
			{:else if activeTab === 'community'}
				<div
					class="absolute inset-0 overflow-y-auto overflow-x-hidden"
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
	 * 50% width of parent flex container.
	 * Internal scroll per tab content (absolute positioned).
	 */
	.context-box {
		position: sticky;
		top: 4.5rem;
		width: 50%;
		flex-shrink: 0;
		height: calc(100vh - 5.5rem);
		overflow: hidden;
	}

	/* Tab button */
	.tab-button {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.4rem 0.65rem;
		padding-bottom: 0.55rem;
		border-radius: var(--r-sm) var(--r-sm) 0 0;
		color: var(--color-muted-foreground);
		transition: color 150ms ease, background-color 150ms ease;
		cursor: pointer;
		white-space: nowrap;
	}

	.tab-button:hover {
		color: var(--color-foreground);
		background: oklch(from var(--c-batu) l c h / 0.15);
	}

	.tab-active {
		color: var(--color-primary);
		font-weight: 600;
	}

	/* Animated underline indicator */
	.tab-indicator {
		position: absolute;
		bottom: 0;
		left: 0.5rem;
		right: 0.5rem;
		height: 2px;
		border-radius: 1px;
		background: var(--color-primary);
	}
</style>

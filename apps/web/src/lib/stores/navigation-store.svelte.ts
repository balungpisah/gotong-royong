/**
 * Navigation Store — manages tab bar state and tag-based navigation.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 */

import type { TabConfig, TagSuggestion, NavigationTag } from '../types';
import { DEFAULT_TABS, WELL_KNOWN_TAGS } from '../types/navigation';
import { iconNameForTag } from '../utils';

export class NavigationStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------
	tabs = $state<TabConfig[]>([...DEFAULT_TABS]);
	activeTabId = $state<string>('pulse');
	suggestions = $state<TagSuggestion[]>([]);
	addPanelOpen = $state(false);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------
	activeTab = $derived(this.tabs.find((t) => t.id === this.activeTabId) ?? this.tabs[0]);
	tabCount = $derived(this.tabs.length);
	dynamicTabs = $derived(this.tabs.filter((t) => !t.pinned));

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/**
	 * Add a new tab. Prevents duplicates by tag.
	 */
	addTab(config: { label: string; iconName: string; tag: NavigationTag }) {
		if (this.tabs.some((t) => t.tag === config.tag)) return;
		const id = `tag-${config.tag}`;
		this.tabs = [...this.tabs, { ...config, id, pinned: false }];
	}

	/**
	 * Remove a tab by id. Pinned tabs cannot be removed.
	 * If the removed tab was active, falls back to Pulse.
	 */
	removeTab(tabId: string) {
		const tab = this.tabs.find((t) => t.id === tabId);
		if (!tab || tab.pinned) return;
		this.tabs = this.tabs.filter((t) => t.id !== tabId);
		if (this.activeTabId === tabId) {
			this.activeTabId = 'pulse';
		}
	}

	/**
	 * Set the active tab by id.
	 */
	setActiveTab(tabId: string) {
		if (this.tabs.some((t) => t.id === tabId)) {
			this.activeTabId = tabId;
		}
	}

	/**
	 * Reorder tabs by moving a tab from one index to another.
	 */
	reorderTabs(fromIndex: number, toIndex: number) {
		if (
			fromIndex < 0 ||
			fromIndex >= this.tabs.length ||
			toIndex < 0 ||
			toIndex >= this.tabs.length
		) {
			return;
		}
		const newTabs = [...this.tabs];
		const [moved] = newTabs.splice(fromIndex, 1);
		newTabs.splice(toIndex, 0, moved);
		this.tabs = newTabs;
	}

	/**
	 * Open the "Add Tab" panel.
	 */
	openAddPanel() {
		this.loadSuggestions();
		this.addPanelOpen = true;
	}

	/**
	 * Close the "Add Tab" panel.
	 */
	closeAddPanel() {
		this.addPanelOpen = false;
	}

	/**
	 * Load mock tag suggestions from well-known tags not yet added as tabs.
	 */
	loadSuggestions() {
		const existingTags = new Set(this.tabs.map((t) => t.tag).filter(Boolean));
		this.suggestions = WELL_KNOWN_TAGS.filter((tag) => !existingTags.has(tag)).map((tag) => ({
			tag,
			label: tag.charAt(0).toUpperCase() + tag.slice(1),
			witnessCount: Math.floor(Math.random() * 8) + 1,
			source: 'ai' as const
		}));
	}
}

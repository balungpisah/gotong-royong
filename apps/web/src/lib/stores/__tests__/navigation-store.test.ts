/**
 * NavigationStore logic tests.
 *
 * Since Svelte 5 runes ($state, $derived) are compiler macros that require the
 * Svelte compiler, and vitest runs in a plain Node environment, we test the
 * store's pure logic by recreating a plain-JS version of the algorithms.
 *
 * This validates:
 * - Tab add/remove/reorder logic
 * - Duplicate prevention
 * - Pinned tab protection
 * - Active tab fallback
 * - Suggestion filtering
 *
 * The Svelte reactivity layer ($state/$derived) is tested implicitly via
 * svelte-check (type safety) and manual dev-server verification.
 */
import { describe, expect, it } from 'vitest';
import { DEFAULT_TABS, WELL_KNOWN_TAGS } from '../../types/navigation';
import type { TabConfig, NavigationTag, TagSuggestion } from '../../types/navigation';

// ---------------------------------------------------------------------------
// Plain-JS mirror of NavigationStore logic (no runes)
// ---------------------------------------------------------------------------

class NavigationStoreLogic {
	tabs: TabConfig[] = [...DEFAULT_TABS];
	activeTabId = 'pulse';
	suggestions: TagSuggestion[] = [];
	addPanelOpen = false;

	get activeTab() {
		return this.tabs.find((t) => t.id === this.activeTabId) ?? this.tabs[0];
	}

	get tabCount() {
		return this.tabs.length;
	}

	get dynamicTabs() {
		return this.tabs.filter((t) => !t.pinned);
	}

	addTab(config: { label: string; iconName: string; tag: NavigationTag }) {
		if (this.tabs.some((t) => t.tag === config.tag)) return;
		const id = `tag-${config.tag}`;
		this.tabs = [...this.tabs, { ...config, id, pinned: false }];
	}

	removeTab(tabId: string) {
		const tab = this.tabs.find((t) => t.id === tabId);
		if (!tab || tab.pinned) return;
		this.tabs = this.tabs.filter((t) => t.id !== tabId);
		if (this.activeTabId === tabId) {
			this.activeTabId = 'pulse';
		}
	}

	setActiveTab(tabId: string) {
		if (this.tabs.some((t) => t.id === tabId)) {
			this.activeTabId = tabId;
		}
	}

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

	openAddPanel() {
		this.loadSuggestions();
		this.addPanelOpen = true;
	}

	closeAddPanel() {
		this.addPanelOpen = false;
	}

	loadSuggestions() {
		const TAG_ICON_MAP: Record<string, string> = {
			tuntaskan: 'flame',
			wujudkan: 'lightbulb',
			telusuri: 'search',
			rayakan: 'party-popper',
			musyawarah: 'users'
		};
		const existingTags = new Set(this.tabs.map((t) => t.tag).filter(Boolean));
		this.suggestions = WELL_KNOWN_TAGS.filter((tag) => !existingTags.has(tag)).map((tag) => ({
			tag,
			label: tag.charAt(0).toUpperCase() + tag.slice(1),
			witnessCount: 1, // deterministic for tests
			source: 'ai' as const
		}));
	}
}

function createStore() {
	return new NavigationStoreLogic();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('NavigationStore logic', () => {
	describe('default state', () => {
		it('starts with only the Pulse tab (pinned)', () => {
			const store = createStore();
			expect(store.tabs).toHaveLength(1);
			expect(store.tabs[0].id).toBe('pulse');
			expect(store.tabs[0].pinned).toBe(true);
			expect(store.tabs[0].tag).toBeNull();
		});

		it('has pulse as the active tab', () => {
			const store = createStore();
			expect(store.activeTabId).toBe('pulse');
		});

		it('starts with no suggestions and add panel closed', () => {
			const store = createStore();
			expect(store.suggestions).toHaveLength(0);
			expect(store.addPanelOpen).toBe(false);
		});

		it('activeTab derived returns Pulse', () => {
			const store = createStore();
			expect(store.activeTab.id).toBe('pulse');
		});

		it('dynamicTabs is empty (only pinned Pulse)', () => {
			const store = createStore();
			expect(store.dynamicTabs).toHaveLength(0);
		});
	});

	describe('addTab()', () => {
		it('adds a new tab with correct id and pinned=false', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			expect(store.tabs).toHaveLength(2);
			expect(store.tabs[1].id).toBe('tag-tuntaskan');
			expect(store.tabs[1].label).toBe('Tuntaskan');
			expect(store.tabs[1].pinned).toBe(false);
			expect(store.tabs[1].tag).toBe('tuntaskan');
		});

		it('prevents duplicate tabs by tag', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			store.addTab({ label: 'Tuntaskan Again', iconName: 'flame', tag: 'tuntaskan' });
			expect(store.tabs).toHaveLength(2);
		});

		it('allows multiple different tags', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			store.addTab({ label: 'Wujudkan', iconName: 'lightbulb', tag: 'wujudkan' });
			expect(store.tabs).toHaveLength(3);
			expect(store.dynamicTabs).toHaveLength(2);
		});
	});

	describe('removeTab()', () => {
		it('removes a non-pinned tab', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			expect(store.tabs).toHaveLength(2);
			store.removeTab('tag-tuntaskan');
			expect(store.tabs).toHaveLength(1);
			expect(store.tabs[0].id).toBe('pulse');
		});

		it('does not remove a pinned tab (Pulse)', () => {
			const store = createStore();
			store.removeTab('pulse');
			expect(store.tabs).toHaveLength(1);
			expect(store.tabs[0].id).toBe('pulse');
		});

		it('resets activeTabId to pulse when the active tab is removed', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			store.setActiveTab('tag-tuntaskan');
			expect(store.activeTabId).toBe('tag-tuntaskan');
			store.removeTab('tag-tuntaskan');
			expect(store.activeTabId).toBe('pulse');
		});

		it('does nothing for a non-existent tab id', () => {
			const store = createStore();
			store.removeTab('non-existent');
			expect(store.tabs).toHaveLength(1);
		});
	});

	describe('setActiveTab()', () => {
		it('updates activeTabId when tab exists', () => {
			const store = createStore();
			store.addTab({ label: 'Telusuri', iconName: 'search', tag: 'telusuri' });
			store.setActiveTab('tag-telusuri');
			expect(store.activeTabId).toBe('tag-telusuri');
			expect(store.activeTab.id).toBe('tag-telusuri');
		});

		it('does not update for non-existent tab', () => {
			const store = createStore();
			store.setActiveTab('non-existent');
			expect(store.activeTabId).toBe('pulse');
		});
	});

	describe('reorderTabs()', () => {
		it('reorders tabs correctly', () => {
			const store = createStore();
			store.addTab({ label: 'A', iconName: 'flame', tag: 'a' });
			store.addTab({ label: 'B', iconName: 'search', tag: 'b' });
			// [pulse, a, b] → move index 2 to index 1 → [pulse, b, a]
			store.reorderTabs(2, 1);
			expect(store.tabs[1].tag).toBe('b');
			expect(store.tabs[2].tag).toBe('a');
		});

		it('ignores out-of-bounds indices', () => {
			const store = createStore();
			store.addTab({ label: 'A', iconName: 'flame', tag: 'a' });
			store.reorderTabs(-1, 0);
			store.reorderTabs(0, 5);
			expect(store.tabs).toHaveLength(2);
			expect(store.tabs[0].id).toBe('pulse');
		});
	});

	describe('openAddPanel() / closeAddPanel()', () => {
		it('opens panel and loads suggestions', () => {
			const store = createStore();
			store.openAddPanel();
			expect(store.addPanelOpen).toBe(true);
			expect(store.suggestions.length).toBe(5);
		});

		it('closes panel', () => {
			const store = createStore();
			store.openAddPanel();
			store.closeAddPanel();
			expect(store.addPanelOpen).toBe(false);
		});
	});

	describe('loadSuggestions()', () => {
		it('returns well-known tags not yet added as tabs', () => {
			const store = createStore();
			store.loadSuggestions();
			expect(store.suggestions.length).toBe(5);
			expect(store.suggestions.every((s) => s.source === 'ai')).toBe(true);
		});

		it('excludes tags that are already added', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			store.addTab({ label: 'Wujudkan', iconName: 'lightbulb', tag: 'wujudkan' });
			store.loadSuggestions();
			expect(store.suggestions.length).toBe(3);
			const suggestedTags = store.suggestions.map((s) => s.tag);
			expect(suggestedTags).not.toContain('tuntaskan');
			expect(suggestedTags).not.toContain('wujudkan');
		});

		it('returns empty when all tags are added', () => {
			const store = createStore();
			store.addTab({ label: 'Tuntaskan', iconName: 'flame', tag: 'tuntaskan' });
			store.addTab({ label: 'Wujudkan', iconName: 'lightbulb', tag: 'wujudkan' });
			store.addTab({ label: 'Telusuri', iconName: 'search', tag: 'telusuri' });
			store.addTab({ label: 'Rayakan', iconName: 'party-popper', tag: 'rayakan' });
			store.addTab({ label: 'Musyawarah', iconName: 'users', tag: 'musyawarah' });
			store.loadSuggestions();
			expect(store.suggestions.length).toBe(0);
		});
	});

	describe('DEFAULT_TABS constant', () => {
		it('defines exactly one default tab (Pulse)', () => {
			expect(DEFAULT_TABS).toHaveLength(1);
			expect(DEFAULT_TABS[0].id).toBe('pulse');
			expect(DEFAULT_TABS[0].iconName).toBe('activity');
			expect(DEFAULT_TABS[0].pinned).toBe(true);
			expect(DEFAULT_TABS[0].tag).toBeNull();
		});
	});

	describe('WELL_KNOWN_TAGS constant', () => {
		it('contains 5 track tags', () => {
			expect(WELL_KNOWN_TAGS).toHaveLength(5);
			expect(WELL_KNOWN_TAGS).toContain('tuntaskan');
			expect(WELL_KNOWN_TAGS).toContain('wujudkan');
			expect(WELL_KNOWN_TAGS).toContain('telusuri');
			expect(WELL_KNOWN_TAGS).toContain('rayakan');
			expect(WELL_KNOWN_TAGS).toContain('musyawarah');
		});
	});
});

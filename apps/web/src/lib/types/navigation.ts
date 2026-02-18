import type { FeedFilter } from './feed';

/** Tag identifier for navigation filtering. Open-ended string type. */
export type NavigationTag = string;

/** Well-known tags matching the 5 community tracks. */
export const WELL_KNOWN_TAGS = [
	'tuntaskan',
	'wujudkan',
	'telusuri',
	'rayakan',
	'musyawarah'
] as const;

export type WellKnownTag = (typeof WELL_KNOWN_TAGS)[number];

/** Configuration for a single navigation tab. */
export interface TabConfig {
	/** Unique stable identifier. */
	id: string;
	/** Display label shown in the tab bar. */
	label: string;
	/** Lucide icon name string (resolved at render time via tab-icons utility). */
	iconName: string;
	/** For tag-filtered tabs, the tag string. null for feed tabs. */
	tag: NavigationTag | null;
	/** Whether this tab can be removed by the user. Feed tabs are always pinned. */
	pinned: boolean;
	/**
	 * For feed-layer tabs, the filter value to apply.
	 * When set, clicking this tab stays on `/` and sets the feed filter
	 * instead of navigating to `/t/{tag}`.
	 */
	feedFilter?: FeedFilter;
}

/** A tag suggestion for the "Add Tab" panel. */
export interface TagSuggestion {
	/** The tag identifier. */
	tag: NavigationTag;
	/** Human-readable display label. */
	label: string;
	/** Number of witnesses with this tag. */
	witnessCount: number;
	/** How the suggestion was generated. */
	source: 'ai' | 'manual';
}

/** The default tab configuration — 4 feed layer tabs + discover. */
export const DEFAULT_TABS: TabConfig[] = [
	{ id: 'feed-semua', label: 'Semua', iconName: 'activity', tag: null, pinned: true, feedFilter: 'semua' },
	{ id: 'feed-ikutan', label: 'Diikuti', iconName: 'bookmark', tag: null, pinned: true, feedFilter: 'ikutan' },
	{ id: 'feed-terlibat', label: 'Terlibat', iconName: 'bell', tag: null, pinned: true, feedFilter: 'terlibat' },
	{ id: 'feed-sekitar', label: 'Sekitar', iconName: 'globe', tag: null, pinned: true, feedFilter: 'sekitar' },
	{ id: 'feed-discover', label: 'Jelajahi', iconName: 'compass', tag: null, pinned: true, feedFilter: 'discover' }
];

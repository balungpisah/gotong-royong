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
	/** Unique stable identifier. 'pulse' is reserved for the home feed. */
	id: string;
	/** Display label shown in the tab bar. */
	label: string;
	/** Lucide icon name string (resolved at render time via tab-icons utility). */
	iconName: string;
	/** For tag-filtered tabs, the tag string. null for Pulse (home). */
	tag: NavigationTag | null;
	/** Whether this tab can be removed by the user. Pulse is always pinned. */
	pinned: boolean;
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

/** The default tab configuration for a new user — just Pulse. */
export const DEFAULT_TABS: TabConfig[] = [
	{ id: 'pulse', label: 'Pulse', iconName: 'activity', tag: null, pinned: true }
];

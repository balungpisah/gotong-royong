import type { Component } from 'svelte';
import Activity from '@lucide/svelte/icons/activity';
import Flame from '@lucide/svelte/icons/flame';
import Lightbulb from '@lucide/svelte/icons/lightbulb';
import Search from '@lucide/svelte/icons/search';
import PartyPopper from '@lucide/svelte/icons/party-popper';
import Users from '@lucide/svelte/icons/users';
import Plus from '@lucide/svelte/icons/plus';
import Tag from '@lucide/svelte/icons/tag';
import type { WellKnownTag } from '$lib/types';

/**
 * Registry mapping icon name strings to Lucide Svelte components.
 * Extend this map when adding new icon options.
 */
const ICON_REGISTRY: Record<string, Component<{ class?: string }>> = {
	activity: Activity,
	flame: Flame,
	lightbulb: Lightbulb,
	search: Search,
	'party-popper': PartyPopper,
	users: Users,
	plus: Plus,
	tag: Tag
};

/**
 * Mapping from well-known track tags to their default icon names.
 */
const TAG_ICON_MAP: Record<WellKnownTag, string> = {
	tuntaskan: 'flame',
	wujudkan: 'lightbulb',
	telusuri: 'search',
	rayakan: 'party-popper',
	musyawarah: 'users'
};

/**
 * Resolve an icon name string to a Lucide Svelte component.
 * Falls back to the Tag icon if the name is not found.
 */
export function resolveTabIcon(iconName: string): Component<{ class?: string }> {
	return ICON_REGISTRY[iconName] ?? Tag;
}

/**
 * Get the default icon name for a given tag.
 * Returns a well-known icon for track tags, 'tag' for everything else.
 */
export function iconNameForTag(tag: string): string {
	return TAG_ICON_MAP[tag as WellKnownTag] ?? 'tag';
}

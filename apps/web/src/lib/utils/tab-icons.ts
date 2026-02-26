import type { Component } from 'svelte';
import Activity from '@lucide/svelte/icons/activity';
import Flame from '@lucide/svelte/icons/flame';
import Lightbulb from '@lucide/svelte/icons/lightbulb';
import Search from '@lucide/svelte/icons/search';
import PartyPopper from '@lucide/svelte/icons/party-popper';
import Users from '@lucide/svelte/icons/users';
import Plus from '@lucide/svelte/icons/plus';
import Tag from '@lucide/svelte/icons/tag';
import Bookmark from '@lucide/svelte/icons/bookmark';
import Bell from '@lucide/svelte/icons/bell';
import Globe from '@lucide/svelte/icons/globe';
import Compass from '@lucide/svelte/icons/compass';
import Eye from '@lucide/svelte/icons/eye';
import Construction from '@lucide/svelte/icons/construction';
import Megaphone from '@lucide/svelte/icons/megaphone';
import Handshake from '@lucide/svelte/icons/handshake';
import Calendar from '@lucide/svelte/icons/calendar';
import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
import Lock from '@lucide/svelte/icons/lock';
import Heart from '@lucide/svelte/icons/heart';
import Trophy from '@lucide/svelte/icons/trophy';
import Siren from '@lucide/svelte/icons/siren';
import type { WellKnownTag } from '$lib/types';
import type { TrajectoryType } from '$lib/types';

/**
 * Registry mapping icon name strings to Lucide Svelte components.
 * Shared registry used by both tab-icons and dynamic-icon resolution.
 */
const ICON_REGISTRY: Record<string, Component<{ class?: string }>> = {
	activity: Activity,
	flame: Flame,
	lightbulb: Lightbulb,
	search: Search,
	'party-popper': PartyPopper,
	users: Users,
	plus: Plus,
	tag: Tag,
	bookmark: Bookmark,
	bell: Bell,
	globe: Globe,
	compass: Compass,
	eye: Eye,
	construction: Construction,
	megaphone: Megaphone,
	handshake: Handshake,
	calendar: Calendar,
	'bar-chart-3': BarChart3,
	lock: Lock,
	heart: Heart,
	trophy: Trophy,
	siren: Siren
};

/**
 * Mapping from well-known track tags to their default icon names.
 * @deprecated Use TRAJECTORY_ICON_MAP for new code.
 */
const TAG_ICON_MAP: Record<WellKnownTag, string> = {
	tuntaskan: 'flame',
	wujudkan: 'lightbulb',
	telusuri: 'search',
	rayakan: 'party-popper',
	musyawarah: 'users'
};

/**
 * Default icon name per trajectory type.
 * Used when no AI-selected icon is available.
 */
const TRAJECTORY_ICON_MAP: Record<TrajectoryType, string> = {
	aksi: 'construction',
	advokasi: 'megaphone',
	pantau: 'eye',
	mufakat: 'users',
	mediasi: 'handshake',
	program: 'calendar',
	data: 'bar-chart-3',
	vault: 'lock',
	bantuan: 'heart',
	pencapaian: 'trophy',
	siaga: 'siren'
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

/**
 * Get the default icon name for a trajectory type.
 * Returns the trajectory default, or 'tag' as ultimate fallback.
 */
export function iconNameForTrajectory(trajectoryType: TrajectoryType): string {
	return TRAJECTORY_ICON_MAP[trajectoryType] ?? 'tag';
}

/**
 * Resolve an icon for a trajectory type to a Lucide Svelte component.
 * Falls back to the Tag icon if the trajectory type is unknown.
 */
export function resolveTrajectoryIcon(
	trajectoryType: TrajectoryType
): Component<{ class?: string }> {
	const name = TRAJECTORY_ICON_MAP[trajectoryType];
	return name ? (ICON_REGISTRY[name] ?? Tag) : Tag;
}

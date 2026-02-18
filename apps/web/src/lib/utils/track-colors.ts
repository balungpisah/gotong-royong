/**
 * Track Color Utility — maps TrackHint to Tailwind CSS class strings.
 *
 * The 5 community tracks each have DNA tokens defined in app.css.
 * This utility provides type-safe class lookups.
 */

import type { TrackHint } from '$lib/types';

export interface TrackColorSet {
	/** Background class (e.g., 'bg-tuntaskan') */
	bg: string;
	/** Light background class (e.g., 'bg-tuntaskan/10') */
	bgLight: string;
	/** Text class (e.g., 'text-tuntaskan') */
	text: string;
	/** Border class (e.g., 'border-tuntaskan') */
	border: string;
}

const TRACK_COLORS: Record<TrackHint, TrackColorSet> = {
	tuntaskan: {
		bg: 'bg-tuntaskan',
		bgLight: 'bg-tuntaskan/10',
		text: 'text-tuntaskan',
		border: 'border-tuntaskan'
	},
	wujudkan: {
		bg: 'bg-wujudkan',
		bgLight: 'bg-wujudkan/10',
		text: 'text-wujudkan',
		border: 'border-wujudkan'
	},
	telusuri: {
		bg: 'bg-telusuri',
		bgLight: 'bg-telusuri/10',
		text: 'text-telusuri',
		border: 'border-telusuri'
	},
	rayakan: {
		bg: 'bg-rayakan',
		bgLight: 'bg-rayakan/10',
		text: 'text-rayakan',
		border: 'border-rayakan'
	},
	musyawarah: {
		bg: 'bg-musyawarah',
		bgLight: 'bg-musyawarah/10',
		text: 'text-musyawarah',
		border: 'border-musyawarah'
	}
};

/** Get the full color class set for a track. */
export function trackColors(track: TrackHint): TrackColorSet {
	return TRACK_COLORS[track];
}

/** Get just the background class for a track. */
export function trackBgClass(track: TrackHint): string {
	return TRACK_COLORS[track].bg;
}

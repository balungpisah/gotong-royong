/**
 * Trajectory Color Utility — maps TrajectoryType to Tailwind CSS class strings.
 *
 * This is the new canonical color utility for the trajectory model.
 * The old `track-colors.ts` is kept for backward compat with components
 * still using the 5-track model.
 *
 * @see docs/design/specs/ai-spec/04b-trajectory-map.md
 */

import type { TrajectoryType } from '$lib/types';

// ---------------------------------------------------------------------------
// Color Set Interface
// ---------------------------------------------------------------------------

/** Extended color key — all 11 trajectory types plus 'kelola' (group management). */
export type TrajectoryColorKey = TrajectoryType | 'kelola';

/** Tailwind class set for a trajectory — same shape as TrackColorSet for compat. */
export interface TrajectoryColorSet {
	/** Background class (e.g., 'bg-amber-600'). */
	bg: string;
	/** Light background class (e.g., 'bg-amber-600/10'). */
	bgLight: string;
	/** Text class (e.g., 'text-amber-600'). */
	text: string;
	/** Border class (e.g., 'border-amber-600'). */
	border: string;
}

// ---------------------------------------------------------------------------
// Trajectory → Color Mapping
// ---------------------------------------------------------------------------

/** Neutral fallback for unknown trajectory types. */
const FALLBACK: TrajectoryColorSet = {
	bg: 'bg-stone-500',
	bgLight: 'bg-stone-500/10',
	text: 'text-stone-500',
	border: 'border-stone-500'
};

const TRAJECTORY_COLORS: Record<TrajectoryColorKey, TrajectoryColorSet> = {
	aksi: {
		bg: 'bg-amber-600',
		bgLight: 'bg-amber-600/10',
		text: 'text-amber-600',
		border: 'border-amber-600'
	},
	advokasi: {
		bg: 'bg-rose-600',
		bgLight: 'bg-rose-600/10',
		text: 'text-rose-600',
		border: 'border-rose-600'
	},
	pantau: {
		bg: 'bg-indigo-600',
		bgLight: 'bg-indigo-600/10',
		text: 'text-indigo-600',
		border: 'border-indigo-600'
	},
	mufakat: {
		bg: 'bg-teal-600',
		bgLight: 'bg-teal-600/10',
		text: 'text-teal-600',
		border: 'border-teal-600'
	},
	mediasi: {
		bg: 'bg-violet-600',
		bgLight: 'bg-violet-600/10',
		text: 'text-violet-600',
		border: 'border-violet-600'
	},
	program: {
		bg: 'bg-emerald-600',
		bgLight: 'bg-emerald-600/10',
		text: 'text-emerald-600',
		border: 'border-emerald-600'
	},
	data: {
		bg: 'bg-sky-600',
		bgLight: 'bg-sky-600/10',
		text: 'text-sky-600',
		border: 'border-sky-600'
	},
	vault: {
		bg: 'bg-slate-600',
		bgLight: 'bg-slate-600/10',
		text: 'text-slate-600',
		border: 'border-slate-600'
	},
	bantuan: {
		bg: 'bg-amber-400',
		bgLight: 'bg-amber-400/10',
		text: 'text-amber-400',
		border: 'border-amber-400'
	},
	pencapaian: {
		bg: 'bg-yellow-500',
		bgLight: 'bg-yellow-500/10',
		text: 'text-yellow-500',
		border: 'border-yellow-500'
	},
	siaga: {
		bg: 'bg-red-600',
		bgLight: 'bg-red-600/10',
		text: 'text-red-600',
		border: 'border-red-600'
	},
	kelola: {
		bg: 'bg-blue-600',
		bgLight: 'bg-blue-600/10',
		text: 'text-blue-600',
		border: 'border-blue-600'
	}
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Get the full color class set for a trajectory type (or 'kelola'). */
export function trajectoryColors(t: TrajectoryColorKey): TrajectoryColorSet {
	return TRAJECTORY_COLORS[t] ?? FALLBACK;
}

/** Get just the background class for a trajectory type (or 'kelola'). */
export function trajectoryBgClass(t: TrajectoryColorKey): string {
	return (TRAJECTORY_COLORS[t] ?? FALLBACK).bg;
}

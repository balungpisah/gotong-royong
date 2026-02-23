/**
 * Mood Color Utility — maps sentiment / trajectory / track-hint to CSS custom property values.
 *
 * Used for mood-colored glow shadows and borders on feed cards and the
 * detail panel.  Returns a `var(--token)` string suitable for inline
 * styles and `color-mix()` expressions.
 *
 * Priority chain: sentiment → trajectory_type → track_hint → neutral
 */

// ── Sentiment → color (design-system tokens) ───────────────────────
const sentimentColorMap: Record<string, string> = {
	angry:       'var(--c-bahaya)',      // danger red
	hopeful:     'var(--c-berhasil)',     // success green
	urgent:      'var(--c-peringatan)',   // warning orange
	celebratory: 'var(--t-rayakan)',      // celebration gold
	sad:         'var(--v-mid)',          // muted slate
	curious:     'var(--t-telusuri)',     // explore purple
	fun:         'var(--c-api-terang)'    // warm amber
};

// ── Trajectory type → color (standard Tailwind oklch values) ───────
const trajectoryColorMap: Record<string, string> = {
	aksi:        'oklch(0.646 0.162 55.09)',     // amber-600
	advokasi:    'oklch(0.586 0.21 16.98)',      // rose-600
	pantau:      'oklch(0.457 0.194 264.05)',    // indigo-600
	mufakat:     'oklch(0.523 0.126 172.09)',    // teal-600
	mediasi:     'oklch(0.541 0.195 285.75)',    // violet-600
	program:     'oklch(0.596 0.145 163.23)',    // emerald-600
	data:        'oklch(0.588 0.158 231.7)',     // sky-600
	vault:       'oklch(0.446 0.043 257.28)',    // slate-600
	bantuan:     'oklch(0.828 0.153 84.57)',     // amber-400
	pencapaian:  'oklch(0.795 0.184 86.05)',     // yellow-500
	siaga:       'oklch(0.577 0.245 27.32)'      // red-600
};

// ── Track-hint fallback (legacy, for cards without sentiment/trajectory) ──
const trackColorMap: Record<string, string> = {
	tuntaskan:  'var(--t-tuntaskan)',
	wujudkan:   'var(--t-wujudkan)',
	telusuri:   'var(--t-telusuri)',
	rayakan:    'var(--t-rayakan)',
	musyawarah: 'var(--t-musyawarah)'
};

const NEUTRAL = 'var(--c-batu)';

/**
 * Resolve a mood color from sentiment, trajectory type, and/or track hint.
 * Priority: sentiment → trajectory_type → track_hint → neutral fallback.
 */
export function getMoodColor(sentiment?: string, trackHint?: string, trajectoryType?: string): string {
	if (sentiment) return sentimentColorMap[sentiment] ?? NEUTRAL;
	if (trajectoryType) return trajectoryColorMap[trajectoryType] ?? NEUTRAL;
	if (trackHint) return trackColorMap[trackHint] ?? NEUTRAL;
	return NEUTRAL;
}

/**
 * Build the "selected / raised" box-shadow string for a given mood color.
 * Shared between the selected feed card and the detail panel so they
 * glow with the same intensity.
 */
export function moodShadow(color: string): string {
	return (
		`0 2px 8px 0 color-mix(in srgb, ${color} 25%, transparent), ` +
		`0 8px 24px -4px color-mix(in srgb, ${color} 35%, transparent)`
	);
}

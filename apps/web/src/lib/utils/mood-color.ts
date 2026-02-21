/**
 * Mood Color Utility — maps sentiment / track-hint to CSS custom property values.
 *
 * Used for mood-colored glow shadows and borders on feed cards and the
 * detail panel.  Returns a `var(--token)` string suitable for inline
 * styles and `color-mix()` expressions.
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

// ── Track-hint fallback (legacy, for cards without sentiment) ──────
const trackColorMap: Record<string, string> = {
	tuntaskan:  'var(--t-tuntaskan)',
	wujudkan:   'var(--t-wujudkan)',
	telusuri:   'var(--t-telusuri)',
	rayakan:    'var(--t-rayakan)',
	musyawarah: 'var(--t-musyawarah)'
};

const NEUTRAL = 'var(--c-batu)';

/**
 * Resolve a mood color from sentiment and/or track hint.
 * Priority: sentiment → track hint → neutral fallback.
 */
export function getMoodColor(sentiment?: string, trackHint?: string): string {
	if (sentiment) return sentimentColorMap[sentiment] ?? NEUTRAL;
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

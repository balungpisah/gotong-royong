import { slide, type SlideParams } from 'svelte/transition';
import type { TransitionConfig } from 'svelte/transition';

/**
 * Wrapper around Svelte's `slide` transition that guards against NaN values
 * in keyframe properties (height, padding, margin, etc.).
 *
 * This happens when `getComputedStyle` returns values that `parseFloat` can't
 * parse before the element has a measured layout.
 */
export function safeSlide(node: Element, params?: SlideParams): TransitionConfig {
	const result = slide(node, params);
	const originalCss = result.css;

	if (originalCss) {
		result.css = (t: number, u: number) => {
			const css = originalCss(t, u);
			// Replace any NaN values (e.g. "height: NaNpx") with 0
			return css.replace(/NaN/g, '0');
		};
	}

	return result;
}

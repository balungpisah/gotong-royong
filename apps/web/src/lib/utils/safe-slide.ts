import {
	slide,
	fly,
	fade,
	scale,
	type SlideParams,
	type FlyParams,
	type FadeParams,
	type ScaleParams,
	type TransitionConfig
} from 'svelte/transition';

/**
 * Wraps a Svelte transition function to sanitize NaN values in its CSS keyframes.
 *
 * Svelte 5 measures element dimensions via getComputedStyle and passes them to
 * the Web Animations API. When an element has no measurable layout yet (e.g.
 * entering the DOM inside a conditional block), parseFloat returns NaN, producing
 * console errors like "Invalid keyframe value for property height: NaNpx".
 *
 * This wrapper intercepts the css() output and replaces NaN with 0.
 */
function wrapTransition<P>(
	fn: (node: Element, params?: P) => TransitionConfig
): (node: Element, params?: P) => TransitionConfig {
	return (node: Element, params?: P): TransitionConfig => {
		const result = fn(node, params);
		const originalCss = result.css;

		if (originalCss) {
			result.css = (t: number, u: number) => {
				const css = originalCss(t, u);
				return css.replace(/NaN/g, '0');
			};
		}

		return result;
	};
}

export const safeSlide = wrapTransition<SlideParams>(slide);
export const safeFly = wrapTransition<FlyParams>(fly);
export const safeFade = wrapTransition<FadeParams>(fade);
export const safeScale = wrapTransition<ScaleParams>(scale);

/**
 * Svelte Transition Helpers — wrappers around svelte/transition with
 * project-consistent easing and durations.
 *
 * Usage in .svelte files:
 *   import { fadeUp, scaleIn, slideDown } from '$lib/utils/transitions';
 *   <div transition:fadeUp>...</div>
 *   <div in:scaleIn={{ delay: 100 }} out:fadeOut>...</div>
 */

import {
	fly,
	fade,
	scale,
	slide,
	type FlyParams,
	type FadeParams,
	type ScaleParams,
	type SlideParams,
	type TransitionConfig
} from 'svelte/transition';
import { cubicOut, quintOut, backOut } from 'svelte/easing';

// ─── Duration presets (match CSS custom properties) ──────────────────────────

export const DURATION = {
	instant: 100,
	fast: 150,
	normal: 250,
	slow: 400,
	slower: 600
} as const;

// ─── Easing presets ──────────────────────────────────────────────────────────

export const EASING = {
	spring: quintOut, // closest to CSS --ease-spring
	smooth: cubicOut, // closest to CSS --ease-smooth
	bounce: backOut // closest to CSS --ease-bounce
} as const;

// ─── Stagger helper ─────────────────────────────────────────────────────────

/** Returns a delay in ms for staggered list items */
export function staggerDelay(index: number, gap = 50): number {
	return index * gap;
}

// ─── Composite transitions ──────────────────────────────────────────────────

type TransitionFn<T> = (
	node: Element,
	params?: T
) => TransitionConfig;

/** Fade in while sliding up 12px — great for cards & list items */
export const fadeUp: TransitionFn<Partial<FlyParams>> = (node, params = {}) =>
	fly(node, {
		y: params.y ?? 12,
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Fade in while sliding down 12px — for dropdowns, popovers */
export const fadeDown: TransitionFn<Partial<FlyParams>> = (node, params = {}) =>
	fly(node, {
		y: params.y ?? -12,
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Fade in while sliding from right 12px */
export const fadeLeft: TransitionFn<Partial<FlyParams>> = (node, params = {}) =>
	fly(node, {
		x: params.x ?? 12,
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Fade in while sliding from left 12px */
export const fadeRight: TransitionFn<Partial<FlyParams>> = (node, params = {}) =>
	fly(node, {
		x: params.x ?? -12,
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Simple fade — for overlays, backdrops */
export const fadeIn: TransitionFn<Partial<FadeParams>> = (node, params = {}) =>
	fade(node, {
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.smooth,
		delay: params.delay ?? 0
	});

/** Fade out quickly */
export const fadeOut: TransitionFn<Partial<FadeParams>> = (node, params = {}) =>
	fade(node, {
		duration: params.duration ?? DURATION.fast,
		easing: params.easing ?? EASING.smooth,
		delay: params.delay ?? 0
	});

/** Scale up from 92% with fade — for modals, toasts */
export const scaleIn: TransitionFn<Partial<ScaleParams>> = (node, params = {}) =>
	scale(node, {
		start: params.start ?? 0.92,
		duration: params.duration ?? DURATION.normal,
		easing: params.easing ?? EASING.bounce,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Scale down to 92% with fade */
export const scaleOut: TransitionFn<Partial<ScaleParams>> = (node, params = {}) =>
	scale(node, {
		start: params.start ?? 0.92,
		duration: params.duration ?? DURATION.fast,
		easing: params.easing ?? EASING.smooth,
		delay: params.delay ?? 0,
		opacity: params.opacity ?? 0
	});

/** Vertical slide — for accordions, expandable panels */
export const slideVertical: TransitionFn<Partial<SlideParams>> = (node, params = {}) =>
	slide(node, {
		duration: params.duration ?? DURATION.slow,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		axis: 'y'
	});

/** Horizontal slide — for sidebars, drawers */
export const slideHorizontal: TransitionFn<Partial<SlideParams>> = (node, params = {}) =>
	slide(node, {
		duration: params.duration ?? DURATION.slow,
		easing: params.easing ?? EASING.spring,
		delay: params.delay ?? 0,
		axis: 'x'
	});

// ─── Svelte action for IntersectionObserver reveal ──────────────────────────

interface RevealOptions {
	/** CSS class to add when element enters viewport */
	class?: string;
	/** IntersectionObserver threshold (0–1) */
	threshold?: number;
	/** Only trigger once */
	once?: boolean;
}

/**
 * Svelte action: reveals an element when it scrolls into view.
 * Adds a CSS class (default: 'animate-fade-in') on intersection.
 *
 * Usage: <div use:reveal>...</div>
 *        <div use:reveal={{ class: 'animate-slide-up', threshold: 0.2 }}>...</div>
 */
export function reveal(node: HTMLElement, options: RevealOptions = {}) {
	const {
		class: revealClass = 'animate-fade-in',
		threshold = 0.1,
		once = true
	} = options;

	// Start invisible
	node.style.opacity = '0';

	const observer = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (entry.isIntersecting) {
					node.style.opacity = '';
					node.classList.add(...revealClass.split(' '));
					if (once) observer.unobserve(node);
				} else if (!once) {
					node.classList.remove(...revealClass.split(' '));
					node.style.opacity = '0';
				}
			}
		},
		{ threshold }
	);

	observer.observe(node);

	return {
		destroy() {
			observer.disconnect();
		}
	};
}

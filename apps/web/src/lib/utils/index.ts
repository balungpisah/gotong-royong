import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]): string {
	return twMerge(clsx(inputs));
}

export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

export { renderMarkdown } from './markdown';
export { resolveTabIcon, iconNameForTag } from './tab-icons';
export {
	// Transitions
	fadeUp,
	fadeDown,
	fadeLeft,
	fadeRight,
	fadeIn,
	fadeOut,
	scaleIn,
	scaleOut,
	slideVertical,
	slideHorizontal,
	// Actions
	reveal,
	// Helpers
	staggerDelay,
	DURATION,
	EASING
} from './transitions';

/**
 * Card Variants — reusable tv() presets for card surface styling.
 *
 * Provides multiple visual styles for cards so we can experiment quickly.
 * Both PulseActivityCard, FeedEventCard, and any future cards can
 * consume these variants via a single `style` prop.
 *
 * Usage:
 *   import { cardVariants, type CardStyle } from '$lib/utils/card-variants';
 *   const classes = cardVariants({ style: 'default', state: selected ? 'selected' : 'idle' });
 */

import { tv, type VariantProps } from 'tailwind-variants';

export const cardVariants = tv({
	base: 'group transition',
	variants: {
		style: {
			/** Design DNA default — rounded-xl, subtle border, bg-card. Matches PulseActivityCard. */
			default:
				'rounded-xl border border-border/60 bg-card p-4',

			/** Elevated — no visible border, lifted shadow. */
			elevated:
				'rounded-xl border-0 bg-card p-4 shadow-md',

			/** Outlined — thicker border, no shadow. */
			outlined:
				'rounded-xl border-2 border-border bg-card p-4',

			/** Flat — minimal, no border, no shadow, muted background. */
			flat:
				'rounded-lg bg-muted/30 p-4',

			/** Glass — frosted glass effect. Best on dark mode. */
			glass:
				'rounded-2xl border border-border/30 bg-card/80 p-4 backdrop-blur supports-[backdrop-filter]:bg-card/60'
		},
		state: {
			idle: '',
			selected: 'border-primary/40 bg-primary/5 shadow-sm',
			hover: ''
		},
		interactive: {
			true: 'cursor-pointer',
			false: ''
		}
	},
	compoundVariants: [
		// Hover effects for interactive idle cards
		{
			state: 'idle',
			interactive: true,
			style: 'default',
			class: 'hover:border-border hover:shadow-sm'
		},
		{
			state: 'idle',
			interactive: true,
			style: 'elevated',
			class: 'hover:shadow-lg'
		},
		{
			state: 'idle',
			interactive: true,
			style: 'outlined',
			class: 'hover:border-primary/30'
		},
		{
			state: 'idle',
			interactive: true,
			style: 'flat',
			class: 'hover:bg-muted/50'
		},
		{
			state: 'idle',
			interactive: true,
			style: 'glass',
			class: 'hover:border-border/50 hover:shadow-sm'
		}
	],
	defaultVariants: {
		style: 'default',
		state: 'idle',
		interactive: false
	}
});

export type CardVariants = VariantProps<typeof cardVariants>;
export type CardStyle = NonNullable<CardVariants['style']>;

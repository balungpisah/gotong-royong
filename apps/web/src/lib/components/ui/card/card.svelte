<script lang="ts" module>
	import { cn, type WithElementRef } from '$lib/utils';
	import type { HTMLAttributes } from 'svelte/elements';
	import { type VariantProps, tv } from 'tailwind-variants';

	export const cardVariants = tv({
		base: 'bg-card text-card-foreground flex flex-col rounded-xl transition',
		variants: {
			variant: {
				default: 'border border-border/60 shadow-sm',
				elevated: 'border-0 shadow-md',
				outlined: 'border-2 border-border',
				flat: 'rounded-lg border-0 bg-muted/30 shadow-none',
				glass:
					'border border-border/30 bg-card/80 rounded-2xl backdrop-blur supports-[backdrop-filter]:bg-card/60'
			},
			padding: {
				default: 'gap-6 py-6',
				compact: 'gap-4 p-4',
				none: 'gap-0 p-0'
			},
			interactive: {
				true: 'cursor-pointer',
				false: ''
			},
			state: {
				idle: '',
				selected: 'border-primary/40 bg-primary/5 shadow-sm'
			}
		},
		compoundVariants: [
			{
				state: 'idle',
				interactive: true,
				variant: 'default',
				class: 'hover:border-border hover:shadow-sm'
			},
			{
				state: 'idle',
				interactive: true,
				variant: 'elevated',
				class: 'hover:shadow-lg'
			},
			{
				state: 'idle',
				interactive: true,
				variant: 'outlined',
				class: 'hover:border-primary/30'
			},
			{
				state: 'idle',
				interactive: true,
				variant: 'flat',
				class: 'hover:bg-muted/50'
			},
			{
				state: 'idle',
				interactive: true,
				variant: 'glass',
				class: 'hover:border-border/50 hover:shadow-sm'
			}
		],
		defaultVariants: {
			variant: 'default',
			padding: 'default',
			interactive: false,
			state: 'idle'
		}
	});

	export type CardVariant = VariantProps<typeof cardVariants>['variant'];
	export type CardPadding = VariantProps<typeof cardVariants>['padding'];
	export type CardState = VariantProps<typeof cardVariants>['state'];
</script>

<script lang="ts">
	type Props = WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		variant?: CardVariant;
		padding?: CardPadding;
		interactive?: boolean;
		state?: CardState;
	};

	let {
		ref = $bindable(null),
		class: className,
		variant = 'default',
		padding = 'default',
		interactive = false,
		state = 'idle',
		children,
		...restProps
	}: Props = $props();
</script>

<div
	bind:this={ref}
	data-slot="card"
	class={cn(cardVariants({ variant, padding, interactive, state }), className)}
	{...restProps}
>
	{@render children?.()}
</div>

<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const statusIndicatorVariants = tv({
		slots: {
			wrapper: 'inline-flex items-center gap-1.5',
			dot: 'size-2 shrink-0 rounded-full',
			label: 'text-[11px] font-bold'
		},
		variants: {
			status: {
				active: {
					dot: 'bg-berhasil',
					label: 'text-berhasil'
				},
				stalled: {
					dot: 'bg-peringatan',
					label: 'text-peringatan'
				},
				done: {
					dot: 'bg-batu',
					label: 'text-batu'
				},
				review: {
					dot: 'bg-api animate-blink',
					label: 'text-api'
				},
				moderation: {
					dot: 'bg-bahaya animate-blink',
					label: 'text-bahaya'
				},
				sealed: {
					dot: 'bg-vault-deep',
					label: 'text-vault-deep'
				}
			}
		},
		defaultVariants: {
			status: 'active'
		}
	});

	export type StatusIndicatorStatus = VariantProps<typeof statusIndicatorVariants>['status'];
</script>

<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		status = 'active',
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> & {
		status?: StatusIndicatorStatus;
	} = $props();

	const variants = $derived(statusIndicatorVariants({ status }));
</script>

<span
	bind:this={ref}
	data-slot="status-indicator"
	class={cn(variants.wrapper(), className)}
	{...restProps}
>
	<span class={variants.dot()} aria-hidden="true"></span>
	{#if children}
		<span class={variants.label()}>{@render children()}</span>
	{/if}
</span>

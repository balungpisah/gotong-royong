<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const pillVariants = tv({
		base: 'inline-flex w-fit shrink-0 items-center gap-1 rounded-full px-3 py-1 text-[11px] font-bold whitespace-nowrap transition-colors [&>svg]:pointer-events-none [&>svg]:size-3',
		variants: {
			variant: {
				default: 'bg-kapas text-tanah',
				active: 'bg-bara text-api',
				success: 'bg-berhasil-lembut text-berhasil',
				removable: 'bg-kapas text-tanah',
				'track-tuntaskan': 'bg-tuntaskan text-white',
				'track-wujudkan': 'bg-wujudkan text-white',
				'track-telusuri': 'bg-telusuri text-white',
				'track-rayakan': 'bg-rayakan text-white',
				'track-musyawarah': 'bg-musyawarah text-white'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type PillVariant = VariantProps<typeof pillVariants>['variant'];
</script>

<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		variant = 'default',
		removable = false,
		onremove,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> & {
		variant?: PillVariant;
		removable?: boolean;
		onremove?: () => void;
	} = $props();
</script>

<span
	bind:this={ref}
	data-slot="pill"
	class={cn(pillVariants({ variant: removable ? 'removable' : variant }), className)}
	{...restProps}
>
	{@render children?.()}
	{#if removable}
		<button
			type="button"
			class="ml-0.5 inline-flex size-4 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-tanah/10"
			onclick={onremove}
			aria-label="Remove"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				width="10"
				height="10"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M18 6 6 18" />
				<path d="m6 6 12 12" />
			</svg>
		</button>
	{/if}
</span>

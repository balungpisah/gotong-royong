<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const badgeVariants = tv({
		base: 'focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:ring-[3px] [&>svg]:pointer-events-none [&>svg]:size-3',
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground [a&]:hover:bg-primary/90 border-transparent',
				secondary:
					'bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90 border-transparent',
				destructive:
					'bg-destructive [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/70 border-transparent text-white',
				outline: 'text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground',
				'track-tuntaskan': 'bg-tuntaskan border-transparent text-white',
				'track-wujudkan': 'bg-wujudkan border-transparent text-white',
				'track-telusuri': 'bg-telusuri border-transparent text-white',
				'track-rayakan': 'bg-rayakan border-transparent text-white',
				'track-musyawarah': 'bg-musyawarah border-transparent text-white',
				success: 'bg-berhasil-lembut border-transparent text-berhasil',
				warning: 'bg-peringatan-lembut border-transparent text-peringatan',
				danger: 'bg-bahaya-lembut border-transparent text-bahaya',
				info: 'bg-keterangan-lembut border-transparent text-keterangan',
				step: 'bg-api border-transparent text-white',
				'step-done': 'bg-berhasil-lembut border-transparent text-berhasil',
				'step-future': 'bg-kapas border-transparent text-kayu',
				rahasia: 'bg-tanah-gelap border-transparent text-white',
				vault: 'bg-vault-deep border-transparent text-white',
				siaga: 'bg-siaga-pulse border-transparent text-white animate-pulse',
				confidence: 'bg-kapas border-transparent text-kayu'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>['variant'];
</script>

<script lang="ts">
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';

	let {
		ref = $bindable(null),
		href,
		class: className,
		variant = 'default',
		children,
		...restProps
	}: WithElementRef<HTMLAnchorAttributes> & {
		variant?: BadgeVariant;
	} = $props();
</script>

<svelte:element
	this={href ? 'a' : 'span'}
	bind:this={ref}
	data-slot="badge"
	{href}
	class={cn(badgeVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</svelte:element>

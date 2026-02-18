<script lang="ts">
	import { tv } from 'tailwind-variants';
	import { resolveTabIcon } from '$lib/utils';
	import X from '@lucide/svelte/icons/x';
	import type { Component } from 'svelte';

	interface Props {
		href: string;
		label: string;
		iconName: string;
		active?: boolean;
		removable?: boolean;
		onremove?: () => void;
	}

	let { href, label, iconName, active = false, removable = false, onremove }: Props = $props();

	const Icon: Component<{ class?: string }> = $derived(resolveTabIcon(iconName));

	const tabVariants = tv({
		base: 'relative inline-flex items-center justify-center font-medium transition',
		variants: {
			layout: {
				mobile:
					'flex-col gap-1 rounded-lg px-2 py-2 text-[11px]',
				desktop:
					'gap-2 rounded-full px-3 py-2 text-sm'
			},
			state: {
				active: '',
				inactive: 'text-muted-foreground hover:bg-muted hover:text-foreground'
			}
		},
		compoundVariants: [
			{
				layout: 'mobile',
				state: 'active',
				class: 'bg-primary/12 text-primary'
			},
			{
				layout: 'desktop',
				state: 'active',
				class: 'bg-primary text-primary-foreground'
			}
		],
		defaultVariants: {
			layout: 'mobile',
			state: 'inactive'
		}
	});
</script>

<!-- Mobile layout -->
<a
	{href}
	class={tabVariants({ layout: 'mobile', state: active ? 'active' : 'inactive' }) +
		' md:hidden group'}
	aria-current={active ? 'page' : undefined}
>
	<Icon class="size-4" />
	<span>{label}</span>
</a>

<!-- Desktop layout -->
<a
	{href}
	class={tabVariants({ layout: 'desktop', state: active ? 'active' : 'inactive' }) +
		' hidden md:inline-flex group'}
	aria-current={active ? 'page' : undefined}
>
	<Icon class="size-4" />
	<span>{label}</span>
	{#if removable && !active}
		<button
			type="button"
			class="ml-1 hidden rounded-full p-0.5 opacity-0 transition hover:bg-foreground/10 group-hover:opacity-100 md:inline-flex"
			onclick={(e) => {
				e.preventDefault();
				e.stopPropagation();
				onremove?.();
			}}
			aria-label="Remove tab"
		>
			<X class="size-3" />
		</button>
	{/if}
</a>

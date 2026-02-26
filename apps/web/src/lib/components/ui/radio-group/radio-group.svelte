<script lang="ts">
	import { RadioGroup as RadioGroupPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';
	import { cn, type WithoutChildrenOrChild } from '$lib/utils';

	let {
		ref = $bindable(null),
		class: className,
		value = $bindable(''),
		children,
		orientation,
		...restProps
	}: WithoutChildrenOrChild<RadioGroupPrimitive.RootProps> & {
		children?: Snippet;
	} = $props();
</script>

<RadioGroupPrimitive.Root
	bind:ref
	bind:value
	{orientation}
	data-slot="radio-group"
	class={cn(
		orientation === 'horizontal' ? 'flex flex-row gap-4' : 'flex flex-col gap-3',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</RadioGroupPrimitive.Root>

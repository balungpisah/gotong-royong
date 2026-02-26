<script lang="ts">
	import { RadioGroup as RadioGroupPrimitive } from "bits-ui";
	import type { Snippet } from "svelte";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithoutChildrenOrChild<RadioGroupPrimitive.ItemProps> & {
		children?: Snippet;
	} = $props();
</script>

<label class={cn("flex items-center gap-2", className)} data-slot="radio-group-item">
	<RadioGroupPrimitive.Item
		bind:ref
		data-slot="radio-group-item-indicator"
		class={cn(
			"border-input text-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 aria-invalid:border-destructive aspect-square size-4 shrink-0 rounded-full border shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-primary"
		)}
		{...restProps}
	>
		{#snippet children({ checked })}
			{#if checked}
				<span class="flex items-center justify-center">
					<span class="bg-primary absolute size-2 rounded-full"></span>
				</span>
			{/if}
		{/snippet}
	</RadioGroupPrimitive.Item>
	{#if children}
		<span class="text-body text-foreground">{@render children()}</span>
	{/if}
</label>

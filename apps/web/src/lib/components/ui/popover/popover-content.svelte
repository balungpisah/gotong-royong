<script lang="ts">
	import { Popover as PopoverPrimitive } from "bits-ui";
	import { cn } from "$lib/utils";
	import PopoverPortal from "./popover-portal.svelte";
	import type { ComponentProps } from "svelte";
	import type { WithoutChildrenOrChild } from "$lib/utils";

	let {
		ref = $bindable(null),
		class: className,
		sideOffset = 8,
		side = "top",
		children,
		portalProps,
		...restProps
	}: PopoverPrimitive.ContentProps & {
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof PopoverPortal>>;
	} = $props();
</script>

<PopoverPortal {...portalProps}>
	<PopoverPrimitive.Content
		bind:ref
		data-slot="popover-content"
		{sideOffset}
		{side}
		class={cn(
			"bg-card border border-border/30 animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-end-2 data-[side=right]:slide-in-from-start-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-fit origin-(--bits-popover-content-transform-origin) rounded-xl shadow-lg",
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</PopoverPrimitive.Content>
</PopoverPortal>

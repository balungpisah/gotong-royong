<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils';
	import InputLabel from '../input/input-label.svelte';
	import InputHint from '../input/input-hint.svelte';
	import InputError from '../input/input-error.svelte';

	type Props = WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		label: string;
		hint?: string;
		error?: string;
		required?: boolean;
		id?: string;
		children: Snippet;
	};

	let {
		ref = $bindable(null),
		class: className,
		label,
		hint,
		error,
		required = false,
		id = `field-${Math.random().toString(36).slice(2, 9)}`,
		children,
		...restProps
	}: Props = $props();

	const hasError = $derived(!!error);
</script>

<div
	bind:this={ref}
	data-slot="form-field"
	class={cn('flex flex-col gap-1.5', className)}
	{...restProps}
>
	<InputLabel for={id}>
		{label}
		{#if required}<span class="text-bahaya">*</span>{/if}
	</InputLabel>

	{@render children()}

	{#if hasError}
		<InputError>{error}</InputError>
	{:else if hint}
		<InputHint>{hint}</InputHint>
	{/if}
</div>

<script lang="ts">
	import type { Block } from '$lib/types';
	import ListBlockRenderer from './list-block-renderer.svelte';
	import DocumentBlockRenderer from './document-block-renderer.svelte';
	import FormBlockRenderer from './form-block-renderer.svelte';
	import ComputedBlockRenderer from './computed-block-renderer.svelte';
	import DisplayBlockRenderer from './display-block-renderer.svelte';
	import VoteBlockRenderer from './vote-block-renderer.svelte';
	import ReferenceBlockRenderer from './reference-block-renderer.svelte';

	let { block }: { block: Block } = $props();
</script>

<div data-slot="block-renderer" data-block-type={block.type}>
	{#if block.type === 'list'}
		<ListBlockRenderer {block} />
	{:else if block.type === 'document'}
		<DocumentBlockRenderer {block} />
	{:else if block.type === 'form'}
		<FormBlockRenderer {block} />
	{:else if block.type === 'computed'}
		<ComputedBlockRenderer {block} />
	{:else if block.type === 'display'}
		<DisplayBlockRenderer {block} />
	{:else if block.type === 'vote'}
		<VoteBlockRenderer {block} />
	{:else if block.type === 'reference'}
		<ReferenceBlockRenderer {block} />
	{/if}
</div>

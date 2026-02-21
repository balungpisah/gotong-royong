<script lang="ts">
	import type { DiffCardMessage } from '$lib/types';
	import DiffCardRenderer from '$lib/components/blocks/diff-card-renderer.svelte';

	let { message }: { message: DiffCardMessage } = $props();
</script>

<div data-slot="diff-card-wrapper">
	<DiffCardRenderer diff={message.diff} bare />
</div>

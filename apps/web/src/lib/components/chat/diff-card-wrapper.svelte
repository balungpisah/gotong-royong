<script lang="ts">
	import type { DiffCardMessage } from '$lib/types';
	import DiffCardRenderer from '$lib/components/blocks/diff-card-renderer.svelte';

	let { message }: { message: DiffCardMessage } = $props();
</script>

<div class="py-1" data-slot="diff-card-wrapper">
	<DiffCardRenderer diff={message.diff} />
</div>

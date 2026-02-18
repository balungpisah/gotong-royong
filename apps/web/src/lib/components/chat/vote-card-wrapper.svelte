<script lang="ts">
	import type { VoteCardMessage } from '$lib/types';
	import VoteBlockRenderer from '$lib/components/blocks/vote-block-renderer.svelte';

	let { message }: { message: VoteCardMessage } = $props();
</script>

<div class="py-1" data-slot="vote-card-wrapper">
	<VoteBlockRenderer block={message.block} />
</div>

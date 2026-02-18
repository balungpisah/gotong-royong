<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import ChatBubble from './chat-bubble.svelte';
	import SystemMessageRenderer from './system-message.svelte';
	import CardEnvelope from './card-envelope.svelte';

	let { messages }: { messages: ChatMessage[] } = $props();

	// Find the index of the last card message (non-user, non-system) to expand it by default
	const lastCardIndex = $derived.by(() => {
		for (let i = messages.length - 1; i >= 0; i--) {
			const t = messages[i].type;
			if (t !== 'user' && t !== 'system') return i;
		}
		return -1;
	});
</script>

<div class="flex flex-col gap-3" data-slot="chat-thread">
	{#each messages as message, i (message.message_id)}
		{#if message.type === 'user'}
			<ChatBubble {message} />
		{:else if message.type === 'system'}
			<SystemMessageRenderer {message} />
		{:else}
			<CardEnvelope {message} defaultExpanded={i === lastCardIndex} />
		{/if}
	{/each}
</div>

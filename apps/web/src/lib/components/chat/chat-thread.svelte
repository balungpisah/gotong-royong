<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import ChatBubble from './chat-bubble.svelte';
	import AiInlineCard from './ai-inline-card.svelte';
	import SystemMessageRenderer from './system-message.svelte';
	import EvidenceCard from './evidence-card.svelte';
	import GalangMessageRenderer from './galang-message.svelte';
	import VoteCardWrapper from './vote-card-wrapper.svelte';
	import DiffCardWrapper from './diff-card-wrapper.svelte';

	let { messages }: { messages: ChatMessage[] } = $props();
</script>

<div class="flex flex-col gap-3" data-slot="chat-thread">
	{#each messages as message (message.message_id)}
		{#if message.type === 'user'}
			<ChatBubble {message} />
		{:else if message.type === 'ai_card'}
			<AiInlineCard {message} />
		{:else if message.type === 'system'}
			<SystemMessageRenderer {message} />
		{:else if message.type === 'evidence'}
			<EvidenceCard {message} />
		{:else if message.type === 'galang'}
			<GalangMessageRenderer {message} />
		{:else if message.type === 'vote_card'}
			<VoteCardWrapper {message} />
		{:else if message.type === 'diff_card'}
			<DiffCardWrapper {message} />
		{/if}
	{/each}
</div>

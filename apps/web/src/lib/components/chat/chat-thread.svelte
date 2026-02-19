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

	/** Is this a card/block type (not user bubble, not system pill)? */
	function isBlock(message: ChatMessage): boolean {
		return message.type !== 'user' && message.type !== 'system';
	}

	/** Phase blocks = major phase actions (vote, diff). Minor = everything else. */
	function getDotVariant(message: ChatMessage): 'phase' | 'minor' {
		switch (message.type) {
			case 'vote_card':
			case 'diff_card':
				return 'phase';
			default:
				return 'minor';
		}
	}
</script>

<!--
  The vertical line runs through the dot center (4px from left).
  Dots live inside CardEnvelope as the first flex child.
  The line is absolutely positioned so it passes behind dots seamlessly.
-->
<div class="relative" data-slot="chat-thread">
	<!-- Vertical timeline line — runs full height, positioned at dot center (3.5px ≈ center of 8px dot) -->
	<div class="absolute top-0 bottom-0 left-[3.5px] w-px bg-border/50"></div>

	<div class="relative flex flex-col">
		{#each messages as message, i (message.message_id)}
			<div class="relative pb-3">
				{#if message.type === 'user'}
					<div class="pl-5">
						<ChatBubble {message} />
					</div>
				{:else if message.type === 'system'}
					<div class="pl-5">
						<SystemMessageRenderer {message} />
					</div>
				{:else}
					<CardEnvelope {message} defaultExpanded={i === lastCardIndex} dotVariant={getDotVariant(message)} />
				{/if}
			</div>
		{/each}
	</div>
</div>

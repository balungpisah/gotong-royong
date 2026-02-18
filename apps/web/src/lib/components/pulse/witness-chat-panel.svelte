<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import Send from '@lucide/svelte/icons/send';
	import ChatThread from '$lib/components/chat/chat-thread.svelte';

	interface Props {
		messages: ChatMessage[];
		onSend?: (content: string) => void;
		sending?: boolean;
	}

	let { messages, onSend, sending = false }: Props = $props();

	let inputValue = $state('');
	let scrollContainer: HTMLDivElement | undefined = $state();

	// Auto-scroll to bottom when messages change
	$effect(() => {
		if (messages.length && scrollContainer) {
			// Use tick-like approach
			requestAnimationFrame(() => {
				scrollContainer?.scrollTo({ top: scrollContainer.scrollHeight, behavior: 'smooth' });
			});
		}
	});

	function handleSend() {
		const trimmed = inputValue.trim();
		if (!trimmed || sending) return;
		onSend?.(trimmed);
		inputValue = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<div class="flex flex-1 flex-col overflow-hidden">
	<!-- Messages area -->
	<div bind:this={scrollContainer} class="flex-1 overflow-y-auto px-3 py-3">
		<ChatThread {messages} />
	</div>

	<!-- Input bar -->
	<div class="border-t border-border/40 bg-card px-3 py-2">
		<div class="flex items-end gap-2">
			<textarea
				bind:value={inputValue}
				onkeydown={handleKeydown}
				placeholder="Tulis pesan..."
				rows={1}
				class="max-h-20 min-h-[36px] flex-1 resize-none rounded-xl border border-border/60 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary/40 focus:outline-none"
			></textarea>
			<button
				onclick={handleSend}
				disabled={!inputValue.trim() || sending}
				class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary text-primary-foreground transition hover:bg-primary/90 disabled:opacity-40"
				aria-label="Kirim pesan"
			>
				<Send class="size-4" />
			</button>
		</div>
	</div>
</div>

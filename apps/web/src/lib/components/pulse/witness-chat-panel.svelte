<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import Send from '@lucide/svelte/icons/send';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import ChatThread from '$lib/components/chat/chat-thread.svelte';

	interface Props {
		messages: ChatMessage[];
		onSend?: (content: string) => void;
		onStempel?: () => void;
		sending?: boolean;
		stempeling?: boolean;
	}

	let { messages, onSend, onStempel, sending = false, stempeling = false }: Props = $props();

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

	function handleStempel() {
		if (stempeling) return;
		onStempel?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<div class="flex w-full flex-1 flex-col overflow-hidden">
	<!-- Messages area — sunken conversation well -->
	<div
		bind:this={scrollContainer}
		class="flex-1 overflow-y-auto bg-background px-3 py-3 shadow-[inset_0_3px_8px_-2px_rgba(0,0,0,0.08)]"
	>
		<ChatThread {messages} />
	</div>

	<!-- Input bar -->
	<div class="border-t border-border/40 bg-card/80 px-3 py-2">
		<div class="flex items-end gap-1.5">
			<!-- Stempel — invoke AI to evaluate phase progress -->
			<button
				onclick={handleStempel}
				disabled={stempeling}
				class="flex h-9 shrink-0 items-center gap-1 rounded-xl border border-primary/30 bg-primary/5 px-2 text-xs font-medium text-primary transition hover:bg-primary/10 active:scale-[0.97] disabled:opacity-50"
				aria-label="Stempel — minta AI evaluasi kemajuan fase"
			>
				{#if stempeling}
					<Loader2 class="size-3.5 animate-spin" />
				{:else}
					<span class="text-xs leading-none">✦</span>
					<span>Stempel</span>
				{/if}
			</button>

			<!-- Message input -->
			<textarea
				bind:value={inputValue}
				onkeydown={handleKeydown}
				placeholder="Tulis pesan..."
				rows={1}
				class="max-h-20 min-h-[36px] min-w-0 flex-1 resize-none rounded-xl border border-border/60 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/50 focus:border-primary/40 focus:outline-none"
			></textarea>

			<!-- Send -->
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

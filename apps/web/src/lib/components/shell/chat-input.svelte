<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getTriageStore } from '$lib/stores';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import SendHorizontal from '@lucide/svelte/icons/send-horizontal';

	const triageStore = getTriageStore();

	let content = $state('');
	let textareaEl = $state<HTMLTextAreaElement | null>(null);

	const canSend = $derived(content.trim().length > 0 && !triageStore.loading);

	function autoResize() {
		if (!textareaEl) return;
		textareaEl.style.height = 'auto';
		textareaEl.style.height = Math.min(textareaEl.scrollHeight, 120) + 'px';
	}

	async function handleSubmit() {
		if (!canSend) return;
		const text = content.trim();
		content = '';
		if (textareaEl) textareaEl.style.height = 'auto';
		await triageStore.startTriage(text);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}
</script>

<div
	class="sticky bottom-16 z-10 border-t border-border/60 bg-background/95 px-4 py-3 backdrop-blur md:bottom-0 supports-[backdrop-filter]:bg-background/85"
>
	<div class="mx-auto flex max-w-screen-md items-end gap-2">
		<div class="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
			<Sparkles class="size-4" />
		</div>

		<div class="relative flex-1">
			<textarea
				bind:this={textareaEl}
				bind:value={content}
				oninput={autoResize}
				onkeydown={handleKeydown}
				placeholder={m.shell_chat_placeholder()}
				disabled={triageStore.loading}
				rows={1}
				class="w-full resize-none rounded-xl border border-border bg-muted/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-1 focus:ring-primary/30 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
			></textarea>
		</div>

		<button
			type="button"
			onclick={handleSubmit}
			disabled={!canSend}
			class="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground transition hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-40"
			aria-label={m.shell_chat_send()}
		>
			{#if triageStore.loading}
				<div class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
			{:else}
				<SendHorizontal class="size-4" />
			{/if}
		</button>
	</div>
</div>

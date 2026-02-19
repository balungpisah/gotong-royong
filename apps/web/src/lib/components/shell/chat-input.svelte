<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getTriageStore } from '$lib/stores';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import SendHorizontal from '@lucide/svelte/icons/send-horizontal';
	import X from '@lucide/svelte/icons/x';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	const triageStore = getTriageStore();

	let content = $state('');
	let expanded = $state(false);
	let textareaEl = $state<HTMLTextAreaElement | null>(null);
	let wrapperEl = $state<HTMLDivElement | null>(null);

	/** Chat messages for the current triage session */
	interface ChatMessage {
		role: 'user' | 'ai';
		text: string;
	}
	let messages = $state<ChatMessage[]>([]);

	const canSend = $derived(content.trim().length > 0 && !triageStore.loading);
	const hasSession = $derived(triageStore.sessionId !== null);

	function autoResize() {
		if (!textareaEl) return;
		textareaEl.style.height = 'auto';
		textareaEl.style.height = Math.min(textareaEl.scrollHeight, 120) + 'px';
	}

	function expand() {
		expanded = true;
		requestAnimationFrame(() => textareaEl?.focus());
	}

	function collapse() {
		expanded = false;
	}

	function handleBackdropClick() {
		collapse();
	}

	function aiResponseText(): string {
		if (!triageStore.result) return '';
		const r = triageStore.result;
		const parts: string[] = [];

		if (r.confidence?.label) parts.push(r.confidence.label);

		if (r.bar_state === 'probing') {
			parts.push('Bisa ceritakan lebih detail tentang situasinya?');
		} else if (r.bar_state === 'leaning') {
			parts.push('Saya mulai memahami. Ada informasi tambahan?');
		} else if (
			r.bar_state === 'ready' ||
			r.bar_state === 'vault-ready' ||
			r.bar_state === 'siaga-ready'
		) {
			if (r.track_hint) parts.push(`Jalur: ${r.track_hint}`);
			if (r.seed_hint) parts.push(`Benih: ${r.seed_hint}`);
			parts.push('Triase selesai! Saksi siap dibuat.');
		}

		return parts.join(' · ') || 'Memproses...';
	}

	async function handleSubmit() {
		if (!canSend) return;
		const text = content.trim();
		content = '';
		if (textareaEl) textareaEl.style.height = 'auto';

		messages.push({ role: 'user', text });

		if (!hasSession) {
			await triageStore.startTriage(text);
		} else {
			await triageStore.updateTriage(text);
		}

		messages.push({ role: 'ai', text: aiResponseText() });
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
		if (e.key === 'Escape') {
			collapse();
		}
	}

	function handleReset() {
		triageStore.reset();
		messages = [];
		content = '';
		collapse();
	}
</script>

<!-- Wrapper: keeps the compact card in flow, expanded panel is absolute -->
<div class="relative" bind:this={wrapperEl}>
	<!-- Compact card — always in document flow to hold space -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="cursor-pointer rounded-2xl border border-primary/20 bg-card p-3 ring-1 ring-primary/8 transition-all hover:border-primary/40 hover:ring-primary/15 hover:shadow-sm"
		class:invisible={expanded}
		onclick={expand}
		onkeydown={(e) => e.key === 'Enter' && expand()}
		role="button"
		tabindex="0"
	>
		<div class="flex items-center gap-2">
			<div class="flex size-8 items-center justify-center rounded-full bg-primary/10">
				<Sparkles class="size-3.5 text-primary" />
			</div>
			<span class="flex-1 text-sm text-muted-foreground">
				{m.shell_chat_placeholder()}
			</span>
			<ChevronDown class="size-4 text-muted-foreground/50" />
		</div>
	</div>

	<!-- Backdrop scrim -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-30 bg-black/15 transition-opacity duration-300 ease-out"
		class:opacity-0={!expanded}
		class:pointer-events-none={!expanded}
		onclick={handleBackdropClick}
		onkeydown={() => {}}
	></div>

	<!-- Expanded panel — absolutely positioned, overlays feed -->
	<div
		class="absolute inset-x-0 top-0 z-40 flex flex-col rounded-2xl border border-primary/15 bg-card shadow-lg ring-1 ring-primary/10 transition-all duration-300 ease-out"
		class:opacity-0={!expanded}
		class:pointer-events-none={!expanded}
		class:scale-y-95={!expanded}
		style="max-height: {expanded ? '70vh' : '0px'}; transform-origin: top;"
	>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-border/40 px-4 py-2.5">
			<div class="flex items-center gap-2 text-xs text-muted-foreground">
				<Sparkles class="size-3.5 text-primary" />
				<span class="font-medium">AI-00 Triage</span>
				{#if triageStore.confidence}
					<span
						class="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary"
					>
						{triageStore.confidence.label}
					</span>
				{/if}
			</div>
			<button
				type="button"
				onclick={collapse}
				class="flex size-7 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground"
				aria-label="Tutup"
				tabindex={expanded ? 0 : -1}
			>
				<X class="size-4" />
			</button>
		</div>

		<!-- Messages area -->
		<div class="flex-1 overflow-y-auto px-4 py-3" style="min-height: 360px;">
			{#if messages.length === 0}
				<div class="flex flex-col items-center justify-center gap-2 py-8 text-center">
					<div class="flex size-10 items-center justify-center rounded-full bg-primary/10">
						<Sparkles class="size-5 text-primary" />
					</div>
					<p class="text-sm font-medium text-foreground">Apa yang terjadi di sekitarmu?</p>
					<p class="max-w-[240px] text-xs text-muted-foreground">
						Ceritakan situasi yang kamu saksikan, AI-00 akan membantu mentriase
					</p>
				</div>
			{:else}
				<div class="flex flex-col gap-3">
					{#each messages as msg}
						{#if msg.role === 'user'}
							<div class="flex justify-end">
								<div
									class="max-w-[80%] rounded-2xl rounded-br-md bg-primary px-3 py-2 text-sm text-primary-foreground"
								>
									{msg.text}
								</div>
							</div>
						{:else}
							<div class="flex gap-2">
								<div
									class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10"
								>
									<Sparkles class="size-3 text-primary" />
								</div>
								<div
									class="max-w-[80%] rounded-2xl rounded-bl-md bg-muted/70 px-3 py-2 text-sm text-foreground"
								>
									{msg.text}
								</div>
							</div>
						{/if}
					{/each}

					{#if triageStore.loading}
						<div class="flex gap-2">
							<div
								class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10"
							>
								<Sparkles class="size-3 text-primary" />
							</div>
							<div class="rounded-2xl rounded-bl-md bg-muted/70 px-3 py-2">
								<div class="flex gap-1">
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 0ms"
									></div>
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 150ms"
									></div>
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 300ms"
									></div>
								</div>
							</div>
						</div>
					{/if}

					{#if triageStore.isReady}
						<div
							class="mt-1 rounded-xl border border-green-500/20 bg-green-500/5 px-3 py-2 text-center"
						>
							<p class="text-xs font-medium text-green-700 dark:text-green-400">✓ Triase selesai</p>
							<button
								type="button"
								onclick={handleReset}
								class="mt-1.5 text-xs font-medium text-primary underline-offset-2 hover:underline"
							>
								Mulai baru
							</button>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Input bar -->
		<div class="border-t border-border/40 px-3 py-2.5">
			<div class="flex items-end gap-2">
				<div class="relative flex-1">
					<textarea
						bind:this={textareaEl}
						bind:value={content}
						oninput={autoResize}
						onkeydown={handleKeydown}
						placeholder={triageStore.isReady ? 'Triase selesai' : m.shell_chat_placeholder()}
						disabled={triageStore.loading || triageStore.isReady}
						rows={1}
						tabindex={expanded ? 0 : -1}
						class="w-full resize-none rounded-lg border border-border/50 bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-1 focus:ring-primary/30 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
					></textarea>
				</div>

				<button
					type="button"
					onclick={handleSubmit}
					disabled={!canSend}
					tabindex={expanded ? 0 : -1}
					class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground transition hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-40"
					aria-label={m.shell_chat_send()}
				>
					{#if triageStore.loading}
						<div
							class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
						></div>
					{:else}
						<SendHorizontal class="size-4" />
					{/if}
				</button>
			</div>
		</div>
	</div>
</div>

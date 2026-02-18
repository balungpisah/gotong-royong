<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import Send from '@lucide/svelte/icons/send';
	import Bot from '@lucide/svelte/icons/bot';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Vote from '@lucide/svelte/icons/vote';
	import Banknote from '@lucide/svelte/icons/banknote';
	import FileText from '@lucide/svelte/icons/file-text';
	import Info from '@lucide/svelte/icons/info';

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

	function formatTime(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString('id-ID', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function getInitials(name: string): string {
		return name
			.split(' ')
			.map((w) => w[0])
			.join('')
			.slice(0, 2)
			.toUpperCase();
	}
</script>

<div class="flex flex-1 flex-col overflow-hidden">
	<!-- Messages area -->
	<div bind:this={scrollContainer} class="flex-1 overflow-y-auto px-3 py-3">
		<div class="flex flex-col gap-3">
			{#each messages as msg (msg.message_id)}
				{#if msg.type === 'system'}
					<!-- System message — centered pill -->
					<div class="flex justify-center">
						<span
							class="inline-flex items-center gap-1.5 rounded-full bg-muted/50 px-3 py-1 text-[10px] text-muted-foreground"
						>
							<Info class="size-3" />
							{msg.content}
						</span>
					</div>
				{:else if msg.type === 'user'}
					<!-- User bubble -->
					<div class="flex gap-2 {msg.is_self ? 'flex-row-reverse' : ''}">
						<!-- Avatar -->
						{#if !msg.is_self}
							<div
								class="flex size-7 shrink-0 items-center justify-center rounded-full bg-muted text-[10px] font-semibold text-muted-foreground"
							>
								{getInitials(msg.author.name)}
							</div>
						{/if}

						<div class="max-w-[75%] min-w-0">
							{#if !msg.is_self}
								<p class="mb-0.5 text-[10px] font-medium text-muted-foreground">
									{msg.author.name}
								</p>
							{/if}
							<div
								class="rounded-2xl px-3 py-2 text-xs leading-relaxed {msg.is_self
									? 'bg-primary text-primary-foreground rounded-br-md'
									: 'bg-muted/70 text-foreground rounded-bl-md'}"
							>
								{msg.content}
							</div>
							<p
								class="mt-0.5 text-[9px] text-muted-foreground/60 {msg.is_self
									? 'text-right'
									: ''}"
							>
								{formatTime(msg.timestamp)}
							</p>
						</div>
					</div>
				{:else if msg.type === 'ai_card'}
					<!-- AI card — left aligned with bot icon -->
					<div class="flex gap-2">
						<div
							class="flex size-7 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary"
						>
							<Bot class="size-3.5" />
						</div>
						<div class="max-w-[80%] min-w-0">
							<div
								class="rounded-2xl rounded-bl-md border border-primary/20 bg-primary/5 px-3 py-2"
							>
								{#if msg.title}
									<p class="text-[10px] font-semibold text-primary">
										{msg.title}
									</p>
								{/if}
								{#if msg.badge}
									<span
										class="mt-1 inline-block rounded-full bg-primary/10 px-2 py-0.5 text-[9px] font-medium text-primary"
									>
										🤖 {msg.badge}
									</span>
								{/if}
							</div>
							<p class="mt-0.5 text-[9px] text-muted-foreground/60">
								{formatTime(msg.timestamp)}
							</p>
						</div>
					</div>
				{:else if msg.type === 'evidence'}
					<!-- Evidence card -->
					<div class="flex gap-2">
						<div
							class="flex size-7 shrink-0 items-center justify-center rounded-full bg-peringatan/10 text-peringatan"
						>
							<ShieldCheck class="size-3.5" />
						</div>
						<div class="max-w-[80%] min-w-0">
							<div
								class="rounded-2xl rounded-bl-md border border-peringatan/20 bg-peringatan/5 px-3 py-2"
							>
								<p class="text-[10px] font-semibold text-peringatan">
									Bukti: {msg.evidence_type}
								</p>
								<p class="mt-1 text-xs leading-relaxed text-foreground">
									{msg.content}
								</p>
							</div>
							<p class="mt-0.5 text-[9px] text-muted-foreground/60">
								{formatTime(msg.timestamp)}
							</p>
						</div>
					</div>
				{:else if msg.type === 'diff_card'}
					<!-- Diff card -->
					<div class="flex gap-2">
						<div
							class="flex size-7 shrink-0 items-center justify-center rounded-full bg-keterangan/10 text-keterangan"
						>
							<FileText class="size-3.5" />
						</div>
						<div class="max-w-[80%] min-w-0">
							<div
								class="rounded-2xl rounded-bl-md border border-keterangan/20 bg-keterangan/5 px-3 py-2"
							>
								<p class="text-[10px] font-semibold text-keterangan">
									Perubahan Diusulkan
								</p>
								<p class="mt-1 text-xs text-foreground">
									{msg.diff.summary}
								</p>
								<p class="mt-1 text-[10px] text-muted-foreground">
									{msg.diff.items.length} perubahan
								</p>
							</div>
							<p class="mt-0.5 text-[9px] text-muted-foreground/60">
								{formatTime(msg.timestamp)}
							</p>
						</div>
					</div>
				{:else if msg.type === 'vote_card'}
					<!-- Vote card -->
					<div class="flex gap-2">
						<div
							class="flex size-7 shrink-0 items-center justify-center rounded-full bg-wujudkan/10 text-wujudkan"
						>
							<Vote class="size-3.5" />
						</div>
						<div class="max-w-[80%] min-w-0">
							<div
								class="rounded-2xl rounded-bl-md border border-wujudkan/20 bg-wujudkan/5 px-3 py-2"
							>
								<p class="text-[10px] font-semibold text-wujudkan">Pemungutan Suara</p>
								<p class="mt-1 text-xs text-foreground">
									{msg.block.question}
								</p>
								<div class="mt-2 flex gap-2">
									{#each msg.block.options as opt (opt.id)}
										<span
											class="rounded-full bg-wujudkan/10 px-2 py-0.5 text-[10px] text-wujudkan"
										>
											{opt.label} ({opt.count})
										</span>
									{/each}
								</div>
							</div>
							<p class="mt-0.5 text-[9px] text-muted-foreground/60">
								{formatTime(msg.timestamp)}
							</p>
						</div>
					</div>
				{:else if msg.type === 'galang'}
					<!-- Galang / financial message -->
					<div class="flex justify-center">
						<span
							class="inline-flex items-center gap-1.5 rounded-full bg-berhasil/10 px-3 py-1 text-[10px] text-berhasil"
						>
							<Banknote class="size-3" />
							{msg.content}
							{#if msg.amount}
								— Rp {msg.amount.toLocaleString('id-ID')}
							{/if}
						</span>
					</div>
				{/if}
			{/each}
		</div>
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

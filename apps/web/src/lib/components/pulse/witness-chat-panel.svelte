<script lang="ts">
	import type { ChatMessage, TriageAttachment } from '$lib/types';
	import Send from '@lucide/svelte/icons/send';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import ChatThread from '$lib/components/chat/chat-thread.svelte';
	import TriageAttachmentPicker from '$lib/components/triage/triage-attachment-picker.svelte';
	import TriageAttachmentPreview from '$lib/components/triage/triage-attachment-preview.svelte';

	const MAX_ATTACHMENTS = 5;

	interface Props {
		messages: ChatMessage[];
		onSend?: (content: string, attachments?: File[]) => void;
		onStempel?: () => void;
		sending?: boolean;
		stempeling?: boolean;
	}

	let { messages, onSend, onStempel, sending = false, stempeling = false }: Props = $props();

	let inputValue = $state('');
	let scrollContainer: HTMLDivElement | undefined = $state();
	let pendingAttachments = $state<TriageAttachment[]>([]);

	const canSend = $derived(inputValue.trim().length > 0 || pendingAttachments.length > 0);

	// Auto-scroll to bottom when messages change
	$effect(() => {
		if (messages.length && scrollContainer) {
			// Use tick-like approach
			requestAnimationFrame(() => {
				scrollContainer?.scrollTo({ top: scrollContainer.scrollHeight, behavior: 'smooth' });
			});
		}
	});

	function handleFilesSelected(files: File[]) {
		const remaining = MAX_ATTACHMENTS - pendingAttachments.length;
		if (remaining <= 0) return;
		const accepted = files.slice(0, remaining);
		const newAttachments: TriageAttachment[] = accepted.map((f) => ({
			id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
			file: f,
			type: f.type.startsWith('image/') ? 'image' : f.type.startsWith('video/') ? 'video' : 'audio',
			preview_url: URL.createObjectURL(f)
		}));
		pendingAttachments = [...pendingAttachments, ...newAttachments];
	}

	function handleRemoveAttachment(id: string) {
		const att = pendingAttachments.find((a) => a.id === id);
		if (att) URL.revokeObjectURL(att.preview_url);
		pendingAttachments = pendingAttachments.filter((a) => a.id !== id);
	}

	function handleSend() {
		const trimmed = inputValue.trim();
		if (!canSend || sending) return;
		const files = pendingAttachments.length > 0 ? pendingAttachments.map((a) => a.file) : undefined;
		onSend?.(trimmed, files);
		inputValue = '';
		// Revoke preview URLs
		for (const att of pendingAttachments) URL.revokeObjectURL(att.preview_url);
		pendingAttachments = [];
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
		class="flex-1 overflow-y-auto bg-background px-3 py-3"
		style="box-shadow: inset 0 3px 8px -2px color-mix(in srgb, var(--accent, gray) 12%, transparent);"
	>
		<ChatThread {messages} />
	</div>

	<!-- Input bar -->
	<div class="border-t bg-card/80 px-3 py-2"
		style="border-color: color-mix(in srgb, var(--accent, gray) 15%, var(--color-border));"
	>
		<!-- Attachment previews -->
		<TriageAttachmentPreview attachments={pendingAttachments} onRemove={handleRemoveAttachment} />

		<div class="flex items-end gap-1.5">
			{#if onStempel}
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
			{/if}

			<!-- Attachment picker -->
			<TriageAttachmentPicker onFilesSelected={handleFilesSelected} disabled={pendingAttachments.length >= MAX_ATTACHMENTS} />

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
				disabled={!canSend || sending}
				class="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary text-primary-foreground transition hover:bg-primary/90 disabled:opacity-40"
				aria-label="Kirim pesan"
			>
				<Send class="size-4" />
			</button>
		</div>
	</div>
</div>

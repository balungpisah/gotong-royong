<script lang="ts">
	import { page } from '$app/state';
	import { getWitnessStore } from '$lib/stores';
	import { WitnessDetailPanel } from '$lib/components/pulse';

	const witnessStore = getWitnessStore();
	const witnessId = $derived(page.params.witness_id?.trim() ?? '');
	const detail = $derived(
		witnessStore.current?.witness_id === witnessId ? witnessStore.current : null
	);

	$effect(() => {
		if (witnessId) {
			witnessStore.loadDetail(witnessId);
		}
	});

	async function retryLoadDetail() {
		if (!witnessId) return;
		await witnessStore.loadDetail(witnessId);
	}

	async function handleSendMessage(content: string, attachments?: File[]) {
		await witnessStore.sendMessage(content, attachments);
	}
</script>

<div class="mx-auto w-full max-w-3xl space-y-4">
	<a href="/" class="text-small font-semibold text-primary hover:underline">Kembali ke beranda</a>

	{#if witnessStore.detailLoading && !detail}
		<div class="flex h-48 items-center justify-center">
			<div class="flex flex-col items-center gap-3 text-muted-foreground">
				<div class="size-7 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
				<p class="text-small">Memuat detail saksi...</p>
			</div>
		</div>
	{:else if detail}
		<div class="h-[calc(100dvh-10rem)] min-h-[28rem] overflow-hidden rounded-xl border border-border/40 bg-card">
			<WitnessDetailPanel detail={detail} onSendMessage={handleSendMessage} />
		</div>
	{:else if witnessStore.detailError}
		<div class="rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3">
			<p class="text-small text-destructive">{witnessStore.detailError}</p>
			<button
				type="button"
				class="mt-2 rounded-md border border-border px-2 py-1 text-small text-foreground hover:bg-muted/40"
				onclick={retryLoadDetail}
			>
				Coba lagi
			</button>
		</div>
	{/if}
</div>

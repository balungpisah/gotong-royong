<script lang="ts">
	import type { VoteBlock } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import CheckCircle from '@lucide/svelte/icons/circle-check';

	let { block, onvote, bare = false }: { block: VoteBlock; onvote?: (optionId: string) => void; bare?: boolean } = $props();

	const quorumPercent = $derived(Math.round((block.total_voted / block.total_eligible) * 100));
	const quorumTarget = $derived(Math.round(block.quorum * 100));
	const isEnded = $derived(new Date(block.ends_at) <= new Date());
	const timeLeft = $derived((() => {
		const diff = new Date(block.ends_at).getTime() - Date.now();
		if (diff <= 0) return 'Selesai';
		const hours = Math.floor(diff / (1000 * 60 * 60));
		if (hours >= 24) return `${Math.floor(hours / 24)} hari lagi`;
		return `${hours} jam lagi`;
	})());

	const voteTypeLabels: Record<string, string> = {
		standard: 'Suara Biasa',
		weighted: 'Suara Berbobot',
		quorum_1_5x: 'Kuorum 1.5×',
		consensus: 'Konsensus'
	};
</script>

<div class={cn('flex flex-col gap-3', !bare && 'rounded-lg border border-border bg-card p-4')} data-slot="vote-block">
	<div class="flex items-start justify-between gap-2">
		<p class="text-sm font-bold text-foreground">{block.question}</p>
		<Badge variant="info" class="shrink-0 text-xs">{voteTypeLabels[block.vote_type] || block.vote_type}</Badge>
	</div>

	<div class="flex flex-col gap-2">
		{#each block.options as option (option.id)}
			{@const optPercent = block.total_voted > 0 ? Math.round((option.count / block.total_voted) * 100) : 0}
			<button
				class={cn(
					'relative flex items-center justify-between rounded-md border px-3 py-2 text-sm transition',
					isEnded || block.user_voted
						? 'cursor-default border-border'
						: 'cursor-pointer border-border hover:border-api hover:bg-api/5'
				)}
				disabled={isEnded || block.user_voted}
				onclick={() => onvote?.(option.id)}
			>
				<span class="z-10 font-medium">{option.label}</span>
				<span class="z-10 text-xs text-muted-foreground">{option.count} ({optPercent}%)</span>
				{#if block.total_voted > 0}
					<div
						class="absolute inset-y-0 left-0 rounded-md bg-api/10"
						style="width: {optPercent}%"
					></div>
				{/if}
			</button>
		{/each}
	</div>

	<div class="flex flex-col gap-1.5">
		<div class="flex items-center justify-between text-xs text-muted-foreground">
			<span>Kuorum: {quorumPercent}% / {quorumTarget}%</span>
			<span>{block.total_voted}/{block.total_eligible} suara</span>
		</div>
		<Progress value={quorumPercent} max={100} class="h-1" />
	</div>

	<div class="flex items-center justify-between text-xs">
		{#if block.user_voted}
			<div class="flex items-center gap-1 text-berhasil">
				<CheckCircle class="size-3" />
				<span class="font-medium">Sudah memilih</span>
			</div>
		{:else}
			<span class="text-muted-foreground">Belum memilih</span>
		{/if}
		<span class={cn('font-medium', isEnded ? 'text-bahaya' : 'text-muted-foreground')}>
			{timeLeft}
		</span>
	</div>
</div>

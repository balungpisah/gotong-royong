<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import ThumbsUp from '@lucide/svelte/icons/thumbs-up';
	import Heart from '@lucide/svelte/icons/heart';
	import type { DukungRecord } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		given: DukungRecord[];
		received: DukungRecord[];
		successRate?: number | null;
	}

	const { given, received, successRate }: Props = $props();

	const SHOW_LIMIT = 5;
	let showAllGiven = $state(false);
	let showAllReceived = $state(false);

	const visibleGiven = $derived(showAllGiven ? given : given.slice(0, SHOW_LIMIT));
	const visibleReceived = $derived(showAllReceived ? received : received.slice(0, SHOW_LIMIT));

	const outcomeColors: Record<string, string> = {
		success: 'text-berhasil bg-berhasil-lembut',
		slashed: 'text-bahaya bg-bahaya-lembut',
		pending: 'text-waspada bg-waspada-lembut'
	};

	const outcomeLabels = $derived({
		success: m.dukung_outcome_success(),
		slashed: m.dukung_outcome_slashed(),
		pending: m.dukung_outcome_pending()
	} as Record<string, string>);

	function formatDate(iso: string): string {
		const locale = getLocale() === 'en' ? 'en-US' : 'id-ID';
		return new Date(iso).toLocaleDateString(locale, { day: 'numeric', month: 'short', year: 'numeric' });
	}
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.25 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4">
		<div class="flex items-center justify-between">
			<h3 class="text-small font-semibold text-foreground">{m.profil_dukung_title()}</h3>
			{#if successRate !== undefined && successRate !== null}
				<span class="rounded-full bg-berhasil-lembut px-2 py-0.5 text-caption font-medium text-berhasil">
					{m.profil_dukung_success_rate({ pct: String(Math.round(successRate * 100)) })}
				</span>
			{/if}
		</div>

		<!-- Given -->
		{#if given.length > 0}
			<div class="mt-3">
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<ThumbsUp class="size-3" />
					<span class="text-caption font-medium">{m.dukung_given_count({ count: String(given.length) })}</span>
				</div>
				<div class="mt-2 space-y-1.5">
					{#each visibleGiven as record}
						<div class="flex items-center justify-between rounded-lg bg-muted/20 px-3 py-2">
							<div class="min-w-0 flex-1">
								<p class="truncate text-caption text-foreground/80">{record.witness_title}</p>
								<p class="text-caption text-muted-foreground">{formatDate(record.created_at)}</p>
							</div>
							{#if record.outcome}
								<span class="ml-2 shrink-0 rounded-full px-2 py-0.5 text-caption font-medium {outcomeColors[record.outcome] ?? ''}">
									{outcomeLabels[record.outcome] ?? record.outcome}
								</span>
							{/if}
						</div>
					{/each}
				</div>
				{#if given.length > SHOW_LIMIT}
					<Button
	variant="ghost"
						onclick={() => (showAllGiven = !showAllGiven)}
						class="mt-2 h-auto p-0 text-caption text-muted-foreground hover:text-foreground"
					>
						{showAllGiven ? m.common_close() : m.common_more({ count: String(given.length - SHOW_LIMIT) })}
					</Button>
				{/if}
			</div>
		{/if}

		<!-- Received -->
		{#if received.length > 0}
			<div class="mt-4">
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<Heart class="size-3" />
					<span class="text-caption font-medium">{m.dukung_received_count({ count: String(received.length) })}</span>
				</div>
				<div class="mt-2 space-y-1.5">
					{#each visibleReceived as record}
						<div class="flex items-center justify-between rounded-lg bg-muted/20 px-3 py-2">
							<div class="min-w-0 flex-1">
								<p class="truncate text-caption text-foreground/80">{record.witness_title}</p>
								<p class="text-caption text-muted-foreground">{formatDate(record.created_at)}</p>
							</div>
							{#if record.outcome}
								<span class="ml-2 shrink-0 rounded-full px-2 py-0.5 text-caption font-medium {outcomeColors[record.outcome] ?? ''}">
									{outcomeLabels[record.outcome] ?? record.outcome}
								</span>
							{/if}
						</div>
					{/each}
				</div>
				{#if received.length > SHOW_LIMIT}
					<Button
	variant="ghost"
						onclick={() => (showAllReceived = !showAllReceived)}
						class="mt-2 h-auto p-0 text-caption text-muted-foreground hover:text-foreground"
					>
						{showAllReceived ? m.common_close() : m.common_more({ count: String(received.length - SHOW_LIMIT) })}
					</Button>
				{/if}
			</div>
		{/if}

		{#if given.length === 0 && received.length === 0}
			<p class="mt-3 text-caption text-muted-foreground">{m.dukung_empty()}</p>
		{/if}
	</div>
</motion.div>

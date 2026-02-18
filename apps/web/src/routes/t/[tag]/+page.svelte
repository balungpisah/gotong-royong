<script lang="ts">
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getWitnessStore } from '$lib/stores';
	import { PulseActivityCard } from '$lib/components/pulse';
	import { Badge } from '$lib/components/ui/badge';
	import type { BadgeVariant } from '$lib/components/ui/badge';
	import Activity from '@lucide/svelte/icons/activity';

	const witnessStore = getWitnessStore();

	const tag = $derived(page.params.tag ?? '');

	const trackVariantMap: Record<string, BadgeVariant> = {
		tuntaskan: 'track-tuntaskan',
		wujudkan: 'track-wujudkan',
		telusuri: 'track-telusuri',
		rayakan: 'track-rayakan',
		musyawarah: 'track-musyawarah'
	};

	// Load witnesses if not already loaded
	$effect(() => {
		if (witnessStore.witnesses.length === 0 && !witnessStore.listLoading) {
			witnessStore.loadList();
		}
	});

	const filteredWitnesses = $derived(
		tag
			? witnessStore.witnesses
					.filter((w) => w.track_hint === tag)
					.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
			: []
	);

	const displayTag = $derived(tag ? tag.charAt(0).toUpperCase() + tag.slice(1) : '');
</script>

<div class="flex flex-1 flex-col gap-6">
	<!-- Title row -->
	<div class="flex items-center gap-3">
		<Badge variant={trackVariantMap[tag] ?? 'secondary'} class="px-3 py-1 text-sm font-semibold">
			{displayTag}
		</Badge>
		<span class="text-sm text-muted-foreground">
			{filteredWitnesses.length} {m.tag_page_witness_count()}
		</span>
	</div>

	<!-- Witness list -->
	{#if witnessStore.listLoading}
		<div class="flex flex-col gap-3">
			{#each { length: 3 } as _}
				<div class="animate-pulse rounded-xl border border-border/40 bg-muted/30 p-4">
					<div class="h-4 w-3/4 rounded bg-muted"></div>
					<div class="mt-2 h-3 w-full rounded bg-muted/60"></div>
					<div class="mt-1 h-3 w-2/3 rounded bg-muted/60"></div>
					<div class="mt-3 flex gap-2">
						<div class="h-5 w-16 rounded-full bg-muted/40"></div>
						<div class="h-5 w-10 rounded bg-muted/40"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if filteredWitnesses.length === 0}
		<div
			class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-border/60 py-12 text-center"
		>
			<div
				class="flex size-12 items-center justify-center rounded-full bg-muted/50 text-muted-foreground"
			>
				<Activity class="size-6" />
			</div>
			<p class="max-w-xs text-sm text-muted-foreground">
				{m.tag_page_empty()}
			</p>
		</div>
	{:else}
		<div class="flex flex-col gap-3">
			{#each filteredWitnesses as witness (witness.witness_id)}
				<PulseActivityCard {witness} />
			{/each}
		</div>
	{/if}
</div>

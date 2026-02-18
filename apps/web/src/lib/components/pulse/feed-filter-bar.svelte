<script lang="ts">
	import type { FeedFilter } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		activeFilter: FeedFilter;
		onFilterChange: (f: FeedFilter) => void;
	}

	let { activeFilter, onFilterChange }: Props = $props();

	const filters: { key: FeedFilter; icon: string; labelKey: () => string }[] = [
		{ key: 'semua', icon: '', labelKey: () => m.pulse_feed_filter_all() },
		{ key: 'ikutan', icon: '📌', labelKey: () => m.pulse_feed_filter_ikutan() },
		{ key: 'terlibat', icon: '🔔', labelKey: () => m.pulse_feed_filter_terlibat() },
		{ key: 'sekitar', icon: '🌏', labelKey: () => m.pulse_feed_filter_sekitar() }
	];
</script>

<div class="flex gap-2 overflow-x-auto pb-1 scrollbar-none" role="tablist" aria-label="Feed filters">
	{#each filters as filter (filter.key)}
		<button
			role="tab"
			aria-selected={activeFilter === filter.key}
			class="inline-flex shrink-0 items-center gap-1 rounded-full px-3 py-1.5 text-xs font-semibold whitespace-nowrap transition-colors
				{activeFilter === filter.key
				? 'bg-primary text-primary-foreground shadow-sm'
				: 'bg-kapas text-kayu hover:bg-batu/40'}"
			onclick={() => onFilterChange(filter.key)}
		>
			{#if filter.icon}
				<span class="text-[11px]">{filter.icon}</span>
			{/if}
			{filter.labelKey()}
		</button>
	{/each}
</div>

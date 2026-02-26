<script lang="ts">
	import type { GdfWeather, WeatherType } from '$lib/types';

	interface Props {
		weather: GdfWeather;
		size?: 'compact' | 'full';
	}

	const { weather, size = 'full' }: Props = $props();

	const bgClasses: Record<WeatherType, string> = {
		cerah: 'bg-berhasil-lembut dark:bg-berhasil/10',
		berawan: 'bg-keterangan-lembut dark:bg-keterangan/10',
		hujan: 'bg-muted/30 dark:bg-muted/20',
		badai: 'bg-bahaya-lembut dark:bg-bahaya/10'
	};

	const bgClass = $derived(bgClasses[weather.weather] ?? 'bg-muted/10');

	function capitalize(s: string): string {
		return s.charAt(0).toUpperCase() + s.slice(1);
	}
</script>

{#if size === 'compact'}
	<span
		class="inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-caption font-medium {bgClass}"
	>
		{weather.emoji}
		{weather.label}
		<span class="font-bold">{weather.multiplier}×</span>
	</span>
{:else}
	<div class="flex items-center gap-4 rounded-xl p-5 {bgClass}">
		<span class="text-4xl">{weather.emoji}</span>
		<div>
			<p class="text-h3 font-bold text-foreground">{capitalize(weather.weather)}</p>
			<p class="text-caption text-muted-foreground">{weather.label}</p>
		</div>
		<span class="ml-auto rounded-full bg-card/50 px-3 py-1 text-body font-bold"
			>{weather.multiplier}×</span
		>
	</div>
{/if}

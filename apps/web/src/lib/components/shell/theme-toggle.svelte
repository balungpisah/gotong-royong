<script lang="ts">
	import { getThemeStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';

	const themeStore = getThemeStore();

	const iconMap = {
		light: Sun,
		dark: Moon,
		system: Monitor
	} as const;

	const Icon = $derived(iconMap[themeStore.mode]);
</script>

<button
	type="button"
	onclick={() => themeStore.toggle()}
	class="relative inline-flex items-center justify-center rounded-full p-2 text-muted-foreground transition hover:bg-muted hover:text-foreground"
	aria-label={m.shell_theme_toggle()}
	title={m.shell_theme_toggle()}
>
	<Icon class="size-5" />
</button>

<script lang="ts">
	import { getThemeStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';
	import { Button } from '$lib/components/ui/button';

	const themeStore = getThemeStore();

	const iconMap = {
		light: Sun,
		dark: Moon,
		system: Monitor
	} as const;

	const Icon = $derived(iconMap[themeStore.mode]);
</script>

<Button
	variant="ghost"
	size="icon"
	onclick={() => themeStore.toggle()}
	class="rounded-full"
	aria-label={m.shell_theme_toggle()}
	title={m.shell_theme_toggle()}
>
	<Icon class="size-5" />
</Button>

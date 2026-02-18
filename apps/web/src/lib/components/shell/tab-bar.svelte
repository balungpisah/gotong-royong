<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getNavigationStore } from '$lib/stores';
	import TabBarItem from './tab-bar-item.svelte';
	import Plus from '@lucide/svelte/icons/plus';

	const navStore = getNavigationStore();

	function hrefForTab(tab: { tag: string | null }): string {
		return tab.tag === null ? `${base}/` : `${base}/t/${tab.tag}`;
	}

	function isActive(tab: { tag: string | null }): boolean {
		const pathname = page.url.pathname;
		if (tab.tag === null) {
			// Pulse is active on root path
			return pathname === `${base}/` || pathname === base || pathname === `${base}`;
		}
		return pathname === `${base}/t/${tab.tag}`;
	}
</script>

<!-- Mobile bottom tab bar -->
<nav
	class="fixed inset-x-0 bottom-0 z-30 border-t border-border/80 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/85 md:hidden"
>
	<div
		class="mx-auto grid max-w-screen-md px-2 py-2"
		style="grid-template-columns: repeat({navStore.tabCount + 1}, minmax(0, 1fr));"
	>
		{#each navStore.tabs as tab (tab.id)}
			<TabBarItem
				href={hrefForTab(tab)}
				label={tab.label}
				iconName={tab.iconName}
				active={isActive(tab)}
				removable={!tab.pinned}
				onremove={() => navStore.removeTab(tab.id)}
			/>
		{/each}

		<!-- Add tab button -->
		<button
			type="button"
			onclick={() => navStore.openAddPanel()}
			class="inline-flex flex-col items-center justify-center gap-1 rounded-lg px-2 py-2 text-[11px] font-medium text-muted-foreground transition hover:bg-muted hover:text-foreground"
			aria-label={m.shell_nav_add_tab()}
		>
			<Plus class="size-4" />
			<span class="sr-only">{m.shell_nav_add_tab()}</span>
		</button>
	</div>
</nav>

<!-- Desktop horizontal pill bar -->
<nav class="hidden border-t border-border/60 md:block">
	<div
		class="mx-auto flex w-full max-w-screen-xl items-center gap-2 px-6 py-2"
	>
		{#each navStore.tabs as tab (tab.id)}
			<TabBarItem
				href={hrefForTab(tab)}
				label={tab.label}
				iconName={tab.iconName}
				active={isActive(tab)}
				removable={!tab.pinned}
				onremove={() => navStore.removeTab(tab.id)}
			/>
		{/each}

		<!-- Add tab button -->
		<button
			type="button"
			onclick={() => navStore.openAddPanel()}
			class="inline-flex items-center gap-2 rounded-full px-3 py-2 text-sm font-medium text-muted-foreground transition hover:bg-muted hover:text-foreground"
			aria-label={m.shell_nav_add_tab()}
		>
			<Plus class="size-4" />
			<span>{m.shell_nav_add_tab()}</span>
		</button>
	</div>
</nav>

<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getNavigationStore, getFeedStore } from '$lib/stores';
	import type { TabConfig } from '$lib/types';
	import TabBarItem from './tab-bar-item.svelte';
	import Plus from '@lucide/svelte/icons/plus';

	const navStore = getNavigationStore();
	const feedStore = getFeedStore();

	function hrefForTab(tab: TabConfig): string {
		// Feed tabs always point to root
		if (tab.feedFilter) return `${base}/`;
		// Tag tabs point to /t/{tag}
		return tab.tag === null ? `${base}/` : `${base}/t/${tab.tag}`;
	}

	function isActive(tab: TabConfig): boolean {
		// Feed tabs: active when this tab is the active tab AND we're on root
		if (tab.feedFilter) {
			return navStore.activeTabId === tab.id;
		}
		// Tag tabs: active by URL
		const pathname = page.url.pathname;
		if (tab.tag === null) {
			return pathname === `${base}/` || pathname === base;
		}
		return pathname === `${base}/t/${tab.tag}`;
	}

	function handleTabClick(tab: TabConfig, e: MouseEvent) {
		if (tab.feedFilter) {
			e.preventDefault();
			navStore.setActiveTab(tab.id);
			feedStore.setFilter(tab.feedFilter);
			// Navigate to root if we're not already there
			const pathname = page.url.pathname;
			if (pathname !== `${base}/` && pathname !== base) {
				goto(`${base}/`);
			}
		}
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
				onclick={(e) => handleTabClick(tab, e)}
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
<nav class="hidden border-t border-border/40 md:block">
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
				onclick={(e) => handleTabClick(tab, e)}
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

<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getNavigationStore, getFeedStore, getNotificationStore, getUserStore, getThemeStore, getPreferencesStore } from '$lib/stores';
	import { resolveTabIcon } from '$lib/utils';
	import type { TabConfig } from '$lib/types';
	import Plus from '@lucide/svelte/icons/plus';
	import BellRing from '@lucide/svelte/icons/bell-ring';
	import User from '@lucide/svelte/icons/user';
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';
	import X from '@lucide/svelte/icons/x';
	import CircleHelp from '@lucide/svelte/icons/circle-help';
	import Tip from '$lib/components/ui/tip.svelte';

	const navStore = getNavigationStore();
	const feedStore = getFeedStore();
	const notificationStore = getNotificationStore();
	const userStore = getUserStore();
	const themeStore = getThemeStore();
	const prefsStore = getPreferencesStore();

	const themeIconMap = { light: Sun, dark: Moon, system: Monitor } as const;
	const ThemeIcon = $derived(themeIconMap[themeStore.mode]);

	// ---------------------------------------------------------------------------
	// Click-to-expand state
	// ---------------------------------------------------------------------------

	let expanded = $state(false);

	function toggleSidebar() {
		expanded = !expanded;
	}

	// ---------------------------------------------------------------------------
	// Tab navigation (same logic as tab-bar.svelte)
	// ---------------------------------------------------------------------------

	function hrefForTab(tab: TabConfig): string {
		if (tab.feedFilter) return `${base}/`;
		return tab.tag === null ? `${base}/` : `${base}/t/${tab.tag}`;
	}

	function isActive(tab: TabConfig): boolean {
		if (tab.feedFilter) {
			return navStore.activeTabId === tab.id;
		}
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
			const pathname = page.url.pathname;
			if (pathname !== `${base}/` && pathname !== base) {
				goto(`${base}/`);
			}
		}
	}
</script>

<!--
	AppSidebar — collapsible left navigation rail.

	Collapsed: 68px icon-only rail.
	Click anywhere on the sidebar to expand/collapse (220px with labels).
	Colors match the AppHeader (bg-background/95, backdrop-blur, border-border).
	Top: feed filter tabs + add tab button.
	Bottom: notifications, profile, theme toggle.

	Hidden on mobile (<lg) — mobile keeps bottom TabBar.
-->

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<nav
	class="sidebar hidden lg:flex"
	class:sidebar-expanded={expanded}
	aria-label="App navigation"
	onclick={toggleSidebar}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleSidebar(); } }}
>
	<!-- Top section: feed filters -->
	<div class="flex flex-1 flex-col gap-0.5 pt-3">
		{#each navStore.tabs as tab (tab.id)}
			{@const Icon = resolveTabIcon(tab.iconName)}
			{@const active = isActive(tab)}
			<Tip text={tab.label} side="right">
			<a
				href={hrefForTab(tab)}
				class="sidebar-item group"
				class:sidebar-item-active={active}
				aria-current={active ? 'page' : undefined}
				onclick={(e) => { e.stopPropagation(); handleTabClick(tab, e); }}
			>
				<span class="sidebar-icon">
					<Icon class="size-5" />
				</span>
				<span class="sidebar-label">{tab.label}</span>

				{#if !tab.pinned && !active}
					<button
						type="button"
						class="sidebar-remove"
						onclick={(e) => {
							e.preventDefault();
							e.stopPropagation();
							navStore.removeTab(tab.id);
						}}
						aria-label="Remove tab"
					>
						<X class="size-3" />
					</button>
				{/if}
			</a>
			</Tip>
		{/each}

		<!-- Add tab button -->
		<Tip text={m.shell_nav_add_tab()} side="right">
		<button
			type="button"
			class="sidebar-item sidebar-item-muted"
			onclick={(e) => { e.stopPropagation(); navStore.openAddPanel(); }}
			aria-label={m.shell_nav_add_tab()}
		>
			<span class="sidebar-icon">
				<Plus class="size-5" />
			</span>
			<span class="sidebar-label">{m.shell_nav_add_tab()}</span>
		</button>
		</Tip>
	</div>

	<!-- Separator -->
	<div class="mx-3 my-1 border-t border-border/40"></div>

	<!-- Bottom section: app nav -->
	<div class="flex flex-col gap-0.5 pb-3 pt-1">
		<!-- Notifications -->
		<Tip text={m.shell_nav_notifikasi()} side="right">
		<a
			href="{base}/notifikasi"
			class="sidebar-item sidebar-item-muted"
			onclick={(e) => e.stopPropagation()}
			aria-label={m.shell_nav_notifikasi()}
		>
			<span class="sidebar-icon">
				<BellRing class="size-5" />
			</span>
			<span class="sidebar-label">{m.shell_nav_notifikasi()}</span>
			{#if notificationStore.hasUnread}
				<span class="notif-badge">
					{notificationStore.unreadCount > 9 ? '9+' : notificationStore.unreadCount}
				</span>
			{/if}
		</a>
		</Tip>

		<!-- Profile -->
		<Tip text={m.shell_nav_profil()} side="right">
		<a
			href="{base}/profil"
			class="sidebar-item sidebar-item-muted"
			onclick={(e) => e.stopPropagation()}
			aria-label={m.shell_nav_profil()}
		>
			<span class="sidebar-icon">
				<User class="size-5" />
			</span>
			<span class="sidebar-label">{userStore.displayName}</span>
		</a>
		</Tip>

		<!-- Theme toggle -->
		<Tip text="Ganti tema" side="right">
		<button
			type="button"
			class="sidebar-item sidebar-item-muted"
			onclick={(e) => { e.stopPropagation(); themeStore.toggle(); }}
			aria-label="Toggle tema"
		>
			<span class="sidebar-icon">
				<ThemeIcon class="size-5" />
			</span>
			<span class="sidebar-label">Tema</span>
		</button>
		</Tip>

		<!-- Tooltip toggle -->
		<Tip text={prefsStore.showTooltips ? 'Tooltip aktif' : 'Tooltip nonaktif'} side="right">
		<button
			type="button"
			class="sidebar-item sidebar-item-muted"
			onclick={(e) => { e.stopPropagation(); prefsStore.toggleTooltips(); }}
			aria-label={prefsStore.showTooltips ? 'Tooltip aktif' : 'Tooltip nonaktif'}
		>
			<span class="sidebar-icon">
				<CircleHelp class="size-5 transition-opacity {prefsStore.showTooltips ? '' : 'opacity-40'}" />
			</span>
			<span class="sidebar-label">{prefsStore.showTooltips ? 'Tooltip aktif' : 'Tooltip mati'}</span>
		</button>
		</Tip>

	</div>
</nav>

<style>
	/* ── Sidebar container ────────────────────────────────── */
	/* Matches AppHeader: bg-background/95, backdrop-blur, border-border */
	.sidebar {
		position: fixed;
		left: 0;
		top: 3.5rem;
		height: calc(100dvh - 3.5rem);
		width: 68px;
		flex-direction: column;
		background: oklch(from var(--color-background) l c h / 0.95);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		border-right: 1px solid var(--color-border);
		overflow-x: hidden;
		overflow-y: auto;
		transition: width 220ms cubic-bezier(0.4, 0, 0.2, 1);
		z-index: 25;
		cursor: pointer;
	}

	.sidebar-expanded {
		width: 220px;
	}

	/* ── Nav item ─────────────────────────────────────────── */
	.sidebar-item {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0;
		height: 44px;
		border-radius: var(--r-md);
		color: var(--color-foreground);
		font-size: 0.8125rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 150ms ease, color 150ms ease;
		text-decoration: none;
		white-space: nowrap;
		flex-shrink: 0;
		/* Reset button defaults so <button> aligns with <a> */
		border: none;
		background: none;
		padding: 0;
		text-align: left;
		width: 100%;
	}

	.sidebar-item:hover {
		background: var(--color-muted);
	}

	.sidebar-item-active {
		background: oklch(from var(--c-batu) l c h / 0.18);
		color: var(--color-primary);
		font-weight: 600;
	}

	.sidebar-item-active:hover {
		background: oklch(from var(--c-batu) l c h / 0.22);
	}

	.sidebar-item-muted {
		color: var(--color-muted-foreground);
	}

	.sidebar-item-muted:hover {
		color: var(--color-foreground);
	}

	/* ── Icon cell — matches sidebar collapsed width for perfect centering ── */
	.sidebar-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 68px;
		flex-shrink: 0;
	}

	/* ── Label — hidden in collapsed, fades in when expanded ── */
	.sidebar-label {
		opacity: 0;
		transition: opacity 180ms ease 60ms;
		pointer-events: none;
		flex: 1;
		min-width: 0;
		padding-right: 12px;
	}

	.sidebar-expanded .sidebar-label {
		opacity: 1;
		pointer-events: auto;
	}

	/* ── Remove button on custom tabs ─────────────────────── */
	.sidebar-remove {
		position: absolute;
		right: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2px;
		border-radius: var(--r-sm);
		color: var(--color-muted-foreground);
		opacity: 0;
		transition: opacity 150ms ease, background-color 150ms ease;
		cursor: pointer;
	}

	.sidebar-expanded .group:hover .sidebar-remove {
		opacity: 1;
	}

	.sidebar-remove:hover {
		background: oklch(from var(--c-batu) l c h / 0.15);
		color: var(--color-foreground);
	}

	/* ── Notification badge — positioned within the sidebar-item flow ── */
	.notif-badge {
		position: absolute;
		top: 6px;
		left: 44px;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		padding: 0 4px;
		border-radius: 999px;
		background: var(--color-destructive);
		color: white;
		font-size: 10px;
		font-weight: 700;
		line-height: 1;
	}
</style>

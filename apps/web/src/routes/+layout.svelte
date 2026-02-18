<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import favicon from '$lib/assets/favicon.svg';
	import BellRing from '@lucide/svelte/icons/bell-ring';
	import HandHeart from '@lucide/svelte/icons/hand-heart';
	import House from '@lucide/svelte/icons/house';
	import UserRound from '@lucide/svelte/icons/user-round';
	import Users from '@lucide/svelte/icons/users';
	import type { Component } from 'svelte';
	import '../app.css';

	let { children } = $props();

	/**
	 * Navigation items for the 5-tab bottom nav (UI Guideline §5.1).
	 * Routes are string paths — pages will be created when the
	 * block renderer system is built.
	 */
	const navItems: { href: string; label: () => string; icon: Component<{ class?: string }> }[] = [
		{ href: '/beranda', label: m.shell_nav_beranda, icon: House },
		{ href: '/terlibat', label: m.shell_nav_terlibat, icon: Users },
		{ href: '/bantu', label: m.shell_nav_bantu, icon: HandHeart },
		{ href: '/notifikasi', label: m.shell_nav_notifikasi, icon: BellRing },
		{ href: '/profil', label: m.shell_nav_profil, icon: UserRound }
	];

	const loginPath = `${base}/masuk`;
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if page.url.pathname === loginPath}
	<main class="mx-auto flex min-h-dvh w-full max-w-screen-md px-4 py-6">
		{@render children()}
	</main>
{:else}
	<div class="min-h-dvh bg-background text-foreground">
		<div class="mx-auto flex min-h-dvh w-full max-w-screen-xl flex-col md:px-6">
			<header
				class="sticky top-0 z-20 border-b border-border/80 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/85"
			>
				<div
					class="mx-auto flex w-full max-w-screen-md items-center justify-between gap-3 px-4 py-3 md:max-w-none md:px-0"
				>
					<a
						href="{base}/beranda"
						class="text-sm font-extrabold tracking-wide text-foreground uppercase"
					>
						{m.shell_brand_name()}
					</a>
					<p class="hidden text-sm text-muted-foreground md:block">{m.shell_brand_tagline()}</p>
				</div>

				<nav class="hidden border-t border-border/60 md:block">
					<div
						class="mx-auto flex w-full max-w-screen-md items-center gap-2 px-4 py-2 md:max-w-none md:px-0"
					>
						{#each navItems as item (item.href)}
							{@const Icon = item.icon}
							{@const resolvedHref = `${base}${item.href}`}
							{@const active = page.url.pathname === resolvedHref}
							<a
								class={[
									'inline-flex items-center gap-2 rounded-full px-3 py-2 text-sm font-medium transition',
									active
										? 'bg-primary text-primary-foreground'
										: 'text-muted-foreground hover:bg-muted hover:text-foreground'
								]}
								href={resolvedHref}
								aria-current={active ? 'page' : undefined}
							>
								<Icon class="size-4" />
								<span>{item.label()}</span>
							</a>
						{/each}
					</div>
				</nav>
			</header>

			<main
				class="mx-auto flex w-full max-w-screen-md flex-1 flex-col px-4 py-6 pb-24 md:max-w-none md:px-0 md:pb-8"
			>
				{@render children()}
			</main>
		</div>

		<nav
			class="fixed inset-x-0 bottom-0 z-30 border-t border-border/80 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/85 md:hidden"
		>
			<div class="mx-auto grid max-w-screen-md grid-cols-5 px-2 py-2">
				{#each navItems as item (item.href)}
					{@const Icon = item.icon}
					{@const resolvedHref = `${base}${item.href}`}
					{@const active = page.url.pathname === resolvedHref}
					<a
						class={[
							'inline-flex flex-col items-center justify-center gap-1 rounded-lg px-2 py-2 text-[11px] font-medium transition',
							active
								? 'bg-primary/12 text-primary'
								: 'text-muted-foreground hover:bg-muted hover:text-foreground'
						]}
						href={resolvedHref}
						aria-current={active ? 'page' : undefined}
					>
						<Icon class="size-4" />
						<span>{item.label()}</span>
					</a>
				{/each}
			</div>
		</nav>
	</div>
{/if}

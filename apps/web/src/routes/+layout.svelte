<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { setContext } from 'svelte';
	import { createServices, SERVICES_KEY } from '$lib/services';
	import {
		WitnessStore,
		UserStore,
		NotificationStore,
		TriageStore,
		NavigationStore,
		FeedStore,
		ThemeStore,
		WITNESS_STORE_KEY,
		USER_STORE_KEY,
		NOTIFICATION_STORE_KEY,
		TRIAGE_STORE_KEY,
		NAVIGATION_STORE_KEY,
		FEED_STORE_KEY,
		THEME_STORE_KEY
	} from '$lib/stores';
	import { AppHeader, AppSidebar, TabBar, AddTabSheet } from '$lib/components/shell';

	// ---------------------------------------------------------------------------
	// Service & Store initialization (mock-first, swap to API later)
	// ---------------------------------------------------------------------------

	const services = createServices();
	const witnessStore = new WitnessStore(services.witness);
	const userStore = new UserStore(services.user);
	const notificationStore = new NotificationStore(services.notification);
	const triageStore = new TriageStore(services.triage);
	const navigationStore = new NavigationStore();
	const feedStore = new FeedStore();
	const themeStore = new ThemeStore();

	setContext(SERVICES_KEY, services);
	setContext(WITNESS_STORE_KEY, witnessStore);
	setContext(USER_STORE_KEY, userStore);
	setContext(NOTIFICATION_STORE_KEY, notificationStore);
	setContext(TRIAGE_STORE_KEY, triageStore);
	setContext(NAVIGATION_STORE_KEY, navigationStore);
	setContext(FEED_STORE_KEY, feedStore);
	setContext(THEME_STORE_KEY, themeStore);

	let { children } = $props();

	// ---------------------------------------------------------------------------
	// Boot — load user profile and notifications on app start
	// ---------------------------------------------------------------------------

	$effect(() => {
		userStore.loadCurrentUser();
		notificationStore.loadNotifications();
	});

	// ---------------------------------------------------------------------------
	// Route detection
	// ---------------------------------------------------------------------------

	const loginPath = `${base}/masuk`;
	const devPrefix = `${base}/dev`;

	const isLoginPage = $derived(page.url.pathname === loginPath);
	const isDevPage = $derived(
		page.url.pathname === devPrefix || page.url.pathname.startsWith(`${devPrefix}/`)
	);

	// ---------------------------------------------------------------------------
	// Detail-open awareness (used by child pages, no layout changes needed here)
	// ---------------------------------------------------------------------------
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if isLoginPage}
	<!-- Login: minimal centered layout -->
	<main class="mx-auto flex min-h-dvh w-full max-w-screen-md px-4 py-6">
		{@render children()}
	</main>
{:else if isDevPage}
	<!-- Dev gallery: pass-through (dev has its own layout with sidebar) -->
	<div class="min-h-dvh bg-background text-foreground">
		{@render children()}
	</div>
{:else}
	<!-- App shell: sidebar (fixed) + header + content -->
	<div class="min-h-dvh bg-background text-foreground quadrille-bg lg:pl-[68px]">
		<!-- Desktop left sidebar — fixed to left edge of viewport -->
		<AppSidebar />

		<div class="mx-auto flex min-h-dvh w-full max-w-screen-xl flex-col">
			<AppHeader />

			<main
				class="relative flex w-full min-w-0 flex-1 flex-col px-4 py-6 pb-24 lg:pb-8"
			>
				{@render children()}
			</main>
		</div>

		<!-- Mobile bottom tab bar (< lg) -->
		<TabBar />
		<AddTabSheet />
	</div>
{/if}

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
		PreferencesStore,
		CommunityStore,
		SignalStore,
		GroupStore,
		WITNESS_STORE_KEY,
		USER_STORE_KEY,
		NOTIFICATION_STORE_KEY,
		TRIAGE_STORE_KEY,
		NAVIGATION_STORE_KEY,
		FEED_STORE_KEY,
		THEME_STORE_KEY,
		PREFERENCES_STORE_KEY,
		COMMUNITY_STORE_KEY,
		SIGNAL_STORE_KEY,
		GROUP_STORE_KEY
	} from '$lib/stores';
	import { AppHeader, AppSidebar, TabBar, AddTabSheet } from '$lib/components/shell';
	import { TooltipProvider } from '$lib/components/ui/tooltip';

	// ---------------------------------------------------------------------------
	// Service & Store initialization (hot-path API-first with per-slice toggles)
	// ---------------------------------------------------------------------------

	const services = createServices();
	const witnessStore = new WitnessStore(services.witness);
	const userStore = new UserStore(services.user);
	const notificationStore = new NotificationStore(services.notification);
	const triageStore = new TriageStore(services.triage);
	const navigationStore = new NavigationStore();
	const feedStore = new FeedStore(services.feed);
	const themeStore = new ThemeStore();
	const preferencesStore = new PreferencesStore();
	const communityStore = new CommunityStore();
	const signalStore = new SignalStore(services.signal);
	const groupStore = new GroupStore(services.group);

	setContext(SERVICES_KEY, services);
	setContext(WITNESS_STORE_KEY, witnessStore);
	setContext(USER_STORE_KEY, userStore);
	setContext(NOTIFICATION_STORE_KEY, notificationStore);
	setContext(TRIAGE_STORE_KEY, triageStore);
	setContext(NAVIGATION_STORE_KEY, navigationStore);
	setContext(FEED_STORE_KEY, feedStore);
	setContext(THEME_STORE_KEY, themeStore);
	setContext(PREFERENCES_STORE_KEY, preferencesStore);
	setContext(COMMUNITY_STORE_KEY, communityStore);
	setContext(SIGNAL_STORE_KEY, signalStore);
	setContext(GROUP_STORE_KEY, groupStore);

	let { children } = $props();

	// ---------------------------------------------------------------------------
	// Boot — load user profile and notifications on app start
	// ---------------------------------------------------------------------------

	$effect(() => {
		userStore.loadCurrentUser();
		notificationStore.loadNotifications();
		communityStore.loadCommunityData();
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

<TooltipProvider delayDuration={300}>
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
</TooltipProvider>

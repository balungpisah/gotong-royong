<script lang="ts">
	import { base } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { getNotificationStore, getUserStore } from '$lib/stores';
	import BellRing from '@lucide/svelte/icons/bell-ring';
	import Activity from '@lucide/svelte/icons/activity';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import ThemeToggle from './theme-toggle.svelte';
	import Tip from '$lib/components/ui/tip.svelte';

	const notificationStore = getNotificationStore();
	const userStore = getUserStore();

	const headerPerson = $derived({
		user_id: userStore.profile?.user_id ?? 'self',
		name: userStore.displayName,
		avatar_url: userStore.profile?.avatar_url
	});
</script>

<header
	class="sticky top-0 z-20 border-b border-border/80 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/85"
>
	<div
		class="mx-auto flex w-full max-w-screen-md items-center justify-between gap-3 px-4 py-3 md:max-w-screen-xl md:px-6"
	>
		<!-- Brand + Detak Komunitas -->
		<div class="flex items-center gap-2.5">
			<a
				href="{base}/"
				class="text-sm font-extrabold tracking-wide text-foreground uppercase"
			>
				{m.shell_brand_name()}
			</a>
			<div class="flex items-center gap-1.5 text-primary/70">
				<Activity class="size-3.5" />
				<span class="text-xs font-medium tracking-wide">Detak Komunitas</span>
			</div>
		</div>

		<!-- Tagline (desktop only) -->
		<p class="hidden flex-1 text-center text-sm text-muted-foreground md:block">
			{m.shell_brand_tagline()}
		</p>

		<!-- Right actions -->
		<div class="flex items-center gap-2">
			<!-- Theme toggle -->
			<ThemeToggle />

			<!-- Notification bell -->
			<Tip text={m.shell_nav_notifikasi()}>
				<a
					href="{base}/notifikasi"
					class="relative inline-flex items-center justify-center rounded-full p-2 text-muted-foreground transition hover:bg-muted hover:text-foreground"
					aria-label={m.shell_nav_notifikasi()}
				>
					<BellRing class="size-5" />
					{#if notificationStore.hasUnread}
						<span
							class="absolute -top-0.5 -right-0.5 flex size-5 items-center justify-center rounded-full bg-destructive text-xs font-bold text-white"
						>
							{notificationStore.unreadCount > 9 ? '9+' : notificationStore.unreadCount}
						</span>
					{/if}
				</a>
			</Tip>

			<!-- User avatar -->
			<Tip text={m.shell_nav_profil()}>
			<a
				href="{base}/profil"
				class="rounded-full ring-2 ring-transparent transition hover:ring-primary/30"
				aria-label={m.shell_nav_profil()}
			>
				<TandangAvatar person={headerPerson} size="sm" isSelf />
			</a>
			</Tip>
		</div>
	</div>
</header>

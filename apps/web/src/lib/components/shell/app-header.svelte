<script lang="ts">
	import { base } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { getNotificationStore, getUserStore } from '$lib/stores';
	import BellRing from '@lucide/svelte/icons/bell-ring';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';

	const notificationStore = getNotificationStore();
	const userStore = getUserStore();

	const initials = $derived(
		userStore.displayName
			.split(' ')
			.map((n) => n[0])
			.join('')
			.slice(0, 2)
			.toUpperCase()
	);
</script>

<header
	class="sticky top-0 z-20 border-b border-border/80 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/85"
>
	<div
		class="mx-auto flex w-full max-w-screen-md items-center justify-between gap-3 px-4 py-3 md:max-w-screen-xl md:px-6"
	>
		<!-- Brand -->
		<a
			href="{base}/"
			class="text-sm font-extrabold tracking-wide text-foreground uppercase"
		>
			{m.shell_brand_name()}
		</a>

		<!-- Tagline (desktop only) -->
		<p class="hidden flex-1 text-center text-sm text-muted-foreground md:block">
			{m.shell_brand_tagline()}
		</p>

		<!-- Right actions -->
		<div class="flex items-center gap-2">
			<!-- Notification bell -->
			<a
				href="{base}/notifikasi"
				class="relative inline-flex items-center justify-center rounded-full p-2 text-muted-foreground transition hover:bg-muted hover:text-foreground"
				aria-label={m.shell_nav_notifikasi()}
			>
				<BellRing class="size-5" />
				{#if notificationStore.hasUnread}
					<span
						class="absolute -top-0.5 -right-0.5 flex size-5 items-center justify-center rounded-full bg-destructive text-[10px] font-bold text-white"
					>
						{notificationStore.unreadCount > 9 ? '9+' : notificationStore.unreadCount}
					</span>
				{/if}
			</a>

			<!-- User avatar -->
			<a
				href="{base}/profil"
				class="rounded-full ring-2 ring-transparent transition hover:ring-primary/30"
				aria-label={m.shell_nav_profil()}
			>
				<Avatar class="size-8">
					{#if userStore.profile?.avatar_url}
						<AvatarImage src={userStore.profile.avatar_url} alt={userStore.displayName} />
					{/if}
					<AvatarFallback class="bg-muted text-xs font-semibold text-muted-foreground">
						{initials}
					</AvatarFallback>
				</Avatar>
			</a>
		</div>
	</div>
</header>

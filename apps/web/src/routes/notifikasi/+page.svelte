<script lang="ts">
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { getNotificationStore } from '$lib/stores';
	import type { NotificationType } from '$lib/types';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { motion } from '@humanspeak/svelte-motion';
	import { timeAgo } from '$lib/utils/time';

	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import Search from '@lucide/svelte/icons/search';
	import Pencil from '@lucide/svelte/icons/pencil';
	import AtSign from '@lucide/svelte/icons/at-sign';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Info from '@lucide/svelte/icons/info';
	import Bell from '@lucide/svelte/icons/bell';
	import CheckCheck from '@lucide/svelte/icons/check-check';
	import Inbox from '@lucide/svelte/icons/inbox';
	import Loader2 from '@lucide/svelte/icons/loader-2';

	const store = getNotificationStore();

	// ---------------------------------------------------------------------------
	// Notification type → icon & style mapping
	// ---------------------------------------------------------------------------

	const typeConfig: Record<NotificationType, { icon: typeof RefreshCw; color: string; label: string }> = {
		phase_change: { icon: RefreshCw, color: 'text-blue-600 bg-blue-500/10', label: 'Fase' },
		vote_open: { icon: BarChart3, color: 'text-purple-600 bg-purple-500/10', label: 'Voting' },
		evidence_needed: { icon: Search, color: 'text-amber-600 bg-amber-500/10', label: 'Bukti' },
		diff_proposed: { icon: Pencil, color: 'text-cyan-600 bg-cyan-500/10', label: 'Perubahan' },
		mention: { icon: AtSign, color: 'text-green-600 bg-green-500/10', label: 'Sebutan' },
		role_assigned: { icon: ShieldCheck, color: 'text-indigo-600 bg-indigo-500/10', label: 'Peran' },
		system: { icon: Info, color: 'text-muted-foreground bg-muted/30', label: 'Sistem' }
	};

	// ---------------------------------------------------------------------------
	// Grouped notifications
	// ---------------------------------------------------------------------------

	const unread = $derived(store.notifications.filter((n) => !n.read));
	const read = $derived(store.notifications.filter((n) => n.read));

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async function handleClick(notifId: string, witnessId?: string) {
		try {
			await store.markRead(notifId);
			if (witnessId) {
				goto(`${base}/saksi/${witnessId}`);
			}
		} catch {
			// store.error is already set by the store
		}
	}

	async function handleMarkAllRead() {
		try {
			await store.markAllRead();
		} catch {
			// store.error is already set by the store
		}
	}
</script>

<div class="mx-auto w-full max-w-2xl space-y-6">
	<!-- Header -->
	<motion.div
		class="flex items-center justify-between"
		initial={{ opacity: 0, y: -8 }}
		animate={{ opacity: 1, y: 0 }}
		transition={{ duration: 0.3 }}
	>
		<div class="flex items-center gap-3">
			<div class="flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
				<Bell class="size-5" />
			</div>
			<div>
				<h1 class="text-[var(--fs-h1)] font-bold text-foreground">{m.page_notifikasi_title()}</h1>
				<p class="text-[var(--fs-caption)] text-muted-foreground">{m.page_notifikasi_description()}</p>
			</div>
		</div>
		{#if store.hasUnread}
			<Button variant="outline" size="sm" onclick={handleMarkAllRead} class="gap-1.5">
				<CheckCheck class="size-3.5" />
				Tandai semua dibaca
			</Button>
		{/if}
	</motion.div>

	<!-- Loading state -->
	{#if store.loading}
		<div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
			<Loader2 class="size-8 animate-spin" />
			<p class="mt-3 text-[var(--fs-small)]">Memuat notifikasi...</p>
		</div>

	<!-- Empty state -->
	{:else if store.notifications.length === 0}
		<motion.div
			class="flex flex-col items-center justify-center rounded-2xl border border-border/30 bg-muted/10 py-16"
			initial={{ opacity: 0, scale: 0.95 }}
			animate={{ opacity: 1, scale: 1 }}
			transition={{ duration: 0.3 }}
		>
			<div class="flex size-14 items-center justify-center rounded-full bg-muted/20">
				<Inbox class="size-7 text-muted-foreground" />
			</div>
			<p class="mt-4 text-[var(--fs-body)] font-medium text-foreground">Tidak ada notifikasi</p>
			<p class="mt-1 text-[var(--fs-caption)] text-muted-foreground">Semua sudah bersih!</p>
		</motion.div>

	<!-- Notification list -->
	{:else}
		<!-- Unread section -->
		{#if unread.length > 0}
			<div>
				<div class="flex items-center gap-2 px-1 pb-2">
					<span class="text-[var(--fs-small)] font-semibold text-foreground">Belum dibaca</span>
					<Badge variant="secondary" class="text-xs">{unread.length}</Badge>
				</div>
				<div class="space-y-2">
					{#each unread as notif, i (notif.notification_id)}
						{@const config = typeConfig[notif.type]}
						<motion.button
							class="notif-card notif-card--unread"
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.25, delay: i * 0.04 }}
							onclick={() => handleClick(notif.notification_id, notif.witness_id)}
						>
							<!-- Unread dot -->
							<div class="absolute left-2.5 top-1/2 size-2 -translate-y-1/2 rounded-full bg-primary"></div>

							<!-- Type icon -->
							<div class="flex size-9 shrink-0 items-center justify-center rounded-lg {config.color}">
								<config.icon class="size-4" />
							</div>

							<!-- Content -->
							<div class="min-w-0 flex-1 text-left">
								<div class="flex items-center gap-2">
									<span class="truncate text-[var(--fs-small)] font-semibold text-foreground">{notif.title}</span>
									<Badge variant="outline" class="shrink-0 text-xs">{config.label}</Badge>
								</div>
								<p class="mt-0.5 line-clamp-2 text-[var(--fs-caption)] leading-relaxed text-muted-foreground">
									{notif.body}
								</p>
								<p class="mt-1 text-xs text-muted-foreground/60">{timeAgo(notif.created_at)}</p>
							</div>
						</motion.button>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Separator between groups -->
		{#if unread.length > 0 && read.length > 0}
			<Separator class="opacity-50" />
		{/if}

		<!-- Read section -->
		{#if read.length > 0}
			<div>
				<div class="px-1 pb-2">
					<span class="text-[var(--fs-small)] font-semibold text-muted-foreground">Sudah dibaca</span>
				</div>
				<div class="space-y-2">
					{#each read as notif, i (notif.notification_id)}
						{@const config = typeConfig[notif.type]}
						<motion.button
							class="notif-card"
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							transition={{ duration: 0.25, delay: (unread.length + i) * 0.04 }}
							onclick={() => handleClick(notif.notification_id, notif.witness_id)}
						>
							<!-- Type icon -->
							<div class="flex size-9 shrink-0 items-center justify-center rounded-lg {config.color} opacity-60">
								<config.icon class="size-4" />
							</div>

							<!-- Content -->
							<div class="min-w-0 flex-1 text-left">
								<div class="flex items-center gap-2">
									<span class="truncate text-[var(--fs-small)] font-medium text-foreground/70">{notif.title}</span>
									<Badge variant="outline" class="shrink-0 text-xs opacity-60">{config.label}</Badge>
								</div>
								<p class="mt-0.5 line-clamp-2 text-[var(--fs-caption)] leading-relaxed text-muted-foreground/60">
									{notif.body}
								</p>
								<p class="mt-1 text-xs text-muted-foreground/40">{timeAgo(notif.created_at)}</p>
							</div>
						</motion.button>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>

<style>
	.notif-card {
		position: relative;
		display: flex;
		width: 100%;
		cursor: pointer;
		align-items: flex-start;
		gap: 0.75rem;
		border-radius: var(--r-lg);
		border: 1px solid oklch(from var(--c-batu) l c h / 0.2);
		background: oklch(from var(--c-susu) l c h / 0.3);
		padding: 0.875rem 1rem;
		text-align: left;
		transition: all 0.15s ease;
	}

	.notif-card:hover {
		border-color: oklch(from var(--c-batu) l c h / 0.4);
		background: oklch(from var(--c-susu) l c h / 0.6);
	}

	.notif-card--unread {
		padding-left: 1.75rem;
		border-color: oklch(from var(--c-biru) l c h / 0.2);
		background: oklch(from var(--c-biru) l c h / 0.04);
	}

	.notif-card--unread:hover {
		background: oklch(from var(--c-biru) l c h / 0.08);
	}
</style>

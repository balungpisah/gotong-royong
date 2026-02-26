<script lang="ts">
	import Activity from '@lucide/svelte/icons/activity';
	import Users from '@lucide/svelte/icons/users';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import Heart from '@lucide/svelte/icons/heart';
	import { motion } from '@humanspeak/svelte-motion';
	import { getCommunityStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';

	const store = getCommunityStore();

	// ---------------------------------------------------------------------------
	// Derived display values
	// ---------------------------------------------------------------------------

	const stats = $derived(store.stats);
	const participation = $derived(store.participation);
	const signals = $derived(store.signals);
	const activity = $derived(store.recentActivity);

	// Signal rows for display
	const signalRows = $derived(
		signals
			? [
					{ label: m.signal_vouch(), count: signals.vouch, color: 'bg-signal-vouch' },
					{ label: m.signal_skeptis(), count: signals.skeptis, color: 'bg-signal-skeptis' },
					{ label: m.signal_proof(), count: signals.proof_of_resolve, color: 'bg-signal-proof' },
					{ label: m.signal_dukung(), count: signals.dukung, color: 'bg-signal-dukung' },
					{ label: m.signal_perlu_dicek(), count: signals.perlu_dicek, color: 'bg-signal-dicek' }
				]
			: []
	);

	// Compute total for percentage bars
	const signalTotal = $derived(
		signalRows.reduce((sum, s) => sum + s.count, 0) || 1
	);

	// Activity icon type → color mapping
	const activityColors: Record<string, string> = {
		vouch: 'bg-signal-vouch/50',
		contribute: 'bg-signal-skeptis/50',
		verify: 'bg-signal-proof/50',
		resolve: 'bg-signal-dukung/50'
	};
</script>

<!--
	CommunityPulse — overview dashboard for the community context box.
	Shows aggregate stats, participation metrics, and health indicators.
	Wired to CommunityStore.
-->

{#if store.loading && !stats}
	<div class="flex h-full items-center justify-center">
		<p class="text-caption text-muted-foreground">{m.loading_community()}</p>
	</div>
{:else}
	<div class="flex h-full flex-col">
		<!-- Header -->
		<div class="flex items-center gap-3 border-b border-border/20 px-5 py-4">
			<div class="flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
				<Activity class="size-5" />
			</div>
			<div>
				<h2 class="text-body font-bold text-foreground">{m.komunitas_pulse_title()}</h2>
				<p class="text-caption text-muted-foreground">{m.komunitas_pulse_subtitle()}</p>
			</div>
		</div>

		<!-- Stats grid -->
		<div class="flex-1 overflow-y-auto px-5 py-4">
			{#if stats}
				<!-- Quick stats row -->
				<div class="grid grid-cols-2 gap-3">
					<motion.div
						class="stat-card"
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ duration: 0.3, delay: 0 }}
					>
						<div class="flex items-center gap-2 text-muted-foreground">
							<Users class="size-3.5" />
							<span class="text-caption">{m.pulse_stats_active()}</span>
						</div>
						<p class="mt-1 text-h2 font-bold text-foreground">{stats.active_witness_count}</p>
						<p class="text-caption text-berhasil">+{stats.active_witness_delta} {m.common_this_week()}</p>
					</motion.div>

					<motion.div
						class="stat-card"
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ duration: 0.3, delay: 0.05 }}
					>
						<div class="flex items-center gap-2 text-muted-foreground">
							<MessageCircle class="size-3.5" />
							<span class="text-caption">{m.komunitas_messages_today()}</span>
						</div>
						<p class="mt-1 text-h2 font-bold text-foreground">{stats.messages_today}</p>
						<p class="text-caption text-muted-foreground">{m.komunitas_conversations({ count: String(stats.conversations_today) })}</p>
					</motion.div>

					<motion.div
						class="stat-card"
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ duration: 0.3, delay: 0.1 }}
					>
						<div class="flex items-center gap-2 text-muted-foreground">
							<TrendingUp class="size-3.5" />
							<span class="text-caption">{m.komunitas_resolution_rate()}</span>
						</div>
						<p class="mt-1 text-h2 font-bold text-foreground">{stats.resolution_rate}%</p>
						<div class="mt-1.5 h-1.5 w-full rounded-full bg-muted/40">
							<div
								class="h-full rounded-full bg-berhasil transition-all duration-500"
								style="width: {stats.resolution_rate}%"
							></div>
						</div>
					</motion.div>

					<motion.div
						class="stat-card"
						initial={{ opacity: 0, y: 8 }}
						animate={{ opacity: 1, y: 0 }}
						transition={{ duration: 0.3, delay: 0.15 }}
					>
						<div class="flex items-center gap-2 text-muted-foreground">
							<Heart class="size-3.5" />
							<span class="text-caption">{m.komunitas_tandang_signals()}</span>
						</div>
						<p class="mt-1 text-h2 font-bold text-foreground">{stats.tandang_signals_this_week}</p>
						<p class="text-caption text-muted-foreground">{m.common_this_week()}</p>
					</motion.div>
				</div>
			{/if}

			<!-- Participation chart -->
			{#if participation.length > 0}
				<motion.div
					class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
					initial={{ opacity: 0, y: 12 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.35, delay: 0.2 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.komunitas_participation_title()}</h3>
					<!-- Mini bar chart — single bar per day, no inner div -->
					<div class="mt-3 flex items-end gap-1.5" style="height: 80px;">
						{#each participation as point, i}
							<motion.div
								class="flex-1 rounded-t-sm bg-primary/60"
								initial={{ height: 0 }}
								animate={{ height: `${point.value}%` }}
								transition={{ duration: 0.4, delay: 0.25 + i * 0.05 }}
							/>
						{/each}
					</div>
					<div class="mt-1.5 flex justify-between text-small text-muted-foreground">
						{#each participation as point}
							<span>{point.day}</span>
						{/each}
					</div>
				</motion.div>
			{/if}

			<!-- Recent activity feed -->
			{#if activity.length > 0}
				<motion.div
					class="mt-4"
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					transition={{ duration: 0.3, delay: 0.4 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.komunitas_activity_title()}</h3>
					<div class="mt-2 space-y-2">
						{#each activity as item}
							<div class="flex items-start gap-2.5 rounded-lg bg-muted/10 px-3 py-2">
								<div class="mt-0.5 size-2 shrink-0 rounded-full {activityColors[item.icon_type] ?? 'bg-primary/50'}"></div>
								<div class="min-w-0 flex-1">
									<p class="text-caption leading-relaxed text-foreground/80">{item.text}</p>
									<p class="mt-0.5 text-small text-muted-foreground">{item.time_label} {m.time_ago_suffix()}</p>
								</div>
							</div>
						{/each}
					</div>
				</motion.div>
			{/if}

			<!-- Tandang signal summary -->
			{#if signalRows.length > 0}
				<motion.div
					class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
					initial={{ opacity: 0, y: 12 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.35, delay: 0.5 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.komunitas_signals_this_week()}</h3>
					<div class="mt-3 space-y-2">
						{#each signalRows as signal}
							<div class="flex items-center gap-2">
								<span class="w-24 text-caption text-muted-foreground">{signal.label}</span>
								<div class="h-2 flex-1 rounded-full bg-muted/30">
									<div
										class="h-full rounded-full {signal.color} transition-all duration-500"
										style="width: {Math.round((signal.count / signalTotal) * 100)}%"
									></div>
								</div>
								<span class="w-8 text-right text-caption font-medium text-foreground">{signal.count}</span>
							</div>
						{/each}
					</div>
				</motion.div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.stat-card {
		padding: 0.75rem;
		border-radius: var(--r-md);
		border: 1px solid oklch(from var(--c-batu) l c h / 0.3);
		background: oklch(from var(--c-susu) l c h / 0.5);
	}
</style>

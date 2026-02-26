<script lang="ts">
	import Shield from '@lucide/svelte/icons/shield';
	import Star from '@lucide/svelte/icons/star';
	import Award from '@lucide/svelte/icons/award';
	import Eye from '@lucide/svelte/icons/eye';
	import HandHeart from '@lucide/svelte/icons/hand-heart';
	import Crown from '@lucide/svelte/icons/crown';
	import { motion } from '@humanspeak/svelte-motion';
	import { getUserStore } from '$lib/stores';
	import { timeAgo } from '$lib/utils/time';
	import { m } from '$lib/paraglide/messages';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';

	interface Props {
		userId?: string | null;
	}

	let { userId = null }: Props = $props();

	const userStore = getUserStore();

	// Load a different profile when userId is provided
	$effect(() => {
		if (userId) {
			userStore.loadProfile(userId);
		}
	});

	// ---------------------------------------------------------------------------
	// Derived values from store
	// ---------------------------------------------------------------------------

	const profile = $derived(userStore.profile);

	const joinedYear = $derived(
		profile?.joined_at ? new Date(profile.joined_at).getFullYear().toString() : '—'
	);

	const locationLabel = $derived(profile?.location ?? '');

	// Tier 0-4 → label mapping
	const tierLabels = $derived({
		0: m.tier_0_label(),
		1: m.tier_1_label(),
		2: m.tier_2_label(),
		3: m.tier_3_label(),
		4: m.tier_4_label()
	} as Record<number, string>);

	const tierLabel = $derived(tierLabels[profile?.tier ?? 0] ?? m.tier_0_label());
	const tierLevel = $derived(m.tier_level({ level: String((profile?.tier ?? 0) + 1) }));

	// Role badge
	const roleBadge = $derived(
		profile?.role === 'admin'
			? { label: m.profil_role_admin(), color: 'text-bahaya bg-bahaya-lembut' }
			: profile?.role === 'moderator'
				? { label: m.profil_role_moderator(), color: 'text-signal-proof bg-signal-proof/10' }
				: null
	);

	// Stats
	const stats = $derived(profile?.stats);
	const tandang = $derived(profile?.tandang_signals);
	const octalysis = $derived(profile?.octalysis);
	const activity = $derived(profile?.recent_activity ?? []);

	// Dynamic max for tandang signal bars
	const maxSignal = $derived(
		tandang
			? Math.max(tandang.vouch, tandang.dukung, tandang.proof_of_resolve, tandang.skeptis, 1)
			: 1
	);

	// Tandang signals for display
	const tandangRows = $derived(
		tandang
			? [
					{
						label: m.signal_vouch(),
						received: tandang.vouch,
						icon: HandHeart,
						color: 'text-signal-vouch bg-signal-vouch/10'
					},
					{
						label: m.signal_dukung(),
						received: tandang.dukung,
						icon: Star,
						color: 'text-signal-dukung bg-signal-dukung/10'
					},
					{
						label: m.signal_proof(),
						received: tandang.proof_of_resolve,
						icon: Award,
						color: 'text-signal-proof bg-signal-proof/10'
					},
					{
						label: m.signal_skeptis(),
						received: tandang.skeptis,
						icon: Eye,
						color: 'text-signal-skeptis bg-signal-skeptis/10'
					}
				]
			: []
	);

	// Octalysis drives for display
	const octalysisRows = $derived(
		octalysis
			? [
					{ core: 'Epic Meaning', score: octalysis.epic_meaning },
					{ core: 'Accomplishment', score: octalysis.accomplishment },
					{ core: 'Empowerment', score: octalysis.empowerment },
					{ core: 'Social Influence', score: octalysis.social_influence },
					{ core: 'Unpredictability', score: octalysis.unpredictability }
				]
			: []
	);
</script>

<!--
	SelfProfile — person profile panel for the context box.
	Shows user identity, contribution stats, tandang reputation,
	and Octalysis engagement metrics. Wired to UserStore.
-->

{#if !profile}
	<div class="flex h-full items-center justify-center">
		<p class="text-caption text-muted-foreground">{m.loading_profile()}</p>
	</div>
{:else}
	<div class="flex h-full flex-col">
		<!-- Profile header -->
		<div class="border-b border-border/20 px-5 py-5">
			<motion.div
				class="flex items-center gap-4"
				initial={{ opacity: 0, x: -8 }}
				animate={{ opacity: 1, x: 0 }}
				transition={{ duration: 0.3 }}
			>
				<!-- Avatar -->
				<div class="relative">
					<TandangAvatar
						person={{
							user_id: profile.user_id,
							name: profile.name,
							avatar_url: profile.avatar_url
						}}
						size="lg"
						isSelf
					/>
					<!-- Online indicator -->
					<div
						class="absolute bottom-0 right-0 size-3.5 rounded-full border-2 border-card bg-online"
					></div>
				</div>
				<div class="min-w-0 flex-1">
					<h2 class="truncate text-body font-bold text-foreground">{profile.name}</h2>
					<p class="text-caption text-muted-foreground">
						{#if locationLabel}{locationLabel} ·
						{/if}{m.profil_member_since_active({ year: joinedYear })}
					</p>
				</div>
			</motion.div>

			<!-- Role badges -->
			<motion.div
				class="mt-3 flex flex-wrap gap-1.5"
				initial={{ opacity: 0 }}
				animate={{ opacity: 1 }}
				transition={{ duration: 0.3, delay: 0.1 }}
			>
				<span
					class="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2.5 py-0.5 text-caption font-medium text-primary"
				>
					<Shield class="size-3" />
					{tierLabel}
				</span>
				<span
					class="inline-flex items-center gap-1 rounded-full bg-waspada-lembut px-2.5 py-0.5 text-caption font-medium text-waspada"
				>
					<Star class="size-3" />
					{tierLevel}
				</span>
				{#if roleBadge}
					<span
						class="inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-caption font-medium {roleBadge.color}"
					>
						<Crown class="size-3" />
						{roleBadge.label}
					</span>
				{/if}
			</motion.div>
		</div>

		<!-- Profile content -->
		<div class="flex-1 overflow-y-auto px-5 py-4">
			<!-- Contribution stats -->
			{#if stats}
				<motion.div
					initial={{ opacity: 0, y: 8 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.3, delay: 0.15 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.profil_stats_title()}</h3>
					<div class="mt-2 grid grid-cols-3 gap-2">
						<div class="rounded-lg bg-muted/20 p-2.5 text-center">
							<p class="text-h2 font-bold text-foreground">{stats.evidence_submitted}</p>
							<p class="text-small text-muted-foreground">{m.profil_stat_tandang()}</p>
						</div>
						<div class="rounded-lg bg-muted/20 p-2.5 text-center">
							<p class="text-h2 font-bold text-foreground">{stats.witnesses_participated}</p>
							<p class="text-small text-muted-foreground">{m.profil_stat_saksi()}</p>
						</div>
						<div class="rounded-lg bg-muted/20 p-2.5 text-center">
							<p class="text-h2 font-bold text-foreground">{stats.resolutions_completed}</p>
							<p class="text-small text-muted-foreground">{m.profil_stat_resolusi()}</p>
						</div>
					</div>
				</motion.div>
			{/if}

			<!-- Tandang reputation -->
			{#if tandangRows.length > 0}
				<motion.div
					class="mt-4"
					initial={{ opacity: 0, y: 8 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.3, delay: 0.2 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.profil_tandang_title()}</h3>
					<p class="mt-0.5 text-caption text-muted-foreground">{m.profil_tandang_subtitle()}</p>
					<div class="mt-3 space-y-2.5">
						{#each tandangRows as signal}
							<div class="flex items-center gap-3">
								<div class="flex size-7 items-center justify-center rounded-md {signal.color}">
									<signal.icon class="size-3.5" />
								</div>
								<div class="min-w-0 flex-1">
									<div class="flex items-center justify-between">
										<span class="text-caption font-medium text-foreground">{signal.label}</span>
										<span class="text-caption font-bold text-foreground">{signal.received}</span>
									</div>
									<div class="mt-1 h-1.5 w-full rounded-full bg-muted/30">
										<div
											class="h-full rounded-full bg-primary/50 transition-all duration-500"
											style="width: {Math.min((signal.received / maxSignal) * 100, 100)}%"
										></div>
									</div>
								</div>
							</div>
						{/each}
					</div>
				</motion.div>
			{/if}

			<!-- Octalysis engagement -->
			{#if octalysisRows.length > 0}
				<motion.div
					class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
					initial={{ opacity: 0, y: 12 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.35, delay: 0.3 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.profil_engagement_title()}</h3>
					<p class="mt-0.5 text-caption text-muted-foreground">{m.profil_octalysis_subtitle()}</p>
					<div class="mt-3 space-y-2">
						{#each octalysisRows as drive}
							<div class="flex items-center gap-2">
								<span class="w-28 text-caption text-muted-foreground">{drive.core}</span>
								<div class="h-1.5 flex-1 rounded-full bg-muted/30">
									<div
										class="h-full rounded-full bg-primary transition-all duration-500"
										style="width: {drive.score}%; opacity: {0.4 + (drive.score / 100) * 0.6}"
									></div>
								</div>
								<span class="w-6 text-right text-small font-medium text-foreground"
									>{drive.score}</span
								>
							</div>
						{/each}
					</div>
				</motion.div>
			{/if}

			<!-- Recent activity -->
			{#if activity.length > 0}
				<motion.div
					class="mt-4"
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					transition={{ duration: 0.3, delay: 0.4 }}
				>
					<h3 class="text-small font-semibold text-foreground">{m.pulse_recent_activity()}</h3>
					<div class="mt-2 space-y-1.5">
						{#each activity as item}
							<div class="flex items-start gap-2 rounded-lg px-2 py-1.5">
								<div class="mt-1.5 size-1.5 shrink-0 rounded-full bg-primary/40"></div>
								<div>
									<p class="text-caption leading-relaxed text-foreground/80">{item.text}</p>
									<p class="text-small text-muted-foreground">{timeAgo(item.timestamp)}</p>
								</div>
							</div>
						{/each}
					</div>
				</motion.div>
			{/if}
		</div>
	</div>
{/if}

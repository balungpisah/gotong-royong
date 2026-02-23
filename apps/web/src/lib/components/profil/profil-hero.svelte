<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { TandangProfile } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';

	interface Props {
		profile: TandangProfile;
		isSelf?: boolean;
		size?: 'compact' | 'full';
	}

	const { profile, isSelf = false, size = 'full' }: Props = $props();

	const isCompact = $derived(size === 'compact');

	const initials = $derived(() => {
		const words = profile.name.trim().split(/\s+/);
		return words
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('');
	});

	const joinedYear = $derived(new Date(profile.joined_at).getFullYear());

	const avatarOpacity = $derived(() => {
		const now = Date.now();
		const lastActive = new Date(profile.last_active_at).getTime();
		const daysDiff = (now - lastActive) / (1000 * 60 * 60 * 24);
		if (daysDiff <= 7) return 1;
		if (daysDiff <= 30) return 0.9;
		return 0.7;
	});

	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};

	const tierColor = $derived(tierColors[profile.tier.level] ?? '#9E9E9E');

	const showFlameBadge = $derived(profile.consistency.streak_days > 7);

	const showRoleBadge = $derived(
		isSelf
	);
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.3 }}
>
	<div class="flex items-start gap-4">
		<!-- Avatar -->
		<div class="relative shrink-0" style="opacity: {avatarOpacity()}">
			<TandangAvatar
				person={{ user_id: profile.user_id, name: profile.name, avatar_url: profile.avatar_url, tier: profile.tier.level }}
				size={isCompact ? 'lg' : 'xl'}
				isSelf={isSelf}
			/>
			<!-- Online indicator -->
			<span class="absolute bottom-0.5 right-0.5 size-3 rounded-full bg-online ring-2 ring-background"></span>
		</div>

		<!-- Info -->
		<div class="flex-1 min-w-0">
			<h2 class="{isCompact ? 'text-base' : 'text-xl'} font-bold text-foreground truncate">{profile.name}</h2>
			<p class="text-caption text-muted-foreground mt-0.5">
				{#if profile.location}{profile.location} · {/if}{#if isCompact && profile.community_name}{profile.community_name} · {/if}{m.profil_member_since({ year: String(joinedYear) })}
			</p>

			<!-- Badge row -->
			<div class="mt-2 flex flex-wrap items-center gap-1.5">
				<!-- Tier pip badge -->
				<span
					class="inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-caption font-medium"
					style="background-color: color-mix(in srgb, {tierColor} 10%, transparent); color: {tierColor};"
				>
					{profile.tier.pips}
					{profile.tier.name}
				</span>

				<!-- Percentile badge (compact mode) -->
				{#if isCompact && profile.tier.percentile > 0}
					<span class="inline-flex items-center rounded-full bg-primary/10 px-2.5 py-0.5 text-caption font-medium text-primary">
						{m.profil_percentile({ pct: String(profile.tier.percentile) })}
					</span>
				{/if}

				<!-- Flame badge -->
				{#if showFlameBadge}
					<span class="inline-flex items-center gap-0.5 rounded-full bg-peringatan/10 px-2.5 py-0.5 text-caption font-medium text-peringatan">
						🔥 {profile.consistency.streak_days}h
					</span>
				{/if}
			</div>
		</div>
	</div>
</motion.div>

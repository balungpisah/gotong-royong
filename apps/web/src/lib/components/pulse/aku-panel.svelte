<script lang="ts">
	import { getUserStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import AkuSkyPortrait from './aku-sky-portrait.svelte';
	import AkuMissionControl from './aku-mission-control.svelte';
	import {
		SkillsSection,
		VouchNetwork,
		ActivityTimeline,
		DecayWarnings,
		DukungHistory
	} from '$lib/components/profil';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import PanelPinnedCard from './panel-pinned-card.svelte';

	interface Props {
		userId?: string | null;
	}

	let { userId = null }: Props = $props();

	const userStore = getUserStore();

	$effect(() => {
		if (userId) {
			userStore.loadTandangProfile(userId);
		} else {
			userStore.loadCurrentTandangProfile();
		}
	});

	const profile = $derived(userStore.tandangProfile);

	// Header identity helpers
	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};
	const tierColor = $derived(profile ? (tierColors[profile.tier.level] ?? '#9E9E9E') : '#9E9E9E');
	const joinedYear = $derived(profile ? new Date(profile.joined_at).getFullYear() : '');
	const showFlameBadge = $derived(profile ? profile.consistency.streak_days > 7 : false);
</script>

{#if userStore.tandangLoading && !profile}
	<div class="flex h-full items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-muted-foreground">
			<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
			<p class="text-small">{m.loading_tandang_profile()}</p>
		</div>
	</div>
{:else if profile}
	<div class="flex h-full flex-col overflow-hidden">
		<!-- Fixed header — PanelPinnedCard shared two-column template -->
		<PanelPinnedCard>
			{#snippet left()}
				<div class="flex items-start gap-2.5">
					<div class="relative shrink-0">
						<TandangAvatar
							person={{
								user_id: profile.user_id,
								name: profile.name,
								avatar_url: profile.avatar_url,
								tier: profile.tier.level
							}}
							size="sm"
							isSelf={!userId}
						/>
						<span
							class="absolute bottom-0 right-0 size-2 rounded-full bg-online ring-1 ring-background"
						></span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="truncate text-body font-bold leading-tight text-foreground">{profile.name}</p>
						<p class="truncate text-caption text-muted-foreground">
							{#if profile.location}{profile.location} ·
							{/if}{m.profil_member_since({ year: String(joinedYear) })}
						</p>
						<div class="mt-1 flex flex-nowrap items-center gap-1">
							<span
								class="inline-flex items-center gap-0.5 rounded-full border px-2 py-0.5 text-small leading-tight font-medium"
								style="border-color: color-mix(in srgb, {tierColor} 30%, transparent); background-color: color-mix(in srgb, {tierColor} 10%, transparent); color: {tierColor};"
								>{profile.tier.pips} {profile.tier.name}</span
							>
							{#if showFlameBadge}
								<span
									class="inline-flex items-center gap-0.5 rounded-full border border-peringatan/30 bg-peringatan/10 px-2 py-0.5 text-small leading-tight font-medium text-peringatan"
								>
									🔥 {profile.consistency.streak_days}h
								</span>
							{/if}
						</div>
					</div>
				</div>
			{/snippet}
			{#snippet right()}
				<div class="icj-col">
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-i)">I</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									profile.scores.integrity.value * 100
								)}%; background: var(--c-tandang-i)"
							></div>
						</div>
						<span class="icj-value">{Math.round(profile.scores.integrity.value * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-c)">C</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									profile.scores.competence.aggregate * 100
								)}%; background: var(--c-tandang-c)"
							></div>
						</div>
						<span class="icj-value">{Math.round(profile.scores.competence.aggregate * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-j)">J</span>
						<div class="icj-bar-track">
							<div
								class="icj-bar-fill"
								style="width: {Math.round(
									profile.scores.judgment.value * 100
								)}%; background: var(--c-tandang-j)"
							></div>
						</div>
						<span class="icj-value">{Math.round(profile.scores.judgment.value * 100)}</span>
					</div>
				</div>
			{/snippet}
		</PanelPinnedCard>

		<!-- Scrollable content -->
		<div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2">
			<!-- Sky portrait — emotional state hero -->
			<AkuSkyPortrait {profile} />

			<!-- Decay warnings — above fold for loss aversion -->
			{#if profile.decay_warnings.length > 0}
				<DecayWarnings warnings={profile.decay_warnings} />
			{/if}

			<!-- Mission control — stats, impact, budget -->
			<AkuMissionControl {profile} />

			<!-- Skills -->
			<SkillsSection skills={profile.skills} />

			<!-- Vouch Network -->
			<VouchNetwork
				vouchedBy={profile.vouched_by}
				vouchingFor={profile.vouching_for}
				budget={profile.vouch_budget}
			/>

			<!-- Full-width sections below the grid -->
			<DukungHistory
				given={profile.dukung_given}
				received={profile.dukung_received}
				successRate={profile.scores.judgment.dukung_success_rate}
			/>

			<ActivityTimeline items={profile.timeline} />
		</div>
	</div>
{:else if userStore.tandangError}
	<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
		<p class="text-small text-red-500">{userStore.tandangError}</p>
	</div>
{/if}

<style>
	/* Right col content — rendered inside PanelPinnedCard's right slot */
	.icj-col {
		flex: 0 0 calc(40% - 1rem);
		min-width: 0;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 0.45rem;
	}

	.icj-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	/* text-xs = 0.75rem, matches phase row labels in tandang */
	.icj-label {
		font-size: 0.75rem;
		font-weight: 800;
		letter-spacing: 0.05em;
		width: 0.75rem;
		flex-shrink: 0;
	}

	.icj-bar-track {
		flex: 1;
		height: 5px;
		border-radius: 99px;
		background: color-mix(in srgb, var(--color-muted) 40%, transparent);
		overflow: hidden;
	}

	.icj-bar-fill {
		height: 100%;
		border-radius: 99px;
		transition: width 500ms ease;
	}

	/* text-caption = 11px / 16px, matches phase count in tandang */
	.icj-value {
		font-size: 11px;
		line-height: 16px;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: var(--color-muted-foreground);
		width: 1.5rem;
		text-align: right;
		flex-shrink: 0;
	}
</style>

<script lang="ts">
	import { page } from '$app/state';
	import { getUserStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import {
		ProfilHero,
		IcjRadar,
		ScoreCards,
		SkillsSection,
		VouchNetwork,
		ActivityTimeline,
		ImpactSummary,
		ConsistencyDetail,
		DukungHistory,
		DecayWarnings
	} from '$lib/components/profil';

	const userStore = getUserStore();

	$effect(() => {
		const userId = page.params.user_id?.trim();
		if (userId) {
			userStore.loadTandangProfile(userId);
		}
	});

	const profile = $derived(userStore.tandangProfile);
</script>

{#if userStore.tandangLoading && !profile}
	<div class="flex h-64 items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-muted-foreground">
			<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
			<p class="text-xs">{m.loading_profile()}</p>
		</div>
	</div>
{:else if profile}
	<div class="mx-auto w-full max-w-3xl py-6">
		<div class="page-header rounded-xl border border-border/30 px-4 py-4 mb-6">
			<ProfilHero {profile} isSelf={false} />
		</div>

		<div class="space-y-6 px-4">
			<IcjRadar scores={profile.scores} isSelf={false} />
			<ScoreCards scores={profile.scores} />

			{#if profile.decay_warnings.length > 0}
				<DecayWarnings warnings={profile.decay_warnings} />
			{/if}

			<SkillsSection skills={profile.skills} />
			<ImpactSummary impact={profile.impact} />
			<ConsistencyDetail consistency={profile.consistency} genesis={profile.genesis} />
			<VouchNetwork
				vouchedBy={profile.vouched_by}
				vouchingFor={profile.vouching_for}
				budget={profile.vouch_budget}
			/>
			<DukungHistory
				given={profile.dukung_given}
				received={profile.dukung_received}
				successRate={profile.scores.judgment.dukung_success_rate}
			/>
			<ActivityTimeline items={profile.timeline} />
		</div>
	</div>
{:else if userStore.tandangError}
	<div class="flex h-64 flex-col items-center justify-center gap-3 text-center">
		<p class="text-xs text-red-500">{userStore.tandangError}</p>
	</div>
{/if}

<style>
	.page-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
	}
</style>

<script lang="ts">
	import { getUserStore } from '$lib/stores';
	import { m } from '$lib/paraglide/messages';
	import {
		ProfilHero,
		IcjRings,
		SkillsSection,
		VouchNetwork,
		ActivityTimeline,
		DecayWarnings,
		DukungHistory
	} from '$lib/components/profil';
	import Zap from '@lucide/svelte/icons/zap';
	import Flame from '@lucide/svelte/icons/flame';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import Target from '@lucide/svelte/icons/target';
	import HeartHandshake from '@lucide/svelte/icons/heart-handshake';
	import Award from '@lucide/svelte/icons/award';

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

	const budgetColor = $derived(() => {
		if (!profile) return 'bg-berhasil';
		const { remaining, max_vouches } = profile.vouch_budget;
		if (remaining === 0) return 'bg-bahaya';
		if (remaining / max_vouches < 0.3) return 'bg-waspada';
		return 'bg-berhasil';
	});

	const genesisPct = $derived(() => {
		if (!profile || profile.genesis.threshold === 0) return 0;
		return Math.min(Math.round((profile.genesis.meaningful_interactions_this_month / profile.genesis.threshold) * 100), 100);
	});
</script>

{#if userStore.tandangLoading && !profile}
	<div class="flex h-full items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-muted-foreground">
			<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
			<p class="text-xs">{m.loading_tandang_profile()}</p>
		</div>
	</div>
{:else if profile}
	<div class="flex h-full flex-col overflow-hidden">
		<!-- Fixed header -->
		<div class="panel-header shrink-0 border-b border-border/60">
			<div class="px-4 py-3">
				<ProfilHero {profile} isSelf={!userId} size="compact" />
			</div>
		</div>

		<!-- Scrollable content -->
		<div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2">
			<!-- ICJ Rings — full width hero above grid -->
			<div class="aku-card">
				<IcjRings scores={profile.scores} />
			</div>

			<!-- Decay warnings — above fold for loss aversion -->
			{#if profile.decay_warnings.length > 0}
				<DecayWarnings warnings={profile.decay_warnings} />
			{/if}

			<!-- 2-column grid -->
			<div class="aku-grid">
				<!-- Stats card -->
				<div class="aku-card">
					<div class="grid grid-cols-3 gap-1">
						<div class="stat-cell">
							<Zap class="size-3.5 text-waspada" />
							<span class="text-xs font-bold text-foreground">{profile.consistency.multiplier.toFixed(2)}×</span>
							<span class="stat-lbl">Multiplier</span>
						</div>
						<div class="stat-cell">
							<Flame class="size-3.5 text-peringatan" />
							<span class="text-xs font-bold text-foreground">{profile.consistency.streak_weeks}w</span>
							<span class="stat-lbl">{m.profil_streak_label()}</span>
						</div>
						<div class="stat-cell">
							<BarChart3 class="size-3.5 text-primary" />
							<span class="text-xs font-bold text-foreground">{Math.round(profile.consistency.quality_avg * 100)}%</span>
							<span class="stat-lbl">Kualitas</span>
						</div>
						<div class="stat-cell">
							<Target class="size-3.5 text-berhasil" />
							<span class="text-xs font-bold text-foreground">{profile.impact.witnesses_resolved}</span>
							<span class="stat-lbl">Selesai</span>
						</div>
						<div class="stat-cell">
							<HeartHandshake class="size-3.5 text-signal-proof" />
							<span class="text-xs font-bold text-foreground">{profile.impact.people_helped}</span>
							<span class="stat-lbl">Dibantu</span>
						</div>
						<div class="stat-cell">
							<Award class="size-3.5 text-berhasil" />
							<span class="text-xs font-bold text-foreground">
								{profile.scores.judgment.dukung_success_rate !== null
									? Math.round(profile.scores.judgment.dukung_success_rate * 100) + '%'
									: '—'}
							</span>
							<span class="stat-lbl">Dukung</span>
						</div>
					</div>
				</div>

				<!-- Vouch budget + Genesis -->
				<div class="aku-card space-y-3">
					<div class="min-w-0">
						<div class="flex items-center justify-between gap-1">
							<span class="text-caption font-medium text-foreground truncate">{m.profil_vouch_budget_title()}</span>
							<span class="text-caption text-muted-foreground shrink-0">
								{profile.vouch_budget.remaining}/{profile.vouch_budget.max_vouches}
							</span>
						</div>
						<div class="mt-1.5 h-2 w-full rounded-full bg-muted/30">
							<div
								class="h-full rounded-full transition-all duration-500 {budgetColor()}"
								style="width: {(profile.vouch_budget.active_vouches / profile.vouch_budget.max_vouches) * 100}%"
							></div>
						</div>
					</div>
					<div class="min-w-0">
						<div class="flex items-center justify-between gap-1">
							<span class="text-caption font-medium text-foreground truncate">{m.profil_genesis_weight()}</span>
							<span class="text-caption text-muted-foreground shrink-0">
								{profile.genesis.meaningful_interactions_this_month}/{profile.genesis.threshold}
							</span>
						</div>
						<div class="mt-1.5 h-2 w-full rounded-full bg-muted/30">
							<div
								class="h-full rounded-full transition-all duration-500 {genesisPct() >= 100 ? 'bg-berhasil' : 'bg-primary/60'}"
								style="width: {genesisPct()}%"
							></div>
						</div>
						{#if genesisPct() >= 100}
							<p class="mt-1 text-caption text-berhasil">{m.profil_genesis_paused()}</p>
						{/if}
					</div>
				</div>

				<!-- Skills -->
				<SkillsSection skills={profile.skills} />

				<!-- Vouch Network -->
				<VouchNetwork
					vouchedBy={profile.vouched_by}
					vouchingFor={profile.vouching_for}
					budget={profile.vouch_budget}
				/>

			</div>

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
		<p class="text-xs text-red-500">{userStore.tandangError}</p>
	</div>
{/if}

<style>
	.panel-header {
		background: color-mix(in srgb, var(--color-foreground) 5%, var(--color-card));
	}

	.aku-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.5rem;
		align-items: start;
	}

	/* Prevent grid children from overflowing their column */
	.aku-grid > :global(*) {
		min-width: 0;
		overflow: hidden;
	}

	.aku-card {
		border-radius: 0.75rem;
		border: 1px solid color-mix(in srgb, var(--color-border) 30%, transparent);
		background: var(--color-card);
		padding: 0.625rem;
		overflow: hidden;
		min-width: 0;
	}

	.stat-cell {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.15rem;
		padding: 0.35rem 0;
	}

	.stat-lbl {
		font-size: 0.625rem;
		line-height: 1.1;
		color: var(--color-muted-foreground);
		text-align: center;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>

<script lang="ts">
	import type { TandangProfile } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import HeartHandshake from '@lucide/svelte/icons/heart-handshake';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import Award from '@lucide/svelte/icons/award';

	interface Props {
		profile: TandangProfile;
	}

	const { profile }: Props = $props();

	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};

	const tierColor = $derived(tierColors[profile.tier.level] ?? '#9E9E9E');

	const I = $derived(Math.round(profile.scores.integrity.value * 100));
	const C = $derived(Math.round(profile.scores.competence.aggregate * 100));
	const J = $derived(Math.round(profile.scores.judgment.value * 100));

	const budgetFillColor = $derived(
		profile.vouch_budget.remaining === 0
			? 'var(--color-bahaya)'
			: profile.vouch_budget.remaining / profile.vouch_budget.max_vouches < 0.3
				? 'var(--color-waspada)'
				: 'var(--color-berhasil)'
	);

	const genesisPct = $derived(
		profile.genesis.threshold === 0
			? 0
			: Math.min(
					Math.round(
						(profile.genesis.meaningful_interactions_this_month / profile.genesis.threshold) * 100
					),
					100
				)
	);

	const dukungRate = $derived(
		profile.scores.judgment.dukung_success_rate !== null
			? Math.round(profile.scores.judgment.dukung_success_rate * 100) + '%'
			: '—'
	);
</script>

<div class="space-y-2">
	<!-- ── ICJ core card ─────────────────────────── -->
	<div class="mc-card" style="--tier: {tierColor}">
		<div class="mc-icj">
			<div class="mc-row">
				<span class="mc-ltr" style="color: var(--c-tandang-i)">I</span>
				<div class="mc-track">
					<div
						class="mc-fill"
						style="width: {I}%; background: var(--c-tandang-i);
						       box-shadow: 0 0 9px color-mix(in srgb, var(--c-tandang-i) 55%, transparent)"
					></div>
				</div>
				<span class="mc-num" style="color: var(--c-tandang-i)">{I}</span>
			</div>
			<div class="mc-row">
				<span class="mc-ltr" style="color: var(--c-tandang-c)">C</span>
				<div class="mc-track">
					<div
						class="mc-fill"
						style="width: {C}%; background: var(--c-tandang-c);
						       box-shadow: 0 0 9px color-mix(in srgb, var(--c-tandang-c) 55%, transparent)"
					></div>
				</div>
				<span class="mc-num" style="color: var(--c-tandang-c)">{C}</span>
			</div>
			<div class="mc-row">
				<span class="mc-ltr" style="color: var(--c-tandang-j)">J</span>
				<div class="mc-track">
					<div
						class="mc-fill"
						style="width: {J}%; background: var(--c-tandang-j);
						       box-shadow: 0 0 9px color-mix(in srgb, var(--c-tandang-j) 55%, transparent)"
					></div>
				</div>
				<span class="mc-num" style="color: var(--c-tandang-j)">{J}</span>
			</div>
		</div>

		<div class="mc-sep"></div>

		<div class="mc-meta">
			<span
				><span class="mc-meta-val">{profile.consistency.multiplier.toFixed(2)}×</span> multiplikasi</span
			>
			<span class="mc-dot">·</span>
			<span
				><span class="mc-meta-val">{Math.round(profile.consistency.quality_avg * 100)}%</span> kualitas</span
			>
			<span class="mc-dot">·</span>
			<span><span class="mc-meta-val">{profile.consistency.streak_weeks}w</span> streak</span>
		</div>
	</div>

	<!-- ── Impact KPI trio ───────────────────────── -->
	<div class="mc-impact">
		<div class="mc-kpi">
			<HeartHandshake />
			<span class="mc-kpi-val">{profile.impact.people_helped}</span>
			<span class="mc-kpi-lbl">Dibantu</span>
		</div>
		<div class="mc-kpi-sep"></div>
		<div class="mc-kpi">
			<CheckCircle />
			<span class="mc-kpi-val">{profile.impact.witnesses_resolved}</span>
			<span class="mc-kpi-lbl">Selesai</span>
		</div>
		<div class="mc-kpi-sep"></div>
		<div class="mc-kpi">
			<Award />
			<span class="mc-kpi-val">{dukungRate}</span>
			<span class="mc-kpi-lbl">Dukung</span>
		</div>
	</div>

	<!-- ── Vouch budget + Genesis ────────────────── -->
	<div class="mc-card mc-card--sm" style="--tier: {tierColor}">
		<div class="mc-budget-row">
			<div class="mc-budget-head">
				<span class="mc-budget-name">{m.profil_vouch_budget_title()}</span>
				<span class="mc-budget-count"
					>{profile.vouch_budget.remaining}/{profile.vouch_budget.max_vouches}</span
				>
			</div>
			<div class="mc-budget-track">
				<div
					class="mc-budget-fill"
					style="width: {(profile.vouch_budget.active_vouches / profile.vouch_budget.max_vouches) *
						100}%;
					       background: {budgetFillColor}"
				></div>
			</div>
		</div>
		<div class="mc-budget-row">
			<div class="mc-budget-head">
				<span class="mc-budget-name">{m.profil_genesis_weight()}</span>
				<span class="mc-budget-count"
					>{profile.genesis.meaningful_interactions_this_month}/{profile.genesis.threshold}</span
				>
			</div>
			<div class="mc-budget-track">
				<div
					class="mc-budget-fill"
					style="width: {genesisPct}%; background: {genesisPct >= 100
						? 'var(--color-berhasil)'
						: 'color-mix(in srgb, var(--color-primary) 80%, transparent)'}"
				></div>
			</div>
			{#if genesisPct >= 100}
				<p class="mc-budget-ok">{m.profil_genesis_paused()}</p>
			{/if}
		</div>
	</div>
</div>

<style>
	/* ── Card shell ─────────────────────────────── */
	.mc-card {
		border-radius: 0.75rem;
		background: color-mix(in srgb, var(--color-foreground) 8%, var(--color-card));
		border: 1px solid color-mix(in srgb, var(--color-foreground) 11%, transparent);
		border-left: 3px solid var(--tier, var(--color-border));
		padding: 0.75rem;
		overflow: hidden;
	}

	.mc-card--sm {
		padding: 0.625rem 0.75rem;
	}

	/* ── ICJ score rows ─────────────────────────── */
	.mc-icj {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	.mc-row {
		display: flex;
		align-items: center;
		gap: 0.625rem;
	}

	.mc-ltr {
		font-size: 0.75rem;
		font-weight: 900;
		letter-spacing: 0.06em;
		width: 0.75rem;
		flex-shrink: 0;
	}

	.mc-track {
		flex: 1;
		height: 6px;
		border-radius: 99px;
		background: color-mix(in srgb, var(--color-foreground) 10%, transparent);
		overflow: hidden;
	}

	.mc-fill {
		height: 100%;
		border-radius: 99px;
		transition: width 600ms ease;
	}

	.mc-num {
		font-size: 1.05rem;
		line-height: 1;
		font-weight: 800;
		font-variant-numeric: tabular-nums;
		width: 2rem;
		text-align: right;
		flex-shrink: 0;
	}

	/* ── Separator + meta ───────────────────────── */
	.mc-sep {
		height: 1px;
		background: color-mix(in srgb, var(--color-foreground) 8%, transparent);
		margin: 0.6rem 0 0.5rem;
	}

	.mc-meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.3rem;
		font-size: 10.5px;
		color: var(--color-muted-foreground);
		line-height: 1.3;
	}

	.mc-meta-val {
		font-weight: 700;
		color: var(--color-foreground);
	}

	.mc-dot {
		color: color-mix(in srgb, var(--color-muted-foreground) 45%, transparent);
	}

	/* ── Impact KPI trio ────────────────────────── */
	.mc-impact {
		display: flex;
		border-radius: 0.75rem;
		background: color-mix(in srgb, var(--color-foreground) 8%, var(--color-card));
		border: 1px solid color-mix(in srgb, var(--color-foreground) 11%, transparent);
		overflow: hidden;
	}

	.mc-kpi {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 0.65rem 0.25rem;
		gap: 0.15rem;
	}

	.mc-kpi :global(svg) {
		width: 0.875rem;
		height: 0.875rem;
		color: var(--color-muted-foreground);
		margin-bottom: 0.1rem;
	}

	.mc-kpi-val {
		font-size: 1.2rem;
		font-weight: 800;
		line-height: 1;
		font-variant-numeric: tabular-nums;
		color: var(--color-foreground);
	}

	.mc-kpi-lbl {
		font-size: 9.5px;
		line-height: 1.2;
		letter-spacing: 0.03em;
		color: var(--color-muted-foreground);
	}

	.mc-kpi-sep {
		width: 1px;
		align-self: stretch;
		background: color-mix(in srgb, var(--color-foreground) 10%, transparent);
	}

	/* ── Budget / Genesis bars ──────────────────── */
	.mc-budget-row {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.mc-budget-row + .mc-budget-row {
		margin-top: 0.6rem;
	}

	.mc-budget-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.mc-budget-name {
		font-size: 10.5px;
		font-weight: 500;
		color: var(--color-foreground);
	}

	.mc-budget-count {
		font-size: 10.5px;
		color: var(--color-muted-foreground);
		font-variant-numeric: tabular-nums;
	}

	.mc-budget-track {
		height: 5px;
		border-radius: 99px;
		background: color-mix(in srgb, var(--color-foreground) 9%, transparent);
		overflow: hidden;
	}

	.mc-budget-fill {
		height: 100%;
		border-radius: 99px;
		transition: width 500ms ease;
	}

	.mc-budget-ok {
		font-size: 10px;
		color: var(--color-berhasil);
		margin-top: 0.2rem;
	}
</style>

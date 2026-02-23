<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { VouchRelation, VouchBudget, VouchType } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';

	interface Props {
		vouchedBy: VouchRelation[];
		vouchingFor: VouchRelation[];
		budget: VouchBudget;
	}

	const { vouchedBy, vouchingFor, budget }: Props = $props();

	let showAllVouchedBy = $state(false);
	let showAllVouchingFor = $state(false);

	const SHOW_LIMIT = 5;

	const visibleVouchedBy = $derived(
		showAllVouchedBy ? vouchedBy : vouchedBy.slice(0, SHOW_LIMIT)
	);
	const visibleVouchingFor = $derived(
		showAllVouchingFor ? vouchingFor : vouchingFor.slice(0, SHOW_LIMIT)
	);

	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};

	const vouchTypeLabels = $derived({
		positive: m.vouch_type_positive(),
		collective: m.vouch_type_collective(),
		skeptical: m.vouch_type_skeptical(),
		conditional: m.vouch_type_conditional(),
		mentorship: m.vouch_type_mentorship(),
		project_scoped: m.vouch_type_project()
	} as Record<VouchType, string>);

	const vouchTypeBadgeStyle: Record<VouchType, string> = {
		positive: 'bg-signal-vouch/10 text-signal-vouch',
		collective: 'bg-signal-proof/10 text-signal-proof',
		skeptical: 'bg-waspada/10 text-waspada',
		conditional: 'bg-muted/30 text-muted-foreground',
		mentorship: 'bg-signal-dukung/10 text-signal-dukung',
		project_scoped: 'bg-tandang/10 text-tandang'
	};

	function getInitials(name: string): string {
		return name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('');
	}

	function relativeDate(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		const days = Math.floor(diff / 86400000);
		if (days === 0) return m.time_today();
		if (days === 1) return m.time_yesterday();
		if (days < 30) return m.time_days_ago({ days: String(days) });
		const months = Math.floor(days / 30);
		if (months < 12) return m.time_months_ago({ months: String(months) });
		return m.time_years_ago({ years: String(Math.floor(months / 12)) });
	}

	const budgetPercent = $derived(
		budget.max_vouches > 0 ? (budget.active_vouches / budget.max_vouches) * 100 : 0
	);
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.2 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4 space-y-4">
		<h3 class="text-xs font-semibold text-foreground">{m.profil_vouch_network()}</h3>

		<!-- Vouch budget bar -->
		<div class="space-y-1">
			<div class="flex items-center justify-between text-caption text-muted-foreground">
				<span>{m.profil_vouch_active({ active: String(budget.active_vouches), max: String(budget.max_vouches) })}</span>
				<span>{m.profil_vouch_remaining({ count: String(budget.remaining) })}</span>
			</div>
			<div class="h-1.5 rounded-full bg-muted/30">
				<div
					class="h-full rounded-full bg-[var(--c-tandang-c,#00695C)] transition-all duration-500"
					style="width: {budgetPercent}%;"
				></div>
			</div>
		</div>

		<!-- Dijamin oleh -->
		<div>
			<p class="mb-2 text-caption font-semibold text-foreground/80">
				{m.profil_vouched_by()}
				<span class="ml-1 rounded-full bg-muted/20 px-1.5 py-0.5 text-caption font-normal text-muted-foreground">
					{vouchedBy.length}
				</span>
			</p>
			<div class="space-y-2">
				{#each visibleVouchedBy as v, i}
					<motion.div
						initial={{ opacity: 0, x: -4 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.2, delay: 0.03 * i }}
					>
						<div class="flex items-center gap-2">
							<!-- Avatar -->
							<TandangAvatar
								person={{ user_id: v.user_id, name: v.user_name, avatar_url: v.user_avatar_url, tier: v.user_tier }}
								size="sm"
								showTierDot
							/>
							<!-- Name + tier dot -->
							<div class="flex min-w-0 flex-1 flex-col">
								<span class="truncate text-caption font-medium text-foreground">
									{v.user_name}
								</span>
								<span class="text-caption text-muted-foreground">
									{relativeDate(v.created_at)}
								</span>
							</div>
							<!-- Vouch type badge -->
							<span
								class="shrink-0 rounded-full px-2 py-0.5 text-caption font-medium {vouchTypeBadgeStyle[v.vouch_type]}"
							>
								{vouchTypeLabels[v.vouch_type]}
							</span>
						</div>
					</motion.div>
				{/each}
			</div>
			{#if vouchedBy.length > SHOW_LIMIT}
				<button
					onclick={() => (showAllVouchedBy = !showAllVouchedBy)}
					class="mt-2 text-caption text-muted-foreground hover:text-foreground transition-colors"
				>
					{showAllVouchedBy ? m.common_collapse() : m.common_view_all({ count: String(vouchedBy.length) })}
				</button>
			{/if}
		</div>

		<hr class="border-border/20" />

		<!-- Menjamin -->
		<div>
			<p class="mb-2 text-caption font-semibold text-foreground/80">
				{m.profil_vouching_for()}
				<span class="ml-1 rounded-full bg-muted/20 px-1.5 py-0.5 text-caption font-normal text-muted-foreground">
					{vouchingFor.length}
				</span>
			</p>
			<div class="space-y-2">
				{#each visibleVouchingFor as v, i}
					<motion.div
						initial={{ opacity: 0, x: -4 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.2, delay: 0.03 * i }}
					>
						<div class="flex items-center gap-2">
							<TandangAvatar
								person={{ user_id: v.user_id, name: v.user_name, avatar_url: v.user_avatar_url, tier: v.user_tier }}
								size="sm"
								showTierDot
							/>
							<div class="flex min-w-0 flex-1 flex-col">
								<span class="truncate text-caption font-medium text-foreground">
									{v.user_name}
								</span>
								<span class="text-caption text-muted-foreground">
									{relativeDate(v.created_at)}
								</span>
							</div>
							<span
								class="shrink-0 rounded-full px-2 py-0.5 text-caption font-medium {vouchTypeBadgeStyle[v.vouch_type]}"
							>
								{vouchTypeLabels[v.vouch_type]}
							</span>
						</div>
					</motion.div>
				{/each}
			</div>
			{#if vouchingFor.length > SHOW_LIMIT}
				<button
					onclick={() => (showAllVouchingFor = !showAllVouchingFor)}
					class="mt-2 text-caption text-muted-foreground hover:text-foreground transition-colors"
				>
					{showAllVouchingFor ? m.common_collapse() : m.common_view_all({ count: String(vouchingFor.length) })}
				</button>
			{/if}
		</div>
	</div>
</motion.div>

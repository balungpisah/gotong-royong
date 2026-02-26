<script lang="ts">
	import { mockTandangProfile1 } from '$lib/fixtures/mock-tandang-profiles';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import PanelPinnedCard from '$lib/components/pulse/panel-pinned-card.svelte';
	import { m } from '$lib/paraglide/messages';

	const profile = mockTandangProfile1;

	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)', 1: 'var(--c-tier-1)', 2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)', 4: 'var(--c-tier-4)'
	};
	const tierColor = tierColors[profile.tier.level] ?? '#9E9E9E';
	const joinedYear = new Date(profile.joined_at).getFullYear();
	const showFlameBadge = profile.consistency.streak_days > 7;
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-h1 font-extrabold">Aku Panel — Header Preview</h1>
		<p class="mt-1 text-body text-muted-foreground">
			Uses <code>PanelPinnedCard</code> template. Compare with Tandang header.
		</p>
	</div>

	<div class="mx-auto w-full max-w-lg rounded-lg border border-border bg-card shadow-sm">
		<PanelPinnedCard>
			{#snippet left()}
				<div class="flex items-start gap-2.5">
					<div class="relative shrink-0">
						<TandangAvatar
							person={{ user_id: profile.user_id, name: profile.name, avatar_url: profile.avatar_url, tier: profile.tier.level }}
							size="sm"
							isSelf={true}
						/>
						<span class="absolute bottom-0 right-0 size-2 rounded-full bg-online ring-1 ring-background"></span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="truncate text-h3 font-semibold leading-tight text-foreground">{profile.name}</p>
						<p class="truncate text-caption text-muted-foreground">
							{#if profile.location}{profile.location} · {/if}{m.profil_member_since({ year: String(joinedYear) })}
						</p>
						<div class="mt-1 flex flex-nowrap items-center gap-1">
							<span
								class="inline-flex items-center gap-0.5 rounded-full border px-2 py-0.5 text-xs leading-tight font-medium"
								style="border-color: color-mix(in srgb, {tierColor} 30%, transparent); background-color: color-mix(in srgb, {tierColor} 10%, transparent); color: {tierColor};"
							>{profile.tier.pips} {profile.tier.name}</span>
							{#if showFlameBadge}
								<span class="inline-flex items-center gap-0.5 rounded-full border border-peringatan/30 bg-peringatan/10 px-2 py-0.5 text-xs leading-tight font-medium text-peringatan">
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
						<div class="icj-bar-track"><div class="icj-bar-fill" style="width: {Math.round(profile.scores.integrity.value * 100)}%; background: var(--c-tandang-i)"></div></div>
						<span class="icj-value">{Math.round(profile.scores.integrity.value * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-c)">C</span>
						<div class="icj-bar-track"><div class="icj-bar-fill" style="width: {Math.round(profile.scores.competence.aggregate * 100)}%; background: var(--c-tandang-c)"></div></div>
						<span class="icj-value">{Math.round(profile.scores.competence.aggregate * 100)}</span>
					</div>
					<div class="icj-row">
						<span class="icj-label" style="color: var(--c-tandang-j)">J</span>
						<div class="icj-bar-track"><div class="icj-bar-fill" style="width: {Math.round(profile.scores.judgment.value * 100)}%; background: var(--c-tandang-j)"></div></div>
						<span class="icj-value">{Math.round(profile.scores.judgment.value * 100)}</span>
					</div>
				</div>
			{/snippet}
		</PanelPinnedCard>
	</div>
</div>

<style>
	.icj-col { display: flex; flex-direction: column; justify-content: center; gap: 0.45rem; height: 100%; }
	.icj-row { display: flex; align-items: center; gap: 0.5rem; }
	.icj-label { font-size: 0.75rem; font-weight: 800; letter-spacing: 0.05em; width: 0.75rem; flex-shrink: 0; }
	.icj-bar-track { flex: 1; height: 5px; border-radius: 99px; background: color-mix(in srgb, var(--color-muted) 40%, transparent); overflow: hidden; }
	.icj-bar-fill { height: 100%; border-radius: 99px; transition: width 500ms ease; }
	.icj-value { font-size: 11px; line-height: 16px; font-weight: 600; font-variant-numeric: tabular-nums; color: var(--color-muted-foreground); width: 1.5rem; text-align: right; flex-shrink: 0; }
</style>

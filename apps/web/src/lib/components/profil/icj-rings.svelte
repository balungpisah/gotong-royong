<script lang="ts">
	import { LayerCake, Svg } from 'layercake';
	import IcjRingsLayer from './icj-rings-layer.svelte';
	import type { TandangScores } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		scores: TandangScores;
	}

	const { scores }: Props = $props();

	let expanded = $state(false);
	let mounted = $state(false);

	$effect(() => {
		const t = setTimeout(() => { mounted = true; }, 50);
		return () => clearTimeout(t);
	});

	const ringData = $derived([
		{ label: m.icj_integrity_label(), color: 'var(--c-tandang-i)', value: scores.integrity.value },
		{ label: m.icj_competence_label(), color: 'var(--c-tandang-c)', value: scores.competence.aggregate },
		{ label: m.icj_judgment_label(), color: 'var(--c-tandang-j)', value: scores.judgment.value }
	]);
</script>

<!-- Responsive chart container -->
<div class="h-[88px]">
	<LayerCake data={ringData} padding={{ top: 4, bottom: 4, left: 4, right: 4 }}>
		<Svg>
			<IcjRingsLayer {mounted} />
		</Svg>
	</LayerCake>
</div>

<!-- Dimension labels below chart -->
<div class="mt-1 flex items-center justify-around">
	{#each ringData as ring}
		<span class="text-caption text-muted-foreground">{ring.label}</span>
	{/each}
</div>

<!-- Competence domain expansion -->
{#if scores.competence.domains.length > 0}
	<div class="mt-2 text-center">
		<button
			onclick={() => (expanded = !expanded)}
			class="text-caption text-muted-foreground hover:text-foreground transition-colors"
		>
			{expanded ? m.common_collapse() : m.profil_view_domains()}
		</button>
	</div>

	{#if expanded}
		<div class="mt-2 space-y-1.5">
			{#each scores.competence.domains as domain}
				<div class="flex items-center gap-2">
					<span class="w-28 truncate text-caption">{domain.skill_name}</span>
					<div class="h-1.5 flex-1 rounded-full bg-muted/30">
						<div
							class="h-full rounded-full bg-[var(--c-tandang-c,#00695C)]"
							style="width: {domain.score * 100}%;"
						></div>
					</div>
					<span class="w-8 text-right text-caption font-medium">
						{(domain.score * 100).toFixed(0)}
					</span>
					{#if domain.decaying}
						<span class="text-caption text-waspada">⚠ {domain.days_until_decay}d</span>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
{/if}

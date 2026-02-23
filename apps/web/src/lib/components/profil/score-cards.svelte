<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { TandangScores } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		scores: TandangScores;
		variant?: 'cards' | 'bars';
	}

	const { scores, variant = 'cards' }: Props = $props();

	let expanded = $state(false);

	const cards = $derived([
		{
			label: m.icj_integrity_label(),
			color: 'var(--c-tandang-i)',
			value: scores.integrity.value
		},
		{
			label: m.icj_competence_label(),
			color: 'var(--c-tandang-c)',
			value: scores.competence.aggregate
		},
		{
			label: m.icj_judgment_label(),
			color: 'var(--c-tandang-j)',
			value: scores.judgment.value
		}
	]);
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.1 }}
>
	{#if variant === 'bars'}
		<!-- Compact horizontal bars variant -->
		<div class="space-y-2.5">
			{#each cards as card, i}
				<motion.div
					initial={{ opacity: 0, x: -8 }}
					animate={{ opacity: 1, x: 0 }}
					transition={{ duration: 0.25, delay: 0.05 * i }}
				>
					<div class="flex items-center gap-3">
						<span class="size-2 shrink-0 rounded-full" style="background: {card.color};"></span>
						<span class="w-20 shrink-0 text-caption font-medium text-muted-foreground">{card.label}</span>
						<div class="h-2 flex-1 rounded-full bg-muted/30">
							<div
								class="h-full rounded-full transition-all duration-500"
								style="width: {card.value * 100}%; background: {card.color};"
							></div>
						</div>
						<span class="w-8 text-right text-sm font-bold text-foreground">{(card.value * 100).toFixed(0)}</span>
					</div>

					<!-- Competence domain breakdown (inline) -->
					{#if i === 1}
						<div class="ml-5 mt-1.5">
							<button
								onclick={() => (expanded = !expanded)}
								class="text-caption text-muted-foreground hover:text-foreground transition-colors"
							>
								{expanded ? m.common_collapse() : m.profil_view_domains()}
							</button>

							{#if expanded}
								<div class="mt-1.5 space-y-1.5">
									{#each scores.competence.domains as domain}
										<div class="flex items-center gap-2">
											<span class="w-28 truncate text-caption">
												{domain.skill_name}
											</span>
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
												<span class="text-caption text-waspada">
													⚠ {domain.days_until_decay}d
												</span>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</motion.div>
			{/each}
		</div>
	{:else}
		<!-- Original cards variant -->
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
			{#each cards as card, i}
				<motion.div
					initial={{ opacity: 0, y: 8 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.3, delay: 0.05 * i }}
				>
					<div
						class="rounded-xl border border-border/30 bg-card p-4 shadow-sm"
						style="border-top: 4px solid {card.color};"
					>
						<div class="text-caption font-medium text-muted-foreground">
							{card.label}
						</div>
						<div
							class="mt-1 text-h1 font-bold"
							style="color: {card.color};"
						>
							{(card.value * 100).toFixed(0)}
						</div>
						<div class="mt-1.5 h-1.5 rounded-full bg-muted/30">
							<div
								class="h-full rounded-full transition-all duration-500"
								style="width: {card.value * 100}%; background: {card.color};"
							></div>
						</div>

						<!-- Competence domain breakdown -->
						{#if i === 1}
							<div class="mt-3">
								<button
									onclick={() => (expanded = !expanded)}
									class="text-caption text-muted-foreground hover:text-foreground transition-colors"
								>
									{expanded ? m.common_collapse() : m.profil_view_domains()}
								</button>

								{#if expanded}
									<div class="mt-2 space-y-1.5">
										{#each scores.competence.domains as domain}
											<div class="flex items-center gap-2">
												<span class="w-40 truncate text-caption">
													{domain.skill_name}
												</span>
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
													<span class="text-caption text-waspada">
														⚠ {domain.days_until_decay}d
													</span>
												{/if}
											</div>
										{/each}
									</div>
								{/if}
							</div>
						{/if}
					</div>
				</motion.div>
			{/each}
		</div>
	{/if}
</motion.div>

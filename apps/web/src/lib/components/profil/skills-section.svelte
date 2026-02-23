<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { UserSkill } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		skills: UserSkill[];
	}

	const { skills }: Props = $props();

	const validated = $derived(skills.filter((s) => s.validated));
	const declared = $derived(skills.filter((s) => !s.validated));
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.15 }}
>
	<div class="rounded-xl border border-border/30 bg-muted/10 p-4">
		<h3 class="text-xs font-semibold text-foreground">{m.profil_skills_title()}</h3>

		<!-- Validated skills -->
		{#if validated.length > 0}
			<div class="mt-3">
				<p class="mb-2 flex items-center gap-1 text-caption text-muted-foreground">
					<span class="text-[var(--c-tandang-c,#00695C)]">●</span>
					<span class="font-medium">{m.profil_skills_validated()}</span>
				</p>
				<div class="space-y-2">
					{#each validated as skill, i}
						<motion.div
							initial={{ opacity: 0, x: -6 }}
							animate={{ opacity: 1, x: 0 }}
							transition={{ duration: 0.25, delay: 0.04 * i }}
						>
							<div class="flex items-center gap-2">
								<span class="w-36 shrink-0 truncate text-caption text-foreground/80">
									{skill.skill_name}
								</span>
								{#if skill.score !== undefined}
									<div class="h-1.5 flex-1 rounded-full bg-muted/30">
										<div
											class="h-full rounded-full bg-[var(--c-tandang-c,#00695C)] transition-all duration-500"
											style="width: {(skill.score ?? 0) * 100}%;"
										></div>
									</div>
									<span class="w-8 shrink-0 text-right text-caption font-medium text-foreground/70">
										{((skill.score ?? 0) * 100).toFixed(0)}
									</span>
								{/if}
								{#if skill.decaying}
									<span class="shrink-0 text-caption text-waspada">
										⚠ {skill.days_until_decay}d
									</span>
								{/if}
							</div>
						</motion.div>
					{/each}
				</div>
			</div>
		{/if}

		{#if validated.length > 0 && declared.length > 0}
			<hr class="my-3 border-border/20" />
		{/if}

		<!-- Declared skills -->
		{#if declared.length > 0}
			<div class="mt-3">
				<p class="mb-2 flex items-center gap-1 text-caption text-muted-foreground">
					<span class="text-muted-foreground/60">○</span>
					<span class="font-medium">{m.profil_skills_declared()}</span>
				</p>
				<div class="flex flex-wrap gap-1.5">
					{#each declared as skill, i}
						<motion.div
							initial={{ opacity: 0, scale: 0.9 }}
							animate={{ opacity: 1, scale: 1 }}
							transition={{ duration: 0.2, delay: 0.03 * i }}
						>
							<span class="rounded-md bg-muted/20 px-2 py-0.5 text-caption text-foreground/70">
								{skill.skill_name}
							</span>
						</motion.div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Add skill button -->
		<div class="mt-4">
			<button
				disabled
				class="cursor-not-allowed rounded-md border border-border/30 px-3 py-1.5 text-caption text-muted-foreground/50 opacity-60"
			>
				{m.profil_skills_add()}
			</button>
		</div>
	</div>
</motion.div>

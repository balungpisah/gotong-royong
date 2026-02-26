<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import { m } from '$lib/paraglide/messages';

	interface DecayWarning {
		domain: string;
		days_until_decay: number;
	}

	interface Props {
		warnings: DecayWarning[];
	}

	const { warnings }: Props = $props();
</script>

{#if warnings.length > 0}
	<motion.div
		initial={{ opacity: 0, y: -4 }}
		animate={{ opacity: 1, y: 0 }}
		transition={{ duration: 0.3 }}
	>
		<div class="rounded-xl border border-waspada/30 bg-waspada-lembut/50 p-4">
			<div class="flex items-center gap-2">
				<AlertTriangle class="size-4 shrink-0 text-waspada" />
				<h3 class="text-small font-semibold text-waspada">{m.profil_decay_title()}</h3>
			</div>
			<p class="mt-1 text-caption text-waspada">
				{m.profil_decay_description()}
			</p>
			<div class="mt-2 flex flex-wrap gap-1.5">
				{#each warnings as warning}
					<span class="rounded-full bg-waspada-lembut px-2.5 py-0.5 text-small font-medium text-waspada">
						{warning.domain}
						{#if warning.days_until_decay <= 7}
							<span class="text-waspada">({warning.days_until_decay}h)</span>
						{/if}
					</span>
				{/each}
			</div>
		</div>
	</motion.div>
{/if}

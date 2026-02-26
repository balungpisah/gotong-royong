<script lang="ts">
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import Lock from '@lucide/svelte/icons/lock';
	import ClipboardCheck from '@lucide/svelte/icons/clipboard-check';
	import { m } from '$lib/paraglide/messages';
	import type { Phase } from '$lib/types';

	interface Props {
		phases: Phase[];
	}

	const { phases }: Props = $props();

	const blockedCount = $derived(
		phases.flatMap((p) => p.checkpoints ?? []).filter((c) => c.status === 'blocked').length
	);

	const evidenceCount = $derived(
		phases
			.flatMap((p) => p.checkpoints ?? [])
			.filter((c) => c.evidence_required === true && c.status !== 'completed').length
	);
</script>

{#if blockedCount > 0 || evidenceCount > 0}
	<div class="rounded-xl border border-peringatan/30 bg-peringatan-lembut/50 px-3 py-2">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1.5">
				<AlertTriangle class="h-3.5 w-3.5 text-peringatan" />
				<span class="text-small font-medium text-peringatan"
					>{m.pulse_action_needs_attention()}</span
				>
			</div>

			{#if blockedCount > 0}
				<div class="flex items-center gap-1 rounded-full bg-bahaya/10 px-2 py-0.5">
					<Lock class="h-3 w-3 text-bahaya" />
					<span class="text-small text-bahaya">{blockedCount} langkah terblokir</span>
				</div>
			{/if}

			{#if evidenceCount > 0}
				<div class="flex items-center gap-1 rounded-full bg-peringatan/10 px-2 py-0.5">
					<ClipboardCheck class="h-3 w-3 text-peringatan" />
					<span class="text-small text-peringatan">{evidenceCount} butuh bukti</span>
				</div>
			{/if}
		</div>
	</div>
{/if}

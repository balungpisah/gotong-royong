<script lang="ts">
	import { onMount } from 'svelte';
	import { ChatInput } from '$lib/components/shell';
	import { getTriageStore } from '$lib/stores';

	const triageStore = getTriageStore();

	let createdWitnessId = $state<string | null>(null);

	onMount(() => {
		triageStore.reset();
	});

	const handleWitnessCreated = (witnessId: string) => {
		createdWitnessId = witnessId;
	};
</script>

<div class="mx-auto flex w-full max-w-3xl flex-col gap-4" data-testid="e2e-triage-harness">
	<div>
		<h1 class="text-h1 font-extrabold text-foreground">Triage E2E Harness</h1>
		<p class="text-body text-muted-foreground">
			Dedicated deterministic surface for triage payload rendering tests.
		</p>
	</div>

	<div
		class="rounded-2xl border border-border/60 bg-card p-4 shadow-sm"
		data-testid="e2e-triage-chat-shell"
	>
		<ChatInput onWitnessCreated={handleWitnessCreated} />
	</div>

	{#if createdWitnessId}
		<p class="text-small text-muted-foreground" data-testid="e2e-triage-created-witness">
			Witness created: {createdWitnessId}
		</p>
	{/if}
</div>

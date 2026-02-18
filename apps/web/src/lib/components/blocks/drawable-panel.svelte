<script lang="ts">
	import type { PathPlan } from '$lib/types';
	import type { Snippet } from 'svelte';
	import * as Sheet from '$lib/components/ui/sheet';
	import PathPlanView from './path-plan-view.svelte';

	let { plan, trigger }: { plan: PathPlan; trigger: Snippet } = $props();
	let open = $state(false);
</script>

<Sheet.Root bind:open>
	<Sheet.Trigger>
		{@render trigger()}
	</Sheet.Trigger>
	<Sheet.Content side="right" class="w-full max-w-md overflow-y-auto sm:max-w-lg">
		<Sheet.Header>
			<Sheet.Title>Rencana</Sheet.Title>
			<Sheet.Description>Detail rencana dan fase</Sheet.Description>
		</Sheet.Header>
		<div class="px-1 py-4">
			<PathPlanView {plan} />
		</div>
	</Sheet.Content>
</Sheet.Root>

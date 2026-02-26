<script lang="ts">
	import type { GroupSummary } from '$lib/types';
	import GroupCard from './group-card.svelte';

	interface Props {
		groups: GroupSummary[];
		title: string;
		onJoin?: (groupId: string) => void;
		onRequestJoin?: (groupId: string) => void;
	}

	let { groups, title, onJoin, onRequestJoin }: Props = $props();
</script>

<section class="space-y-3">
	<h2 class="text-body font-bold text-foreground">{title}</h2>
	{#if groups.length === 0}
		<div class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-6 text-center">
			<p class="text-small text-muted-foreground/80">Belum ada kelompok di sini.</p>
		</div>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2">
			{#each groups as group (group.group_id)}
				<GroupCard {group} {onJoin} {onRequestJoin} />
			{/each}
		</div>
	{/if}
</section>


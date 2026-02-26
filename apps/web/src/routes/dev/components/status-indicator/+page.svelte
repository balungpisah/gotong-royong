<script lang="ts">
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';

	const statuses: StatusIndicatorStatus[] = [
		'active',
		'stalled',
		'done',
		'review',
		'moderation',
		'sealed'
	];
</script>

<h1 class="mb-6 text-h1 font-extrabold">Status Indicator</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">All States</h2>
	<div class="flex flex-wrap items-center gap-6">
		{#each statuses as status (status)}
			<StatusIndicator {status}>{status}</StatusIndicator>
		{/each}
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Dot Only (no label)</h2>
	<div class="flex flex-wrap items-center gap-4">
		{#each statuses as status (status)}
			<StatusIndicator {status} />
		{/each}
	</div>
</section>

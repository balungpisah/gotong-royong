<script lang="ts">
	import type { FeedStreamItem } from '$lib/types';
	import { mockFeedItem1, mockFeedItem2, mockFeedItem3 } from '$lib/fixtures/mock-feed';
	import {
		registeredFeedStreamKinds,
		resolveFeedStreamRenderer
	} from '$lib/components/pulse';
	import systemMatrix from './matrix.json';

	let selectedWitnessId = $state<string | null>(null);

	const witnessMatrix: FeedStreamItem[] = [mockFeedItem1, mockFeedItem2, mockFeedItem3].map((item) => ({
		stream_id: `w-${item.witness_id}`,
		sort_timestamp: item.latest_event.timestamp,
		kind: 'witness',
		data: item
	}));

	const streamMatrix = [
		witnessMatrix[0],
		...(systemMatrix as FeedStreamItem[]),
		witnessMatrix[1],
		witnessMatrix[2]
	];
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-h1 font-extrabold">Feed Cards Matrix</h1>
		<p class="mt-1 text-body text-muted-foreground">
			Dev surface to preview feed rendering using registry + JSON scenarios.
		</p>
		<p class="mt-2 text-small text-muted-foreground">
			Registered stream kinds: {registeredFeedStreamKinds.join(', ')}
		</p>
	</div>

	<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
		{#each streamMatrix as streamItem (streamItem.stream_id)}
			{@const rendered = resolveFeedStreamRenderer(streamItem, {
				selectedWitnessId,
				onSelectWitness: (witnessId) => {
					selectedWitnessId = selectedWitnessId === witnessId ? null : witnessId;
				},
				onToggleMonitor: () => undefined,
				onShareWitness: () => undefined,
				onDismissSystemCard: () => undefined
			})}
			{@const CardComponent = rendered.component as any}
			{@const cardProps = rendered.props as any}
			<CardComponent {...cardProps} />
		{/each}
	</div>
</div>

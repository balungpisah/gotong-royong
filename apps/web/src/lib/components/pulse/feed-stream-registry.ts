import type { FeedItem, FeedStreamItem } from '$lib/types';
import FeedEventCard from './feed-event-card.svelte';
import FeedSystemCard from './feed-system-card.svelte';

export interface FeedStreamRenderContext {
	selectedWitnessId: string | null;
	onSelectWitness: (witnessId: string) => void;
	onToggleMonitor: (witnessId: string) => void;
	onShareWitness: (item: FeedItem) => void;
	onDismissSystemCard: (streamId: string) => void;
}

export interface FeedStreamRenderNode {
	component: typeof FeedEventCard | typeof FeedSystemCard;
	props: Record<string, unknown>;
}

type FeedStreamRenderer<K extends FeedStreamItem['kind']> = (
	item: Extract<FeedStreamItem, { kind: K }>,
	context: FeedStreamRenderContext
) => FeedStreamRenderNode;

const registry: { [K in FeedStreamItem['kind']]: FeedStreamRenderer<K> } = {
	witness: (item, context) => ({
		component: FeedEventCard,
		props: {
			item: item.data,
			selected: context.selectedWitnessId === item.data.witness_id,
			onclick: () => context.onSelectWitness(item.data.witness_id),
			onToggleMonitor: () => context.onToggleMonitor(item.data.witness_id),
			onShare: () => context.onShareWitness(item.data)
		}
	}),
	system: (item, context) => ({
		component: FeedSystemCard,
		props: {
			card: item.data,
			onDismiss: () => context.onDismissSystemCard(item.stream_id)
		}
	}),
	data: (item) => ({
		component: FeedSystemCard,
		props: {
			card: {
				variant: 'prompt',
				icon: '🗂️',
				title: item.data.title,
				description: item.data.claim,
				dismissible: false,
				payload: {
					variant: 'prompt',
					cta_label: 'Buka Detail',
					cta_action: `open_data:${item.data.data_id}`
				}
			}
		}
	})
};

export const registeredFeedStreamKinds = Object.freeze(
	Object.keys(registry) as FeedStreamItem['kind'][]
);

export const resolveFeedStreamRenderer = (
	item: FeedStreamItem,
	context: FeedStreamRenderContext
): FeedStreamRenderNode => {
	const renderer = registry[item.kind] as FeedStreamRenderer<typeof item.kind>;
	return renderer(item as never, context);
};

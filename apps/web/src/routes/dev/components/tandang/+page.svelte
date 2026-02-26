<script lang="ts">
	import { WitnessDetailPanel } from '$lib/components/pulse';
	import { mockWitnessDetail } from '$lib/fixtures/mock-witnesses';
	import { mockFeedItem1 } from '$lib/fixtures/mock-feed';

	// Mutable copy for local send-message interactions in component gallery
	let detail = $state({ ...mockWitnessDetail, messages: [...mockWitnessDetail.messages] });
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-h1 font-extrabold">Tandang Panel (Witness Detail)</h1>
		<p class="mt-1 text-body text-muted-foreground">
			Full witness detail with Octalysis enrichment: progress bar, member presence, scarcity nudges, story narrative phases, celebration moments
		</p>
	</div>

	<!-- Panel container — simulates right-panel context box width -->
	<div class="mx-auto w-full max-w-lg rounded-lg border border-border bg-card shadow-sm" style="height: 700px;">
		<WitnessDetailPanel
			{detail}
			feedItem={mockFeedItem1}
			onClose={() => {}}
			onSendMessage={(msg, attachments) => console.log('Send:', msg, attachments)}
			sending={false}
		/>
	</div>
</div>

<script lang="ts">
	import { WitnessDetailPanel } from '$lib/components/pulse';
	import { mockWitnessDetail } from '$lib/fixtures/mock-witnesses';
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-bold">Tandang Panel (Witness Detail)</h1>
		<p class="mt-1 text-sm text-muted-foreground">
			Full witness detail with Octalysis enrichment: progress bar, member presence, scarcity nudges, story narrative phases, celebration moments
		</p>
	</div>

	<!-- Panel container — simulates right-panel context box width -->
	<div class="mx-auto w-full max-w-lg rounded-lg border border-border bg-card shadow-sm" style="height: 700px;">
		<WitnessDetailPanel
			detail={mockWitnessDetail}
			onClose={() => {}}
			onSendMessage={(msg) => console.log('Send:', msg)}
			sending={false}
		/>
	</div>
</div>

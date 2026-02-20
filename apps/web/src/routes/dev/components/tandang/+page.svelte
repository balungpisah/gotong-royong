<script lang="ts">
	import { WitnessDetailPanel } from '$lib/components/pulse';
	import { mockWitnessDetail } from '$lib/fixtures/mock-witnesses';
	import { mockFeedItem1 } from '$lib/fixtures/mock-feed';
	import type { ChatMessage, AiCardMessage, SystemMessage } from '$lib/types';
	import type { ListBlock } from '$lib/types/blocks';

	// Mutable copy of messages so we can inject stempel results
	let detail = $state({ ...mockWitnessDetail, messages: [...mockWitnessDetail.messages] });
	let stempeling = $state(false);

	function handleStempel() {
		if (stempeling) return;
		stempeling = true;

		// Simulate AI evaluation delay (2 seconds)
		setTimeout(() => {
			const now = new Date().toISOString();

			// 1. AI evaluation card — checklist showing what's done / what's missing
			const stempelBlock: ListBlock = {
				type: 'list',
				id: 'stempel-eval-001',
				display: 'checklist',
				title: 'Evaluasi Fase: Penggalangan Dana',
				items: [
					{
						id: 'se-1',
						label: 'Target dana ditetapkan',
						status: 'completed',
						source: 'ai',
						locked_fields: []
					},
					{
						id: 'se-2',
						label: 'Minimal 3 kontributor',
						status: 'completed',
						source: 'ai',
						locked_fields: []
					},
					{
						id: 'se-3',
						label: 'Bukti transfer/kuitansi diunggah',
						status: 'open',
						source: 'ai',
						locked_fields: []
					},
					{
						id: 'se-4',
						label: 'Rencana pencairan disetujui voting',
						status: 'open',
						source: 'ai',
						locked_fields: []
					}
				]
			};

			const aiCard: AiCardMessage = {
				message_id: `stempel-${Date.now()}`,
				timestamp: now,
				witness_id: detail.witness_id,
				type: 'ai_card',
				blocks: [stempelBlock],
				badge: 'ringkasan',
				title: '✦ Hasil Stempel — 2/4 terpenuhi'
			};

			// 2. System message — checkpoint update
			const checkpoint: SystemMessage = {
				message_id: `sys-stempel-${Date.now()}`,
				timestamp: now,
				witness_id: detail.witness_id,
				type: 'system',
				subtype: 'checkpoint_completed',
				content: '✦ Stempel: 2 dari 4 langkah terpenuhi — lanjutkan diskusi'
			};

			// Inject into messages
			detail.messages = [...detail.messages, aiCard, checkpoint];
			stempeling = false;
		}, 2000);
	}
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
			{detail}
			feedItem={mockFeedItem1}
			onClose={() => {}}
			onSendMessage={(msg) => console.log('Send:', msg)}
			onStempel={handleStempel}
			sending={false}
			{stempeling}
		/>
	</div>
</div>

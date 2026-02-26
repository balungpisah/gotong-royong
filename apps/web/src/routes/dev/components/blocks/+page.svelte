<script lang="ts">
	import { BlockRenderer } from '$lib/components/blocks';
	import {
		mockListBlock,
		mockListBlockTable,
		mockListBlockTimeline,
		mockListBlockGallery,
		mockDocumentBlock,
		mockFormBlock,
		mockComputedBlock,
		mockDisplayBlock,
		mockVoteBlock,
		mockReferenceBlock
	} from '$lib/fixtures';
</script>

<div class="flex flex-col gap-8">
	<div>
		<h1 class="text-h1 font-extrabold">Block Renderers</h1>
		<p class="mt-1 text-body text-muted-foreground">
			7 block primitives rendered via BlockRenderer dispatcher
		</p>
	</div>

	<!-- List Block: Checklist -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">List Block — Checklist</h2>
		<BlockRenderer block={mockListBlock} />
	</section>

	<!-- List Block: Table -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">List Block — Table</h2>
		<BlockRenderer block={mockListBlockTable} />
	</section>

	<!-- List Block: Timeline -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">List Block — Timeline</h2>
		<BlockRenderer block={mockListBlockTimeline} />
	</section>

	<!-- List Block: Gallery -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">List Block — Gallery</h2>
		<BlockRenderer block={mockListBlockGallery} />
	</section>

	<!-- Document Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Document Block</h2>
		<BlockRenderer block={mockDocumentBlock} />
	</section>

	<!-- Form Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Form Block</h2>
		<BlockRenderer block={mockFormBlock} />
	</section>

	<!-- Computed Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Computed Block</h2>
		<BlockRenderer block={mockComputedBlock} />
	</section>

	<!-- Display Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Display Block</h2>
		<BlockRenderer block={mockDisplayBlock} />
	</section>

	<!-- Vote Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Vote Block</h2>
		<BlockRenderer block={mockVoteBlock} />
	</section>

	<!-- Reference Block -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Reference Block</h2>
		<BlockRenderer block={mockReferenceBlock} />
	</section>
</div>

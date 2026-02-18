<script lang="ts">
	import { DiffCardRenderer } from '$lib/components/blocks';
	import type { DiffCard } from '$lib/types';

	const mockDiff: DiffCard = {
		diff_id: 'diff-demo-001',
		target_type: 'list',
		target_id: 'list-001',
		summary: 'Ditambah 3 item, diubah 1, dihapus 1',
		evidence: [
			'Berdasarkan laporan foto warga RT 05',
			'Koordinasi dengan ketua RT sudah dilakukan'
		],
		items: [
			{
				operation: 'add',
				path: 'items[4]',
				label: 'Hubungi Dinas Pekerjaan Umum',
				protected: false
			},
			{ operation: 'add', path: 'items[5]', label: 'Siapkan proposal anggaran', protected: false },
			{
				operation: 'modify',
				path: 'items[1].status',
				label: 'Koordinasi RT selesai',
				old_value: 'open',
				new_value: 'completed',
				protected: false
			},
			{ operation: 'remove', path: 'items[3]', label: 'Hapus item duplikat', protected: false },
			{ operation: 'add', path: 'items[6]', label: 'Update nomor rekening', protected: true },
			{
				operation: 'reorder',
				path: 'items',
				label: 'Urutkan berdasarkan prioritas',
				protected: false
			}
		],
		source: 'ai',
		generated_at: new Date().toISOString(),
		plan_version: 3
	};

	function handleDiffAction(action: string) {
		console.log('Diff action:', action);
	}
</script>

<div class="flex flex-col gap-8">
	<div>
		<h1 class="text-2xl font-bold">Diff Card</h1>
		<p class="mt-1 text-sm text-muted-foreground">
			"Suggest-Don't-Overwrite" pattern — AI suggests changes via diff cards
		</p>
	</div>

	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-lg font-semibold">Full Diff Card</h2>
		<div class="max-w-lg">
			<DiffCardRenderer diff={mockDiff} ondiffaction={handleDiffAction} />
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-lg font-semibold">Operations Legend</h2>
		<div class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
			<div class="flex items-center gap-2">
				<div
					class="flex size-5 items-center justify-center rounded bg-berhasil-lembut text-berhasil text-xs"
				>
					+
				</div>
				<span>Tambah (add)</span>
			</div>
			<div class="flex items-center gap-2">
				<div
					class="flex size-5 items-center justify-center rounded bg-bahaya-lembut text-bahaya text-xs"
				>
					−
				</div>
				<span>Hapus (remove)</span>
			</div>
			<div class="flex items-center gap-2">
				<div
					class="flex size-5 items-center justify-center rounded bg-peringatan-lembut text-peringatan text-xs"
				>
					✎
				</div>
				<span>Ubah (modify)</span>
			</div>
			<div class="flex items-center gap-2">
				<div
					class="flex size-5 items-center justify-center rounded bg-keterangan-lembut text-keterangan text-xs"
				>
					↕
				</div>
				<span>Urutkan (reorder)</span>
			</div>
		</div>
	</section>
</div>

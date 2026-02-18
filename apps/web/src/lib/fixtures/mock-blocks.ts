/**
 * Mock block fixtures for the dev gallery.
 * One instance per block type (7 types) with realistic Indonesian-language content.
 * Additional list block variants for all 4 display modes.
 */

import type {
	ListBlock,
	DocumentBlock,
	FormBlock,
	ComputedBlock,
	DisplayBlock,
	VoteBlock,
	ReferenceBlock,
	Block
} from '$lib/types';

// ---------------------------------------------------------------------------
// List Block — checklist (default)
// ---------------------------------------------------------------------------

export const mockListBlock: ListBlock = {
	type: 'list',
	id: 'list-001',
	display: 'checklist',
	title: 'Langkah Penyelesaian',
	items: [
		{
			id: 'item-1',
			label: 'Dokumentasi foto lokasi',
			status: 'completed',
			source: 'human',
			locked_fields: []
		},
		{
			id: 'item-2',
			label: 'Koordinasi dengan RT setempat',
			status: 'completed',
			source: 'ai',
			locked_fields: []
		},
		{
			id: 'item-3',
			label: 'Pengajuan ke kelurahan',
			status: 'open',
			source: 'ai',
			locked_fields: [],
			children: [
				{
					id: 'item-3a',
					label: 'Siapkan surat pengantar',
					status: 'open',
					source: 'ai',
					locked_fields: []
				},
				{
					id: 'item-3b',
					label: 'Kumpulkan tanda tangan warga',
					status: 'open',
					source: 'ai',
					locked_fields: []
				}
			]
		},
		{
			id: 'item-4',
			label: 'Menunggu tanggapan dinas',
			status: 'blocked',
			source: 'system',
			locked_fields: []
		}
	]
};

// ---------------------------------------------------------------------------
// List Block — table display
// ---------------------------------------------------------------------------

export const mockListBlockTable: ListBlock = {
	type: 'list',
	id: 'list-002',
	display: 'table',
	title: 'Daftar Peserta Gotong Royong',
	items: [
		{
			id: 'row-1',
			label: 'Ahmad Hidayat',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { rt: 'RT 05', kontribusi: 'Koordinator', hadir: 'Ya' }
		},
		{
			id: 'row-2',
			label: 'Sari Dewi',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { rt: 'RT 05', kontribusi: 'Relawan', hadir: 'Ya' }
		},
		{
			id: 'row-3',
			label: 'Budi Santoso',
			status: 'open',
			source: 'ai',
			locked_fields: [],
			meta: { rt: 'RT 06', kontribusi: 'Pendukung', hadir: 'Belum' }
		},
		{
			id: 'row-4',
			label: 'Rina Kartika',
			status: 'open',
			source: 'human',
			locked_fields: [],
			meta: { rt: 'RT 05', kontribusi: 'Pelapor', hadir: 'Belum' }
		},
		{
			id: 'row-5',
			label: 'Hendra Wijaya',
			status: 'skipped',
			source: 'human',
			locked_fields: [],
			meta: { rt: 'RT 07', kontribusi: 'Saksi', hadir: 'Tidak' }
		}
	]
};

// ---------------------------------------------------------------------------
// List Block — timeline display
// ---------------------------------------------------------------------------

export const mockListBlockTimeline: ListBlock = {
	type: 'list',
	id: 'list-003',
	display: 'timeline',
	title: 'Kronologi Kejadian Jalan Rusak',
	items: [
		{
			id: 'tl-1',
			label: 'Oktober 2024 — Jalan mulai retak akibat hujan deras',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { date: '2024-10-15', verified: true }
		},
		{
			id: 'tl-2',
			label: 'November 2024 — Lubang pertama muncul, lebar 20cm',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { date: '2024-11-03', verified: true }
		},
		{
			id: 'tl-3',
			label: 'Desember 2024 — Dua motor jatuh karena lubang',
			status: 'completed',
			source: 'human',
			locked_fields: ['label'],
			meta: { date: '2024-12-08', verified: true }
		},
		{
			id: 'tl-4',
			label: 'Januari 2025 — Laporan resmi ke kelurahan diajukan',
			status: 'completed',
			source: 'ai',
			locked_fields: [],
			meta: { date: '2025-01-15', verified: false }
		},
		{
			id: 'tl-5',
			label: 'Februari 2025 — Penggalangan dana swadaya dimulai',
			status: 'open',
			source: 'ai',
			locked_fields: [],
			meta: { date: '2025-02-01', verified: false }
		},
		{
			id: 'tl-6',
			label: 'Maret 2025 — Target perbaikan selesai',
			status: 'open',
			source: 'ai',
			locked_fields: [],
			meta: { date: '2025-03-01', verified: false }
		}
	]
};

// ---------------------------------------------------------------------------
// List Block — gallery display
// ---------------------------------------------------------------------------

export const mockListBlockGallery: ListBlock = {
	type: 'list',
	id: 'list-004',
	display: 'gallery',
	title: 'Dokumentasi Foto Lokasi',
	items: [
		{
			id: 'gal-1',
			label: 'Lubang besar di depan nomor 15',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { url: 'https://placehold.co/400x300/C05621/white?text=Lubang+1', date: '2025-01-10' }
		},
		{
			id: 'gal-2',
			label: 'Kondisi aspal yang terkelupas',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: {
				url: 'https://placehold.co/400x300/7B341E/white?text=Aspal+Rusak',
				date: '2025-01-10'
			}
		},
		{
			id: 'gal-3',
			label: 'Genangan air saat hujan',
			status: 'completed',
			source: 'human',
			locked_fields: [],
			meta: { url: 'https://placehold.co/400x300/2C5282/white?text=Genangan', date: '2025-01-12' }
		},
		{
			id: 'gal-4',
			label: 'Rambu peringatan sementara',
			status: 'open',
			source: 'ai',
			locked_fields: [],
			meta: { url: 'https://placehold.co/400x300/744210/white?text=Rambu', date: '2025-01-14' }
		}
	]
};

// ---------------------------------------------------------------------------
// Document Block
// ---------------------------------------------------------------------------

export const mockDocumentBlock: DocumentBlock = {
	type: 'document',
	id: 'doc-001',
	title: 'Laporan Gotong Royong',
	sections: [
		{
			id: 'sec-1',
			heading: 'Latar Belakang',
			content:
				'Warga **RT 05** mengadukan kondisi jalan rusak di Jl. Mawar yang sudah berlangsung selama *3 bulan*. Lubang-lubang besar membahayakan pengguna jalan, terutama pengendara motor.',
			source: 'ai',
			locked_fields: []
		},
		{
			id: 'sec-2',
			heading: 'Tindakan yang Sudah Dilakukan',
			content:
				'1. Pelaporan ke kelurahan\n2. Pemasangan rambu peringatan\n3. Penggalangan dana swadaya',
			source: 'human',
			locked_fields: ['content']
		},
		{
			id: 'sec-3',
			heading: 'Rekomendasi',
			content:
				'Perlu dilakukan perbaikan segera dengan estimasi biaya **Rp 15.000.000**. Dana dapat diperoleh dari kombinasi swadaya dan bantuan kelurahan.',
			source: 'ai',
			locked_fields: []
		}
	]
};

// ---------------------------------------------------------------------------
// Form Block
// ---------------------------------------------------------------------------

export const mockFormBlock: FormBlock = {
	type: 'form',
	id: 'form-001',
	title: 'Formulir Pengaduan',
	fields: [
		{
			id: 'f-1',
			label: 'Nama Pelapor',
			field_type: 'text',
			value: 'Ahmad Hidayat',
			protected: false,
			source: 'human',
			locked_fields: ['value']
		},
		{
			id: 'f-2',
			label: 'Lokasi Kejadian',
			field_type: 'text',
			placeholder: 'Masukkan alamat lengkap',
			protected: false,
			source: 'ai',
			locked_fields: []
		},
		{
			id: 'f-3',
			label: 'Tanggal Kejadian',
			field_type: 'date',
			value: '2025-01-15',
			protected: false,
			source: 'human',
			locked_fields: []
		},
		{
			id: 'f-4',
			label: 'Kategori',
			field_type: 'select',
			value: 'infrastruktur',
			protected: false,
			source: 'ai',
			locked_fields: [],
			options: [
				{ value: 'infrastruktur', label: 'Infrastruktur' },
				{ value: 'lingkungan', label: 'Lingkungan' },
				{ value: 'sosial', label: 'Sosial' },
				{ value: 'keamanan', label: 'Keamanan' }
			]
		},
		{
			id: 'f-5',
			label: 'Deskripsi',
			field_type: 'textarea',
			placeholder: 'Ceritakan kronologi kejadian...',
			protected: false,
			source: 'ai',
			locked_fields: []
		},
		{
			id: 'f-6',
			label: 'Nomor KTP',
			field_type: 'text',
			value: '3201****',
			protected: true,
			source: 'human',
			locked_fields: ['value'],
			validation: { required: true }
		},
		{
			id: 'f-7',
			label: 'Butuh tindak lanjut segera',
			field_type: 'toggle',
			value: true,
			protected: false,
			source: 'human',
			locked_fields: []
		}
	]
};

// ---------------------------------------------------------------------------
// Computed Block
// ---------------------------------------------------------------------------

export const mockComputedBlock: ComputedBlock = {
	type: 'computed',
	id: 'comp-001',
	display: 'progress',
	label: 'Kemajuan Penyelesaian',
	value: 72,
	max: 100,
	unit: '%'
};

export const mockComputedBlockScore: ComputedBlock = {
	type: 'computed',
	id: 'comp-002',
	display: 'score',
	label: 'Skor Kepercayaan Laporan',
	value: 87,
	max: 100,
	unit: 'poin'
};

export const mockComputedBlockCounter: ComputedBlock = {
	type: 'computed',
	id: 'comp-003',
	display: 'counter',
	label: 'Jumlah Saksi',
	value: 14
};

// ---------------------------------------------------------------------------
// Display Block
// ---------------------------------------------------------------------------

export const mockDisplayBlock: DisplayBlock = {
	type: 'display',
	id: 'disp-001',
	title: 'Penghargaan Warga Aktif',
	content:
		'Terima kasih kepada **Ibu Sari** yang telah aktif mengkoordinasi kegiatan gotong royong selama 3 bulan terakhir. Kontribusinya sangat berarti bagi lingkungan RT 05.',
	media: [
		{
			type: 'image',
			url: 'https://placehold.co/400x300/C05621/white?text=Gotong+Royong',
			alt: 'Kegiatan gotong royong'
		}
	],
	meta: { awarded_by: 'Ketua RT 05', date: '2025-02-01' }
};

// ---------------------------------------------------------------------------
// Vote Block
// ---------------------------------------------------------------------------

export const mockVoteBlock: VoteBlock = {
	type: 'vote',
	id: 'vote-001',
	question: 'Setuju dengan rencana perbaikan jalan menggunakan dana swadaya?',
	vote_type: 'quorum_1_5x',
	options: [
		{ id: 'opt-1', label: 'Setuju', count: 23 },
		{ id: 'opt-2', label: 'Tidak Setuju', count: 5 },
		{ id: 'opt-3', label: 'Abstain', count: 3 }
	],
	quorum: 0.3,
	total_eligible: 45,
	total_voted: 31,
	duration_hours: 72,
	ends_at: new Date(Date.now() + 48 * 60 * 60 * 1000).toISOString(),
	user_voted: false
};

// ---------------------------------------------------------------------------
// Reference Block
// ---------------------------------------------------------------------------

export const mockReferenceBlock: ReferenceBlock = {
	type: 'reference',
	id: 'ref-001',
	ref_id: 'seed-042',
	ref_type: 'seed',
	title: 'Laporan Jalan Rusak Jl. Mawar',
	snippet:
		'Jalan rusak dengan lubang besar di depan rumah nomor 15, sudah 3 bulan belum diperbaiki.',
	track_hint: 'tuntaskan'
};

// ---------------------------------------------------------------------------
// All blocks array
// ---------------------------------------------------------------------------

export const mockAllBlocks: Block[] = [
	mockListBlock,
	mockDocumentBlock,
	mockFormBlock,
	mockComputedBlock,
	mockDisplayBlock,
	mockVoteBlock,
	mockReferenceBlock
];

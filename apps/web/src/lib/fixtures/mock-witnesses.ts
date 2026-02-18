/**
 * Mock witness fixtures for the dev gallery.
 * 5 witness summaries covering all status/track/seed/rahasia variants,
 * plus 1 detailed witness with full chat, plan, blocks, and members.
 */

import type { Witness, WitnessDetail, WitnessMember } from '$lib/types';
import { mockChatMessages } from './mock-chat';
import { mockPathPlan } from './mock-plan';
import { mockAllBlocks } from './mock-blocks';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();

/** Returns an ISO timestamp N minutes in the past. */
const ts = (minutesAgo: number): string => new Date(now - minutesAgo * 60 * 1000).toISOString();

/** Returns an ISO timestamp N days in the past. */
const tsDay = (daysAgo: number): string => ts(daysAgo * 24 * 60);

// ---------------------------------------------------------------------------
// Individual witnesses (exported for component-level testing)
// ---------------------------------------------------------------------------

export const mockWitness1: Witness = {
	witness_id: 'witness-001',
	title: 'Jalan Rusak Jl. Mawar RT 05',
	summary:
		'Kondisi jalan rusak parah di Jl. Mawar RT 05 dengan lubang besar yang membahayakan warga. Sudah 3 bulan tidak ada perbaikan dari dinas terkait.',
	track_hint: 'tuntaskan',
	seed_hint: 'Keresahan',
	status: 'active',
	rahasia_level: 'L0',
	created_at: tsDay(60),
	updated_at: ts(10),
	created_by: 'u-001',
	member_count: 12,
	message_count: 28,
	unread_count: 3
};

export const mockWitness2: Witness = {
	witness_id: 'witness-002',
	title: 'Pembangunan Taman Warga RW 03',
	summary:
		'Aspirasi warga RW 03 untuk membangun taman bermain anak dan area hijau di lahan kosong milik kelurahan yang sudah lama tidak terpakai.',
	track_hint: 'wujudkan',
	seed_hint: 'Aspirasi',
	status: 'open',
	rahasia_level: 'L0',
	created_at: tsDay(45),
	updated_at: ts(120),
	created_by: 'u-001',
	member_count: 8,
	message_count: 15,
	unread_count: 0
};

export const mockWitness3: Witness = {
	witness_id: 'witness-003',
	title: 'Penyelidikan Limbah Pabrik Sungai Ciliwung',
	summary:
		'Dugaan pembuangan limbah ilegal ke Sungai Ciliwung oleh pabrik di kawasan industri. Warga mencium bau menyengat dan menemukan ikan mati massal.',
	track_hint: 'telusuri',
	seed_hint: 'Kejadian',
	status: 'active',
	rahasia_level: 'L1',
	created_at: tsDay(30),
	updated_at: ts(15),
	created_by: 'u-001',
	member_count: 5,
	message_count: 42,
	unread_count: 7
};

export const mockWitness4: Witness = {
	witness_id: 'witness-004',
	title: 'Festival Budaya Tahunan Kelurahan',
	summary:
		'Penyelenggaraan festival budaya tahunan untuk mempererat kebersamaan warga kelurahan. Menampilkan seni tradisional, pameran UMKM, dan lomba antar RT.',
	track_hint: 'rayakan',
	seed_hint: 'Rencana',
	status: 'resolved',
	rahasia_level: 'L0',
	created_at: tsDay(90),
	updated_at: tsDay(5),
	created_by: 'u-001',
	member_count: 20,
	message_count: 56,
	unread_count: 0
};

export const mockWitness5: Witness = {
	witness_id: 'witness-005',
	title: 'Musyawarah Anggaran RT 05 2025',
	summary:
		'Pertanyaan dan diskusi mengenai rencana penggunaan anggaran RT 05 tahun 2025. Warga ingin transparansi pengelolaan dana iuran bulanan.',
	track_hint: 'musyawarah',
	seed_hint: 'Pertanyaan',
	status: 'draft',
	rahasia_level: 'L0',
	created_at: ts(180),
	updated_at: ts(90),
	created_by: 'u-001',
	member_count: 3,
	message_count: 2,
	unread_count: 1
};

// ---------------------------------------------------------------------------
// All witnesses array
// ---------------------------------------------------------------------------

export const mockWitnesses: Witness[] = [
	mockWitness1,
	mockWitness2,
	mockWitness3,
	mockWitness4,
	mockWitness5
];

// ---------------------------------------------------------------------------
// Witness members (for the detailed witness)
// ---------------------------------------------------------------------------

export const mockWitnessMembers: WitnessMember[] = [
	{
		user_id: 'u-001',
		name: 'Ahmad Hidayat',
		role: 'pelapor',
		tier: 2,
		joined_at: tsDay(60)
	},
	{
		user_id: 'u-002',
		name: 'Sari Dewi',
		role: 'relawan',
		tier: 3,
		joined_at: tsDay(58)
	},
	{
		user_id: 'u-003',
		name: 'Budi Santoso',
		role: 'saksi',
		tier: 1,
		joined_at: tsDay(55)
	},
	{
		user_id: 'u-004',
		name: 'Rina Kartika',
		avatar_url: 'https://placehold.co/40x40/C05621/white?text=RK',
		role: 'koordinator',
		tier: 2,
		joined_at: tsDay(50)
	}
];

// ---------------------------------------------------------------------------
// Detailed witness (witness-001 with full aggregate data)
// ---------------------------------------------------------------------------

export const mockWitnessDetail: WitnessDetail = {
	...mockWitness1,
	messages: mockChatMessages,
	plan: mockPathPlan,
	blocks: mockAllBlocks,
	members: mockWitnessMembers
};

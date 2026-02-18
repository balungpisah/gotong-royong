/**
 * Mock chat message fixtures for the dev gallery.
 * A realistic thread covering all 7 ChatMessage types.
 */

import type {
	ChatMessage,
	UserMessage,
	AiCardMessage,
	DiffCardMessage,
	VoteCardMessage,
	SystemMessage,
	EvidenceMessage,
	GalangMessage
} from '$lib/types';
import { mockComputedBlock, mockListBlock } from './mock-blocks';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const baseWitnessId = 'witness-001';
const now = Date.now();

/** Returns an ISO timestamp N minutes in the past. */
const ts = (minutesAgo: number): string => new Date(now - minutesAgo * 60 * 1000).toISOString();

/** Returns an ISO timestamp N hours in the future. */
const tsAhead = (hoursAhead: number): string =>
	new Date(now + hoursAhead * 60 * 60 * 1000).toISOString();

// ---------------------------------------------------------------------------
// Authors
// ---------------------------------------------------------------------------

const author1 = {
	user_id: 'u-001',
	name: 'Ahmad Hidayat',
	tier: 2,
	role: 'pelapor'
};

const author2 = {
	user_id: 'u-002',
	name: 'Sari Dewi',
	tier: 3,
	role: 'relawan'
};

// ---------------------------------------------------------------------------
// Individual messages (exported for component-level testing)
// ---------------------------------------------------------------------------

export const mockSystemMessageJoined: SystemMessage = {
	message_id: 'msg-001',
	timestamp: ts(60),
	witness_id: baseWitnessId,
	type: 'system',
	subtype: 'member_joined',
	content: 'Ahmad Hidayat bergabung dalam saksi ini'
};

export const mockUserMessageSelf: UserMessage = {
	message_id: 'msg-002',
	timestamp: ts(55),
	witness_id: baseWitnessId,
	type: 'user',
	author: author1,
	is_self: true,
	content:
		'Saya mau laporkan kondisi jalan rusak di Jl. Mawar RT 05. Sudah 3 bulan tidak ada perbaikan.'
};

export const mockAiCardClassified: AiCardMessage = {
	message_id: 'msg-003',
	timestamp: ts(54),
	witness_id: baseWitnessId,
	type: 'ai_card',
	blocks: [mockComputedBlock],
	badge: 'classified',
	title: 'Klasifikasi Otomatis'
};

export const mockUserMessageOther: UserMessage = {
	message_id: 'msg-004',
	timestamp: ts(45),
	witness_id: baseWitnessId,
	type: 'user',
	author: author2,
	is_self: false,
	content: 'Saya juga tinggal di RT 05. Memang kondisinya sudah parah, terutama saat hujan.'
};

export const mockEvidenceMessage: EvidenceMessage = {
	message_id: 'msg-005',
	timestamp: ts(40),
	witness_id: baseWitnessId,
	type: 'evidence',
	author: author1,
	evidence_type: 'testimony',
	content: 'Lubang di depan rumah nomor 15 sudah menyebabkan 2 kecelakaan motor bulan lalu.',
	attachments: [
		{
			type: 'image',
			url: 'https://placehold.co/400x300/666/white?text=Bukti+Foto',
			alt: 'Foto jalan rusak'
		}
	]
};

export const mockDiffCardMessage: DiffCardMessage = {
	message_id: 'msg-006',
	timestamp: ts(35),
	witness_id: baseWitnessId,
	type: 'diff_card',
	diff: {
		diff_id: 'diff-001',
		target_type: 'list',
		target_id: 'list-001',
		summary: 'Ditambah 2 item, dicentang 1',
		evidence: ['Berdasarkan laporan foto dan keterangan warga'],
		items: [
			{
				operation: 'add',
				path: 'items[4]',
				label: 'Hubungi Dinas PU Kota',
				protected: false
			},
			{
				operation: 'modify',
				path: 'items[1].status',
				label: 'Koordinasi RT — tandai selesai',
				old_value: 'open',
				new_value: 'completed',
				protected: false
			},
			{
				operation: 'add',
				path: 'items[5]',
				label: 'Pasang rambu peringatan sementara',
				protected: false
			}
		],
		source: 'ai',
		generated_at: ts(35)
	}
};

export const mockVoteCardMessage: VoteCardMessage = {
	message_id: 'msg-007',
	timestamp: ts(20),
	witness_id: baseWitnessId,
	type: 'vote_card',
	block: {
		type: 'vote',
		id: 'vote-inline-001',
		question: 'Lanjutkan dengan penggalangan dana swadaya?',
		vote_type: 'standard',
		options: [
			{ id: 'y', label: 'Ya', count: 12 },
			{ id: 'n', label: 'Tidak', count: 3 }
		],
		quorum: 0.5,
		total_eligible: 30,
		total_voted: 15,
		duration_hours: 48,
		ends_at: tsAhead(24),
		user_voted: true
	}
};

export const mockSystemMessagePhase: SystemMessage = {
	message_id: 'msg-008',
	timestamp: ts(10),
	witness_id: baseWitnessId,
	type: 'system',
	subtype: 'phase_completed',
	content: 'Fase "Pengumpulan Bukti" selesai'
};

export const mockGalangMessage: GalangMessage = {
	message_id: 'msg-009',
	timestamp: ts(5),
	witness_id: baseWitnessId,
	type: 'galang',
	subtype: 'contribution',
	content: 'Ibu Sari menyumbang untuk perbaikan jalan',
	amount: 500000,
	currency: 'IDR'
};

export const mockAiCardRingkasan: AiCardMessage = {
	message_id: 'msg-010',
	timestamp: ts(2),
	witness_id: baseWitnessId,
	type: 'ai_card',
	blocks: [mockListBlock],
	badge: 'ringkasan',
	title: 'Ringkasan Kemajuan'
};

// ---------------------------------------------------------------------------
// Full thread (all 7 message types)
// ---------------------------------------------------------------------------

export const mockChatMessages: ChatMessage[] = [
	mockSystemMessageJoined, // system
	mockUserMessageSelf, // user (is_self)
	mockAiCardClassified, // ai_card
	mockUserMessageOther, // user (other)
	mockEvidenceMessage, // evidence
	mockDiffCardMessage, // diff_card
	mockVoteCardMessage, // vote_card
	mockSystemMessagePhase, // system
	mockGalangMessage, // galang
	mockAiCardRingkasan // ai_card (with list block)
];

import type { WitnessService, Paginated } from '../types';
import type {
	Witness,
	WitnessDetail,
	ChatMessage,
	PathPlan,
	DiffResponse,
	UserMessage
} from '$lib/types';
import { mockWitnesses, mockWitnessDetail, mockPathPlan } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockWitnessService implements WitnessService {
	private witnesses = [...mockWitnesses];
	private detail = { ...mockWitnessDetail, messages: [...mockWitnessDetail.messages] };

	async list(opts?: {
		status?: string;
		cursor?: string;
		limit?: number;
	}): Promise<Paginated<Witness>> {
		await delay();
		let items = this.witnesses;
		if (opts?.status) {
			items = items.filter((w) => w.status === opts.status);
		}
		const limit = opts?.limit ?? 20;
		return { items: items.slice(0, limit), total: items.length };
	}

	async get(witnessId: string): Promise<WitnessDetail> {
		await delay();
		if (witnessId === this.detail.witness_id) {
			return this.detail;
		}
		// For any other ID, return the detail with modified ID
		return { ...this.detail, witness_id: witnessId };
	}

	async getMessages(
		witnessId: string,
		opts?: { cursor?: string; limit?: number }
	): Promise<Paginated<ChatMessage>> {
		await delay();
		const messages = this.detail.messages;
		const limit = opts?.limit ?? 50;
		return { items: messages.slice(0, limit), total: messages.length };
	}

	async sendMessage(
		witnessId: string,
		content: string,
		_attachments?: File[]
	): Promise<ChatMessage> {
		await delay(300);
		const newMessage: UserMessage = {
			message_id: `msg-${Date.now()}`,
			timestamp: new Date().toISOString(),
			witness_id: witnessId,
			type: 'user',
			author: {
				user_id: 'u-001',
				name: 'Ahmad Hidayat',
				tier: 2,
				role: 'pelapor'
			},
			is_self: true,
			content
		};
		this.detail.messages = [...this.detail.messages, newMessage];
		return newMessage;
	}

	async getPlan(witnessId: string): Promise<PathPlan | null> {
		await delay();
		return mockPathPlan;
	}

	async respondToDiff(witnessId: string, diffId: string, response: DiffResponse): Promise<void> {
		await delay();
		console.log('[MockWitnessService] respondToDiff:', { witnessId, diffId, response });
	}

	async castVote(witnessId: string, voteId: string, optionId: string): Promise<void> {
		await delay();
		console.log('[MockWitnessService] castVote:', { witnessId, voteId, optionId });
	}
}

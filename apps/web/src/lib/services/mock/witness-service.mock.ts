import type { WitnessService, Paginated } from '../types';
import type {
	Witness,
	WitnessDetail,
	WitnessCreateInput,
	WitnessCloseReason,
	WitnessStatus,
	ChatMessage,
	PathPlan,
	DiffResponse,
	ContentSignal,
	UserMessage
} from '$lib/types';
import { mockWitnesses, mockWitnessDetail, mockPathPlan } from '$lib/fixtures';
import { MockSignalService } from './signal-service.mock';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockWitnessService implements WitnessService {
	private witnesses = [...mockWitnesses];
	private detail = { ...mockWitnessDetail, messages: [...mockWitnessDetail.messages] };

	/** Optional reference to MockSignalService for resolution demo. */
	private signalService?: MockSignalService;

	/** Inject MockSignalService for closeWitness resolution demo. */
	setSignalService(signalService: MockSignalService): void {
		this.signalService = signalService;
	}

	async create(input: WitnessCreateInput): Promise<WitnessDetail> {
		await delay(400);
		const now = new Date().toISOString();
		const witnessId = `w-${Date.now()}`;
		const messages: ChatMessage[] = [];
		const title = `Saksi ${input.triage_session_id.slice(-6)}`;
		const summary = 'Draft triase sudah diubah menjadi saksi mock.';

		const witness: Witness = {
			witness_id: witnessId,
			title,
			summary,
			status: 'open',
			rahasia_level: 'L0',
			created_at: now,
			updated_at: now,
			created_by: 'u-001',
			member_count: 1,
			message_count: messages.length,
			unread_count: 0
		};

		const detail: WitnessDetail = {
			...witness,
			messages,
			plan: null,
			blocks: [],
			members: [
				{
					user_id: 'u-001',
					name: 'Ahmad Hidayat',
					role: 'pelapor',
					tier: 2,
					joined_at: now
				}
			]
		};

		// Prepend to internal list
		this.witnesses = [witness, ...this.witnesses];

		return detail;
	}

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
		attachments?: File[]
	): Promise<ChatMessage> {
		await delay(300);
		const messageAttachments = attachments?.map((f) => ({
			type: (f.type.startsWith('image/')
				? 'image'
				: f.type.startsWith('video/')
					? 'video'
					: 'audio') as 'image' | 'video' | 'audio',
			url: URL.createObjectURL(f),
			alt: f.name
		}));
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
			content,
			attachments: messageAttachments?.length ? messageAttachments : undefined
		};
		this.detail.messages = [...this.detail.messages, newMessage];
		return newMessage;
	}

	async getPlan(_witnessId: string): Promise<PathPlan | null> {
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

	/**
	 * Close a witness with a reason — demo flow for signal resolution.
	 * Updates witness status and triggers signal resolution via MockSignalService.
	 */
	async closeWitness(
		witnessId: string,
		closeReason: WitnessCloseReason
	): Promise<{
		status: WitnessStatus;
		close_reason: WitnessCloseReason;
		resolved_signals: ContentSignal[];
	}> {
		await delay(300);

		const terminalStatus: WitnessStatus = closeReason === 'selesai' ? 'resolved' : 'closed';

		// Update internal witness list
		this.witnesses = this.witnesses.map((w) =>
			w.witness_id === witnessId
				? {
						...w,
						status: terminalStatus,
						close_reason: closeReason,
						updated_at: new Date().toISOString()
					}
				: w
		);

		// Update detail if it matches
		if (this.detail.witness_id === witnessId) {
			this.detail = {
				...this.detail,
				status: terminalStatus,
				close_reason: closeReason,
				updated_at: new Date().toISOString()
			};
		}

		// Trigger signal resolution if signal service is wired
		let resolved_signals: ContentSignal[] = [];
		if (this.signalService) {
			resolved_signals = await this.signalService.simulateResolution(witnessId, closeReason);
		}

		console.log('[MockWitnessService] closeWitness:', {
			witnessId,
			closeReason,
			terminalStatus,
			resolved_signals
		});

		return { status: terminalStatus, close_reason: closeReason, resolved_signals };
	}
}

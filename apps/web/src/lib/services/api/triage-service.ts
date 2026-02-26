import type { ApiClient } from '$lib/api';
import type {
	ContextBarState,
	EntryRoute,
	TriageConversationBlockId,
	TriageKind,
	TriageResult,
	TriageStructuredBlockId,
	TriageStatus
} from '$lib/types';
import type { TriageService } from '../types';

type JsonRecord = Record<string, unknown>;

interface StartTriageResponse {
	session_id?: string;
	result?: unknown;
}

interface ContinueTriageResponse {
	result?: unknown;
}

const BAR_STATES = new Set<ContextBarState>([
	'listening',
	'probing',
	'leaning',
	'ready',
	'vault-ready',
	'siaga-ready',
	'split-ready',
	'manual'
]);

const ENTRY_ROUTES = new Set<EntryRoute>([
	'komunitas',
	'vault',
	'siaga',
	'catatan_komunitas',
	'kelola'
]);
const TRIAGE_STATUSES = new Set<TriageStatus>(['draft', 'final']);
const TRIAGE_KINDS = new Set<TriageKind>(['witness', 'data']);
const TRIAGE_SCHEMA_VERSION = 'triage.v1';
const TRIAGE_CONVERSATION_BLOCKS = new Set<TriageConversationBlockId>([
	'chat_message',
	'ai_inline_card',
	'diff_card',
	'vote_card',
	'moderation_hold_card',
	'duplicate_detection_card',
	'credit_nudge_card'
]);
const TRIAGE_STRUCTURED_BLOCKS = new Set<TriageStructuredBlockId>([
	'list',
	'document',
	'form',
	'computed',
	'display',
	'vote',
	'reference'
]);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value : undefined;

const readStringArray = (value: unknown): string[] | undefined => {
	if (!Array.isArray(value)) return undefined;
	const strings = value
		.map((entry) => asString(entry))
		.filter((entry): entry is string => entry !== undefined);
	return strings.length === value.length ? strings : undefined;
};

const readBlocks = (value: unknown): TriageResult['blocks'] | undefined => {
	if (!isRecord(value)) return undefined;
	const conversationRaw = readStringArray(value.conversation);
	const structuredRaw = readStringArray(value.structured);
	if (!conversationRaw || !structuredRaw) return undefined;

	const conversation = conversationRaw.filter((item): item is TriageConversationBlockId =>
		TRIAGE_CONVERSATION_BLOCKS.has(item as TriageConversationBlockId)
	);
	const structured = structuredRaw.filter((item): item is TriageStructuredBlockId =>
		TRIAGE_STRUCTURED_BLOCKS.has(item as TriageStructuredBlockId)
	);

	if (conversation.length !== conversationRaw.length || structured.length !== structuredRaw.length) {
		return undefined;
	}

	return { conversation, structured };
};

const readResult = (raw: unknown): TriageResult | undefined => {
	if (!isRecord(raw)) return undefined;
	const barState = asString(raw.bar_state);
	const route = asString(raw.route);
	const schemaVersion = asString(raw.schema_version);
	const status = asString(raw.status);
	const kind = asString(raw.kind);
	if (!barState || !route || !schemaVersion || !status || !kind) return undefined;
	if (!BAR_STATES.has(barState as ContextBarState)) return undefined;
	if (!ENTRY_ROUTES.has(route as EntryRoute)) return undefined;
	if (schemaVersion !== TRIAGE_SCHEMA_VERSION) return undefined;
	if (!TRIAGE_STATUSES.has(status as TriageStatus)) return undefined;
	if (!TRIAGE_KINDS.has(kind as TriageKind)) return undefined;
	return {
		...(raw as unknown as TriageResult),
		blocks: readBlocks(raw.blocks)
	};
};

const toAttachmentPayload = (attachments?: File[]) => {
	if (!attachments || attachments.length === 0) return undefined;
	return attachments.map((file) => ({
		name: file.name,
		mime_type: file.type || 'application/octet-stream',
		size_bytes: file.size
	}));
};

interface ApiTriageServiceOptions {
	allowMockFallback?: boolean;
}

export class ApiTriageService implements TriageService {
	private readonly client: ApiClient;
	private readonly fallback: TriageService;
	private readonly allowMockFallback: boolean;
	private activeSessionId: string | null = null;

	constructor(client: ApiClient, fallback: TriageService, options: ApiTriageServiceOptions = {}) {
		this.client = client;
		this.fallback = fallback;
		this.allowMockFallback = options.allowMockFallback ?? true;
	}

	private fallbackOrThrow<T>(fallback: () => Promise<T>, error?: unknown): Promise<T> {
		if (this.allowMockFallback) {
			return fallback();
		}
		if (error instanceof Error) {
			throw error;
		}
		throw new Error('Mock fallback disabled for triage service');
	}

	async startTriage(content: string, attachments?: File[]): Promise<TriageResult> {
		try {
			const response = await this.client.post<StartTriageResponse>('/triage/sessions', {
				body: {
					content,
					attachments: toAttachmentPayload(attachments)
				}
			});
			const result = readResult(response.result);
			const sessionId = asString(response.session_id);
			if (!result || !sessionId) {
				throw new Error('invalid triage start response');
			}
			this.activeSessionId = sessionId;
			return {
				...result,
				session_id: sessionId
			};
		} catch (error) {
			const fallbackResult = await this.fallbackOrThrow(
				() => this.fallback.startTriage(content, attachments),
				error
			);
			this.activeSessionId = fallbackResult.session_id ?? this.activeSessionId;
			return fallbackResult;
		}
	}

	async updateTriage(
		sessionId: string,
		answer: string,
		attachments?: File[]
	): Promise<TriageResult> {
		const targetSessionId = sessionId.trim() || this.activeSessionId;
		if (!targetSessionId) {
			return this.fallbackOrThrow(() => this.fallback.updateTriage(sessionId, answer, attachments));
		}

		try {
			const response = await this.client.post<ContinueTriageResponse>(
				`/triage/sessions/${encodeURIComponent(targetSessionId)}/messages`,
				{
					body: {
						answer,
						attachments: toAttachmentPayload(attachments)
					}
				}
			);
			const result = readResult(response.result);
			if (!result) {
				throw new Error('invalid triage update response');
			}
			this.activeSessionId = targetSessionId;
			return {
				...result,
				session_id: targetSessionId
			};
		} catch (error) {
			return this.fallbackOrThrow(
				() => this.fallback.updateTriage(sessionId, answer, attachments),
				error
			);
		}
	}
}

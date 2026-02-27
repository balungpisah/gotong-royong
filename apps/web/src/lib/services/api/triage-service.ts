import type { ApiClient } from '$lib/api';
import type {
	Block,
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

export const TRIAGE_FALLBACK_FLAG_KEY = '__gr_triage_api_fallback_used__';

type TriageFallbackOperation = 'start' | 'update' | 'update_no_session';

interface TriageFallbackDiagnostics {
	total: number;
	by_operation: Record<TriageFallbackOperation, number>;
	last_error_message?: string;
}

const createFallbackDiagnostics = (): TriageFallbackDiagnostics => ({
	total: 0,
	by_operation: {
		start: 0,
		update: 0,
		update_no_session: 0
	}
});

let fallbackDiagnostics = createFallbackDiagnostics();

const fallbackFlagStore = (): Record<string, unknown> =>
	(globalThis as unknown as Record<string, unknown>);

const readErrorMessage = (error: unknown): string | undefined =>
	error instanceof Error && error.message.trim().length > 0 ? error.message : undefined;

const markFallbackUsage = (operation: TriageFallbackOperation, error?: unknown) => {
	fallbackDiagnostics.total += 1;
	fallbackDiagnostics.by_operation[operation] += 1;
	const message = readErrorMessage(error);
	if (message) {
		fallbackDiagnostics.last_error_message = message;
	}
	fallbackFlagStore()[TRIAGE_FALLBACK_FLAG_KEY] = true;
};

export const getTriageFallbackDiagnostics = (): TriageFallbackDiagnostics => ({
	total: fallbackDiagnostics.total,
	by_operation: { ...fallbackDiagnostics.by_operation },
	last_error_message: fallbackDiagnostics.last_error_message
});

export const resetTriageFallbackDiagnostics = () => {
	fallbackDiagnostics = createFallbackDiagnostics();
	delete fallbackFlagStore()[TRIAGE_FALLBACK_FLAG_KEY];
};

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
const SOURCE_TAGS = new Set(['ai', 'human', 'system']);
const LIST_ITEM_STATUSES = new Set(['open', 'completed', 'blocked', 'skipped']);
const LIST_DISPLAYS = new Set(['checklist', 'table', 'timeline', 'gallery']);
const FORM_FIELD_TYPES = new Set(['text', 'number', 'date', 'select', 'textarea', 'toggle', 'file']);
const COMPUTED_DISPLAYS = new Set(['progress', 'status', 'score', 'counter', 'confidence']);
const VOTE_TYPES = new Set(['standard', 'weighted', 'quorum_1_5x', 'consensus']);
const REFERENCE_TYPES = new Set(['seed', 'plan', 'checkpoint', 'document']);
const DIFF_OPERATIONS = new Set(['add', 'remove', 'modify', 'reorder']);
const DIFF_TARGET_TYPES = new Set(['list', 'document', 'form', 'checkpoint', 'phase']);
const AI_BADGE_VARIANTS = new Set([
	'classified',
	'suggested',
	'stalled',
	'dampak',
	'ringkasan',
	'duplikat'
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

const isStringArray = (value: unknown): value is string[] =>
	Array.isArray(value) && value.every((item) => typeof item === 'string');

const isSourceMeta = (value: unknown): boolean => {
	if (!isRecord(value)) return false;
	return SOURCE_TAGS.has(String(value.source)) && isStringArray(value.locked_fields);
};

const isListItem = (value: unknown): boolean => {
	if (!isRecord(value)) return false;
	if (!asString(value.id) || !asString(value.label)) return false;
	if (!LIST_ITEM_STATUSES.has(String(value.status))) return false;
	if (!isSourceMeta(value)) return false;
	if (value.children !== undefined) {
		if (!Array.isArray(value.children)) return false;
		if (!value.children.every((child) => isListItem(child))) return false;
	}
	return true;
};

const isBlock = (value: unknown): value is Block => {
	if (!isRecord(value)) return false;
	const type = asString(value.type);
	const id = asString(value.id);
	if (!type || !id || !TRIAGE_STRUCTURED_BLOCKS.has(type as TriageStructuredBlockId)) return false;

	switch (type) {
		case 'list':
			return LIST_DISPLAYS.has(String(value.display)) && Array.isArray(value.items) && value.items.every(isListItem);
		case 'document':
			return (
				Array.isArray(value.sections) &&
				value.sections.every(
					(section) =>
						isRecord(section) &&
						!!asString(section.id) &&
						!!asString(section.content) &&
						isSourceMeta(section)
				)
			);
		case 'form':
			return (
				Array.isArray(value.fields) &&
				value.fields.every(
					(field) =>
						isRecord(field) &&
						!!asString(field.id) &&
						!!asString(field.label) &&
						FORM_FIELD_TYPES.has(String(field.field_type)) &&
						typeof field.protected === 'boolean' &&
						isSourceMeta(field)
				)
			);
		case 'computed':
			return (
				COMPUTED_DISPLAYS.has(String(value.display)) &&
				!!asString(value.label) &&
				typeof value.value === 'number'
			);
		case 'display':
			return !!asString(value.title) && !!asString(value.content);
		case 'vote':
			return (
				!!asString(value.question) &&
				VOTE_TYPES.has(String(value.vote_type)) &&
				Array.isArray(value.options) &&
				value.options.every(
					(option) =>
						isRecord(option) &&
						!!asString(option.id) &&
						!!asString(option.label) &&
						typeof option.count === 'number'
				) &&
				typeof value.quorum === 'number' &&
				typeof value.total_eligible === 'number' &&
				typeof value.total_voted === 'number' &&
				typeof value.duration_hours === 'number' &&
				!!asString(value.ends_at)
			);
		case 'reference':
			return (
				!!asString(value.ref_id) &&
				REFERENCE_TYPES.has(String(value.ref_type)) &&
				!!asString(value.title)
			);
		default:
			return false;
	}
};

const readStructuredPayload = (value: unknown): Block[] | undefined => {
	if (!Array.isArray(value)) return undefined;
	const blocks = value.filter((item): item is Block => isBlock(item));
	return blocks.length === value.length ? blocks : undefined;
};

const isChatMessageBase = (value: unknown): boolean =>
	isRecord(value) && !!asString(value.message_id) && !!asString(value.timestamp) && !!asString(value.witness_id);

const isAiCardMessage = (value: unknown): boolean => {
	if (!isRecord(value) || value.type !== 'ai_card') return false;
	if (!isChatMessageBase(value) || !Array.isArray(value.blocks) || !value.blocks.every(isBlock)) return false;
	if (value.badge !== undefined && !AI_BADGE_VARIANTS.has(String(value.badge))) return false;
	return true;
};

const isDiffCardMessage = (value: unknown): boolean => {
	if (!isRecord(value) || value.type !== 'diff_card') return false;
	if (!isChatMessageBase(value)) return false;
	if (!isRecord(value.diff)) return false;
	const diff = value.diff;
	if (
		!asString(diff.diff_id) ||
		!DIFF_TARGET_TYPES.has(String(diff.target_type)) ||
		!asString(diff.target_id) ||
		!asString(diff.summary) ||
		diff.source !== 'ai' ||
		!asString(diff.generated_at) ||
		!Array.isArray(diff.items)
	) {
		return false;
	}

	return diff.items.every(
		(item) =>
			isRecord(item) &&
			DIFF_OPERATIONS.has(String(item.operation)) &&
			!!asString(item.path) &&
			!!asString(item.label) &&
			typeof item.protected === 'boolean'
	);
};

const isVoteCardMessage = (value: unknown): boolean => {
	if (!isRecord(value) || value.type !== 'vote_card') return false;
	if (!isChatMessageBase(value)) return false;
	return isBlock(value.block) && value.block.type === 'vote';
};

const readConversationPayload = (value: unknown): TriageResult['conversation_payload'] | undefined => {
	if (!Array.isArray(value)) return undefined;
	const parsed = value.filter(
		(item): item is NonNullable<TriageResult['conversation_payload']>[number] =>
			isAiCardMessage(item) || isDiffCardMessage(item) || isVoteCardMessage(item)
	);
	return parsed.length === value.length ? parsed : undefined;
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
		blocks: readBlocks(raw.blocks),
		structured_payload: readStructuredPayload(raw.structured_payload),
		conversation_payload: readConversationPayload(raw.conversation_payload)
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

	private fallbackOrThrow<T>(
		fallback: () => Promise<T>,
		operation: TriageFallbackOperation,
		error?: unknown
	): Promise<T> {
		if (this.allowMockFallback) {
			markFallbackUsage(operation, error);
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
				'start',
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
			return this.fallbackOrThrow(
				() => this.fallback.updateTriage(sessionId, answer, attachments),
				'update_no_session'
			);
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
				'update',
				error
			);
		}
	}
}

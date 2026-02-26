import type { ApiClient } from '$lib/api';
import type {
	ChatMessage,
	DiffResponse,
	PathPlan,
	RahasiaLevel,
	SeedHint,
	SystemMessage,
	UserMessage,
	Witness,
	WitnessCreateInput,
	WitnessDetail,
	WitnessMember,
	WitnessStatus
} from '$lib/types';
import type { Paginated, WitnessService } from '../types';

interface ApiChatThread {
	thread_id: string;
	scope_id: string;
}

interface ApiFeedItem {
	feed_id: string;
	source_id: string;
	actor_id: string;
	title: string;
	summary?: string | null;
	privacy_level?: string | null;
	occurred_at_ms: number;
	created_at_ms: number;
	participant_ids?: string[];
	payload?: unknown;
}

interface ApiFeedWitnessStreamItem {
	kind: 'witness';
	data: ApiFeedItem;
}

interface ApiFeedSystemStreamItem {
	kind: 'system';
	data: unknown;
}

type ApiFeedStreamItem = ApiFeedWitnessStreamItem | ApiFeedSystemStreamItem;

interface ApiPagedFeed {
	items: Array<ApiFeedItem | ApiFeedStreamItem>;
	next_cursor?: string | null;
}

interface ApiChatMessage {
	thread_id: string;
	message_id: string;
	author_id: string;
	author?: {
		user_id?: string;
		name?: string;
		avatar_url?: string | null;
		tier?: number | null;
		role?: string | null;
	} | null;
	body: string;
	attachments: unknown[];
	created_at_ms: number;
}

interface ApiChatAttachmentUploadResponse {
	attachment_id: string;
	file_name: string;
	mime_type: string;
	size_bytes: number;
	media_type: 'image' | 'video' | 'audio';
	url: string;
	expires_at_ms: number;
}

interface ApiAuthMe {
	user_id?: string;
}

interface ApiChatMember {
	user_id: string;
	role: 'owner' | 'admin' | 'member';
	joined_at_ms: number;
	left_at_ms?: number | null;
}

interface ApiWitnessCreateResponse {
	witness_id?: string;
	title?: string;
	summary?: string | null;
	track_hint?: string | null;
	seed_hint?: string | null;
	rahasia_level?: string | null;
	author_id?: string;
	created_at_ms?: number;
}

type JsonRecord = Record<string, unknown>;

const WITNESS_STATUSES = new Set<WitnessStatus>(['draft', 'open', 'active', 'resolved', 'closed']);
const SEED_HINTS = new Set<SeedHint>([
	'Keresahan',
	'Aspirasi',
	'Kejadian',
	'Rencana',
	'Pertanyaan'
]);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const isApiFeedItem = (value: unknown): value is ApiFeedItem => {
	if (!isRecord(value)) return false;
	return Boolean(
		asString(value.feed_id) &&
			asString(value.source_id) &&
			asString(value.actor_id) &&
			asString(value.title)
	);
};

const isFeedWitnessStreamItem = (value: unknown): value is ApiFeedWitnessStreamItem => {
	if (!isRecord(value)) return false;
	return value.kind === 'witness' && isApiFeedItem(value.data);
};

const mapPrivacyToRahasia = (privacy: string | null | undefined): RahasiaLevel => {
	const normalized = (privacy ?? '').trim().toLowerCase();
	if (!normalized || normalized === 'public' || normalized === 'open') return 'L0';
	if (normalized === 'l1' || normalized === 'private') return 'L1';
	if (normalized === 'l2' || normalized === 'secret') return 'L2';
	if (normalized === 'l3' || normalized === 'very_secret') return 'L3';
	return 'L1';
};

const toIso = (timestampMs: number | undefined) =>
	new Date(timestampMs && Number.isFinite(timestampMs) ? timestampMs : Date.now()).toISOString();

const mapMemberRole = (role: ApiChatMember['role']): WitnessMember['role'] => {
	switch (role) {
		case 'owner':
		case 'admin':
			return 'koordinator';
		default:
			return 'relawan';
	}
};

const parseCursor = (
	cursor?: string
): { since_created_at_ms?: number; since_message_id?: string } => {
	if (!cursor) {
		return {};
	}

	const [createdAtPart, messageIdPart] = cursor.split(':', 2);
	const createdAt = Number(createdAtPart);
	if (!Number.isFinite(createdAt) || !messageIdPart) {
		return {};
	}

	return {
		since_created_at_ms: createdAt,
		since_message_id: messageIdPart
	};
};

const toCursor = (message: ApiChatMessage) => `${message.created_at_ms}:${message.message_id}`;

const mapAttachment = (
	value: unknown
): NonNullable<UserMessage['attachments']>[number] | undefined => {
	if (!value || typeof value !== 'object' || Array.isArray(value)) {
		return undefined;
	}

	const record = value as Record<string, unknown>;
	const url = typeof record.url === 'string' ? record.url : undefined;
	if (!url) {
		return undefined;
	}

	const rawType = typeof record.type === 'string' ? record.type.toLowerCase() : '';
	const type = rawType === 'video' || rawType === 'audio' ? rawType : 'image';
	const alt = typeof record.alt === 'string' ? record.alt : undefined;
	return { type, url, alt };
};

const mapToUserMessage = (
	message: ApiChatMessage,
	witnessId: string,
	currentUserId?: string
): ChatMessage => {
	const attachments = Array.isArray(message.attachments)
		? message.attachments
				.map(mapAttachment)
				.filter((attachment): attachment is NonNullable<UserMessage['attachments']>[number] =>
					Boolean(attachment)
				)
		: [];

	const isSelf = Boolean(currentUserId && message.author_id === currentUserId);
	const authorSnapshot = isRecord(message.author) ? message.author : undefined;
	const authorName = isSelf ? 'Saya' : (asString(authorSnapshot?.name) ?? message.author_id);
	return {
		message_id: message.message_id,
		timestamp: new Date(message.created_at_ms).toISOString(),
		witness_id: witnessId,
		type: 'user',
		author: {
			user_id: message.author_id,
			name: authorName,
			avatar_url: asString(authorSnapshot?.avatar_url),
			tier: asNumber(authorSnapshot?.tier),
			role: asString(authorSnapshot?.role)
		},
		is_self: isSelf,
		content: message.body,
		attachments: attachments.length ? attachments : undefined
	} satisfies UserMessage;
};

const makeRequestId = (prefix: string) => {
	const randomPart =
		typeof globalThis.crypto?.randomUUID === 'function'
			? globalThis.crypto.randomUUID()
			: `${Date.now()}-${Math.random().toString(16).slice(2)}`;
	return `${prefix}-${randomPart}`;
};

const buildCreatedWitnessMessages = (
	input: WitnessCreateInput,
	witnessId: string,
	nowIso: string
): ChatMessage[] =>
	input.triage_messages.map((message, index) => {
		if (message.role === 'user') {
			return {
				message_id: `msg-${witnessId}-${index}`,
				timestamp: nowIso,
				witness_id: witnessId,
				type: 'user',
				author: {
					user_id: 'me',
					name: 'Saya',
					role: 'pelapor'
				},
				is_self: true,
				content: message.text
			} satisfies UserMessage;
		}

		return {
			message_id: `msg-${witnessId}-${index}`,
			timestamp: nowIso,
			witness_id: witnessId,
			type: 'system',
			subtype: 'plan_updated',
			content: message.text
		} satisfies SystemMessage;
	});

interface ApiWitnessServiceOptions {
	allowMockFallback?: boolean;
}

export class ApiWitnessService implements WitnessService {
	private readonly client: ApiClient;
	private readonly fallback: WitnessService;
	private readonly allowMockFallback: boolean;
	private readonly scopeToThread = new Map<string, string>();
	private currentUserIdPromise: Promise<string | undefined> | null = null;

	constructor(client: ApiClient, fallback: WitnessService, options: ApiWitnessServiceOptions = {}) {
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
		throw new Error('Mock fallback disabled for witness service');
	}

	async create(input: WitnessCreateInput): Promise<WitnessDetail> {
		try {
			const response = await this.client.post<ApiWitnessCreateResponse>('/witnesses', {
				headers: this.idempotencyHeaders('witness-create'),
				body: {
					title: input.title,
					summary: input.summary,
					route: input.route,
					track_hint: input.track_hint,
					seed_hint: input.seed_hint,
					rahasia_level: input.rahasia_level,
					triage_result: input.triage_result,
					triage_messages: input.triage_messages
				}
			});

			const witnessId = asString(response.witness_id);
			if (!witnessId) {
				throw new Error('invalid witness create response');
			}

			const detail = this.buildCreatedWitnessDetail(witnessId, input, response);
			void this.ensureThread(detail.witness_id, true).catch(() => undefined);
			return detail;
		} catch (error) {
			const detail = await this.fallbackOrThrow(() => this.fallback.create(input), error);
			void this.ensureThread(detail.witness_id, true).catch(() => undefined);
			return detail;
		}
	}

	async list(opts?: {
		status?: WitnessStatus;
		cursor?: string;
		limit?: number;
	}): Promise<Paginated<Witness>> {
		const response = await this.client.get<ApiPagedFeed>('/feed', {
			query: {
				cursor: opts?.cursor,
				limit: opts?.limit
			}
		});
		const deduped = this.mapFeedToWitnesses(this.extractFeedItems(response.items));
		const filtered = opts?.status ? deduped.filter((item) => item.status === opts.status) : deduped;
		return {
			items: filtered,
			total: filtered.length,
			cursor: response.next_cursor ?? undefined
		};
	}

	async get(witnessId: string): Promise<WitnessDetail> {
		const summary = await this.fetchWitnessSummary(witnessId);
		const messages = await this.getMessages(witnessId, { limit: 200 }).catch(() => ({
			items: [] as ChatMessage[],
			total: 0
		}));
		const members = await this.fetchWitnessMembers(witnessId).catch(() => []);
		const resolvedMembers =
			members.length > 0
				? members
				: [
						{
							user_id: summary.created_by,
							name: summary.created_by,
							role: 'pelapor' as const,
							joined_at: summary.created_at
						}
					];
		const latestMessageAt =
			messages.items.length > 0
				? messages.items[messages.items.length - 1].timestamp
				: summary.updated_at;
		return {
			...summary,
			member_count: Math.max(summary.member_count, resolvedMembers.length),
			message_count: Math.max(summary.message_count, messages.total),
			updated_at: latestMessageAt,
			messages: messages.items,
			plan: null,
			blocks: [],
			members: resolvedMembers
		};
	}

	async getMessages(
		witnessId: string,
		opts?: { cursor?: string; limit?: number }
	): Promise<Paginated<ChatMessage>> {
		const threadId = await this.ensureThread(witnessId, false);
		if (!threadId) {
			return { items: [], total: 0 };
		}

		await this.joinThread(threadId);
		const cursor = parseCursor(opts?.cursor);
		const rawMessages = await this.client.get<ApiChatMessage[]>(
			`/chat/threads/${threadId}/messages/poll`,
			{
				query: {
					since_created_at_ms: cursor.since_created_at_ms,
					since_message_id: cursor.since_message_id,
					limit: opts?.limit
				}
			}
		);

		const currentUserId = await this.resolveCurrentUserId();
		const items = rawMessages.map((message) => mapToUserMessage(message, witnessId, currentUserId));
		const nextCursor =
			rawMessages.length > 0 ? toCursor(rawMessages[rawMessages.length - 1]) : undefined;

		return {
			items,
			total: rawMessages.length,
			cursor: nextCursor
		};
	}

	async sendMessage(
		witnessId: string,
		content: string,
		attachments?: File[]
	): Promise<ChatMessage> {
		const threadId = await this.ensureThread(witnessId, true);
		if (!threadId) {
			throw new Error('Gagal membuat thread chat');
		}

		const uploadedAttachments = await this.uploadOutgoingAttachments(attachments);
		await this.joinThread(threadId);
		const response = await this.client.post<ApiChatMessage>(
			`/chat/threads/${threadId}/messages/send`,
			{
				headers: this.idempotencyHeaders('chat-send'),
				body: {
					body: content,
					attachments: uploadedAttachments
				}
			}
		);

		const currentUserId = await this.resolveCurrentUserId();
		return mapToUserMessage(response, witnessId, currentUserId);
	}

	async getPlan(witnessId: string): Promise<PathPlan | null> {
		return this.fallbackOrThrow(() => this.fallback.getPlan(witnessId));
	}

	async respondToDiff(witnessId: string, diffId: string, response: DiffResponse): Promise<void> {
		return this.fallbackOrThrow(() => this.fallback.respondToDiff(witnessId, diffId, response));
	}

	async castVote(witnessId: string, voteId: string, optionId: string): Promise<void> {
		return this.fallbackOrThrow(() => this.fallback.castVote(witnessId, voteId, optionId));
	}

	private async ensureThread(
		scopeId: string,
		createIfMissing: boolean
	): Promise<string | undefined> {
		const cached = this.scopeToThread.get(scopeId);
		if (cached) {
			return cached;
		}

		const threads = await this.client.get<ApiChatThread[]>('/chat/threads', {
			query: { scope_id: scopeId }
		});
		const existing = threads.find((thread) => thread.scope_id === scopeId);
		if (existing) {
			this.scopeToThread.set(scopeId, existing.thread_id);
			return existing.thread_id;
		}

		if (!createIfMissing) {
			return undefined;
		}

		const created = await this.client.post<ApiChatThread>('/chat/threads', {
			headers: this.idempotencyHeaders('chat-thread-create'),
			body: {
				scope_id: scopeId,
				privacy_level: 'public'
			}
		});
		this.scopeToThread.set(scopeId, created.thread_id);
		return created.thread_id;
	}

	private async joinThread(threadId: string): Promise<void> {
		await this.client.post(`/chat/threads/${threadId}/join`, {
			headers: this.idempotencyHeaders('chat-join')
		});
	}

	private async uploadOutgoingAttachments(
		attachments?: File[]
	): Promise<Array<Record<string, unknown>>> {
		if (!attachments || attachments.length === 0) {
			return [];
		}

		const uploads = await Promise.all(
			attachments.map(async (attachment) => {
				const formData = new FormData();
				formData.append('file', attachment, attachment.name);
				const uploaded = await this.client.post<ApiChatAttachmentUploadResponse>(
					'/chat/attachments/upload',
					{
						headers: this.idempotencyHeaders('chat-attachment-upload'),
						body: formData
					}
				);
				return {
					attachment_id: uploaded.attachment_id,
					name: uploaded.file_name,
					mime_type: uploaded.mime_type,
					size_bytes: uploaded.size_bytes,
					type: uploaded.media_type,
					url: uploaded.url,
					alt: uploaded.file_name
				};
			})
		);

		return uploads;
	}

	private idempotencyHeaders(operation: string) {
		const requestId = makeRequestId(operation);
		return {
			'x-request-id': requestId,
			'x-correlation-id': requestId
		};
	}

	private async resolveCurrentUserId(): Promise<string | undefined> {
		if (!this.currentUserIdPromise) {
			this.currentUserIdPromise = this.client
				.get<ApiAuthMe>('/auth/me')
				.then((response) => response.user_id)
				.catch(() => undefined);
		}
		return this.currentUserIdPromise;
	}

	private mapFeedToWitnesses(items: ApiFeedItem[]): Witness[] {
		const seen = new Set<string>();
		const results: Witness[] = [];

		for (const item of items) {
			const mapped = this.mapFeedItemToWitness(item);
			if (seen.has(mapped.witness_id)) {
				continue;
			}
			seen.add(mapped.witness_id);
			results.push(mapped);
		}

		return results;
	}

	private buildCreatedWitnessDetail(
		witnessId: string,
		input: WitnessCreateInput,
		response: ApiWitnessCreateResponse
	): WitnessDetail {
		const createdAt = toIso(asNumber(response.created_at_ms));
		const messages = buildCreatedWitnessMessages(input, witnessId, createdAt);
		const createdBy = asString(response.author_id) ?? 'me';

		return {
			witness_id: witnessId,
			title: asString(response.title) ?? input.title,
			summary: asString(response.summary) ?? input.summary,
			track_hint: input.track_hint,
			seed_hint: input.seed_hint,
			status: 'open',
			rahasia_level: input.rahasia_level,
			created_at: createdAt,
			updated_at: createdAt,
			created_by: createdBy,
			member_count: 1,
			message_count: messages.length,
			unread_count: 0,
			messages,
			plan: input.proposed_plan ?? null,
			blocks: [],
			members: [
				{
					user_id: createdBy,
					name: createdBy === 'me' ? 'Saya' : createdBy,
					role: 'pelapor',
					joined_at: createdAt
				}
			],
			triage: input.triage_result
		};
	}

	private mapFeedItemToWitness(item: ApiFeedItem): Witness {
		const payload = isRecord(item.payload) ? item.payload : undefined;
		const witnessId = asString(payload?.witness_id) ?? item.source_id;
		const statusCandidate = asString(payload?.status);
		const status =
			statusCandidate && WITNESS_STATUSES.has(statusCandidate as WitnessStatus)
				? (statusCandidate as WitnessStatus)
				: 'open';
		const trackHint = asString(payload?.track_hint);
		const seedHintCandidate = asString(payload?.seed_hint);
		const participantIds = Array.isArray(item.participant_ids) ? item.participant_ids : [];
		const memberCount = Math.max(1, new Set([item.actor_id, ...participantIds]).size);
		const createdAt = toIso(asNumber(item.occurred_at_ms) ?? asNumber(item.created_at_ms));
		const updatedAt = toIso(asNumber(item.created_at_ms) ?? asNumber(item.occurred_at_ms));
		const unreadCount = asNumber(payload?.unread_count) ?? 0;
		const messageCount = asNumber(payload?.message_count) ?? 0;

		return {
			witness_id: witnessId,
			title: item.title,
			summary: item.summary ?? item.title,
			track_hint: trackHint,
			seed_hint:
				seedHintCandidate && SEED_HINTS.has(seedHintCandidate as SeedHint)
					? (seedHintCandidate as SeedHint)
					: undefined,
			status,
			rahasia_level: mapPrivacyToRahasia(item.privacy_level),
			created_at: createdAt,
			updated_at: updatedAt,
			created_by: item.actor_id,
			member_count: memberCount,
			message_count: messageCount,
			unread_count: unreadCount
		};
	}

	private async fetchWitnessSummary(witnessId: string): Promise<Witness> {
		const response = await this.client.get<ApiPagedFeed>('/feed', {
			query: {
				limit: 50
			}
		});
		const mapped = this.mapFeedToWitnesses(this.extractFeedItems(response.items));
		const match = mapped.find((item) => item.witness_id === witnessId);
		if (match) {
			return match;
		}

		const now = new Date().toISOString();
		return {
			witness_id: witnessId,
			title: `Saksi ${witnessId}`,
			summary: 'Detail saksi',
			status: 'open',
			rahasia_level: 'L0',
			created_at: now,
			updated_at: now,
			created_by: 'unknown',
			member_count: 1,
			message_count: 0,
			unread_count: 0
		};
	}

	private extractFeedItems(items: Array<ApiFeedItem | ApiFeedStreamItem>): ApiFeedItem[] {
		return items
			.map((item) => {
				if (isFeedWitnessStreamItem(item)) {
					return item.data;
				}
				return isApiFeedItem(item) ? item : undefined;
			})
			.filter((item): item is ApiFeedItem => Boolean(item));
	}

	private async fetchWitnessMembers(witnessId: string): Promise<WitnessMember[]> {
		const threadId = await this.ensureThread(witnessId, false);
		if (!threadId) {
			return [];
		}

		await this.joinThread(threadId);
		const members = await this.client.get<ApiChatMember[]>(`/chat/threads/${threadId}/members`);
		return members
			.filter((member) => member.left_at_ms === null || member.left_at_ms === undefined)
			.map((member) => ({
				user_id: member.user_id,
				name: member.user_id,
				role: mapMemberRole(member.role),
				joined_at: toIso(member.joined_at_ms)
			}));
	}
}

import type { ApiClient } from '$lib/api';
import type {
	FeedDevMeta,
	FeedEventType,
	FeedItem,
	FeedSource,
	FeedStreamItem,
	FollowableEntity,
	ImpactVerification,
	MyRelation,
	ProgramReference,
	RahasiaLevel,
	SignalCounts,
	SignalLabels,
	Sentiment,
	SystemCardData,
	TriageStempelState,
	TrajectoryType
} from '$lib/types';
import type { FeedService, Paginated } from '../types';

interface ApiFeedItem {
	feed_id: string;
	source_type: string;
	source_id: string;
	actor_id: string;
	actor_username: string;
	title: string;
	summary?: string | null;
	privacy_level?: string | null;
	occurred_at_ms: number;
	created_at_ms: number;
	participant_ids?: string[];
	payload?: unknown;
}

interface ApiFeedWitnessStreamItem {
	stream_id?: string;
	sort_timestamp?: string;
	kind: 'witness';
	data: ApiFeedItem;
}

interface ApiFeedSystemStreamItem {
	stream_id?: string;
	sort_timestamp?: string;
	kind: 'system';
	data: SystemCardData;
}

type ApiFeedStreamItem = ApiFeedWitnessStreamItem | ApiFeedSystemStreamItem;

interface ApiFeedStreamResponse {
	items?: ApiFeedItem[] | ApiFeedStreamItem[];
	stream?: ApiFeedStreamItem[];
	next_cursor?: string | null;
	has_more?: boolean;
}

interface ApiPagedFeedLegacy {
	items: ApiFeedItem[];
	next_cursor?: string | null;
}

interface ApiFeedSuggestion {
	entity_id?: string;
	entity_type?: string;
	label?: string;
	followed?: boolean;
	description?: string | null;
	witness_count?: number;
	follower_count?: number;
}

interface ApiFeedSuggestionsResponse {
	entities?: ApiFeedSuggestion[];
}

type JsonRecord = Record<string, unknown>;

const SENTIMENTS = new Set<Sentiment>([
	'angry',
	'hopeful',
	'urgent',
	'celebratory',
	'sad',
	'curious',
	'fun'
]);

const TRAJECTORY_TYPES = new Set<TrajectoryType>([
	'aksi',
	'advokasi',
	'pantau',
	'mufakat',
	'mediasi',
	'program',
	'data',
	'vault',
	'bantuan',
	'pencapaian',
	'siaga'
]);

const FEED_SOURCES = new Set<FeedSource>(['ikutan', 'terlibat', 'sekitar']);
const ENTITY_TYPES = new Set(['lingkungan', 'topik', 'kelompok', 'lembaga', 'warga']);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim() ? value : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const asBoolean = (value: unknown): boolean | undefined =>
	typeof value === 'boolean' ? value : undefined;

const toIsoTime = (timestampMs: number | undefined) =>
	new Date(timestampMs && Number.isFinite(timestampMs) ? timestampMs : Date.now()).toISOString();

const mapPrivacyToRahasia = (
	privacyLevel: string | undefined,
	noteRahasias: unknown
): RahasiaLevel => {
	if (typeof noteRahasias === 'string') {
		const normalized = noteRahasias.toUpperCase();
		if (normalized === 'L0' || normalized === 'L1' || normalized === 'L2' || normalized === 'L3') {
			return normalized;
		}
	}

	if (typeof noteRahasias === 'number') {
		if (noteRahasias <= 0) return 'L0';
		if (noteRahasias === 1) return 'L1';
		if (noteRahasias === 2) return 'L2';
		return 'L3';
	}

	const normalized = (privacyLevel ?? '').trim().toLowerCase();
	if (
		!normalized ||
		normalized === 'public' ||
		normalized === 'open' ||
		normalized === 'unrestricted'
	) {
		return 'L0';
	}
	if (normalized === 'l0') return 'L0';
	if (normalized === 'l1' || normalized === 'private' || normalized === 'restricted') return 'L1';
	if (normalized === 'l2' || normalized === 'secret' || normalized === 'confidential') return 'L2';
	if (normalized === 'l3' || normalized === 'very_secret') return 'L3';
	return 'L1';
};

const mapSourceToEventType = (sourceType: string): FeedEventType => {
	switch (sourceType) {
		case 'vouch':
			return 'joined';
		case 'vault':
			return 'evidence';
		case 'ontology_note':
			return 'community_note';
		case 'moderation':
			return 'checkpoint';
		case 'siaga':
			return 'checkpoint';
		default:
			return 'created';
	}
};

const mapSourceToVerb = (sourceType: string): string => {
	switch (sourceType) {
		case 'vouch':
			return 'memberi vouch';
		case 'vault':
			return 'menambahkan catatan';
		case 'ontology_note':
			return 'membagikan catatan komunitas';
		case 'moderation':
			return 'memperbarui status moderasi';
		case 'siaga':
			return 'membagikan peringatan';
		default:
			return 'menambahkan kontribusi';
	}
};

const mapSourceToFeedSource = (sourceType: string, payload: JsonRecord | undefined): FeedSource => {
	const explicit = asString(payload?.source);
	if (explicit && FEED_SOURCES.has(explicit as FeedSource)) {
		return explicit as FeedSource;
	}

	if (sourceType === 'vouch') {
		return 'terlibat';
	}

	return 'sekitar';
};

const mapSourceToTrajectory = (sourceType: string): TrajectoryType => {
	switch (sourceType) {
		case 'vouch':
			return 'advokasi';
		case 'vault':
			return 'vault';
		case 'siaga':
			return 'siaga';
		case 'moderation':
			return 'pantau';
		case 'ontology_note':
			return 'data';
		default:
			return 'aksi';
	}
};

const mapUrgency = (sourceType: string): FeedItem['urgency'] | undefined => {
	if (sourceType === 'siaga') return 'baru';
	if (sourceType === 'vouch') return 'ramai';
	return undefined;
};

const mapSignalLabels = (enrichment: JsonRecord | undefined): SignalLabels | undefined => {
	const rawLabels = enrichment?.signal_labels;
	if (!isRecord(rawLabels)) {
		return undefined;
	}

	const parseOne = (value: unknown) => {
		if (!isRecord(value)) return undefined;
		const label = asString(value.label);
		const desc = asString(value.desc);
		if (!label || !desc) return undefined;
		const icon = asString(value.icon);
		return icon ? { label, desc, icon } : { label, desc };
	};

	const saksi = parseOne(rawLabels.saksi);
	const perluDicek = parseOne(rawLabels.perlu_dicek);
	if (!saksi || !perluDicek) {
		return undefined;
	}

	return {
		saksi,
		perlu_dicek: perluDicek
	};
};

const mapEntityTags = (enrichment: JsonRecord | undefined): FeedItem['entity_tags'] => {
	const rawTags = enrichment?.entity_tags;
	if (!Array.isArray(rawTags)) {
		return [];
	}

	return rawTags
		.map((tag) => {
			if (!isRecord(tag)) return undefined;
			const label = asString(tag.label);
			const entityType = asString(tag.entity_type);
			if (!label || !entityType || !ENTITY_TYPES.has(entityType)) {
				return undefined;
			}
			return {
				entity_id: `${entityType}:${label.toLowerCase().replace(/\s+/g, '-')}`,
				entity_type: entityType as FollowableEntity['entity_type'],
				label,
				followed: false
			};
		})
		.filter((tag): tag is FeedItem['entity_tags'][number] => Boolean(tag));
};

const mapMyRelation = (payload: JsonRecord | undefined): MyRelation | undefined => {
	const raw = payload?.my_relation;
	if (!isRecord(raw)) {
		return undefined;
	}

	const vouched = asBoolean(raw.vouched) ?? false;
	const witnessed = asBoolean(raw.witnessed) ?? false;
	const flagged = asBoolean(raw.flagged) ?? false;
	const supported = asBoolean(raw.supported) ?? false;
	const voteCast = asString(raw.vote_cast);
	const vouchType = asString(raw.vouch_type);
	const allowedVouchType =
		vouchType && ['positive', 'skeptical', 'conditional', 'mentorship'].includes(vouchType)
			? vouchType
			: undefined;

	const relation: MyRelation = {
		vouched,
		witnessed,
		flagged,
		supported
	};

	if (voteCast === 'yes' || voteCast === 'no') {
		relation.vote_cast = voteCast;
	}
	if (allowedVouchType) {
		relation.vouch_type = allowedVouchType as MyRelation['vouch_type'];
	}

	return relation;
};

const mapSignalCounts = (payload: JsonRecord | undefined): SignalCounts | undefined => {
	const raw = payload?.signal_counts;
	if (!isRecord(raw)) {
		return undefined;
	}

	return {
		vouch_positive: asNumber(raw.vouch_positive) ?? 0,
		vouch_skeptical: asNumber(raw.vouch_skeptical) ?? 0,
		witness_count: asNumber(raw.witness_count) ?? 0,
		dukung_count: asNumber(raw.dukung_count) ?? 0,
		flags: asNumber(raw.flags) ?? 0
	};
};

const mapProgramRefs = (payload: JsonRecord | undefined): ProgramReference[] | undefined => {
	const triageResult = isRecord(payload?.triage_result) ? payload?.triage_result : undefined;
	const raw =
		(Array.isArray(payload?.program_refs) ? payload?.program_refs : undefined) ??
		(Array.isArray(triageResult?.program_refs) ? triageResult?.program_refs : undefined);
	if (!Array.isArray(raw)) {
		return undefined;
	}
	const mapped = raw
		.map((item) => {
			if (!isRecord(item)) return undefined;
			const programId = asString(item.program_id);
			const label = asString(item.label);
			const source = asString(item.source);
			const confidence = asNumber(item.confidence);
			if (!programId || !label || !source || confidence == null) {
				return undefined;
			}
			return {
				program_id: programId,
				label,
				source,
				confidence
			} satisfies ProgramReference;
		})
		.filter((item): item is ProgramReference => Boolean(item));
	return mapped.length > 0 ? mapped : undefined;
};

const mapStempelState = (payload: JsonRecord | undefined): TriageStempelState | undefined => {
	const triageResult = isRecord(payload?.triage_result) ? payload?.triage_result : undefined;
	const raw = isRecord(payload?.stempel_state)
		? payload?.stempel_state
		: isRecord(triageResult?.stempel_state)
			? triageResult?.stempel_state
			: undefined;
	if (!isRecord(raw)) return undefined;
	const state = asString(raw.state);
	if (
		state !== 'draft' &&
		state !== 'proposed' &&
		state !== 'objection_window' &&
		state !== 'locked'
	) {
		return undefined;
	}
	const minParticipants = asNumber(raw.min_participants);
	const participantCount = asNumber(raw.participant_count);
	const objectionCount = asNumber(raw.objection_count);
	if (minParticipants == null || participantCount == null || objectionCount == null) {
		return undefined;
	}
	return {
		state,
		proposed_at_ms: asNumber(raw.proposed_at_ms),
		objection_deadline_ms: asNumber(raw.objection_deadline_ms),
		locked_at_ms: asNumber(raw.locked_at_ms),
		min_participants: minParticipants,
		participant_count: participantCount,
		objection_count: objectionCount,
		latest_objection_at_ms: asNumber(raw.latest_objection_at_ms),
		latest_objection_reason: asString(raw.latest_objection_reason)
	};
};

const mapImpactVerification = (payload: JsonRecord | undefined): ImpactVerification | undefined => {
	const raw = payload?.impact_verification;
	if (!isRecord(raw)) return undefined;
	const status = asString(raw.status);
	if (
		status !== 'not_open' &&
		status !== 'open' &&
		status !== 'verified' &&
		status !== 'disputed'
	) {
		return undefined;
	}
	const yesCount = asNumber(raw.yes_count);
	const noCount = asNumber(raw.no_count);
	const minVouches = asNumber(raw.min_vouches);
	if (yesCount == null || noCount == null || minVouches == null) {
		return undefined;
	}
	return {
		status,
		opened_at_ms: asNumber(raw.opened_at_ms),
		closes_at_ms: asNumber(raw.closes_at_ms),
		yes_count: yesCount,
		no_count: noCount,
		min_vouches: minVouches
	};
};

const mapDevMeta = (payload: JsonRecord | undefined): FeedDevMeta | undefined => {
	const raw = payload?.dev_meta;
	if (!isRecord(raw)) return undefined;
	const isSeed = asBoolean(raw.is_seed);
	if (isSeed == null) return undefined;
	const originCandidate = asString(raw.seed_origin);
	const seedOrigin =
		originCandidate === 'fixture' || originCandidate === 'db' || originCandidate === 'operator_stub'
			? originCandidate
			: undefined;
	return {
		is_seed: isSeed,
		seed_batch_id: asString(raw.seed_batch_id),
		seed_origin: seedOrigin
	};
};

const resolveTrajectory = (
	sourceType: string,
	enrichment: JsonRecord | undefined
): TrajectoryType => {
	const candidate = asString(enrichment?.trajectory_type);
	if (candidate && TRAJECTORY_TYPES.has(candidate as TrajectoryType)) {
		return candidate as TrajectoryType;
	}
	return mapSourceToTrajectory(sourceType);
};

const resolveSentiment = (enrichment: JsonRecord | undefined): Sentiment | undefined => {
	const candidate = asString(enrichment?.sentiment);
	if (candidate && SENTIMENTS.has(candidate as Sentiment)) {
		return candidate as Sentiment;
	}
	return undefined;
};

const toFeedItem = (item: ApiFeedItem): FeedItem => {
	const payload = isRecord(item.payload) ? item.payload : undefined;
	const enrichment = isRecord(payload?.enrichment) ? payload.enrichment : undefined;
	const note = isRecord(payload?.note) ? payload.note : undefined;
	const occurredAt = asNumber(item.occurred_at_ms) ?? asNumber(item.created_at_ms);
	const timestamp = toIsoTime(occurredAt);
	const actorRole = asString(payload?.actor_role);
	const participantIds = Array.isArray(item.participant_ids) ? item.participant_ids : [];
	const uniquePeople = new Set([item.actor_id, ...participantIds]);
	const memberCount = Math.max(1, uniquePeople.size);
	const trajectoryType = resolveTrajectory(item.source_type, enrichment);
	const relation = mapMyRelation(payload);

	return {
		witness_id: asString(payload?.witness_id) ?? item.source_id,
		title: asString(enrichment?.title) ?? item.title,
		trajectory_type: trajectoryType,
		icon: asString(enrichment?.icon),
		status: 'open',
		rahasia_level: mapPrivacyToRahasia(item.privacy_level ?? undefined, note?.rahasia_level),
		latest_event: {
			event_id: item.feed_id,
			event_type: mapSourceToEventType(item.source_type),
			actor_name: item.actor_username,
			actor_role:
				actorRole === 'pelapor' ||
				actorRole === 'relawan' ||
				actorRole === 'koordinator' ||
				actorRole === 'saksi'
					? actorRole
					: undefined,
			timestamp,
			verb: asString(payload?.event_verb) ?? mapSourceToVerb(item.source_type),
			snippet: item.summary ?? undefined
		},
		collapsed_count: Math.max(0, memberCount - 1),
		member_count: memberCount,
		members_preview: [
			{
				user_id: item.actor_id,
				name: item.actor_username,
				role: 'pelapor'
			}
		],
		entity_tags: mapEntityTags(enrichment),
		urgency: mapUrgency(item.source_type),
		source: mapSourceToFeedSource(item.source_type, payload),
		hook_line: asString(enrichment?.hook_line),
		pull_quote: asString(enrichment?.pull_quote),
		sentiment: resolveSentiment(enrichment),
		intensity: asNumber(enrichment?.intensity),
		cover_url: asString(payload?.cover_url) ?? asString(enrichment?.cover_url),
		body: asString(enrichment?.body) ?? item.summary ?? undefined,
		signal_labels: mapSignalLabels(enrichment),
		my_relation: relation,
		signal_counts: mapSignalCounts(payload),
		monitored:
			asBoolean(payload?.monitored) ??
			(relation
				? relation.vouched || relation.witnessed || relation.flagged || relation.vote_cast != null
				: false),
		deadline: asString(payload?.deadline),
		deadline_label: asString(payload?.deadline_label),
		program_refs: mapProgramRefs(payload),
		stempel_state: mapStempelState(payload),
		impact_verification: mapImpactVerification(payload),
		quorum_target: asNumber(payload?.quorum_target),
		quorum_current: asNumber(payload?.quorum_current),
		dev_meta: mapDevMeta(payload)
	};
};

const toFollowableEntity = (item: ApiFeedSuggestion): FollowableEntity | undefined => {
	const label = asString(item.label);
	const entityType = asString(item.entity_type);
	if (!label || !entityType || !ENTITY_TYPES.has(entityType)) {
		return undefined;
	}

	const fallbackEntityId = `${entityType}:${label.toLowerCase().replace(/\s+/g, '-')}`;
	const entityId = asString(item.entity_id) ?? fallbackEntityId;
	const witnessCount = asNumber(item.witness_count) ?? 0;
	const followerCount = asNumber(item.follower_count) ?? witnessCount;

	return {
		entity_id: entityId,
		entity_type: entityType as FollowableEntity['entity_type'],
		label,
		followed: asBoolean(item.followed) ?? false,
		description: asString(item.description),
		witness_count: witnessCount,
		follower_count: followerCount
	};
};

const extractSuggestionRows = (
	response: ApiFeedSuggestion[] | ApiFeedSuggestionsResponse
): ApiFeedSuggestion[] => {
	if (Array.isArray(response)) {
		return response;
	}
	if (Array.isArray(response.entities)) {
		return response.entities;
	}
	return [];
};

const isSystemCardData = (value: unknown): value is SystemCardData => {
	if (!isRecord(value)) return false;
	const variant = asString(value.variant);
	if (
		variant !== 'suggestion' &&
		variant !== 'tip' &&
		variant !== 'milestone' &&
		variant !== 'prompt'
	) {
		return false;
	}
	if (!asString(value.icon) || !asString(value.title) || typeof value.dismissible !== 'boolean') {
		return false;
	}
	const payload = value.payload;
	if (!isRecord(payload)) return false;
	const payloadVariant = asString(payload.variant);
	return payloadVariant === variant;
};

const toFeedStreamItemFromLegacy = (item: ApiFeedItem): FeedStreamItem => {
	const sortTimestamp = toIsoTime(asNumber(item.occurred_at_ms) ?? asNumber(item.created_at_ms));
	return {
		stream_id: `w-${item.feed_id}`,
		sort_timestamp: sortTimestamp,
		kind: 'witness',
		data: toFeedItem(item)
	};
};

const toFeedStreamItem = (item: ApiFeedStreamItem): FeedStreamItem | undefined => {
	if (item.kind === 'witness') {
		const sortTimestamp =
			asString(item.sort_timestamp) ??
			toIsoTime(asNumber(item.data.occurred_at_ms) ?? asNumber(item.data.created_at_ms));
		return {
			stream_id: asString(item.stream_id) ?? `w-${item.data.feed_id}`,
			sort_timestamp: sortTimestamp,
			kind: 'witness',
			data: toFeedItem(item.data)
		};
	}

	if (!isSystemCardData(item.data)) {
		return undefined;
	}
	return {
		stream_id: asString(item.stream_id) ?? `sys-${Math.random().toString(36).slice(2, 8)}`,
		sort_timestamp: asString(item.sort_timestamp) ?? new Date().toISOString(),
		kind: 'system',
		data: item.data
	};
};

const isApiFeedItem = (value: unknown): value is ApiFeedItem => {
	if (!isRecord(value)) return false;
	return Boolean(
		asString(value.feed_id) &&
		asString(value.source_type) &&
		asString(value.source_id) &&
		asString(value.actor_id) &&
		asString(value.actor_username) &&
		asString(value.title)
	);
};

const isApiFeedStreamItem = (value: unknown): value is ApiFeedStreamItem => {
	if (!isRecord(value)) return false;
	const kind = asString(value.kind);
	if (kind !== 'witness' && kind !== 'system') return false;
	if (!('data' in value)) return false;
	if (kind === 'witness') {
		return isApiFeedItem(value.data);
	}
	return isSystemCardData(value.data);
};

export class ApiFeedService implements FeedService {
	private readonly client: ApiClient;

	constructor(client: ApiClient) {
		this.client = client;
	}

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<FeedStreamItem>> {
		const response = await this.client.get<ApiFeedStreamResponse | ApiPagedFeedLegacy>('/feed', {
			query: {
				cursor: opts?.cursor,
				limit: opts?.limit
			}
		});

		const responseStream = (response as ApiFeedStreamResponse).stream;
		const rawStreamItems: unknown[] =
			Array.isArray(responseStream) && responseStream.length > 0
				? responseStream
				: Array.isArray(response.items)
					? response.items
					: [];
		const streamItems = rawStreamItems.every(isApiFeedStreamItem)
			? rawStreamItems
					.map((item) => toFeedStreamItem(item))
					.filter((item): item is FeedStreamItem => Boolean(item))
			: rawStreamItems
					.filter((item): item is ApiFeedItem => isApiFeedItem(item))
					.map((item) => toFeedStreamItemFromLegacy(item));

		return {
			items: streamItems,
			total: streamItems.length,
			cursor: response.next_cursor ?? undefined
		};
	}

	async listSuggestions(): Promise<FollowableEntity[]> {
		const response = await this.client.get<ApiFeedSuggestion[] | ApiFeedSuggestionsResponse>(
			'/feed/suggestions'
		);
		return extractSuggestionRows(response)
			.map((item) => toFollowableEntity(item))
			.filter((item): item is FollowableEntity => Boolean(item));
	}

	async setMonitorPreference(witnessId: string, monitored: boolean): Promise<void> {
		await this.client.post(`/feed/preferences/monitor/${encodeURIComponent(witnessId)}`, {
			body: { monitored }
		});
	}

	async setEntityFollowPreference(entityId: string, followed: boolean): Promise<void> {
		await this.client.post(`/feed/preferences/follow/${encodeURIComponent(entityId)}`, {
			body: { followed }
		});
	}
}

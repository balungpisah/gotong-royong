import type { ApiClient } from '$lib/api';
import type {
	ContentSignal,
	ContentSignalType,
	MyRelation,
	SignalCounts,
	SignalResolutionOutcome
} from '$lib/types';
import type { SignalService } from '../types';

type JsonRecord = Record<string, unknown>;

const SIGNAL_TYPES = new Set<ContentSignalType>(['saksi', 'perlu_dicek']);
const SIGNAL_OUTCOMES = new Set<SignalResolutionOutcome>([
	'pending',
	'resolved_positive',
	'resolved_negative',
	'resolved_neutral',
	'expired'
]);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const asBoolean = (value: unknown): boolean | undefined =>
	typeof value === 'boolean' ? value : undefined;

const readSignalType = (value: unknown): ContentSignalType | undefined => {
	const normalized = asString(value)?.toLowerCase();
	if (!normalized) return undefined;
	if (!SIGNAL_TYPES.has(normalized as ContentSignalType)) return undefined;
	return normalized as ContentSignalType;
};

const readSignalOutcome = (value: unknown): SignalResolutionOutcome | undefined => {
	const normalized = asString(value)?.toLowerCase();
	if (!normalized) return undefined;
	if (!SIGNAL_OUTCOMES.has(normalized as SignalResolutionOutcome)) return undefined;
	return normalized as SignalResolutionOutcome;
};

const toIsoString = (value: unknown) => {
	if (typeof value === 'number' && Number.isFinite(value)) {
		return new Date(value).toISOString();
	}
	if (typeof value === 'string' && value.trim().length > 0) {
		const directDate = Date.parse(value);
		if (!Number.isNaN(directDate)) {
			return new Date(directDate).toISOString();
		}
		const fromNumber = Number(value);
		if (!Number.isNaN(fromNumber) && Number.isFinite(fromNumber)) {
			return new Date(fromNumber).toISOString();
		}
	}
	return new Date().toISOString();
};

const parseSignal = (value: unknown): ContentSignal | undefined => {
	if (!isRecord(value)) return undefined;
	const signalId = asString(value.signal_id);
	const witnessId = asString(value.witness_id);
	const userId = asString(value.user_id);
	const signalType = readSignalType(value.signal_type);
	const outcome = readSignalOutcome(value.outcome);
	if (!signalId || !witnessId || !userId || !signalType || !outcome) {
		return undefined;
	}
	return {
		signal_id: signalId,
		witness_id: witnessId,
		user_id: userId,
		signal_type: signalType,
		outcome,
		created_at: toIsoString(value.created_at),
		resolved_at: value.resolved_at === undefined ? undefined : toIsoString(value.resolved_at),
		credit_delta: asNumber(value.credit_delta)
	};
};

const parseRelation = (value: unknown): MyRelation | undefined => {
	if (!isRecord(value)) return undefined;
	const voteCast = asString(value.vote_cast);
	return {
		vouched: asBoolean(value.vouched) ?? false,
		vouch_type: asString(value.vouch_type) as MyRelation['vouch_type'],
		witnessed: asBoolean(value.witnessed) ?? false,
		flagged: asBoolean(value.flagged) ?? false,
		supported: asBoolean(value.supported) ?? false,
		vote_cast: voteCast === 'yes' || voteCast === 'no' ? voteCast : undefined
	};
};

const parseCounts = (value: unknown): SignalCounts | undefined => {
	if (!isRecord(value)) return undefined;
	return {
		vouch_positive: asNumber(value.vouch_positive) ?? 0,
		vouch_skeptical: asNumber(value.vouch_skeptical) ?? 0,
		witness_count: asNumber(value.witness_count) ?? 0,
		dukung_count: asNumber(value.dukung_count) ?? 0,
		flags: asNumber(value.flags) ?? 0
	};
};

interface ApiSignalServiceOptions {
	allowMockFallback?: boolean;
}

export class ApiSignalService implements SignalService {
	private readonly client: ApiClient;
	private readonly fallback: SignalService;
	private readonly allowMockFallback: boolean;

	constructor(client: ApiClient, fallback: SignalService, options: ApiSignalServiceOptions = {}) {
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
		throw new Error('Mock fallback disabled for signal service');
	}

	async sendSignal(witnessId: string, signalType: ContentSignalType): Promise<ContentSignal> {
		try {
			const response = await this.client.post<unknown>(
				`/witnesses/${encodeURIComponent(witnessId)}/signals`,
				{
					body: {
						signal_type: signalType
					}
				}
			);
			const signal = parseSignal(response);
			if (!signal) {
				throw new Error('invalid signal response');
			}
			return signal;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.sendSignal(witnessId, signalType), error);
		}
	}

	async removeSignal(witnessId: string, signalType: ContentSignalType): Promise<void> {
		try {
			await this.client.delete<unknown>(
				`/witnesses/${encodeURIComponent(witnessId)}/signals/${encodeURIComponent(signalType)}`
			);
		} catch (error) {
			await this.fallbackOrThrow(() => this.fallback.removeSignal(witnessId, signalType), error);
		}
	}

	async getMyRelation(witnessId: string): Promise<MyRelation> {
		try {
			const response = await this.client.get<unknown>(
				`/witnesses/${encodeURIComponent(witnessId)}/signals/my-relation`
			);
			const relation = parseRelation(response);
			if (!relation) {
				throw new Error('invalid relation response');
			}
			return relation;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.getMyRelation(witnessId), error);
		}
	}

	async getSignalCounts(witnessId: string): Promise<SignalCounts> {
		try {
			const response = await this.client.get<unknown>(
				`/witnesses/${encodeURIComponent(witnessId)}/signals/counts`
			);
			const counts = parseCounts(response);
			if (!counts) {
				throw new Error('invalid counts response');
			}
			return counts;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.getSignalCounts(witnessId), error);
		}
	}

	async getResolutions(witnessId: string): Promise<ContentSignal[]> {
		try {
			const response = await this.client.get<unknown>(
				`/witnesses/${encodeURIComponent(witnessId)}/signals/resolutions`
			);
			if (!Array.isArray(response)) {
				throw new Error('invalid resolutions response');
			}
			return response.map(parseSignal).filter((item): item is ContentSignal => Boolean(item));
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.getResolutions(witnessId), error);
		}
	}
}

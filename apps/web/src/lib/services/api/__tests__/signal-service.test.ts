import { describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type { ContentSignal, MyRelation, SignalCounts } from '$lib/types';
import type { SignalService } from '$lib/services/types';
import { ApiSignalService } from '../signal-service';

const makeApiClient = () => {
	const get = vi.fn();
	const post = vi.fn();
	const del = vi.fn();
	const client = {
		request: vi.fn(),
		get,
		post,
		put: vi.fn(),
		patch: vi.fn(),
		delete: del
	} as unknown as ApiClient;

	return { client, get, post, del };
};

const makeFallbackSignal = (): ContentSignal => ({
	signal_id: 'sig-fallback',
	witness_id: 'wit-fallback',
	user_id: 'user-fallback',
	signal_type: 'saksi',
	outcome: 'pending',
	created_at: '2026-01-01T00:00:00.000Z'
});

const makeFallbackService = () => {
	const signal = makeFallbackSignal();
	const relation: MyRelation = {
		vouched: false,
		witnessed: false,
		flagged: false,
		supported: false
	};
	const counts: SignalCounts = {
		vouch_positive: 0,
		vouch_skeptical: 0,
		witness_count: 0,
		dukung_count: 0,
		flags: 0
	};
	const resolutions: ContentSignal[] = [];

	const service: SignalService = {
		sendSignal: vi.fn(async () => signal),
		removeSignal: vi.fn(async () => undefined),
		getMyRelation: vi.fn(async () => relation),
		getSignalCounts: vi.fn(async () => counts),
		getResolutions: vi.fn(async () => resolutions)
	};
	return { service, signal, relation, counts, resolutions };
};

describe('ApiSignalService', () => {
	it('sends signal and maps backend response', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockResolvedValue({
			signal_id: 'sig-1',
			witness_id: 'wit-1',
			user_id: 'user-1',
			signal_type: 'saksi',
			outcome: 'pending',
			created_at: 1_700_000_000_000
		});

		const service = new ApiSignalService(client, fallback);
		const signal = await service.sendSignal('wit-1', 'saksi');

		expect(signal.signal_id).toBe('sig-1');
		expect(signal.signal_type).toBe('saksi');
		expect(signal.outcome).toBe('pending');
		expect(signal.created_at).toBe(new Date(1_700_000_000_000).toISOString());
		expect(post).toHaveBeenCalledWith('/witnesses/wit-1/signals', {
			body: { signal_type: 'saksi' }
		});
	});

	it('loads relation, counts, and resolutions from backend', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();

		get.mockImplementation(async (path: string) => {
			if (path === '/witnesses/wit-2/signals/my-relation') {
				return {
					vouched: false,
					witnessed: true,
					flagged: false,
					supported: false
				};
			}
			if (path === '/witnesses/wit-2/signals/counts') {
				return {
					vouch_positive: 0,
					vouch_skeptical: 0,
					witness_count: 4,
					dukung_count: 2,
					flags: 1
				};
			}
			if (path === '/witnesses/wit-2/signals/resolutions') {
				return [
					{
						signal_id: 'sig-res-1',
						witness_id: 'wit-2',
						user_id: 'user-1',
						signal_type: 'perlu_dicek',
						outcome: 'resolved_positive',
						created_at: 1_700_000_100_000,
						resolved_at: 1_700_000_200_000,
						credit_delta: 2.5
					}
				];
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiSignalService(client, fallback);
		await expect(service.getMyRelation('wit-2')).resolves.toMatchObject({
			witnessed: true,
			flagged: false
		});
		await expect(service.getSignalCounts('wit-2')).resolves.toMatchObject({
			witness_count: 4,
			flags: 1
		});
		const resolutions = await service.getResolutions('wit-2');
		expect(resolutions).toHaveLength(1);
		expect(resolutions[0]).toMatchObject({
			signal_id: 'sig-res-1',
			signal_type: 'perlu_dicek',
			outcome: 'resolved_positive',
			credit_delta: 2.5
		});
	});

	it('falls back when backend is unavailable', async () => {
		const { client, get, post, del } = makeApiClient();
		const { service: fallback, signal, relation, counts, resolutions } = makeFallbackService();
		get.mockRejectedValue(new Error('backend unavailable'));
		post.mockRejectedValue(new Error('backend unavailable'));
		del.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiSignalService(client, fallback);
		await expect(service.sendSignal('wit-3', 'saksi')).resolves.toEqual(signal);
		await expect(service.getMyRelation('wit-3')).resolves.toEqual(relation);
		await expect(service.getSignalCounts('wit-3')).resolves.toEqual(counts);
		await expect(service.getResolutions('wit-3')).resolves.toEqual(resolutions);
		await expect(service.removeSignal('wit-3', 'perlu_dicek')).resolves.toBeUndefined();
		expect(fallback.removeSignal).toHaveBeenCalledWith('wit-3', 'perlu_dicek');
	});

	it('does not use mock fallback when disabled', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiSignalService(client, fallback, { allowMockFallback: false });
		await expect(service.sendSignal('wit-3', 'saksi')).rejects.toThrow('backend unavailable');
		expect(fallback.sendSignal).not.toHaveBeenCalled();
	});
});

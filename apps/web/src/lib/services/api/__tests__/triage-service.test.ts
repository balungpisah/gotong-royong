import { describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type { TriageResult } from '$lib/types';
import type { TriageService } from '$lib/services/types';
import { ApiTriageService } from '../triage-service';

const makeApiClient = () => {
	const post = vi.fn();
	const client = {
		request: vi.fn(),
		get: vi.fn(),
		post,
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, post };
};

const makeFallbackResult = (): TriageResult => ({
	bar_state: 'probing',
	route: 'komunitas',
	confidence: { score: 0.4, label: 'Menganalisis...' }
});

const makeFallbackService = () => {
	const startResult = makeFallbackResult();
	const updateResult: TriageResult = {
		...startResult,
		bar_state: 'leaning',
		confidence: { score: 0.7, label: 'Komunitas · 70%' }
	};
	const service: TriageService = {
		startTriage: vi.fn(async () => startResult),
		updateTriage: vi.fn(async () => updateResult)
	};
	return { service, startResult, updateResult };
};

describe('ApiTriageService', () => {
	it('starts triage session and maps backend result', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockResolvedValue({
			session_id: 'triage-sess-1',
			result: {
				bar_state: 'probing',
				route: 'komunitas',
				trajectory_type: 'aksi',
				confidence: { score: 0.4, label: 'Komunitas · 40%' }
			}
		});

		const service = new ApiTriageService(client, fallback);
		const result = await service.startTriage('jalan rusak');

		expect(result.session_id).toBe('triage-sess-1');
		expect(result.bar_state).toBe('probing');
		expect(result.route).toBe('komunitas');
		expect(result.trajectory_type).toBe('aksi');
		expect(post).toHaveBeenCalledWith('/triage/sessions', {
			body: { content: 'jalan rusak', attachments: undefined }
		});
	});

	it('uses session id for follow-up message', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();

		post.mockImplementation(async (path: string) => {
			if (path === '/triage/sessions') {
				return {
					session_id: 'triage-sess-2',
					result: {
						bar_state: 'probing',
						route: 'komunitas',
						confidence: { score: 0.4, label: 'Komunitas · 40%' }
					}
				};
			}
			if (path === '/triage/sessions/triage-sess-2/messages') {
				return {
					result: {
						bar_state: 'leaning',
						route: 'komunitas',
						confidence: { score: 0.7, label: 'Komunitas · 70%' }
					}
				};
			}
			throw new Error(`unexpected path: ${path}`);
		});

		const service = new ApiTriageService(client, fallback);
		await service.startTriage('halo');
		const next = await service.updateTriage('triage-sess-2', 'lanjut');

		expect(next.session_id).toBe('triage-sess-2');
		expect(next.bar_state).toBe('leaning');
		expect(post).toHaveBeenCalledWith('/triage/sessions/triage-sess-2/messages', {
			body: { answer: 'lanjut', attachments: undefined }
		});
	});

	it('falls back when backend triage calls fail', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback, startResult, updateResult } = makeFallbackService();
		post.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiTriageService(client, fallback);
		await expect(service.startTriage('test')).resolves.toEqual(startResult);
		await expect(service.updateTriage('session-mock', 'jawab')).resolves.toEqual(updateResult);
		expect(fallback.startTriage).toHaveBeenCalled();
		expect(fallback.updateTriage).toHaveBeenCalled();
	});

	it('does not use mock fallback when disabled', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiTriageService(client, fallback, { allowMockFallback: false });
		await expect(service.startTriage('test')).rejects.toThrow('backend unavailable');
		expect(fallback.startTriage).not.toHaveBeenCalled();
	});
});

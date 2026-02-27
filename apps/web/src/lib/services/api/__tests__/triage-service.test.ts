import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type { TriageResult } from '$lib/types';
import type { TriageService } from '$lib/services/types';
import {
	ApiTriageService,
	getTriageFallbackDiagnostics,
	resetTriageFallbackDiagnostics,
	TRIAGE_FALLBACK_FLAG_KEY
} from '../triage-service';

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
	schema_version: 'triage.v1',
	status: 'draft',
	kind: 'witness',
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
	beforeEach(() => {
		resetTriageFallbackDiagnostics();
		delete (globalThis as Record<string, unknown>)[TRIAGE_FALLBACK_FLAG_KEY];
	});

	it('starts triage session and maps backend result', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockResolvedValue({
			session_id: 'triage-sess-1',
			result: {
				schema_version: 'triage.v1',
				status: 'draft',
				kind: 'witness',
				bar_state: 'probing',
				route: 'komunitas',
				trajectory_type: 'aksi',
				blocks: {
					conversation: ['ai_inline_card', 'diff_card'],
					structured: ['document', 'list', 'computed']
				},
				structured_payload: [
					{
						type: 'document',
						id: 'doc-1',
						sections: [
							{
								id: 'sec-1',
								content: 'Ringkasan konteks',
								source: 'ai',
								locked_fields: []
							}
						]
					}
				],
				conversation_payload: [
					{
						message_id: 'm-1',
						type: 'ai_card',
						timestamp: '2026-02-26T00:00:00Z',
						witness_id: 'triage-preview',
						title: 'Ringkasan Operator',
						blocks: [
							{
								type: 'document',
								id: 'doc-2',
								sections: [
									{
										id: 'sec-2',
										content: 'Butuh validasi lokasi',
										source: 'ai',
										locked_fields: []
									}
								]
							}
						]
					}
				],
				confidence: { score: 0.4, label: 'Komunitas · 40%' }
			}
		});

		const service = new ApiTriageService(client, fallback);
		const result = await service.startTriage('jalan rusak');

		expect(result.session_id).toBe('triage-sess-1');
		expect(result.bar_state).toBe('probing');
		expect(result.route).toBe('komunitas');
		expect(result.trajectory_type).toBe('aksi');
		expect(result.blocks?.conversation).toEqual(['ai_inline_card', 'diff_card']);
		expect(result.blocks?.structured).toEqual(['document', 'list', 'computed']);
		expect(result.structured_payload?.length).toBe(1);
		expect(result.structured_payload?.[0]?.type).toBe('document');
		expect(result.conversation_payload?.length).toBe(1);
		expect(result.conversation_payload?.[0]?.type).toBe('ai_card');
		expect(post).toHaveBeenCalledWith('/triage/sessions', {
			body: { content: 'jalan rusak', attachments: undefined }
		});
	});

	it('drops invalid block declarations from backend result', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockResolvedValue({
			session_id: 'triage-sess-1',
			result: {
				schema_version: 'triage.v1',
				status: 'draft',
				kind: 'witness',
				bar_state: 'probing',
				route: 'komunitas',
				blocks: {
					conversation: ['ai_inline_card', 'unknown_block'],
					structured: ['document']
				},
				structured_payload: [
					{
						type: 'vote',
						id: 'vote-1',
						question: 'Setuju?',
						vote_type: 'consensus',
						options: [{ id: 'o1', label: 'Ya', count: 1 }],
						quorum: 0.5,
						total_eligible: 10,
						total_voted: 1,
						duration_hours: 24
					}
				],
				conversation_payload: [
					{
						message_id: 'm-2',
						type: 'diff_card',
						timestamp: '2026-02-26T00:00:00Z',
						witness_id: 'triage-preview',
						diff: {
							diff_id: 'd-1',
							target_type: 'document',
							target_id: 'doc-1',
							summary: 'Saran',
							items: [{ operation: 'modify', path: 'title', label: 'Ubah judul' }],
							source: 'ai',
							generated_at: '2026-02-26T00:00:00Z'
						}
					}
				],
				confidence: { score: 0.4, label: 'Komunitas · 40%' }
			}
		});

		const service = new ApiTriageService(client, fallback);
		const result = await service.startTriage('jalan rusak');

		expect(result.blocks).toBeUndefined();
		expect(result.structured_payload).toBeUndefined();
		expect(result.conversation_payload).toBeUndefined();
	});

	it('uses session id for follow-up message', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();

		post.mockImplementation(async (path: string) => {
			if (path === '/triage/sessions') {
				return {
					session_id: 'triage-sess-2',
					result: {
						schema_version: 'triage.v1',
						status: 'draft',
						kind: 'witness',
						bar_state: 'probing',
						route: 'komunitas',
						confidence: { score: 0.4, label: 'Komunitas · 40%' }
					}
				};
			}
			if (path === '/triage/sessions/triage-sess-2/messages') {
				return {
					result: {
						schema_version: 'triage.v1',
						status: 'draft',
						kind: 'witness',
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
		expect(getTriageFallbackDiagnostics()).toEqual({
			total: 2,
			by_operation: {
				start: 1,
				update: 1,
				update_no_session: 0
			},
			last_error_message: 'backend unavailable'
		});
		expect((globalThis as Record<string, unknown>)[TRIAGE_FALLBACK_FLAG_KEY]).toBe(true);
	});

	it('does not use mock fallback when disabled', async () => {
		const { client, post } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		post.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiTriageService(client, fallback, { allowMockFallback: false });
		await expect(service.startTriage('test')).rejects.toThrow('backend unavailable');
		expect(fallback.startTriage).not.toHaveBeenCalled();
		expect(getTriageFallbackDiagnostics().total).toBe(0);
		expect((globalThis as Record<string, unknown>)[TRIAGE_FALLBACK_FLAG_KEY]).toBeUndefined();
	});

	it('tracks missing-session update fallback as guardrail signal', async () => {
		const { client } = makeApiClient();
		const { service: fallback, updateResult } = makeFallbackService();
		const service = new ApiTriageService(client, fallback);

		await expect(service.updateTriage('', 'lanjut')).resolves.toEqual(updateResult);
		expect(fallback.updateTriage).toHaveBeenCalledWith('', 'lanjut', undefined);
		expect(getTriageFallbackDiagnostics()).toEqual({
			total: 1,
			by_operation: {
				start: 0,
				update: 0,
				update_no_session: 1
			},
			last_error_message: undefined
		});
	});
});

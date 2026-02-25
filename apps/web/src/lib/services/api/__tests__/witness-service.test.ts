import { describe, it, expect, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type { WitnessService } from '../../types';
import { ApiWitnessService } from '../witness-service';

const createFallback = (): WitnessService => ({
	create: vi.fn(),
	list: vi.fn(),
	get: vi.fn(),
	getMessages: vi.fn(),
	sendMessage: vi.fn(),
	getPlan: vi.fn(),
	respondToDiff: vi.fn(),
	castVote: vi.fn()
});

const createApiClient = () => {
	const get = vi.fn();
	const post = vi.fn();

	const client = {
		request: vi.fn(),
		get,
		post,
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, get, post };
};

describe('ApiWitnessService', () => {
	it('creates thread on first send and maps backend message', async () => {
		const fallback = createFallback();
		const { client, get, post } = createApiClient();

		get.mockImplementation(async (path: string) => {
			if (path === '/chat/threads') return [];
			if (path === '/auth/me') return { user_id: 'user-1' };
			throw new Error(`unexpected GET ${path}`);
		});

		post.mockImplementation(async (path: string) => {
			if (path === '/chat/threads') {
				return { thread_id: 'thread-1', scope_id: 'witness-1' };
			}
			if (path === '/chat/threads/thread-1/join') {
				return { ok: true };
			}
			if (path === '/chat/threads/thread-1/messages/send') {
				return {
					thread_id: 'thread-1',
					message_id: 'msg-1',
					author_id: 'user-1',
					author: {
						user_id: 'user-1',
						name: 'Rina Koordinator',
						role: 'member'
					},
					body: 'Halo',
					attachments: [],
					created_at_ms: 1_700_000_000_000
				};
			}
			throw new Error(`unexpected POST ${path}`);
		});

		const service = new ApiWitnessService(client, fallback);
		const message = await service.sendMessage('witness-1', 'Halo');

		expect(message).toMatchObject({
			type: 'user',
			witness_id: 'witness-1',
			message_id: 'msg-1',
			content: 'Halo',
			is_self: true,
			author: { user_id: 'user-1', name: 'Saya' }
		});
		expect(post).toHaveBeenCalledWith(
			'/chat/threads',
			expect.objectContaining({
				body: {
					scope_id: 'witness-1',
					privacy_level: 'public'
				}
			})
		);
		expect(post).toHaveBeenCalledWith(
			'/chat/threads/thread-1/messages/send',
			expect.objectContaining({
				body: {
					body: 'Halo',
					attachments: []
				}
			})
		);
	});

	it('uploads attachments before sending chat message', async () => {
		const fallback = createFallback();
		const { client, get, post } = createApiClient();

		get.mockImplementation(async (path: string) => {
			if (path === '/chat/threads')
				return [{ thread_id: 'thread-attach', scope_id: 'witness-attach' }];
			if (path === '/auth/me') return { user_id: 'user-1' };
			throw new Error(`unexpected GET ${path}`);
		});

		post.mockImplementation(async (path: string) => {
			if (path === '/chat/attachments/upload') {
				return {
					attachment_id: 'att-1',
					file_name: 'foto.png',
					mime_type: 'image/png',
					size_bytes: 100,
					media_type: 'image',
					url: '/v1/chat/attachments/att-1/download?exp=9&sig=abc',
					expires_at_ms: 9
				};
			}
			if (path === '/chat/threads/thread-attach/join') return { ok: true };
			if (path === '/chat/threads/thread-attach/messages/send') {
				return {
					thread_id: 'thread-attach',
					message_id: 'msg-attach-1',
					author_id: 'user-1',
					body: 'pakai lampiran',
					attachments: [
						{
							type: 'image',
							url: '/v1/chat/attachments/att-1/download?exp=9&sig=abc',
							alt: 'foto.png'
						}
					],
					created_at_ms: 1_700_000_000_999
				};
			}
			throw new Error(`unexpected POST ${path}`);
		});

		const service = new ApiWitnessService(client, fallback);
		const file = new File([new Uint8Array([1, 2, 3])], 'foto.png', { type: 'image/png' });
		await service.sendMessage('witness-attach', 'pakai lampiran', [file]);

		expect(post).toHaveBeenCalledWith(
			'/chat/attachments/upload',
			expect.objectContaining({
				body: expect.any(FormData)
			})
		);
		expect(post).toHaveBeenCalledWith(
			'/chat/threads/thread-attach/messages/send',
			expect.objectContaining({
				body: {
					body: 'pakai lampiran',
					attachments: [
						expect.objectContaining({
							attachment_id: 'att-1',
							type: 'image',
							url: '/v1/chat/attachments/att-1/download?exp=9&sig=abc'
						})
					]
				}
			})
		);
	});

	it('loads messages with cursor mapping', async () => {
		const fallback = createFallback();
		const { client, get, post } = createApiClient();

		get.mockImplementation(async (path: string) => {
			if (path === '/chat/threads') return [{ thread_id: 'thread-7', scope_id: 'witness-7' }];
			if (path === '/chat/threads/thread-7/messages/poll') {
				return [
					{
						thread_id: 'thread-7',
						message_id: 'msg-10',
						author_id: 'other-user',
						author: {
							user_id: 'other-user',
							name: 'Pak RT 05',
							role: 'user',
							tier: 3
						},
						body: 'Update terbaru',
						attachments: [{ type: 'image', url: 'https://img.test/a.png', alt: 'bukti' }],
						created_at_ms: 1_700_000_111_000
					}
				];
			}
			if (path === '/auth/me') return { user_id: 'user-me' };
			throw new Error(`unexpected GET ${path}`);
		});

		post.mockResolvedValue({ ok: true });

		const service = new ApiWitnessService(client, fallback);
		const page = await service.getMessages('witness-7', {
			cursor: '1700000000000:msg-9',
			limit: 20
		});

		expect(page.total).toBe(1);
		expect(page.cursor).toBe('1700000111000:msg-10');
		expect(page.items[0]).toMatchObject({
			type: 'user',
			witness_id: 'witness-7',
			is_self: false,
			content: 'Update terbaru',
			author: { user_id: 'other-user', name: 'Pak RT 05', role: 'user', tier: 3 }
		});
		if (page.items[0].type === 'user') {
			expect(page.items[0].attachments?.[0]).toEqual({
				type: 'image',
				url: 'https://img.test/a.png',
				alt: 'bukti'
			});
		}
		expect(get).toHaveBeenCalledWith('/chat/threads/thread-7/messages/poll', {
			query: {
				since_created_at_ms: 1_700_000_000_000,
				since_message_id: 'msg-9',
				limit: 20
			}
		});
	});

	it('delegates remaining methods to fallback service', async () => {
		const fallback = createFallback();
		const { client } = createApiClient();

		(fallback.getPlan as ReturnType<typeof vi.fn>).mockResolvedValue(null);

		const service = new ApiWitnessService(client, fallback);
		await expect(service.getPlan('witness-a')).resolves.toBeNull();

		expect(fallback.getPlan).toHaveBeenCalledWith('witness-a');
	});

	it('does not use mock fallback when disabled', async () => {
		const fallback = createFallback();
		const { client } = createApiClient();

		const service = new ApiWitnessService(client, fallback, { allowMockFallback: false });
		await expect(service.getPlan('witness-a')).rejects.toThrow(
			'Mock fallback disabled for witness service'
		);
		expect(fallback.getPlan).not.toHaveBeenCalled();
	});
});

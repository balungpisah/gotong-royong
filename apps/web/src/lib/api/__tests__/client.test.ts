import { describe, expect, it, vi } from 'vitest';
import { createApiClient, isApiClientError } from '../index';

describe('Api client error contracts', () => {
	it('parses standard backend error envelope and propagates ids', async () => {
		const fetchFn = vi.fn(async () => {
			return new Response(
				JSON.stringify({
					error: {
						code: 'forbidden',
						message: 'forbidden',
						details: {
							reason: 'scope_mismatch'
						}
					}
				}),
				{
					status: 403,
					headers: {
						'content-type': 'application/json',
						'x-request-id': 'req-123',
						'x-correlation-id': 'corr-123'
					}
				}
			);
		});

		const client = createApiClient({
			baseUrl: '/v1',
			fetchFn
		});

		await expect(client.get('/feed')).rejects.toMatchObject({
			name: 'ApiClientError',
			status: 403,
			code: 'forbidden',
			message: 'forbidden',
			requestId: 'req-123',
			correlationId: 'corr-123',
			details: {
				reason: 'scope_mismatch'
			}
		});
	});

	it('maps unknown error shape to unknown_error fallback', async () => {
		const fetchFn = vi.fn(async () => {
			return new Response(JSON.stringify({ message: 123 }), {
				status: 500,
				headers: {
					'content-type': 'application/json'
				}
			});
		});

		const client = createApiClient({
			baseUrl: '/v1',
			fetchFn
		});

		try {
			await client.get('/notifications');
			throw new Error('expected failure');
		} catch (error) {
			expect(isApiClientError(error)).toBe(true);
			if (!isApiClientError(error)) {
				return;
			}
			expect(error.code).toBe('unknown_error');
			expect(error.status).toBe(500);
			expect(error.message).toContain('GET /v1/notifications failed with status 500');
		}
	});

	it('keeps plain-text error body in details for debugging', async () => {
		const fetchFn = vi.fn(async () => {
			return new Response('upstream unavailable', {
				status: 502,
				headers: {
					'content-type': 'text/plain'
				}
			});
		});

		const client = createApiClient({
			baseUrl: '/v1',
			fetchFn
		});

		try {
			await client.get('/chat/threads');
			throw new Error('expected failure');
		} catch (error) {
			expect(isApiClientError(error)).toBe(true);
			if (!isApiClientError(error)) {
				return;
			}
			expect(error.code).toBe('unknown_error');
			expect(error.status).toBe(502);
			expect(error.details).toBe('upstream unavailable');
		}
	});
});

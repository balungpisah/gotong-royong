import { describe, it, expect, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import { ApiNotificationService } from '../notification-service';

const makeApiClient = () => {
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

describe('ApiNotificationService', () => {
	it('maps paged notification response to AppNotification', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					notification_id: 'notif-1',
					notification_type: 'vouch',
					source_type: 'contribution',
					source_id: 'contrib-1',
					title: 'Ada pembaruan',
					body: 'Kontribusi kamu diverifikasi',
					created_at_ms: 1_700_000_000_000,
					read_at_ms: null
				}
			],
			next_cursor: 'cursor-2'
		});

		const service = new ApiNotificationService(client);
		const page = await service.list({ cursor: 'cursor-1', limit: 10 });

		expect(get).toHaveBeenCalledWith('/notifications', {
			query: {
				cursor: 'cursor-1',
				limit: 10,
				include_read: true
			}
		});
		expect(page.cursor).toBe('cursor-2');
		expect(page.total).toBe(1);
		expect(page.items[0]).toEqual({
			notification_id: 'notif-1',
			type: 'system',
			title: 'Ada pembaruan',
			body: 'Kontribusi kamu diverifikasi',
			read: false,
			created_at: '2023-11-14T22:13:20.000Z'
		});
	});

	it('marks a notification as read', async () => {
		const { client, post } = makeApiClient();
		post.mockResolvedValueOnce({
			notification_id: 'notif-1'
		});

		const service = new ApiNotificationService(client);
		await service.markRead('notif-1');

		expect(post).toHaveBeenCalledWith('/notifications/notif-1/read');
	});

	it('reads unread count endpoint', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			unread_count: 7
		});

		const service = new ApiNotificationService(client);
		await expect(service.getUnreadCount()).resolves.toBe(7);
		expect(get).toHaveBeenCalledWith('/notifications/unread-count');
	});

	it('marks all unread notifications across pages', async () => {
		const { client, get, post } = makeApiClient();
		get
			.mockResolvedValueOnce({
				items: [
					{
						notification_id: 'notif-1',
						notification_type: 'system',
						source_type: 'contribution',
						source_id: 'source-1',
						title: 'Satu',
						body: 'Body 1',
						created_at_ms: 1_700_000_000_000,
						read_at_ms: null
					}
				],
				next_cursor: 'cursor-2'
			})
			.mockResolvedValueOnce({
				items: [
					{
						notification_id: 'notif-2',
						notification_type: 'system',
						source_type: 'contribution',
						source_id: 'source-2',
						title: 'Dua',
						body: 'Body 2',
						created_at_ms: 1_700_000_000_001,
						read_at_ms: null
					}
				],
				next_cursor: null
			});
		post.mockResolvedValue({});

		const service = new ApiNotificationService(client);
		await service.markAllRead();

		expect(get).toHaveBeenNthCalledWith(1, '/notifications', {
			query: {
				cursor: undefined,
				limit: 50,
				include_read: false
			}
		});
		expect(get).toHaveBeenNthCalledWith(2, '/notifications', {
			query: {
				cursor: 'cursor-2',
				limit: 50,
				include_read: false
			}
		});
		expect(post).toHaveBeenCalledTimes(2);
		expect(post).toHaveBeenCalledWith('/notifications/notif-1/read');
		expect(post).toHaveBeenCalledWith('/notifications/notif-2/read');
	});
});

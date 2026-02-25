import type { ApiClient } from '$lib/api';
import type { AppNotification, NotificationType } from '$lib/types';
import type { NotificationService, Paginated } from '../types';

interface ApiNotificationItem {
	notification_id: string;
	notification_type: string;
	source_type: string;
	source_id: string;
	title: string;
	body: string;
	payload?: unknown;
	created_at_ms: number;
	read_at_ms?: number | null;
}

interface ApiNotificationsPage {
	items: ApiNotificationItem[];
	next_cursor?: string | null;
}

interface ApiUnreadCount {
	unread_count: number;
}

const APP_NOTIFICATION_TYPES = new Set<NotificationType>([
	'phase_change',
	'vote_open',
	'evidence_needed',
	'diff_proposed',
	'mention',
	'role_assigned',
	'system'
]);

const toNotificationType = (item: ApiNotificationItem): NotificationType => {
	if (APP_NOTIFICATION_TYPES.has(item.notification_type as NotificationType)) {
		return item.notification_type as NotificationType;
	}
	return 'system';
};

const toIsoTimestamp = (timestampMs: number) => {
	if (!Number.isFinite(timestampMs)) {
		return new Date().toISOString();
	}
	return new Date(timestampMs).toISOString();
};

const toAppNotification = (item: ApiNotificationItem): AppNotification => ({
	notification_id: item.notification_id,
	type: toNotificationType(item),
	title: item.title,
	body: item.body,
	read: item.read_at_ms !== null && item.read_at_ms !== undefined,
	created_at: toIsoTimestamp(item.created_at_ms)
});

export class ApiNotificationService implements NotificationService {
	private readonly client: ApiClient;

	constructor(client: ApiClient) {
		this.client = client;
	}

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<AppNotification>> {
		const response = await this.client.get<ApiNotificationsPage>('/notifications', {
			query: {
				cursor: opts?.cursor,
				limit: opts?.limit,
				include_read: true
			}
		});

		return {
			items: response.items.map(toAppNotification),
			total: response.items.length,
			cursor: response.next_cursor ?? undefined
		};
	}

	async markRead(notificationId: string): Promise<void> {
		await this.client.post(`/notifications/${notificationId}/read`);
	}

	async markAllRead(): Promise<void> {
		let cursor: string | undefined;

		while (true) {
			const response = await this.client.get<ApiNotificationsPage>('/notifications', {
				query: {
					cursor,
					limit: 50,
					include_read: false
				}
			});

			const items = response.items;
			if (items.length === 0) {
				return;
			}

			const nextCursor = response.next_cursor ?? undefined;
			await Promise.all(items.map((item) => this.markRead(item.notification_id)));

			if (!nextCursor) {
				return;
			}
			cursor = nextCursor;
		}
	}

	async getUnreadCount(): Promise<number> {
		const response = await this.client.get<ApiUnreadCount>('/notifications/unread-count');
		return response.unread_count;
	}
}

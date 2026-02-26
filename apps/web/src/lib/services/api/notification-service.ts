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

type JsonRecord = Record<string, unknown>;

const APP_NOTIFICATION_TYPES = new Set<NotificationType>([
	'phase_change',
	'vote_open',
	'evidence_needed',
	'diff_proposed',
	'mention',
	'role_assigned',
	'system'
]);

const WITNESS_SOURCE_TYPES = new Set(['contribution', 'witness']);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim() ? value.trim() : undefined;

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

const normalizeTargetPath = (value: string | undefined): string | undefined => {
	if (!value) {
		return undefined;
	}
	if (value.startsWith('//') || /^[a-z][a-z0-9+\-.]*:\/\//i.test(value)) {
		return undefined;
	}
	return value.startsWith('/') ? value : `/${value}`;
};

const resolveWitnessIdFromPayload = (payload: JsonRecord | undefined): string | undefined => {
	const target = isRecord(payload?.target) ? payload.target : undefined;
	const context = isRecord(payload?.context) ? payload.context : undefined;
	return (
		asString(payload?.witness_id) ?? asString(target?.witness_id) ?? asString(context?.witness_id)
	);
};

const resolveWitnessId = (
	item: ApiNotificationItem,
	payload: JsonRecord | undefined
): string | undefined => {
	const fromPayload = resolveWitnessIdFromPayload(payload);
	if (fromPayload) {
		return fromPayload;
	}
	if (WITNESS_SOURCE_TYPES.has(item.source_type)) {
		return asString(item.source_id);
	}
	return undefined;
};

const resolveTargetPath = (
	payload: JsonRecord | undefined,
	witnessId: string | undefined
): string | undefined => {
	const target = isRecord(payload?.target) ? payload.target : undefined;
	const context = isRecord(payload?.context) ? payload.context : undefined;
	const explicitPath =
		asString(payload?.target_path) ??
		asString(payload?.path) ??
		asString(target?.path) ??
		asString(context?.target_path);
	const normalizedExplicitPath = normalizeTargetPath(explicitPath);
	if (normalizedExplicitPath) {
		return normalizedExplicitPath;
	}
	if (!witnessId) {
		return undefined;
	}
	return `/saksi/${encodeURIComponent(witnessId)}`;
};

const toAppNotification = (item: ApiNotificationItem): AppNotification => {
	const payload = isRecord(item.payload) ? item.payload : undefined;
	const witnessId = resolveWitnessId(item, payload);
	const targetPath = resolveTargetPath(payload, witnessId);

	return {
		notification_id: item.notification_id,
		type: toNotificationType(item),
		title: item.title,
		body: item.body,
		witness_id: witnessId,
		target_path: targetPath,
		read: item.read_at_ms !== null && item.read_at_ms !== undefined,
		created_at: toIsoTimestamp(item.created_at_ms)
	};
};

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

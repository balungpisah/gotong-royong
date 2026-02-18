import type { NotificationService, Paginated } from '../types';
import type { AppNotification } from '$lib/types';
import { mockNotifications } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockNotificationService implements NotificationService {
	private notifications = mockNotifications.map((n) => ({ ...n }));

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<AppNotification>> {
		await delay();
		const limit = opts?.limit ?? 20;
		return {
			items: this.notifications.slice(0, limit),
			total: this.notifications.length
		};
	}

	async markRead(notificationId: string): Promise<void> {
		await delay(100);
		const notif = this.notifications.find((n) => n.notification_id === notificationId);
		if (notif) {
			notif.read = true;
		}
	}

	async markAllRead(): Promise<void> {
		await delay(100);
		this.notifications.forEach((n) => {
			n.read = true;
		});
	}

	async getUnreadCount(): Promise<number> {
		await delay(100);
		return this.notifications.filter((n) => !n.read).length;
	}
}

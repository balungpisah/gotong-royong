/**
 * Notification Store — manages the notification feed and unread counts.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Consumes NotificationService interface for data operations.
 */

import type { NotificationService } from '$lib/services/types';
import type { AppNotification } from '$lib/types';

export class NotificationStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	notifications = $state<AppNotification[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	unreadCount = $derived(this.notifications.filter((n) => !n.read).length);
	hasUnread = $derived(this.unreadCount > 0);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	private readonly service: NotificationService;

	constructor(service: NotificationService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async loadNotifications(opts?: { limit?: number }) {
		this.loading = true;
		this.error = null;
		try {
			const result = await this.service.list(opts);
			this.notifications = result.items;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat notifikasi';
		} finally {
			this.loading = false;
		}
	}

	async markRead(notificationId: string) {
		try {
			await this.service.markRead(notificationId);
			this.notifications = this.notifications.map((n) =>
				n.notification_id === notificationId ? { ...n, read: true } : n
			);
		} catch (err) {
			console.error('[NotificationStore] markRead failed:', err);
			throw err;
		}
	}

	async markAllRead() {
		try {
			await this.service.markAllRead();
			this.notifications = this.notifications.map((n) => ({ ...n, read: true }));
		} catch (err) {
			console.error('[NotificationStore] markAllRead failed:', err);
			throw err;
		}
	}

	async refresh() {
		await this.loadNotifications();
	}
}

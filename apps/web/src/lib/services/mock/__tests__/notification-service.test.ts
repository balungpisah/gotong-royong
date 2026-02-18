import { describe, it, expect } from 'vitest';
import { MockNotificationService } from '../notification-service.mock';

describe('MockNotificationService', () => {
	it('list() returns all notifications', async () => {
		const service = new MockNotificationService();
		const result = await service.list();
		expect(result.items.length).toBe(10);
		expect(result.total).toBe(10);
	});

	it('list() respects limit', async () => {
		const service = new MockNotificationService();
		const result = await service.list({ limit: 3 });
		expect(result.items.length).toBe(3);
		expect(result.total).toBe(10);
	});

	it('getUnreadCount() returns correct count', async () => {
		const service = new MockNotificationService();
		const count = await service.getUnreadCount();
		expect(count).toBe(6); // 6 unread in mock data
	});

	it('markRead() marks a notification as read', async () => {
		const service = new MockNotificationService();
		const beforeCount = await service.getUnreadCount();

		// Mark the first notification (which is unread) as read
		await service.markRead('notif-001');

		const afterCount = await service.getUnreadCount();
		expect(afterCount).toBe(beforeCount - 1);

		// Verify it's actually read in the list
		const result = await service.list();
		const notif = result.items.find((n) => n.notification_id === 'notif-001');
		expect(notif?.read).toBe(true);
	});

	it('markAllRead() marks all notifications as read', async () => {
		const service = new MockNotificationService();
		await service.markAllRead();

		const count = await service.getUnreadCount();
		expect(count).toBe(0);

		const result = await service.list();
		expect(result.items.every((n) => n.read)).toBe(true);
	});

	it('each instance has independent state', async () => {
		const service1 = new MockNotificationService();
		const service2 = new MockNotificationService();

		await service1.markAllRead();

		const count1 = await service1.getUnreadCount();
		const count2 = await service2.getUnreadCount();

		expect(count1).toBe(0);
		expect(count2).toBe(6); // service2 is independent
	});
});

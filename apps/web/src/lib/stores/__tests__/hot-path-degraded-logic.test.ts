import { describe, expect, it } from 'vitest';

type AppNotification = {
	notification_id: string;
	read: boolean;
};

type NotificationServiceLike = {
	list: () => Promise<{ items: AppNotification[] }>;
	markRead: (notificationId: string) => Promise<void>;
	markAllRead: () => Promise<void>;
};

/**
 * Plain JS mirror of NotificationStore degraded-state behavior.
 * We keep this logic test outside Svelte runes to validate error/retry contracts.
 */
class NotificationStoreLogic {
	notifications: AppNotification[] = [];
	loading = false;
	error: string | null = null;

	constructor(private readonly service: NotificationServiceLike) {}

	async loadNotifications() {
		this.loading = true;
		this.error = null;
		try {
			const result = await this.service.list();
			this.notifications = result.items;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat notifikasi';
		} finally {
			this.loading = false;
		}
	}

	async markRead(notificationId: string) {
		this.error = null;
		try {
			await this.service.markRead(notificationId);
			this.notifications = this.notifications.map((n) =>
				n.notification_id === notificationId ? { ...n, read: true } : n
			);
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal menandai notifikasi';
			throw err;
		}
	}

	async markAllRead() {
		this.error = null;
		try {
			await this.service.markAllRead();
			this.notifications = this.notifications.map((n) => ({ ...n, read: true }));
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal menandai semua notifikasi';
			throw err;
		}
	}
}

type TriageSubmitState = {
	content: string;
	messageCount: number;
	attachmentCount: number;
	submitError: string | null;
};

type ContentSignalType = 'saksi' | 'perlu_dicek';

type SignalServiceLike = {
	sendSignal: (witnessId: string, signalType: ContentSignalType) => Promise<{ signal_id: string }>;
	removeSignal: (witnessId: string, signalType: ContentSignalType) => Promise<void>;
	getMyRelation: (witnessId: string) => Promise<{ witnessed: boolean; flagged: boolean }>;
	getSignalCounts: (witnessId: string) => Promise<{ witness_count: number; flags: number }>;
};

class SignalStoreLogic {
	relations = new Map<string, { witnessed: boolean; flagged: boolean }>();
	counts = new Map<string, { witness_count: number; flags: number }>();
	errors = new Map<string, string>();
	sending = false;

	constructor(private readonly service: SignalServiceLike) {}

	getError(witnessId: string): string | null {
		return this.errors.get(witnessId) ?? null;
	}

	private setError(witnessId: string, message: string) {
		this.errors = new Map(this.errors).set(witnessId, message);
	}

	clearError(witnessId: string) {
		if (!this.errors.has(witnessId)) return;
		const next = new Map(this.errors);
		next.delete(witnessId);
		this.errors = next;
	}

	async loadRelation(witnessId: string) {
		const relation = await this.service.getMyRelation(witnessId);
		this.relations = new Map(this.relations).set(witnessId, relation);
	}

	async loadCounts(witnessId: string) {
		const counts = await this.service.getSignalCounts(witnessId);
		this.counts = new Map(this.counts).set(witnessId, counts);
	}

	async refreshWitness(witnessId: string) {
		this.clearError(witnessId);
		await Promise.all([this.loadRelation(witnessId), this.loadCounts(witnessId)]);
		this.clearError(witnessId);
	}

	async toggleSignal(witnessId: string, signalType: ContentSignalType) {
		if (this.sending) return;
		this.sending = true;
		this.clearError(witnessId);
		try {
			const relation = this.relations.get(witnessId);
			const isActive = signalType === 'saksi' ? relation?.witnessed : relation?.flagged;
			if (isActive) {
				await this.service.removeSignal(witnessId, signalType);
			} else {
				await this.service.sendSignal(witnessId, signalType);
			}
			await this.refreshWitness(witnessId);
		} catch (err) {
			this.setError(witnessId, err instanceof Error ? err.message : 'signal failed');
			throw err;
		} finally {
			this.sending = false;
		}
	}
}

type FeedSignalView = {
	relation?: {
		vouched: boolean;
		vouch_type?: 'positive' | 'skeptical' | 'conditional' | 'mentorship';
		witnessed: boolean;
		flagged: boolean;
		supported: boolean;
		vote_cast?: 'yes' | 'no';
	};
	counts?: {
		vouch_positive: number;
		vouch_skeptical: number;
		witness_count: number;
		dukung_count: number;
		flags: number;
	};
};

function mergeFeedSignalView(
	base: FeedSignalView,
	live: {
		relation?: { witnessed: boolean; flagged: boolean };
		counts?: { witness_count: number; flags: number };
	}
): FeedSignalView {
	const relation = (() => {
		if (!base.relation && !live.relation) return undefined;
		const safeBase = base.relation ?? {
			vouched: false,
			witnessed: false,
			flagged: false,
			supported: false
		};
		if (!live.relation) return safeBase;
		return {
			...safeBase,
			witnessed: live.relation.witnessed,
			flagged: live.relation.flagged
		};
	})();

	const counts = (() => {
		if (!base.counts && !live.counts) return undefined;
		const safeBase = base.counts ?? {
			vouch_positive: 0,
			vouch_skeptical: 0,
			witness_count: 0,
			dukung_count: 0,
			flags: 0
		};
		if (!live.counts) return safeBase;
		return {
			...safeBase,
			witness_count: live.counts.witness_count,
			flags: live.counts.flags
		};
	})();

	return { relation, counts };
}

function applyTriageSubmitResult(
	state: TriageSubmitState,
	triageError: string | null
): TriageSubmitState {
	if (triageError) {
		return { ...state, submitError: triageError };
	}

	return {
		content: '',
		messageCount: state.messageCount + 2,
		attachmentCount: 0,
		submitError: null
	};
}

describe('hot-path degraded UX logic', () => {
	it('keeps stale notifications when reload fails and exposes error', async () => {
		const service: NotificationServiceLike = {
			list: async () => {
				throw new Error('backend down');
			},
			markRead: async () => undefined,
			markAllRead: async () => undefined
		};
		const store = new NotificationStoreLogic(service);
		store.notifications = [{ notification_id: 'n-1', read: false }];

		await store.loadNotifications();

		expect(store.loading).toBe(false);
		expect(store.error).toBe('backend down');
		expect(store.notifications).toHaveLength(1);
	});

	it('sets action error when mark-read fails', async () => {
		const service: NotificationServiceLike = {
			list: async () => ({ items: [] }),
			markRead: async () => {
				throw new Error('write blocked');
			},
			markAllRead: async () => undefined
		};
		const store = new NotificationStoreLogic(service);
		store.notifications = [{ notification_id: 'n-1', read: false }];

		await expect(store.markRead('n-1')).rejects.toThrow('write blocked');
		expect(store.error).toBe('write blocked');
		expect(store.notifications[0].read).toBe(false);
	});

	it('keeps triage draft when submit fails and clears draft on success', () => {
		const start: TriageSubmitState = {
			content: 'jalan rusak',
			messageCount: 0,
			attachmentCount: 1,
			submitError: null
		};

		const failed = applyTriageSubmitResult(start, 'service unavailable');
		expect(failed.content).toBe('jalan rusak');
		expect(failed.attachmentCount).toBe(1);
		expect(failed.messageCount).toBe(0);
		expect(failed.submitError).toBe('service unavailable');

		const success = applyTriageSubmitResult(failed, null);
		expect(success.content).toBe('');
		expect(success.attachmentCount).toBe(0);
		expect(success.messageCount).toBe(2);
		expect(success.submitError).toBeNull();
	});

	it('sets per-witness signal error and clears after successful refresh', async () => {
		let failOnce = true;
		const service: SignalServiceLike = {
			sendSignal: async () => {
				if (failOnce) {
					failOnce = false;
					throw new Error('signal backend unavailable');
				}
				return { signal_id: 'sig-1' };
			},
			removeSignal: async () => undefined,
			getMyRelation: async () => ({ witnessed: true, flagged: false }),
			getSignalCounts: async () => ({ witness_count: 1, flags: 0 })
		};

		const store = new SignalStoreLogic(service);
		const witnessId = 'wit-1';
		store.relations.set(witnessId, { witnessed: false, flagged: false });

		await expect(store.toggleSignal(witnessId, 'saksi')).rejects.toThrow(
			'signal backend unavailable'
		);
		expect(store.getError(witnessId)).toBe('signal backend unavailable');

		await expect(store.toggleSignal(witnessId, 'saksi')).resolves.toBeUndefined();
		expect(store.getError(witnessId)).toBeNull();
		expect(store.relations.get(witnessId)?.witnessed).toBe(true);
		expect(store.counts.get(witnessId)?.witness_count).toBe(1);
	});

	it('merges live signal relation/counts without overwriting social metrics', () => {
		const merged = mergeFeedSignalView(
			{
				relation: {
					vouched: true,
					vouch_type: 'positive',
					witnessed: false,
					flagged: false,
					supported: true,
					vote_cast: 'yes'
				},
				counts: {
					vouch_positive: 4,
					vouch_skeptical: 1,
					witness_count: 0,
					dukung_count: 3,
					flags: 0
				}
			},
			{
				relation: { witnessed: true, flagged: true },
				counts: { witness_count: 7, flags: 2 }
			}
		);

		expect(merged.relation?.witnessed).toBe(true);
		expect(merged.relation?.flagged).toBe(true);
		expect(merged.relation?.supported).toBe(true);
		expect(merged.counts?.witness_count).toBe(7);
		expect(merged.counts?.flags).toBe(2);
		expect(merged.counts?.dukung_count).toBe(3);
		expect(merged.counts?.vouch_positive).toBe(4);
	});
});

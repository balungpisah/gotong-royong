import { describe, it, expect } from 'vitest';
import { MockWitnessService } from '../witness-service.mock';

describe('MockWitnessService', () => {
	it('list() returns all witnesses', async () => {
		const service = new MockWitnessService();
		const result = await service.list();
		expect(result.items.length).toBe(5);
		expect(result.total).toBe(5);
		expect(result.items[0].witness_id).toBe('witness-001');
	});

	it('list() filters by status', async () => {
		const service = new MockWitnessService();
		const result = await service.list({ status: 'active' });
		expect(result.items.every((w) => w.status === 'active')).toBe(true);
		expect(result.items.length).toBeGreaterThan(0);
	});

	it('get() returns witness detail', async () => {
		const service = new MockWitnessService();
		const detail = await service.get('witness-001');
		expect(detail.witness_id).toBe('witness-001');
		expect(detail.messages).toBeDefined();
		expect(detail.messages.length).toBeGreaterThan(0);
		expect(detail.plan).toBeDefined();
		expect(detail.blocks).toBeDefined();
		expect(detail.members).toBeDefined();
	});

	it('getMessages() returns paginated messages', async () => {
		const service = new MockWitnessService();
		const result = await service.getMessages('witness-001');
		expect(result.items.length).toBeGreaterThan(0);
		expect(result.total).toBeGreaterThan(0);
	});

	it('sendMessage() appends a new message and returns it', async () => {
		const service = new MockWitnessService();
		const before = await service.getMessages('witness-001');
		const beforeCount = before.items.length;

		const newMsg = await service.sendMessage('witness-001', 'Test message from vitest');
		expect(newMsg.type).toBe('user');
		expect(newMsg.witness_id).toBe('witness-001');
		// Check it has content (it's a UserMessage)
		expect((newMsg as any).content).toBe('Test message from vitest');
		expect((newMsg as any).is_self).toBe(true);

		const after = await service.getMessages('witness-001');
		expect(after.items.length).toBe(beforeCount + 1);
	});

	it('getPlan() returns a path plan', async () => {
		const service = new MockWitnessService();
		const plan = await service.getPlan('witness-001');
		expect(plan).not.toBeNull();
		expect(plan?.plan_id).toBeDefined();
		expect(plan?.branches.length).toBeGreaterThan(0);
	});

	it('respondToDiff() resolves without error', async () => {
		const service = new MockWitnessService();
		await expect(
			service.respondToDiff('witness-001', 'diff-001', {
				diff_id: 'diff-001',
				action: 'apply_all'
			})
		).resolves.toBeUndefined();
	});

	it('castVote() resolves without error', async () => {
		const service = new MockWitnessService();
		await expect(service.castVote('witness-001', 'vote-001', 'opt-1')).resolves.toBeUndefined();
	});
});

import { describe, expect, it, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import type {
	GroupDetail,
	GroupMember,
	GroupSummary,
	GroupUpdateInput,
	MembershipRequest
} from '$lib/types';
import type { GroupService, Paginated } from '$lib/services/types';
import { ApiGroupService } from '../group-service';

const makeApiClient = () => {
	const get = vi.fn();
	const post = vi.fn();
	const patch = vi.fn();
	const client = {
		request: vi.fn(),
		get,
		post,
		put: vi.fn(),
		patch,
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, get, post, patch };
};

const makeSummary = (): GroupSummary => ({
	group_id: 'g-1',
	name: 'Karang Taruna RT 07',
	description: 'Wadah pemuda.',
	entity_type: 'kelompok',
	join_policy: 'persetujuan',
	member_count: 2,
	witness_count: 5,
	entity_tag: {
		entity_id: 'g-1',
		entity_type: 'kelompok',
		label: 'Karang Taruna RT 07',
		followed: false
	}
});

const makeMember = (): GroupMember => ({
	user_id: 'u-2',
	name: 'Budi',
	avatar_url: 'https://example.com/u-2.png',
	role: 'anggota',
	joined_at: '2026-02-25T00:00:00.000Z'
});

const makeRequest = (): MembershipRequest => ({
	request_id: 'req-1',
	user_id: 'u-3',
	name: 'Sari',
	avatar_url: 'https://example.com/u-3.png',
	message: 'Mau ikut ronda',
	status: 'pending',
	requested_at: '2026-02-25T01:00:00.000Z'
});

const makeDetail = (): GroupDetail => ({
	...makeSummary(),
	members: [
		{
			user_id: 'u-1',
			name: 'Admin',
			role: 'admin',
			joined_at: '2026-02-24T00:00:00.000Z'
		},
		makeMember()
	],
	pending_requests: [makeRequest()],
	my_role: 'admin',
	my_membership_status: 'approved'
});

const toApiDetail = (detail: GroupDetail) => ({
	group_id: detail.group_id,
	name: detail.name,
	description: detail.description,
	entity_type: detail.entity_type,
	join_policy: detail.join_policy,
	member_count: detail.member_count,
	witness_count: detail.witness_count,
	entity_tag: detail.entity_tag,
	members: detail.members,
	pending_requests: detail.pending_requests,
	my_role: detail.my_role,
	my_membership_status: detail.my_membership_status
});

const makeFallbackService = () => {
	const detail = makeDetail();
	const summary = makeSummary();
	const member = makeMember();
	const request = makeRequest();
	const paged: Paginated<GroupSummary> = { items: [summary], total: 1 };

	const service: GroupService = {
		create: vi.fn(async () => detail),
		list: vi.fn(async () => paged),
		listMyGroups: vi.fn(async () => [summary]),
		get: vi.fn(async () => detail),
		update: vi.fn(async () => detail),
		join: vi.fn(async () => member),
		requestJoin: vi.fn(async () => request),
		approveRequest: vi.fn(async () => member),
		rejectRequest: vi.fn(async () => undefined),
		invite: vi.fn(async () => undefined),
		leave: vi.fn(async () => undefined),
		removeMember: vi.fn(async () => undefined),
		updateMemberRole: vi.fn(async () => undefined)
	};
	return { service, detail, summary, member, request, paged };
};

describe('ApiGroupService', () => {
	it('maps group list/detail and mutation responses', async () => {
		const { client, get, post, patch } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		const detail = makeDetail();
		const summary = makeSummary();
		const member = makeMember();
		const request = makeRequest();

		get.mockImplementation(async (path: string) => {
			if (path === '/groups') {
				return {
					items: [summary],
					total: 1
				};
			}
			if (path === '/groups/me') {
				return [summary];
			}
			if (path === '/groups/g-1') {
				return toApiDetail(detail);
			}
			throw new Error(`unexpected path ${path}`);
		});

		post.mockImplementation(async (path: string) => {
			if (path === '/groups') return toApiDetail(detail);
			if (path === '/groups/g-1/join') return member;
			if (path === '/groups/g-1/requests') return request;
			if (path === '/groups/g-1/requests/req-1/approve') return member;
			if (path === '/groups/g-1/requests/req-1/reject') return { rejected: true };
			if (path === '/groups/g-1/invite') return { added: true };
			if (path === '/groups/g-1/leave') return { left: true };
			if (path === '/groups/g-1/members/u-3/remove') return { removed: true };
			if (path === '/groups/g-1/members/u-2/role') return { updated: true };
			throw new Error(`unexpected path ${path}`);
		});

		patch.mockResolvedValue(toApiDetail({ ...detail, name: 'Karang Taruna RT 07 (Aktif)' }));

		const service = new ApiGroupService(client, fallback);
		await expect(
			service.create({
				name: 'Karang Taruna RT 07',
				description: 'Wadah pemuda.',
				entity_type: 'kelompok',
				join_policy: 'persetujuan'
			})
		).resolves.toMatchObject({ group_id: 'g-1', my_role: 'admin' });
		await expect(service.list({ limit: 10 })).resolves.toMatchObject({
			total: 1,
			items: [{ group_id: 'g-1' }]
		});
		await expect(service.listMyGroups()).resolves.toMatchObject([{ group_id: 'g-1' }]);
		await expect(service.get('g-1')).resolves.toMatchObject({
			group_id: 'g-1',
			members: expect.any(Array)
		});
		const updated = await service.update('g-1', {
			name: 'Karang Taruna RT 07 (Aktif)'
		} as GroupUpdateInput);
		expect(updated.name).toBe('Karang Taruna RT 07 (Aktif)');
		await expect(service.join('g-1')).resolves.toMatchObject({ user_id: 'u-2' });
		await expect(service.requestJoin('g-1', 'Mau ikut ronda')).resolves.toMatchObject({
			request_id: 'req-1'
		});
		await expect(service.approveRequest('g-1', 'req-1')).resolves.toMatchObject({
			user_id: 'u-2'
		});
		await expect(service.rejectRequest('g-1', 'req-1')).resolves.toBeUndefined();
		await expect(service.invite('g-1', 'u-3')).resolves.toBeUndefined();
		await expect(service.leave('g-1')).resolves.toBeUndefined();
		await expect(service.removeMember('g-1', 'u-3')).resolves.toBeUndefined();
		await expect(service.updateMemberRole('g-1', 'u-2', 'moderator')).resolves.toBeUndefined();

		expect(get).toHaveBeenCalledWith('/groups', {
			query: { cursor: undefined, limit: 10 }
		});
		expect(post).toHaveBeenCalledWith('/groups/g-1/requests', {
			body: { message: 'Mau ikut ronda' }
		});
		expect(post).toHaveBeenCalledWith('/groups/g-1/invite', {
			body: { user_id: 'u-3' }
		});
		expect(post).toHaveBeenCalledWith('/groups/g-1/members/u-2/role', {
			body: { role: 'moderator' }
		});
	});

	it('falls back when backend is unavailable', async () => {
		const { client, get, post, patch } = makeApiClient();
		const { service: fallback, detail, paged, summary, member, request } = makeFallbackService();
		get.mockRejectedValue(new Error('backend unavailable'));
		post.mockRejectedValue(new Error('backend unavailable'));
		patch.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiGroupService(client, fallback);
		await expect(
			service.create({
				name: 'Fallback Group',
				description: 'Fallback',
				entity_type: 'kelompok',
				join_policy: 'terbuka'
			})
		).resolves.toEqual(detail);
		await expect(service.list()).resolves.toEqual(paged);
		await expect(service.listMyGroups()).resolves.toEqual([summary]);
		await expect(service.get('g-fallback')).resolves.toEqual(detail);
		await expect(service.update('g-fallback', { name: 'updated' })).resolves.toEqual(detail);
		await expect(service.join('g-fallback')).resolves.toEqual(member);
		await expect(service.requestJoin('g-fallback')).resolves.toEqual(request);
		await expect(service.approveRequest('g-fallback', 'req-fallback')).resolves.toEqual(member);
		await expect(service.rejectRequest('g-fallback', 'req-fallback')).resolves.toBeUndefined();
		await expect(service.invite('g-fallback', 'u-9')).resolves.toBeUndefined();
		await expect(service.leave('g-fallback')).resolves.toBeUndefined();
		await expect(service.removeMember('g-fallback', 'u-9')).resolves.toBeUndefined();
		await expect(service.updateMemberRole('g-fallback', 'u-9', 'anggota')).resolves.toBeUndefined();

		expect(fallback.create).toHaveBeenCalled();
		expect(fallback.list).toHaveBeenCalled();
		expect(fallback.listMyGroups).toHaveBeenCalled();
		expect(fallback.get).toHaveBeenCalled();
		expect(fallback.update).toHaveBeenCalled();
		expect(fallback.join).toHaveBeenCalled();
		expect(fallback.requestJoin).toHaveBeenCalled();
		expect(fallback.approveRequest).toHaveBeenCalled();
		expect(fallback.rejectRequest).toHaveBeenCalled();
		expect(fallback.invite).toHaveBeenCalled();
		expect(fallback.leave).toHaveBeenCalled();
		expect(fallback.removeMember).toHaveBeenCalled();
		expect(fallback.updateMemberRole).toHaveBeenCalled();
	});

	it('does not use mock fallback when disabled', async () => {
		const { client, get } = makeApiClient();
		const { service: fallback } = makeFallbackService();
		get.mockRejectedValue(new Error('backend unavailable'));

		const service = new ApiGroupService(client, fallback, { allowMockFallback: false });
		await expect(service.list()).rejects.toThrow('backend unavailable');
		expect(fallback.list).not.toHaveBeenCalled();
	});
});

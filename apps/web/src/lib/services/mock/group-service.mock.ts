import type { GroupService, Paginated } from '../types';
import type {
	GroupCreateInput,
	GroupDetail,
	GroupMember,
	GroupMemberRole,
	GroupSummary,
	GroupUpdateInput,
	MembershipRequest
} from '$lib/types';
import { mockCurrentUser, mockGroups } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

const currentUser = {
	user_id: mockCurrentUser.user_id,
	name: mockCurrentUser.name,
	avatar_url: mockCurrentUser.avatar_url
};

function isDiscoverable(group: GroupDetail): boolean {
	return group.join_policy !== 'undangan';
}

function isMember(group: GroupDetail, userId: string): boolean {
	return group.members.some((m) => m.user_id === userId);
}

function computeMy(group: GroupDetail, userId: string): Pick<GroupDetail, 'my_role' | 'my_membership_status'> {
	const member = group.members.find((m) => m.user_id === userId);
	if (member) return { my_role: member.role, my_membership_status: 'approved' };

	const pending = group.pending_requests.some((r) => r.user_id === userId && r.status === 'pending');
	if (pending) return { my_role: null, my_membership_status: 'pending' };

	return { my_role: null, my_membership_status: 'none' };
}

function toSummary(group: GroupDetail): GroupSummary {
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { members, pending_requests, my_role, my_membership_status, ...summary } = group;
	return summary;
}

export class MockGroupService implements GroupService {
	private groups: GroupDetail[] = mockGroups.map((g) => ({
		...g,
		members: g.members.map((m) => ({ ...m })),
		pending_requests: g.pending_requests.map((r) => ({ ...r }))
	}));

	// ---------------------------------------------------------------------------
	// Queries
	// ---------------------------------------------------------------------------

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<GroupSummary>> {
		await delay();
		const limit = opts?.limit ?? 20;
		const items = this.groups.filter(isDiscoverable).map(toSummary);
		return { items: items.slice(0, limit), total: items.length };
	}

	async listMyGroups(): Promise<GroupSummary[]> {
		await delay();
		return this.groups.filter((g) => isMember(g, currentUser.user_id)).map(toSummary);
	}

	async get(groupId: string): Promise<GroupDetail> {
		await delay();
		const group = this.groups.find((g) => g.group_id === groupId);
		if (!group) throw new Error('Kelompok tidak ditemukan');
		const my = computeMy(group, currentUser.user_id);
		return { ...group, ...my, members: [...group.members], pending_requests: [...group.pending_requests] };
	}

	// ---------------------------------------------------------------------------
	// Mutations
	// ---------------------------------------------------------------------------

	async create(input: GroupCreateInput): Promise<GroupDetail> {
		await delay(400);
		const now = new Date().toISOString();
		const groupId = `ent-${Date.now()}`;

		const group: GroupDetail = {
			group_id: groupId,
			name: input.name,
			description: input.description,
			entity_type: input.entity_type,
			join_policy: input.join_policy,
			member_count: 1,
			witness_count: 0,
			entity_tag: {
				entity_id: groupId,
				entity_type: input.entity_type,
				label: input.name,
				followed: false
			},
			members: [
				{
					user_id: currentUser.user_id,
					name: currentUser.name,
					avatar_url: currentUser.avatar_url,
					role: 'admin',
					joined_at: now
				}
			],
			pending_requests: [],
			my_role: 'admin',
			my_membership_status: 'approved'
		};

		this.groups = [group, ...this.groups];
		console.log('[MockGroupService] create:', { input, groupId });
		return group;
	}

	async update(groupId: string, input: GroupUpdateInput): Promise<GroupDetail> {
		await delay(250);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const prev = this.groups[i];
		const next: GroupDetail = {
			...prev,
			...input,
			entity_tag: {
				...prev.entity_tag,
				label: input.name ?? prev.entity_tag.label
			}
		};

		this.groups[i] = next;
		console.log('[MockGroupService] update:', { groupId, input });
		return this.get(groupId);
	}

	async join(groupId: string): Promise<GroupMember> {
		await delay(250);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		if (group.join_policy !== 'terbuka') {
			throw new Error('Kelompok ini tidak bisa langsung digabung. Gunakan permintaan bergabung.');
		}

		const existing = group.members.find((m) => m.user_id === currentUser.user_id);
		if (existing) return existing;

		const now = new Date().toISOString();
		const newMember: GroupMember = {
			user_id: currentUser.user_id,
			name: currentUser.name,
			avatar_url: currentUser.avatar_url,
			role: 'anggota',
			joined_at: now
		};

		group.members = [...group.members, newMember];
		group.member_count = group.members.length;
		group.pending_requests = group.pending_requests.filter((r) => r.user_id !== currentUser.user_id);

		console.log('[MockGroupService] join:', { groupId });
		return newMember;
	}

	async requestJoin(groupId: string, message?: string): Promise<MembershipRequest> {
		await delay(250);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		if (group.join_policy !== 'persetujuan') {
			throw new Error('Kelompok ini tidak menerima permintaan bergabung.');
		}
		if (isMember(group, currentUser.user_id)) {
			throw new Error('Kamu sudah menjadi anggota.');
		}

		const existing = group.pending_requests.find(
			(r) => r.user_id === currentUser.user_id && r.status === 'pending'
		);
		if (existing) return existing;

		const request: MembershipRequest = {
			request_id: `req-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
			user_id: currentUser.user_id,
			name: currentUser.name,
			avatar_url: currentUser.avatar_url,
			message,
			status: 'pending',
			requested_at: new Date().toISOString()
		};

		group.pending_requests = [request, ...group.pending_requests];
		console.log('[MockGroupService] requestJoin:', { groupId, message });
		return request;
	}

	async approveRequest(groupId: string, requestId: string): Promise<GroupMember> {
		await delay(250);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		const request = group.pending_requests.find((r) => r.request_id === requestId);
		if (!request) throw new Error('Permintaan tidak ditemukan');

		const now = new Date().toISOString();
		const newMember: GroupMember = {
			user_id: request.user_id,
			name: request.name,
			avatar_url: request.avatar_url,
			role: 'anggota',
			joined_at: now
		};

		group.pending_requests = group.pending_requests.filter((r) => r.request_id !== requestId);
		group.members = [...group.members, newMember];
		group.member_count = group.members.length;

		console.log('[MockGroupService] approveRequest:', { groupId, requestId });
		return newMember;
	}

	async rejectRequest(groupId: string, requestId: string): Promise<void> {
		await delay(250);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		group.pending_requests = group.pending_requests.map((r) =>
			r.request_id === requestId ? { ...r, status: 'rejected' as const } : r
		);
		console.log('[MockGroupService] rejectRequest:', { groupId, requestId });
	}

	async invite(groupId: string, userId: string): Promise<void> {
		await delay(200);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		if (group.members.some((m) => m.user_id === userId)) return;

		const newMember: GroupMember = {
			user_id: userId,
			name: userId,
			role: 'anggota',
			joined_at: new Date().toISOString()
		};

		group.members = [...group.members, newMember];
		group.member_count = group.members.length;
		console.log('[MockGroupService] invite:', { groupId, userId });
	}

	async leave(groupId: string): Promise<void> {
		await delay(200);
		await this.removeMember(groupId, currentUser.user_id);
		console.log('[MockGroupService] leave:', { groupId });
	}

	async removeMember(groupId: string, userId: string): Promise<void> {
		await delay(200);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		group.members = group.members.filter((m) => m.user_id !== userId);
		group.member_count = group.members.length;
		group.pending_requests = group.pending_requests.filter((r) => r.user_id !== userId);
		console.log('[MockGroupService] removeMember:', { groupId, userId });
	}

	async updateMemberRole(groupId: string, userId: string, role: GroupMemberRole): Promise<void> {
		await delay(200);
		const i = this.groups.findIndex((g) => g.group_id === groupId);
		if (i < 0) throw new Error('Kelompok tidak ditemukan');

		const group = this.groups[i];
		group.members = group.members.map((m) => (m.user_id === userId ? { ...m, role } : m));
		console.log('[MockGroupService] updateMemberRole:', { groupId, userId, role });
	}
}


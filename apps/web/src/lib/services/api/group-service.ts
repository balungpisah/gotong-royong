import type { ApiClient } from '$lib/api';
import type {
	GroupDetail,
	GroupJoinPolicy,
	GroupMember,
	GroupMemberRole,
	GroupSummary,
	GroupUpdateInput,
	MembershipRequest
} from '$lib/types';
import type { GroupCreateInput } from '$lib/types';
import type { GroupService, Paginated } from '../types';

type JsonRecord = Record<string, unknown>;

const GROUP_JOIN_POLICIES = new Set<GroupJoinPolicy>(['terbuka', 'persetujuan', 'undangan']);
const GROUP_ENTITY_TYPES = new Set<GroupSummary['entity_type']>(['kelompok', 'lembaga']);
const GROUP_MEMBER_ROLES = new Set<GroupMemberRole>(['admin', 'moderator', 'anggota']);
const MEMBERSHIP_REQUEST_STATUSES = new Set<MembershipRequest['status']>([
	'pending',
	'approved',
	'rejected'
]);
const MEMBERSHIP_STATUSES = new Set<GroupDetail['my_membership_status']>([
	'none',
	'pending',
	'approved',
	'rejected'
]);

const isRecord = (value: unknown): value is JsonRecord =>
	typeof value === 'object' && value !== null && !Array.isArray(value);

const asString = (value: unknown): string | undefined =>
	typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined;

const asNumber = (value: unknown): number | undefined =>
	typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const asBoolean = (value: unknown): boolean | undefined =>
	typeof value === 'boolean' ? value : undefined;

const readJoinPolicy = (value: unknown): GroupJoinPolicy | undefined => {
	const normalized = asString(value)?.toLowerCase() as GroupJoinPolicy | undefined;
	if (!normalized || !GROUP_JOIN_POLICIES.has(normalized)) return undefined;
	return normalized;
};

const readEntityType = (value: unknown): GroupSummary['entity_type'] | undefined => {
	const normalized = asString(value)?.toLowerCase() as GroupSummary['entity_type'] | undefined;
	if (!normalized || !GROUP_ENTITY_TYPES.has(normalized)) return undefined;
	return normalized;
};

const readMemberRole = (value: unknown): GroupMemberRole | undefined => {
	const normalized = asString(value)?.toLowerCase() as GroupMemberRole | undefined;
	if (!normalized || !GROUP_MEMBER_ROLES.has(normalized)) return undefined;
	return normalized;
};

const readRequestStatus = (value: unknown): MembershipRequest['status'] | undefined => {
	const normalized = asString(value)?.toLowerCase() as MembershipRequest['status'] | undefined;
	if (!normalized || !MEMBERSHIP_REQUEST_STATUSES.has(normalized)) return undefined;
	return normalized;
};

const readMembershipStatus = (value: unknown): GroupDetail['my_membership_status'] | undefined => {
	const normalized = asString(value)?.toLowerCase() as
		| GroupDetail['my_membership_status']
		| undefined;
	if (!normalized || !MEMBERSHIP_STATUSES.has(normalized)) return undefined;
	return normalized;
};

const toIsoString = (value: unknown): string | undefined => {
	const asText = asString(value);
	if (asText) {
		const parsed = Date.parse(asText);
		if (!Number.isNaN(parsed)) return new Date(parsed).toISOString();
	}
	const asNumeric = typeof value === 'string' ? Number(value) : asNumber(value);
	if (asNumeric !== undefined && Number.isFinite(asNumeric)) {
		return new Date(asNumeric).toISOString();
	}
	return undefined;
};

const parseMember = (value: unknown): GroupMember | undefined => {
	if (!isRecord(value)) return undefined;
	const userId = asString(value.user_id);
	const name = asString(value.name);
	const role = readMemberRole(value.role);
	const joinedAt = toIsoString(value.joined_at);
	if (!userId || !name || !role || !joinedAt) return undefined;
	return {
		user_id: userId,
		name,
		avatar_url: asString(value.avatar_url),
		role,
		joined_at: joinedAt
	};
};

const parseMembershipRequest = (value: unknown): MembershipRequest | undefined => {
	if (!isRecord(value)) return undefined;
	const requestId = asString(value.request_id);
	const userId = asString(value.user_id);
	const name = asString(value.name);
	const status = readRequestStatus(value.status);
	const requestedAt = toIsoString(value.requested_at);
	if (!requestId || !userId || !name || !status || !requestedAt) return undefined;
	return {
		request_id: requestId,
		user_id: userId,
		name,
		avatar_url: asString(value.avatar_url),
		message: asString(value.message),
		status,
		requested_at: requestedAt
	};
};

const parseGroupSummary = (value: unknown): GroupSummary | undefined => {
	if (!isRecord(value)) return undefined;
	const groupId = asString(value.group_id);
	const name = asString(value.name);
	const description = asString(value.description);
	const entityType = readEntityType(value.entity_type);
	const joinPolicy = readJoinPolicy(value.join_policy);
	const memberCount = asNumber(value.member_count);
	const witnessCount = asNumber(value.witness_count);
	if (
		!groupId ||
		!name ||
		!description ||
		!entityType ||
		!joinPolicy ||
		memberCount === undefined ||
		witnessCount === undefined
	) {
		return undefined;
	}
	const rawTag = isRecord(value.entity_tag) ? value.entity_tag : undefined;
	return {
		group_id: groupId,
		name,
		description,
		entity_type: entityType,
		join_policy: joinPolicy,
		member_count: memberCount,
		witness_count: witnessCount,
		entity_tag: {
			entity_id: asString(rawTag?.entity_id) ?? groupId,
			entity_type: readEntityType(rawTag?.entity_type) ?? entityType,
			label: asString(rawTag?.label) ?? name,
			followed: asBoolean(rawTag?.followed) ?? false
		}
	};
};

const parseGroupDetail = (value: unknown): GroupDetail | undefined => {
	const summary = parseGroupSummary(value);
	if (!summary || !isRecord(value)) return undefined;
	if (!Array.isArray(value.members) || !Array.isArray(value.pending_requests)) return undefined;
	const members = value.members
		.map(parseMember)
		.filter((member): member is GroupMember => Boolean(member));
	if (members.length !== value.members.length) return undefined;
	const pendingRequests = value.pending_requests
		.map(parseMembershipRequest)
		.filter((request): request is MembershipRequest => Boolean(request));
	if (pendingRequests.length !== value.pending_requests.length) return undefined;
	const myRoleValue = value.my_role;
	const myRole =
		myRoleValue === null || myRoleValue === undefined
			? null
			: (readMemberRole(myRoleValue) ?? null);
	const membershipStatus = readMembershipStatus(value.my_membership_status);
	if (!membershipStatus) return undefined;
	return {
		...summary,
		members,
		pending_requests: pendingRequests,
		my_role: myRole,
		my_membership_status: membershipStatus
	};
};

interface ApiGroupServiceOptions {
	allowMockFallback?: boolean;
}

export class ApiGroupService implements GroupService {
	private readonly client: ApiClient;
	private readonly fallback: GroupService;
	private readonly allowMockFallback: boolean;

	constructor(client: ApiClient, fallback: GroupService, options: ApiGroupServiceOptions = {}) {
		this.client = client;
		this.fallback = fallback;
		this.allowMockFallback = options.allowMockFallback ?? true;
	}

	private fallbackOrThrow<T>(fallback: () => Promise<T>, error?: unknown): Promise<T> {
		if (this.allowMockFallback) {
			return fallback();
		}
		if (error instanceof Error) {
			throw error;
		}
		throw new Error('Mock fallback disabled for group service');
	}

	async create(input: GroupCreateInput): Promise<GroupDetail> {
		try {
			const response = await this.client.post<unknown>('/groups', { body: input });
			const detail = parseGroupDetail(response);
			if (!detail) throw new Error('invalid group create response');
			return detail;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.create(input), error);
		}
	}

	async list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<GroupSummary>> {
		try {
			const response = await this.client.get<unknown>('/groups', {
				query: {
					cursor: opts?.cursor,
					limit: opts?.limit
				}
			});
			if (!isRecord(response) || !Array.isArray(response.items)) {
				throw new Error('invalid group list response');
			}
			const items = response.items
				.map(parseGroupSummary)
				.filter((item): item is GroupSummary => Boolean(item));
			if (items.length !== response.items.length) {
				throw new Error('invalid group list item');
			}
			return {
				items,
				total: asNumber(response.total) ?? items.length,
				cursor: asString(response.cursor)
			};
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.list(opts), error);
		}
	}

	async listMyGroups(): Promise<GroupSummary[]> {
		try {
			const response = await this.client.get<unknown>('/groups/me');
			const rawItems = Array.isArray(response)
				? response
				: isRecord(response) && Array.isArray(response.items)
					? response.items
					: undefined;
			if (!rawItems) throw new Error('invalid my groups response');
			const items = rawItems
				.map(parseGroupSummary)
				.filter((item): item is GroupSummary => Boolean(item));
			if (items.length !== rawItems.length) {
				throw new Error('invalid my groups item');
			}
			return items;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.listMyGroups(), error);
		}
	}

	async get(groupId: string): Promise<GroupDetail> {
		try {
			const response = await this.client.get<unknown>(`/groups/${encodeURIComponent(groupId)}`);
			const detail = parseGroupDetail(response);
			if (!detail) throw new Error('invalid group detail response');
			return detail;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.get(groupId), error);
		}
	}

	async update(groupId: string, input: GroupUpdateInput): Promise<GroupDetail> {
		try {
			const response = await this.client.patch<unknown>(`/groups/${encodeURIComponent(groupId)}`, {
				body: input
			});
			const detail = parseGroupDetail(response);
			if (!detail) throw new Error('invalid group update response');
			return detail;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.update(groupId, input), error);
		}
	}

	async join(groupId: string): Promise<GroupMember> {
		try {
			const response = await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/join`
			);
			const member = parseMember(response);
			if (!member) throw new Error('invalid group join response');
			return member;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.join(groupId), error);
		}
	}

	async requestJoin(groupId: string, message?: string): Promise<MembershipRequest> {
		try {
			const response = await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/requests`,
				{
					body: {
						message
					}
				}
			);
			const request = parseMembershipRequest(response);
			if (!request) throw new Error('invalid group request response');
			return request;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.requestJoin(groupId, message), error);
		}
	}

	async approveRequest(groupId: string, requestId: string): Promise<GroupMember> {
		try {
			const response = await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/requests/${encodeURIComponent(requestId)}/approve`
			);
			const member = parseMember(response);
			if (!member) throw new Error('invalid group approve response');
			return member;
		} catch (error) {
			return this.fallbackOrThrow(() => this.fallback.approveRequest(groupId, requestId), error);
		}
	}

	async rejectRequest(groupId: string, requestId: string): Promise<void> {
		try {
			await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/requests/${encodeURIComponent(requestId)}/reject`
			);
		} catch (error) {
			await this.fallbackOrThrow(() => this.fallback.rejectRequest(groupId, requestId), error);
		}
	}

	async invite(groupId: string, userId: string): Promise<void> {
		try {
			await this.client.post<unknown>(`/groups/${encodeURIComponent(groupId)}/invite`, {
				body: {
					user_id: userId
				}
			});
		} catch (error) {
			await this.fallbackOrThrow(() => this.fallback.invite(groupId, userId), error);
		}
	}

	async leave(groupId: string): Promise<void> {
		try {
			await this.client.post<unknown>(`/groups/${encodeURIComponent(groupId)}/leave`);
		} catch (error) {
			await this.fallbackOrThrow(() => this.fallback.leave(groupId), error);
		}
	}

	async removeMember(groupId: string, userId: string): Promise<void> {
		try {
			await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/members/${encodeURIComponent(userId)}/remove`
			);
		} catch (error) {
			await this.fallbackOrThrow(() => this.fallback.removeMember(groupId, userId), error);
		}
	}

	async updateMemberRole(groupId: string, userId: string, role: GroupMemberRole): Promise<void> {
		try {
			await this.client.post<unknown>(
				`/groups/${encodeURIComponent(groupId)}/members/${encodeURIComponent(userId)}/role`,
				{
					body: {
						role
					}
				}
			);
		} catch (error) {
			await this.fallbackOrThrow(
				() => this.fallback.updateMemberRole(groupId, userId, role),
				error
			);
		}
	}
}

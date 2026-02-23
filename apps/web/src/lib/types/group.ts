import type { EntityTag } from './feed';

// ---------------------------------------------------------------------------
// Group domain types (Kelompok / Lembaga)
// ---------------------------------------------------------------------------

/** 3-tier join policy (separate from content Rahasia levels). */
export type GroupJoinPolicy = 'terbuka' | 'persetujuan' | 'undangan';

export type GroupMemberRole = 'admin' | 'moderator' | 'anggota';

export type MembershipRequestStatus = 'pending' | 'approved' | 'rejected';

export type GroupEntityType = 'kelompok' | 'lembaga';

/** List/card representation (for discovery + "My Groups"). */
export interface GroupSummary {
	group_id: string;
	name: string;
	description: string;
	entity_type: GroupEntityType;
	join_policy: GroupJoinPolicy;
	member_count: number;
	witness_count: number;
	/** EntityTag used in feed integration (entity_id SHOULD match group_id in mocks). */
	entity_tag: EntityTag;
}

export interface GroupMember {
	user_id: string;
	name: string;
	avatar_url?: string;
	role: GroupMemberRole;
	joined_at: string;
}

export interface MembershipRequest {
	request_id: string;
	user_id: string;
	name: string;
	avatar_url?: string;
	message?: string;
	status: MembershipRequestStatus;
	requested_at: string;
}

/** Full group detail. */
export interface GroupDetail extends GroupSummary {
	members: GroupMember[];
	/** Pending join requests (only relevant for 'persetujuan'). */
	pending_requests: MembershipRequest[];
	my_role: GroupMemberRole | null;
	/** Current user's join state in this group. */
	my_membership_status: 'none' | MembershipRequestStatus;
}

export interface GroupCreateInput {
	name: string;
	description: string;
	entity_type: GroupEntityType;
	join_policy: GroupJoinPolicy;
}

export type GroupUpdateInput = Partial<Pick<GroupSummary, 'name' | 'description' | 'join_policy'>>;


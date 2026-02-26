/**
 * Mock groups (Kelompok / Lembaga) fixtures.
 *
 * Notes:
 * - For feed integration, group_id is aligned with EntityTag.entity_id (ent-xxx).
 * - Join policy:
 *   - terbuka: join immediately
 *   - persetujuan: request + approve
 *   - undangan: hidden from discovery + invite only
 */

import type { GroupDetail, GroupMember, GroupSummary, MembershipRequest } from '$lib/types';
import { mockUsers } from './mock-users';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();
const tsDay = (daysAgo: number): string =>
	new Date(now - daysAgo * 24 * 60 * 60 * 1000).toISOString();
const tsHour = (hoursAgo: number): string =>
	new Date(now - hoursAgo * 60 * 60 * 1000).toISOString();

const userById = new Map(mockUsers.map((u) => [u.user_id, u]));
const userInfo = (userId: string) => {
	const u = userById.get(userId);
	return { user_id: userId, name: u?.name ?? userId, avatar_url: u?.avatar_url };
};

const member = (userId: string, role: GroupMember['role'], joinedAt: string): GroupMember => ({
	...userInfo(userId),
	role,
	joined_at: joinedAt
});

const req = (
	requestId: string,
	userId: string,
	message: string,
	requestedAt: string
): MembershipRequest => ({
	request_id: requestId,
	...userInfo(userId),
	message,
	status: 'pending',
	requested_at: requestedAt
});

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

export const mockGroup1: GroupDetail = {
	group_id: 'ent-003', // reuses mock-feed entity ent-003
	name: 'Karang Taruna RT 05',
	description: 'Wadah pemuda RT 05 untuk kerja bakti, acara warga, dan respons cepat.',
	entity_type: 'kelompok',
	join_policy: 'terbuka',
	member_count: 5,
	witness_count: 9,
	entity_tag: {
		entity_id: 'ent-003',
		entity_type: 'kelompok',
		label: 'Karang Taruna RT 05',
		followed: false
	},
	members: [
		member('u-001', 'admin', tsDay(120)),
		member('u-002', 'moderator', tsDay(210)),
		member('u-003', 'anggota', tsDay(40)),
		member('u-004', 'moderator', tsDay(365)),
		member('u-005', 'anggota', tsDay(12))
	],
	pending_requests: [],
	my_role: 'admin',
	my_membership_status: 'approved'
};

export const mockGroup2: GroupDetail = {
	group_id: 'ent-004', // reuses mock-feed entity ent-004
	name: 'Komite Sekolah SDN 3 Menteng',
	description: 'Koordinasi orang tua, sekolah, dan warga sekitar untuk kegiatan dan transparansi.',
	entity_type: 'lembaga',
	join_policy: 'persetujuan',
	member_count: 4,
	witness_count: 6,
	entity_tag: {
		entity_id: 'ent-004',
		entity_type: 'lembaga',
		label: 'Komite Sekolah SDN 3 Menteng',
		followed: false
	},
	members: [
		member('u-004', 'admin', tsDay(520)),
		member('u-002', 'moderator', tsDay(320)),
		member('u-003', 'anggota', tsDay(60)),
		member('u-005', 'anggota', tsDay(18))
	],
	pending_requests: [
		req(
			'req-ent-004-001',
			'u-001',
			'Saya ingin membantu komunikasi agenda dan dokumentasi.',
			tsHour(6)
		)
	],
	my_role: null,
	my_membership_status: 'pending'
};

export const mockGroup3: GroupDetail = {
	group_id: 'ent-101',
	name: 'Komunitas Peduli Lingkungan',
	description: 'Aksi rutin bersih lingkungan, bank sampah, dan edukasi warga.',
	entity_type: 'kelompok',
	join_policy: 'terbuka',
	member_count: 3,
	witness_count: 11,
	entity_tag: {
		entity_id: 'ent-101',
		entity_type: 'kelompok',
		label: 'Peduli Lingkungan',
		followed: true
	},
	members: [
		member('u-002', 'admin', tsDay(240)),
		member('u-003', 'anggota', tsDay(80)),
		member('u-005', 'anggota', tsDay(20))
	],
	pending_requests: [],
	my_role: null,
	my_membership_status: 'none'
};

export const mockGroup4: GroupDetail = {
	group_id: 'ent-102',
	name: 'Forum Pemuda RW 03',
	description: 'Forum koordinasi pemuda RW 03 untuk kegiatan sosial dan olahraga.',
	entity_type: 'kelompok',
	join_policy: 'persetujuan',
	member_count: 6,
	witness_count: 4,
	entity_tag: {
		entity_id: 'ent-102',
		entity_type: 'kelompok',
		label: 'Forum Pemuda RW 03',
		followed: false
	},
	members: [
		member('u-001', 'admin', tsDay(90)),
		member('u-002', 'moderator', tsDay(180)),
		member('u-003', 'anggota', tsDay(14)),
		member('u-004', 'moderator', tsDay(365)),
		member('u-005', 'anggota', tsDay(10)),
		member('u-999', 'anggota', tsDay(1))
	].map((m) => (m.user_id === 'u-999' ? { ...m, name: 'Nisa Putri', avatar_url: undefined } : m)),
	pending_requests: [
		req(
			'req-ent-102-001',
			'u-005',
			'Saya siap bantu koordinasi jadwal olahraga mingguan.',
			tsHour(10)
		),
		req('req-ent-102-002', 'u-003', 'Ingin ikut dan bantu dokumentasi kegiatan RW.', tsHour(30))
	],
	my_role: 'admin',
	my_membership_status: 'approved'
};

export const mockGroup5: GroupDetail = {
	group_id: 'ent-103',
	name: 'Yayasan Gotong Royong',
	description: 'Lembaga internal untuk pengelolaan program, pendanaan, dan kemitraan.',
	entity_type: 'lembaga',
	join_policy: 'undangan',
	member_count: 3,
	witness_count: 2,
	entity_tag: {
		entity_id: 'ent-103',
		entity_type: 'lembaga',
		label: 'Yayasan Gotong Royong',
		followed: false
	},
	members: [
		member('u-004', 'admin', tsDay(800)),
		member('u-002', 'moderator', tsDay(620)),
		member('u-003', 'anggota', tsDay(90))
	],
	pending_requests: [],
	my_role: null,
	my_membership_status: 'none'
};

export const mockGroups: GroupDetail[] = [
	mockGroup1,
	mockGroup2,
	mockGroup3,
	mockGroup4,
	mockGroup5
];

export const mockGroupSummaries: GroupSummary[] = mockGroups.map((g) => {
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { members, pending_requests, my_role, my_membership_status, ...summary } = g;
	return summary;
});

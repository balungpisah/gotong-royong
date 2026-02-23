/**
 * Group Store — manages Kelompok/Lembaga list + membership state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Consumes GroupService interface for data operations (mock-first).
 */

import type { GroupService } from '$lib/services/types';
import type { GroupCreateInput, GroupDetail, GroupMemberRole, GroupSummary, GroupUpdateInput } from '$lib/types';

type StoreErrors = Partial<{
	list: string;
	detail: string;
	create: string;
	action: string;
}>;

function upsertById<T extends { group_id: string }>(items: T[], next: T): T[] {
	const idx = items.findIndex((g) => g.group_id === next.group_id);
	if (idx < 0) return [next, ...items];
	return items.map((g) => (g.group_id === next.group_id ? next : g));
}

function removeById<T extends { group_id: string }>(items: T[], groupId: string): T[] {
	return items.filter((g) => g.group_id !== groupId);
}

export class GroupStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	groups = $state<GroupSummary[]>([]);
	myGroups = $state<GroupSummary[]>([]);
	current = $state<GroupDetail | null>(null);

	listLoading = $state(false);
	detailLoading = $state(false);
	creating = $state(false);

	errors = $state<StoreErrors>({});

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	myGroupCount = $derived(this.myGroups.length);

	isCurrentMember = $derived(this.current?.my_membership_status === 'approved');
	isCurrentAdmin = $derived(this.current?.my_role === 'admin');
	canManage = $derived(this.current?.my_role === 'admin' || this.current?.my_role === 'moderator');

	pendingRequestCount = $derived(this.current?.pending_requests?.length ?? 0);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	private readonly service: GroupService;

	constructor(service: GroupService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async loadGroups(opts?: { limit?: number }) {
		this.listLoading = true;
		this.errors = { ...this.errors, list: undefined };
		try {
			const result = await this.service.list({ limit: opts?.limit });
			this.groups = result.items;
		} catch (err) {
			this.errors = {
				...this.errors,
				list: err instanceof Error ? err.message : 'Gagal memuat daftar kelompok'
			};
		} finally {
			this.listLoading = false;
		}
	}

	async loadMyGroups() {
		this.errors = { ...this.errors, list: undefined };
		try {
			this.myGroups = await this.service.listMyGroups();
		} catch (err) {
			this.errors = {
				...this.errors,
				list: err instanceof Error ? err.message : 'Gagal memuat kelompok saya'
			};
		}
	}

	async loadDetail(groupId: string) {
		this.detailLoading = true;
		this.errors = { ...this.errors, detail: undefined };
		try {
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				detail: err instanceof Error ? err.message : 'Gagal memuat detail kelompok'
			};
			this.current = null;
		} finally {
			this.detailLoading = false;
		}
	}

	async createGroup(input: GroupCreateInput): Promise<string | null> {
		this.creating = true;
		this.errors = { ...this.errors, create: undefined };
		try {
			const detail = await this.service.create(input);
			this.current = detail;
			const { members, pending_requests, my_role, my_membership_status, ...summary } = detail;
			this.myGroups = upsertById(this.myGroups, summary);
			// Only discoverable groups appear in public list.
			if (detail.join_policy !== 'undangan') {
				this.groups = upsertById(this.groups, summary);
			}
			return detail.group_id;
		} catch (err) {
			this.errors = {
				...this.errors,
				create: err instanceof Error ? err.message : 'Gagal membuat kelompok'
			};
			return null;
		} finally {
			this.creating = false;
		}
	}

	async updateGroup(groupId: string, input: GroupUpdateInput) {
		this.errors = { ...this.errors, action: undefined };
		try {
			const detail = await this.service.update(groupId, input);
			this.current = detail;
			const { members, pending_requests, my_role, my_membership_status, ...summary } = detail;
			this.groups = detail.join_policy === 'undangan' ? removeById(this.groups, groupId) : upsertById(this.groups, summary);
			this.myGroups = upsertById(this.myGroups, summary);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal memperbarui kelompok'
			};
		}
	}

	async joinGroup(groupId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.join(groupId);
			const detail = await this.service.get(groupId);
			this.current = detail;
			const { members, pending_requests, my_role, my_membership_status, ...summary } = detail;
			this.myGroups = upsertById(this.myGroups, summary);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal bergabung'
			};
		}
	}

	async requestJoinGroup(groupId: string, message?: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.requestJoin(groupId, message);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal mengirim permintaan'
			};
		}
	}

	async approveRequest(groupId: string, requestId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.approveRequest(groupId, requestId);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal menyetujui permintaan'
			};
		}
	}

	async rejectRequest(groupId: string, requestId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.rejectRequest(groupId, requestId);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal menolak permintaan'
			};
		}
	}

	async leaveGroup(groupId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.leave(groupId);
			this.myGroups = removeById(this.myGroups, groupId);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal keluar'
			};
		}
	}

	async removeMember(groupId: string, userId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.removeMember(groupId, userId);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal mengeluarkan anggota'
			};
		}
	}

	async updateMemberRole(groupId: string, userId: string, role: GroupMemberRole) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.updateMemberRole(groupId, userId, role);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal mengubah peran'
			};
		}
	}

	async invite(groupId: string, userId: string) {
		this.errors = { ...this.errors, action: undefined };
		try {
			await this.service.invite(groupId, userId);
			this.current = await this.service.get(groupId);
		} catch (err) {
			this.errors = {
				...this.errors,
				action: err instanceof Error ? err.message : 'Gagal mengundang anggota'
			};
		}
	}
}

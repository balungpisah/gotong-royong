<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import GroupPrivacyBadge from './group-privacy-badge.svelte';
	import GroupMemberList from './group-member-list.svelte';
	import GroupPendingRequests from './group-pending-requests.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';

	const store = getGroupStore();

	type Tab = 'anggota' | 'permintaan' | 'pengaturan';
	let tab = $state<Tab>('anggota');

	let requestMessage = $state('');

	let editName = $state('');
	let editDescription = $state('');
	let editJoinPolicy = $state<'terbuka' | 'persetujuan' | 'undangan'>('terbuka');
	let inviteUserId = $state('');

	const group = $derived(store.current);
	const canManage = $derived(store.canManage);

	$effect(() => {
		if (!group) return;
		editName = group.name;
		editDescription = group.description;
		editJoinPolicy = group.join_policy;
	});

	const showRequestsTab = $derived(
		!!group && group.join_policy === 'persetujuan' && canManage
	);
	const showSettingsTab = $derived(!!group && canManage);

	const canJoinNow = $derived(!!group && group.join_policy === 'terbuka' && group.my_membership_status !== 'approved');
	const canRequestJoin = $derived(
		!!group &&
			group.join_policy === 'persetujuan' &&
			group.my_membership_status !== 'approved' &&
			group.my_membership_status !== 'pending'
	);
	const isPending = $derived(!!group && group.my_membership_status === 'pending');
</script>

{#if store.detailLoading && !group}
	<div class="flex h-48 items-center justify-center">
		<div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
	</div>
{:else if group}
	<section class="rounded-xl border border-border/50 bg-card p-4">
		<header class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h2 class="truncate text-base font-bold text-foreground">{group.name}</h2>
				<p class="mt-1 text-xs leading-relaxed text-muted-foreground/80">{group.description}</p>
				<div class="mt-2 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground/70">
					<GroupPrivacyBadge joinPolicy={group.join_policy} />
					<span class="inline-flex items-center gap-1">
						<UsersIcon class="size-3" /> {m.group_stat_members({ count: group.member_count })}
					</span>
					<span>•</span>
					<span>{m.group_stat_witnesses({ count: group.witness_count })}</span>
				</div>
			</div>

			<div class="shrink-0">
				{#if store.isCurrentMember}
					<button
						type="button"
						onclick={() => store.leaveGroup(group.group_id)}
						class="inline-flex items-center justify-center rounded-lg bg-muted/40 px-3 py-2 text-xs font-semibold text-foreground transition hover:bg-muted/60"
					>
						<UserMinusIcon class="mr-1 size-3.5" />
						{m.group_action_leave()}
					</button>
				{:else if canJoinNow}
					<button
						type="button"
						onclick={() => store.joinGroup(group.group_id)}
						class="inline-flex items-center justify-center rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
					>
						<UserPlusIcon class="mr-1 size-3.5" />
						{m.group_action_join()}
					</button>
				{:else if canRequestJoin}
					<button
						type="button"
						onclick={() => store.requestJoinGroup(group.group_id, requestMessage.trim() || undefined)}
						class="inline-flex items-center justify-center rounded-lg bg-primary/10 px-3 py-2 text-xs font-semibold text-primary transition hover:bg-primary/15"
					>
						<UserPlusIcon class="mr-1 size-3.5" />
						{m.group_action_request_join()}
					</button>
				{:else if isPending}
					<span class="inline-flex items-center rounded-lg bg-muted/40 px-3 py-2 text-xs font-semibold text-muted-foreground">
						{m.group_request_pending()}
					</span>
				{:else}
					<span class="inline-flex items-center rounded-lg bg-muted/40 px-3 py-2 text-xs font-semibold text-muted-foreground">
						{m.group_action_invite_only()}
					</span>
				{/if}
			</div>
		</header>

		{#if group.join_policy === 'persetujuan' && !store.isCurrentMember && group.my_membership_status === 'none'}
			<div class="mt-3 rounded-lg border border-border/40 bg-muted/10 p-3">
				<label class="block">
					<span class="text-xs font-semibold text-foreground">{m.group_request_message_label()}</span>
					<input
						class="mt-1 w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
						placeholder={m.group_request_message_placeholder()}
						bind:value={requestMessage}
					/>
				</label>
			</div>
		{/if}

		{#if store.errors.action}
			<p class="mt-3 text-xs text-bahaya">{store.errors.action}</p>
		{/if}

		<!-- Tabs -->
		<div class="mt-4 flex flex-wrap items-center gap-2">
			<button
				type="button"
				class="rounded-full px-3 py-1.5 text-xs font-semibold transition
					{tab === 'anggota' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}"
				onclick={() => (tab = 'anggota')}
			>
				{m.group_tab_members()}
			</button>

			{#if showRequestsTab}
				<button
					type="button"
					class="rounded-full px-3 py-1.5 text-xs font-semibold transition
						{tab === 'permintaan' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}"
					onclick={() => (tab = 'permintaan')}
				>
					{m.group_tab_requests({ count: store.pendingRequestCount })}
				</button>
			{/if}

			{#if showSettingsTab}
				<button
					type="button"
					class="inline-flex items-center rounded-full px-3 py-1.5 text-xs font-semibold transition
						{tab === 'pengaturan' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}"
					onclick={() => (tab = 'pengaturan')}
				>
					<SettingsIcon class="mr-1 size-3.5" />
					{m.group_tab_settings()}
				</button>
			{/if}
		</div>

		<!-- Tab content -->
		<div class="mt-4">
			{#if tab === 'anggota'}
				<GroupMemberList
					members={group.members}
					myRole={group.my_role}
					onRemove={(userId) => store.removeMember(group.group_id, userId)}
					onUpdateRole={(userId, role) => store.updateMemberRole(group.group_id, userId, role)}
				/>
			{:else if tab === 'permintaan'}
				<GroupPendingRequests
					requests={group.pending_requests}
					onApprove={(requestId) => store.approveRequest(group.group_id, requestId)}
					onReject={(requestId) => store.rejectRequest(group.group_id, requestId)}
				/>
			{:else if tab === 'pengaturan'}
				<div class="rounded-lg border border-border/40 bg-muted/10 p-3">
					<div class="flex items-center gap-2 text-xs font-semibold text-foreground">
						<SettingsIcon class="size-4" />
						<span>{m.group_settings_title()}</span>
					</div>

					<div class="mt-3 grid gap-3">
						<label class="block">
							<span class="text-xs font-semibold text-foreground">{m.group_settings_name()}</span>
							<input
								class="mt-1 w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
								bind:value={editName}
							/>
						</label>

						<label class="block">
							<span class="text-xs font-semibold text-foreground">{m.group_settings_description()}</span>
							<textarea
								class="mt-1 w-full resize-none rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
								rows={3}
								bind:value={editDescription}
							></textarea>
						</label>

						<label class="block">
							<span class="text-xs font-semibold text-foreground">{m.group_settings_join_policy()}</span>
							<select
								class="mt-1 w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
								bind:value={editJoinPolicy}
							>
								<option value="terbuka">{m.group_policy_terbuka()}</option>
								<option value="persetujuan">{m.group_policy_persetujuan()}</option>
								<option value="undangan">{m.group_policy_undangan()}</option>
							</select>
							<p class="mt-1 text-[11px] text-muted-foreground/70">
								{m.group_settings_privacy_note()}
							</p>
						</label>

						<div class="flex justify-end">
							<button
								type="button"
								class="inline-flex items-center rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
								onclick={() =>
									store.updateGroup(group.group_id, {
										name: editName.trim(),
										description: editDescription.trim(),
										join_policy: editJoinPolicy
									})
								}
							>
								{m.group_settings_save()}
							</button>
						</div>

						<div class="mt-2 rounded-lg border border-border/40 bg-background p-3">
							<p class="text-xs font-semibold text-foreground">{m.group_invite_title()}</p>
							<div class="mt-2 flex items-center gap-2">
								<input
									class="w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
									placeholder={m.group_invite_user_placeholder()}
									bind:value={inviteUserId}
								/>
								<button
									type="button"
									class="inline-flex shrink-0 items-center rounded-lg bg-primary/10 px-3 py-2 text-xs font-semibold text-primary transition hover:bg-primary/15"
									disabled={inviteUserId.trim().length === 0}
									onclick={() => {
										store.invite(group.group_id, inviteUserId.trim());
										inviteUserId = '';
									}}
								>
									{m.group_invite_send()}
								</button>
							</div>
							<p class="mt-1 text-[11px] text-muted-foreground/70">{m.group_invite_note()}</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</section>
{:else if store.errors.detail}
	<div class="rounded-xl border border-border/50 bg-card p-4">
		<p class="text-xs text-bahaya">{store.errors.detail}</p>
	</div>
{/if}

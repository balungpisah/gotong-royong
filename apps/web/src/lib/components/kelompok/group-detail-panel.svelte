<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import GroupPrivacyBadge from './group-privacy-badge.svelte';
	import GroupMemberList from './group-member-list.svelte';
	import GroupPendingRequests from './group-pending-requests.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Select } from '$lib/components/ui/select';
	import InputLabel from '$lib/components/ui/input/input-label.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';
	import { Button } from '$lib/components/ui/button';

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
	<Card.Root padding="compact">
		<header class="flex items-start justify-between gap-3">
			<div class="min-w-0">
				<h2 class="truncate text-h3 font-bold text-foreground">{group.name}</h2>
				<p class="mt-1 text-small leading-relaxed text-muted-foreground/80">{group.description}</p>
				<div class="mt-2 flex flex-wrap items-center gap-2 text-caption text-muted-foreground/70">
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
					<Button
						variant="secondary"
						size="sm"
						onclick={() => store.leaveGroup(group.group_id)}
					>
						<UserMinusIcon class="mr-1 size-3.5" />
						{m.group_action_leave()}
					</Button>
				{:else if canJoinNow}
					<Button
						variant="default"
						size="sm"
						onclick={() => store.joinGroup(group.group_id)}
					>
						<UserPlusIcon class="mr-1 size-3.5" />
						{m.group_action_join()}
					</Button>
				{:else if canRequestJoin}
					<Button
						variant="outline"
						size="sm"
						class="bg-primary/10 text-primary hover:bg-primary/15"
						onclick={() => store.requestJoinGroup(group.group_id, requestMessage.trim() || undefined)}
					>
						<UserPlusIcon class="mr-1 size-3.5" />
						{m.group_action_request_join()}
					</Button>
				{:else if isPending}
					<span class="inline-flex items-center rounded-lg bg-muted/40 px-3 py-2 text-small font-semibold text-muted-foreground">
						{m.group_request_pending()}
					</span>
				{:else}
					<span class="inline-flex items-center rounded-lg bg-muted/40 px-3 py-2 text-small font-semibold text-muted-foreground">
						{m.group_action_invite_only()}
					</span>
				{/if}
			</div>
		</header>

		{#if group.join_policy === 'persetujuan' && !store.isCurrentMember && group.my_membership_status === 'none'}
			<div class="mt-3 rounded-lg border border-border/40 bg-muted/10 p-3">
				<div class="flex flex-col gap-1.5">
					<InputLabel>{m.group_request_message_label()}</InputLabel>
					<Input
						placeholder={m.group_request_message_placeholder()}
						bind:value={requestMessage}
					/>
				</div>
			</div>
		{/if}

		{#if store.errors.action}
			<p class="mt-3 text-small text-bahaya">{store.errors.action}</p>
		{/if}

		<!-- Tabs -->
		<div class="mt-4 flex flex-wrap items-center gap-2">
			<Button
				variant="ghost"
				size="pill"
				class={tab === 'anggota' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}
				onclick={() => (tab = 'anggota')}
			>
				{m.group_tab_members()}
			</Button>

			{#if showRequestsTab}
				<Button
					variant="ghost"
					size="pill"
					class={tab === 'permintaan' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}
					onclick={() => (tab = 'permintaan')}
				>
					{m.group_tab_requests({ count: store.pendingRequestCount })}
				</Button>
			{/if}

			{#if showSettingsTab}
				<Button
					variant="ghost"
					size="pill"
					class={tab === 'pengaturan' ? 'bg-primary/10 text-primary' : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}
					onclick={() => (tab = 'pengaturan')}
				>
					<SettingsIcon class="mr-1 size-3.5" />
					{m.group_tab_settings()}
				</Button>
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
					<div class="flex items-center gap-2 text-small font-semibold text-foreground">
						<SettingsIcon class="size-4" />
						<span>{m.group_settings_title()}</span>
					</div>

					<div class="mt-3 grid gap-3">
						<div class="flex flex-col gap-1.5">
							<InputLabel>{m.group_settings_name()}</InputLabel>
							<Input bind:value={editName} />
						</div>

						<div class="flex flex-col gap-1.5">
							<InputLabel>{m.group_settings_description()}</InputLabel>
							<Textarea
								rows={3}
								class="resize-none"
								bind:value={editDescription}
							/>
						</div>

						<div class="flex flex-col gap-1.5">
							<InputLabel>{m.group_settings_join_policy()}</InputLabel>
							<Select bind:value={editJoinPolicy}>
								<option value="terbuka">{m.group_policy_terbuka()}</option>
								<option value="persetujuan">{m.group_policy_persetujuan()}</option>
								<option value="undangan">{m.group_policy_undangan()}</option>
							</Select>
							<p class="mt-1 text-caption text-muted-foreground/70">
								{m.group_settings_privacy_note()}
							</p>
						</div>

						<div class="flex justify-end">
							<Button
								variant="default"
								onclick={() =>
									store.updateGroup(group.group_id, {
										name: editName.trim(),
										description: editDescription.trim(),
										join_policy: editJoinPolicy
									})
								}
							>
								{m.group_settings_save()}
							</Button>
						</div>

						<div class="mt-2 rounded-lg border border-border/40 bg-background p-3">
							<p class="text-small font-semibold text-foreground">{m.group_invite_title()}</p>
							<div class="mt-2 flex items-center gap-2">
								<Input
									placeholder={m.group_invite_user_placeholder()}
									bind:value={inviteUserId}
								/>
								<Button
									variant="outline"
									size="sm"
									class="shrink-0 bg-primary/10 text-primary hover:bg-primary/15"
									disabled={inviteUserId.trim().length === 0}
									onclick={() => {
										store.invite(group.group_id, inviteUserId.trim());
										inviteUserId = '';
									}}
								>
									{m.group_invite_send()}
								</Button>
							</div>
							<p class="mt-1 text-caption text-muted-foreground/70">{m.group_invite_note()}</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</Card.Root>
{:else if store.errors.detail}
	<Card.Root padding="compact">
		<p class="text-small text-bahaya">{store.errors.detail}</p>
	</Card.Root>
{/if}

<script lang="ts">
	import type { GroupSummary } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import GroupPrivacyBadge from './group-privacy-badge.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		group: GroupSummary;
		onJoin?: (groupId: string) => void;
		onRequestJoin?: (groupId: string) => void;
		showManageHint?: boolean;
	}

	let { group, onJoin, onRequestJoin, showManageHint = false }: Props = $props();

	const href = $derived(`/komunitas/kelompok/${group.group_id}`);
	const canJoinNow = $derived(group.join_policy === 'terbuka');
	const canRequest = $derived(group.join_policy === 'persetujuan');
</script>

<article
	class="group rounded-xl border border-border/50 bg-card p-4 transition hover:border-border"
>
	<div class="flex items-start gap-3">
		<div
			class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary"
		>
			<UsersIcon class="size-5" />
		</div>

		<div class="min-w-0 flex-1">
			<div class="flex items-start justify-between gap-2">
				<a {href} class="min-w-0">
					<h3 class="truncate text-body font-bold text-foreground hover:underline">
						{group.name}
					</h3>
				</a>
				<GroupPrivacyBadge joinPolicy={group.join_policy} />
			</div>

			<p class="mt-1 line-clamp-2 text-small leading-relaxed text-muted-foreground/80">
				{group.description}
			</p>

			<div class="mt-2 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground/70">
				<span class="inline-flex items-center gap-1">
					<UsersIcon class="size-3" />
					{m.group_stat_members({ count: group.member_count })}
				</span>
				<span>•</span>
				<span>{m.group_stat_witnesses({ count: group.witness_count })}</span>
				{#if showManageHint}
					<span>•</span>
					<span class="inline-flex items-center gap-1">
						<SettingsIcon class="size-3" />
						{m.group_manage_hint()}
					</span>
				{/if}
			</div>
		</div>
	</div>

	<div class="mt-3 flex items-center gap-2">
		<Button variant="secondary" {href} class="flex-1">
			{m.group_action_view()}
		</Button>

		{#if canJoinNow}
			<Button variant="default" onclick={() => onJoin?.(group.group_id)}>
				<UserPlusIcon class="mr-1 size-3.5" />
				{m.group_action_join()}
			</Button>
		{:else if canRequest}
			<Button
				variant="outline"
				class="bg-primary/10 text-primary hover:bg-primary/15"
				onclick={() => onRequestJoin?.(group.group_id)}
			>
				<UserPlusIcon class="mr-1 size-3.5" />
				{m.group_action_request_join()}
			</Button>
		{:else}
			<Button
				variant="secondary"
				disabled
				class="text-muted-foreground"
				title={m.group_invite_only_tooltip()}
			>
				{m.group_action_invite_only()}
			</Button>
		{/if}
	</div>
</article>

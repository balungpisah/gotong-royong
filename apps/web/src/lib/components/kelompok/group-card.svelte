<script lang="ts">
	import type { GroupSummary } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import GroupPrivacyBadge from './group-privacy-badge.svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import SettingsIcon from '@lucide/svelte/icons/settings';

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

<article class="group rounded-xl border border-border/50 bg-card p-4 transition hover:border-border">
	<div class="flex items-start gap-3">
		<div class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
			<UsersIcon class="size-5" />
		</div>

		<div class="min-w-0 flex-1">
			<div class="flex items-start justify-between gap-2">
				<a href={href} class="min-w-0">
					<h3 class="truncate text-sm font-bold text-foreground hover:underline">
						{group.name}
					</h3>
				</a>
				<GroupPrivacyBadge joinPolicy={group.join_policy} />
			</div>

			<p class="mt-1 line-clamp-2 text-xs leading-relaxed text-muted-foreground/80">
				{group.description}
			</p>

			<div class="mt-2 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground/70">
				<span class="inline-flex items-center gap-1">
					<UsersIcon class="size-3" /> {m.group_stat_members({ count: group.member_count })}
				</span>
				<span>•</span>
				<span>{m.group_stat_witnesses({ count: group.witness_count })}</span>
				{#if showManageHint}
					<span>•</span>
					<span class="inline-flex items-center gap-1">
						<SettingsIcon class="size-3" /> {m.group_manage_hint()}
					</span>
				{/if}
			</div>
		</div>
	</div>

	<div class="mt-3 flex items-center gap-2">
		<a
			href={href}
			class="inline-flex flex-1 items-center justify-center rounded-lg bg-muted/40 px-3 py-2 text-xs font-semibold text-foreground transition hover:bg-muted/60"
		>
			{m.group_action_view()}
		</a>

		{#if canJoinNow}
			<button
				type="button"
				onclick={() => onJoin?.(group.group_id)}
				class="inline-flex items-center justify-center rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
			>
				<UserPlusIcon class="mr-1 size-3.5" />
				{m.group_action_join()}
			</button>
		{:else if canRequest}
			<button
				type="button"
				onclick={() => onRequestJoin?.(group.group_id)}
				class="inline-flex items-center justify-center rounded-lg bg-primary/10 px-3 py-2 text-xs font-semibold text-primary transition hover:bg-primary/15"
			>
				<UserPlusIcon class="mr-1 size-3.5" />
				{m.group_action_request_join()}
			</button>
		{:else}
			<button
				type="button"
				disabled
				class="inline-flex items-center justify-center rounded-lg bg-muted/30 px-3 py-2 text-xs font-semibold text-muted-foreground"
				title={m.group_invite_only_tooltip()}
			>
				{m.group_action_invite_only()}
			</button>
		{/if}
	</div>
</article>


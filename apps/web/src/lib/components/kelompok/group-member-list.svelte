<script lang="ts">
	import type { GroupMember, GroupMemberRole } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import CrownIcon from '@lucide/svelte/icons/crown';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';

	interface Props {
		members: GroupMember[];
		myRole: GroupMemberRole | null;
		onRemove?: (userId: string) => void;
		onUpdateRole?: (userId: string, role: GroupMemberRole) => void;
	}

	let { members, myRole, onRemove, onUpdateRole }: Props = $props();

	const canManage = $derived(myRole === 'admin' || myRole === 'moderator');

	const roleLabel = (role: GroupMemberRole) => {
		if (role === 'admin') return m.group_role_admin();
		if (role === 'moderator') return m.group_role_moderator();
		return m.group_role_anggota();
	};
</script>

<div class="space-y-2">
	{#each members as member (member.user_id)}
		<div class="flex items-center gap-3 rounded-lg border border-border/40 bg-muted/10 px-3 py-2">
			{#if member.avatar_url}
				<img src={member.avatar_url} alt={member.name} class="size-9 rounded-full object-cover" />
			{:else}
				<div class="flex size-9 items-center justify-center rounded-full bg-primary/10 text-primary">
					<span class="text-xs font-bold">{member.name.slice(0, 2).toUpperCase()}</span>
				</div>
			{/if}

			<div class="min-w-0 flex-1">
				<p class="truncate text-sm font-semibold text-foreground">{member.name}</p>
				<p class="text-[11px] text-muted-foreground/70">{roleLabel(member.role)}</p>
			</div>

			{#if canManage && onUpdateRole}
				<div class="hidden items-center gap-1 sm:flex">
					<button
						type="button"
						class="rounded-md p-1 text-muted-foreground/70 transition hover:bg-muted hover:text-foreground"
						title={m.group_action_make_admin()}
						onclick={() => onUpdateRole?.(member.user_id, 'admin')}
					>
						<CrownIcon class="size-4" />
					</button>
					<button
						type="button"
						class="rounded-md p-1 text-muted-foreground/70 transition hover:bg-muted hover:text-foreground"
						title={m.group_action_make_moderator()}
						onclick={() => onUpdateRole?.(member.user_id, 'moderator')}
					>
						<ShieldIcon class="size-4" />
					</button>
					<button
						type="button"
						class="rounded-md p-1 text-muted-foreground/70 transition hover:bg-muted hover:text-foreground"
						title={m.group_action_make_member()}
						onclick={() => onUpdateRole?.(member.user_id, 'anggota')}
					>
						<span class="text-xs font-bold">A</span>
					</button>
				</div>
			{/if}

			{#if canManage && onRemove}
				<button
					type="button"
					class="rounded-md p-1 text-bahaya/80 transition hover:bg-bahaya/10 hover:text-bahaya"
					title={m.group_action_remove_member()}
					onclick={() => onRemove?.(member.user_id)}
				>
					<UserMinusIcon class="size-4" />
				</button>
			{/if}
		</div>
	{/each}
</div>


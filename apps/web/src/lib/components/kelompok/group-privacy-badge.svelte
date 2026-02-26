<script lang="ts">
	import type { GroupJoinPolicy } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import LockOpenIcon from '@lucide/svelte/icons/lock-open';
	import ShieldQuestionIcon from '@lucide/svelte/icons/shield-question';
	import LockIcon from '@lucide/svelte/icons/lock';

	interface Props {
		joinPolicy: GroupJoinPolicy;
	}

	let { joinPolicy }: Props = $props();

	const meta = $derived(
		joinPolicy === 'terbuka'
			? {
					label: m.group_policy_terbuka(),
					Icon: LockOpenIcon,
					class: 'bg-berhasil/10 text-berhasil border-berhasil/20'
				}
			: joinPolicy === 'persetujuan'
				? {
						label: m.group_policy_persetujuan(),
						Icon: ShieldQuestionIcon,
						class: 'bg-primary/10 text-primary border-primary/20'
					}
				: {
						label: m.group_policy_undangan(),
						Icon: LockIcon,
						class: 'bg-muted/40 text-muted-foreground border-border/50'
					}
	);

	const BadgeIcon = $derived(meta.Icon);
</script>

<span
	class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-semibold {meta.class}"
>
	<BadgeIcon class="size-3" />
	<span>{meta.label}</span>
</span>

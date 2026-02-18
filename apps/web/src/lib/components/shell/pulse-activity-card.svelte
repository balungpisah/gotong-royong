<script lang="ts">
	import type { Witness } from '$lib/types';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import { StatusIndicator, type StatusIndicatorStatus } from '$lib/components/ui/status-indicator';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import UsersIcon from '@lucide/svelte/icons/users';
	import Clock from '@lucide/svelte/icons/clock';

	interface Props {
		witness: Witness;
		selected?: boolean;
		onclick?: () => void;
	}

	let { witness, selected = false, onclick }: Props = $props();

	const statusMap: Record<string, StatusIndicatorStatus> = {
		draft: 'done',
		open: 'stalled',
		active: 'active',
		resolved: 'done',
		closed: 'sealed'
	};

	const trackVariantMap: Record<string, BadgeVariant> = {
		tuntaskan: 'track-tuntaskan',
		wujudkan: 'track-wujudkan',
		telusuri: 'track-telusuri',
		rayakan: 'track-rayakan',
		musyawarah: 'track-musyawarah'
	};

	function timeAgo(dateStr: string): string {
		const now = Date.now();
		const then = new Date(dateStr).getTime();
		const diff = Math.max(0, now - then);
		const minutes = Math.floor(diff / 60000);
		if (minutes < 1) return 'Baru saja';
		if (minutes < 60) return `${minutes}m lalu`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}j lalu`;
		const days = Math.floor(hours / 24);
		if (days < 7) return `${days}h lalu`;
		return new Date(dateStr).toLocaleDateString('id-ID', { day: 'numeric', month: 'short' });
	}

	function handleKeydown(e: KeyboardEvent) {
		if (onclick && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onclick();
		}
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	role={onclick ? 'button' : 'article'}
	tabindex={onclick ? 0 : undefined}
	onclick={onclick}
	onkeydown={onclick ? handleKeydown : undefined}
	class="group rounded-xl border p-4 transition {selected
		? 'border-primary/40 bg-primary/5 shadow-sm'
		: 'border-border/60 bg-card hover:border-border hover:shadow-sm'} {onclick
		? 'cursor-pointer'
		: ''}"
>
	<div class="flex items-start gap-3">
		<div class="min-w-0 flex-1">
			<!-- Title row -->
			<div class="flex items-center gap-2">
				<StatusIndicator status={statusMap[witness.status] ?? 'active'} />
				<h3 class="truncate text-sm font-semibold text-foreground">
					{witness.title}
				</h3>
			</div>

			<!-- Summary -->
			<p class="mt-1 line-clamp-2 text-xs leading-relaxed text-muted-foreground">
				{witness.summary}
			</p>

			<!-- Meta row -->
			<div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
				{#if witness.track_hint}
					<Badge
						variant={trackVariantMap[witness.track_hint] ?? 'secondary'}
						class="text-[10px]"
					>
						{witness.track_hint}
					</Badge>
				{/if}

				<span class="inline-flex items-center gap-1">
					<UsersIcon class="size-3" />
					{witness.member_count}
				</span>

				<span class="inline-flex items-center gap-1">
					<MessageSquare class="size-3" />
					{witness.message_count}
				</span>

				{#if witness.unread_count > 0}
					<Badge variant="destructive" class="text-[10px]">
						{witness.unread_count} baru
					</Badge>
				{/if}

				<span class="ml-auto inline-flex items-center gap-1 text-muted-foreground">
					<Clock class="size-3" />
					{timeAgo(witness.updated_at)}
				</span>
			</div>
		</div>
	</div>
</div>

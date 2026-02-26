<script lang="ts">
	import type { MembershipRequest } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		requests: MembershipRequest[];
		onApprove?: (requestId: string) => void;
		onReject?: (requestId: string) => void;
	}

	let { requests, onApprove, onReject }: Props = $props();
</script>

{#if requests.length === 0}
	<p class="text-small text-muted-foreground/80">{m.group_requests_empty()}</p>
{:else}
	<div class="space-y-2">
		{#each requests as r (r.request_id)}
			<div class="rounded-lg border border-border/40 bg-muted/10 p-3">
				<div class="flex items-start gap-3">
					<div class="flex size-9 items-center justify-center rounded-full bg-primary/10 text-primary">
						<span class="text-small font-bold">{r.name.slice(0, 2).toUpperCase()}</span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="truncate text-body font-semibold text-foreground">{r.name}</p>
						{#if r.message}
							<p class="mt-0.5 text-small text-muted-foreground/80">{r.message}</p>
						{/if}
					</div>
					<div class="flex items-center gap-1">
						<Button
							variant="default"
							size="sm"
							onclick={() => onApprove?.(r.request_id)}
						>
							<UserPlusIcon class="mr-1 size-3.5" />
							{m.group_action_approve()}
						</Button>
						<Button
							variant="secondary"
							size="sm"
							onclick={() => onReject?.(r.request_id)}
						>
							<UserMinusIcon class="mr-1 size-3.5" />
							{m.group_action_reject()}
						</Button>
					</div>
				</div>
			</div>
		{/each}
	</div>
{/if}


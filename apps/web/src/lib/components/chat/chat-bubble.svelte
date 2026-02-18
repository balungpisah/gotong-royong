<script lang="ts">
	import type { UserMessage } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';

	let { message }: { message: UserMessage } = $props();

	const initials = $derived(message.author.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase());
	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));
</script>

<div class={cn('flex gap-2', message.is_self ? 'flex-row-reverse' : 'flex-row')} data-slot="chat-bubble">
	{#if !message.is_self}
		<Avatar class="size-8 shrink-0">
			<AvatarFallback class="text-[10px]">{initials}</AvatarFallback>
		</Avatar>
	{/if}
	<div class={cn('max-w-[75%] flex flex-col gap-1', message.is_self ? 'items-end' : 'items-start')}>
		{#if !message.is_self}
			<span class="text-[10px] font-medium text-muted-foreground">{message.author.name}</span>
		{/if}
		<div class={cn(
			'rounded-2xl px-3 py-2 text-sm',
			message.is_self
				? 'rounded-tr-sm bg-primary text-primary-foreground'
				: 'rounded-tl-sm bg-card border border-border'
		)}>
			{message.content}
		</div>
		{#if message.attachments?.length}
			<div class="flex flex-wrap gap-1.5 mt-1">
				{#each message.attachments as att}
					{#if att.type === 'image'}
						<img src={att.url} alt={att.alt || ''} class="h-20 w-auto rounded-lg border border-border object-cover" loading="lazy" />
					{/if}
				{/each}
			</div>
		{/if}
		<span class="text-[9px] text-muted-foreground">{timeStr}</span>
	</div>
</div>

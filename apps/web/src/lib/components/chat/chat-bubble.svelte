<script lang="ts">
	import type { UserMessage } from '$lib/types';
	import { cn } from '$lib/utils';
	import { TandangAvatar } from '$lib/components/ui/tandang-avatar';
	import Video from '@lucide/svelte/icons/video';
	import Mic from '@lucide/svelte/icons/mic';

	let { message }: { message: UserMessage } = $props();

	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));

	// ---------------------------------------------------------------------------
	// CD5: Social Influence — role badge + tier
	// ---------------------------------------------------------------------------

	const roleLabels: Record<string, string> = {
		pelapor: 'Pelapor',
		relawan: 'Relawan',
		koordinator: 'Koordinator',
		saksi: 'Saksi'
	};

	const roleColors: Record<string, string> = {
		pelapor: 'bg-peringatan/15 text-peringatan',
		relawan: 'bg-berhasil/15 text-berhasil',
		koordinator: 'bg-primary/15 text-primary',
		saksi: 'bg-bahaya/15 text-bahaya'
	};

	/** Tier display: small stars or shield */
	const tierDisplay = $derived.by(() => {
		const t = message.author.tier ?? 0;
		if (t <= 0) return '';
		if (t === 1) return '★';
		if (t === 2) return '★★';
		if (t === 3) return '★★★';
		return '★★★★';
	});

	const avatarPerson = $derived({
		user_id: message.author.user_id,
		name: message.author.name,
		avatar_url: message.author.avatar_url,
		tier: message.author.tier as import('$lib/types').TandangTierLevel | undefined,
		role: message.author.role
	});
</script>

<div class={cn('flex gap-2', message.is_self ? 'flex-row-reverse' : 'flex-row')} data-slot="chat-bubble">
	{#if !message.is_self}
		<a
			href="/profil/{message.author.user_id}"
			aria-label="Profil {message.author.name}"
			class="inline-flex rounded-full"
		>
			<TandangAvatar
				person={avatarPerson}
				size="sm"
				showTierDot
				interactive={false}
			/>
		</a>
	{/if}
	<div class={cn('max-w-[75%] flex flex-col gap-1', message.is_self ? 'items-end' : 'items-start')}>
		{#if !message.is_self}
			<div class="flex items-center gap-1.5">
				<a
					href="/profil/{message.author.user_id}"
					class="text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
				>
					{message.author.name}
				</a>
				<!-- CD5: Role badge -->
				{#if message.author.role}
					<span class="rounded px-1 py-0.5 text-[10px] font-bold uppercase tracking-wider {roleColors[message.author.role] ?? 'bg-muted text-muted-foreground'}">
						{roleLabels[message.author.role] ?? message.author.role}
					</span>
				{/if}
				<!-- CD5: Tier stars -->
				{#if tierDisplay}
					<span class="text-[10px] text-peringatan/70" title="Tier {message.author.tier}">
						{tierDisplay}
					</span>
				{/if}
			</div>
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
					{:else if att.type === 'video'}
						<div class="flex h-20 w-20 items-center justify-center rounded-lg bg-muted">
							<Video class="size-6 text-muted-foreground" />
						</div>
					{:else}
						<div class="flex h-20 w-20 items-center justify-center rounded-lg bg-muted">
							<Mic class="size-6 text-muted-foreground" />
						</div>
					{/if}
				{/each}
			</div>
		{/if}
		<span class="text-[10px] text-muted-foreground">{timeStr}</span>
	</div>
</div>

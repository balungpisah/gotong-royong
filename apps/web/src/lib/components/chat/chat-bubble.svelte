<script lang="ts">
	import type { UserMessage } from '$lib/types';
	import { cn } from '$lib/utils';
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';

	let { message }: { message: UserMessage } = $props();

	const initials = $derived(message.author.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase());
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

	/** Deterministic hue from name for avatar ring color */
	function nameHue(name: string): number {
		let hash = 0;
		for (let i = 0; i < name.length; i++) {
			hash = name.charCodeAt(i) + ((hash << 5) - hash);
		}
		return Math.abs(hash) % 360;
	}

	const avatarRingColor = $derived(`hsl(${nameHue(message.author.name)}, 55%, 45%)`);
</script>

<div class={cn('flex gap-2', message.is_self ? 'flex-row-reverse' : 'flex-row')} data-slot="chat-bubble">
	{#if !message.is_self}
		<div class="relative shrink-0">
			<Avatar class="size-8 ring-2 ring-offset-1 ring-offset-background" style="--tw-ring-color: {avatarRingColor}">
				<AvatarFallback class="text-[11px]">{initials}</AvatarFallback>
			</Avatar>
			<!-- Role dot on avatar -->
			{#if message.author.role}
				<div
					class="absolute -bottom-0.5 -right-0.5 size-2.5 rounded-full border-2 border-background {roleColors[message.author.role]?.split(' ')[0] ?? 'bg-muted'}"
				></div>
			{/if}
		</div>
	{/if}
	<div class={cn('max-w-[75%] flex flex-col gap-1', message.is_self ? 'items-end' : 'items-start')}>
		{#if !message.is_self}
			<div class="flex items-center gap-1.5">
				<span class="text-[11px] font-medium text-muted-foreground">{message.author.name}</span>
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
					{/if}
				{/each}
			</div>
		{/if}
		<span class="text-[10px] text-muted-foreground">{timeStr}</span>
	</div>
</div>

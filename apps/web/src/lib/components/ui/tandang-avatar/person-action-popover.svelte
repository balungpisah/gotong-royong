<script lang="ts">
	import type { TandangAvatarPerson, PersonRelation } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { HandshakeIcon, CircleHelpIcon, ChevronRightIcon } from '@lucide/svelte';

	interface Props {
		person: TandangAvatarPerson;
		relation?: PersonRelation;
		onvouch?: (userId: string) => void;
		onskeptis?: (userId: string) => void;
	}

	const { person, relation, onvouch, onskeptis }: Props = $props();

	const tierNames: Record<number, string> = {
		0: 'Bayangan',
		1: 'Pemula',
		2: 'Kontributor',
		3: 'Pilar',
		4: 'Kunci'
	};

	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};

	const tierPips = $derived.by(() => {
		const level = person.tier ?? 0;
		return '◆'.repeat(level) + '◇'.repeat(4 - level);
	});

	const tierName = $derived(tierNames[person.tier ?? 0] ?? 'Bayangan');
	const tierColor = $derived(tierColors[person.tier ?? 0] ?? '#9E9E9E');

	const initials = $derived.by(() => {
		const words = person.name.trim().split(/\s+/);
		return words
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('');
	});

	const isVouched = $derived(relation?.vouched ?? false);
	const isSkeptical = $derived(relation?.skeptical ?? false);
</script>

<div class="w-56 p-3 space-y-2.5">
	<!-- Person header -->
	<div class="flex items-center gap-2.5">
		<!-- Mini avatar -->
		<div class="size-9 shrink-0 rounded-full bg-muted flex items-center justify-center overflow-hidden">
			{#if person.avatar_url}
				<img src={person.avatar_url} alt={person.name} class="size-full object-cover" />
			{:else}
				<span class="text-[13px] font-bold text-foreground/70">{initials}</span>
			{/if}
		</div>
		<div class="min-w-0 flex-1">
			<p class="truncate text-sm font-semibold text-foreground">{person.name}</p>
			{#if person.tier !== undefined}
				<p class="text-caption" style="color: {tierColor};">
					{tierPips} {tierName}
				</p>
			{/if}
		</div>
	</div>

	<!-- Action buttons (vertical stack for full-width tap targets) -->
	<div class="flex flex-col gap-1.5">
		<button
			onclick={() => onvouch?.(person.user_id)}
			class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-micro font-medium transition-colors
				{isVouched
					? 'bg-signal-vouch/20 text-signal-vouch'
					: 'bg-muted/30 text-foreground/70 hover:bg-signal-vouch/10 hover:text-signal-vouch'}"
		>
			<HandshakeIcon class="size-3 shrink-0" />
			{m.tandang_aku_jamin()}
		</button>
		<button
			onclick={() => onskeptis?.(person.user_id)}
			class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-micro font-medium transition-colors
				{isSkeptical
					? 'bg-signal-skeptis/20 text-signal-skeptis'
					: 'bg-muted/30 text-foreground/70 hover:bg-signal-skeptis/10 hover:text-signal-skeptis'}"
		>
			<CircleHelpIcon class="size-3 shrink-0" />
			{m.tandang_skeptis()}
		</button>
	</div>

	<!-- View profile link -->
	<a
		href="/profil/{person.user_id}"
		class="flex items-center justify-between rounded-lg px-2 py-1 text-micro text-muted-foreground hover:bg-muted/20 hover:text-foreground transition-colors"
	>
		{m.tandang_view_profile()}
		<ChevronRightIcon class="size-3.5" />
	</a>
</div>

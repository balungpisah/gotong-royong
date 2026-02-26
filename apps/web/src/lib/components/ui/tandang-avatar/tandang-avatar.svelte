<script lang="ts">
	import type { TandangAvatarPerson, PersonRelation } from '$lib/types';
	import * as Avatar from '$lib/components/ui/avatar';
	import * as Popover from '$lib/components/ui/popover';
	import PersonActionPopover from './person-action-popover.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		person: TandangAvatarPerson;
		size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
		relation?: PersonRelation;
		interactive?: boolean;
		isSelf?: boolean;
		showTierDot?: boolean;
		class?: string;
		onvouch?: (userId: string) => void;
		onskeptis?: (userId: string) => void;
	}

	const {
		person,
		size = 'md',
		relation,
		interactive: interactiveProp,
		isSelf = false,
		showTierDot: showTierDotProp,
		class: className,
		onvouch,
		onskeptis
	}: Props = $props();

	// Default interactive to true for all sizes when not self
	const interactive = $derived(interactiveProp ?? !isSelf);

	// Default showTierDot to true when tier is available and size > xs
	const showTierDot = $derived(showTierDotProp ?? (person.tier !== undefined && size !== 'xs'));

	const initials = $derived.by(() => {
		const words = person.name.trim().split(/\s+/);
		return words
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('');
	});

	// --- Ring styles by relation state ---
	const ringColor = $derived.by(() => {
		if (isSelf) return '';
		if (relation?.vouched && relation?.vouched_back)
			return 'ring-signal-vouch shadow-[0_0_6px_var(--c-signal-vouch)]';
		if (relation?.vouched) return 'ring-signal-vouch';
		if (relation?.skeptical) return 'ring-signal-skeptis';
		return 'ring-transparent';
	});

	// --- Ring thickness by size ---
	const ringThickness: Record<string, string> = {
		xs: 'ring-[1.5px]',
		sm: 'ring-[1.5px] ring-offset-1',
		md: 'ring-2 ring-offset-1',
		lg: 'ring-2 ring-offset-2',
		xl: 'ring-[3px] ring-offset-2'
	};

	const ringClass = $derived.by(() => {
		if (isSelf) return '';
		return `${ringThickness[size]} ${ringColor} ring-offset-background`;
	});

	// --- Tier dot ---
	const tierColors: Record<number, string> = {
		0: 'var(--c-tier-0)',
		1: 'var(--c-tier-1)',
		2: 'var(--c-tier-2)',
		3: 'var(--c-tier-3)',
		4: 'var(--c-tier-4)'
	};

	const dotSizes: Record<string, string> = {
		sm: 'size-2',
		md: 'size-2.5',
		lg: 'size-3',
		xl: 'size-3.5'
	};

	const dotSize = $derived(dotSizes[size] ?? 'size-2');
	const tierColor = $derived(tierColors[person.tier ?? 0] ?? '#9E9E9E');

	let popoverOpen = $state(false);
</script>

{#if interactive}
	<Popover.Root bind:open={popoverOpen}>
		<Popover.Trigger
			onclick={(e: MouseEvent) => e.stopPropagation()}
			class={cn(
				'relative inline-flex shrink-0 cursor-pointer rounded-full focus:outline-none',
				className
			)}
		>
			<Avatar.Root {size} class={cn(ringClass, 'transition-shadow duration-200')}>
				{#if person.avatar_url}
					<Avatar.Image src={person.avatar_url} alt={person.name} />
				{/if}
				<Avatar.Fallback>{initials}</Avatar.Fallback>
			</Avatar.Root>
			{#if showTierDot && person.tier !== undefined}
				<span
					class="{dotSize} absolute bottom-0 right-0 rounded-full border border-background"
					style="background-color: {tierColor};"
				></span>
			{/if}
		</Popover.Trigger>
		<Popover.Content side="top" sideOffset={8}>
			<PersonActionPopover {person} {relation} {onvouch} {onskeptis} />
		</Popover.Content>
	</Popover.Root>
{:else}
	<span class={cn('relative inline-flex shrink-0 rounded-full', className)}>
		<Avatar.Root {size} class={cn(ringClass, 'transition-shadow duration-200')}>
			{#if person.avatar_url}
				<Avatar.Image src={person.avatar_url} alt={person.name} />
			{/if}
			<Avatar.Fallback>{initials}</Avatar.Fallback>
		</Avatar.Root>
		{#if showTierDot && person.tier !== undefined}
			<span
				class="{dotSize} absolute bottom-0 right-0 rounded-full border border-background"
				style="background-color: {tierColor};"
			></span>
		{/if}
	</span>
{/if}

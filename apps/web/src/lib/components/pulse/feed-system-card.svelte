<script lang="ts">
	import type { SystemCardData } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getFeedStore } from '$lib/stores';
	import X from '@lucide/svelte/icons/x';
	interface Props {
		card: SystemCardData;
		onDismiss?: () => void;
	}

	let { card, onDismiss }: Props = $props();

	const feedStore = getFeedStore();

	const iconMap: Record<string, string> = {
		lingkungan: '📍',
		topik: '🏷️',
		kelompok: '👥',
		lembaga: '🏢',
		warga: '👤'
	};
</script>

<div class="rounded-lg border border-border/30 bg-muted/20 p-3">
	<!-- Header row -->
	<div class="flex items-start justify-between gap-2">
		<div class="flex items-center gap-2 min-w-0">
			<span class="text-sm shrink-0">{card.icon}</span>
			<span class="text-xs font-semibold text-foreground truncate">{card.title}</span>
		</div>
		{#if card.dismissible && onDismiss}
			<button
				onclick={onDismiss}
				class="shrink-0 rounded-full p-0.5 text-muted-foreground/60 transition-colors hover:bg-muted hover:text-muted-foreground"
				aria-label={m.pulse_system_card_dismiss()}
			>
				<X class="size-3.5" />
			</button>
		{/if}
	</div>

	<!-- Description -->
	{#if card.description}
		<p class="mt-1 text-xs leading-relaxed text-muted-foreground">{card.description}</p>
	{/if}

	<!-- Variant-specific content -->
	{#if card.payload.variant === 'suggestion'}
		<div class="mt-2 flex flex-wrap gap-1.5">
			{#each card.payload.entities as entity (entity.entity_id)}
				<button
					onclick={() => feedStore.toggleFollow(entity.entity_id)}
					class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium transition-colors
						{entity.followed
						? 'bg-primary/10 text-primary'
						: 'bg-background border border-border/60 text-foreground hover:border-primary/40 hover:text-primary'}"
				>
					<span class="text-[10px]">{iconMap[entity.entity_type] ?? '📌'}</span>
					<span>{entity.label}</span>
					{#if !entity.followed}
						<span class="text-[10px] text-primary">+</span>
					{/if}
				</button>
			{/each}
		</div>
	{:else if card.payload.variant === 'tip'}
		<button class="mt-2 text-xs font-semibold text-primary hover:underline">
			{m.pulse_system_card_tip_learn()} →
		</button>
	{:else if card.payload.variant === 'milestone'}
		<div class="mt-2 flex items-baseline gap-1.5">
			<span class="text-lg font-bold text-primary">{card.payload.metric_value}</span>
			<span class="text-xs text-muted-foreground">{card.payload.metric_label}</span>
		</div>
	{:else if card.payload.variant === 'prompt'}
		<button
			class="mt-2 rounded-md bg-primary px-3 py-1 text-xs font-semibold text-primary-foreground transition-colors hover:bg-primary/90"
		>
			{card.payload.cta_label}
		</button>
	{/if}
</div>

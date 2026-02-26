<script lang="ts">
	import type { SystemCardData } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getFeedStore } from '$lib/stores';
	import X from '@lucide/svelte/icons/x';
	import { Button } from '$lib/components/ui/button';
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
			<span class="text-body shrink-0">{card.icon}</span>
			<span class="text-small font-semibold text-foreground truncate">{card.title}</span>
		</div>
		{#if card.dismissible && onDismiss}
			<Button
				variant="ghost"
				size="icon-sm"
				class="shrink-0 text-muted-foreground/60 hover:text-muted-foreground"
				onclick={onDismiss}
				aria-label={m.pulse_system_card_dismiss()}
			>
				<X class="size-3.5" />
			</Button>
		{/if}
	</div>

	<!-- Description -->
	{#if card.description}
		<p class="mt-1 text-small leading-relaxed text-muted-foreground">{card.description}</p>
	{/if}

	<!-- Variant-specific content -->
	{#if card.payload.variant === 'suggestion'}
		<div class="mt-2 flex flex-wrap gap-1.5">
			{#each card.payload.entities as entity (entity.entity_id)}
				<Button
					variant="ghost"
					size="pill"
					class={entity.followed
						? 'bg-primary/10 text-primary'
						: 'bg-background border border-border/60 text-foreground hover:border-primary/40 hover:text-primary'}
					onclick={() => feedStore.toggleFollow(entity.entity_id)}
				>
					<span class="text-[10px]">{iconMap[entity.entity_type] ?? '📌'}</span>
					<span>{entity.label}</span>
					{#if !entity.followed}
						<span class="text-[10px] text-primary">+</span>
					{/if}
				</Button>
			{/each}
		</div>
	{:else if card.payload.variant === 'tip'}
		<Button variant="link" class="mt-2 h-auto p-0">
			{m.pulse_system_card_tip_learn()} →
		</Button>
	{:else if card.payload.variant === 'milestone'}
		<div class="mt-2 flex items-baseline gap-1.5">
			<span class="text-h3 font-bold text-primary">{card.payload.metric_value}</span>
			<span class="text-small text-muted-foreground">{card.payload.metric_label}</span>
		</div>
	{:else if card.payload.variant === 'prompt'}
		<Button variant="default" size="sm" class="mt-2">
			{card.payload.cta_label}
		</Button>
	{/if}
</div>

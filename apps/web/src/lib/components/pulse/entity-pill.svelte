<script lang="ts">
	import type { EntityTag } from '$lib/types';
	import Tip from '$lib/components/ui/tip.svelte';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		tag: EntityTag;
		onclick?: () => void;
	}

	let { tag, onclick }: Props = $props();

	const iconMap: Record<string, string> = {
		lingkungan: '📍',
		topik: '🏷️',
		kelompok: '👥',
		lembaga: '🏢',
		warga: '👤'
	};

	const tooltip = $derived(
		tag.entity_type === 'kelompok' || tag.entity_type === 'lembaga'
			? 'Buka detail'
			: tag.followed
				? 'Diikuti'
				: 'Ketuk untuk ikuti'
	);

	const pillClass = $derived(
		`inline-flex items-center gap-0.5 rounded-full border px-2 py-0.5 text-small leading-tight transition-colors ${
			tag.followed
				? 'border-primary/30 bg-primary/5 text-primary'
				: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/50'
		} ${onclick || tag.entity_type === 'kelompok' || tag.entity_type === 'lembaga' ? 'cursor-pointer' : ''}`
	);

	function handleKeydown(e: KeyboardEvent) {
		if (onclick && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onclick();
		}
	}
</script>

{#snippet pillContent()}
	<span class="text-caption">{iconMap[tag.entity_type] ?? '📌'}</span>
	<span class="max-w-[8rem] truncate">{tag.label}</span>
{/snippet}

<Tip text={tooltip}>
	{#if tag.entity_type === 'kelompok' || tag.entity_type === 'lembaga'}
		<a href={`/komunitas/kelompok/${tag.entity_id}`} class={pillClass}>
			{@render pillContent()}
		</a>
	{:else if onclick}
		<Button
			variant="ghost"
			size="pill"
			onclick={onclick}
			onkeydown={handleKeydown}
			class={pillClass}
		>
			{@render pillContent()}
		</Button>
	{:else}
		<span class={pillClass}>
			{@render pillContent()}
		</span>
	{/if}
</Tip>

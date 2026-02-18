<script lang="ts">
	import type { EntityTag } from '$lib/types';

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

	function handleKeydown(e: KeyboardEvent) {
		if (onclick && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onclick();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
	onclick={onclick}
	onkeydown={onclick ? handleKeydown : undefined}
	class="inline-flex items-center gap-0.5 rounded-full border px-2 py-0.5 text-[10px] leading-tight transition-colors
		{tag.followed
		? 'border-primary/30 bg-primary/5 text-primary'
		: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/50'}
		{onclick ? 'cursor-pointer' : ''}"
>
	<span class="text-[9px]">{iconMap[tag.entity_type] ?? '📌'}</span>
	<span class="max-w-[8rem] truncate">{tag.label}</span>
</span>

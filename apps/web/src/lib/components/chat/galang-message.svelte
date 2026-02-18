<script lang="ts">
	import type { GalangMessage } from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import Coins from '@lucide/svelte/icons/coins';
	import ArrowDownRight from '@lucide/svelte/icons/arrow-down-right';
	import Trophy from '@lucide/svelte/icons/trophy';
	import type { Component } from 'svelte';

	let { message }: { message: GalangMessage } = $props();

	const subtypeConfig: Record<string, { icon: Component<{ class?: string }>; variant: string; label: string }> = {
		contribution: { icon: Coins, variant: 'success', label: 'Kontribusi' },
		disbursement: { icon: ArrowDownRight, variant: 'warning', label: 'Pencairan' },
		milestone: { icon: Trophy, variant: 'info', label: 'Tonggak' }
	};

	const config = $derived(subtypeConfig[message.subtype] || subtypeConfig.contribution);
	const Icon = $derived(config.icon);
	const timeStr = $derived(new Date(message.timestamp).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' }));

	const formatCurrency = (amount: number, currency: string = 'IDR') => {
		return new Intl.NumberFormat('id-ID', { style: 'currency', currency, maximumFractionDigits: 0 }).format(amount);
	};
</script>

<div class="flex items-center justify-center py-2" data-slot="galang-message">
	<div class="flex items-center gap-2 rounded-lg border border-border bg-card px-4 py-2.5 shadow-sm">
		<Icon class="size-4 text-muted-foreground" />
		<div class="flex flex-col">
			<div class="flex items-center gap-2">
				<span class="text-xs font-medium text-foreground">{message.content}</span>
				<Badge variant={config.variant as any} class="text-[9px]">{config.label}</Badge>
			</div>
			{#if message.amount != null}
				<span class="text-sm font-bold text-foreground">{formatCurrency(message.amount, message.currency)}</span>
			{/if}
		</div>
		<span class="text-[9px] text-muted-foreground">{timeStr}</span>
	</div>
</div>

<script lang="ts">
	import type { GalangMessage } from '$lib/types';

	let { message }: { message: GalangMessage } = $props();

	const formatCurrency = (amount: number, currency: string = 'IDR') => {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency,
			maximumFractionDigits: 0
		}).format(amount);
	};
</script>

<div class="flex flex-col gap-1" data-slot="galang-message">
	<span class="text-foreground">{message.content}</span>
	{#if message.amount != null}
		<span class="text-body font-medium text-foreground"
			>{formatCurrency(message.amount, message.currency)}</span
		>
	{/if}
</div>

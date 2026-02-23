<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { SignalFlowDataPoint } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		data: SignalFlowDataPoint[];
	}

	const { data }: Props = $props();

	const signalKeys = ['vouch', 'skeptis', 'dukung', 'proof_of_resolve', 'perlu_dicek'] as const;

	const signalColors: Record<string, string> = {
		vouch: 'var(--c-signal-vouch)',
		skeptis: 'var(--c-signal-skeptis)',
		dukung: 'var(--c-signal-dukung)',
		proof_of_resolve: 'var(--c-signal-proof)',
		perlu_dicek: 'var(--c-signal-dicek)'
	};

	const signalLabels = $derived({
		vouch: m.signal_vouch(),
		skeptis: m.signal_skeptis(),
		dukung: m.signal_dukung(),
		proof_of_resolve: m.signal_proof(),
		perlu_dicek: m.signal_perlu_dicek()
	} as Record<string, string>);

	// Chart layout constants
	const viewW = 400;
	const viewH = 200;
	const padLeft = 40;
	const padBottom = 24;
	const chartH = viewH - padBottom;
	const barW = 60;
	const weekGap = 90;

	type BarRect = {
		x: number;
		y: number;
		width: number;
		height: number;
		color: string;
		key: string;
	};

	const bars: BarRect[][] = $derived.by(() => {
		const totals = data.map((d) => signalKeys.reduce((sum, k) => sum + d[k], 0));
		const maxTotal = Math.max(...totals, 1);

		return data.map((d, i) => {
			const x = padLeft + i * weekGap;
			let yOffset = chartH;
			const rects: BarRect[] = [];

			for (const key of signalKeys) {
				const val = d[key];
				const h = (val / maxTotal) * (chartH - 10);
				yOffset -= h;
				rects.push({
					x,
					y: yOffset,
					width: barW,
					height: h,
					color: signalColors[key],
					key
				});
			}

			return rects;
		});
	});
</script>

<motion.div
	initial={{ opacity: 0, y: 12 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35, delay: 0.4 }}
	class="rounded-xl border border-border/30 bg-muted/10 p-4"
>
	<h3 class="text-xs font-semibold text-foreground">{m.komunitas_signal_flow_title()}</h3>

	<div class="mt-3 w-full overflow-hidden">
		<svg
			viewBox="0 0 {viewW} {viewH}"
			class="w-full"
			style="height: 160px;"
			role="img"
			aria-label={m.komunitas_signal_flow_title()}
		>
			<!-- Y-axis baseline -->
			<line
				x1={padLeft}
				y1={chartH}
				x2={viewW - 10}
				y2={chartH}
				stroke="currentColor"
				stroke-opacity="0.12"
				stroke-width="1"
			/>

			<!-- Bars -->
			{#each bars as weekBars, i}
				{#each weekBars as rect}
					<rect
						x={rect.x}
						y={rect.y}
						width={rect.width}
						height={rect.height}
						fill={rect.color}
						rx="2"
					/>
				{/each}

				<!-- Week label -->
				{#if data[i]}
					<text
						x={padLeft + i * weekGap + barW / 2}
						y={viewH - 4}
						text-anchor="middle"
						style="font-size: var(--font-size-caption)"
						fill="currentColor"
						fill-opacity="0.5"
					>{data[i].week_label}</text>
				{/if}
			{/each}
		</svg>
	</div>

	<!-- Legend -->
	<div class="mt-3 flex flex-wrap gap-3">
		{#each signalKeys as key}
			<div class="flex items-center gap-1.5">
				<div class="size-2.5 rounded-full" style="background: {signalColors[key]}"></div>
				<span class="text-caption text-muted-foreground">{signalLabels[key]}</span>
			</div>
		{/each}
	</div>
</motion.div>

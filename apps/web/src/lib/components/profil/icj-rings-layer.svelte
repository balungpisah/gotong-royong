<script lang="ts">
	import { getContext } from 'svelte';
	import type { Readable } from 'svelte/store';

	interface RingDatum {
		label: string;
		color: string;
		value: number;
	}

	interface Props {
		mounted: boolean;
	}

	const { mounted }: Props = $props();

	const { data, width, height } = getContext('LayerCake') as {
		data: Readable<RingDatum[]>;
		width: Readable<number>;
		height: Readable<number>;
	};

	const SW = 5;
	const GAP = 8;
</script>

{#each $data as ring, i}
	{@const cellW = $width / $data.length}
	{@const cx = cellW * i + cellW / 2}
	{@const cy = $height / 2}
	{@const maxR = Math.min(cellW / 2 - GAP, $height / 2 - 4)}
	{@const R = Math.max(maxR, 12)}
	{@const C = 2 * Math.PI * R}

	<!-- Background ring -->
	<circle
		cx={cx} cy={cy} r={R}
		fill="none"
		stroke="var(--color-muted)" stroke-opacity="0.15"
		stroke-width={SW}
	/>
	<!-- Animated progress arc -->
	<circle
		cx={cx} cy={cy} r={R}
		fill="none"
		stroke={ring.color}
		stroke-width={SW}
		stroke-linecap="round"
		stroke-dasharray={C}
		style="stroke-dashoffset: {mounted ? C * (1 - ring.value) : C}; transition: stroke-dashoffset 1s ease-out {i * 0.15}s;"
		transform="rotate(-90 {cx} {cy})"
	/>
	<!-- Score number -->
	<text
		x={cx} y={cy}
		text-anchor="middle"
		dominant-baseline="central"
		fill="var(--color-foreground)"
		font-size={R > 20 ? 14 : 11}
		font-weight="700"
	>
		{(ring.value * 100).toFixed(0)}
	</text>
{/each}

<script lang="ts">
	import { motion } from '@humanspeak/svelte-motion';
	import type { TandangScores } from '$lib/types';

	interface Props {
		scores: TandangScores;
		isSelf?: boolean;
		size?: number;
	}

	const { scores, isSelf = false, size = 200 }: Props = $props();

	const cx = $derived(size / 2);
	const cy = $derived(size / 2);
	const maxRadius = $derived(size / 2 - 30);

	function polarToCartesian(
		centerX: number,
		centerY: number,
		r: number,
		angleDeg: number
	): [number, number] {
		const rad = (angleDeg - 90) * (Math.PI / 180);
		return [centerX + r * Math.cos(rad), centerY + r * Math.sin(rad)];
	}

	// Axes: Integrity=0°, Competence=120°, Judgment=240°
	const axes = [
		{ label: 'I', angle: 0, color: 'var(--c-tandang-i)' },
		{ label: 'C', angle: 120, color: 'var(--c-tandang-c)' },
		{ label: 'J', angle: 240, color: 'var(--c-tandang-j)' }
	];

	const axisValues = $derived([
		scores.integrity.value,
		scores.competence.aggregate,
		scores.judgment.value
	]);

	// Ring points at 33%, 66%, 100%
	const ringFractions = [0.33, 0.66, 1.0];

	const rings = $derived(
		ringFractions.map((frac) => {
			const r = maxRadius * frac;
			const pts = axes.map((ax) => {
				const [x, y] = polarToCartesian(cx, cy, r, ax.angle);
				return `${x},${y}`;
			});
			return pts.join(' ');
		})
	);

	const dataPolygon = $derived(() => {
		const pts = axes.map((ax, i) => {
			const r = maxRadius * axisValues[i];
			const [x, y] = polarToCartesian(cx, cy, r, ax.angle);
			return `${x},${y}`;
		});
		return pts.join(' ');
	});

	const vertexPoints = $derived(
		axes.map((ax, i) => {
			const r = maxRadius * axisValues[i];
			const [x, y] = polarToCartesian(cx, cy, r, ax.angle);
			return { x, y, color: ax.color };
		})
	);

	const labelPoints = $derived(
		axes.map((ax, i) => {
			const r = maxRadius + 16;
			const [x, y] = polarToCartesian(cx, cy, r, ax.angle);
			const value = axisValues[i];
			return {
				x,
				y,
				label: ax.label,
				color: ax.color,
				value: (value * 100).toFixed(0),
				angle: ax.angle
			};
		})
	);

	const axisLines = $derived(
		axes.map((ax) => {
			const [x, y] = polarToCartesian(cx, cy, maxRadius, ax.angle);
			return { x1: cx, y1: cy, x2: x, y2: y };
		})
	);
</script>

<motion.div
	initial={{ opacity: 0, scale: 0.9 }}
	animate={{ opacity: 1, scale: 1 }}
	transition={{ duration: 0.4 }}
	class="flex justify-center"
>
	<svg
		width={size}
		height={size}
		viewBox="0 0 {size} {size}"
		role="img"
		aria-label="ICJ Radar Chart"
	>
		<!-- Background rings -->
		{#each rings as pts, ri (ri)}
			<polygon
				points={pts}
				fill="none"
				stroke="currentColor"
				stroke-opacity="0.08"
				stroke-width="1"
			/>
		{/each}

		<!-- Axis lines -->
		{#each axisLines as line, li (li)}
			<line
				x1={line.x1}
				y1={line.y1}
				x2={line.x2}
				y2={line.y2}
				stroke="currentColor"
				stroke-opacity="0.12"
				stroke-width="1"
			/>
		{/each}

		<!-- Data polygon -->
		<polygon
			points={dataPolygon()}
			fill="var(--color-primary)"
			fill-opacity="0.2"
			stroke="var(--color-primary)"
			stroke-width="1.5"
			stroke-opacity="0.8"
		/>

		<!-- Score dots -->
		{#each vertexPoints as pt, vi (vi)}
			<circle cx={pt.x} cy={pt.y} r="4" fill={pt.color} opacity="0.9" />
		{/each}

		<!-- Labels -->
		{#each labelPoints as lp, lpi (lpi)}
			<text
				x={lp.x}
				y={lp.y}
				text-anchor="middle"
				dominant-baseline="middle"
				style="font-size: var(--font-size-caption)"
				font-weight="600"
				fill={lp.color}
			>
				{#if isSelf}
					{lp.label} {lp.value}
				{:else}
					{lp.label}
				{/if}
			</text>
		{/each}
	</svg>
</motion.div>

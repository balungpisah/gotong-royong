<script lang="ts">
	import type { TandangProfile } from '$lib/types';

	interface Props {
		profile: TandangProfile;
	}

	const { profile }: Props = $props();

	type SkyMood = 'dawn' | 'morning' | 'midday' | 'golden' | 'overcast';

	function getMood(p: TandangProfile): SkyMood {
		const hasDecay = p.decay_warnings.length > 0;
		const streak = p.consistency.streak_weeks;
		const quality = p.consistency.quality_avg;
		const tier = p.tier.level;

		if (hasDecay && streak === 0) return 'overcast';
		if (tier >= 3 && quality >= 0.65) return 'golden';
		if (tier >= 2 && streak >= 2) return 'midday';
		if (streak >= 1 || tier >= 1) return 'morning';
		return 'dawn';
	}

	const mood = $derived(getMood(profile));

	const gradients: Record<SkyMood, string> = {
		dawn:     'linear-gradient(to bottom, #1c0a3a 0%, #4a1a6b 40%, #b84a28 72%, #f4a261 100%)',
		morning:  'linear-gradient(to bottom, #0a3d6b 0%, #1565c0 38%, #42a5f5 72%, #b3e5fc 100%)',
		midday:   'linear-gradient(to bottom, #072e5c 0%, #0d47a1 30%, #1976d2 62%, #64b5f6 100%)',
		golden:   'linear-gradient(to bottom, #150422 0%, #6a0f55 28%, #c94a00 62%, #ffb300 100%)',
		overcast: 'linear-gradient(to bottom, #1c2526 0%, #37474f 40%, #607d8b 72%, #b0bec5 100%)',
	};

	const labels: Record<SkyMood, string> = {
		dawn:     'Fajar — perjalanan baru dimulai',
		morning:  'Pagi cerah — momentum tumbuh',
		midday:   'Siang penuh — kontribusi nyata',
		golden:   'Senja keemasan — puncak kontribusi',
		overcast: 'Mendung sejenak — butuh perhatian',
	};
</script>

<div class="sky-portrait" style="background: {gradients[mood]}">
	<!-- Celestial SVG layer -->
	<svg
		class="sky-svg"
		viewBox="0 0 400 130"
		preserveAspectRatio="xMidYMid slice"
		xmlns="http://www.w3.org/2000/svg"
		aria-hidden="true"
	>
		<defs>
			<filter id="blur-soft" x="-60%" y="-60%" width="220%" height="220%">
				<feGaussianBlur stdDeviation="9" />
			</filter>
			<filter id="blur-hard" x="-100%" y="-100%" width="300%" height="300%">
				<feGaussianBlur stdDeviation="18" />
			</filter>
			<radialGradient id="glow-dawn" cx="50%" cy="100%" r="60%">
				<stop offset="0%"   stop-color="#ffd54f" stop-opacity="0.9"/>
				<stop offset="45%"  stop-color="#ff8a65" stop-opacity="0.5"/>
				<stop offset="100%" stop-color="#c05428" stop-opacity="0"/>
			</radialGradient>
			<radialGradient id="glow-morning" cx="50%" cy="50%" r="50%">
				<stop offset="0%"   stop-color="#fff9c4" stop-opacity="1"/>
				<stop offset="40%"  stop-color="#ffee58" stop-opacity="0.7"/>
				<stop offset="100%" stop-color="#ffa726" stop-opacity="0"/>
			</radialGradient>
			<radialGradient id="glow-midday" cx="50%" cy="50%" r="50%">
				<stop offset="0%"   stop-color="#ffffff"  stop-opacity="1"/>
				<stop offset="35%"  stop-color="#fff9c4"  stop-opacity="0.8"/>
				<stop offset="100%" stop-color="#42a5f5"  stop-opacity="0"/>
			</radialGradient>
			<radialGradient id="glow-golden" cx="50%" cy="80%" r="65%">
				<stop offset="0%"   stop-color="#ffec3d" stop-opacity="1"/>
				<stop offset="30%"  stop-color="#ff9800" stop-opacity="0.75"/>
				<stop offset="65%"  stop-color="#e64a19" stop-opacity="0.35"/>
				<stop offset="100%" stop-color="#c94a00" stop-opacity="0"/>
			</radialGradient>
			<radialGradient id="glow-overcast" cx="40%" cy="35%" r="50%">
				<stop offset="0%"   stop-color="#eceff1" stop-opacity="0.4"/>
				<stop offset="100%" stop-color="#90a4ae" stop-opacity="0"/>
			</radialGradient>
		</defs>

		{#if mood === 'dawn'}
			<!-- Wide horizon glow -->
			<ellipse cx="200" cy="135" rx="220" ry="90" fill="url(#glow-dawn)" class="pulse-slow" />
			<!-- Sun disc, half-risen -->
			<circle cx="200" cy="133" r="46" fill="#e86830" filter="url(#blur-soft)" class="pulse-slow" />
			<circle cx="200" cy="133" r="34" fill="#f4a261" />
			<circle cx="200" cy="133" r="22" fill="#ffd699" />
			<circle cx="200" cy="133" r="12" fill="#fff3e0" />
			<!-- Horizon blush -->
			<ellipse cx="200" cy="130" rx="400" ry="18" fill="#c05428" opacity="0.18" />
		{/if}

		{#if mood === 'morning'}
			<!-- Sun corona -->
			<circle cx="200" cy="30" r="62" fill="url(#glow-morning)" class="pulse-slow" />
			<!-- Sun core -->
			<circle cx="200" cy="30" r="22" fill="#ffe082" filter="url(#blur-soft)" />
			<circle cx="200" cy="30" r="15" fill="#fff9c4" />
			<!-- Left cloud -->
			<g class="drift-right" opacity="0.85">
				<ellipse cx="72"  cy="76" rx="44" ry="17" fill="white"/>
				<ellipse cx="50"  cy="83" rx="27" ry="14" fill="white"/>
				<ellipse cx="98"  cy="83" rx="32" ry="13" fill="white"/>
			</g>
			<!-- Right cloud -->
			<g class="drift-left" opacity="0.70">
				<ellipse cx="318" cy="60" rx="38" ry="14" fill="white"/>
				<ellipse cx="299" cy="67" rx="24" ry="11" fill="white"/>
				<ellipse cx="340" cy="66" rx="26" ry="11" fill="white"/>
			</g>
		{/if}

		{#if mood === 'midday'}
			<!-- Sun corona -->
			<circle cx="200" cy="20" r="55" fill="url(#glow-midday)" class="pulse-slow" />
			<!-- Sun core -->
			<circle cx="200" cy="20" r="20" fill="#fff9c4" filter="url(#blur-soft)" />
			<circle cx="200" cy="20" r="13" fill="white" />
			<!-- Left cloud (small) -->
			<g class="drift-right" opacity="0.60">
				<ellipse cx="88"  cy="68" rx="34" ry="12" fill="white"/>
				<ellipse cx="68"  cy="74" rx="22" ry="10" fill="white"/>
				<ellipse cx="110" cy="74" rx="24" ry="10" fill="white"/>
			</g>
			<!-- Right cloud (small) -->
			<g class="drift-left" opacity="0.50">
				<ellipse cx="322" cy="82" rx="30" ry="11" fill="white"/>
				<ellipse cx="305" cy="88" rx="19" ry="9"  fill="white"/>
				<ellipse cx="342" cy="88" rx="22" ry="9"  fill="white"/>
			</g>
		{/if}

		{#if mood === 'golden'}
			<!-- Deep atmospheric glow -->
			<ellipse cx="200" cy="165" rx="280" ry="130" fill="url(#glow-golden)" class="pulse-slow" />
			<!-- Sun body layers -->
			<circle cx="200" cy="118" r="80"  fill="#ff9800" opacity="0.18" filter="url(#blur-hard)" />
			<circle cx="200" cy="118" r="55"  fill="#ffc107" opacity="0.50" />
			<circle cx="200" cy="118" r="36"  fill="#ffec3d" opacity="0.82" />
			<circle cx="200" cy="118" r="20"  fill="#fff9c4" />
			<!-- Atmospheric bands -->
			<ellipse cx="200" cy="130" rx="400" ry="45" fill="#c94a00" opacity="0.10"/>
			<ellipse cx="200" cy="130" rx="400" ry="28" fill="#ff9800" opacity="0.08"/>
		{/if}

		{#if mood === 'overcast'}
			<!-- Diffuse light source -->
			<ellipse cx="155" cy="42" rx="95" ry="52" fill="url(#glow-overcast)" />
			<!-- Cloud layer 1 (back) -->
			<g opacity="0.48">
				<ellipse cx="115" cy="36" rx="105" ry="28" fill="#cfd8dc"/>
				<ellipse cx="78"  cy="47" rx="68"  ry="22" fill="#b0bec5"/>
				<ellipse cx="152" cy="49" rx="82"  ry="24" fill="#cfd8dc"/>
			</g>
			<!-- Cloud layer 2 (front right) -->
			<g opacity="0.60">
				<ellipse cx="285" cy="52" rx="112" ry="30" fill="#b0bec5"/>
				<ellipse cx="248" cy="63" rx="72"  ry="22" fill="#90a4ae"/>
				<ellipse cx="320" cy="62" rx="88"  ry="24" fill="#b0bec5"/>
			</g>
			<!-- Cloud layer 3 (foreground sweep) -->
			<g class="drift-right" opacity="0.38">
				<ellipse cx="200" cy="96" rx="210" ry="32" fill="#90a4ae"/>
			</g>
		{/if}
	</svg>

	<!-- Text scrim + mood label -->
	<div class="sky-label">
		{labels[mood]}
	</div>
</div>

<style>
	.sky-portrait {
		position: relative;
		height: 128px;
		border-radius: 0.75rem;
		overflow: hidden;
		transition: background 1.4s ease;
	}

	.sky-svg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	.sky-label {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 0.45rem 0.75rem;
		background: linear-gradient(to top, rgba(0, 0, 0, 0.48) 0%, transparent 100%);
		font-size: 10.5px;
		line-height: 1.45;
		color: rgba(255, 255, 255, 0.88);
		font-weight: 500;
		letter-spacing: 0.015em;
	}

	@keyframes pulse-slow {
		0%, 100% { opacity: 0.80; }
		50%       { opacity: 1;    }
	}

	@keyframes drift-right {
		0%, 100% { transform: translateX(0);   }
		50%      { transform: translateX(7px); }
	}

	@keyframes drift-left {
		0%, 100% { transform: translateX(0);    }
		50%      { transform: translateX(-6px); }
	}

	.pulse-slow  { animation: pulse-slow  5s ease-in-out infinite; }
	.drift-right { animation: drift-right 9s ease-in-out infinite; }
	.drift-left  { animation: drift-left 11s ease-in-out infinite; }

	@media (prefers-reduced-motion: reduce) {
		.pulse-slow, .drift-right, .drift-left { animation: none; }
	}
</style>

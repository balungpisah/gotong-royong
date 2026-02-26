<script lang="ts">
	import Activity from '@lucide/svelte/icons/activity';
	import { motion } from '@humanspeak/svelte-motion';
	import type { CommunityDashboard } from '$lib/types';
	import GdfWeatherWidget from './gdf-weather-widget.svelte';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		dashboard: CommunityDashboard;
	}

	const { dashboard }: Props = $props();
</script>

<motion.div
	initial={{ opacity: 0, y: 8 }}
	animate={{ opacity: 1, y: 0 }}
	transition={{ duration: 0.35 }}
	class="space-y-4"
>
	<!-- Title row -->
	<div class="flex items-center gap-3">
		<div class="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
			<Activity class="size-5" />
		</div>
		<div>
			<h2 class="text-h2 font-bold text-foreground">{dashboard.community_name}</h2>
			<p class="text-caption text-muted-foreground">
				{m.komunitas_member_count({ count: String(dashboard.member_count) })}
			</p>
		</div>
	</div>

	<!-- Weather widget -->
	<GdfWeatherWidget weather={dashboard.weather} size="full" />
</motion.div>

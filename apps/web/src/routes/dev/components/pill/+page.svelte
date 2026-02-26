<script lang="ts">
	import { Pill, type PillVariant } from '$lib/components/ui/pill';

	const defaultVariants: PillVariant[] = ['default', 'active', 'success'];

	const trackVariants: PillVariant[] = [
		'track-tuntaskan',
		'track-wujudkan',
		'track-telusuri',
		'track-rayakan',
		'track-musyawarah'
	];
</script>

<h1 class="mb-6 text-h1 font-extrabold">Pill / Tag</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Default Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each defaultVariants as variant (variant)}
			<Pill {variant}>{variant}</Pill>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Track Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each trackVariants as variant (variant)}
			<Pill {variant}>{variant}</Pill>
		{/each}
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Removable</h2>
	<div class="flex flex-wrap items-center gap-3">
		<Pill removable onremove={() => {}}>Removable default</Pill>
		<Pill variant="active" removable onremove={() => {}}>Removable active</Pill>
		<Pill variant="track-tuntaskan" removable onremove={() => {}}>Removable track</Pill>
	</div>
</section>

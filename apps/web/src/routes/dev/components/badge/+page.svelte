<script lang="ts">
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';

	const baseVariants: BadgeVariant[] = ['default', 'secondary', 'destructive', 'outline'];

	const trackVariants: BadgeVariant[] = [
		'track-tuntaskan',
		'track-wujudkan',
		'track-telusuri',
		'track-rayakan',
		'track-musyawarah'
	];

	const semanticVariants: BadgeVariant[] = ['success', 'warning', 'danger', 'info'];

	const stepperVariants: BadgeVariant[] = ['step', 'step-done', 'step-future'];

	const specialVariants: BadgeVariant[] = ['rahasia', 'vault', 'siaga', 'confidence'];
</script>

<h1 class="mb-6 text-2xl font-bold">Badge</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Base Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each baseVariants as variant (variant)}
			<Badge {variant}>{variant}</Badge>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Track Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each trackVariants as variant (variant)}
			<Badge {variant}>{variant}</Badge>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Semantic Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each semanticVariants as variant (variant)}
			<Badge {variant}>{variant}</Badge>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Stepper Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each stepperVariants as variant (variant)}
			<Badge {variant}>{variant}</Badge>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Special Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each specialVariants as variant (variant)}
			<Badge {variant}>{variant}</Badge>
		{/each}
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">As Link</h2>
	<div class="flex flex-wrap items-center gap-3">
		<Badge href="#" variant="default">Default link</Badge>
		<Badge href="#" variant="track-wujudkan">Track link</Badge>
		<Badge href="#" variant="vault">Vault link</Badge>
	</div>
</section>

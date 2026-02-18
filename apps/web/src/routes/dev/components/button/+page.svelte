<script lang="ts">
	import { Button, type ButtonVariant, type ButtonSize } from '$lib/components/ui/button';

	const baseVariants: ButtonVariant[] = [
		'default',
		'destructive',
		'outline',
		'secondary',
		'ghost',
		'link'
	];

	const dnaVariants: ButtonVariant[] = [
		'track-tuntaskan',
		'track-wujudkan',
		'track-telusuri',
		'track-rayakan',
		'track-musyawarah',
		'vault',
		'siaga'
	];

	const sizes: ButtonSize[] = ['sm', 'default', 'lg'];

	const sizeShowcaseVariants: ButtonVariant[] = ['default', 'track-tuntaskan', 'vault'];
</script>

<h1 class="mb-6 text-2xl font-bold">Button</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Base Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each baseVariants as variant (variant)}
			<Button {variant}>{variant}</Button>
		{/each}
		<Button disabled>disabled</Button>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Track Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		{#each dnaVariants.slice(0, 5) as variant (variant)}
			<Button {variant}>{variant}</Button>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">DNA — Mode Variants</h2>
	<div class="flex flex-wrap items-center gap-3">
		<Button variant="vault">vault</Button>
		<Button variant="siaga">siaga</Button>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Sizes</h2>
	{#each sizeShowcaseVariants as variant (variant)}
		<div class="mb-3 flex flex-wrap items-center gap-3">
			{#each sizes as size (size)}
				<Button {variant} {size}>{variant} — {size}</Button>
			{/each}
		</div>
	{/each}
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">As Link</h2>
	<div class="flex flex-wrap items-center gap-3">
		<Button href="#" variant="default">Default link</Button>
		<Button href="#" variant="track-wujudkan">Track link</Button>
		<Button href="#" variant="vault">Vault link</Button>
	</div>
</section>

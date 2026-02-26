<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';
	import type { CardVariant, CardPadding } from '$lib/components/ui/card';

	const variants: CardVariant[] = ['default', 'elevated', 'outlined', 'flat', 'glass'];
	const paddings: CardPadding[] = ['default', 'compact', 'none'];

	const trackBadges: { variant: BadgeVariant; label: string }[] = [
		{ variant: 'track-tuntaskan', label: 'tuntaskan' },
		{ variant: 'track-wujudkan', label: 'wujudkan' },
		{ variant: 'track-telusuri', label: 'telusuri' }
	];
</script>

<h1 class="mb-2 text-h1 font-extrabold">Card</h1>
<p class="mb-8 text-body text-muted-foreground">
	7-part card system with 5 visual variants, 3 padding options, interactive and selected states.
</p>

<!-- Variant Matrix -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Variants</h2>
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each variants as variant (variant)}
			<Card.Root {variant} padding="compact">
				<Card.Header>
					<Card.Title class="text-small">{variant}</Card.Title>
					<Card.Description>Card variant: {variant}</Card.Description>
				</Card.Header>
				<Card.Content>
					<p class="text-small text-muted-foreground">Content area for the {variant} variant.</p>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
</section>

<!-- Padding Options -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Padding</h2>
	<div class="grid gap-4 sm:grid-cols-3">
		{#each paddings as padding (padding)}
			<Card.Root {padding}>
				<Card.Header>
					<Card.Title class="text-small">padding="{padding}"</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-small text-muted-foreground">
						{padding === 'default'
							? 'Standard spacing (py-6, px-6 on children)'
							: padding === 'compact'
								? 'Compact spacing (p-4) for dense layouts'
								: 'No padding — full control to children'}
					</p>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
</section>

<!-- Interactive -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Interactive + States</h2>
	<div class="grid gap-4 sm:grid-cols-3">
		<Card.Root interactive padding="compact">
			<Card.Header>
				<Card.Title class="text-small">Interactive (idle)</Card.Title>
			</Card.Header>
			<Card.Content>
				<p class="text-small text-muted-foreground">Hover me — cursor and border change.</p>
			</Card.Content>
		</Card.Root>

		<Card.Root interactive state="selected" padding="compact">
			<Card.Header>
				<Card.Title class="text-small">Selected</Card.Title>
			</Card.Header>
			<Card.Content>
				<p class="text-small text-muted-foreground">Primary border + subtle background tint.</p>
			</Card.Content>
		</Card.Root>

		<Card.Root variant="elevated" interactive padding="compact">
			<Card.Header>
				<Card.Title class="text-small">Elevated + Interactive</Card.Title>
			</Card.Header>
			<Card.Content>
				<p class="text-small text-muted-foreground">Shadow grows on hover.</p>
			</Card.Content>
		</Card.Root>
	</div>
</section>

<!-- Full Composition -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Full Composition</h2>
	<div class="max-w-sm">
		<Card.Root>
			<Card.Header>
				<Card.Title>Project Update</Card.Title>
				<Card.Description>Latest status for this project.</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="space-y-2">
					<div class="flex items-center justify-between text-body">
						<span>Progress</span>
						<span class="font-semibold">72%</span>
					</div>
					<div class="flex gap-2">
						<Badge variant="success">On Track</Badge>
						<Badge variant="info">Sprint 3</Badge>
					</div>
				</div>
			</Card.Content>
			<Card.Footer class="justify-end gap-2">
				<Button variant="ghost" size="sm">Details</Button>
				<Button variant="track-tuntaskan" size="sm">Take Action</Button>
			</Card.Footer>
		</Card.Root>
	</div>
</section>

<!-- Track Badges Grid -->
<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Cards Grid with Track Badges</h2>
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each trackBadges as item (item.variant)}
			<Card.Root padding="compact">
				<Card.Header>
					<div class="flex items-center justify-between">
						<Card.Title class="text-small">{item.variant}</Card.Title>
						<Badge variant={item.variant}>{item.label}</Badge>
					</div>
				</Card.Header>
				<Card.Content>
					<p class="text-small text-muted-foreground">Example card with a track badge variant.</p>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
</section>

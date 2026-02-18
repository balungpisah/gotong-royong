<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';

	const trackBadges: { variant: BadgeVariant; label: string }[] = [
		{ variant: 'track-tuntaskan', label: 'tuntaskan' },
		{ variant: 'track-wujudkan', label: 'wujudkan' },
		{ variant: 'track-telusuri', label: 'telusuri' }
	];
</script>

<h1 class="mb-6 text-2xl font-bold">Card</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Basic Card</h2>
	<div class="max-w-sm">
		<Card.Root>
			<Card.Header>
				<Card.Title>Card Title</Card.Title>
				<Card.Description>Card description goes here.</Card.Description>
			</Card.Header>
			<Card.Content>
				<p class="text-sm">This is the card content area.</p>
			</Card.Content>
			<Card.Footer>
				<Button variant="outline" size="sm">Cancel</Button>
				<Button size="sm">Save</Button>
			</Card.Footer>
		</Card.Root>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Cards Grid</h2>
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
		{#each trackBadges as item (item.variant)}
			<Card.Root>
				<Card.Header>
					<div class="flex items-center justify-between">
						<Card.Title class="text-sm">{item.variant}</Card.Title>
						<Badge variant={item.variant}>{item.label}</Badge>
					</div>
				</Card.Header>
				<Card.Content>
					<p class="text-xs text-muted-foreground">Example card with a track badge variant.</p>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Card with Actions</h2>
	<div class="max-w-sm">
		<Card.Root>
			<Card.Header>
				<Card.Title>Project Update</Card.Title>
				<Card.Description>Latest status for this project.</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="space-y-2">
					<div class="flex items-center justify-between text-sm">
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

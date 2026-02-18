<script lang="ts">
	import { Avatar, AvatarImage, AvatarFallback, AvatarGroup } from '$lib/components/ui/avatar';
	import type { AvatarSize } from '$lib/components/ui/avatar/avatar.svelte';

	const sizes: AvatarSize[] = ['xs', 'sm', 'md', 'lg', 'xl'];
</script>

<h1 class="mb-6 text-2xl font-bold">Avatar</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Sizes</h2>
	<div class="flex flex-wrap items-end gap-4">
		{#each sizes as size (size)}
			<div class="flex flex-col items-center gap-1">
				<Avatar {size}>
					<AvatarFallback>{size?.toUpperCase()}</AvatarFallback>
				</Avatar>
				<span class="text-xs text-muted-foreground">{size}</span>
			</div>
		{/each}
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">With Image</h2>
	<div class="flex flex-wrap items-end gap-4">
		{#each sizes as size (size)}
			<Avatar {size}>
				<AvatarImage
					src="https://api.dicebear.com/9.x/initials/svg?seed={size}"
					alt={size ?? 'md'}
				/>
				<AvatarFallback>{size?.toUpperCase()}</AvatarFallback>
			</Avatar>
		{/each}
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Avatar Group</h2>
	<AvatarGroup>
		<Avatar size="md">
			<AvatarFallback>AB</AvatarFallback>
		</Avatar>
		<Avatar size="md">
			<AvatarFallback>CD</AvatarFallback>
		</Avatar>
		<Avatar size="md">
			<AvatarFallback>EF</AvatarFallback>
		</Avatar>
		<Avatar size="md">
			<AvatarFallback>+3</AvatarFallback>
		</Avatar>
	</AvatarGroup>
</section>

<script lang="ts">
	import { Separator } from '$lib/components/ui/separator';
</script>

<h1 class="mb-6 text-h1 font-extrabold">Separator</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Horizontal</h2>
	<div class="max-w-md space-y-3">
		<p class="text-body">Content above separator</p>
		<Separator />
		<p class="text-body">Content below separator</p>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Vertical</h2>
	<div class="flex h-8 items-center gap-3">
		<span class="text-body">Left</span>
		<Separator orientation="vertical" />
		<span class="text-body">Middle</span>
		<Separator orientation="vertical" />
		<span class="text-body">Right</span>
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">In Context</h2>
	<div class="max-w-md rounded-lg border p-4">
		<h3 class="text-h3 font-semibold">Section Title</h3>
		<p class="mt-1 text-small text-muted-foreground">Description text here.</p>
		<Separator class="my-3" />
		<h3 class="text-h3 font-semibold">Another Section</h3>
		<p class="mt-1 text-small text-muted-foreground">More description text.</p>
		<Separator class="my-3" />
		<div class="flex items-center gap-2 text-small text-muted-foreground">
			<span>Item A</span>
			<Separator orientation="vertical" class="h-4" />
			<span>Item B</span>
			<Separator orientation="vertical" class="h-4" />
			<span>Item C</span>
		</div>
	</div>
</section>

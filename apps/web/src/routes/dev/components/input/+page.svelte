<script lang="ts">
	import { Input, InputLabel, InputHint, InputError } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
</script>

<h1 class="mb-6 text-2xl font-bold">Input</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Default</h2>
	<div class="max-w-sm space-y-4">
		<Input placeholder="Default input" />
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">With Label + Hint</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="demo-name">Full Name</InputLabel>
		<Input id="demo-name" placeholder="Enter your name" />
		<InputHint>This will be shown publicly.</InputHint>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Error State</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="demo-email">Email</InputLabel>
		<Input id="demo-email" type="email" aria-invalid="true" value="not-an-email" />
		<InputError>Please enter a valid email address.</InputError>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Disabled</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="demo-disabled">Disabled</InputLabel>
		<Input id="demo-disabled" disabled value="Cannot edit" />
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">File Input</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="demo-file">Upload File</InputLabel>
		<Input id="demo-file" type="file" />
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Textarea</h2>
	<div class="max-w-sm space-y-4">
		<div class="space-y-1">
			<InputLabel for="demo-textarea">Description</InputLabel>
			<Textarea id="demo-textarea" placeholder="Enter a description..." class="min-h-20" />
			<InputHint>Max 500 characters.</InputHint>
		</div>
		<div class="space-y-1">
			<InputLabel for="demo-textarea-err">With Error</InputLabel>
			<Textarea id="demo-textarea-err" aria-invalid="true" value="Too short" class="min-h-20" />
			<InputError>Description must be at least 20 characters.</InputError>
		</div>
		<div class="space-y-1">
			<InputLabel for="demo-textarea-dis">Disabled</InputLabel>
			<Textarea id="demo-textarea-dis" disabled value="Read-only content" class="min-h-20" />
		</div>
	</div>
</section>

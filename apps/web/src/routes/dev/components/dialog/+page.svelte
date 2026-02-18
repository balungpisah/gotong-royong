<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input, InputLabel } from '$lib/components/ui/input';
</script>

<h1 class="mb-6 text-2xl font-bold">Dialog</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Basic Dialog</h2>
	<Dialog.Root>
		<Dialog.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="outline">Open Dialog</Button>
			{/snippet}
		</Dialog.Trigger>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Basic Dialog</Dialog.Title>
				<Dialog.Description>This is a basic dialog with title and description.</Dialog.Description>
			</Dialog.Header>
			<p class="text-sm text-muted-foreground">Dialog body content goes here.</p>
			<Dialog.Footer>
				<Dialog.Close>
					{#snippet child({ props })}
						<Button {...props} variant="outline">Cancel</Button>
					{/snippet}
				</Dialog.Close>
				<Button>Confirm</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Dialog with Form</h2>
	<Dialog.Root>
		<Dialog.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="track-tuntaskan">Create Task</Button>
			{/snippet}
		</Dialog.Trigger>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Create New Task</Dialog.Title>
				<Dialog.Description>Add a new task to your project.</Dialog.Description>
			</Dialog.Header>
			<div class="space-y-3 py-2">
				<div class="space-y-1">
					<InputLabel for="dialog-title">Title</InputLabel>
					<Input id="dialog-title" placeholder="Task title" />
				</div>
				<div class="space-y-1">
					<InputLabel for="dialog-desc">Description</InputLabel>
					<Input id="dialog-desc" placeholder="Brief description" />
				</div>
			</div>
			<Dialog.Footer>
				<Dialog.Close>
					{#snippet child({ props })}
						<Button {...props} variant="outline">Cancel</Button>
					{/snippet}
				</Dialog.Close>
				<Button variant="track-tuntaskan">Create</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Destructive Dialog</h2>
	<Dialog.Root>
		<Dialog.Trigger>
			{#snippet child({ props })}
				<Button {...props} variant="destructive">Delete Item</Button>
			{/snippet}
		</Dialog.Trigger>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Are you sure?</Dialog.Title>
				<Dialog.Description>
					This action cannot be undone. This will permanently delete the item.
				</Dialog.Description>
			</Dialog.Header>
			<Dialog.Footer>
				<Dialog.Close>
					{#snippet child({ props })}
						<Button {...props} variant="outline">Cancel</Button>
					{/snippet}
				</Dialog.Close>
				<Button variant="destructive">Delete</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</section>

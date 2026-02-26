<script lang="ts">
	import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';

	let fruit = $state('apple');
	let size = $state('medium');
	let disabled = $state('option1');
</script>

<h1 class="mb-6 text-h1 font-extrabold">RadioGroup</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Default (Vertical)</h2>
	<div class="rounded-lg border border-border bg-card p-4">
		<RadioGroup bind:value={fruit}>
			<RadioGroupItem value="apple">Apple</RadioGroupItem>
			<RadioGroupItem value="banana">Banana</RadioGroupItem>
			<RadioGroupItem value="cherry">Cherry</RadioGroupItem>
		</RadioGroup>
		<p class="mt-3 text-small text-muted-foreground">Selected: <strong>{fruit}</strong></p>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Horizontal</h2>
	<div class="rounded-lg border border-border bg-card p-4">
		<RadioGroup bind:value={size} orientation="horizontal">
			<RadioGroupItem value="small">Small</RadioGroupItem>
			<RadioGroupItem value="medium">Medium</RadioGroupItem>
			<RadioGroupItem value="large">Large</RadioGroupItem>
		</RadioGroup>
		<p class="mt-3 text-small text-muted-foreground">Selected: <strong>{size}</strong></p>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Disabled</h2>
	<div class="rounded-lg border border-border bg-card p-4">
		<RadioGroup bind:value={disabled} disabled>
			<RadioGroupItem value="option1">Option 1 (selected)</RadioGroupItem>
			<RadioGroupItem value="option2">Option 2</RadioGroupItem>
			<RadioGroupItem value="option3">Option 3</RadioGroupItem>
		</RadioGroup>
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">With Descriptions</h2>
	<div class="rounded-lg border border-border bg-card p-4">
		<RadioGroup bind:value={fruit} class="gap-4">
			<div class="flex items-start gap-2">
				<RadioGroupItem value="apple" class="mt-0.5" />
				<div>
					<p class="text-body font-medium text-foreground">Apple</p>
					<p class="text-small text-muted-foreground">A crisp, sweet fruit</p>
				</div>
			</div>
			<div class="flex items-start gap-2">
				<RadioGroupItem value="banana" class="mt-0.5" />
				<div>
					<p class="text-body font-medium text-foreground">Banana</p>
					<p class="text-small text-muted-foreground">A tropical yellow fruit</p>
				</div>
			</div>
			<div class="flex items-start gap-2">
				<RadioGroupItem value="cherry" class="mt-0.5" />
				<div>
					<p class="text-body font-medium text-foreground">Cherry</p>
					<p class="text-small text-muted-foreground">A small, red stone fruit</p>
				</div>
			</div>
		</RadioGroup>
	</div>
</section>

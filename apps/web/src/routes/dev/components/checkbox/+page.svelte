<script lang="ts">
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { InputLabel } from '$lib/components/ui/input';

	let checked1 = $state(false);
	let checked2 = $state(true);
	let indeterminate1 = $state(true);
</script>

<h1 class="mb-6 text-h1 font-extrabold">Checkbox</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">States</h2>
	<div class="flex flex-col gap-4">
		<label class="flex items-center gap-3">
			<Checkbox bind:checked={checked1} />
			<span class="text-body">Unchecked — {checked1 ? 'ON' : 'OFF'}</span>
		</label>
		<label class="flex items-center gap-3">
			<Checkbox bind:checked={checked2} />
			<span class="text-body">Checked — {checked2 ? 'ON' : 'OFF'}</span>
		</label>
		<label class="flex items-center gap-3">
			<Checkbox disabled />
			<span class="text-body text-muted-foreground">Disabled</span>
		</label>
		<label class="flex items-center gap-3">
			<Checkbox disabled checked={true} />
			<span class="text-body text-muted-foreground">Disabled (checked)</span>
		</label>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Indeterminate</h2>
	<div class="flex flex-col gap-4">
		<label class="flex items-center gap-3">
			<Checkbox bind:indeterminate={indeterminate1} checked={false} />
			<span class="text-body">Indeterminate state (partial selection)</span>
		</label>
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">With Label</h2>
	<div class="flex flex-col gap-3">
		<label class="flex items-center gap-3">
			<Checkbox id="terms" />
			<InputLabel for="terms">Accept terms and conditions</InputLabel>
		</label>
		<label class="flex items-center gap-3">
			<Checkbox id="newsletter" />
			<InputLabel for="newsletter">Subscribe to newsletter</InputLabel>
		</label>
	</div>
</section>

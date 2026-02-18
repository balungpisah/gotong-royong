<script lang="ts">
	import { Switch } from '$lib/components/ui/switch';
	import { InputLabel } from '$lib/components/ui/input';

	let checked1 = $state(false);
	let checked2 = $state(true);
	let checked3 = $state(false);
</script>

<h1 class="mb-6 text-2xl font-bold">Switch</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">States</h2>
	<div class="flex flex-col gap-4">
		<label class="flex items-center gap-3">
			<Switch bind:checked={checked1} />
			<span class="text-sm">Off state — {checked1 ? 'ON' : 'OFF'}</span>
		</label>
		<label class="flex items-center gap-3">
			<Switch bind:checked={checked2} />
			<span class="text-sm">On state — {checked2 ? 'ON' : 'OFF'}</span>
		</label>
		<label class="flex items-center gap-3">
			<Switch disabled />
			<span class="text-sm text-muted-foreground">Disabled</span>
		</label>
		<label class="flex items-center gap-3">
			<Switch disabled checked={true} />
			<span class="text-sm text-muted-foreground">Disabled (checked)</span>
		</label>
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">With Label</h2>
	<div class="flex items-center gap-3">
		<Switch bind:checked={checked3} id="switch-label" />
		<InputLabel for="switch-label">Enable notifications</InputLabel>
	</div>
</section>

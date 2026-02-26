<script lang="ts">
	import { Select } from '$lib/components/ui/select';
	import { InputLabel } from '$lib/components/ui/input';
</script>

<h1 class="mb-2 text-h1 font-extrabold">Select</h1>
<p class="mb-8 text-body text-muted-foreground">
	Native select wrapper matching the Input component's visual style.
</p>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Default</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="sel-default">Province</InputLabel>
		<Select id="sel-default">
			<option value="" disabled selected>Choose a province...</option>
			<option value="jatim">Jawa Timur</option>
			<option value="jateng">Jawa Tengah</option>
			<option value="jabar">Jawa Barat</option>
			<option value="dki">DKI Jakarta</option>
		</Select>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Error State</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="sel-error">Category</InputLabel>
		<Select id="sel-error" aria-invalid="true">
			<option value="">Please select a category</option>
		</Select>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Disabled</h2>
	<div class="max-w-sm space-y-1">
		<InputLabel for="sel-disabled">Locked</InputLabel>
		<Select id="sel-disabled" disabled>
			<option>Cannot change</option>
		</Select>
	</div>
</section>

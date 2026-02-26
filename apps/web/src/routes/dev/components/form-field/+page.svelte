<script lang="ts">
	import { FormField } from '$lib/components/ui/form-field';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Select } from '$lib/components/ui/select';
</script>

<h1 class="mb-2 text-h1 font-extrabold">FormField</h1>
<p class="mb-8 text-body text-muted-foreground">
	Compound component that composes Label + Input + Hint/Error with consistent spacing.
</p>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">With Input</h2>
	<div class="max-w-sm space-y-4">
		<FormField label="Full Name" hint="This will be shown publicly." id="ff-name">
			<Input id="ff-name" placeholder="Enter your name" />
		</FormField>

		<FormField label="Email" error="Please enter a valid email address." id="ff-email">
			<Input id="ff-email" type="email" aria-invalid="true" value="not-an-email" />
		</FormField>

		<FormField label="Username" hint="Letters and numbers only." required id="ff-user">
			<Input id="ff-user" placeholder="warga123" />
		</FormField>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">With Textarea</h2>
	<div class="max-w-sm space-y-4">
		<FormField label="Description" hint="Max 500 characters." id="ff-desc">
			<Textarea id="ff-desc" placeholder="Describe the issue..." class="min-h-20" />
		</FormField>

		<FormField label="Feedback" error="Feedback must be at least 20 characters." id="ff-fb">
			<Textarea id="ff-fb" aria-invalid="true" value="Too short" class="min-h-20" />
		</FormField>
	</div>
</section>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">With Select</h2>
	<div class="max-w-sm space-y-4">
		<FormField label="Province" hint="Where this issue is happening." id="ff-province">
			<Select id="ff-province">
				<option value="" disabled selected>Choose a province...</option>
				<option value="jatim">Jawa Timur</option>
				<option value="jateng">Jawa Tengah</option>
				<option value="jabar">Jawa Barat</option>
				<option value="dki">DKI Jakarta</option>
			</Select>
		</FormField>

		<FormField label="Priority" required id="ff-priority">
			<Select id="ff-priority">
				<option value="low">Rendah</option>
				<option value="medium">Sedang</option>
				<option value="high">Tinggi</option>
			</Select>
		</FormField>
	</div>
</section>

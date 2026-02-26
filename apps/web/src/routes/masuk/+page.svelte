<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import InputLabel from '$lib/components/ui/input/input-label.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { ActionData } from './$types';
	import { Button } from '$lib/components/ui/button';

	export let form: ActionData;
</script>

<section class="mx-auto flex w-full max-w-md flex-1 items-center py-8">
	<Card class="w-full border-border/70 shadow-sm">
		<CardHeader>
			<CardTitle>{m.auth_login_title()}</CardTitle>
			<CardDescription>{m.auth_login_description()}</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4 text-body">
			<p class="text-muted-foreground">{m.auth_login_hint()}</p>
			{#if form?.error}
				<p class="text-body text-destructive">{form.error}</p>
			{/if}
			<form method="POST" class="space-y-3">
				<div class="flex flex-col gap-1.5">
					<InputLabel for="email">Email</InputLabel>
					<Input
						id="email"
						name="email"
						type="email"
						autocomplete="email"
						required
					/>
				</div>
				<div class="flex flex-col gap-1.5">
					<InputLabel for="pass">Password</InputLabel>
					<Input
						id="pass"
						name="pass"
						type="password"
						autocomplete="current-password"
						required
					/>
				</div>
				<Button
					type="submit"
					variant="default"
					class="w-full"
				>
					{m.auth_login_title()}
				</Button>
			</form>
		</CardContent>
	</Card>
</section>

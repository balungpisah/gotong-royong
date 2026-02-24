<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { m } from '$lib/paraglide/messages';
	import type { ActionData } from './$types';

	export let form: ActionData;
</script>

<section class="mx-auto flex w-full max-w-md flex-1 items-center py-8">
	<Card class="w-full border-border/70 shadow-sm">
		<CardHeader>
			<CardTitle>{m.auth_login_title()}</CardTitle>
			<CardDescription>{m.auth_login_description()}</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4 text-sm">
			<p class="text-muted-foreground">{m.auth_login_hint()}</p>
			{#if form?.error}
				<p class="text-sm text-destructive">{form.error}</p>
			{/if}
			<form method="POST" class="space-y-3">
				<div class="space-y-1">
					<label for="email" class="text-sm font-medium">Email</label>
					<input
						id="email"
						name="email"
						type="email"
						autocomplete="email"
						required
						class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
					/>
				</div>
				<div class="space-y-1">
					<label for="pass" class="text-sm font-medium">Password</label>
					<input
						id="pass"
						name="pass"
						type="password"
						autocomplete="current-password"
						required
						class="w-full rounded-md border border-border bg-background px-3 py-2 text-sm"
					/>
				</div>
				<button
					type="submit"
					class="w-full rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground"
				>
					{m.auth_login_title()}
				</button>
			</form>
		</CardContent>
	</Card>
</section>

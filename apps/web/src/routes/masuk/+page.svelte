<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { m } from '$lib/paraglide/messages';
</script>

<section class="mx-auto flex w-full max-w-md flex-1 items-center py-8">
	<Card class="w-full border-border/70 shadow-sm">
		<CardHeader>
			<CardTitle>{m.auth_login_title()}</CardTitle>
			<CardDescription>{m.auth_login_description()}</CardDescription>
		</CardHeader>
		<CardContent class="text-sm text-muted-foreground">
			<p>{m.auth_login_hint()}</p>
		</CardContent>
	</Card>
</section>

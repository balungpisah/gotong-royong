<script lang="ts">
	import { resolve } from '$app/paths';
	import { Button, type ButtonVariant } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';

	type Action = {
		href: string;
		label: string;
		variant?: ButtonVariant;
	};

	let {
		badge,
		title,
		description,
		actions = []
	} = $props<{
		badge: string;
		title: string;
		description: string;
		actions?: Action[];
	}>();
</script>

<Card class="border-border/70 shadow-sm">
	<CardHeader class="space-y-3">
		<p
			class="inline-flex w-fit rounded-full bg-accent px-3 py-1 text-[11px] font-semibold tracking-wide uppercase text-accent-foreground"
		>
			{badge}
		</p>
		<CardTitle class="text-xl md:text-2xl">{title}</CardTitle>
		<CardDescription class="text-sm md:text-base">{description}</CardDescription>
	</CardHeader>
	{#if actions.length > 0}
		<CardContent>
			<CardFooter class="flex flex-wrap gap-3 px-0 pt-1 pb-0">
				{#each actions as action (action.href)}
					<Button href={resolve(action.href)} variant={action.variant ?? 'default'}>
						{action.label}
					</Button>
				{/each}
			</CardFooter>
		</CardContent>
	{/if}
</Card>

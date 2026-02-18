<script lang="ts">
	import type { Snippet } from 'svelte';

	const { children }: { children: Snippet } = $props();

	const components = [
		{ name: 'Button', href: '/dev/components/button' },
		{ name: 'Badge', href: '/dev/components/badge' },
		{ name: 'Input', href: '/dev/components/input' },
		{ name: 'Avatar', href: '/dev/components/avatar' },
		{ name: 'Pill', href: '/dev/components/pill' },
		{ name: 'Status Indicator', href: '/dev/components/status-indicator' },
		{ name: 'Progress', href: '/dev/components/progress' },
		{ name: 'Switch', href: '/dev/components/switch' },
		{ name: 'Tooltip', href: '/dev/components/tooltip' },
		{ name: 'Separator', href: '/dev/components/separator' },
		{ name: 'Card', href: '/dev/components/card' },
		{ name: 'Dialog', href: '/dev/components/dialog' }
	];

	const renderers = [
		{ name: 'Blocks', href: '/dev/components/blocks' },
		{ name: 'Chat', href: '/dev/components/chat' },
		{ name: 'Plan', href: '/dev/components/plan' },
		{ name: 'Diff Card', href: '/dev/components/diff-card' }
	];
</script>

<div class="flex min-h-screen">
	<aside
		class="sticky top-0 hidden h-screen w-56 shrink-0 overflow-y-auto border-r border-border bg-card p-4 md:block"
	>
		<a href="/dev/components" class="mb-4 block text-lg font-bold text-foreground">
			Component Gallery
		</a>
		<nav class="flex flex-col gap-1">
			{#each components as comp (comp.href)}
				<a
					href={comp.href}
					class="rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
				>
					{comp.name}
				</a>
			{/each}
			<div class="mt-4 border-t border-border pt-4">
				<p class="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
					Renderers
				</p>
				{#each renderers as comp (comp.href)}
					<a
						href={comp.href}
						class="rounded-md px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
					>
						{comp.name}
					</a>
				{/each}
			</div>
		</nav>
	</aside>

	<main class="flex-1 p-6 md:p-8">
		{@render children()}
	</main>
</div>

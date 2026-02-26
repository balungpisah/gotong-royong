<script lang="ts">
	import type { ListBlock, ListItem } from '$lib/types';
	import { cn } from '$lib/utils';
	import { SourceBadge } from '$lib/components/ui/source-badge';
	import { Badge } from '$lib/components/ui/badge';
	import Circle from '@lucide/svelte/icons/circle';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleX from '@lucide/svelte/icons/circle-x';
	import CircleMinus from '@lucide/svelte/icons/circle-minus';

	let { block }: { block: ListBlock } = $props();

	const statusIcons = {
		open: Circle,
		completed: CircleCheck,
		blocked: CircleX,
		skipped: CircleMinus
	};

	const statusColors = {
		open: 'text-kayu',
		completed: 'text-berhasil',
		blocked: 'text-bahaya',
		skipped: 'text-batu'
	};
</script>

{#if block.title}
	<h3 class="mb-3 text-body font-bold text-foreground">{block.title}</h3>
{/if}

{#if block.display === 'checklist'}
	<div class="flex flex-col gap-2" data-slot="list-block">
		{#each block.items as item (item.id)}
			{@const StatusIcon = statusIcons[item.status]}
			<div class="flex items-start gap-2">
				<StatusIcon class={cn('mt-0.5 size-4 shrink-0', statusColors[item.status])} />
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<span class={cn('text-body', item.status === 'completed' && 'line-through text-muted-foreground')}>
							{item.label}
						</span>
						<SourceBadge source={item.source} />
					</div>
					{#if item.children?.length}
						<div class="ml-4 mt-2 flex flex-col gap-2">
							{#each item.children as child (child.id)}
								{@const ChildIcon = statusIcons[child.status]}
								<div class="flex items-center gap-2">
									<ChildIcon class={cn('size-3.5 shrink-0', statusColors[child.status])} />
									<span class={cn('text-small', child.status === 'completed' && 'line-through text-muted-foreground')}>
										{child.label}
									</span>
									<SourceBadge source={child.source} />
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>
{:else if block.display === 'table'}
	<div class="overflow-x-auto rounded-lg border border-border" data-slot="list-block">
		<table class="w-full text-body">
			<thead>
				<tr class="border-b border-border bg-kapas">
					<th class="px-3 py-2 text-left font-semibold text-foreground">Item</th>
					<th class="px-3 py-2 text-left font-semibold text-foreground">Status</th>
					<th class="px-3 py-2 text-left font-semibold text-foreground">Sumber</th>
				</tr>
			</thead>
			<tbody>
				{#each block.items as item (item.id)}
					<tr class="border-b border-border/50 last:border-0">
						<td class="px-3 py-2">{item.label}</td>
						<td class="px-3 py-2">
							<Badge variant={item.status === 'completed' ? 'success' : item.status === 'blocked' ? 'danger' : 'secondary'}>
								{item.status}
							</Badge>
						</td>
						<td class="px-3 py-2"><SourceBadge source={item.source} /></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else if block.display === 'timeline'}
	<div class="relative ml-3 border-l-2 border-batu pl-6" data-slot="list-block">
		{#each block.items as item (item.id)}
			{@const StatusIcon = statusIcons[item.status]}
			<div class={cn('relative pb-6 last:pb-0')}>
				<div class={cn('absolute -left-[31px] flex size-5 items-center justify-center rounded-full border-2 border-background',
					item.status === 'completed' ? 'bg-berhasil' : item.status === 'open' ? 'bg-api' : item.status === 'blocked' ? 'bg-bahaya' : 'bg-batu')}>
					<StatusIcon class="size-3 text-white" />
				</div>
				<div>
					<p class="text-body font-medium">{item.label}</p>
					<div class="mt-1 flex items-center gap-2">
						<SourceBadge source={item.source} />
					</div>
				</div>
			</div>
		{/each}
	</div>
{:else if block.display === 'gallery'}
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-3" data-slot="list-block">
		{#each block.items as item (item.id)}
			<div class="rounded-lg border border-border bg-card p-3">
				<p class="text-body font-medium">{item.label}</p>
				<div class="mt-2 flex items-center gap-2">
					<Badge variant={item.status === 'completed' ? 'success' : 'secondary'} class="text-small">
						{item.status}
					</Badge>
					<SourceBadge source={item.source} />
				</div>
			</div>
		{/each}
	</div>
{/if}

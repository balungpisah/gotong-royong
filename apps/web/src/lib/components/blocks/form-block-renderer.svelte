<script lang="ts">
	import type { FormBlock } from '$lib/types';
	import { cn } from '$lib/utils';
	import { SourceBadge, ProtectedBadge } from '$lib/components/ui/source-badge';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Switch } from '$lib/components/ui/switch';

	let { block }: { block: FormBlock } = $props();
</script>

{#if block.title}
	<h3 class="mb-4 text-sm font-bold text-foreground">{block.title}</h3>
{/if}

<div class="flex flex-col gap-4" data-slot="form-block">
	{#each block.fields as field (field.id)}
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center gap-2">
				<label for={field.id} class="text-[11px] font-bold uppercase tracking-wide text-foreground">
					{field.label}
				</label>
				<SourceBadge source={field.source} />
				{#if field.protected}
					<ProtectedBadge />
				{/if}
			</div>

			{#if field.field_type === 'text' || field.field_type === 'number' || field.field_type === 'date'}
				<Input
					id={field.id}
					type={field.field_type}
					value={field.value != null ? String(field.value) : ''}
					placeholder={field.placeholder}
					disabled={field.protected}
					class={cn(field.protected && 'opacity-60')}
				/>
			{:else if field.field_type === 'textarea'}
				<Textarea
					id={field.id}
					value={field.value != null ? String(field.value) : ''}
					placeholder={field.placeholder}
					disabled={field.protected}
					class={cn('min-h-[80px]', field.protected && 'opacity-60')}
				/>
			{:else if field.field_type === 'select'}
				<select
					id={field.id}
					class={cn(
						'flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors',
						'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring',
						'disabled:cursor-not-allowed disabled:opacity-50',
						field.protected && 'opacity-60'
					)}
					disabled={field.protected}
					value={field.value != null ? String(field.value) : ''}
				>
					{#if field.options}
						{#each field.options as opt}
							<option value={opt.value}>{opt.label}</option>
						{/each}
					{/if}
				</select>
			{:else if field.field_type === 'toggle'}
				<div class="flex items-center gap-2">
					<Switch
						id={field.id}
						checked={Boolean(field.value)}
						disabled={field.protected}
					/>
					<span class="text-sm text-muted-foreground">
						{field.value ? 'Aktif' : 'Nonaktif'}
					</span>
				</div>
			{:else if field.field_type === 'file'}
				<input
					id={field.id}
					type="file"
					disabled={field.protected}
					class={cn(
						'flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm',
						'file:border-0 file:bg-transparent file:text-sm file:font-medium',
						field.protected && 'opacity-60'
					)}
				/>
			{/if}
		</div>
	{/each}
</div>

<script lang="ts">
	import type { FormBlock } from '$lib/types';
	import { cn } from '$lib/utils';
	import { SourceBadge, ProtectedBadge } from '$lib/components/ui/source-badge';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Select } from '$lib/components/ui/select';
	import { Switch } from '$lib/components/ui/switch';

	let { block }: { block: FormBlock } = $props();
</script>

{#if block.title}
	<h3 class="mb-4 text-body font-bold text-foreground">{block.title}</h3>
{/if}

<div class="flex flex-col gap-4" data-slot="form-block">
	{#each block.fields as field (field.id)}
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center gap-2">
				<label
					for={field.id}
					class="text-caption font-bold uppercase tracking-wide text-foreground"
				>
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
				<Select
					id={field.id}
					disabled={field.protected}
					value={field.value != null ? String(field.value) : ''}
					class={cn(field.protected && 'opacity-60')}
				>
					{#if field.options}
						{#each field.options as opt (opt.value)}
							<option value={opt.value}>{opt.label}</option>
						{/each}
					{/if}
				</Select>
			{:else if field.field_type === 'toggle'}
				<div class="flex items-center gap-2">
					<Switch id={field.id} checked={Boolean(field.value)} disabled={field.protected} />
					<span class="text-body text-muted-foreground">
						{field.value ? 'Aktif' : 'Nonaktif'}
					</span>
				</div>
			{:else if field.field_type === 'file'}
				<Input
					id={field.id}
					type="file"
					disabled={field.protected}
					class={cn(field.protected && 'opacity-60')}
				/>
			{/if}
		</div>
	{/each}
</div>

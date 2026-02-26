<script lang="ts">
	import type { GroupCreateInput, GroupEntityType, GroupJoinPolicy } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import InputLabel from '$lib/components/ui/input/input-label.svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import { Button } from '$lib/components/ui/button';
	import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';

	interface Props {
		oncreated?: (groupId: string) => void;
	}

	let { oncreated }: Props = $props();

	const store = getGroupStore();

	let name = $state('');
	let description = $state('');
	let entityType = $state<GroupEntityType>('kelompok');
	let joinPolicy = $state<GroupJoinPolicy>('terbuka');

	const canSubmit = $derived(name.trim().length >= 3 && description.trim().length >= 10);

	async function submit() {
		if (!canSubmit) return;
		const input: GroupCreateInput = {
			name: name.trim(),
			description: description.trim(),
			entity_type: entityType,
			join_policy: joinPolicy
		};
		const groupId = await store.createGroup(input);
		if (groupId) {
			oncreated?.(groupId);
			name = '';
			description = '';
			entityType = 'kelompok';
			joinPolicy = 'terbuka';
		}
	}
</script>

<form
	class="rounded-xl border border-border/50 bg-card p-4"
	onsubmit={(e) => {
		e.preventDefault();
		submit();
	}}
>
	<div class="flex items-start justify-between gap-3">
		<div>
			<h3 class="text-body font-bold text-foreground">{m.group_create_title()}</h3>
			<p class="mt-0.5 text-small text-muted-foreground/80">{m.group_create_subtitle()}</p>
		</div>
		<div class="flex size-9 items-center justify-center rounded-xl bg-primary/10 text-primary">
			<PlusIcon class="size-5" />
		</div>
	</div>

	<div class="mt-4 space-y-3">
		<div class="flex flex-col gap-1.5">
			<InputLabel>{m.group_create_name_label()}</InputLabel>
			<Input placeholder={m.group_create_name_placeholder()} bind:value={name} />
		</div>

		<div class="flex flex-col gap-1.5">
			<InputLabel>{m.group_create_description_label()}</InputLabel>
			<Textarea
				rows={3}
				class="resize-none"
				placeholder={m.group_create_description_placeholder()}
				bind:value={description}
			/>
		</div>

		<div class="grid gap-3 sm:grid-cols-2">
			<fieldset class="rounded-lg border border-border/50 p-3">
				<legend class="px-1 text-small font-semibold text-foreground">
					{m.group_create_entity_type_label()}
				</legend>
				<RadioGroup bind:value={entityType} class="mt-2 gap-2">
					<RadioGroupItem value="kelompok">{m.entity_type_kelompok()}</RadioGroupItem>
					<RadioGroupItem value="lembaga">{m.entity_type_lembaga()}</RadioGroupItem>
				</RadioGroup>
			</fieldset>

			<fieldset class="rounded-lg border border-border/50 p-3">
				<legend class="px-1 text-small font-semibold text-foreground">
					{m.group_create_join_policy_label()}
				</legend>
				<RadioGroup bind:value={joinPolicy} class="mt-2 gap-2">
					<RadioGroupItem value="terbuka">{m.group_policy_terbuka()}</RadioGroupItem>
					<RadioGroupItem value="persetujuan">{m.group_policy_persetujuan()}</RadioGroupItem>
					<RadioGroupItem value="undangan">{m.group_policy_undangan()}</RadioGroupItem>
				</RadioGroup>
			</fieldset>
		</div>

		{#if store.errors.create}
			<p class="text-small text-bahaya">{store.errors.create}</p>
		{/if}

		<div class="flex items-center justify-end gap-2">
			<Button type="submit" variant="default" disabled={!canSubmit || store.creating}>
				{store.creating ? m.group_create_creating() : m.group_create_submit()}
			</Button>
		</div>
	</div>
</form>

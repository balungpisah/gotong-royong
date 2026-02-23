<script lang="ts">
	import type { GroupCreateInput, GroupEntityType, GroupJoinPolicy } from '$lib/types';
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import PlusIcon from '@lucide/svelte/icons/plus';

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
			<h3 class="text-sm font-bold text-foreground">{m.group_create_title()}</h3>
			<p class="mt-0.5 text-xs text-muted-foreground/80">{m.group_create_subtitle()}</p>
		</div>
		<div class="flex size-9 items-center justify-center rounded-xl bg-primary/10 text-primary">
			<PlusIcon class="size-5" />
		</div>
	</div>

	<div class="mt-4 space-y-3">
		<label class="block">
			<span class="text-xs font-semibold text-foreground">{m.group_create_name_label()}</span>
			<input
				class="mt-1 w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
				placeholder={m.group_create_name_placeholder()}
				bind:value={name}
			/>
		</label>

		<label class="block">
			<span class="text-xs font-semibold text-foreground">{m.group_create_description_label()}</span>
			<textarea
				class="mt-1 w-full resize-none rounded-lg border border-border/60 bg-background px-3 py-2 text-sm outline-none focus:border-primary/60"
				rows={3}
				placeholder={m.group_create_description_placeholder()}
				bind:value={description}
			></textarea>
		</label>

		<div class="grid gap-3 sm:grid-cols-2">
			<fieldset class="rounded-lg border border-border/50 p-3">
				<legend class="px-1 text-xs font-semibold text-foreground">
					{m.group_create_entity_type_label()}
				</legend>
				<div class="mt-2 space-y-2 text-sm">
					<label class="flex items-center gap-2">
						<input type="radio" name="entityType" value="kelompok" bind:group={entityType} />
						<span>{m.entity_type_kelompok()}</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="radio" name="entityType" value="lembaga" bind:group={entityType} />
						<span>{m.entity_type_lembaga()}</span>
					</label>
				</div>
			</fieldset>

			<fieldset class="rounded-lg border border-border/50 p-3">
				<legend class="px-1 text-xs font-semibold text-foreground">
					{m.group_create_join_policy_label()}
				</legend>
				<div class="mt-2 space-y-2 text-sm">
					<label class="flex items-center gap-2">
						<input type="radio" name="joinPolicy" value="terbuka" bind:group={joinPolicy} />
						<span>{m.group_policy_terbuka()}</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="radio" name="joinPolicy" value="persetujuan" bind:group={joinPolicy} />
						<span>{m.group_policy_persetujuan()}</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="radio" name="joinPolicy" value="undangan" bind:group={joinPolicy} />
						<span>{m.group_policy_undangan()}</span>
					</label>
				</div>
			</fieldset>
		</div>

		{#if store.errors.create}
			<p class="text-xs text-bahaya">{store.errors.create}</p>
		{/if}

		<div class="flex items-center justify-end gap-2">
			<button
				type="submit"
				disabled={!canSubmit || store.creating}
				class="inline-flex items-center justify-center rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition disabled:opacity-50"
			>
				{store.creating ? m.group_create_creating() : m.group_create_submit()}
			</button>
		</div>
	</div>
</form>

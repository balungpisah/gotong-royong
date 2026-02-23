<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import { GroupCreateForm, GroupListView } from '$lib/components/kelompok';

	const store = getGroupStore();

	let showCreate = $state(false);

	$effect(() => {
		untrack(() => {
			store.loadMyGroups();
			store.loadGroups();
		});
	});

	const discoverable = $derived(store.groups);
	const myGroups = $derived(store.myGroups);

	const onJoin = async (groupId: string) => {
		await store.joinGroup(groupId);
	};
	const onRequestJoin = (groupId: string) => {
		goto(`/komunitas/kelompok/${groupId}`);
	};
</script>

<div class="mx-auto w-full max-w-3xl space-y-6">
	<header class="rounded-xl border border-border/30 bg-card px-4 py-4">
		<div class="flex items-start justify-between gap-3">
			<div>
				<h1 class="text-base font-bold text-foreground">{m.group_page_title()}</h1>
				<p class="mt-1 text-xs text-muted-foreground/80">{m.group_page_subtitle()}</p>
			</div>
			<button
				type="button"
				class="rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
				onclick={() => (showCreate = !showCreate)}
			>
				{m.group_create_button()}
			</button>
		</div>
	</header>

	{#if showCreate}
		<GroupCreateForm
			oncreated={() => {
				showCreate = false;
				store.loadMyGroups();
				store.loadGroups();
			}}
		/>
	{/if}

	{#if myGroups.length > 0}
		<GroupListView groups={myGroups} title={m.group_section_my()} {onJoin} {onRequestJoin} />
	{:else}
		<section class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-4">
			<h2 class="text-sm font-bold text-foreground">{m.group_section_my()}</h2>
			<p class="mt-1 text-xs text-muted-foreground/80">{m.group_empty_my()}</p>
		</section>
	{/if}

	{#if discoverable.length > 0}
		<GroupListView groups={discoverable} title={m.group_section_discover()} {onJoin} {onRequestJoin} />
	{:else}
		<section class="rounded-xl border border-dashed border-border/40 bg-muted/10 p-4">
			<h2 class="text-sm font-bold text-foreground">{m.group_section_discover()}</h2>
			<p class="mt-1 text-xs text-muted-foreground/80">{m.group_empty_discover()}</p>
		</section>
	{/if}
</div>

<script lang="ts">
	import { page } from '$app/state';
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getGroupStore } from '$lib/stores';
	import { GroupDetailPanel } from '$lib/components/kelompok';

	const store = getGroupStore();

	$effect(() => {
		const groupId = page.params.id;
		if (groupId) {
			untrack(() => store.loadDetail(groupId));
		}
	});
</script>

<div class="mx-auto w-full max-w-3xl space-y-4">
	<a
		href="/komunitas/kelompok"
		class="text-xs font-semibold text-primary hover:underline"
	>
		{m.group_back_to_list()}
	</a>
	<GroupDetailPanel />
</div>

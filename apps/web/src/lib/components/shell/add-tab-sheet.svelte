<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getNavigationStore } from '$lib/stores';
	import { resolveTabIcon, iconNameForTag } from '$lib/utils';
	import {
		Sheet,
		SheetContent,
		SheetHeader,
		SheetTitle,
		SheetDescription
	} from '$lib/components/ui/sheet';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import Check from '@lucide/svelte/icons/check';
	import Plus from '@lucide/svelte/icons/plus';

	const navStore = getNavigationStore();

	function isTagAdded(tag: string): boolean {
		return navStore.tabs.some((t) => t.tag === tag);
	}

	function handleAdd(suggestion: { tag: string; label: string }) {
		navStore.addTab({
			label: suggestion.label,
			iconName: iconNameForTag(suggestion.tag),
			tag: suggestion.tag
		});
	}
</script>

<Sheet bind:open={navStore.addPanelOpen}>
	<SheetContent side="bottom" class="max-h-[70vh]">
		<SheetHeader>
			<SheetTitle>{m.shell_add_tab_title()}</SheetTitle>
			<SheetDescription>{m.shell_add_tab_description()}</SheetDescription>
		</SheetHeader>

		<div class="mt-4 space-y-2 overflow-y-auto pb-6">
			{#if navStore.suggestions.length > 0}
				<p class="mb-3 text-small font-semibold uppercase tracking-wider text-muted-foreground">
					{m.shell_add_tab_suggested()}
				</p>

				<div class="grid gap-2">
					{#each navStore.suggestions as suggestion (suggestion.tag)}
						{@const added = isTagAdded(suggestion.tag)}
						{@const TagIcon = resolveTabIcon(iconNameForTag(suggestion.tag))}
						<div
							class="flex items-center gap-3 rounded-lg border border-border/60 p-3 transition"
							class:opacity-50={added}
						>
							<div class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted">
								<TagIcon class="size-4 text-muted-foreground" />
							</div>

							<div class="min-w-0 flex-1">
								<p class="text-body font-medium capitalize text-foreground">
									{suggestion.label}
								</p>
								<p class="text-small text-muted-foreground">
									{suggestion.witnessCount}
									{m.tag_page_witness_count()}
								</p>
							</div>

							{#if added}
								<Badge variant="success">
									<Check class="size-3" />
								</Badge>
							{:else}
								<Button variant="outline" size="sm" onclick={() => handleAdd(suggestion)}>
									<Plus class="size-3" />
									{m.shell_nav_add_tab()}
								</Button>
							{/if}
						</div>
					{/each}
				</div>
			{:else}
				<p class="py-8 text-center text-body text-muted-foreground">
					{m.shell_add_tab_all_tags()}
				</p>
			{/if}
		</div>
	</SheetContent>
</Sheet>

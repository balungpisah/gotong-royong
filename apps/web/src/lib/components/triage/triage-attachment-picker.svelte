<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Tip from '$lib/components/ui/tip.svelte';
	import ImageIcon from '@lucide/svelte/icons/image';
	import Video from '@lucide/svelte/icons/video';
	import Mic from '@lucide/svelte/icons/mic';

	interface Props {
		onFilesSelected: (files: File[]) => void;
		disabled?: boolean;
	}

	let { onFilesSelected, disabled = false }: Props = $props();

	let imageInput = $state<HTMLInputElement | null>(null);
	let videoInput = $state<HTMLInputElement | null>(null);
	let audioInput = $state<HTMLInputElement | null>(null);

	function handleChange(e: Event) {
		const input = e.target as HTMLInputElement;
		if (!input.files?.length) return;
		onFilesSelected(Array.from(input.files));
		input.value = '';
	}
</script>

<div class="flex items-center gap-0.5">
	<Tip text={m.triage_attach_photo()}>
		<button
			type="button"
			{disabled}
			onclick={() => imageInput?.click()}
			class="flex size-8 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
			aria-label={m.triage_attach_photo()}
		>
			<ImageIcon class="size-4" />
		</button>
	</Tip>

	<Tip text={m.triage_attach_video()}>
		<button
			type="button"
			{disabled}
			onclick={() => videoInput?.click()}
			class="flex size-8 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
			aria-label={m.triage_attach_video()}
		>
			<Video class="size-4" />
		</button>
	</Tip>

	<Tip text={m.triage_attach_audio()}>
		<button
			type="button"
			{disabled}
			onclick={() => audioInput?.click()}
			class="flex size-8 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
			aria-label={m.triage_attach_audio()}
		>
			<Mic class="size-4" />
		</button>
	</Tip>

	<input bind:this={imageInput} type="file" accept="image/*" multiple class="hidden" onchange={handleChange} />
	<input bind:this={videoInput} type="file" accept="video/*" class="hidden" onchange={handleChange} />
	<input bind:this={audioInput} type="file" accept="audio/*" class="hidden" onchange={handleChange} />
</div>

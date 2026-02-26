<script lang="ts">
	import type { ChatMessage, SystemMessage } from '$lib/types';
	import ChatBubble from './chat-bubble.svelte';
	import CardEnvelope from './card-envelope.svelte';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import { safeSlide as slide } from '$lib/utils/safe-slide';
	import { untrack } from 'svelte';

	let { messages }: { messages: ChatMessage[] } = $props();

	// Onboarding card — expanded by default, auto-collapses once when first
	// system message arrives, but manual toggle always works after that.
	const hasSystemMessage = $derived(messages.some((m) => m.type === 'system'));
	let onboardingExpanded = $state(!untrack(() => hasSystemMessage));
	let autoCollapsed = $state(false);

	// Auto-collapse once when the first system message arrives
	$effect(() => {
		if (hasSystemMessage && !autoCollapsed) {
			onboardingExpanded = false;
			autoCollapsed = true;
		}
	});

	function toggleOnboarding() {
		onboardingExpanded = !onboardingExpanded;
	}

	// Find the index of the last block message (non-user) to expand it by default
	const lastCardIndex = $derived.by(() => {
		for (let i = messages.length - 1; i >= 0; i--) {
			if (messages[i].type !== 'user') return i;
		}
		return -1;
	});

	/** Phase blocks = major phase actions (vote, diff, system phase events). Minor = everything else. */
	function getDotVariant(message: ChatMessage): 'phase' | 'minor' {
		switch (message.type) {
			case 'vote_card':
			case 'diff_card':
				return 'phase';
			case 'system': {
				const sub = (message as SystemMessage).subtype;
				if (sub === 'phase_completed' || sub === 'checkpoint_completed' || sub === 'phase_activated')
					return 'phase';
				return 'minor';
			}
			default:
				return 'minor';
		}
	}
</script>

<!--
  The vertical line runs through the dot center (4px from left).
  Dots live inside CardEnvelope as the first flex child.
  The line is absolutely positioned so it passes behind dots seamlessly.
-->
<div class="relative" data-slot="chat-thread">
	<!-- Vertical timeline line — runs full height, positioned at dot center (3.5px ≈ center of 8px dot) -->
	<div class="absolute top-0 bottom-0 left-[3.5px] w-px bg-border/50"></div>

	<div class="relative flex flex-col">
		<!-- Onboarding card — teaches zone purpose + expand/collapse mechanic -->
		<div class="relative pb-3">
			<button
				type="button"
				onclick={toggleOnboarding}
				class="group flex w-full items-center gap-1.5 py-0.5 text-left transition"
			>
				<div class="relative z-10 size-2 shrink-0 rounded-full bg-primary/40"></div>
				<MessageCircle class="size-3 shrink-0 text-primary/50" />
				<span class="min-w-0 flex-1 truncate text-xs italic text-muted-foreground/70 group-hover:text-muted-foreground transition-colors">
					Ruang Interaksi
				</span>
				<ChevronDown
					class="size-3 shrink-0 text-muted-foreground/40 transition-transform duration-150 {onboardingExpanded ? 'rotate-180' : ''}"
				/>
			</button>

			{#if onboardingExpanded}
				<div class="pl-5 pt-1" transition:slide={{ duration: 150 }}>
					<div class="space-y-2.5 rounded-lg border border-border/40 bg-card/60 px-3 py-2.5 text-xs leading-relaxed text-muted-foreground">
						<p class="font-medium text-foreground/70">👋 Selamat datang di ruang diskusi!</p>
						<p>Ini tempat kamu dan anggota lain berdiskusi, berbagi informasi, dan bekerja sama menuntaskan saksi ini tahap demi tahap.</p>

						<div class="space-y-1 rounded-md bg-primary/5 px-2.5 py-2">
							<p class="font-medium text-foreground/70">✦ Evaluasi fase</p>
							<p>Status fase diperbarui berdasarkan event backend yang masuk ke ruang interaksi ini.</p>
							<p>Tombol evaluasi manual akan ditampilkan kembali setelah kontrak backend Stempel tersedia.</p>
						</div>

						<div class="space-y-1">
							<p class="font-medium text-foreground/60">Cara membaca timeline:</p>
							<ul class="space-y-0.5 pl-3">
								<li><span class="text-muted-foreground/50">●</span> <strong>Titik di kiri</strong> — kartu aktivitas (saran AI, bukti, voting, dana). Ketuk untuk buka/tutup detail.</li>
								<li><span class="text-muted-foreground/50">—</span> <strong>Garis di tengah</strong> — penanda fase (fase dimulai, selesai, hasil voting). Pembatas antar tahap.</li>
							</ul>
						</div>
						<p>Lihat daftar fase di panel atas — ketuk untuk lihat langkah yang perlu diselesaikan bersama di setiap fase.</p>
						<p class="text-muted-foreground/50">Kartu ini akan otomatis tertutup saat aktivitas dimulai. Ketuk "Ruang Interaksi" untuk buka lagi.</p>
					</div>
				</div>
			{/if}
		</div>

		{#each messages as message, i (message.message_id)}
			<div class="relative pb-3">
				{#if message.type === 'user'}
					<div class="pl-5">
						<ChatBubble {message} />
					</div>
				{:else}
					<CardEnvelope {message} defaultExpanded={i === lastCardIndex} dotVariant={getDotVariant(message)} />
				{/if}
			</div>
		{/each}
	</div>
</div>

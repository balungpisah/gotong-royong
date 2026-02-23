<script lang="ts">
	/**
	 * TrajectoryGrid — onboarding wallpaper for the empty triage chat.
	 *
	 * Shows 8 broad user-intent categories as tappable cards. These map
	 * to the 11 internal TrajectoryTypes, but users see only what they
	 * can self-identify at conversation start:
	 *
	 *   Masalah  → aksi + advokasi  (AI decides which during triage)
	 *   Musyawarah → mufakat + mediasi  (proposal vs dispute discovered later)
	 *   Pantau   → pantau
	 *   Catat    → data + vault  (public vs sealed decided during triage)
	 *   Bantuan  → bantuan
	 *   Rayakan  → pencapaian
	 *   Siaga    → siaga
	 *   Program  → program
	 *
	 * Clicking sends a broad primer message; the AI probes further to
	 * determine the exact trajectory. Disappears once first message is sent.
	 *
	 * @see docs/design/specs/ai-spec/04b-trajectory-map.md
	 */

	import DynamicIcon from '$lib/components/ui/dynamic-icon.svelte';
	import { trajectoryColors, type TrajectoryColorKey } from '$lib/utils/trajectory-colors';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		/** Called when user taps an intent — sends the primer message. */
		onSelect: (primer: string) => void;
	}

	let { onSelect }: Props = $props();

	// ── User-facing intent catalogue ────────────────────────────────
	// `color` picks the primary trajectory (or 'kelola') for visual treatment.
	// `primer` is broad enough that AI can route to any sub-trajectory.

	interface IntentItem {
		/** Primary trajectory type — used for color & icon only. */
		color: TrajectoryColorKey;
		name: string;
		icon: string;
		desc: string;
		primer: string;
		/** Whether this item spans the full grid width. */
		fullWidth?: boolean;
	}

	const INTENTS: IntentItem[] = $derived([
		{
			color: 'aksi',
			name: m.triage_intent_masalah_name(),
			icon: 'construction',
			desc: m.triage_intent_masalah_desc(),
			primer: m.triage_intent_masalah_primer()
		},
		{
			color: 'mufakat',
			name: m.triage_intent_musyawarah_name(),
			icon: 'users',
			desc: m.triage_intent_musyawarah_desc(),
			primer: m.triage_intent_musyawarah_primer()
		},
		{
			color: 'pantau',
			name: m.triage_intent_pantau_name(),
			icon: 'eye',
			desc: m.triage_intent_pantau_desc(),
			primer: m.triage_intent_pantau_primer()
		},
		{
			color: 'data',
			name: m.triage_intent_catat_name(),
			icon: 'file-text',
			desc: m.triage_intent_catat_desc(),
			primer: m.triage_intent_catat_primer()
		},
		{
			color: 'bantuan',
			name: m.triage_intent_bantuan_name(),
			icon: 'heart',
			desc: m.triage_intent_bantuan_desc(),
			primer: m.triage_intent_bantuan_primer()
		},
		{
			color: 'pencapaian',
			name: m.triage_intent_rayakan_name(),
			icon: 'trophy',
			desc: m.triage_intent_rayakan_desc(),
			primer: m.triage_intent_rayakan_primer()
		},
		{
			color: 'siaga',
			name: m.triage_intent_siaga_name(),
			icon: 'siren',
			desc: m.triage_intent_siaga_desc(),
			primer: m.triage_intent_siaga_primer()
		},
		{
			color: 'program',
			name: m.triage_intent_program_name(),
			icon: 'calendar',
			desc: m.triage_intent_program_desc(),
			primer: m.triage_intent_program_primer()
		},
		{
			color: 'kelola',
			name: m.triage_intent_kelola_name(),
			icon: 'settings',
			desc: m.triage_intent_kelola_desc(),
			primer: m.triage_intent_kelola_primer(),
			fullWidth: true
		}
	]);
</script>

<div class="trajectory-grid flex flex-col gap-3 px-1 py-2">
	<!-- Header -->
	<div class="text-center">
		<p class="text-sm font-medium text-foreground">Apa yang ingin kamu lakukan?</p>
		<p class="mt-0.5 text-xs text-muted-foreground">Pilih salah satu, atau langsung ceritakan di bawah</p>
	</div>

	<!-- Grid -->
	<div class="grid grid-cols-2 gap-2">
		{#each INTENTS as item (item.name)}
			{@const colors = trajectoryColors(item.color)}
			<button
				type="button"
				class="trajectory-chip group flex items-start gap-2.5 rounded-xl border border-border/40 bg-card/60 px-3 py-2.5 text-left transition-all hover:border-border/80 hover:bg-card hover:shadow-sm active:scale-[0.97]"
				class:col-span-2={item.fullWidth}
				onclick={() => onSelect(item.primer)}
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-lg {colors.bgLight}">
					<DynamicIcon name={item.icon} fallback={item.color === 'kelola' ? 'program' : item.color} class="size-4 {colors.text}" />
				</div>
				<div class="min-w-0 flex-1">
					<p class="text-[13px] font-semibold leading-tight {colors.text}">
						{item.name}
					</p>
					<p class="mt-0.5 text-[11px] leading-snug text-muted-foreground">
						{item.desc}
					</p>
				</div>
			</button>
		{/each}
	</div>

	<!-- Free-text prompt — same visual weight as grid items -->
	<div class="flex items-center gap-2.5 rounded-xl border border-dashed border-primary/30 bg-primary/5 px-3 py-2.5">
		<div class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10">
			<DynamicIcon name="message-circle" class="size-4 text-primary" />
		</div>
		<div class="min-w-0 flex-1">
			<p class="text-[13px] font-semibold leading-tight text-primary">
				Atau ceritakan langsung
			</p>
			<p class="mt-0.5 text-[11px] leading-snug text-muted-foreground">
				Ketik ceritamu di bawah — AI akan menentukan jalur yang tepat
			</p>
		</div>
	</div>
</div>

<style>
	.trajectory-grid {
		mask-image: linear-gradient(to bottom, transparent 0%, black 4%, black 92%, transparent 100%);
	}

	.trajectory-chip {
		backdrop-filter: blur(8px);
	}
</style>

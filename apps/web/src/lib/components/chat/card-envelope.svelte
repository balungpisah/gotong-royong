<script lang="ts">
	import { safeSlide as slide } from '$lib/utils/safe-slide';
	import type {
		ChatMessage,
		AiCardMessage,
		DiffCardMessage,
		VoteCardMessage,
		EvidenceMessage,
		GalangMessage,
		SystemMessage
	} from '$lib/types';
	import { Badge } from '$lib/components/ui/badge';
	import { m } from '$lib/paraglide/messages';
	import AiInlineCard from './ai-inline-card.svelte';
	import DiffCardWrapper from './diff-card-wrapper.svelte';
	import VoteCardWrapper from './vote-card-wrapper.svelte';
	import EvidenceCard from './evidence-card.svelte';
	import GalangMessageRenderer from './galang-message.svelte';
	import Bot from '@lucide/svelte/icons/bot';
	import FilePen from '@lucide/svelte/icons/file-pen';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import FileCheck from '@lucide/svelte/icons/file-check';
	import Coins from '@lucide/svelte/icons/coins';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Trophy from '@lucide/svelte/icons/trophy';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import Shield from '@lucide/svelte/icons/shield';
	import FileText from '@lucide/svelte/icons/file-text';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import type { Component } from 'svelte';

	type CardMessage = AiCardMessage | DiffCardMessage | VoteCardMessage | EvidenceMessage | GalangMessage | SystemMessage;

	interface Props {
		message: CardMessage;
		defaultExpanded?: boolean;
		/** 'phase' = filled dot colored by state, 'minor' = hollow dark grey ring */
		dotVariant?: 'phase' | 'minor';
	}

	let { message, defaultExpanded = false, dotVariant = 'minor' }: Props = $props();

	let expanded = $state(false);

	// Sync initial value from prop (avoids state_referenced_locally warning)
	$effect(() => {
		expanded = defaultExpanded;
	});

	function toggle() {
		expanded = !expanded;
	}

	// ---------------------------------------------------------------------------
	// Badge labels (reused from ai-inline-card)
	// ---------------------------------------------------------------------------
	const badgeLabels: Record<string, string> = {
		classified: '🤖 Klasifikasi',
		suggested: '🤖 Saran',
		stalled: '⚠ Macet',
		dampak: '🌱 Dampak',
		ringkasan: '📝 Ringkasan',
		duplikat: '⚠ Duplikat'
	};

	const evidenceTypeLabels: Record<string, string> = {
		testimony: 'Kesaksian',
		corroboration: 'Korroborasi',
		document: 'Dokumen'
	};

	const galangSubtypeLabels: Record<string, string> = {
		contribution: 'Kontribusi',
		disbursement: 'Pencairan',
		milestone: 'Tonggak'
	};

	// ---------------------------------------------------------------------------
	// System message icon mapping
	// ---------------------------------------------------------------------------
	const subtypeIcons: Record<string, Component<{ class?: string }>> = {
		checkpoint_completed: CircleCheck,
		phase_activated: ArrowRight,
		phase_completed: Trophy,
		vote_result: BarChart3,
		member_joined: UserPlus,
		role_assigned: Shield,
		plan_updated: FileText,
		galang_transaction: Coins
	};

	// ---------------------------------------------------------------------------
	// Handlebar config — derived per message type
	// ---------------------------------------------------------------------------

	interface HandlebarConfig {
		icon: Component<{ class?: string }>;
		title: string;
		metric: string;
		actionLabel: string;
		actionVariant: 'default' | 'success' | 'warning' | 'danger' | 'info' | 'secondary';
	}

	const timeStr = $derived(
		new Date(message.timestamp).toLocaleTimeString('id-ID', {
			hour: '2-digit',
			minute: '2-digit'
		})
	);

	const config = $derived.by((): HandlebarConfig => {
		switch (message.type) {
			case 'ai_card': {
				const msg = message as AiCardMessage;
				const title = msg.badge
					? (badgeLabels[msg.badge] ?? msg.badge)
					: msg.title ?? 'AI';
				const blockCount = msg.blocks.length;
				const metric = blockCount > 1 ? `${blockCount} ${m.chat_card_blocks()}` : '';
				return { icon: Bot, title, metric, actionLabel: '', actionVariant: 'secondary' };
			}
			case 'diff_card': {
				const msg = message as DiffCardMessage;
				const adds = msg.diff.items.filter((i) => i.operation === 'add').length;
				const mods = msg.diff.items.filter((i) => i.operation === 'modify').length;
				const removes = msg.diff.items.filter((i) => i.operation === 'remove').length;
				const parts: string[] = [];
				if (adds > 0) parts.push(`+${adds}`);
				if (mods > 0) parts.push(`~${mods}`);
				if (removes > 0) parts.push(`-${removes}`);
				return {
					icon: FilePen,
					title: msg.diff.summary,
					metric: parts.join(' '),
					actionLabel: m.chat_card_review(),
					actionVariant: 'warning'
				};
			}
			case 'vote_card': {
				const msg = message as VoteCardMessage;
				const metric = `${msg.block.total_voted}/${msg.block.total_eligible}`;
				const isEnded = new Date(msg.block.ends_at) <= new Date();
				let actionLabel: string;
				let actionVariant: HandlebarConfig['actionVariant'];
				if (isEnded) {
					actionLabel = m.chat_card_ended();
					actionVariant = 'secondary';
				} else if (msg.block.user_voted) {
					actionLabel = `✓ ${m.chat_card_voted()}`;
					actionVariant = 'success';
				} else {
					actionLabel = m.chat_card_vote();
					actionVariant = 'danger';
				}
				return { icon: BarChart3, title: msg.block.question, metric, actionLabel, actionVariant };
			}
			case 'evidence': {
				const msg = message as EvidenceMessage;
				const typeLabel = evidenceTypeLabels[msg.evidence_type] ?? msg.evidence_type;
				const attachCount = msg.attachments?.length ?? 0;
				const metric = attachCount > 0 ? `${typeLabel} · ${attachCount} lampiran` : typeLabel;
				return { icon: FileCheck, title: msg.author.name, metric, actionLabel: '', actionVariant: 'secondary' };
			}
			case 'galang': {
				const msg = message as GalangMessage;
				const amount =
					msg.amount != null
						? new Intl.NumberFormat('id-ID', {
								style: 'currency',
								currency: msg.currency ?? 'IDR',
								maximumFractionDigits: 0
							}).format(msg.amount)
						: '';
				const subtypeLabel = galangSubtypeLabels[msg.subtype] ?? msg.subtype;
				return {
					icon: Coins,
					title: msg.content,
					metric: amount,
					actionLabel: subtypeLabel,
					actionVariant: msg.subtype === 'contribution' ? 'success' : msg.subtype === 'disbursement' ? 'warning' : 'info'
				};
			}
			case 'system': {
				const msg = message as SystemMessage;
				const icon = subtypeIcons[msg.subtype] || ArrowRight;
				return { icon, title: msg.content, metric: '', actionLabel: '', actionVariant: 'secondary' };
			}
			default:
				return { icon: Bot, title: '', metric: '', actionLabel: '', actionVariant: 'secondary' };
		}
	});

	const Icon = $derived(config.icon);

	// ---------------------------------------------------------------------------
	// Dot styling — phase blocks get filled color, minor blocks get hollow ring
	// ---------------------------------------------------------------------------
	const dotClass = $derived.by((): string => {
		if (dotVariant === 'phase') {
			// System messages get their own color logic
			if (message.type === 'system') {
				const sub = (message as SystemMessage).subtype;
				if (sub === 'phase_completed' || sub === 'checkpoint_completed')
					return 'bg-berhasil';
				if (sub === 'phase_activated')
					return 'bg-primary/60';
				return 'bg-muted-foreground/40';
			}
			// Filled dot — color reflects state of the block
			switch (config.actionVariant) {
				case 'success':
					return 'bg-berhasil';
				case 'warning':
					return 'bg-peringatan';
				case 'danger':
					return 'bg-bahaya';
				default:
					return 'bg-primary/60';
			}
		}
		// Minor block — hollow ring, dark grey
		return 'ring-1 ring-muted-foreground/40';
	});
</script>

<div class="flex flex-col" data-slot="card-envelope" data-card-type={message.type}>
	<!-- Handlebar — always visible, italic text style -->
	<button
		type="button"
		onclick={toggle}
		class="group flex w-full items-center gap-1.5 py-0.5 text-left transition"
	>
		<!-- Timeline dot — sits on top of the vertical line behind it -->
		<div class="relative z-10 size-2 shrink-0 rounded-full bg-background {dotClass}"></div>
		<Icon class="size-3 shrink-0 text-muted-foreground/50" />
		<span class="min-w-0 flex-1 truncate text-xs italic text-muted-foreground/70 group-hover:text-muted-foreground transition-colors">
			{config.title}
		</span>
		{#if config.metric}
			<span class="shrink-0 text-[11px] italic text-muted-foreground/50">{config.metric}</span>
		{/if}
		{#if config.actionLabel}
			<Badge variant={config.actionVariant} class="shrink-0 text-[10px]">
				{config.actionLabel}
			</Badge>
		{/if}
		<span class="shrink-0 text-[10px] text-muted-foreground/40">{timeStr}</span>
		<ChevronDown
			class="size-3 shrink-0 text-muted-foreground/40 transition-transform duration-150 {expanded
				? 'rotate-180'
				: ''}"
		/>
	</button>

	<!-- Expanded content — unified Ruang Interaksi style wrapper -->
	{#if expanded}
		<div class="pl-5 pt-1" transition:slide={{ duration: 150 }}>
			<div class="space-y-2.5 rounded-lg border border-border/40 bg-card/60 px-3 py-2.5 text-[11px] leading-relaxed text-muted-foreground">
				{#if message.type === 'system'}
					<p>{(message as SystemMessage).content}</p>
				{:else if message.type === 'ai_card'}
					<AiInlineCard message={message as AiCardMessage} />
				{:else if message.type === 'diff_card'}
					<DiffCardWrapper message={message as DiffCardMessage} />
				{:else if message.type === 'vote_card'}
					<VoteCardWrapper message={message as VoteCardMessage} />
				{:else if message.type === 'evidence'}
					<EvidenceCard message={message as EvidenceMessage} />
				{:else if message.type === 'galang'}
					<GalangMessageRenderer message={message as GalangMessage} />
				{/if}
			</div>
		</div>
	{/if}
</div>

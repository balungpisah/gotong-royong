<script lang="ts">
	import { slide } from 'svelte/transition';
	import type {
		ChatMessage,
		AiCardMessage,
		DiffCardMessage,
		VoteCardMessage,
		EvidenceMessage,
		GalangMessage
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
	import type { Component } from 'svelte';

	type CardMessage = AiCardMessage | DiffCardMessage | VoteCardMessage | EvidenceMessage | GalangMessage;

	interface Props {
		message: CardMessage;
		defaultExpanded?: boolean;
	}

	let { message, defaultExpanded = false }: Props = $props();

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
			default:
				return { icon: Bot, title: '', metric: '', actionLabel: '', actionVariant: 'secondary' };
		}
	});

	const Icon = $derived(config.icon);
</script>

<div class="flex flex-col" data-slot="card-envelope" data-card-type={message.type}>
	<!-- Handlebar — always visible -->
	<button
		type="button"
		onclick={toggle}
		class="flex w-full items-center gap-2 rounded-lg border border-border/40 bg-muted/20 px-3 py-1.5 text-left transition hover:bg-muted/40"
	>
		<Icon class="size-3.5 shrink-0 text-muted-foreground" />
		<span class="min-w-0 flex-1 truncate text-xs font-medium italic text-muted-foreground">
			{config.title}
		</span>
		{#if config.metric}
			<span class="shrink-0 text-[10px] italic text-muted-foreground">{config.metric}</span>
		{/if}
		{#if config.actionLabel}
			<Badge variant={config.actionVariant} class="shrink-0 text-[9px]">
				{config.actionLabel}
			</Badge>
		{/if}
		<span class="shrink-0 text-[9px] text-muted-foreground/60">{timeStr}</span>
		<ChevronDown
			class="size-3.5 shrink-0 text-muted-foreground transition-transform duration-150 {expanded
				? 'rotate-180'
				: ''}"
		/>
	</button>

	<!-- Expanded content -->
	{#if expanded}
		<div class="pt-1" transition:slide={{ duration: 150 }}>
			{#if message.type === 'ai_card'}
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
	{/if}
</div>

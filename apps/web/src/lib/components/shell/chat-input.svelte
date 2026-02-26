<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import { getTriageStore, getWitnessStore, getFeedStore, getGroupStore } from '$lib/stores';
	import { Badge } from '$lib/components/ui/badge';
	import Tip from '$lib/components/ui/tip.svelte';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import SendHorizontal from '@lucide/svelte/icons/send-horizontal';
	import X from '@lucide/svelte/icons/x';
	import Layers from '@lucide/svelte/icons/layers';
	import Sprout from '@lucide/svelte/icons/sprout';
	import Gauge from '@lucide/svelte/icons/gauge';
	import Plus from '@lucide/svelte/icons/plus';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import Settings from '@lucide/svelte/icons/settings';
	import TrajectoryGrid from '$lib/components/triage/trajectory-grid.svelte';
	import TriageAttachmentPicker from '$lib/components/triage/triage-attachment-picker.svelte';
	import TriageAttachmentPreview from '$lib/components/triage/triage-attachment-preview.svelte';
	import type { WitnessCreateInput, TriageBudget, TriageAttachment } from '$lib/types';
	import Zap from '@lucide/svelte/icons/zap';
	import Video from '@lucide/svelte/icons/video';
	import Mic from '@lucide/svelte/icons/mic';
	import type { BadgeVariant } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		onWitnessCreated?: (witnessId: string) => void;
	}

	let { onWitnessCreated }: Props = $props();

	const triageStore = getTriageStore();
	const witnessStore = getWitnessStore();
	const feedStore = getFeedStore();
	const groupStore = getGroupStore();

	let content = $state('');
	let expanded = $state(false);
	let textareaEl = $state<HTMLTextAreaElement | null>(null);
	let wrapperEl = $state<HTMLDivElement | null>(null);

	/** Cooldown — prevents rapid re-submission after completing a triage session. */
	const COOLDOWN_MS = 30_000;
	let cooldownUntil = $state(0);
	let cooldownRemaining = $state(0);
	let cooldownTimer: ReturnType<typeof setInterval> | null = null;
	const isOnCooldown = $derived(cooldownRemaining > 0);

	function startCooldown() {
		cooldownUntil = Date.now() + COOLDOWN_MS;
		cooldownRemaining = COOLDOWN_MS;
		cooldownTimer = setInterval(() => {
			cooldownRemaining = Math.max(0, cooldownUntil - Date.now());
			if (cooldownRemaining <= 0 && cooldownTimer) {
				clearInterval(cooldownTimer);
				cooldownTimer = null;
			}
		}, 1000);
	}

	/** Chat messages for the current triage session */
	interface TriageChatMessage {
		role: 'user' | 'ai';
		text: string;
		attachments?: { type: 'image' | 'video' | 'audio'; preview_url: string }[];
	}
	let messages = $state<TriageChatMessage[]>([]);
	let submitError = $state<string | null>(null);
	const triageSubmitError = $derived(submitError);

	/** Pending file attachments for current message */
	let pendingAttachments = $state<TriageAttachment[]>([]);

	function handleFilesSelected(files: File[]) {
		const remaining = 5 - pendingAttachments.length;
		const toAdd = Array.from(files).slice(0, remaining);

		for (const file of toAdd) {
			const type = file.type.startsWith('image/')
				? 'image'
				: file.type.startsWith('video/')
					? 'video'
					: ('audio' as const);
			pendingAttachments.push({
				id: `att-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
				file,
				type,
				preview_url: URL.createObjectURL(file)
			});
		}
	}

	function handleRemoveAttachment(id: string) {
		const idx = pendingAttachments.findIndex((a) => a.id === id);
		if (idx >= 0) {
			URL.revokeObjectURL(pendingAttachments[idx].preview_url);
			pendingAttachments.splice(idx, 1);
		}
	}

	const canSend = $derived(
		(content.trim().length > 0 || pendingAttachments.length > 0) &&
			!triageStore.loading &&
			!triageStore.isReady &&
			!isOnCooldown
	);
	const hasSession = $derived(triageStore.sessionId !== null);

	/** Budget tracking — drives the "Sisa Energi" energy bar. */
	const budget = $derived(triageStore.result?.budget as TriageBudget | undefined);
	const budgetPct = $derived(budget ? budget.budget_pct : 0);
	const budgetColor = $derived(
		budgetPct > 0.7 ? 'bg-bahaya' : budgetPct > 0.4 ? 'bg-waspada' : 'bg-berhasil'
	);

	/** Map track hint to badge variant */
	function trackBadgeVariant(track?: string): BadgeVariant {
		if (!track) return 'secondary';
		const map: Record<string, BadgeVariant> = {
			tuntaskan: 'track-tuntaskan',
			wujudkan: 'track-wujudkan',
			telusuri: 'track-telusuri',
			rayakan: 'track-rayakan',
			musyawarah: 'track-musyawarah'
		};
		return map[track] ?? 'secondary';
	}

	/** Phase summary from proposed plan */
	const planPhaseCount = $derived(triageStore.proposedPlan?.branches?.[0]?.phases?.length ?? 0);
	const firstPhaseTitle = $derived(
		triageStore.proposedPlan?.branches?.[0]?.phases?.[0]?.title ?? null
	);
	const declaredConversationBlocks = $derived(triageStore.blocks?.conversation ?? []);
	const declaredStructuredBlocks = $derived(triageStore.blocks?.structured ?? []);

	function blockLabel(blockId: string): string {
		const labels: Record<string, string> = {
			chat_message: 'Chat Message',
			ai_inline_card: 'AI Inline',
			diff_card: 'Diff Card',
			vote_card: 'Vote Card',
			moderation_hold_card: 'Moderation Hold',
			duplicate_detection_card: 'Duplicate Detection',
			credit_nudge_card: 'Credit Nudge',
			list: 'List',
			document: 'Document',
			form: 'Form',
			computed: 'Computed',
			display: 'Display',
			vote: 'Vote',
			reference: 'Reference'
		};
		return labels[blockId] ?? blockId;
	}

	function autoResize() {
		if (!textareaEl) return;
		textareaEl.style.height = 'auto';
		textareaEl.style.height = Math.min(textareaEl.scrollHeight, 120) + 'px';
	}

	function expand() {
		expanded = true;
		requestAnimationFrame(() => textareaEl?.focus());
	}

	function collapse() {
		expanded = false;
		submitError = null;
	}

	function handleBackdropClick() {
		collapse();
	}

	function aiResponseText(): string {
		if (!triageStore.result) return '';
		const r = triageStore.result;
		const parts: string[] = [];

		if (r.confidence?.label) parts.push(r.confidence.label);

		if (r.bar_state === 'probing') {
			parts.push(m.triage_ai_probing());
		} else if (r.bar_state === 'leaning') {
			parts.push(m.triage_ai_leaning());
		} else if (
			r.bar_state === 'ready' ||
			r.bar_state === 'vault-ready' ||
			r.bar_state === 'siaga-ready'
		) {
			if (r.track_hint) parts.push(m.triage_track_label({ track: r.track_hint! }));
			if (r.seed_hint) parts.push(m.triage_seed_label({ seed: r.seed_hint! }));
			parts.push(m.triage_ready());
		}

		return parts.join(' · ') || m.triage_processing();
	}

	async function handleSubmit(injectedText?: string) {
		const text = (injectedText ?? content).trim();
		if ((!text && pendingAttachments.length === 0) || triageStore.loading || isOnCooldown) return;
		submitError = null;

		// Capture attachment previews for the message bubble
		const msgAttachments =
			pendingAttachments.length > 0
				? pendingAttachments.map((a) => ({ type: a.type, preview_url: a.preview_url }))
				: undefined;

		// Extract files for the service call
		const files = pendingAttachments.length > 0 ? pendingAttachments.map((a) => a.file) : undefined;

		if (!hasSession) {
			await triageStore.startTriage(text, files);
		} else {
			await triageStore.updateTriage(text, files);
		}

		if (triageStore.error) {
			submitError = triageStore.error;
			if (injectedText && content.trim().length === 0) {
				content = injectedText;
				autoResize();
			}
			requestAnimationFrame(() => textareaEl?.focus());
			return;
		}

		content = '';
		if (textareaEl) textareaEl.style.height = 'auto';

		messages.push({ role: 'user', text: text || '(lampiran)', attachments: msgAttachments });
		messages.push({ role: 'ai', text: aiResponseText() });

		// Clear pending (don't revoke URLs — they're referenced by message bubbles now)
		pendingAttachments = [];

		// Re-focus textarea for next message
		requestAnimationFrame(() => textareaEl?.focus());
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
		if (e.key === 'Escape') {
			collapse();
		}
	}

	function handleReset() {
		triageStore.reset();
		submitError = null;
		messages = [];
		content = '';
		for (const attachment of pendingAttachments) {
			URL.revokeObjectURL(attachment.preview_url);
		}
		pendingAttachments = [];
		startCooldown();
		collapse();
	}

	/** User tapped a trajectory chip — submit the primer directly. */
	function handleTrajectorySelect(primer: string) {
		handleSubmit(primer);
	}

	async function handleCreateWitness() {
		if (witnessStore.creating || !triageStore.result || !triageStore.sessionId) return;
		const schemaVersion = triageStore.result.schema_version ?? 'triage.v1';
		const input: WitnessCreateInput = {
			schema_version: schemaVersion,
			triage_session_id: triageStore.sessionId
		};

		const witnessId = await witnessStore.createWitness(input);
		if (!witnessId) return;
		await feedStore.loadFeed();
		onWitnessCreated?.(witnessId);

		// Reset triage + cooldown to prevent rapid re-submission
		triageStore.reset();
		submitError = null;
		messages = [];
		content = '';
		startCooldown();
		collapse();
	}

	async function handleCreateGroup() {
		if (groupStore.creating || !triageStore.result?.kelola_result) return;

		const kelola = triageStore.result.kelola_result;
		if (kelola.action !== 'create' || !kelola.group_detail) return;

		const { name, description, join_policy, entity_type } = kelola.group_detail;
		const groupId = await groupStore.createGroup({ name, description, join_policy, entity_type });
		if (!groupId) return;

		// Reset triage + cooldown, then navigate to new group
		triageStore.reset();
		submitError = null;
		messages = [];
		content = '';
		startCooldown();
		collapse();
		goto(`/komunitas/kelompok/${groupId}`);
	}
</script>

<!-- Wrapper: keeps the compact card in flow, expanded panel is absolute -->
<div class="relative" bind:this={wrapperEl}>
	<!-- Pulsing arrow — comically large, points at the start box -->
	{#if !expanded && !hasSession}
		<div class="onboarding-arrow" aria-hidden="true">
			<svg viewBox="0 0 120 60" fill="none" xmlns="http://www.w3.org/2000/svg">
				<path
					d="M0 22h80v-22l40 30-40 30v-22h-80z"
					fill="var(--color-primary)"
					fill-opacity="0.85"
				/>
			</svg>
		</div>
	{/if}

	<!-- Compact card — always in document flow to hold space -->
	<div
		class="triage-card cursor-pointer rounded-2xl border-2 border-primary/40 p-3.5 transition-all hover:border-primary/60 hover:shadow-lg"
		class:invisible={expanded}
		onclick={expand}
		onkeydown={(e) => e.key === 'Enter' && expand()}
		role="button"
		tabindex="0"
	>
		<!-- "Mulai di sini" badge -->
		<div class="mb-2.5">
			<span
				class="inline-flex items-center gap-1 rounded-full bg-primary px-2.5 py-1 text-small font-bold tracking-wide text-primary-foreground uppercase"
			>
				<Sparkles class="size-2.5" />
				{m.triage_start_here()}
			</span>
		</div>

		<div class="flex items-center gap-2.5">
			<div class="flex size-9 items-center justify-center rounded-xl bg-primary/15">
				<Sparkles class="size-4 text-primary" />
			</div>
			<div class="flex-1 min-w-0">
				<p class="text-[13px] font-semibold text-foreground leading-tight">
					{m.triage_tell_story()}
				</p>
				<p class="mt-0.5 text-small text-muted-foreground truncate">
					{m.triage_tap_to_start()}
				</p>
			</div>
		</div>
	</div>

	<!-- Backdrop scrim -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-30 bg-black/15 transition-opacity duration-300 ease-out"
		class:opacity-0={!expanded}
		class:pointer-events-none={!expanded}
		onclick={handleBackdropClick}
		onkeydown={() => {}}
	></div>

	<!-- Expanded panel — fixed centered overlay for focused writing -->
	<div
		class="triage-panel fixed inset-0 z-40 m-auto flex flex-col rounded-2xl border border-primary/15 bg-card shadow-2xl ring-1 ring-primary/10 transition-all duration-300 ease-out"
		class:opacity-0={!expanded}
		class:pointer-events-none={!expanded}
		class:scale-95={!expanded}
	>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-border/40 px-4 py-2.5">
			<div class="flex items-center gap-2 text-small text-muted-foreground">
				<Sparkles class="size-3.5 text-primary" />
				<span class="font-medium">{m.triage_ai_label()}</span>
				{#if triageStore.confidence}
					<span class="rounded-full bg-primary/10 px-2 py-0.5 text-small font-medium text-primary">
						{triageStore.confidence.label}
					</span>
				{/if}
				{#if budget && hasSession}
					<div
						class="ml-1 flex items-center gap-1.5"
						title={m.triage_energy_remaining({ pct: String(Math.round((1 - budgetPct) * 100)) })}
					>
						<Zap
							class="size-3 {budgetPct > 0.7
								? 'text-bahaya'
								: budgetPct > 0.4
									? 'text-waspada'
									: 'text-berhasil'}"
						/>
						<div class="h-1.5 w-14 overflow-hidden rounded-full bg-muted">
							<div
								class="h-full rounded-full transition-all duration-500 ease-out {budgetColor}"
								style="width: {Math.max(4, (1 - budgetPct) * 100)}%"
							></div>
						</div>
					</div>
				{/if}
			</div>
			<div class="flex items-center gap-1">
				{#if hasSession}
					<Tip text={m.triage_restart()}>
						<Button
							variant="ghost"
							size="icon-sm"
							onclick={handleReset}
							class="size-7 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
							aria-label={m.triage_restart()}
							tabindex={expanded ? 0 : -1}
						>
							<RotateCcw class="size-3.5" />
						</Button>
					</Tip>
				{/if}
				<Tip text={m.common_close()}>
					<Button
						variant="ghost"
						size="icon-sm"
						onclick={collapse}
						class="size-7"
						aria-label={m.common_close()}
						tabindex={expanded ? 0 : -1}
					>
						<X class="size-4" />
					</Button>
				</Tip>
			</div>
		</div>

		<!-- Messages area -->
		<div class="flex-1 overflow-y-auto px-4 py-3">
			{#if messages.length === 0 && isOnCooldown}
				<div class="flex flex-col items-center justify-center gap-2 py-12 text-center">
					<div class="flex size-10 items-center justify-center rounded-full bg-muted">
						<RotateCcw class="size-5 text-muted-foreground" />
					</div>
					<p class="text-body font-medium text-foreground">{m.triage_session_complete()}</p>
					<p class="text-small text-muted-foreground">
						{m.triage_cooldown({ seconds: String(Math.ceil(cooldownRemaining / 1000)) })}
					</p>
				</div>
			{:else if messages.length === 0}
				<TrajectoryGrid onSelect={handleTrajectorySelect} />
			{:else}
				<div class="flex flex-col gap-3">
					{#each messages as msg}
						{#if msg.role === 'user'}
							<div class="flex justify-end">
								<div
									class="max-w-[80%] rounded-2xl rounded-br-md bg-primary px-3 py-2 text-body text-primary-foreground"
								>
									{msg.text}
									{#if msg.attachments?.length}
										<div class="mt-1.5 flex gap-1.5 flex-wrap">
											{#each msg.attachments as att}
												{#if att.type === 'image'}
													<img
														src={att.preview_url}
														alt=""
														class="size-16 rounded-lg object-cover ring-1 ring-white/20"
													/>
												{:else if att.type === 'video'}
													<div
														class="flex size-16 items-center justify-center rounded-lg bg-black/20 ring-1 ring-white/20"
													>
														<Video class="size-6 text-primary-foreground/70" />
													</div>
												{:else}
													<div
														class="flex size-16 items-center justify-center rounded-lg bg-black/20 ring-1 ring-white/20"
													>
														<Mic class="size-6 text-primary-foreground/70" />
													</div>
												{/if}
											{/each}
										</div>
									{/if}
								</div>
							</div>
						{:else}
							<div class="flex gap-2">
								<div
									class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10"
								>
									<Sparkles class="size-3 text-primary" />
								</div>
								<div
									class="max-w-[80%] rounded-2xl rounded-bl-md bg-muted/70 px-3 py-2 text-body text-foreground"
								>
									{msg.text}
								</div>
							</div>
						{/if}
					{/each}

					{#if triageStore.loading}
						<div class="flex gap-2">
							<div
								class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10"
							>
								<Sparkles class="size-3 text-primary" />
							</div>
							<div class="rounded-2xl rounded-bl-md bg-muted/70 px-3 py-2">
								<div class="flex gap-1">
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 0ms"
									></div>
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 150ms"
									></div>
									<div
										class="size-1.5 animate-bounce rounded-full bg-muted-foreground/40"
										style="animation-delay: 300ms"
									></div>
								</div>
							</div>
						</div>
					{/if}

					{#if triageStore.isReady}
						{#if triageStore.route === 'kelola'}
							<!-- Kelola result card -->
							<div
								class="mt-2 overflow-hidden rounded-xl border border-blue-600/30 bg-card shadow-sm"
							>
								<div class="flex items-center gap-2 border-b border-border/30 px-3 py-2">
									<Badge variant="secondary">
										<Settings class="mr-1 size-3" />
										{m.triage_intent_kelola_name()}
									</Badge>
									{#if triageStore.confidence}
										<span class="ml-auto flex items-center gap-1 text-small text-muted-foreground">
											<Gauge class="size-3" />
											{triageStore.confidence.label}
										</span>
									{/if}
								</div>
								{#if triageStore.result?.kelola_result?.group_detail}
									{@const detail = triageStore.result.kelola_result.group_detail}
									<div class="px-3 py-2.5">
										<p class="text-body font-semibold text-foreground leading-tight">
											{detail.name}
										</p>
										<p class="mt-1 text-small text-muted-foreground line-clamp-2">
											{detail.description}
										</p>
										{#if declaredConversationBlocks.length > 0 || declaredStructuredBlocks.length > 0}
											<div class="mt-2 rounded-lg border border-border/30 bg-muted/30 p-2">
												<p class="text-small font-medium text-foreground">Operator blocks</p>
												<div class="mt-1 flex flex-wrap gap-1">
													{#each declaredConversationBlocks as blockId (blockId)}
														<Badge variant="outline" class="text-[10px] uppercase">
															CHAT · {blockLabel(blockId)}
														</Badge>
													{/each}
													{#each declaredStructuredBlocks as blockId (blockId)}
														<Badge variant="outline" class="text-[10px] uppercase">
															STRUCT · {blockLabel(blockId)}
														</Badge>
													{/each}
												</div>
											</div>
										{/if}
									</div>
								{/if}
								<div class="flex items-center gap-2 border-t border-border/30 px-3 py-2.5">
									<Button
										variant="default"
										onclick={handleCreateGroup}
										disabled={groupStore.creating}
										class="flex-1 bg-blue-600 hover:bg-blue-700"
									>
										{#if groupStore.creating}
											<div
												class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
											></div>
											{m.group_create_creating()}
										{:else}
											<Plus class="size-4" />
											{m.triage_create_group()}
										{/if}
									</Button>
									<Button variant="outline" disabled title={m.common_coming_soon()}>
										{m.triage_modify()}
									</Button>
								</div>
							</div>
						{:else}
							<!-- Path plan preview card -->
							<div
								class="mt-2 overflow-hidden rounded-xl border border-border/60 bg-card shadow-sm"
							>
								<!-- Track + seed header -->
								<div class="flex items-center gap-2 border-b border-border/30 px-3 py-2">
									{#if triageStore.trackHint}
										<Badge variant={trackBadgeVariant(triageStore.trackHint)}>
											{triageStore.trackHint}
										</Badge>
									{/if}
									{#if triageStore.seedHint}
										<span class="flex items-center gap-1 text-small text-muted-foreground">
											<Sprout class="size-3" />
											{triageStore.seedHint}
										</span>
									{/if}
									{#if triageStore.confidence}
										<span class="ml-auto flex items-center gap-1 text-small text-muted-foreground">
											<Gauge class="size-3" />
											{triageStore.confidence.label}
										</span>
									{/if}
								</div>

								<!-- Plan summary -->
								{#if triageStore.proposedPlan}
									<div class="px-3 py-2.5">
										<p class="text-body font-semibold text-foreground leading-tight">
											{triageStore.proposedPlan.title}
										</p>
										{#if triageStore.proposedPlan.summary}
											<p class="mt-1 text-small text-muted-foreground line-clamp-2">
												{triageStore.proposedPlan.summary}
											</p>
										{/if}
										<div class="mt-2 flex items-center gap-1.5 text-small text-muted-foreground">
											<Layers class="size-3" />
											<span>{m.triage_phases({ count: String(planPhaseCount) })}</span>
											{#if firstPhaseTitle}
												<span class="text-muted-foreground/50">·</span>
												<span class="truncate">{firstPhaseTitle}</span>
											{/if}
										</div>
										{#if declaredConversationBlocks.length > 0 || declaredStructuredBlocks.length > 0}
											<div class="mt-2 rounded-lg border border-border/30 bg-muted/30 p-2">
												<p class="text-small font-medium text-foreground">Operator blocks</p>
												<div class="mt-1 flex flex-wrap gap-1">
													{#each declaredConversationBlocks as blockId (blockId)}
														<Badge variant="outline" class="text-[10px] uppercase">
															CHAT · {blockLabel(blockId)}
														</Badge>
													{/each}
													{#each declaredStructuredBlocks as blockId (blockId)}
														<Badge variant="outline" class="text-[10px] uppercase">
															STRUCT · {blockLabel(blockId)}
														</Badge>
													{/each}
												</div>
											</div>
										{/if}
									</div>
								{/if}

								<!-- Action buttons -->
								<div class="flex items-center gap-2 border-t border-border/30 px-3 py-2.5">
									{#if triageStore.kind === 'witness'}
										<Button
											variant="default"
											onclick={handleCreateWitness}
											disabled={witnessStore.creating}
											class="flex-1"
										>
											{#if witnessStore.creating}
												<div
													class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
												></div>
												{m.triage_creating()}
											{:else}
												<Plus class="size-4" />
												{m.triage_create_witness()}
											{/if}
										</Button>
									{:else}
										<Button variant="outline" disabled class="flex-1">
											Data siap dipublikasikan
										</Button>
									{/if}
									<Button variant="outline" disabled title={m.common_coming_soon()}>
										{m.triage_modify()}
									</Button>
								</div>
							</div>
						{/if}
					{/if}
				</div>
			{/if}
		</div>

		{#if triageSubmitError}
			<div
				class="mx-3 mb-2 flex items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2"
				role="status"
				aria-live="polite"
			>
				<p class="text-small text-destructive">{triageSubmitError}</p>
				<Button variant="outline" size="sm" onclick={() => handleSubmit()} disabled={!canSend}>
					Coba lagi
				</Button>
			</div>
		{/if}

		<!-- Attachment preview row -->
		{#if pendingAttachments.length > 0}
			<div class="border-t border-border/40 pt-2">
				<TriageAttachmentPreview
					attachments={pendingAttachments}
					onRemove={handleRemoveAttachment}
				/>
			</div>
		{/if}

		<!-- Input bar -->
		<div class="border-t border-border/40 px-3 py-2.5">
			<div class="flex items-end gap-2">
				<TriageAttachmentPicker
					onFilesSelected={handleFilesSelected}
					disabled={triageStore.loading ||
						triageStore.isReady ||
						isOnCooldown ||
						pendingAttachments.length >= 5}
				/>
				<div class="relative flex-1">
					<textarea
						bind:this={textareaEl}
						bind:value={content}
						oninput={autoResize}
						onkeydown={handleKeydown}
						placeholder={isOnCooldown
							? m.triage_wait()
							: triageStore.isReady
								? m.triage_complete()
								: m.shell_chat_placeholder()}
						disabled={triageStore.loading || triageStore.isReady || isOnCooldown}
						rows={1}
						tabindex={expanded ? 0 : -1}
						class="w-full resize-none rounded-lg border border-border/50 bg-background px-3 py-2 text-body text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-1 focus:ring-primary/30 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
					></textarea>
				</div>

				<Tip text={m.shell_chat_send()}>
					<Button
						variant="default"
						size="icon"
						onclick={() => handleSubmit()}
						disabled={!canSend}
						tabindex={expanded ? 0 : -1}
						class="shrink-0"
						aria-label={m.shell_chat_send()}
					>
						{#if triageStore.loading}
							<div
								class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
							></div>
						{:else}
							<SendHorizontal class="size-4" />
						{/if}
					</Button>
				</Tip>
			</div>
		</div>
	</div>
</div>

<style>
	/* Triage card — gradient bg + breathing glow for prominence */
	.triage-card {
		background: linear-gradient(
			135deg,
			oklch(from var(--color-primary) l c h / 0.06) 0%,
			oklch(from var(--color-card) l c h / 1) 70%
		);
		animation: triage-glow 3s ease-in-out infinite;
	}

	@keyframes triage-glow {
		0%,
		100% {
			box-shadow: 0 0 0 0 oklch(from var(--color-primary) l c h / 0);
		}
		50% {
			box-shadow: 0 0 16px 3px oklch(from var(--color-primary) l c h / 0.15);
		}
	}

	.triage-card:hover {
		animation: none;
		box-shadow: 0 0 20px 4px oklch(from var(--color-primary) l c h / 0.12);
	}

	/* Expanded panel — fixed centered, large overlay */
	.triage-panel {
		width: min(640px, 92vw);
		height: min(80vh, 720px);
		top: 0;
		right: 0;
		bottom: 0;
		left: 0;
		margin: auto;
	}

	.triage-panel.scale-95 {
		transform: scale(0.95);
	}

	/* ── Onboarding arrow — comically large pulsing pointer ─────── */
	.onboarding-arrow {
		position: absolute;
		top: 50%;
		right: calc(100% + 0.25rem);
		width: 4.5rem;
		transform: translateY(-50%);
		pointer-events: none;
		z-index: 5;
		animation: arrow-nudge 1.2s ease-in-out infinite;
		filter: drop-shadow(0 2px 6px oklch(from var(--color-primary) l c h / 0.35));
	}

	.onboarding-arrow svg {
		width: 100%;
		height: auto;
	}

	@keyframes arrow-nudge {
		0%,
		100% {
			transform: translateY(-50%) translateX(0);
		}
		50% {
			transform: translateY(-50%) translateX(0.5rem);
		}
	}
</style>

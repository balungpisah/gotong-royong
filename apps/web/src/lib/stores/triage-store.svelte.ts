/**
 * Triage Store — manages AI-00 entry triage state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Consumes TriageService interface for data operations.
 */

import type { TriageService } from '$lib/services/types';
import type { TriageResult } from '$lib/types';

export class TriageStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	result = $state<TriageResult | null>(null);
	loading = $state(false);
	error = $state<string | null>(null);
	sessionId = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	barState = $derived(this.result?.bar_state ?? 'listening');
	status = $derived(this.result?.status ?? 'draft');
	kind = $derived(this.result?.kind ?? null);
	isReady = $derived(
		this.status === 'final' ||
			this.barState === 'ready' ||
			this.barState === 'vault-ready' ||
			this.barState === 'siaga-ready'
	);
	proposedPlan = $derived(this.result?.proposed_plan ?? null);
	blocks = $derived(this.result?.blocks ?? null);
	structuredPayload = $derived(this.result?.structured_payload ?? []);
	conversationPayload = $derived(this.result?.conversation_payload ?? []);
	confidence = $derived(this.result?.confidence ?? null);
	trackHint = $derived(this.result?.track_hint ?? null);
	seedHint = $derived(this.result?.seed_hint ?? null);
	route = $derived(this.result?.route ?? null);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	private readonly service: TriageService;

	constructor(service: TriageService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async startTriage(content: string, attachments?: File[]) {
		this.loading = true;
		this.error = null;
		try {
			const result = await this.service.startTriage(content, attachments);
			this.result = result;
			this.sessionId = result.session_id ?? `triage-${Date.now()}`;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memulai triase';
		} finally {
			this.loading = false;
		}
	}

	async updateTriage(answer: string, attachments?: File[]) {
		if (!this.sessionId) return;
		this.loading = true;
		this.error = null;
		try {
			this.result = await this.service.updateTriage(this.sessionId, answer, attachments);
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memperbarui triase';
		} finally {
			this.loading = false;
		}
	}

	reset() {
		this.result = null;
		this.error = null;
		this.sessionId = null;
		this.loading = false;
	}
}

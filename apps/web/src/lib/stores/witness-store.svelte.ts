/**
 * Witness Store — manages witness list and detail state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Consumes WitnessService interface for data operations.
 */

import type { WitnessService } from '$lib/services/types';
import type { Witness, WitnessDetail, WitnessStatus, DiffResponse } from '$lib/types';

export class WitnessStore {
	// ---------------------------------------------------------------------------
	// List state
	// ---------------------------------------------------------------------------

	witnesses = $state<Witness[]>([]);
	listLoading = $state(false);
	listError = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Detail state
	// ---------------------------------------------------------------------------

	current = $state<WitnessDetail | null>(null);
	detailLoading = $state(false);
	detailError = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	activeWitnesses = $derived(this.witnesses.filter((w) => w.status === 'active'));
	unreadTotal = $derived(this.witnesses.reduce((sum, w) => sum + w.unread_count, 0));
	currentMessages = $derived(this.current?.messages ?? []);
	currentPlan = $derived(this.current?.plan ?? null);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	private readonly service: WitnessService;

	constructor(service: WitnessService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async loadList(opts?: { status?: WitnessStatus }) {
		this.listLoading = true;
		this.listError = null;
		try {
			const result = await this.service.list(opts);
			this.witnesses = result.items;
		} catch (err) {
			this.listError = err instanceof Error ? err.message : 'Gagal memuat daftar saksi';
		} finally {
			this.listLoading = false;
		}
	}

	async loadDetail(witnessId: string) {
		this.detailLoading = true;
		this.detailError = null;
		try {
			this.current = await this.service.get(witnessId);
		} catch (err) {
			this.detailError = err instanceof Error ? err.message : 'Gagal memuat detail saksi';
		} finally {
			this.detailLoading = false;
		}
	}

	async sendMessage(content: string) {
		if (!this.current) return;
		try {
			const message = await this.service.sendMessage(this.current.witness_id, content);
			this.current = {
				...this.current,
				messages: [...this.current.messages, message],
				message_count: this.current.message_count + 1
			};
		} catch (err) {
			console.error('[WitnessStore] sendMessage failed:', err);
			throw err;
		}
	}

	async respondToDiff(diffId: string, response: DiffResponse) {
		if (!this.current) return;
		try {
			await this.service.respondToDiff(this.current.witness_id, diffId, response);
		} catch (err) {
			console.error('[WitnessStore] respondToDiff failed:', err);
			throw err;
		}
	}

	async castVote(voteId: string, optionId: string) {
		if (!this.current) return;
		try {
			await this.service.castVote(this.current.witness_id, voteId, optionId);
		} catch (err) {
			console.error('[WitnessStore] castVote failed:', err);
			throw err;
		}
	}

	async refresh() {
		await this.loadList();
		if (this.current) {
			await this.loadDetail(this.current.witness_id);
		}
	}
}

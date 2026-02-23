/**
 * Signal Store — manages content-directed signal state per witness.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Backed by SignalService — currently mock, will swap to API.
 *
 * See spec: 13a-signal-completion-resolution.md
 */

import type { SignalService } from '$lib/services/types';
import type {
	ContentSignal,
	ContentSignalType,
	MyRelation,
	SignalCounts,
	WitnessCloseReason
} from '$lib/types';

export class SignalStore {
	private service: SignalService;

	// ---------------------------------------------------------------------------
	// State — per-witness, keyed by witness_id
	// ---------------------------------------------------------------------------

	relations = $state<Map<string, MyRelation>>(new Map());
	counts = $state<Map<string, SignalCounts>>(new Map());
	resolutions = $state<Map<string, ContentSignal[]>>(new Map());
	sending = $state(false);

	constructor(service: SignalService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Getters
	// ---------------------------------------------------------------------------

	getRelation(witnessId: string): MyRelation | undefined {
		return this.relations.get(witnessId);
	}

	getCounts(witnessId: string): SignalCounts | undefined {
		return this.counts.get(witnessId);
	}

	getResolutionsForWitness(witnessId: string): ContentSignal[] {
		return this.resolutions.get(witnessId) ?? [];
	}

	// ---------------------------------------------------------------------------
	// Actions — load data
	// ---------------------------------------------------------------------------

	async loadRelation(witnessId: string): Promise<void> {
		const relation = await this.service.getMyRelation(witnessId);
		this.relations = new Map(this.relations).set(witnessId, relation);
	}

	async loadCounts(witnessId: string): Promise<void> {
		const counts = await this.service.getSignalCounts(witnessId);
		this.counts = new Map(this.counts).set(witnessId, counts);
	}

	async loadResolutions(witnessId: string): Promise<void> {
		const resolved = await this.service.getResolutions(witnessId);
		this.resolutions = new Map(this.resolutions).set(witnessId, resolved);
	}

	// ---------------------------------------------------------------------------
	// Actions — send / remove signals
	// ---------------------------------------------------------------------------

	async sendSignal(witnessId: string, signalType: ContentSignalType): Promise<ContentSignal | null> {
		if (this.sending) return null;
		this.sending = true;

		try {
			const signal = await this.service.sendSignal(witnessId, signalType);

			// Optimistic update: refresh relation and counts from service
			await Promise.all([
				this.loadRelation(witnessId),
				this.loadCounts(witnessId)
			]);

			return signal;
		} finally {
			this.sending = false;
		}
	}

	async removeSignal(witnessId: string, signalType: ContentSignalType): Promise<void> {
		if (this.sending) return;
		this.sending = true;

		try {
			await this.service.removeSignal(witnessId, signalType);

			// Refresh relation and counts from service
			await Promise.all([
				this.loadRelation(witnessId),
				this.loadCounts(witnessId)
			]);
		} finally {
			this.sending = false;
		}
	}

	/**
	 * Toggle a signal: send if not active, remove if active.
	 * Returns the new active state.
	 */
	async toggleSignal(witnessId: string, signalType: ContentSignalType): Promise<boolean> {
		const relation = this.getRelation(witnessId);
		const isActive = signalType === 'saksi'
			? relation?.witnessed
			: relation?.flagged;

		if (isActive) {
			await this.removeSignal(witnessId, signalType);
			return false;
		} else {
			await this.sendSignal(witnessId, signalType);
			return true;
		}
	}

	// ---------------------------------------------------------------------------
	// Actions — resolution (mock demo)
	// ---------------------------------------------------------------------------

	async simulateResolution(
		witnessId: string,
		closeReason: WitnessCloseReason
	): Promise<ContentSignal[]> {
		if (!this.service.simulateResolution) return [];

		const resolved = await this.service.simulateResolution(witnessId, closeReason);
		this.resolutions = new Map(this.resolutions).set(witnessId, [
			...(this.resolutions.get(witnessId) ?? []),
			...resolved
		]);
		return resolved;
	}
}

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
import { SvelteMap } from 'svelte/reactivity';

export class SignalStore {
	private service: SignalService;

	// ---------------------------------------------------------------------------
	// State — per-witness, keyed by witness_id
	// ---------------------------------------------------------------------------

	relations = $state<SvelteMap<string, MyRelation>>(new SvelteMap());
	counts = $state<SvelteMap<string, SignalCounts>>(new SvelteMap());
	resolutions = $state<SvelteMap<string, ContentSignal[]>>(new SvelteMap());
	errors = $state<SvelteMap<string, string>>(new SvelteMap());
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

	getError(witnessId: string): string | null {
		return this.errors.get(witnessId) ?? null;
	}

	clearError(witnessId: string): void {
		if (!this.errors.has(witnessId)) return;
		const next = new SvelteMap(this.errors);
		next.delete(witnessId);
		this.errors = next;
	}

	private setError(witnessId: string, message: string): void {
		this.errors = new SvelteMap(this.errors).set(witnessId, message);
	}

	private toErrorMessage(err: unknown, fallback: string): string {
		return err instanceof Error ? err.message : fallback;
	}

	// ---------------------------------------------------------------------------
	// Actions — load data
	// ---------------------------------------------------------------------------

	async loadRelation(witnessId: string): Promise<void> {
		try {
			const relation = await this.service.getMyRelation(witnessId);
			this.relations = new SvelteMap(this.relations).set(witnessId, relation);
		} catch (err) {
			this.setError(witnessId, this.toErrorMessage(err, 'Gagal memuat status sinyal'));
			throw err;
		}
	}

	async loadCounts(witnessId: string): Promise<void> {
		try {
			const counts = await this.service.getSignalCounts(witnessId);
			this.counts = new SvelteMap(this.counts).set(witnessId, counts);
		} catch (err) {
			this.setError(witnessId, this.toErrorMessage(err, 'Gagal memuat hitungan sinyal'));
			throw err;
		}
	}

	async loadResolutions(witnessId: string): Promise<void> {
		try {
			const resolved = await this.service.getResolutions(witnessId);
			this.resolutions = new SvelteMap(this.resolutions).set(witnessId, resolved);
		} catch (err) {
			this.setError(witnessId, this.toErrorMessage(err, 'Gagal memuat riwayat sinyal'));
			throw err;
		}
	}

	async refreshWitness(witnessId: string): Promise<void> {
		this.clearError(witnessId);
		await Promise.all([this.loadRelation(witnessId), this.loadCounts(witnessId)]);
		this.clearError(witnessId);
	}

	// ---------------------------------------------------------------------------
	// Actions — send / remove signals
	// ---------------------------------------------------------------------------

	async sendSignal(
		witnessId: string,
		signalType: ContentSignalType
	): Promise<ContentSignal | null> {
		if (this.sending) return null;
		this.sending = true;
		this.clearError(witnessId);

		try {
			const signal = await this.service.sendSignal(witnessId, signalType);

			// Optimistic update: refresh relation and counts from service
			await this.refreshWitness(witnessId);

			return signal;
		} catch (err) {
			this.setError(witnessId, this.toErrorMessage(err, 'Gagal mengirim sinyal'));
			throw err;
		} finally {
			this.sending = false;
		}
	}

	async removeSignal(witnessId: string, signalType: ContentSignalType): Promise<void> {
		if (this.sending) return;
		this.sending = true;
		this.clearError(witnessId);

		try {
			await this.service.removeSignal(witnessId, signalType);

			// Refresh relation and counts from service
			await this.refreshWitness(witnessId);
		} catch (err) {
			this.setError(witnessId, this.toErrorMessage(err, 'Gagal membatalkan sinyal'));
			throw err;
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
		const isActive = signalType === 'saksi' ? relation?.witnessed : relation?.flagged;
		if (this.sending) return Boolean(isActive);

		if (isActive) {
			await this.removeSignal(witnessId, signalType);
			return false;
		} else {
			const signal = await this.sendSignal(witnessId, signalType);
			return signal ? true : Boolean(isActive);
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
		this.resolutions = new SvelteMap(this.resolutions).set(witnessId, [
			...(this.resolutions.get(witnessId) ?? []),
			...resolved
		]);
		return resolved;
	}
}

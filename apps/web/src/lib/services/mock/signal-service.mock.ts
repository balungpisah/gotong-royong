import type { SignalService } from '../types';
import type {
	ContentSignal,
	ContentSignalType,
	MyRelation,
	SignalCounts,
	WitnessCloseReason,
	SignalResolutionOutcome
} from '$lib/types';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/**
 * Resolution matrix: maps (signal_type × close_reason) → outcome.
 * See spec 13a-signal-completion-resolution.md §3.
 */
function resolveOutcome(
	signalType: ContentSignalType,
	closeReason: WitnessCloseReason
): SignalResolutionOutcome {
	if (closeReason === 'selesai') {
		// Witness resolved successfully — positive signals rewarded
		return signalType === 'perlu_dicek' ? 'resolved_negative' : 'resolved_positive';
	}
	if (closeReason === 'tidak_valid') {
		// Content was false — doubt signals rewarded
		return signalType === 'perlu_dicek' ? 'resolved_positive' : 'resolved_negative';
	}
	// duplikat, kedaluwarsa, ditarik → neutral
	return 'resolved_neutral';
}

/** Base credit points per signal type. */
const BASE_POINTS: Record<ContentSignalType, number> = {
	saksi: 5,
	perlu_dicek: 4
};

export class MockSignalService implements SignalService {
	/** Per-witness relation state (keyed by witness_id). */
	private relations = new Map<string, MyRelation>();

	/** Per-witness signal counts (keyed by witness_id). */
	private counts = new Map<string, SignalCounts>();

	/** All signals cast (for resolution tracking). */
	private signals: ContentSignal[] = [];

	// ── Helpers ──────────────────────────────────────────────────

	private ensureRelation(witnessId: string): MyRelation {
		if (!this.relations.has(witnessId)) {
			this.relations.set(witnessId, {
				vouched: false,
				witnessed: false,
				flagged: false,
				supported: false
			});
		}
		return this.relations.get(witnessId)!;
	}

	private ensureCounts(witnessId: string): SignalCounts {
		if (!this.counts.has(witnessId)) {
			this.counts.set(witnessId, {
				vouch_positive: 0,
				vouch_skeptical: 0,
				witness_count: 0,
				dukung_count: 0,
				flags: 0
			});
		}
		return this.counts.get(witnessId)!;
	}

	// ── Public API ───────────────────────────────────────────────

	async sendSignal(witnessId: string, signalType: ContentSignalType): Promise<ContentSignal> {
		await delay(150);

		const relation = this.ensureRelation(witnessId);
		const counts = this.ensureCounts(witnessId);

		// Update relation
		if (signalType === 'saksi') {
			relation.witnessed = true;
			counts.witness_count++;
		} else if (signalType === 'perlu_dicek') {
			relation.flagged = true;
			counts.flags++;
		}

		const signal: ContentSignal = {
			signal_id: `sig-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
			witness_id: witnessId,
			user_id: 'u-001',
			signal_type: signalType,
			outcome: 'pending',
			created_at: new Date().toISOString()
		};

		this.signals.push(signal);
		return signal;
	}

	async removeSignal(witnessId: string, signalType: ContentSignalType): Promise<void> {
		await delay(100);

		const relation = this.ensureRelation(witnessId);
		const counts = this.ensureCounts(witnessId);

		if (signalType === 'saksi') {
			relation.witnessed = false;
			counts.witness_count = Math.max(0, counts.witness_count - 1);
		} else if (signalType === 'perlu_dicek') {
			relation.flagged = false;
			counts.flags = Math.max(0, counts.flags - 1);
		}

		// Remove the pending signal
		this.signals = this.signals.filter(
			(s) =>
				!(s.witness_id === witnessId && s.signal_type === signalType && s.outcome === 'pending')
		);
	}

	async getMyRelation(witnessId: string): Promise<MyRelation> {
		await delay(100);
		return this.ensureRelation(witnessId);
	}

	async getSignalCounts(witnessId: string): Promise<SignalCounts> {
		await delay(100);
		return this.ensureCounts(witnessId);
	}

	async getResolutions(witnessId: string): Promise<ContentSignal[]> {
		await delay(100);
		return this.signals.filter((s) => s.witness_id === witnessId && s.outcome !== 'pending');
	}

	/**
	 * Mock-only: resolve all pending signals for a witness.
	 * Called by MockWitnessService when a witness reaches terminal state.
	 */
	async simulateResolution(
		witnessId: string,
		closeReason: WitnessCloseReason
	): Promise<ContentSignal[]> {
		await delay(200);

		const resolved: ContentSignal[] = [];
		const now = new Date().toISOString();

		for (const signal of this.signals) {
			if (signal.witness_id === witnessId && signal.outcome === 'pending') {
				const outcome = resolveOutcome(signal.signal_type, closeReason);
				const sign =
					outcome === 'resolved_positive' ? 1 : outcome === 'resolved_negative' ? -0.5 : 0;

				signal.outcome = outcome;
				signal.resolved_at = now;
				signal.credit_delta = BASE_POINTS[signal.signal_type] * sign;

				resolved.push(signal);
			}
		}

		return resolved;
	}

	/**
	 * Seed initial relation/counts for a witness (used by fixtures).
	 */
	seed(witnessId: string, relation: MyRelation, counts: SignalCounts): void {
		this.relations.set(witnessId, { ...relation });
		this.counts.set(witnessId, { ...counts });
	}
}

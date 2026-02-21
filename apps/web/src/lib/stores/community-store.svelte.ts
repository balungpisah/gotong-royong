/**
 * Community Store — manages community health stats and activity.
 *
 * Uses Svelte 5 runes ($state) for reactive state management.
 * Currently mock-backed — will be swapped to a CommunityService when backend is ready.
 * Follows the FeedStore mock-backed pattern (no service injection).
 */

import type {
	CommunityStats,
	ParticipationDataPoint,
	CommunitySignalSummary,
	CommunityActivityItem
} from '$lib/types';
import {
	mockCommunityStats,
	mockParticipation,
	mockCommunitySignals,
	mockCommunityActivity
} from '$lib/fixtures';

export class CommunityStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	stats = $state<CommunityStats | null>(null);
	participation = $state<ParticipationDataPoint[]>([]);
	signals = $state<CommunitySignalSummary | null>(null);
	recentActivity = $state<CommunityActivityItem[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	hasData = $derived(this.stats !== null);
	signalTotal = $derived(
		this.signals
			? this.signals.vouch + this.signals.skeptis + this.signals.proof_of_resolve +
				this.signals.bagus + this.signals.perlu_dicek
			: 0
	);

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/**
	 * Load community data. Currently returns mock data.
	 * TODO: Replace with CommunityService when backend is ready.
	 */
	async loadCommunityData() {
		this.loading = true;
		this.error = null;
		try {
			// Simulate network delay
			await new Promise((resolve) => setTimeout(resolve, 300));
			this.stats = { ...mockCommunityStats };
			this.participation = [...mockParticipation];
			this.signals = { ...mockCommunitySignals };
			this.recentActivity = [...mockCommunityActivity];
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat data komunitas';
		} finally {
			this.loading = false;
		}
	}

	/** Reload community data. */
	async refresh() {
		await this.loadCommunityData();
	}
}

/**
 * Community Store — manages community health stats and activity.
 *
 * Uses Svelte 5 runes ($state) for reactive state management.
 * Dashboard path is API-backed via CommunityService.
 * Compact pulse cards still use fixtures until dedicated pulse contract lands.
 */

import type { CommunityService } from '$lib/services/types';
import type {
	CommunityStats,
	ParticipationDataPoint,
	CommunitySignalSummary,
	CommunityActivityItem,
	CommunityDashboard
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

	// Full community dashboard (for Komunitas page)
	dashboard = $state<CommunityDashboard | null>(null);
	dashboardLoading = $state(false);
	dashboardError = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	hasData = $derived(this.stats !== null);
	hasDashboard = $derived(this.dashboard !== null);
	weather = $derived(this.dashboard?.weather ?? null);
	signalTotal = $derived(
		this.signals
			? this.signals.vouch +
					this.signals.skeptis +
					this.signals.proof_of_resolve +
					this.signals.dukung +
					this.signals.perlu_dicek
			: 0
	);

	private readonly service: CommunityService;

	constructor(service: CommunityService) {
		this.service = service;
	}

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

	/**
	 * Load full community dashboard data. Currently returns mock data.
	 */
	async loadDashboard() {
		this.dashboardLoading = true;
		this.dashboardError = null;
		try {
			this.dashboard = await this.service.getDashboard();
		} catch (err) {
			this.dashboardError = err instanceof Error ? err.message : 'Gagal memuat dashboard komunitas';
		} finally {
			this.dashboardLoading = false;
		}
	}
}

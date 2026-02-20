/**
 * Preferences Store — manages user UI preferences with localStorage persistence.
 *
 * Uses Svelte 5 runes ($state) for reactive state management.
 * Future-proof: add more preferences here (compact mode, animations, etc.)
 */

import { browser } from '$app/environment';

const STORAGE_KEY = 'gr-preferences';

interface PreferencesData {
	showTooltips: boolean;
}

const DEFAULTS: PreferencesData = {
	showTooltips: true
};

export class PreferencesStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	/** Whether tooltips are shown on icon-only buttons. */
	showTooltips = $state(DEFAULTS.showTooltips);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	constructor() {
		if (browser) {
			try {
				const raw = localStorage.getItem(STORAGE_KEY);
				if (raw) {
					const parsed = JSON.parse(raw) as Partial<PreferencesData>;
					if (typeof parsed.showTooltips === 'boolean') {
						this.showTooltips = parsed.showTooltips;
					}
				}
			} catch {
				// Corrupted data — use defaults
			}
		}

		// Persist whenever state changes
		$effect(() => {
			this.persist();
		});
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/** Toggle tooltips on/off. */
	toggleTooltips() {
		this.showTooltips = !this.showTooltips;
	}

	// ---------------------------------------------------------------------------
	// Private helpers
	// ---------------------------------------------------------------------------

	private persist() {
		if (!browser) return;
		const data: PreferencesData = {
			showTooltips: this.showTooltips
		};
		localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
	}
}

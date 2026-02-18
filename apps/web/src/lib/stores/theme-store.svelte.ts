/**
 * Theme Store — manages dark/light mode with localStorage persistence.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * The `.dark` class is toggled on <html> to activate the dark theme
 * defined in app.css.
 */

import { browser } from '$app/environment';

export type ThemeMode = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'gr-theme';

export class ThemeStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	/** User preference: 'light', 'dark', or 'system' (follow OS). */
	mode = $state<ThemeMode>('light');

	/** The resolved active theme after evaluating system preference. */
	resolved = $derived<'light' | 'dark'>(this.resolveTheme());

	// ---------------------------------------------------------------------------
	// Private
	// ---------------------------------------------------------------------------

	private systemDark = $state(false);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	constructor() {
		if (browser) {
			// Read persisted preference
			const stored = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
			if (stored === 'light' || stored === 'dark' || stored === 'system') {
				this.mode = stored;
			}

			// Detect system preference
			const mq = window.matchMedia('(prefers-color-scheme: dark)');
			this.systemDark = mq.matches;
			mq.addEventListener('change', (e) => {
				this.systemDark = e.matches;
			});
		}

		// Apply theme class whenever resolved changes
		$effect(() => {
			this.applyThemeClass(this.resolved);
		});
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	/** Set the theme mode and persist to localStorage. */
	setMode(mode: ThemeMode) {
		this.mode = mode;
		if (browser) {
			localStorage.setItem(STORAGE_KEY, mode);
		}
	}

	/** Cycle through modes: light → dark → system → light. */
	toggle() {
		const next: Record<ThemeMode, ThemeMode> = {
			light: 'dark',
			dark: 'system',
			system: 'light'
		};
		this.setMode(next[this.mode]);
	}

	/** Quick toggle between light and dark only (skip system). */
	toggleSimple() {
		this.setMode(this.resolved === 'light' ? 'dark' : 'light');
	}

	// ---------------------------------------------------------------------------
	// Private helpers
	// ---------------------------------------------------------------------------

	private resolveTheme(): 'light' | 'dark' {
		if (this.mode === 'system') {
			return this.systemDark ? 'dark' : 'light';
		}
		return this.mode;
	}

	private applyThemeClass(theme: 'light' | 'dark') {
		if (!browser) return;
		const html = document.documentElement;
		if (theme === 'dark') {
			html.classList.add('dark');
		} else {
			html.classList.remove('dark');
		}
	}
}

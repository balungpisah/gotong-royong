/**
 * User Store — manages current user session and profile state.
 *
 * Uses Svelte 5 runes ($state, $derived) for reactive state management.
 * Consumes UserService interface for data operations.
 */

import type { UserService } from '$lib/services/types';
import type { UserProfile } from '$lib/types';

export class UserStore {
	// ---------------------------------------------------------------------------
	// State
	// ---------------------------------------------------------------------------

	profile = $state<UserProfile | null>(null);
	loading = $state(false);
	error = $state<string | null>(null);

	// ---------------------------------------------------------------------------
	// Derived
	// ---------------------------------------------------------------------------

	isAuthenticated = $derived(this.profile !== null);
	displayName = $derived(this.profile?.name ?? 'Pengguna');
	userRole = $derived(this.profile?.role ?? 'anonymous');
	userTier = $derived(this.profile?.tier ?? 0);

	// ---------------------------------------------------------------------------
	// Constructor
	// ---------------------------------------------------------------------------

	private readonly service: UserService;

	constructor(service: UserService) {
		this.service = service;
	}

	// ---------------------------------------------------------------------------
	// Actions
	// ---------------------------------------------------------------------------

	async loadCurrentUser() {
		this.loading = true;
		this.error = null;
		try {
			this.profile = await this.service.getCurrentUser();
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat profil pengguna';
		} finally {
			this.loading = false;
		}
	}

	async loadProfile(userId: string) {
		this.loading = true;
		this.error = null;
		try {
			this.profile = await this.service.getProfile(userId);
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Gagal memuat profil';
		} finally {
			this.loading = false;
		}
	}

	logout() {
		this.profile = null;
		this.error = null;
	}
}

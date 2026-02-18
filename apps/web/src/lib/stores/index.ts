/**
 * Stores barrel — re-exports all stores and context helpers.
 *
 * Stores are Svelte 5 runes-based classes initialized in the root layout
 * and injected via setContext/getContext.
 */

import { getContext } from 'svelte';

// ---------------------------------------------------------------------------
// Store re-exports
// ---------------------------------------------------------------------------

export { WitnessStore } from './witness-store.svelte';
export { UserStore } from './user-store.svelte';
export { NotificationStore } from './notification-store.svelte';
export { TriageStore } from './triage-store.svelte';
export { NavigationStore } from './navigation-store.svelte';

// ---------------------------------------------------------------------------
// Import types for context helpers
// ---------------------------------------------------------------------------

import type { WitnessStore } from './witness-store.svelte';
import type { UserStore } from './user-store.svelte';
import type { NotificationStore } from './notification-store.svelte';
import type { TriageStore } from './triage-store.svelte';
import type { NavigationStore } from './navigation-store.svelte';

// ---------------------------------------------------------------------------
// Context keys
// ---------------------------------------------------------------------------

export const WITNESS_STORE_KEY = Symbol('witness-store');
export const USER_STORE_KEY = Symbol('user-store');
export const NOTIFICATION_STORE_KEY = Symbol('notification-store');
export const TRIAGE_STORE_KEY = Symbol('triage-store');
export const NAVIGATION_STORE_KEY = Symbol('navigation-store');

// ---------------------------------------------------------------------------
// Typed context getters (for use in components)
// ---------------------------------------------------------------------------

/** Get the WitnessStore from component context. */
export function getWitnessStore(): WitnessStore {
	return getContext<WitnessStore>(WITNESS_STORE_KEY);
}

/** Get the UserStore from component context. */
export function getUserStore(): UserStore {
	return getContext<UserStore>(USER_STORE_KEY);
}

/** Get the NotificationStore from component context. */
export function getNotificationStore(): NotificationStore {
	return getContext<NotificationStore>(NOTIFICATION_STORE_KEY);
}

/** Get the TriageStore from component context. */
export function getTriageStore(): TriageStore {
	return getContext<TriageStore>(TRIAGE_STORE_KEY);
}

/** Get the NavigationStore from component context. */
export function getNavigationStore(): NavigationStore {
	return getContext<NavigationStore>(NAVIGATION_STORE_KEY);
}

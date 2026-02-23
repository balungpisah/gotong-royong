/**
 * Service factory — provides mock or real service implementations.
 *
 * Toggle USE_MOCK to false when the Rust Axum backend is ready.
 * The factory returns a ServiceProvider containing all services.
 */

import type {
	WitnessService,
	UserService,
	TriageService,
	NotificationService,
	SignalService,
	GroupService
} from './types';
import {
	MockWitnessService,
	MockUserService,
	MockTriageService,
	MockNotificationService,
	MockSignalService,
	MockGroupService
} from './mock';

// Re-export types for convenience
export type {
	WitnessService,
	UserService,
	TriageService,
	NotificationService,
	SignalService,
	GroupService,
	Paginated
} from './types';

// ---------------------------------------------------------------------------
// Service Provider
// ---------------------------------------------------------------------------

/**
 * Container for all service instances.
 * Injected via setContext in the root layout.
 */
export interface ServiceProvider {
	witness: WitnessService;
	user: UserService;
	triage: TriageService;
	notification: NotificationService;
	signal: SignalService;
	group: GroupService;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/** Toggle to false when the Rust Axum backend at /v1 is ready. */
const USE_MOCK = true;

/**
 * Creates the service provider with mock or real implementations.
 */
export function createServices(): ServiceProvider {
	if (USE_MOCK) {
		return {
			witness: new MockWitnessService(),
			user: new MockUserService(),
			triage: new MockTriageService(),
			notification: new MockNotificationService(),
			signal: new MockSignalService(),
			group: new MockGroupService()
		};
	}

	// Future: return real API implementations using apiClient from $lib/api
	// import { apiClient } from '$lib/api';
	// return {
	//   witness: new ApiWitnessService(apiClient),
	//   user: new ApiUserService(apiClient),
	//   triage: new ApiTriageService(apiClient),
	//   notification: new ApiNotificationService(apiClient),
	// };
	throw new Error('API services not yet implemented. Set USE_MOCK = true.');
}

// ---------------------------------------------------------------------------
// Context key for DI
// ---------------------------------------------------------------------------

/** Symbol key for injecting ServiceProvider via setContext/getContext. */
export const SERVICES_KEY = Symbol('services');

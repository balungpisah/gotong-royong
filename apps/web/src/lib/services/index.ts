/**
 * Service factory — provides mock or real service implementations.
 *
 * Hot-path domains (feed/chat/notifications) + user/triage/signal/group are API-first by default.
 * Set PUBLIC_GR_USE_API_* to false for isolated mock-only frontend runs in dev/test.
 */

import { dev } from '$app/environment';
import { env } from '$env/dynamic/public';
import { apiClient } from '$lib/api';
import type {
	WitnessService,
	UserService,
	TriageService,
	NotificationService,
	FeedService,
	SignalService,
	GroupService,
	CommunityService
} from './types';
import {
	MockWitnessService,
	MockUserService,
	MockTriageService,
	MockNotificationService,
	MockFeedService,
	MockSignalService,
	MockGroupService,
	MockCommunityService
} from './mock';
import {
	ApiFeedService,
	ApiNotificationService,
	ApiGroupService,
	ApiSignalService,
	ApiTriageService,
	ApiUserService,
	ApiWitnessService,
	ApiCommunityService
} from './api';

// Re-export types for convenience
export type {
	WitnessService,
	UserService,
	TriageService,
	NotificationService,
	FeedService,
	SignalService,
	GroupService,
	CommunityService,
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
	feed: FeedService;
	signal: SignalService;
	group: GroupService;
	community: CommunityService;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

const parseApiToggle = (value: string | undefined, defaultValue = true) => {
	if (value === undefined || value.trim() === '') {
		return defaultValue;
	}

	const normalized = value.trim().toLowerCase();
	if (normalized === 'true' || normalized === '1' || normalized === 'yes') {
		return true;
	}
	if (normalized === 'false' || normalized === '0' || normalized === 'no') {
		return false;
	}

	return defaultValue;
};

const IS_PRODUCTION_RUNTIME = !dev;
const ALLOW_MOCK_FALLBACK = !IS_PRODUCTION_RUNTIME;

const USE_API_NOTIFICATIONS = parseApiToggle(env.PUBLIC_GR_USE_API_NOTIFICATIONS, true);
const USE_API_FEED = parseApiToggle(env.PUBLIC_GR_USE_API_FEED, true);
const USE_API_CHAT = parseApiToggle(env.PUBLIC_GR_USE_API_CHAT, true);
const USE_API_USER = parseApiToggle(env.PUBLIC_GR_USE_API_USER, true);
const USE_API_TRIAGE = parseApiToggle(env.PUBLIC_GR_USE_API_TRIAGE, true);
const USE_API_SIGNAL = parseApiToggle(env.PUBLIC_GR_USE_API_SIGNAL, true);
const USE_API_GROUP = parseApiToggle(env.PUBLIC_GR_USE_API_GROUP, true);
const USE_API_COMMUNITY = parseApiToggle(env.PUBLIC_GR_USE_API_COMMUNITY, true);

const assertApiEnabledInProduction = (slice: string, enabled: boolean, envKey: string) => {
	if (IS_PRODUCTION_RUNTIME && !enabled) {
		throw new Error(
			`Production runtime requires API-backed ${slice} service. Remove ${envKey}=false.`
		);
	}
};

assertApiEnabledInProduction(
	'notifications',
	USE_API_NOTIFICATIONS,
	'PUBLIC_GR_USE_API_NOTIFICATIONS'
);
assertApiEnabledInProduction('feed', USE_API_FEED, 'PUBLIC_GR_USE_API_FEED');
assertApiEnabledInProduction('chat', USE_API_CHAT, 'PUBLIC_GR_USE_API_CHAT');
assertApiEnabledInProduction('user', USE_API_USER, 'PUBLIC_GR_USE_API_USER');
assertApiEnabledInProduction('triage', USE_API_TRIAGE, 'PUBLIC_GR_USE_API_TRIAGE');
assertApiEnabledInProduction('signal', USE_API_SIGNAL, 'PUBLIC_GR_USE_API_SIGNAL');
assertApiEnabledInProduction('group', USE_API_GROUP, 'PUBLIC_GR_USE_API_GROUP');
assertApiEnabledInProduction('community', USE_API_COMMUNITY, 'PUBLIC_GR_USE_API_COMMUNITY');

/**
 * Creates the service provider with mock or real implementations.
 */
export function createServices(): ServiceProvider {
	const mockWitness = new MockWitnessService();
	const mockUser = new MockUserService();
	const mockTriage = new MockTriageService();
	const mockSignal = new MockSignalService();
	const mockGroup = new MockGroupService();
	const mockCommunity = new MockCommunityService();

	return {
		witness: USE_API_CHAT
			? new ApiWitnessService(apiClient, mockWitness, { allowMockFallback: ALLOW_MOCK_FALLBACK })
			: mockWitness,
		user: USE_API_USER
			? new ApiUserService(apiClient, mockUser, { allowMockFallback: ALLOW_MOCK_FALLBACK })
			: mockUser,
		triage: USE_API_TRIAGE
			? new ApiTriageService(apiClient, mockTriage, { allowMockFallback: ALLOW_MOCK_FALLBACK })
			: mockTriage,
		notification: USE_API_NOTIFICATIONS
			? new ApiNotificationService(apiClient)
			: new MockNotificationService(),
		feed: USE_API_FEED ? new ApiFeedService(apiClient) : new MockFeedService(),
		signal: USE_API_SIGNAL
			? new ApiSignalService(apiClient, mockSignal, { allowMockFallback: ALLOW_MOCK_FALLBACK })
			: mockSignal,
		group: USE_API_GROUP
			? new ApiGroupService(apiClient, mockGroup, { allowMockFallback: ALLOW_MOCK_FALLBACK })
			: mockGroup,
		community: USE_API_COMMUNITY ? new ApiCommunityService(apiClient) : mockCommunity
	};
}

// ---------------------------------------------------------------------------
// Context key for DI
// ---------------------------------------------------------------------------

/** Symbol key for injecting ServiceProvider via setContext/getContext. */
export const SERVICES_KEY = Symbol('services');

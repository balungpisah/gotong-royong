/**
 * Service layer interfaces — contracts between stores and data providers.
 *
 * Each service has a mock implementation (for development) and will
 * eventually have a real API implementation using the ApiClient.
 */

import type {
	Witness,
	WitnessDetail,
	WitnessStatus,
	ChatMessage,
	PathPlan,
	DiffResponse,
	UserProfile,
	AppNotification,
	TriageResult
} from '$lib/types';

// ---------------------------------------------------------------------------
// Paginated response envelope
// ---------------------------------------------------------------------------

/**
 * Generic paginated response for list endpoints.
 * Supports cursor-based pagination.
 */
export interface Paginated<T> {
	items: T[];
	total: number;
	cursor?: string;
}

// ---------------------------------------------------------------------------
// Witness Service
// ---------------------------------------------------------------------------

export interface WitnessService {
	/** List witnesses with optional filters. */
	list(opts?: {
		status?: WitnessStatus;
		cursor?: string;
		limit?: number;
	}): Promise<Paginated<Witness>>;

	/** Get full witness detail by ID. */
	get(witnessId: string): Promise<WitnessDetail>;

	/** Get messages for a witness with pagination. */
	getMessages(
		witnessId: string,
		opts?: { cursor?: string; limit?: number }
	): Promise<Paginated<ChatMessage>>;

	/** Send a new message in a witness thread. */
	sendMessage(witnessId: string, content: string, attachments?: File[]): Promise<ChatMessage>;

	/** Get the path plan for a witness. */
	getPlan(witnessId: string): Promise<PathPlan | null>;

	/** Respond to a diff card suggestion. */
	respondToDiff(witnessId: string, diffId: string, response: DiffResponse): Promise<void>;

	/** Cast a vote on a vote card. */
	castVote(witnessId: string, voteId: string, optionId: string): Promise<void>;
}

// ---------------------------------------------------------------------------
// User Service
// ---------------------------------------------------------------------------

export interface UserService {
	/** Get a user profile by ID. */
	getProfile(userId: string): Promise<UserProfile>;

	/** Get the currently authenticated user's profile. */
	getCurrentUser(): Promise<UserProfile>;
}

// ---------------------------------------------------------------------------
// Triage Service
// ---------------------------------------------------------------------------

export interface TriageService {
	/** Start a new AI-00 triage session with initial content. */
	startTriage(content: string): Promise<TriageResult>;

	/** Continue a triage session with a follow-up answer. */
	updateTriage(sessionId: string, answer: string): Promise<TriageResult>;
}

// ---------------------------------------------------------------------------
// Notification Service
// ---------------------------------------------------------------------------

export interface NotificationService {
	/** List notifications with pagination. */
	list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<AppNotification>>;

	/** Mark a single notification as read. */
	markRead(notificationId: string): Promise<void>;

	/** Mark all notifications as read. */
	markAllRead(): Promise<void>;

	/** Get the count of unread notifications. */
	getUnreadCount(): Promise<number>;
}

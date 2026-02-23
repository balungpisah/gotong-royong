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
	WitnessCreateInput,
	ChatMessage,
	PathPlan,
	DiffResponse,
	UserProfile,
	AppNotification,
	TriageResult,
	TandangProfile,
	ContentSignal,
	ContentSignalType,
	MyRelation,
	SignalCounts,
	WitnessCloseReason,
	GroupSummary,
	GroupDetail,
	GroupCreateInput,
	GroupUpdateInput,
	GroupMember,
	MembershipRequest,
	GroupMemberRole
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
	/** Create a new witness from triage results. */
	create(input: WitnessCreateInput): Promise<WitnessDetail>;

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

	/** Get the full tandang-enriched profile for the CV Hidup page. */
	getTandangProfile(userId: string): Promise<TandangProfile>;

	/** Get the current user's tandang-enriched profile. */
	getCurrentTandangProfile(): Promise<TandangProfile>;
}

// ---------------------------------------------------------------------------
// Triage Service
// ---------------------------------------------------------------------------

export interface TriageService {
	/** Start a new AI-00 triage session with initial content. */
	startTriage(content: string, attachments?: File[]): Promise<TriageResult>;

	/** Continue a triage session with a follow-up answer. */
	updateTriage(sessionId: string, answer: string, attachments?: File[]): Promise<TriageResult>;
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

// ---------------------------------------------------------------------------
// Signal Service (AI-09a: Signal Completion Resolution)
// ---------------------------------------------------------------------------

export interface SignalService {
	/** Cast a content-directed signal on a witness. */
	sendSignal(witnessId: string, signalType: ContentSignalType): Promise<ContentSignal>;

	/** Remove a previously cast signal (undo). */
	removeSignal(witnessId: string, signalType: ContentSignalType): Promise<void>;

	/** Get current user's relation to a witness. */
	getMyRelation(witnessId: string): Promise<MyRelation>;

	/** Get aggregate signal counts for a witness. */
	getSignalCounts(witnessId: string): Promise<SignalCounts>;

	/** Get signal resolution history for a witness (after completion). */
	getResolutions(witnessId: string): Promise<ContentSignal[]>;

	/** Resolve all pending signals when witness reaches terminal state. Mock-only. */
	simulateResolution?(witnessId: string, closeReason: WitnessCloseReason): Promise<ContentSignal[]>;
}

// ---------------------------------------------------------------------------
// Group Service (Kelompok / Lembaga)
// ---------------------------------------------------------------------------

export interface GroupService {
	create(input: GroupCreateInput): Promise<GroupDetail>;

	list(opts?: { cursor?: string; limit?: number }): Promise<Paginated<GroupSummary>>;

	listMyGroups(): Promise<GroupSummary[]>;

	get(groupId: string): Promise<GroupDetail>;

	update(groupId: string, input: GroupUpdateInput): Promise<GroupDetail>;

	/** Join immediately (only for join_policy: 'terbuka'). */
	join(groupId: string): Promise<GroupMember>;

	/** Request to join (only for join_policy: 'persetujuan'). */
	requestJoin(groupId: string, message?: string): Promise<MembershipRequest>;

	approveRequest(groupId: string, requestId: string): Promise<GroupMember>;
	rejectRequest(groupId: string, requestId: string): Promise<void>;

	/** Invite a user by ID (admin-only). */
	invite(groupId: string, userId: string): Promise<void>;

	leave(groupId: string): Promise<void>;
	removeMember(groupId: string, userId: string): Promise<void>;
	updateMemberRole(groupId: string, userId: string, role: GroupMemberRole): Promise<void>;
}

/**
 * Feed domain types — event-based activity stream for Pulse.
 *
 * The Pulse feed is an event stream, not a case list. Each witness
 * appears once, showing its latest significant event as the headline.
 * Feed items come from 3 layers: Ikutan (followed), Terlibat (participating),
 * and Sekitar (nearby/trending).
 */

import type { WitnessStatus, WitnessMemberRole } from './witness';
import type { RahasiaLevel } from './triage';

// ── Feed Event Types ──────────────────────────────────────────────

/** The ~8 significant event types that appear in feed cards. */
export type FeedEventType =
	| 'created'
	| 'joined'
	| 'checkpoint'
	| 'vote_opened'
	| 'evidence'
	| 'resolved'
	| 'galang_milestone'
	| 'community_note';

/** A single feed event — the latest one becomes the card headline. */
export interface FeedEvent {
	event_id: string;
	event_type: FeedEventType;
	actor_name: string;
	actor_avatar?: string;
	actor_role?: WitnessMemberRole;
	timestamp: string;
	/** Short verb text, e.g. "menambah bukti", "bergabung sebagai Relawan" */
	verb: string;
	/** Optional snippet — varies by event type */
	snippet?: string;
}

// ── Feed Item (one per witness in the feed) ───────────────────────

/** Urgency badge type for visual priority. */
export type UrgencyBadge = 'baru' | 'voting' | 'selesai' | 'ramai';

/** Feed layer source. */
export type FeedSource = 'ikutan' | 'terlibat' | 'sekitar';

/** Feed filter tab values. */
export type FeedFilter = 'semua' | 'ikutan' | 'terlibat' | 'sekitar';

/** A single feed card — one per witness, latest event as headline. */
export interface FeedItem {
	witness_id: string;
	title: string;
	track_hint?: string;
	status: WitnessStatus;
	rahasia_level: RahasiaLevel;
	latest_event: FeedEvent;
	collapsed_count: number;
	member_count: number;
	members_preview: FeedMemberPreview[];
	entity_tags: EntityTag[];
	urgency?: UrgencyBadge;
	source: FeedSource;
	repost?: RepostFrame;
}

/** Preview of a witness member for the avatar stack (max 5). */
export interface FeedMemberPreview {
	user_id: string;
	name: string;
	avatar_url?: string;
	role: WitnessMemberRole;
}

// ── Repost Frame (brag rights) ────────────────────────────────────

/** When a user's followers see their contribution framed through their role. */
export interface RepostFrame {
	reposter_name: string;
	reposter_avatar?: string;
	reposter_role: WitnessMemberRole;
	/** e.g. "melaporkan", "bergabung sebagai Relawan", "menambah bukti" */
	action_verb: string;
}

// ── Followable Entities (Ikutan) ──────────────────────────────────

/** The 5 followable entity types. */
export type EntityType = 'lingkungan' | 'topik' | 'kelompok' | 'lembaga' | 'warga';

/** Compact entity reference shown as a pill on feed cards. */
export interface EntityTag {
	entity_id: string;
	entity_type: EntityType;
	label: string;
	followed: boolean;
}

/** Full entity detail for suggestion cards and entity pages. */
export interface FollowableEntity extends EntityTag {
	description?: string;
	witness_count: number;
	follower_count: number;
}

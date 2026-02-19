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

// ── Tandang Signal Types (Phase 2) ────────────────────────────────

/** The 5 explicit chip types shown on feed cards.
 *  Each maps to a tandang reputation signal (I/C/J).
 *  Contextual PoR wording is resolved at render time based on card type. */
export type SignalChipType =
	| 'vouch'       // 🤝 Saya Vouch — positive trust signal → I+C
	| 'skeptis'     // 🤔 Skeptis — healthy doubt signal → J
	| 'saksi'        // 👁️ PoR chip — contextual: Saya Saksi / Sudah Beres / Bukti Valid → I
	| 'bagus'       // 👍 Bagus — quality upvote → C
	| 'perlu_dicek' // ⚠️ Perlu Dicek — quality flag → I+J
	| 'inline_vote'; // 🗳️ Ya/Tidak — inline voting (vote_opened cards only)

/** Current user's relation to this witness/entity.
 *  Populated from tandang query: GET /user/{uid}/relation/{entity_id} */
export interface MyRelation {
	vouched: boolean;
	vouch_type?: 'positive' | 'skeptical' | 'conditional' | 'mentorship';
	witnessed: boolean;
	flagged: boolean;
	quality_voted: boolean;
	vote_cast?: 'yes' | 'no';
}

/** Aggregate signal counts for social proof display.
 *  Populated from tandang query: GET /entity/{id}/signals */
export interface SignalCounts {
	vouch_positive: number;
	vouch_skeptical: number;
	witness_count: number;
	quality_avg: number;
	quality_votes: number;
	flags: number;
}

// ── Feed Item (one per witness in the feed) ───────────────────────

/** Urgency badge type for visual priority. */
export type UrgencyBadge = 'baru' | 'voting' | 'selesai' | 'ramai';

/** Feed layer source. */
export type FeedSource = 'ikutan' | 'terlibat' | 'sekitar';

/** Feed filter tab values. */
export type FeedFilter = 'semua' | 'ikutan' | 'terlibat' | 'sekitar' | 'discover';

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

	// ── LLM-enriched card fields (extracted during triage) ──────
	/** The hook — a punchy editorial 1-liner that makes the reader curious. */
	hook_line?: string;
	/** The most emotionally resonant sentence from the conversation. */
	pull_quote?: string;
	/** Emotional mood for visual styling. */
	sentiment?: 'angry' | 'hopeful' | 'urgent' | 'celebratory' | 'sad' | 'curious' | 'fun';
	/** Conversation heat level (1–5). */
	intensity?: number;

	// ── Rich media & narrative ───────────────────────────────────
	/** Cover image URL — photo evidence, location shot, or community photo. */
	cover_url?: string;
	/** AI-summarized narrative from the saksi conversation. Massaged for
	 *  civility while preserving emotional intensity. 2-4 sentences. */
	body?: string;

	// ── Engagement: Story Peek (Phase 3) ────────────────────────
	/** Recent conversation snippets for the auto-rotating peek strip. */
	peek_messages?: PeekMessage[];

	// ── Tandang Signals (Phase 2) ────────────────────────────────
	/** Current user's relation to this entity (from tandang). */
	my_relation?: MyRelation;
	/** Aggregate signal counts for social proof (from tandang). */
	signal_counts?: SignalCounts;

	// ── Engagement: Pulse & Urgency (Phase 1) ────────────────────
	/** Number of users currently active on this witness (last 30 min). */
	active_now?: number;
	/** Real deadline ISO timestamp — voting close, phase end, etc. */
	deadline?: string;
	/** Label explaining the deadline, e.g. "Voting ditutup", "Fase berakhir". */
	deadline_label?: string;
	/** Quorum: how many participants needed for a threshold. */
	quorum_target?: number;
	/** Quorum: how many participants currently. */
	quorum_current?: number;
}

/** Preview of a witness member for the avatar stack (max 5). */
export interface FeedMemberPreview {
	user_id: string;
	name: string;
	avatar_url?: string;
	role: WitnessMemberRole;
}

// ── Peek Messages (Phase 3 — Story Peek) ─────────────────────────

/** A single chat message shown in the auto-rotating peek strip. */
export interface PeekMessage {
	/** Display name of the message author. */
	author: string;
	/** Short message text (will be truncated to ~80 chars on card). */
	text: string;
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

// ── Polymorphic Feed Stream ──────────────────────────────────────

/** Base for all feed stream items. */
interface FeedStreamBase {
	/** Unique ID for keying in {#each}. */
	stream_id: string;
	/** Timestamp for sorting. */
	sort_timestamp: string;
}

/** A witness activity card (existing FeedItem, now tagged). */
export interface FeedWitnessItem extends FeedStreamBase {
	kind: 'witness';
	data: FeedItem;
}

/** An inline system card (suggestions, tips, announcements). */
export interface FeedSystemItem extends FeedStreamBase {
	kind: 'system';
	data: SystemCardData;
}

/** The polymorphic feed stream type. */
export type FeedStreamItem = FeedWitnessItem | FeedSystemItem;

// ── System Card Variants ─────────────────────────────────────────

export type SystemCardVariant = 'suggestion' | 'tip' | 'milestone' | 'prompt';

export interface SystemCardData {
	variant: SystemCardVariant;
	/** Icon emoji or Lucide icon name. */
	icon: string;
	/** Short headline. */
	title: string;
	/** Optional description. */
	description?: string;
	/** Dismissible? */
	dismissible: boolean;
	/** Variant-specific payload. */
	payload: SuggestionPayload | TipPayload | MilestonePayload | PromptPayload;
}

/** Entity suggestion — "Ikuti RT 05 Menteng". */
export interface SuggestionPayload {
	variant: 'suggestion';
	entities: FollowableEntity[];
}

/** Platform tip — "Tahukah kamu? Kamu bisa melampirkan bukti". */
export interface TipPayload {
	variant: 'tip';
	tip_id: string;
}

/** Community milestone — "10 saksi selesai bulan ini!". */
export interface MilestonePayload {
	variant: 'milestone';
	metric_label: string;
	metric_value: string;
}

/** Engagement prompt — "Belum ada laporan di sekitarmu minggu ini". */
export interface PromptPayload {
	variant: 'prompt';
	cta_label: string;
	cta_action: string;
}

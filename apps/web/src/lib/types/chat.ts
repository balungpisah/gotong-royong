/**
 * Chat Message Types — JSON contracts for the chat-first interaction model.
 *
 * The chat surface follows WhatsApp-style conventions with additional
 * inline cards for AI suggestions, diff cards, vote cards, and system
 * messages. All AI content renders via the 7 block primitives.
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §2.5, §2.6
 */

import type { Block, SourceTag } from './blocks';
import type { DiffCard } from './diff-card';

// ---------------------------------------------------------------------------
// Message Author
// ---------------------------------------------------------------------------

export interface MessageAuthor {
	user_id: string;
	name: string;
	avatar_url?: string;
	/** Reputation tier (0-4). */
	tier?: number;
	/** Role in this witness/case. */
	role?: string;
}

// ---------------------------------------------------------------------------
// Message Types
// ---------------------------------------------------------------------------

export type ChatMessageType =
	| 'user'        // Regular chat bubble (left or right aligned)
	| 'ai_card'     // AI inline card (suggestion, summary, alert)
	| 'diff_card'   // Diff card (suggest-don't-overwrite)
	| 'vote_card'   // Inline vote
	| 'system'      // System message (centered, muted)
	| 'evidence'    // Evidence submission card
	| 'galang';     // Financial transaction system message

// ---------------------------------------------------------------------------
// Base Message
// ---------------------------------------------------------------------------

export interface ChatMessageBase {
	message_id: string;
	/** ISO timestamp. */
	timestamp: string;
	/** Which witness/case this message belongs to. */
	witness_id: string;
}

// ---------------------------------------------------------------------------
// User Message
// ---------------------------------------------------------------------------

export interface UserMessage extends ChatMessageBase {
	type: 'user';
	author: MessageAuthor;
	/** Whether this is the current user's own message. */
	is_self: boolean;
	content: string;
	/** Optional media attachments (max 5 per spec). */
	attachments?: {
		type: 'image' | 'video';
		url: string;
		alt?: string;
	}[];
}

// ---------------------------------------------------------------------------
// AI Inline Card
// ---------------------------------------------------------------------------

/** AI badge variants shown in seed cards and chat. */
export type AiBadgeVariant =
	| 'classified'     // 🤖 Tuntaskan · 92%
	| 'suggested'      // 🤖 Wujudkan? · 74%
	| 'stalled'        // ⚠ Macet 48j
	| 'dampak'         // 🌱 Dampak
	| 'ringkasan'      // 📝 Ringkasan
	| 'duplikat';      // ⚠ Duplikat

export interface AiCardMessage extends ChatMessageBase {
	type: 'ai_card';
	/** The block primitive(s) rendered in this card. */
	blocks: Block[];
	/** AI badge variant for the card. */
	badge?: AiBadgeVariant;
	/** Brief title/heading for the card. */
	title?: string;
}

// ---------------------------------------------------------------------------
// Diff Card Message
// ---------------------------------------------------------------------------

export interface DiffCardMessage extends ChatMessageBase {
	type: 'diff_card';
	diff: DiffCard;
}

// ---------------------------------------------------------------------------
// Vote Card Message
// ---------------------------------------------------------------------------

export interface VoteCardMessage extends ChatMessageBase {
	type: 'vote_card';
	/** Vote block embedded as a chat message. */
	block: Extract<Block, { type: 'vote' }>;
}

// ---------------------------------------------------------------------------
// System Message
// ---------------------------------------------------------------------------

export type SystemMessageSubtype =
	| 'checkpoint_completed'
	| 'phase_activated'
	| 'phase_completed'
	| 'vote_result'
	| 'galang_transaction'
	| 'member_joined'
	| 'role_assigned'
	| 'plan_updated';

export interface SystemMessage extends ChatMessageBase {
	type: 'system';
	subtype: SystemMessageSubtype;
	content: string;
	/** Optional structured data for rendering (e.g., transaction details). */
	data?: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Evidence Message
// ---------------------------------------------------------------------------

/** Evidence triad types: Testimony, Corroboration, Document. */
export type EvidenceType = 'testimony' | 'corroboration' | 'document';

export interface EvidenceMessage extends ChatMessageBase {
	type: 'evidence';
	author: MessageAuthor;
	evidence_type: EvidenceType;
	content: string;
	attachments?: {
		type: 'image' | 'video' | 'receipt';
		url: string;
		alt?: string;
	}[];
}

// ---------------------------------------------------------------------------
// Galang Message
// ---------------------------------------------------------------------------

export interface GalangMessage extends ChatMessageBase {
	type: 'galang';
	subtype: 'contribution' | 'disbursement' | 'milestone';
	content: string;
	/** Amount (only shown, never editable by AI — protected). */
	amount?: number;
	currency?: string;
}

// ---------------------------------------------------------------------------
// Union type
// ---------------------------------------------------------------------------

/** Discriminated union of all chat message types. */
export type ChatMessage =
	| UserMessage
	| AiCardMessage
	| DiffCardMessage
	| VoteCardMessage
	| SystemMessage
	| EvidenceMessage
	| GalangMessage;

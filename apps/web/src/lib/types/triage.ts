/**
 * AI-00 Triage — JSON contracts for the Bagikan entry flow.
 *
 * When users tap [+], AI-00 greets them conversationally. As context
 * builds, the context bar morphs through 8 states. The LLM emits
 * triage results that drive the context bar and entry routing.
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §6
 */

import type { Block, TrackHint } from './blocks';
import type { PathPlan, SeedHint } from './path-plan';
import type { TrajectoryType, CardEnrichment } from './card-enrichment';
import type { KelolaPayload } from './operator';
import type { AiCardMessage, DiffCardMessage, VoteCardMessage } from './chat';

// ---------------------------------------------------------------------------
// Context Bar States
// ---------------------------------------------------------------------------

/**
 * The 8 context bar states during AI-00 conversational triage.
 * The bar morphs as confidence builds.
 */
export type ContextBarState =
	| 'listening' // Empty bar, wave indicator — AI listening
	| 'probing' // Bar + signal bars — AI asking follow-up
	| 'leaning' // Tappable track pill — AI has initial guess
	| 'ready' // Full card: track + confidence + seed type — path plan proposed
	| 'vault-ready' // Dark card (vault palette) — directed to Catatan Saksi
	| 'siaga-ready' // Red pulsing card — emergency detected
	| 'split-ready' // Split card — story can split to 2 flows
	| 'manual'; // Grid: 5 track hints + vault — user tapped "Pilih sendiri"

// ---------------------------------------------------------------------------
// Triage Result
// ---------------------------------------------------------------------------

/** Entry route determined by triage. */
export type EntryRoute = 'komunitas' | 'vault' | 'siaga' | 'catatan_komunitas' | 'kelola';
export type TriageStatus = 'draft' | 'final';
export type TriageKind = 'witness' | 'data';
export type TriageConversationBlockId =
	| 'chat_message'
	| 'ai_inline_card'
	| 'diff_card'
	| 'vote_card'
	| 'moderation_hold_card'
	| 'duplicate_detection_card'
	| 'credit_nudge_card';
export type TriageStructuredBlockId =
	| 'list'
	| 'document'
	| 'form'
	| 'computed'
	| 'display'
	| 'vote'
	| 'reference';
export type TaxonomyCategoryCode =
	| 'commodity_price'
	| 'public_service'
	| 'training'
	| 'employment'
	| 'health'
	| 'education'
	| 'infrastructure'
	| 'safety_alert'
	| 'environment'
	| 'community_event'
	| 'other_custom';
export type TaxonomyQuality = 'official_source' | 'community_observation' | 'unverified_claim';
export type StempelLifecycleState = 'draft' | 'proposed' | 'objection_window' | 'locked';

export interface TriageTaxonomy {
	category_code: TaxonomyCategoryCode;
	category_label: string;
	custom_label?: string;
	quality: TaxonomyQuality;
}

export interface ProgramReference {
	program_id: string;
	label: string;
	source: string;
	confidence: number;
}

export interface TriageStempelState {
	state: StempelLifecycleState;
	proposed_at_ms?: number;
	objection_deadline_ms?: number;
	locked_at_ms?: number;
	min_participants: number;
	participant_count: number;
	objection_count: number;
	latest_objection_at_ms?: number;
	latest_objection_reason?: string;
}

export interface TriageCard {
	icon?: string;
	trajectory_type?: TrajectoryType;
	title?: string;
	hook_line?: string;
	body?: string;
	sentiment?: string;
	intensity?: number;
}

export interface TriageBlocks {
	conversation: TriageConversationBlockId[];
	structured: TriageStructuredBlockId[];
}

export type TriageConversationMessage = AiCardMessage | DiffCardMessage | VoteCardMessage;

/** Confidence level for the AI classification. */
export interface TriageConfidence {
	/** 0-1 confidence score. */
	score: number;
	/** Classification label (e.g., "Tuntaskan · 92%"). */
	label: string;
}

/**
 * The triage result emitted by AI-00 after conversational classification.
 * This drives the context bar state and determines the entry route.
 */
export interface TriageResult {
	/** Backend triage session id for follow-up turns. */
	session_id?: string;
	/** Strict contract version. */
	schema_version?: string;
	/** Draft/final readiness state for create actions. */
	status?: TriageStatus;
	/** Final output kind produced by triage. */
	kind?: TriageKind;
	/** Missing fields when status='draft'. */
	missing_fields?: string[];
	/** Declared conversation/structured blocks from operator output. */
	blocks?: TriageBlocks;
	/** Data taxonomy for controlled vocabulary classification. */
	taxonomy?: TriageTaxonomy;
	/** Structured program references (e.g., MBG). */
	program_refs?: ProgramReference[];
	/** Backend-governed stempel consensus state. */
	stempel_state?: TriageStempelState;
	/** Current context bar state. */
	bar_state: ContextBarState;
	/** Determined entry route. */
	route: EntryRoute;
	/** Short human-readable triage summary from backend. */
	summary_text?: string;
	/** Suggested track hint (optional — metadata only). */
	track_hint?: TrackHint;
	/** Seed type hint. */
	seed_hint?: SeedHint;
	/** Classification confidence. */
	confidence?: TriageConfidence;
	/** Canonical trajectory type — the new routing field. */
	trajectory_type?: TrajectoryType;
	/** AI-generated card enrichment (icon, title, hook_line, sentiment). */
	card_enrichment?: CardEnrichment;
	/** Canonical renderable card payload from backend triage. */
	card?: TriageCard;
	/** Structured block payload rendered in triage-ready preview. */
	structured_payload?: Block[];
	/** Conversation-layer card payload rendered in triage chat preview. */
	conversation_payload?: TriageConversationMessage[];
	/** Token budget tracking — drives the "Sisa Energi AI" energy bar. */
	budget?: TriageBudget;
	/** Proposed path plan (when bar_state is 'ready'). */
	proposed_plan?: PathPlan;
	/** Kelola result — group creation/management data (when route is 'kelola'). */
	kelola_result?: KelolaPayload;
	/** Duplicate detection result (AI-03). */
	duplicate?: {
		/** Similarity percentage (e.g., 87). */
		similarity: number;
		/** ID of the similar seed. */
		similar_seed_id: string;
		/** Title preview. */
		similar_seed_title: string;
	};
}

// ---------------------------------------------------------------------------
// Triage Session Budget
// ---------------------------------------------------------------------------

/**
 * Token budget tracking for a triage session.
 * Instead of a flat turn limit, each session gets a token budget based on
 * user tier × trajectory complexity. The AI adjusts depth dynamically.
 *
 * @see docs/design/specs/ai-spec/04a-ai-00-edge-contract.md §4.1
 */
export type TrajectoryComplexity = 'simple' | 'standard' | 'complex';

export interface TriageBudget {
	/** Total input tokens allocated for this session. */
	total_tokens: number;
	/** Tokens consumed so far. */
	used_tokens: number;
	/** Tokens remaining. */
	remaining_tokens: number;
	/** Percentage of budget used (0.0–1.0). */
	budget_pct: number;
	/** Whether the session can accept more messages. */
	can_continue: boolean;
	/** Current turn number (1-indexed). */
	turn_count: number;
	/** Hard turn cap (always 8). */
	max_turns: number;
}

// ---------------------------------------------------------------------------
// Rahasia (Privacy) Levels
// ---------------------------------------------------------------------------

/**
 * 4-level privacy overlay.
 * L2 and L3 are IRREVERSIBLE once set.
 */
export type RahasiaLevel = 'L0' | 'L1' | 'L2' | 'L3';

export interface RahasiaConfig {
	level: RahasiaLevel;
	/** Display name for this level. */
	label: 'Terbuka' | 'Terbatas' | 'Rahasia' | 'Sangat Rahasia';
}

// ---------------------------------------------------------------------------
// Emergency Types (Siaga)
// ---------------------------------------------------------------------------

export type EmergencyType =
	| 'kebakaran'
	| 'banjir'
	| 'gempa'
	| 'darurat_medis'
	| 'kecelakaan'
	| 'keamanan'
	| 'lainnya';

// ---------------------------------------------------------------------------
// Triage Attachments
// ---------------------------------------------------------------------------

/** A file selected during triage — not yet uploaded. */
export interface TriageAttachment {
	id: string;
	file: File;
	type: 'image' | 'video' | 'audio';
	/** Local blob URL for preview (via URL.createObjectURL). */
	preview_url: string;
}

/**
 * AI-00 Triage — JSON contracts for the Bagikan entry flow.
 *
 * When users tap [+], AI-00 greets them conversationally. As context
 * builds, the context bar morphs through 8 states. The LLM emits
 * triage results that drive the context bar and entry routing.
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §6
 */

import type { TrackHint } from './blocks';
import type { PathPlan, SeedHint } from './path-plan';

// ---------------------------------------------------------------------------
// Context Bar States
// ---------------------------------------------------------------------------

/**
 * The 8 context bar states during AI-00 conversational triage.
 * The bar morphs as confidence builds.
 */
export type ContextBarState =
	| 'listening'     // Empty bar, wave indicator — AI listening
	| 'probing'       // Bar + signal bars — AI asking follow-up
	| 'leaning'       // Tappable track pill — AI has initial guess
	| 'ready'         // Full card: track + confidence + seed type — path plan proposed
	| 'vault-ready'   // Dark card (vault palette) — directed to Catatan Saksi
	| 'siaga-ready'   // Red pulsing card — emergency detected
	| 'split-ready'   // Split card — story can split to 2 flows
	| 'manual';       // Grid: 5 track hints + vault — user tapped "Pilih sendiri"

// ---------------------------------------------------------------------------
// Triage Result
// ---------------------------------------------------------------------------

/** Entry route determined by triage. */
export type EntryRoute = 'komunitas' | 'vault' | 'siaga';

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
	/** Current context bar state. */
	bar_state: ContextBarState;
	/** Determined entry route. */
	route: EntryRoute;
	/** Suggested track hint (optional — metadata only). */
	track_hint?: TrackHint;
	/** Seed type hint. */
	seed_hint?: SeedHint;
	/** Classification confidence. */
	confidence?: TriageConfidence;
	/** Proposed path plan (when bar_state is 'ready'). */
	proposed_plan?: PathPlan;
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

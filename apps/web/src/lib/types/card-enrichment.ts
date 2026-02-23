/**
 * Card Enrichment & Trajectory types — AI-generated feed presentation.
 *
 * Every trajectory produces a CardEnrichment at triage completion.
 * The enrichment drives icon, color, title, and sentiment rendering
 * in feed cards. TrajectoryType is the canonical routing field that
 * replaces the old 5-track model.
 *
 * @see docs/design/specs/ai-spec/04b-trajectory-map.md
 */

import type { EntityType } from './feed';

// ---------------------------------------------------------------------------
// Trajectory Type — the 11 human-intent archetypes
// ---------------------------------------------------------------------------

/**
 * Canonical trajectory type. Each maps to a distinct human intent pattern
 * and determines mood color treatment in the feed.
 *
 * Ongoing work (Witness): aksi, advokasi, pantau, mufakat, mediasi, program
 * One-off (Data Item):    data, vault, bantuan, pencapaian, siaga
 */
export type TrajectoryType =
	| 'aksi'        // A: collective action      — amber
	| 'advokasi'    // B: advocacy/escalation     — rose
	| 'pantau'      // D: watchdog/monitor        — indigo
	| 'mufakat'     // F: proposal/musyawarah     — teal
	| 'mediasi'     // L: dispute resolution      — violet
	| 'program'     // M: ongoing program         — emerald
	| 'data'        // C: community data/survey   — sky
	| 'vault'       // E: private sealed record   — slate
	| 'bantuan'     // G: help request            — amber (lighter)
	| 'pencapaian'  // I: celebration             — yellow
	| 'siaga';      // J: alert/warning           — red

// ---------------------------------------------------------------------------
// Sentiment
// ---------------------------------------------------------------------------

/** Emotional mood for visual styling — 7 canonical values. */
export type Sentiment = 'angry' | 'hopeful' | 'urgent' | 'celebratory' | 'sad' | 'curious' | 'fun';

// ---------------------------------------------------------------------------
// Card Enrichment — AI-generated feed presentation
// ---------------------------------------------------------------------------

/**
 * AI-generated enrichment attached to every triage result at completion.
 * Drives how the case appears in the feed — icon, title, hook, sentiment.
 */
export interface CardEnrichment {
	/** Lucide icon name, AI-selected per case content (e.g., "construction", "scale"). */
	icon: string;
	/** Canonical trajectory type — drives mood color, NOT icon. */
	trajectory_type: TrajectoryType;
	/** AI-crafted title: factual + specific + compelling. Max 80 chars. */
	title: string;
	/** One-liner that draws attention — the hook. */
	hook_line: string;
	/** Most striking phrase from user's story. */
	pull_quote?: string;
	/** 2-3 sentence summary, massaged for civility. */
	body: string;
	/** Emotional mood for visual styling. */
	sentiment: Sentiment;
	/** Conversation heat level (1-5). */
	intensity: number;
	/** AI-suggested entity tags for discoverability. */
	entity_tags?: EntityTagSuggestion[];
	/** LLM-generated contextual labels for signal chips. */
	signal_labels?: SignalLabels;
}

// ---------------------------------------------------------------------------
// Entity Tag Suggestion
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Signal Labels — LLM-generated contextual reaction chip labels
// ---------------------------------------------------------------------------

/**
 * LLM-generated label for a content-directed signal chip.
 * The underlying signal TYPE (saksi/perlu_dicek) stays fixed for credit math,
 * but the user-facing label/desc/icon are contextualized per trajectory and story.
 */
export interface SignalLabel {
	/** User-facing label, e.g. "Saya Lihat", "Sudah Bantu". Max 15 chars. */
	label: string;
	/** Explanation shown in expanded mode. One sentence. */
	desc: string;
	/** Optional Lucide icon name override (from DynamicIcon registry). */
	icon?: string;
}

/** LLM-generated contextual labels for the 2 content-directed signal chips. */
export interface SignalLabels {
	saksi: SignalLabel;
	perlu_dicek: SignalLabel;
}

// ---------------------------------------------------------------------------
// Entity Tag Suggestion
// ---------------------------------------------------------------------------

/** AI-suggested tag for a case — proposed during enrichment, confirmed by user. */
export interface EntityTagSuggestion {
	/** Display label (e.g., "Jl. Mawar", "RT 05", "Dinas PU"). */
	label: string;
	/** Entity type from the existing taxonomy. */
	entity_type: EntityType;
	/** AI confidence in this tag suggestion. */
	confidence: number;
}

/**
 * AI Trigger Modes — JSON contracts for the 4 ways AI engages.
 *
 * These define when and how the LLM produces output (diff cards,
 * alerts, suggestions) within a witness/case.
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §8.4
 * @see docs/design/specs/ui-ux-spec/29-llm-architecture.md §28.4
 */

// ---------------------------------------------------------------------------
// Trigger Modes
// ---------------------------------------------------------------------------

export type TriggerMode =
	| 'manual' // User taps 🔄 Perbarui → diff card in chat
	| 'milestone' // Keyword/pattern at breakpoints → stage transition suggestion
	| 'time_triggered' // Scheduled interval → alert in chat
	| 'passive'; // Continuous monitoring → badge/indicator only

/**
 * A trigger event that causes the LLM to produce output.
 */
export interface TriggerEvent {
	/** Which trigger mode fired. */
	mode: TriggerMode;
	/** ID of the witness/case. */
	witness_id: string;
	/** ISO timestamp. */
	triggered_at: string;
	/** What AI touch point produced this (AI-00 through AI-09). */
	ai_id?: string;
	/** Context for the trigger (e.g., matched keyword, schedule config). */
	context?: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// AI Touch Points
// ---------------------------------------------------------------------------

/**
 * The 10 AI touch points in the system.
 * Not all are LLM — some are backend/Tandang.
 */
export type AiTouchPoint =
	| 'AI-00' // Conversational Triage (Bagikan)
	| 'AI-01' // Track & Seed Hint Classifier
	| 'AI-02' // Redaction LLM
	| 'AI-03' // Duplicate Detector
	| 'AI-04' // Content Moderation
	| 'AI-05' // Gaming Detection
	| 'AI-06' // Criteria Suggestion
	| 'AI-07' // Discussion Summary
	| 'AI-08' // Media Redaction
	| 'AI-09'; // Credit Accreditation

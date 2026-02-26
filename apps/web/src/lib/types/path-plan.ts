/**
 * Adaptive Path Plan — JSON contracts for LLM-emitted path plans.
 *
 * The LLM proposes a case-specific path (phases and checkpoints) and
 * guides the user through it. The frontend parses and renders using
 * block primitives.
 *
 * Hierarchy: PathPlan → Branch → Phase → Checkpoint
 *
 * @see docs/design/specs/ADAPTIVE-PATH-SPEC-v0.1.md
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §4
 */

import type { SourceTag, TrackHint } from './blocks';

// ---------------------------------------------------------------------------
// Statuses
// ---------------------------------------------------------------------------

/** Status for phases and checkpoints. */
export type PlanItemStatus = 'planned' | 'active' | 'open' | 'completed' | 'blocked' | 'skipped';

// ---------------------------------------------------------------------------
// Seed Hints
// ---------------------------------------------------------------------------

/**
 * Optional seed type hint — the emotional/contextual origin of the witness.
 * Used by AI-00 triage but does not force a lifecycle.
 */
export type SeedHint = 'Keresahan' | 'Aspirasi' | 'Kejadian' | 'Rencana' | 'Pertanyaan';

// ---------------------------------------------------------------------------
// Checkpoint
// ---------------------------------------------------------------------------

export interface Checkpoint {
	checkpoint_id: string;
	title: string;
	status: PlanItemStatus;
	source: SourceTag;
	locked_fields: string[];
	/** Optional description or completion criteria. */
	description?: string;
	/** Evidence requirements for verification-heavy checkpoints. */
	evidence_required?: boolean;
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

export type AssistNeedUrgency = 'low' | 'medium' | 'high';

export interface PhaseAssistNeed {
	esco_skill_uri: string;
	skill_label: string;
	reason: string;
	urgency: AssistNeedUrgency;
	min_people: number;
}

export interface Phase {
	phase_id: string;
	title: string;
	objective: string;
	status: PlanItemStatus;
	source: SourceTag;
	locked_fields: string[];
	checkpoints: Checkpoint[];
	/** Optional ESCO-coded support requirements for this phase. */
	assist_needs?: PhaseAssistNeed[];
}

// ---------------------------------------------------------------------------
// Branch
// ---------------------------------------------------------------------------

export interface Branch {
	branch_id: string;
	label: string;
	/**
	 * The checkpoint from which this branch forks.
	 * `null` for the main branch.
	 */
	parent_checkpoint_id: string | null;
	phases: Phase[];
}

// ---------------------------------------------------------------------------
// Path Plan (top-level)
// ---------------------------------------------------------------------------

export interface PathPlan {
	plan_id: string;
	version: number;
	title: string;
	summary: string;
	/** Optional track color hint — metadata only, not a lifecycle driver. */
	track_hint?: TrackHint;
	/** Optional seed type hint from AI-00 triage. */
	seed_hint?: SeedHint;
	branches: Branch[];
}

// ---------------------------------------------------------------------------
// Path Plan Envelope (wire format from LLM)
// ---------------------------------------------------------------------------

/**
 * The envelope the LLM returns. Contains the plan plus audit metadata.
 * All plan generations include these fields for version control.
 */
export interface PathPlanEnvelope {
	path_plan: PathPlan;
	/** LLM model used for generation. */
	model_id?: string;
	/** Prompt template version. */
	prompt_version?: string;
	/** ISO timestamp of generation. */
	generated_at?: string;
}

/**
 * Diff Card — JSON contracts for the "Suggest-Don't-Overwrite" pattern.
 *
 * When the LLM wants to modify structured content, it emits a diff card
 * into the chat. The user reviews and decides: Terapkan (apply),
 * Tinjau Satu-satu (review one by one), or Abaikan (dismiss).
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §8.5
 * @see docs/design/specs/ui-ux-spec/29-llm-architecture.md §28.5
 */

import type { SourceTag } from './blocks';

// ---------------------------------------------------------------------------
// Diff Operations
// ---------------------------------------------------------------------------

export type DiffOperation = 'add' | 'remove' | 'modify' | 'reorder';

/** A single change within a diff card. */
export interface DiffItem {
	/** What kind of change. */
	operation: DiffOperation;
	/** Path to the affected field/item (e.g., "checkpoints[2].title"). */
	path: string;
	/** Human-readable description of the change. */
	label: string;
	/** Previous value (for modify/remove). */
	old_value?: unknown;
	/** New value (for add/modify). */
	new_value?: unknown;
	/** Whether this field is protected (financial, identity). */
	protected: boolean;
}

// ---------------------------------------------------------------------------
// Diff Card
// ---------------------------------------------------------------------------

/** The target type that this diff applies to. */
export type DiffTargetType = 'list' | 'document' | 'form' | 'checkpoint' | 'phase';

export interface DiffCard {
	/** Unique ID for this diff suggestion. */
	diff_id: string;
	/** What the diff applies to. */
	target_type: DiffTargetType;
	/** ID of the target block/entity being modified. */
	target_id: string;
	/** Human-readable summary (e.g., "Ditambah 2 item, dicentang 1"). */
	summary: string;
	/** Evidence or reasoning quotes from conversation. */
	evidence?: string[];
	/** Individual changes. */
	items: DiffItem[];
	/** Source — always 'ai' for diff cards. */
	source: Extract<SourceTag, 'ai'>;
	/** ISO timestamp when the diff was generated. */
	generated_at: string;
	/** Reference to the plan version this diff targets. */
	plan_version?: number;
}

// ---------------------------------------------------------------------------
// Diff Card User Actions
// ---------------------------------------------------------------------------

export type DiffAction =
	/** Apply all changes at once. */
	| 'apply_all'
	/** Open review-one-by-one mode. */
	| 'review'
	/** Dismiss the entire diff. */
	| 'dismiss';

/** Per-item decision when reviewing one-by-one. */
export type DiffItemDecision = 'accept' | 'reject' | 'edit';

export interface DiffItemReview {
	/** Index into DiffCard.items. */
	item_index: number;
	decision: DiffItemDecision;
	/** Edited value if decision is 'edit'. */
	edited_value?: unknown;
}

/** User response to a diff card. */
export interface DiffResponse {
	diff_id: string;
	action: DiffAction;
	/** Per-item reviews (only when action is 'review'). */
	item_reviews?: DiffItemReview[];
}

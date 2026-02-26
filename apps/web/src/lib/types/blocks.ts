/**
 * Block Primitives — JSON contracts for LLM-emitted UI blocks.
 *
 * The LLM emits structured JSON using these 7 block types.
 * The frontend parses and renders each block using the appropriate
 * Svelte component from the block registry.
 *
 * @see docs/design/specs/UI-GUIDELINE-v1.0.md §8.2
 * @see docs/design/specs/ui-ux-spec/29-llm-architecture.md §28.2
 */

// ---------------------------------------------------------------------------
// Source Tags
// ---------------------------------------------------------------------------

/** Who created or last modified this piece of structured data. */
export type SourceTag = 'ai' | 'human' | 'system';

/**
 * Source metadata attached to every editable unit.
 * - `ai`     → LLM-generated, can be overwritten by next pass or human edit
 * - `human`  → Human-created/edited, AI stops touching (locked)
 * - `system` → System-computed, nobody edits
 */
export interface SourceMeta {
	source: SourceTag;
	/** Field names that have been manually edited and are locked from LLM. */
	locked_fields: string[];
}

// ---------------------------------------------------------------------------
// Block Discriminator
// ---------------------------------------------------------------------------

export type BlockType =
	| 'list'
	| 'document'
	| 'form'
	| 'computed'
	| 'display'
	| 'vote'
	| 'reference';

// ---------------------------------------------------------------------------
// List Block
// ---------------------------------------------------------------------------

/** Status values for list items (checkpoints, tasks, etc.) */
export type ListItemStatus = 'open' | 'completed' | 'blocked' | 'skipped';

export interface ListItem extends SourceMeta {
	id: string;
	label: string;
	status: ListItemStatus;
	/** Nested children for nestable lists (e.g., sub-tasks). */
	children?: ListItem[];
	/** Optional metadata key-value pairs. */
	meta?: Record<string, unknown>;
}

/**
 * `list` — Checklist, table, timeline, gallery.
 * AI rule: Additive. Nestable. Status-changeable.
 * Source tag: Per-item.
 */
export interface ListBlock {
	type: 'list';
	id: string;
	/** Display mode hint for the renderer. */
	display: 'checklist' | 'table' | 'timeline' | 'gallery';
	title?: string;
	items: ListItem[];
}

// ---------------------------------------------------------------------------
// Document Block
// ---------------------------------------------------------------------------

export interface DocumentSection extends SourceMeta {
	id: string;
	heading?: string;
	/** Rich text content (markdown or plain). */
	content: string;
}

/**
 * `document` — Rich text with tracked changes.
 * AI rule: AI drafts, human edits sections.
 * Source tag: Per-section.
 */
export interface DocumentBlock {
	type: 'document';
	id: string;
	title?: string;
	sections: DocumentSection[];
}

// ---------------------------------------------------------------------------
// Form Block
// ---------------------------------------------------------------------------

export type FormFieldType = 'text' | 'number' | 'date' | 'select' | 'textarea' | 'toggle' | 'file';

export interface FormField extends SourceMeta {
	id: string;
	label: string;
	field_type: FormFieldType;
	value?: unknown;
	placeholder?: string;
	/** Whether this field is protected (financial, identity). */
	protected: boolean;
	/** Validation rules (optional). */
	validation?: {
		required?: boolean;
		min?: number;
		max?: number;
		pattern?: string;
	};
	/** Options for select fields. */
	options?: { value: string; label: string }[];
}

/**
 * `form` — Labeled input fields.
 * AI rule: AI suggests per field. Protected fields = hands-off.
 * Source tag: Per-field.
 */
export interface FormBlock {
	type: 'form';
	id: string;
	title?: string;
	fields: FormField[];
}

// ---------------------------------------------------------------------------
// Computed Block
// ---------------------------------------------------------------------------

export type ComputedDisplay = 'progress' | 'status' | 'score' | 'counter' | 'confidence';

/**
 * `computed` — Read-only derived data (progress bar, status indicator).
 * AI rule: System-derived. Nobody edits.
 * Source tag: Always `system`.
 */
export interface ComputedBlock {
	type: 'computed';
	id: string;
	display: ComputedDisplay;
	label: string;
	/** Current value (e.g., 72 for 72% progress). */
	value: number;
	/** Maximum value (e.g., 100). */
	max?: number;
	/** Optional unit label (e.g., "%", "hari"). */
	unit?: string;
}

// ---------------------------------------------------------------------------
// Display Block
// ---------------------------------------------------------------------------

/**
 * `display` — Presentation card (recognition, appreciation).
 * AI rule: One-way render. No edit.
 * Source tag: Always `system`.
 */
export interface DisplayBlock {
	type: 'display';
	id: string;
	title: string;
	content: string;
	/** Optional media attachments. */
	media?: {
		type: 'image' | 'video';
		url: string;
		alt?: string;
		captions_url?: string;
	}[];
	/** Optional metadata (author, date, etc.) */
	meta?: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Vote Block
// ---------------------------------------------------------------------------

export type VoteType = 'standard' | 'weighted' | 'quorum_1_5x' | 'consensus';

export interface VoteOption {
	id: string;
	label: string;
	count: number;
	/** Weighted count (for weighted votes). */
	weighted_count?: number;
}

/**
 * `vote` — Voting interface + tally.
 * AI rule: System tallies. Not AI.
 * Source tag: Always `system`.
 */
export interface VoteBlock {
	type: 'vote';
	id: string;
	question: string;
	vote_type: VoteType;
	options: VoteOption[];
	/** Quorum requirement (e.g., 0.3 = 30%). */
	quorum: number;
	/** Total eligible voters. */
	total_eligible: number;
	/** Total votes cast so far. */
	total_voted: number;
	/** Duration in hours. */
	duration_hours: number;
	/** ISO timestamp when voting ends. */
	ends_at: string;
	/** Whether the current user has already voted. */
	user_voted?: boolean;
}

// ---------------------------------------------------------------------------
// Reference Block
// ---------------------------------------------------------------------------

/**
 * `reference` — Preview card linking to another witness/case.
 * AI rule: Links to other cards.
 * Source tag: Always `reference`.
 */
export interface ReferenceBlock {
	type: 'reference';
	id: string;
	/** ID of the referenced entity (seed, plan, etc.) */
	ref_id: string;
	/** Type of referenced entity. */
	ref_type: 'seed' | 'plan' | 'checkpoint' | 'document';
	/** Preview title. */
	title: string;
	/** Preview snippet. */
	snippet?: string;
	/** Track hint color for the reference card. */
	track_hint?: TrackHint;
}

// ---------------------------------------------------------------------------
// Union type
// ---------------------------------------------------------------------------

/** Discriminated union of all 7 block primitives. */
export type Block =
	| ListBlock
	| DocumentBlock
	| FormBlock
	| ComputedBlock
	| DisplayBlock
	| VoteBlock
	| ReferenceBlock;

// ---------------------------------------------------------------------------
// Track Hints (shared across blocks and path plans)
// ---------------------------------------------------------------------------

/**
 * Track hint label — dynamically generated by AI.
 *
 * Common values include 'tuntaskan', 'wujudkan', 'telusuri', 'rayakan',
 * 'musyawarah', but any contextually appropriate string is valid.
 * These are color/display metadata, not lifecycle drivers.
 */
export type TrackHint = string;

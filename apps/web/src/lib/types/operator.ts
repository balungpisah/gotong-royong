/**
 * Operator types — consultation protocol for AI-00 triage operators.
 *
 * 9 uniform operators, each mapping 1:1 to a trajectory grid item
 * (plus Kelola for group lifecycle). All return the same canonical
 * OperatorResponse envelope with an operator-specific payload.
 *
 * Classification is inline in the orchestrator, not a separate operator.
 *
 * @see docs/design/specs/ai-spec/04c-operator-skill-map.md (v0.2)
 */

import type { PathPlan } from './path-plan';

// ---------------------------------------------------------------------------
// Canonical Envelope — same for all 9 operators
// ---------------------------------------------------------------------------

/** All possible operator payload types. */
export type OperatorPayload =
	| MasalahPayload
	| MusyawarahPayload
	| PantauPayload
	| CatatPayload
	| BantuanPayload
	| RayakanPayload
	| SiagaPayload
	| ProgramPayload
	| KelolaPayload;

/**
 * The canonical operator response envelope.
 * The orchestrator reads `status`, `checklist`, and `questions` uniformly;
 * the `payload` is operator-specific and only meaningful when status = 'ready'.
 */
export interface OperatorResponse {
	/** Whether the operator has enough information to produce a result. */
	status: 'need_more' | 'ready';
	/** What's filled, what's missing — orchestrator renders this as progress. */
	checklist: ChecklistItem[];
	/** Suggested next questions for the orchestrator to ask the user. */
	questions?: string[];
	/** Specialized payload — only meaningful when status = 'ready'. */
	payload: OperatorPayload;
}

/** A single checklist item tracking information gathering progress. */
export interface ChecklistItem {
	/** Field identifier (e.g., "problem_scope", "stakeholders"). */
	field: string;
	/** Whether this field has been captured from the conversation. */
	filled: boolean;
	/** Summary of the captured info (when filled). */
	value?: string;
}

// ---------------------------------------------------------------------------
// 1. Masalah — Problem → Action/Escalation (trajectories A + B)
// ---------------------------------------------------------------------------

/**
 * Masalah operator payload — problem → phases.
 * Handles both trajectory A (self-solve) and B (escalate).
 * The KEY question (can community self-solve?) determines A vs B.
 */
export interface MasalahPayload {
	/** Whether community can self-solve (A) or needs authority (B). */
	trajectory: 'A' | 'B';
	/** The proposed action plan. */
	path_plan: PathPlan;
}

// ---------------------------------------------------------------------------
// 2. Musyawarah — Deliberation/Mediation (trajectories F + L)
// ---------------------------------------------------------------------------

/**
 * Musyawarah operator payload — complex issue → decision steps.
 * Handles both trajectory F (proposal) and L (dispute).
 * The mechanism is identical; context differs.
 */
export interface MusyawarahPayload {
	/** Whether this is a proposal deliberation or dispute resolution. */
	context: 'proposal' | 'dispute';
	/** AI-structured decision steps, each becoming a VoteBlock. */
	decision_steps: DecisionStep[];
	/** Gateway trigger — if all steps reach consensus, spawn trajectory A. */
	on_consensus?: 'spawn_aksi';
}

/** A single decision step in a musyawarah/mediation process. */
export interface DecisionStep {
	/** The question to be decided (e.g., "Apakah lahannya bisa dipakai?"). */
	question: string;
	/** Available options (e.g., ["Setuju", "Tidak", "Perlu survey"]). */
	options?: string[];
	/** Why this must be decided — AI's rationale. */
	rationale: string;
	/** Order number — fundamentals first, details later. */
	order: number;
}

// ---------------------------------------------------------------------------
// 3. Pantau — Watchdog/Monitor (trajectory D)
// ---------------------------------------------------------------------------

/**
 * Pantau operator payload — case → timeline.
 * Handles trajectory D (watchdog/monitor).
 */
export interface PantauPayload {
	/** Case category (e.g., "legal", "political", "environmental"). */
	case_type: string;
	/** Initial timeline events seeded from user's story. */
	timeline_seed: TimelineEvent[];
	/** What to watch for next — tracking guidance. */
	tracking_points: string[];
}

/** A single event in a watchdog timeline. */
export interface TimelineEvent {
	/** What happened. */
	event: string;
	/** When it happened (ISO date string). */
	date: string;
	/** Source of the information. */
	source: 'user' | 'news' | 'official';
	/** Link to supporting evidence. */
	evidence_url?: string;
}

// ---------------------------------------------------------------------------
// 4. Catat — Record data or vault (trajectories C + E)
// ---------------------------------------------------------------------------

/**
 * Catat operator payload — structured data point.
 * Handles both trajectory C (public data) and E (private vault).
 * The `record_type` determines storage and visibility.
 */
export interface CatatPayload {
	/** Public community data (C) or sealed private record (E). */
	record_type: 'data' | 'vault';
	/** The factual claim or record content. */
	claim: string;
	/** Location context (RT/RW/address). */
	location?: string;
	/** ISO timestamp of the observation. */
	observed_at: string;
	/** Category for aggregation (e.g., "harga", "infrastruktur", "lingkungan"). */
	category: string;
	/** URL or reference to attached proof (photo, document). */
	proof_url?: string;
	/** Content hash for vault records (tamper-evident). */
	hash?: string;
}

// ---------------------------------------------------------------------------
// 5. Bantuan — Help matching (trajectory G)
// ---------------------------------------------------------------------------

/**
 * Bantuan operator payload — help request + matched resources.
 * Handles trajectory G (help request).
 */
export interface BantuanPayload {
	/** Type of help needed (e.g., "hukum", "kesehatan", "teknis"). */
	help_type: string;
	/** Description of what's needed. */
	description: string;
	/** How urgent the help is. */
	urgency: 'rendah' | 'sedang' | 'tinggi';
	/** Matched resources from expertise registry. */
	matched_resources: MatchedResource[];
}

/** A matched helper/resource from the expertise registry. */
export interface MatchedResource {
	/** User ID or entity ID of the helper. */
	resource_id: string;
	/** Display name. */
	name: string;
	/** Relevance score (0-1). */
	relevance: number;
	/** Why this resource was matched. */
	match_reason: string;
}

// ---------------------------------------------------------------------------
// 6. Rayakan — Celebration/Achievement (trajectory I)
// ---------------------------------------------------------------------------

/**
 * Rayakan operator payload — achievement record.
 * Handles trajectory I (pencapaian).
 * Also auto-triggered when a witness reaches "completed" status.
 */
export interface RayakanPayload {
	/** What was achieved. */
	achievement: string;
	/** Who contributed — user IDs or names. */
	contributors: string[];
	/** Linked witness ID (if celebrating a completed witness). */
	linked_witness_id?: string;
	/** Summary of community impact. */
	impact_summary: string;
}

// ---------------------------------------------------------------------------
// 7. Siaga — Alert/Warning (trajectory J)
// ---------------------------------------------------------------------------

/**
 * Siaga operator payload — urgent alert.
 * Handles trajectory J (siaga).
 * Community verification (confirm/deny) enabled by default.
 */
export interface SiagaPayload {
	/** Type of threat (e.g., "banjir", "kebakaran", "penipuan", "wabah"). */
	threat_type: string;
	/** Severity level. */
	severity: 'waspada' | 'siaga' | 'darurat';
	/** Affected location. */
	location: string;
	/** Description of the threat. */
	description: string;
	/** Source of the alert. */
	source: string;
	/** ISO timestamp when alert should auto-expire. */
	expires_at: string;
}

// ---------------------------------------------------------------------------
// 8. Program — Recurring activity (trajectory M)
// ---------------------------------------------------------------------------

/**
 * Program operator payload — schedule definition.
 * Handles trajectory M (program/recurring activity).
 */
export interface ProgramPayload {
	/** Name of the activity (e.g., "Ronda Malam", "Kerja Bakti"). */
	activity_name: string;
	/** How often it recurs. */
	frequency: 'harian' | 'mingguan' | 'bulanan' | 'custom';
	/** Custom frequency description (when frequency = 'custom'). */
	frequency_detail?: string;
	/** Location of the activity. */
	location?: string;
	/** Participant rotation schedule. */
	rotation: RotationEntry[];
	/** ISO timestamp of next occurrence. */
	next_occurrence?: string;
}

/** A single entry in a rotation schedule. */
export interface RotationEntry {
	/** User ID or name. */
	participant: string;
	/** Slot label (e.g., "Senin", "Minggu ke-1"). */
	slot: string;
}

// ---------------------------------------------------------------------------
// 9. Kelola — Group lifecycle management (no trajectory)
// ---------------------------------------------------------------------------

/**
 * Kelola operator payload — group CRUD via conversational triage.
 * Not tied to any trajectory — handles group lifecycle management.
 */
export interface KelolaPayload {
	/** What group action to perform. */
	action: 'create' | 'edit' | 'invite' | 'join' | 'leave';
	/** Group details (for create/edit). */
	group_detail?: KelolaGroupDetail;
	/** Target group ID (for edit/invite/join/leave). */
	group_id?: string;
	/** User IDs to invite (for invite action). */
	invited_user_ids?: string[];
}

/** Group detail for create/edit actions. */
export interface KelolaGroupDetail {
	/** Group name. */
	name: string;
	/** Group description. */
	description: string;
	/** Join policy. */
	join_policy: 'terbuka' | 'persetujuan' | 'undangan';
	/** Entity type. */
	entity_type: 'kelompok' | 'lembaga';
}

// ---------------------------------------------------------------------------
// Legacy aliases (backward compat — will be removed)
// ---------------------------------------------------------------------------

/** @deprecated Use MasalahPayload */
export type AksiPayload = MasalahPayload;
/** @deprecated Use MusyawarahPayload */
export type MufakatPayload = MusyawarahPayload;

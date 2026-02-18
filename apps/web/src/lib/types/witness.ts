/**
 * Witness domain types — the central aggregate entity.
 *
 * A Witness is a community case/thread — the main unit of civic engagement.
 * It wraps chat messages, path plans, blocks, and member participation.
 */

import type { TrackHint, Block } from './blocks';
// NOTE: SeedHint is in path-plan.ts, TrackHint in blocks.ts, RahasiaLevel in triage.ts
// But we import from sibling files since this is inside types/
import type { SeedHint, PathPlan } from './path-plan';
import type { RahasiaLevel, TriageResult } from './triage';
import type { ChatMessage } from './chat';

/** Witness lifecycle status. */
export type WitnessStatus = 'draft' | 'open' | 'active' | 'resolved' | 'closed';

/** Roles within a witness/case. */
export type WitnessMemberRole = 'pelapor' | 'relawan' | 'koordinator' | 'saksi';

/**
 * Witness summary — lightweight representation for lists and cards.
 */
export interface Witness {
	witness_id: string;
	title: string;
	summary: string;
	track_hint?: TrackHint;
	seed_hint?: SeedHint;
	status: WitnessStatus;
	rahasia_level: RahasiaLevel;
	created_at: string;
	updated_at: string;
	created_by: string;
	member_count: number;
	message_count: number;
	unread_count: number;
}

/**
 * WitnessDetail — full aggregate loaded on the witness detail page.
 * Extends Witness with messages, plan, blocks, and members.
 */
export interface WitnessDetail extends Witness {
	messages: ChatMessage[];
	plan: PathPlan | null;
	blocks: Block[];
	members: WitnessMember[];
	triage?: TriageResult;
}

/**
 * A member participating in a witness/case.
 */
export interface WitnessMember {
	user_id: string;
	name: string;
	avatar_url?: string;
	role: WitnessMemberRole;
	tier?: number;
	joined_at: string;
}

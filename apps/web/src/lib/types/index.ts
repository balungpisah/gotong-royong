// Auth types
export type { AuthRole, AuthSession, SessionUser } from '$lib/auth';

// LLM JSON contracts — Block Primitives
export type {
	SourceTag,
	SourceMeta,
	BlockType,
	ListItemStatus,
	ListItem,
	ListBlock,
	DocumentSection,
	DocumentBlock,
	FormFieldType,
	FormField,
	FormBlock,
	ComputedDisplay,
	ComputedBlock,
	DisplayBlock,
	VoteType,
	VoteOption,
	VoteBlock,
	ReferenceBlock,
	Block,
	TrackHint
} from './blocks';

// LLM JSON contracts — Adaptive Path Plan
export type {
	PlanItemStatus,
	SeedHint,
	Checkpoint,
	Phase,
	Branch,
	PathPlan,
	PathPlanEnvelope
} from './path-plan';

// LLM JSON contracts — Diff Cards
export type {
	DiffOperation,
	DiffItem,
	DiffTargetType,
	DiffCard,
	DiffAction,
	DiffItemDecision,
	DiffItemReview,
	DiffResponse
} from './diff-card';

// LLM JSON contracts — Chat Messages
export type {
	MessageAuthor,
	ChatMessageType,
	ChatMessageBase,
	UserMessage,
	AiBadgeVariant,
	AiCardMessage,
	DiffCardMessage,
	VoteCardMessage,
	SystemMessageSubtype,
	SystemMessage,
	EvidenceType,
	EvidenceMessage,
	GalangMessage,
	ChatMessage
} from './chat';

// LLM JSON contracts — AI-00 Triage
export type {
	ContextBarState,
	EntryRoute,
	TriageConfidence,
	TriageResult,
	RahasiaLevel,
	RahasiaConfig,
	EmergencyType
} from './triage';

// LLM JSON contracts — AI Trigger Modes
export type { TriggerMode, TriggerEvent, AiTouchPoint } from './trigger';

// Domain aggregates — Witness
export type {
	WitnessStatus,
	WitnessMemberRole,
	Witness,
	WitnessDetail,
	WitnessMember
} from './witness';

// Domain aggregates — User
export type { UserProfile, UserStats } from './user';

// Domain aggregates — Notification
export type { NotificationType, AppNotification } from './notification';

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------
export type { NavigationTag, TabConfig, TagSuggestion, WellKnownTag } from './navigation';
export { WELL_KNOWN_TAGS, DEFAULT_TABS } from './navigation';

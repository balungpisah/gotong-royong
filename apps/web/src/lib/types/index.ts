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
	TrajectoryComplexity,
	TriageBudget,
	RahasiaLevel,
	RahasiaConfig,
	EmergencyType,
	TriageAttachment
} from './triage';

// LLM JSON contracts — Card Enrichment & Trajectory
export type {
	TrajectoryType,
	Sentiment,
	CardEnrichment,
	EntityTagSuggestion,
	SignalLabel,
	SignalLabels
} from './card-enrichment';

// LLM JSON contracts — Operator Consultation Protocol (v0.2: 9 uniform operators)
export type {
	OperatorResponse,
	OperatorPayload,
	ChecklistItem,
	MasalahPayload,
	MusyawarahPayload,
	PantauPayload,
	CatatPayload,
	BantuanPayload,
	MatchedResource,
	RayakanPayload,
	SiagaPayload,
	ProgramPayload,
	RotationEntry,
	KelolaPayload,
	KelolaGroupDetail,
	DecisionStep,
	TimelineEvent,
	// Legacy aliases (deprecated)
	AksiPayload,
	MufakatPayload
} from './operator';

// LLM JSON contracts — AI Trigger Modes
export type { TriggerMode, TriggerEvent, AiTouchPoint } from './trigger';

// Domain aggregates — Witness
export type {
	WitnessStatus,
	WitnessMemberRole,
	Witness,
	WitnessDetail,
	WitnessMember,
	WitnessCreateInput
} from './witness';

// Domain aggregates — User
export type { UserProfile, UserStats, TandangSignals, OctalysisScores, ActivityItem } from './user';

// Domain aggregates — Tandang (Full I/C/J model)
export type {
	TandangTierLevel,
	TandangTierName,
	TandangTier,
	IntegrityScore,
	CompetenceDomain,
	CompetenceScore,
	JudgmentScore,
	TandangScores,
	ConsistencyInfo,
	GenesisInfo,
	VouchType,
	VouchRelation,
	PersonRelation,
	TandangAvatarPerson,
	VouchBudget,
	DukungRecord,
	GrSkillDomain,
	UserSkill,
	WeatherType,
	GdfWeather,
	ImpactMetrics,
	ActivityTimelineItem,
	TandangProfile
} from './tandang';

// Domain aggregates — Notification
export type { NotificationType, AppNotification } from './notification';

// Domain aggregates — Feed
export type {
	FeedEventType,
	FeedEvent,
	UrgencyBadge,
	FeedSource,
	FeedFilter,
	FeedItem,
	FeedMemberPreview,
	RepostFrame,
	EntityType,
	EntityTag,
	FollowableEntity,
	FeedWitnessItem,
	FeedDataItem,
	DataItemRecord,
	FeedSystemItem,
	FeedStreamItem,
	SystemCardVariant,
	SystemCardData,
	SuggestionPayload,
	TipPayload,
	MilestonePayload,
	PromptPayload,
	PeekMessage,
	SignalChipType,
	MyRelation,
	SignalCounts,
	FeedListRequest,
	FeedListResponse,
	AutoPantauTrigger,
	UserMonitorRecord,
	WitnessCloseReason,
	SignalResolutionOutcome,
	ContentSignalType,
	ContentSignal
} from './feed';
export { shouldAutoMonitor } from './feed';

// Domain aggregates — Groups (Kelompok / Lembaga)
export type {
	GroupJoinPolicy,
	GroupMemberRole,
	MembershipRequestStatus,
	GroupEntityType,
	GroupSummary,
	GroupDetail,
	GroupMember,
	MembershipRequest,
	GroupCreateInput,
	GroupUpdateInput
} from './group';

// Domain aggregates — Community
export type {
	CommunityStats,
	ParticipationDataPoint,
	CommunitySignalSummary,
	CommunityActivityItem
} from './community';

// Domain aggregates — Community Dashboard (Full page model)
export type {
	TierDistribution,
	CommunityIcjSummary,
	ActiveMemberHighlight,
	SignalFlowDataPoint,
	CommunityDashboard
} from './komunitas';

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------
export type { NavigationTag, TabConfig, TagSuggestion, WellKnownTag } from './navigation';
export { WELL_KNOWN_TAGS, DEFAULT_TABS } from './navigation';

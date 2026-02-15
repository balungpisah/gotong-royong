-- 0009_audit_retention_fields
-- Add immutable audit hash and retention tags to audit-heavy entities (BE-012).

DEFINE FIELD event_hash ON TABLE track_state_transition TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE track_state_transition TYPE option<string>;

DEFINE FIELD event_hash ON TABLE vault_entry TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE vault_entry TYPE option<string>;

DEFINE FIELD event_hash ON TABLE vault_timeline_event TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE vault_timeline_event TYPE option<string>;

DEFINE FIELD event_hash ON TABLE siaga_broadcast TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE siaga_broadcast TYPE option<string>;

DEFINE FIELD event_hash ON TABLE siaga_timeline_event TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE siaga_timeline_event TYPE option<string>;

DEFINE FIELD event_hash ON TABLE content_moderation TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE content_moderation TYPE option<string>;

DEFINE FIELD event_hash ON TABLE moderation_decision TYPE option<string>;
DEFINE FIELD retention_tag ON TABLE moderation_decision TYPE option<string>;

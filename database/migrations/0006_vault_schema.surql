-- 0006_vault_schema
-- Vault lifecycle persistence and timeline ledger for BE-006 / BE-007.

DEFINE TABLE vault_entry SCHEMAFULL;
DEFINE TABLE vault_timeline_event SCHEMAFULL;

DEFINE FIELD vault_entry_id ON TABLE vault_entry TYPE string;
DEFINE FIELD author_id ON TABLE vault_entry TYPE string;
DEFINE FIELD author_username ON TABLE vault_entry TYPE string;
DEFINE FIELD state ON TABLE vault_entry TYPE string;
DEFINE FIELD created_at ON TABLE vault_entry TYPE datetime;
DEFINE FIELD updated_at ON TABLE vault_entry TYPE datetime;
DEFINE FIELD sealed_at ON TABLE vault_entry TYPE option<datetime>;
DEFINE FIELD sealed_hash ON TABLE vault_entry TYPE option<string>;
DEFINE FIELD encryption_key_id ON TABLE vault_entry TYPE option<string>;
DEFINE FIELD attachment_refs ON TABLE vault_entry TYPE array;
DEFINE FIELD wali ON TABLE vault_entry TYPE array;
DEFINE FIELD payload ON TABLE vault_entry TYPE option<object>;
DEFINE FIELD publish_target ON TABLE vault_entry TYPE option<string>;
DEFINE FIELD retention_policy ON TABLE vault_entry TYPE option<object>;
DEFINE FIELD audit ON TABLE vault_entry TYPE option<object>;
DEFINE FIELD request_id ON TABLE vault_entry TYPE string;
DEFINE FIELD correlation_id ON TABLE vault_entry TYPE string;

DEFINE FIELD event_id ON TABLE vault_timeline_event TYPE string;
DEFINE FIELD vault_entry_id ON TABLE vault_timeline_event TYPE string;
DEFINE FIELD event_type ON TABLE vault_timeline_event TYPE string;
DEFINE FIELD actor ON TABLE vault_timeline_event TYPE object;
DEFINE FIELD request_id ON TABLE vault_timeline_event TYPE string;
DEFINE FIELD correlation_id ON TABLE vault_timeline_event TYPE string;
DEFINE FIELD occurred_at ON TABLE vault_timeline_event TYPE datetime;
DEFINE FIELD metadata ON TABLE vault_timeline_event TYPE option<object>;

DEFINE INDEX uniq_vault_entry_id
ON TABLE vault_entry FIELDS vault_entry_id UNIQUE;

DEFINE INDEX uniq_vault_author_request
ON TABLE vault_entry FIELDS author_id, request_id UNIQUE;

DEFINE INDEX idx_vault_entry_author
ON TABLE vault_entry FIELDS author_id;

DEFINE INDEX uniq_vault_timeline_event_id
ON TABLE vault_timeline_event FIELDS event_id UNIQUE;

DEFINE INDEX idx_vault_timeline_entry
ON TABLE vault_timeline_event FIELDS vault_entry_id, occurred_at, event_id;

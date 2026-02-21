# Prompt ID Registration Log (Research)

Last updated: `2026-02-14`

Use this log to keep prompt versioning decisions explicit before Edge-Pod ticket creation.

## Status legend

- `REGISTERED`: prompt_id exists in `docs/design/specs/ai-spec/14-prompt-versioning.md`
- `REGISTERED+GAP`: base prompt is registered but companion/scoring sub-prompts are still missing
- `PENDING`: spec exists but prompt_id not yet in the version registry
- `OUT_OF_SCOPE`: not a registry-managed AI touchpoint (helper/local extraction)

## Consolidated prompt-id table

| AI touchpoint | Source document | Prompt source | Version source | Registry status | Notes / Decision needed |
|---|---|---|---|---|---|
| AI-00 | `docs/design/specs/ai-spec/04-ai-00-triage.md` | TRIAGE-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-01 | `docs/design/specs/ai-spec/05-ai-01-classification.md` | CLASS-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-02 | `docs/design/specs/ai-spec/06-ai-02-redaction.md` | REDACT-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-03 | `docs/design/specs/ai-spec/07-ai-03-duplicate-detection.md` | DUPLICATE-001 | `14-prompt-versioning.md` | REGISTERED | Approved for implementation |
| AI-04 | `docs/design/specs/ai-spec/08-ai-04-content-moderation.md` | MOD-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-05 | `docs/design/specs/ai-spec/09-ai-05-gaming-detection.md` | GAMING-001 | `14-prompt-versioning.md` | REGISTERED | Approved for implementation |
| AI-06 | `docs/design/specs/ai-spec/10-ai-06-criteria-suggestions.md` | CRITERIA-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-07 | `docs/design/specs/ai-spec/11-ai-07-discussion-summarization.md` | SUM-001 | `14-prompt-versioning.md` | REGISTERED | Stable |
| AI-08 | `docs/design/specs/ai-spec/12-ai-08-sensitive-media.md` | SENSITIVE-001 | `14-prompt-versioning.md` | REGISTERED | Approved for implementation |
| AI-09 | `docs/design/specs/ai-spec/13-ai-09-credit-accreditation.md` | CREDIT-001 only | `14-prompt-versioning.md` | REGISTERED | no companion scoring sub-prompt required for current flow |
| Skill extraction | `docs/design/specs/ai-spec/10-ai-06-criteria-suggestions.md` (scope note) | ESCO extraction | n/a | OUT_OF_SCOPE | Not in 14-prompt-versioning; decide if local extraction or registry extension (EP-10) is required |

## Downstream usage

- `docs/research/ai-contract-log.md` references this log for AI-03/05/08 blockers.
- `docs/research/edgepod-endpoint-contracts.md` references this log for EP-03/05/08 implementation prerequisites.

## Open actions

1. Confirm and keep the approved IDs in this file and reflect in implementation tickets.
2. If future credit scoring decomposition is required, log owner and prompt_id in a follow-up entry.
3. Link approved IDs into issue templates used by Edge-Pod implementation.

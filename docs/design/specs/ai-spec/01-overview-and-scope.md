> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

# Gotong Royong — AI Layer Specification v0.2

> **Note (2026-02-15):** Entry flow assumptions in AI-00/AI-01 that rely on fixed track lifecycles are superseded for new experiences by `docs/design/specs/ADAPTIVE-PATH-SPEC-v0.1.md`. Legacy track classification remains as optional metadata only.

## Changelog from v0.1

- **Added AI-00 (Conversational Triage)** — new entry-point touch point for guided witness story collection
- **Added AI-09 (Credit Accreditation)** — Tandang credit distribution and reputation tracking
- **Updated AI-01** — now triggered by AI-00 internally, not at Bagikan Step 2; can be invoked multiple times during conversation
- **Updated AI-03** — dual-threshold system (≥80% blocking, 50-79% non-blocking, <50% silent); context bar pill pre-submission
- **Updated AI-04** — moderation hold 3-perspective system from C2 design; "Dalam peninjauan" vs "Menunggu moderasi" states
- **Updated cross-references** — all UI-UX-SPEC references now point to v0.5; added AI-00 and AI-09 to decision log

---

## 1. Overview & Scope

Gotong Royong's AI layer comprises **10 AI touch points** spanning the full lifecycle of a community issue (from triage through resolution). This document specifies their inputs, outputs, decision rules, fallback behaviors, and integration points.

### 1.1 Touch Points (AI-00 through AI-09)

| Touch Point | Name | Phase | Purpose |
|---|---|---|---|
| **AI-00** | Conversational Triage | Entry | Greet user, listen to story, probe for clarity, propose adaptive path plan and classify into entry flow |
| **AI-01** | Triple Refinement | Entry | Validate and refine RDF triples from AI-00; extract Action type, temporal class, and skills (optional metadata) |
| **AI-02** | Redaction LLM | Entry | Identify and mask PII from story text |
| **AI-03** | Duplicate Detection | Entry / Ongoing | Find similar existing seeds; flag or merge |
| **AI-04** | Content Moderation | Submission | Policy compliance check; hold for review if needed |
| **AI-05** | Gaming Pattern Detection | Ongoing | Detect coordinated abuse, false endorsements, reputation gaming |
| **AI-06** | Criteria & Task Suggestion | Planning / Execution phases | Recommend objective criteria and decompose into tasks |
| **AI-07** | Discussion Summarization | Discussion phases | Summarize discussion threads for quick reference |
| **AI-08** | Sensitive Media Detection & Redaction | Submission | Detect faces, plates, private locations; offer redaction |
| **AI-09** | Credit Accreditation | Lifecycle / Completion | Track and distribute community credit across adaptive path phases |

### 1.2 Scope Boundaries

- **In scope:** AI models, prompts, decision rules, context carryover, fallback behaviors
- **Out of scope:** Tandang backend capabilities (handled by Tandang team)
- **UI integration:** Detailed screen layouts remain in UI-UX-SPEC v0.5; this document references them

---

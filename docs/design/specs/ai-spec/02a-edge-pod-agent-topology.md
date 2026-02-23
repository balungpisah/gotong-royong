> [← Back to AI Spec index](../AI-SPEC-v0.2.md)
> Parent: [02-three-layer-architecture.md](./02-three-layer-architecture.md)

# Edge-Pod Agent Topology

> **Status:** Draft v0.1
> **Last updated:** 2026-02-21
>
> This document defines the internal agent architecture of the edge-pod.
> Each AI touchpoint is served by an **orchestrator** that may delegate
> work to **operators** (sub-agents) and invoke **skills** (reusable
> capabilities). We specify **what** is needed and **when** it's invoked;
> the edge-pod team handles the internal implementation.

---

## 1. Core Concepts

### Orchestrator

The main LLM agent that holds the user-facing conversation. One orchestrator
per AI touchpoint session. It:

- Manages the conversational flow with the user
- Maintains the state machine (e.g., `listening → probing → ready`)
- Decides **when** to delegate to operators
- Synthesizes operator results into user-facing responses
- Produces the final structured output (`TriageResult`, `PathPlan`, etc.)

### Operators

Specialized sub-agents that the orchestrator delegates discrete tasks to.
Operators are:

- **Focused** — each does one thing well (classification, duplicate check, plan generation)
- **Stateless** — receive input, return output, no conversation memory
- **Composable** — orchestrator can call multiple operators per turn
- **Replaceable** — can swap models/prompts independently of the orchestrator

### Skills

Reusable atomic capabilities available to the orchestrator and/or operators.
Skills are smaller than operators — think utility functions vs. services:

- **Deterministic** — consistent behavior given same input
- **Cross-cutting** — may be used by multiple operators or touchpoints
- **Lightweight** — typically rule-based or single-inference

---

## 2. General Pattern

```
                         User message (delta)
                              │
                              ▼
                    ┌───────────────────┐
                    │   ORCHESTRATOR    │
                    │                   │
                    │  • Converses      │
                    │  • State machine  │
                    │  • Delegates      │
                    │  • Synthesizes    │
                    └───┬───┬───┬───┬───┘
                        │   │   │   │
             ┌──────────┘   │   │   └──────────┐
             ▼              ▼   ▼              ▼
        ┌─────────┐   ┌────────┐  ┌────────┐  ┌─────────┐
        │ Op: A   │   │ Op: B  │  │ Op: C  │  │ Op: D   │
        │         │   │        │  │        │  │         │
        └────┬────┘   └───┬────┘  └───┬────┘  └────┬────┘
             │             │          │             │
             ▼             ▼          ▼             ▼
        ┌─────────────────────────────────────────────┐
        │              SKILLS (shared)                │
        │  PII Redaction · Vault Detection · Locale   │
        │  Urgency Scoring · ESCO Extraction · ...    │
        └─────────────────────────────────────────────┘
```

---

## 3. AI-00 Triage — Operators

| Operator | Purpose | Trigger | Suggested Timing |
|----------|---------|---------|------------------|
| **Aksi** (A+B) | Build action/escalation `PathPlan` from problem description | When orchestrator classifies problem/complaint signal | Sequential — depends on inline classification |
| **Mufakat** (F+L) | Structure complex issue into `DecisionStep[]` for voting | When orchestrator classifies proposal/deliberation signal | Sequential — depends on inline classification |
| **Pantau** (D) | Build timeline structure + tracking points from case description | When orchestrator classifies monitoring/tracking signal | Sequential — depends on inline classification |
| **Duplicate Detector** (AI-03) | Find similar existing witnesses | When confidence ≥ 0.5 (leaning or above) | Parallel — runs alongside operator consultation |

### Delegation Flow (AI-00)

```
Turn 1 (startTriage):
  User message → Orchestrator
    ├─ [every turn] Skills: PII, Vault Detection, Urgency Scoring
    ├─ [inline] Classify: signal = 'problem', confidence = 0.5
    ├─ [sequential] Aksi operator: need_more, checklist partially filled
    └─ Orchestrator synthesizes follow-up question from operator's suggestions
  Return: bar_state=probing

Turn 2 (updateTriage):
  User message → Orchestrator
    ├─ [every turn] Skills: PII, Vault Detection, Urgency Scoring
    ├─ [inline] Classify: confidence = 0.72
    ├─ [parallel] Duplicate Detector(text) → { duplicate: null }
    ├─ [sequential] Aksi operator: need_more, checklist mostly filled
    └─ Orchestrator synthesizes follow-up
  Return: bar_state=leaning, trajectory_type=aksi

Turn 3 (updateTriage):
  User message → Orchestrator
    ├─ [inline] Classify: confidence = 0.92
    ├─ [parallel] Duplicate Detector(text) → { duplicate: null }
    ├─ [sequential] Aksi operator: ready, payload = AksiPayload with PathPlan
    ├─ [sequential] Card Enrichment skill → CardEnrichment
    └─ Orchestrator synthesizes final response
  Return: bar_state=ready, proposed_plan={...}, card_enrichment={...}
```

### When NOT to Delegate

The orchestrator handles these **directly** (no operator needed):

- Generating conversational responses (greeting, follow-up questions)
- State machine transitions (probing → leaning → ready)
- Vault signal detection (keyword/pattern matching — use Vault Detection skill)
- Siaga signal detection (keyword/pattern matching — use Urgency Scoring skill)

> **Note:** Classification is now performed **inline** by the orchestrator (not a separate operator). The orchestrator analyzes the user's message and determines which operator to consult based on signal patterns.

---

## 4. AI-00 Triage — Skills

| Skill | Purpose | Used By | Invocation |
|-------|---------|---------|------------|
| **PII Redaction** | Detect and mask personally identifiable information | Orchestrator (before storing transcript) | Every turn — scan user message |
| **Vault Detection** | Detect sensitive/private content signals | Orchestrator (state machine decision) | Every turn — check for vault keywords |
| **Urgency Scoring** | Detect emergency signals for Siaga routing | Orchestrator (state machine decision) | Every turn — check for siaga keywords |
| **ESCO Extraction** | Extract skill codes from conversation | Classifier operator | When confidence ≥ 0.5 |
| **Locale Formatting** | Format responses in correct Bahasa Indonesia | Orchestrator + all operators | Every response |
| **Confidence Labeling** | Generate human-readable confidence label (e.g., "Tuntaskan · 92%") | Orchestrator | When confidence changes |

### Skill vs. Operator Decision Rule

| Use a **Skill** when... | Use an **Operator** when... |
|--------------------------|------------------------------|
| Deterministic / rule-based | Requires LLM inference |
| Single-pass, no reasoning chain | Multi-step reasoning |
| < 100ms expected latency | May take 1-5 seconds |
| Reusable across touchpoints | Specific to one touchpoint |
| Example: PII regex scan | Example: PathPlan generation |

---

## 5. Operator Registry (All Touch Points)

This section will grow incrementally as we spec each AI touchpoint. Each row
is an operator that the edge-pod needs to implement.

| Operator | Touch Point | Purpose | Model Tier |
|----------|-------------|---------|------------|
| Aksi | AI-00 | Problem → action/escalation PathPlan (trajectories A+B) | Strong (Sonnet-class) |
| Mufakat | AI-00 | Complex issue → decision steps for voting (trajectories F+L) | Strong (Sonnet-class) |
| Pantau | AI-00 | Case → timeline structure + tracking points (trajectory D) | Strong (Sonnet-class) |
| Duplicate Detector | AI-00, AI-03 | Similarity search against existing witnesses | Medium (Haiku-class) + embedding |
| Redaction LLM | AI-02 | Deep PII detection beyond regex | Medium (Haiku-class) |
| Content Moderator | AI-04 | Policy compliance check | Strong (Sonnet-class) |
| Gaming Detector | AI-05 | Coordinated abuse pattern detection | Strong (Sonnet-class) |
| Criteria Suggester | AI-06 | Recommend checkpoints and task decomposition | Strong (Sonnet-class) |
| Media Analyzer | AI-08 | Sensitive media detection (faces, plates) | Vision model |
| Credit Calculator | AI-09 | Distribute community credit across phases | Medium (Haiku-class) |

> **Note:** Model tier is a suggestion. Edge-pod team selects actual models
> based on their benchmarking and cost constraints.
>
> **Note:** Classifier, Plan Generator, and Summarizer are removed. Classification is now inline in the orchestrator. Plan generation is part of operator output. Summarization is handled by the orchestrator.

---

## 6. Skill Registry (All Touch Points)

Shared skills available to any orchestrator or operator:

| Skill | Purpose | Type | Used By |
|-------|---------|------|---------|
| PII Redaction | Mask names, addresses, phone numbers, ID numbers | Regex + pattern | AI-00, AI-02, AI-04 |
| Vault Detection | Detect privacy/sensitivity signals in text | Keyword + pattern | AI-00 |
| Urgency Scoring | Detect emergency signals (fire, flood, medical) | Keyword + pattern | AI-00 |
| ESCO Extraction | Extract skill/occupation codes from text | LLM-assisted | AI-00, AI-01 |
| Locale Formatting | Bahasa Indonesia grammar and tone | Rule-based | All |
| Confidence Labeling | Generate label from score + track (e.g., "Tuntaskan · 92%") | Template | AI-00 |
| Duplicate Embedding | Generate text embedding for similarity search | Embedding model | AI-03 |
| Location Enrichment | Resolve lat/lng to RT/RW/Kelurahan | Geocoding API | AI-00, AI-03 |
| Timestamp Parsing | Extract temporal references from text ("3 bulan lalu") | NLP / regex | AI-00, AI-06 |
| Catat | Structure data point (claim + location + timestamp + proof) | Template + LLM | AI-00 |
| Brankas | Structure sealed vault record | Template + LLM | AI-00 |
| Cocok | Match help request to expertise registry | LLM + embedding | AI-00 |
| Jadwal | Structure recurring program schedule | Template + LLM | AI-00 |
| Rayakan | Generate achievement record from completion | Template + LLM | AI-00 |
| Card Enrichment | Generate title, icon, hook_line, body, sentiment at triage completion | LLM-assisted | AI-00 |

---

## 7. Template: Documenting Operators for a New Touch Point

When adding a new AI touchpoint, add a section to this document following
this pattern:

```markdown
## N. AI-{NN} {Name} — Operators

| Operator | Purpose | Trigger | Suggested Timing |
|----------|---------|---------|------------------|
| ...      | ...     | ...     | parallel / sequential |

### Delegation Flow (AI-{NN})

(Describe the turn-by-turn or step-by-step delegation)

### When NOT to Delegate

(What the orchestrator handles directly)
```

Also update the Operator Registry (§5) and Skill Registry (§6) tables
with any new entries.

---

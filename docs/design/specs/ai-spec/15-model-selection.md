> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 15. Model Selection Guidelines

All AI touch points use tiered model selection based on task complexity and latency requirements.

### 15.1 Model Tier Summary

| Tier | Model Example | Cost | Latency | Best For |
|---|---|---|---|---|
| **Strong** | Claude Sonnet | $$$ | 2–5s | Complex reasoning, nuance, multi-step |
| **Medium** | Claude Haiku | $$ | 1–3s | Classification, summarization, extraction |
| **Lightweight** | Specialized classifier | $ | <500ms | Binary decisions, rules-based |
| **ML Pipeline** | Embedding / CV | $ | 1–3s | Vector search, image detection |

### 15.2 Touch Point Model Assignment

| Touch Point | Model Tier | Model Spec | Why |
|---|---|---|---|
| **AI-00 Triage** | Strong | Sonnet | Conversational, multi-turn, context-aware, must probe naturally |
| **AI-01 Classification** | Medium | Haiku | Structured 5-track classification; latency-sensitive (called by AI-00) |
| **AI-02 Redaction** | Strong | Sonnet | Nuanced PII detection in Bahasa Indonesia; false positives costly |
| **AI-03 Duplicate** | ML Pipeline | Embedding model | Vector search via Tandang; no LLM needed |
| **AI-04 Moderation** | Lightweight | Fine-tuned classifier | Policy violation binary decision; must be fast + safe |
| **AI-05 Gaming** | Statistical + LLM | Rules + optional Sonnet | Statistical baselines primary; LLM for edge cases |
| **AI-06 Criteria** | Strong | Sonnet | Complex reasoning about objectives, task decomposition |
| **AI-07 Summary** | Medium | Haiku | Summarization + extraction; lightweight task |
| **AI-08 Media** | ML Pipeline | CV (faces, plates) | Specialized computer vision; not LLM-based |
| **AI-09 Credit** | Medium | Haiku | Structured scoring + aggregation; low-complexity logic |

---


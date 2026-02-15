> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 2. Three-Layer Architecture

All AI decision points follow a consistent three-layer pattern:

```
┌─────────────────────────────────────────┐
│   Backend Layer (Gotong Royong)         │
│   ├─ Data collection & validation       │
│   ├─ Pre-processing (redaction, QA)     │
│   └─ Triggers for AI decision points    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Tandang Integration Layer             │
│   ├─ Feature vector creation            │
│   ├─ Caching & retrieval                │
│   └─ Fallback data sources              │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   AI Decision Layer (LLM / ML)          │
│   ├─ Model inference                    │
│   ├─ Confidence scoring                 │
│   └─ Reasoning extraction               │
└─────────────────────────────────────────┘
```

Each AI touch point:
1. Receives structured input from Backend or Tandang
2. Queries Tandang for contextual data (if applicable)
3. Runs model inference with system prompt + few-shot examples
4. Returns structured output + confidence + reasoning
5. Falls back gracefully if model unavailable

---


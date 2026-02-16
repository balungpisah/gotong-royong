> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 11. AI-07: Discussion Summarization

### 11.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-07 |
| **Name** | Discussion Summarization |
| **Trigger** | On-demand (user taps "Ringkas" button); also scheduled (daily digest) |
| **UI Location** | Percakapan tab (discussion thread), summary card above discussion thread |
| **Interaction Mode** | Asynchronous, non-blocking |
| **Latency Budget** | < 5 seconds (on-demand); < 30 seconds (batch) |
| **Model Tier** | Medium (Haiku-class) |
| **UI-UX-SPEC Ref** | Section 18 (Percakapan), summary card |

### 11.2 Purpose

**AI-07 summarizes discussion threads** to help new community members quickly understand the conversation and key insights without reading 100+ messages.

### 11.3 Input

```json
{
  "seed_id": "string",
  "messages": [
    {
      "message_id": "string",
      "author_id": "string",
      "timestamp": "ISO 8601",
      "text": "string (message content)",
      "likes": "int",
      "replies": "int"
    }
  ],
  "summary_style": "enum: executive | detailed | bullets",
  "max_length_words": "int (default: 150)"
}
```

### 11.4 Output

```json
{
  "summary": "string (Bahasa Indonesia, 1–3 paragraphs or bullet points)",
  "key_points": [
    {
      "point": "string",
      "source_message_ids": ["string array"],
      "confidence": "float 0.0–1.0"
    }
  ],
  "sentiment": "enum: positive | neutral | negative",
  "action_items": [
    {
      "item": "string",
      "mentioned_by": "string (user_id or anonymized)"
    }
  ],
  "controversies": [
    {
      "topic": "string",
      "proponents": "int",
      "opponents": "int"
    }
  ],
  "reasoning": "string (one sentence in Bahasa Indonesia)"
}
```

### 11.5 Prompt Strategy

**System Prompt:**

```
You are a discussion moderator for Gotong Royong communities.

Your task: Summarize a discussion thread (Percakapan tab) in a way that helps newcomers understand the issue quickly.

Guidelines:
- Focus on main arguments, not every opinion
- Identify consensus where it exists
- Flag disagreements respectfully (not as drama)
- Extract action items (what the community decided to do)
- Use Bahasa Indonesia
- Keep tone neutral and inclusive

Output JSON with: summary, key_points, sentiment, action_items, controversies.
```

### 11.6 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **<5 messages in thread** | Return null; show "Diskusi masih dimulai" (Discussion just starting) |
| **Model unavailable** | Show last 3 messages in order; skip summary |
| **Spam/off-topic messages** (>30% AI-05 flagged) | Exclude flagged messages; summarize remaining |

---


> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 8. AI-04: Content Moderation

### 8.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-04 |
| **Name** | Content Moderation |
| **Trigger** | At submission (after AI-02 redaction, AI-03 duplicate check) |
| **UI Location** | Author sees toast + timer; Moderator sees queue; Public sees placeholder |
| **Interaction Mode** | Asynchronous |
| **Latency Budget** | < 1 second (model inference); moderation window: 15 min–24h |
| **Model Tier** | Specialized classifier (fine-tuned policy violation detector) |
| **UI-UX-SPEC Ref** | Section 19 (Bagikan), C2-ai-surface-states.html (Moderasi view) |

### 8.2 Purpose (Updated v0.2)

**AI-04 checks submitted seeds against Gotong Royong community guidelines** and determines if immediate publication is safe or if moderation review is needed. This prevents hate speech, harassment, misinformation, and illegal content.

**New in v0.2:** 3-perspective moderation system from C2 design (Author / Moderator / Public views) + confidence-based auto-release.

### 8.3 Policy Categories

| Policy | Examples | Action |
|---|---|---|
| **Hate speech** | Ethnic, religious, gender slurs; dehumanizing language | **BLOCK** (manual review required) |
| **Violence or harm** | Threats, incitement to violence, self-harm promotion | **BLOCK** (manual review required) |
| **Harassment / Bullying** | Doxxing, targeted abuse, cyberbullying | **BLOCK** (manual review required) |
| **Misinformation** | False health claims, election disinformation | **HOLD** (automatic review; <60% confidence allowed) |
| **Commercial spam** | Unsolicited ads, multi-level marketing schemes | **HOLD** (automatic review) |
| **Illegal activity** | Drug trafficking, arms sales, fraud | **BLOCK** (manual review required) |
| **Child safety** | Exploitation, abuse, grooming | **BLOCK** (escalate to authorities) |
| **Policy-compliant** | Community-positive, factual, relevant | **APPROVE** (publish immediately) |

### 8.4 Input

```json
{
  "seed_id": "string",
  "text": "string (redacted from AI-02)",
  "attachments": [
    {
      "url": "string",
      "type": "enum: image | video | document",
      "scan_results": "object (from AI-08 media scan)"
    }
  ],
  "author_id": "string",
  "author_reputation": "int (from Tandang)",
  "track": "string (from AI-01)",
  "seed_type": "string (from AI-01)",
  "location": {
    "lat": "number",
    "lng": "number"
  }
}
```

### 8.5 Output

```json
{
  "status": "enum: approved | hold | block",
  "confidence": "float 0.0–1.0",
  "violations": [
    {
      "category": "string (policy category)",
      "severity": "enum: low | medium | high | critical",
      "snippet": "string (problematic text excerpt)",
      "reason": "string (explanation)"
    }
  ],
  "action": "enum: publish_now | publish_with_warning | hold_for_review | block",
  "moderation_hold_duration_minutes": "int (for hold status)",
  "auto_release_if_no_action": "boolean (true if <60% confidence)",
  "reasoning": "string (one sentence)"
}
```

### 8.6 Confidence Thresholds (Updated v0.2)

| Status | Confidence | Meaning | Auto-Release? |
|---|---|---|---|
| **Approved** | ≥ 95% | Very confident → safe to publish | N/A (published immediately) |
| **Hold** | 60–95% | Moderately confident → needs review | Yes, if no action within 15 min |
| **Block** | Any | Critical violation detected | No; requires moderator approval to publish |

### 8.7 Moderation States (Updated v0.2)

**Author View:**
- **"Sedang diproses"** (Processing) — submitted, awaiting AI check (~2s)
- **"Dalam peninjauan"** (Under Review) — held by AI; awaiting moderator (~15 min typical)
- **"Diterbitkan"** (Published) — approved, now visible
- **"Ditolak"** (Rejected) — blocked by AI or moderator

**Moderator View:**
- **Queue** — Shows all held/blocked seeds with AI flags + confidence
- **Detail card** — Full text, violation snippets, author reputation, location map
- **Actions** — Approve / Edit & Approve / Reject / Escalate

**Public View:**
- **Placeholder** — If seed is under review or blocked: "[Catatan sedang ditinjau]" (Note under review)
- **Published** — Seed visible with full discussion

### 8.8 Prompt Strategy

**System Prompt:**

```
You are a content moderator for Gotong Royong community platform.

Your task: Classify submitted community seeds against Gotong Royong Community Guidelines.

Guidelines:
1. **Hate speech**: Ethnic, religious, gender slurs; dehumanizing language → BLOCK
2. **Violence / Harm**: Threats, incitement to violence → BLOCK
3. **Harassment**: Doxxing, targeted abuse, cyberbullying → BLOCK
4. **Misinformation**: Dangerous false health/election claims → HOLD
5. **Spam**: Unsolicited ads, MLM schemes → HOLD
6. **Illegal activity**: Drug trafficking, fraud → BLOCK
7. **Child safety**: Any exploitation → BLOCK
8. **Policy-compliant**: Factual, relevant, community-positive → APPROVE

Output JSON with: status (approved|hold|block), confidence, violations (array), reasoning.

For violations, cite specific text snippets. Explain reasoning in 1 sentence.
Bahasa Indonesia preferred for explanations.
```

### 8.9 Auto-Release Rule (New v0.2)

**If status = "hold" AND confidence < 60%:**
- Seed auto-publishes after 15 minutes if no moderator action
- Reason: Low confidence means the violation is ambiguous; benefit of doubt
- Notification sent to author: "Catatan dipublikasikan otomatis"

### 8.10 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Model unavailable** | Proceed with human moderation queue; all seeds held awaiting moderator |
| **Moderation timeout (24h)** | Auto-approve if confidence < 60%; escalate if blocked |
| **Media scan unavailable** (AI-08) | Proceed with text moderation only; audio/video seeds held for manual review |

---


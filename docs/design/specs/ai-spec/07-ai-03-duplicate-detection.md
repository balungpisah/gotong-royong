> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 7. AI-03: Duplicate Detection

### 7.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-03 |
| **Name** | Duplicate Detection |
| **Trigger** | During AI-00 triage; also at submission before moderation |
| **UI Location** | Context bar pill (during triage); modal card at submission |
| **Interaction Mode** | Synchronous |
| **Latency Budget** | < 2 seconds |
| **Model Tier** | Embedding model (vector search via Tandang) |
| **UI-UX-SPEC Ref** | Section 19 (Bagikan), C2-ai-surface-states.html (Duplikat view) |

### 7.2 Purpose (Updated v0.2)

**AI-03 detects duplicate or highly similar existing seeds** during triage and at submission. This prevents:
- Redundant discussion threads
- Spamming the same issue multiple times
- Diluting community attention

**New in v0.2:** Context bar pill visualization during triage (non-blocking) + dual-threshold system.

### 7.3 Similarity Thresholds

| Confidence | Action | UX |
|---|---|---|
| **≥ 80%** | High confidence match (blocking) | Show warning pill + comparison modal; strong merge prompt |
| **50–79%** | Medium confidence (non-blocking) | Show info pill + optional comparison; user can proceed or merge |
| **< 50%** | Low confidence (silent) | No flag shown |

### 7.4 Input

```json
{
  "query_text": "string (redacted story from AI-02)",
  "query_embedding": "array of floats (vector from Tandang)",
  "location": {
    "lat": "number",
    "lng": "number"
  },
  "radius_km": "int (default: 5)",
  "exclude_seed_ids": ["string array (seeds to exclude from search)"],
  "search_archived": "boolean (default: false)"
}
```

### 7.5 Output

```json
{
  "matches": [
    {
      "seed_id": "string",
      "similarity": "float 0.0–1.0",
      "distance_km": "float (geographic distance)",
      "seed_title": "string",
      "seed_author": "string (anonymized if needed)",
      "published_date": "ISO 8601",
      "track": "string",
      "status": "enum: active | completed | archived",
      "discussion_count": "int"
    }
  ],
  "top_match": {
    // (same structure as matches[0])
  },
  "confidence_level": "enum: high | medium | low",
  "recommendation": "merge | proceed | manual_review"
}
```

### 7.6 Duplicate Detection Logic (Updated v0.2)

**Semantic + Geographic Matching:**

1. **Vector search** via Tandang: Find top 10 seeds with highest cosine similarity
2. **Filter by geography:** Keep only seeds within `radius_km` (default 5km)
3. **Filter by status:** Exclude archived/deleted seeds unless `search_archived=true`
4. **Rank by recency:** Prioritize recent seeds (last 30 days) over older ones
5. **Return top match** + confidence level

**Confidence Calculation:**

```
confidence = (semantic_similarity * 0.7) + (geographic_proximity_factor * 0.3)

where:
  semantic_similarity = cosine(query_embedding, seed_embedding)
  geographic_proximity_factor = max(0, 1 - distance_km / radius_km)
```

### 7.7 Context Bar Pill (During Triage)

**Pill appearance (if match ≥50%):**

```
⚠ Saksi serupa (87% mirip)
```

**Tappable:** Opens side-by-side comparison card with:
- Original story (left)
- Existing seed (right)
- Similarity score (87%)
- 3 options:
  - Merge / Gabung (confirm merge)
  - Beda / Berbeda (proceed as new seed, explain why)
  - Lihat / View (open full seed detail)

### 7.8 Comparison Card (At Submission)

**Full modal card showing:**

| Section | Content |
|---|---|
| **Similarity bar** | Visual bar 0–100% |
| **Side-by-side text** | Original vs existing |
| **Seed metadata** | Author, date, discussion count, track |
| **Options** | Merge button, Proceed button, Manual review button |

**Merge flow:**
- User taps "Merge"
- Existing seed's Bahas Diskusi is linked to new story
- New story marked as `merged_into: [seed_id]`
- Credits allocated to original author (see AI-09)

### 7.9 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Embedding unavailable** | Proceed without duplicate check; flag for manual moderation review |
| **Tandang search timeout** | Proceed without duplicate check; async retry within 24h |
| **No matches found** | Return empty `matches` array; show no pill; proceed normally |

---


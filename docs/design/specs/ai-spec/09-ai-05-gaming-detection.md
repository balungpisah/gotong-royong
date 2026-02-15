> [← Back to AI Spec index](../AI-SPEC-v0.2.md)

## 9. AI-05: Gaming Pattern Detection

### 9.1 Property Table

| Property | Value |
|---|---|
| **ID** | AI-05 |
| **Name** | Gaming Pattern Detection |
| **Trigger** | Continuous (async, non-blocking) |
| **UI Location** | None (background process); flags show in Moderator dashboard |
| **Interaction Mode** | Batch processing (hourly) |
| **Latency Budget** | N/A (non-urgent) |
| **Model Tier** | Statistical + optional LLM |
| **UI-UX-SPEC Ref** | None (no UI) |

### 9.2 Purpose

**AI-05 detects coordinated abuse, false endorsements, and reputation gaming** by analyzing patterns across users and seeds over time. This protects the integrity of Gotong Royong's decision-making and credit system.

### 9.3 Gaming Patterns to Detect

| Pattern | Examples | Detection Method |
|---|---|---|
| **Sock puppet networks** | Same person with multiple accounts voting on seeds | Device fingerprint + IP + voting pattern correlation |
| **Coordinated voting** | 10 new accounts all vote "Setuju" on same seed within 5 min | Temporal clustering + account age |
| **Fake endorsements** | Account created today with 100 "Jaminkan" (vouches) | Account age vs. endorsement count ratio |
| **Discussion bombing** | Same user posts 50 messages in 1 hour | Frequency + content repetition |
| **Report abuse** | Same user files 50 reports in 1 day | Report volume + false report rate |
| **Location spoofing** | User claims 10 different locations across 100km in 1 hour | Geographic impossibility detection |
| **Credit stacking** | Same task completed by same user 100 times for points | Task repetition + outcome patterns |

### 9.4 Input

```json
{
  "lookback_hours": "int (default: 24)",
  "seed_ids": ["string array (optional; if not provided, scan all seeds)"],
  "user_ids": ["string array (optional; if not provided, scan all users)"],
  "focus_metric": "enum: voting | endorsement | reporting | discussion | location | credit"
}
```

### 9.5 Output

```json
{
  "flags": [
    {
      "flag_id": "string (unique)",
      "severity": "enum: low | medium | high | critical",
      "pattern_type": "enum: sock_puppet | coordinated_voting | fake_endorsement | discussion_bombing | report_abuse | location_spoofing | credit_stacking",
      "users_involved": ["string array of user_ids"],
      "seeds_involved": ["string array of seed_ids"],
      "evidence": {
        "metric": "string",
        "baseline": "number",
        "observed": "number",
        "z_score": "float (statistical significance)"
      },
      "recommendation": "enum: investigate | escalate | auto_block | none",
      "reasoning": "string"
    }
  ],
  "summary": {
    "total_flags": "int",
    "critical_count": "int",
    "lookback_period": "string (e.g., '2026-02-14T00:00Z to 2026-02-15T00:00Z')"
  }
}
```

### 9.6 Statistical Baselines

| Pattern | Baseline | Alert Threshold | Method |
|---|---|---|---|
| **Votes per user per seed** | μ = 1.2, σ = 0.5 | > μ + 3σ | Z-score |
| **Endorsements per day** | μ = 2, σ = 1 | > 10 (new account age < 7d) | Ratio test |
| **Discussion messages per hour** | μ = 1, σ = 0.8 | > 20 | Frequency test |
| **Reports per day** | μ = 0.5, σ = 0.3 | > 10 + false report ratio | Volume + accuracy |
| **Geographic velocity** | Max realistic: 100 km/hour | > 200 km/hour detected | Great-circle distance |

### 9.7 Fallback Behavior

| Scenario | Behavior |
|---|---|
| **Baseline data insufficient** (<100 users) | Reduce alert thresholds; prioritize manual review |
| **Computation timeout** | Defer to next hourly batch; log partial results |
| **Persistent flag** (same pattern, 3+ days) | Escalate to moderator for manual action |

---


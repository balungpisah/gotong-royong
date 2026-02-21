# Implementation Prompt: Dukung→J-Score Retroactive Pathway

> **Target**: tandang/markov-engine
> **Priority**: MEDIUM | **Effort**: MEDIUM-HIGH
> **Prerequisite**: None (independent, but conceptually builds on the Dukung/Jamin split)
> **Branch suggestion**: `feat/dukung-j-score-pathway`

---

## Objective

Create a new J-score update pathway where the **outcome** of projects a user supported (Dukung'd) retroactively affects their Judgment score. This gives Dukung lightweight "skin in the game" without making it enter the trust graph.

**This adds a new input channel to JudgmentScore. It does NOT create trust graph edges, does NOT modify PageRank, and does NOT change how existing VouchOutcome works.**

---

## Rationale

Currently J-score only updates from `VouchOutcome`:

```
Good vouch:   +0.02 J
Poor vouch:   -0.05 J
Slashed vouch: -0.10 J
Fraud vouch:  -0.20 J
```

But "Judgment" should also capture: **can you identify quality projects?** If you Dukung a project that later succeeds, your judgment was good. If it gets slashed, your judgment was poor.

This creates a lightweight "prediction market" effect — Dukung has consequences, but much smaller than Jamin.

---

## Design

### New JudgmentEvent Variants

**File**: wherever `JudgmentScore` update events are defined

```rust
pub enum JudgmentEvent {
    // ---- EXISTING ----
    /// Vouch target performed well
    VouchGood,          // +0.02
    /// Vouch target performed poorly
    VouchPoor,          // -0.05
    /// Vouch target was slashed
    VouchSlashed,       // -0.10
    /// Vouch target committed fraud
    VouchFraud,         // -0.20

    // ---- NEW ----
    /// User Dukung'd a project that was later verified/completed successfully
    DukungSuccess,      // +0.01
    /// User Dukung'd a project that was later slashed
    DukungSlashed,      // -0.02
}
```

### Impact Values

| Event | J-Score Change | Rationale |
|---|---|---|
| `DukungSuccess` | **+0.01** | Half of VouchGood — Dukung is a lighter commitment |
| `DukungSlashed` | **-0.02** | Less than half of VouchPoor — Dukung shouldn't be punishing |

**Asymmetry is intentional**: The downside (-0.02) is twice the upside (+0.01) to discourage blind Dukung-everything behavior, but both are small enough that casual users won't be significantly affected.

### Processing Flow

```
1. User Dukung's a project/witness (recorded in GR database, NOT in tandang)
2. Time passes...
3. Project reaches terminal state:
   - Verified/Completed ✅ → GR calls tandang: report_dukung_outcomes(witness_id, "success", [user_ids])
   - Slashed ❌ → GR calls tandang: report_dukung_outcomes(witness_id, "slashed", [user_ids])
4. Tandang applies J-score adjustment to each user who Dukung'd
5. Done — no trust graph changes, no PageRank recalculation needed
```

---

## Implementation Steps

### Step 1: Add JudgmentEvent variants

**File**: wherever JudgmentEvent/JudgmentScore is defined (likely `reputation/judgment_score.rs` or similar)

```rust
impl JudgmentScore {
    pub fn apply_event(&mut self, event: JudgmentEvent) {
        let delta = match event {
            // Existing
            JudgmentEvent::VouchGood => dec!(0.02),
            JudgmentEvent::VouchPoor => dec!(-0.05),
            JudgmentEvent::VouchSlashed => dec!(-0.10),
            JudgmentEvent::VouchFraud => dec!(-0.20),

            // New — Dukung outcomes
            JudgmentEvent::DukungSuccess => dec!(0.01),
            JudgmentEvent::DukungSlashed => dec!(-0.02),
        };

        self.score = (self.score + delta).clamp(dec!(0.0), dec!(1.0));
    }
}
```

### Step 2: Create new API endpoint / service method

**New function** (in an appropriate service layer):

```rust
/// Report the outcome of a Dukung'd project to update J-scores.
/// Called by GR when a witness/project reaches a terminal state.
///
/// # Arguments
/// * `witness_id` - The project/witness that completed
/// * `outcome` - Whether it was verified or slashed
/// * `dukung_user_ids` - All users who Dukung'd this project
/// * `completed_at` - When the project reached terminal state
///
/// # Returns
/// Number of users whose J-score was updated
pub fn report_dukung_outcomes(
    &mut self,
    witness_id: Uuid,
    outcome: DukungOutcome,
    dukung_user_ids: Vec<UserId>,
    completed_at: DateTime<Utc>,
) -> Result<u32, ReputationError> {
    let event = match outcome {
        DukungOutcome::Verified => JudgmentEvent::DukungSuccess,
        DukungOutcome::Slashed => JudgmentEvent::DukungSlashed,
    };

    let mut updated_count = 0u32;

    for user_id in dukung_user_ids {
        // Optional: time limit check
        // Skip if Dukung was too long ago (see Step 4)

        if let Some(reputation) = self.get_user_reputation_mut(&user_id) {
            reputation.judgment_mut().apply_event(event.clone());
            updated_count += 1;
        }
        // If user doesn't exist in tandang, silently skip
        // (they may have been a GR-only user with no tandang profile yet)
    }

    Ok(updated_count)
}
```

### Step 3: Define the DukungOutcome enum

```rust
/// Terminal state of a Dukung'd project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DukungOutcome {
    /// Project was verified / completed successfully
    Verified,
    /// Project was slashed / found problematic
    Slashed,
}
```

### Step 4: Optional time limit

Should Dukung interactions expire? If a user Dukung'd a project 2 years ago and it finally completes now, should it still count?

**Recommended**: Yes, apply a time limit. Dukung interactions older than 90 days at the time of project completion are excluded.

```rust
const DUKUNG_RELEVANCE_WINDOW_DAYS: i64 = 90;

fn is_dukung_relevant(dukung_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> bool {
    let age = completed_at - dukung_at;
    age.num_days() <= DUKUNG_RELEVANCE_WINDOW_DAYS
}
```

If time limit is applied, GR must send `dukung_at` timestamps along with user IDs:

```rust
pub struct DukungRecord {
    pub user_id: UserId,
    pub dukung_at: DateTime<Utc>,
}

pub fn report_dukung_outcomes(
    &mut self,
    witness_id: Uuid,
    outcome: DukungOutcome,
    dukung_records: Vec<DukungRecord>,
    completed_at: DateTime<Utc>,
) -> Result<u32, ReputationError> {
    let event = match outcome {
        DukungOutcome::Verified => JudgmentEvent::DukungSuccess,
        DukungOutcome::Slashed => JudgmentEvent::DukungSlashed,
    };

    let mut updated_count = 0u32;

    for record in dukung_records {
        if !is_dukung_relevant(record.dukung_at, completed_at) {
            continue; // Too old, skip
        }

        if let Some(reputation) = self.get_user_reputation_mut(&record.user_id) {
            reputation.judgment_mut().apply_event(event.clone());
            updated_count += 1;
        }
    }

    Ok(updated_count)
}
```

### Step 5: Rate limiting / anti-gaming

Prevent gaming via mass-Dukung:

```rust
/// Maximum number of Dukung→J-score updates per user per day.
/// Prevents someone from Dukung-ing 100 projects just for J-score farming.
const MAX_DUKUNG_J_UPDATES_PER_DAY: u32 = 5;
```

This is enforced at the service layer — track how many Dukung J-updates a user has received today, skip if over limit.

---

## API Contract (What GR Sends)

### HTTP Endpoint (or gRPC, match existing pattern)

```
POST /api/v1/dukung-outcomes
```

### Request Body

```json
{
  "witness_id": "uuid-of-witness",
  "outcome": "verified",
  "completed_at": "2025-02-21T15:00:00Z",
  "dukung_records": [
    { "user_id": "user-uuid-1", "dukung_at": "2025-01-15T10:00:00Z" },
    { "user_id": "user-uuid-2", "dukung_at": "2025-02-01T08:30:00Z" },
    { "user_id": "user-uuid-3", "dukung_at": "2025-02-10T14:00:00Z" }
  ]
}
```

### Response

```json
{
  "updated_count": 3,
  "skipped_expired": 0,
  "skipped_rate_limited": 0,
  "skipped_not_found": 0
}
```

---

## Tests

### Test 1: DukungSuccess increases J-score

```rust
#[test]
fn test_dukung_success_increases_j_score() {
    let mut j = JudgmentScore::default(); // starts at 0.5
    let initial = j.score();

    j.apply_event(JudgmentEvent::DukungSuccess);

    assert_eq!(j.score(), initial + dec!(0.01));
    assert_eq!(j.score(), dec!(0.51));
}
```

### Test 2: DukungSlashed decreases J-score

```rust
#[test]
fn test_dukung_slashed_decreases_j_score() {
    let mut j = JudgmentScore::default(); // 0.5
    let initial = j.score();

    j.apply_event(JudgmentEvent::DukungSlashed);

    assert_eq!(j.score(), initial - dec!(0.02));
    assert_eq!(j.score(), dec!(0.48));
}
```

### Test 3: Dukung impact is smaller than Vouch impact

```rust
#[test]
fn test_dukung_impact_smaller_than_vouch() {
    let mut j_dukung = JudgmentScore::default();
    let mut j_vouch = JudgmentScore::default();

    // Positive outcomes
    j_dukung.apply_event(JudgmentEvent::DukungSuccess);
    j_vouch.apply_event(JudgmentEvent::VouchGood);

    assert!(j_dukung.score() < j_vouch.score(),
        "Dukung success (+0.01) should have less impact than VouchGood (+0.02)");

    // Negative outcomes
    let mut j_dukung_neg = JudgmentScore::default();
    let mut j_vouch_neg = JudgmentScore::default();

    j_dukung_neg.apply_event(JudgmentEvent::DukungSlashed);
    j_vouch_neg.apply_event(JudgmentEvent::VouchPoor);

    assert!(j_dukung_neg.score() > j_vouch_neg.score(),
        "Dukung slashed (-0.02) should be less punishing than VouchPoor (-0.05)");
}
```

### Test 4: J-score clamps to [0, 1]

```rust
#[test]
fn test_dukung_j_score_clamps_at_boundaries() {
    // Test floor
    let mut j_low = JudgmentScore::new(dec!(0.01));
    j_low.apply_event(JudgmentEvent::DukungSlashed); // -0.02 → would be -0.01
    assert_eq!(j_low.score(), dec!(0.0), "J-score should not go below 0");

    // Test ceiling
    let mut j_high = JudgmentScore::new(dec!(0.995));
    j_high.apply_event(JudgmentEvent::DukungSuccess); // +0.01 → would be 1.005
    assert_eq!(j_high.score(), dec!(1.0), "J-score should not exceed 1");
}
```

### Test 5: report_dukung_outcomes batch processing

```rust
#[test]
fn test_report_dukung_outcomes_batch() {
    let mut service = ReputationService::new();

    // Create 3 users with tandang profiles
    let user_a = UserId::new_v4();
    let user_b = UserId::new_v4();
    let user_c = UserId::new_v4();
    service.create_user(user_a);
    service.create_user(user_b);
    service.create_user(user_c);

    let initial_j = dec!(0.5);
    let witness_id = Uuid::new_v4();
    let now = Utc::now();

    let records = vec![
        DukungRecord { user_id: user_a, dukung_at: now - Duration::days(30) },
        DukungRecord { user_id: user_b, dukung_at: now - Duration::days(15) },
        DukungRecord { user_id: user_c, dukung_at: now - Duration::days(5) },
    ];

    let result = service.report_dukung_outcomes(
        witness_id,
        DukungOutcome::Verified,
        records,
        now,
    ).unwrap();

    assert_eq!(result, 3);

    // All 3 users should have J = 0.51
    for user_id in [user_a, user_b, user_c] {
        let rep = service.get_user_reputation(&user_id).unwrap();
        assert_eq!(rep.judgment().score(), initial_j + dec!(0.01));
    }
}
```

### Test 6: Time limit — expired Dukung is skipped

```rust
#[test]
fn test_expired_dukung_is_skipped() {
    let mut service = ReputationService::new();

    let user_recent = UserId::new_v4();
    let user_old = UserId::new_v4();
    service.create_user(user_recent);
    service.create_user(user_old);

    let now = Utc::now();
    let witness_id = Uuid::new_v4();

    let records = vec![
        DukungRecord { user_id: user_recent, dukung_at: now - Duration::days(30) }, // within 90 days
        DukungRecord { user_id: user_old, dukung_at: now - Duration::days(120) },   // outside 90 days
    ];

    let result = service.report_dukung_outcomes(
        witness_id,
        DukungOutcome::Verified,
        records,
        now,
    ).unwrap();

    assert_eq!(result, 1); // Only user_recent was updated

    let rep_recent = service.get_user_reputation(&user_recent).unwrap();
    assert_eq!(rep_recent.judgment().score(), dec!(0.51)); // Updated

    let rep_old = service.get_user_reputation(&user_old).unwrap();
    assert_eq!(rep_old.judgment().score(), dec!(0.50)); // Not updated
}
```

### Test 7: Unknown user is silently skipped

```rust
#[test]
fn test_unknown_user_silently_skipped() {
    let mut service = ReputationService::new();

    let known_user = UserId::new_v4();
    let unknown_user = UserId::new_v4();
    service.create_user(known_user);
    // unknown_user is NOT created in tandang

    let now = Utc::now();
    let records = vec![
        DukungRecord { user_id: known_user, dukung_at: now - Duration::days(10) },
        DukungRecord { user_id: unknown_user, dukung_at: now - Duration::days(10) },
    ];

    let result = service.report_dukung_outcomes(
        Uuid::new_v4(),
        DukungOutcome::Verified,
        records,
        now,
    ).unwrap();

    assert_eq!(result, 1); // Only known_user was updated
}
```

### Test 8: Existing VouchOutcome pathway unchanged

```rust
#[test]
fn test_existing_vouch_outcomes_unchanged() {
    let mut j = JudgmentScore::default();

    // Verify existing events still have original impacts
    j.apply_event(JudgmentEvent::VouchGood);
    assert_eq!(j.score(), dec!(0.52));

    j.apply_event(JudgmentEvent::VouchPoor);
    assert_eq!(j.score(), dec!(0.47));

    j.apply_event(JudgmentEvent::VouchSlashed);
    assert_eq!(j.score(), dec!(0.37));

    j.apply_event(JudgmentEvent::VouchFraud);
    assert_eq!(j.score(), dec!(0.17));
}
```

### Test 9: Trust graph NOT affected by Dukung outcomes

```rust
#[test]
fn test_dukung_outcomes_do_not_touch_trust_graph() {
    let mut service = ReputationService::new();
    let user = UserId::new_v4();
    service.create_user(user);

    let graph_before = service.trust_graph().edge_count();

    service.report_dukung_outcomes(
        Uuid::new_v4(),
        DukungOutcome::Verified,
        vec![DukungRecord { user_id: user, dukung_at: Utc::now() }],
        Utc::now(),
    ).unwrap();

    let graph_after = service.trust_graph().edge_count();

    assert_eq!(graph_before, graph_after,
        "Dukung outcomes must NOT create trust graph edges");
}
```

---

## Validation Checklist

- [ ] `JudgmentEvent::DukungSuccess` and `DukungSlashed` variants added
- [ ] `apply_event()` handles new variants with correct deltas (+0.01 / -0.02)
- [ ] `DukungOutcome` enum created (Verified / Slashed)
- [ ] `report_dukung_outcomes()` function implemented
- [ ] Time limit (90 days) enforced
- [ ] Unknown users silently skipped (no error)
- [ ] Existing VouchOutcome pathway unchanged (Test 8)
- [ ] Trust graph NOT affected (Test 9)
- [ ] J-score clamping works at boundaries (Test 4)
- [ ] API endpoint / service method accessible from GR
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## What NOT To Do

- **Do NOT** add Dukung records to the trust graph — Dukung is GR-native, not a trust edge
- **Do NOT** change how existing VouchOutcome impacts work — those values are unchanged
- **Do NOT** trigger PageRank recalculation — J-score is a standalone metric, not a graph weight
- **Do NOT** make this synchronous / blocking — batch processing is fine, even preferred

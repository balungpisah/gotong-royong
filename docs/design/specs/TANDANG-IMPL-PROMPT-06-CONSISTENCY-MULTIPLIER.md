# Implementation Prompt: Consistency Multiplier

> **Target**: tandang/markov-engine
> **Priority**: LOW | **Effort**: MEDIUM
> **Prerequisite**: None (independent, but benefits from ActivityTracker improvements in Prompt 05)
> **Branch suggestion**: `feat/consistency-multiplier`

---

## Objective

Add a `ConsistencyMultiplier` to `UserReputation` that rewards sustained, regular activity. Users who are consistently active over weeks receive a small boost to **incoming** vouch effectiveness. This rewards reliability without giving disproportionate influence.

**This modifies incoming vouch weight calculation. It does NOT change the trust graph structure or PageRank algorithm itself — it's a pre-processing modifier on edge weights.**

---

## Design

### Core Concept

```
A user who has been active every week for 10 weeks
gets a 1.20× multiplier on incoming vouches.

A user who was active for 3 weeks then disappeared
gets a 1.06× multiplier (and it resets after 2 inactive weeks).
```

### Formula

```
multiplier = 1.0 + min(0.20, active_weeks_streak × 0.02)

Range: [1.0, 1.2]
  0 weeks  → 1.00× (no bonus)
  1 week   → 1.02×
  5 weeks  → 1.10×
  10+ weeks → 1.20× (cap)

Reset: After 2 consecutive inactive weeks, streak resets to 0
```

### Applied To

- **Incoming** vouch weight only: when someone vouches FOR a consistent user, the vouch is slightly more effective
- **NOT** applied to outgoing vouches (you don't get more influential by being consistent — that's tier/percentile's job)
- **NOT** applied to Dukung (which doesn't enter tandang)

---

## Implementation Steps

### Step 1: Define the struct

**File**: `crates/domain/src/reputation/user_reputation/mod.rs` (or new file `consistency.rs`)

```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Tracks consecutive active weeks and computes a multiplier
/// applied to incoming vouch weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyMultiplier {
    /// Number of consecutive weeks with qualifying activity
    active_weeks_streak: u32,
    /// Last week that was counted as active (ISO week number + year)
    last_active_week: Option<(i32, u32)>, // (year, week_number)
    /// Cached multiplier value
    multiplier: Decimal,
}

/// Minimum interactions per week to count as "active"
const CONSISTENCY_WEEKLY_THRESHOLD: u32 = 2;

/// Bonus per active week
const CONSISTENCY_BONUS_PER_WEEK: Decimal = dec!(0.02);

/// Maximum bonus (cap)
const CONSISTENCY_BONUS_CAP: Decimal = dec!(0.20);

/// Consecutive inactive weeks before streak resets
const CONSISTENCY_RESET_AFTER_WEEKS: u32 = 2;

impl ConsistencyMultiplier {
    pub fn new() -> Self {
        Self {
            active_weeks_streak: 0,
            last_active_week: None,
            multiplier: dec!(1.0),
        }
    }

    /// Current multiplier value [1.0, 1.2]
    pub fn multiplier(&self) -> Decimal {
        self.multiplier
    }

    /// Current streak in weeks
    pub fn streak(&self) -> u32 {
        self.active_weeks_streak
    }

    /// Update streak based on this week's activity.
    /// Called by the weekly maintenance job.
    pub fn record_week(
        &mut self,
        year: i32,
        week: u32,
        interaction_count: u32,
    ) {
        let is_active = interaction_count >= CONSISTENCY_WEEKLY_THRESHOLD;

        if is_active {
            // Check if this is a consecutive week
            let is_consecutive = match self.last_active_week {
                Some((prev_year, prev_week)) => {
                    // Simple check: either same year + next week,
                    // or year rollover + week 1
                    (year == prev_year && week == prev_week + 1)
                        || (year == prev_year + 1 && week == 1 && prev_week >= 52)
                }
                None => true, // First active week starts the streak
            };

            if is_consecutive {
                self.active_weeks_streak += 1;
            } else {
                // Gap detected — check if within reset tolerance
                let gap = self.weeks_since_last_active(year, week);
                if gap <= CONSISTENCY_RESET_AFTER_WEEKS {
                    self.active_weeks_streak += 1; // Forgive small gaps
                } else {
                    self.active_weeks_streak = 1; // Reset, this week counts as first
                }
            }

            self.last_active_week = Some((year, week));
        } else {
            // Inactive week — check if streak should reset
            let gap = match self.last_active_week {
                Some((prev_year, prev_week)) => self.weeks_between(prev_year, prev_week, year, week),
                None => 0,
            };

            if gap > CONSISTENCY_RESET_AFTER_WEEKS {
                self.active_weeks_streak = 0;
            }
            // Otherwise, streak is preserved (within tolerance)
        }

        self.recalculate_multiplier();
    }

    fn recalculate_multiplier(&mut self) {
        let bonus = (Decimal::from(self.active_weeks_streak) * CONSISTENCY_BONUS_PER_WEEK)
            .min(CONSISTENCY_BONUS_CAP);
        self.multiplier = dec!(1.0) + bonus;
    }

    fn weeks_since_last_active(&self, current_year: i32, current_week: u32) -> u32 {
        match self.last_active_week {
            Some((prev_year, prev_week)) => self.weeks_between(prev_year, prev_week, current_year, current_week),
            None => u32::MAX,
        }
    }

    fn weeks_between(&self, y1: i32, w1: u32, y2: i32, w2: u32) -> u32 {
        let total_w1 = y1 as u32 * 52 + w1;
        let total_w2 = y2 as u32 * 52 + w2;
        total_w2.saturating_sub(total_w1)
    }
}

impl Default for ConsistencyMultiplier {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 2: Add to UserReputation

**File**: `crates/domain/src/reputation/user_reputation/mod.rs`

```rust
pub struct UserReputation {
    // ... existing fields ...
    consistency: ConsistencyMultiplier, // NEW
}

impl UserReputation {
    pub fn consistency(&self) -> &ConsistencyMultiplier {
        &self.consistency
    }

    pub fn consistency_mut(&mut self) -> &mut ConsistencyMultiplier {
        &mut self.consistency
    }
}
```

### Step 3: Apply to incoming vouch weight

**File**: wherever `VouchWeight::effective_weight()` or incoming vouch processing happens

```rust
// When processing an incoming vouch for user X:
fn process_incoming_vouch(
    &self,
    vouch: &Vouch,
    vouchee_reputation: &UserReputation,
) -> Decimal {
    let base_weight = vouch.weight().effective_weight();

    // Existing dampening: reciprocity, skeptical, temporal
    let dampened = self.apply_existing_dampening(base_weight, vouch);

    // NEW: Apply consistency multiplier of the VOUCHEE (not voucher)
    let with_consistency = dampened * vouchee_reputation.consistency().multiplier();

    with_consistency
}
```

**Important**: The multiplier is of the **vouchee** (person being vouched for), not the voucher. This rewards the person who has been consistently active, not the person giving the vouch.

### Step 4: Weekly maintenance job

**File**: wherever periodic jobs are defined (near DecayJob)

```rust
/// Weekly job: update ConsistencyMultiplier for all users.
/// Should run once per week (e.g., Sunday midnight UTC).
pub fn update_consistency_multipliers(
    users: &mut [UserReputation],
    activity_tracker: &ActivityTracker,
    current_year: i32,
    current_week: u32,
) {
    for user in users {
        let weekly_count = activity_tracker.weekly_interaction_count(
            user.user_id(),
            current_year,
            current_week,
        );

        user.consistency_mut().record_week(
            current_year,
            current_week,
            weekly_count,
        );
    }
}
```

---

## Tests

### Test 1: Fresh user has no bonus

```rust
#[test]
fn test_new_user_no_consistency_bonus() {
    let cm = ConsistencyMultiplier::new();
    assert_eq!(cm.multiplier(), dec!(1.0));
    assert_eq!(cm.streak(), 0);
}
```

### Test 2: Multiplier grows with consecutive weeks

```rust
#[test]
fn test_multiplier_grows_with_streak() {
    let mut cm = ConsistencyMultiplier::new();

    // Simulate 5 consecutive active weeks
    for week in 1..=5u32 {
        cm.record_week(2025, week, 3); // 3 interactions per week
    }

    assert_eq!(cm.streak(), 5);
    assert_eq!(cm.multiplier(), dec!(1.10)); // 1.0 + 5 × 0.02
}
```

### Test 3: Multiplier caps at 1.20

```rust
#[test]
fn test_multiplier_caps_at_120() {
    let mut cm = ConsistencyMultiplier::new();

    // 15 consecutive active weeks
    for week in 1..=15u32 {
        cm.record_week(2025, week, 5);
    }

    assert_eq!(cm.streak(), 15);
    assert_eq!(cm.multiplier(), dec!(1.20)); // Capped, not 1.30

    // 20 weeks: still 1.20
    for week in 16..=20u32 {
        cm.record_week(2025, week, 5);
    }
    assert_eq!(cm.multiplier(), dec!(1.20));
}
```

### Test 4: Streak resets after 2 inactive weeks

```rust
#[test]
fn test_streak_resets_after_gap() {
    let mut cm = ConsistencyMultiplier::new();

    // 5 active weeks
    for week in 1..=5u32 {
        cm.record_week(2025, week, 3);
    }
    assert_eq!(cm.streak(), 5);
    assert_eq!(cm.multiplier(), dec!(1.10));

    // Week 6: inactive
    cm.record_week(2025, 6, 0);
    // Within tolerance (1 week gap) — streak preserved
    assert_eq!(cm.streak(), 5);

    // Week 7: inactive
    cm.record_week(2025, 7, 0);
    // Within tolerance (2 week gap) — streak preserved
    assert_eq!(cm.streak(), 5);

    // Week 8: inactive — 3 weeks gap, exceeds tolerance
    cm.record_week(2025, 8, 0);
    assert_eq!(cm.streak(), 0);
    assert_eq!(cm.multiplier(), dec!(1.0));
}
```

### Test 5: One inactive week is forgiven

```rust
#[test]
fn test_one_inactive_week_forgiven() {
    let mut cm = ConsistencyMultiplier::new();

    // Weeks 1-3: active
    for week in 1..=3u32 {
        cm.record_week(2025, week, 3);
    }
    assert_eq!(cm.streak(), 3);

    // Week 4: inactive (gap = 1, within tolerance)
    cm.record_week(2025, 4, 0);

    // Week 5: active again — streak should continue
    cm.record_week(2025, 5, 3);
    assert_eq!(cm.streak(), 4); // Continues from 3, not reset
}
```

### Test 6: Below-threshold activity doesn't count

```rust
#[test]
fn test_below_threshold_not_active() {
    let mut cm = ConsistencyMultiplier::new();

    // Week 1: 1 interaction (below threshold of 2)
    cm.record_week(2025, 1, 1);
    assert_eq!(cm.streak(), 0);
    assert_eq!(cm.multiplier(), dec!(1.0));

    // Week 2: exactly at threshold
    cm.record_week(2025, 2, 2);
    assert_eq!(cm.streak(), 1);
    assert_eq!(cm.multiplier(), dec!(1.02));
}
```

### Test 7: Multiplier applies to incoming vouch weight

```rust
#[test]
fn test_consistency_applied_to_incoming_vouch() {
    let voucher = UserId::new_v4();
    let vouchee = UserId::new_v4();

    let mut vouchee_rep = UserReputation::new(vouchee);

    // Vouchee has 5-week streak → 1.10× multiplier
    for week in 1..=5u32 {
        vouchee_rep.consistency_mut().record_week(2025, week, 3);
    }
    assert_eq!(vouchee_rep.consistency().multiplier(), dec!(1.10));

    // Positive vouch with base weight 1.0
    let vouch = Vouch::new(
        voucher, vouchee,
        VouchType::Positive,
        VouchContext::Direct {},
    );

    let base_weight = vouch.weight().effective_weight(); // 1.0
    let with_consistency = base_weight * vouchee_rep.consistency().multiplier();

    assert_eq!(with_consistency, dec!(1.10),
        "Incoming vouch should be boosted by vouchee's consistency");
}
```

### Test 8: Multiplier does NOT apply to outgoing vouches

```rust
#[test]
fn test_consistency_not_applied_to_outgoing() {
    // This is a design verification test:
    // The voucher's consistency multiplier should NOT affect
    // the weight of vouches they give to others.

    let consistent_voucher = UserId::new_v4();
    let inconsistent_voucher = UserId::new_v4();
    let vouchee = UserId::new_v4();

    // Both vouch for same person with same type
    let vouch_a = Vouch::new(
        consistent_voucher, vouchee,
        VouchType::Positive,
        VouchContext::Direct {},
    );
    let vouch_b = Vouch::new(
        inconsistent_voucher, vouchee,
        VouchType::Positive,
        VouchContext::Direct {},
    );

    // Base weights should be identical — voucher consistency doesn't matter
    assert_eq!(
        vouch_a.weight().effective_weight(),
        vouch_b.weight().effective_weight(),
        "Voucher's consistency should not affect outgoing vouch weight"
    );
}
```

### Test 9: Serialization roundtrip

```rust
#[test]
fn test_consistency_multiplier_serialization() {
    let mut cm = ConsistencyMultiplier::new();
    for week in 1..=7u32 {
        cm.record_week(2025, week, 5);
    }

    let json = serde_json::to_string(&cm).unwrap();
    let deserialized: ConsistencyMultiplier = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.streak(), 7);
    assert_eq!(deserialized.multiplier(), dec!(1.14));
    assert_eq!(deserialized.last_active_week, Some((2025, 7)));
}
```

### Test 10: Year rollover

```rust
#[test]
fn test_year_rollover_continues_streak() {
    let mut cm = ConsistencyMultiplier::new();

    // Active from week 50 of 2024
    cm.record_week(2024, 50, 3);
    cm.record_week(2024, 51, 3);
    cm.record_week(2024, 52, 3);

    // Rolls into 2025
    cm.record_week(2025, 1, 3);
    cm.record_week(2025, 2, 3);

    assert_eq!(cm.streak(), 5, "Streak should continue across year boundary");
    assert_eq!(cm.multiplier(), dec!(1.10));
}
```

---

## Validation Checklist

- [ ] `ConsistencyMultiplier` struct implemented with streak tracking
- [ ] `CONSISTENCY_WEEKLY_THRESHOLD` = 2, `CONSISTENCY_BONUS_PER_WEEK` = 0.02, `CONSISTENCY_BONUS_CAP` = 0.20
- [ ] Fresh user starts at 1.0× (Test 1)
- [ ] Multiplier grows correctly (Test 2)
- [ ] Cap enforced at 1.20× (Test 3)
- [ ] Streak resets after 2+ inactive weeks (Test 4)
- [ ] 1 inactive week forgiven (Test 5)
- [ ] Below-threshold activity doesn't count (Test 6)
- [ ] Applied to incoming vouch weight only (Test 7)
- [ ] NOT applied to outgoing vouches (Test 8)
- [ ] Serialization works (Test 9)
- [ ] Year rollover handled (Test 10)
- [ ] Added to `UserReputation` struct
- [ ] Weekly maintenance job created
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## What NOT To Do

- **Do NOT** apply multiplier to outgoing vouches — consistency doesn't make you more influential, just more trusted
- **Do NOT** make the multiplier too large — 1.2× cap is intentionally conservative
- **Do NOT** count trivial actions (login, views) — threshold of 2 meaningful interactions per week
- **Do NOT** make this affect tier calculation — tier is percentile-based, consistency is a weight modifier

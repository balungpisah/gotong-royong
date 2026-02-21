# Implementation Prompt: Context-Aware Genesis Decay

> **Target**: tandang/markov-engine
> **Priority**: LOW | **Effort**: LOW
> **Prerequisite**: None (independent)
> **Branch suggestion**: `feat/context-aware-genesis-decay`

---

## Objective

Modify genesis weight decay so it **pauses during active months**. Currently genesis weight decays unconditionally at `W_g(t) = W_g(0) × 0.9^months`. Active genesis users who contribute regularly should not lose their earned initial trust.

**This changes the decay formula only. It does NOT affect the trust graph, PageRank, or any other reputation component.**

---

## Current Behavior

```rust
// Genesis weight decays every month regardless of activity
// W_g(t) = W_g(0) × 0.9^months_elapsed
```

A genesis user who joined 12 months ago has their weight reduced to `0.9^12 ≈ 0.282` of the original — even if they've been contributing every single day.

---

## Proposed Behavior

```rust
// Genesis weight decays only during INACTIVE months
// W_g(t) = W_g(0) × 0.9^inactive_months
//
// Where inactive_month = a month with fewer than ACTIVITY_THRESHOLD meaningful interactions
```

A genesis user active for 10 of 12 months would only decay by 2 inactive months: `0.9^2 = 0.81` instead of `0.9^12 = 0.282`.

---

## Implementation Steps

### Step 1: Define "active month"

**File**: wherever genesis decay logic lives (likely `decay/` module)

```rust
/// Minimum number of meaningful interactions to count a month as "active".
/// Prevents gaming via 1 trivial action per month.
const GENESIS_ACTIVE_MONTH_THRESHOLD: u32 = 3;

/// Meaningful interaction types that count toward active month:
/// - Create/participate in a witness
/// - Submit evidence
/// - Give/receive a Jamin (vouch)
/// - Give a Dukung (support)
/// - Serve on jury
/// - Submit Proof-of-Reality
///
/// NOT counted:
/// - Login only
/// - Profile views
/// - Reading feed without interaction
```

### Step 2: Modify the decay function

**File**: wherever `decay_genesis_weight` or equivalent is implemented

```rust
// BEFORE:
fn apply_genesis_decay(
    &self,
    genesis_weight: Decimal,
    months_since_genesis: u32,
) -> Decimal {
    genesis_weight * dec!(0.9).powd(Decimal::from(months_since_genesis))
}

// AFTER:
fn apply_genesis_decay(
    &self,
    genesis_weight: Decimal,
    months_since_genesis: u32,
    active_month_count: u32,
) -> Decimal {
    let inactive_months = months_since_genesis.saturating_sub(active_month_count);
    if inactive_months == 0 {
        genesis_weight // No decay — fully active
    } else {
        genesis_weight * dec!(0.9).powd(Decimal::from(inactive_months))
    }
}
```

### Step 3: Wire ActivityTracker to genesis decay

`ActivityTracker` already exists in the decay module. It needs to:

1. Track monthly interaction counts per user
2. Provide `active_month_count(user_id, since: DateTime) -> u32`
3. Feed this into the genesis decay calculation

```rust
impl ActivityTracker {
    /// Count how many months since `since` the user had >= threshold interactions.
    pub fn active_month_count(
        &self,
        user_id: &UserId,
        since: DateTime<Utc>,
        threshold: u32,
    ) -> u32 {
        let monthly_counts = self.get_monthly_interaction_counts(user_id, since);
        monthly_counts
            .iter()
            .filter(|&count| *count >= threshold)
            .count() as u32
    }
}
```

### Step 4: Update the decay job

**File**: wherever the periodic decay batch job runs (likely `DecayJob`)

```rust
// In the decay job loop for genesis users:
for user in genesis_users {
    let months_since_genesis = months_between(user.created_at, now);
    let active_months = activity_tracker.active_month_count(
        &user.user_id,
        user.created_at,
        GENESIS_ACTIVE_MONTH_THRESHOLD,
    );

    let new_weight = decay_calculator.apply_genesis_decay(
        user.genesis_weight.unwrap(),
        months_since_genesis,
        active_months,
    );

    user.set_genesis_weight(new_weight);
    user.set_genesis_last_decay_at(now);
}
```

---

## Tests

### Test 1: Fully active user — no decay

```rust
#[test]
fn test_fully_active_genesis_no_decay() {
    let calc = DecayCalculator::new();
    let initial_weight = dec!(1.0);

    // 12 months elapsed, all 12 active
    let result = calc.apply_genesis_decay(initial_weight, 12, 12);

    assert_eq!(result, dec!(1.0), "Fully active genesis user should not decay");
}
```

### Test 2: Fully inactive user — standard decay

```rust
#[test]
fn test_fully_inactive_genesis_standard_decay() {
    let calc = DecayCalculator::new();
    let initial_weight = dec!(1.0);

    // 12 months elapsed, 0 active
    let result = calc.apply_genesis_decay(initial_weight, 12, 0);

    // Should be same as old formula: 0.9^12
    let expected = dec!(0.9).powd(dec!(12));
    assert_eq!(result, expected, "Fully inactive should match old decay formula");
}
```

### Test 3: Partially active user

```rust
#[test]
fn test_partially_active_genesis_decay() {
    let calc = DecayCalculator::new();
    let initial_weight = dec!(1.0);

    // 12 months elapsed, 10 active → 2 inactive months
    let result = calc.apply_genesis_decay(initial_weight, 12, 10);

    // Expected: 0.9^2 = 0.81
    let expected = dec!(0.9).powd(dec!(2));
    assert_eq!(result, expected);
}
```

### Test 4: Active months cannot exceed total months

```rust
#[test]
fn test_active_months_capped_at_total() {
    let calc = DecayCalculator::new();
    let initial_weight = dec!(1.0);

    // Edge case: active_months > months_since_genesis (shouldn't happen, but handle gracefully)
    let result = calc.apply_genesis_decay(initial_weight, 6, 10);

    // saturating_sub: 6 - 10 = 0 inactive months → no decay
    assert_eq!(result, dec!(1.0));
}
```

### Test 5: Zero months elapsed

```rust
#[test]
fn test_zero_months_no_decay() {
    let calc = DecayCalculator::new();
    let initial_weight = dec!(1.0);

    let result = calc.apply_genesis_decay(initial_weight, 0, 0);

    assert_eq!(result, dec!(1.0), "No time elapsed = no decay");
}
```

### Test 6: Activity threshold enforcement

```rust
#[test]
fn test_activity_threshold_enforcement() {
    let mut tracker = ActivityTracker::new();
    let user = UserId::new_v4();
    let genesis_date = Utc::now() - Duration::days(90); // 3 months ago

    // Month 1: 5 interactions (above threshold of 3) → active
    for _ in 0..5 {
        tracker.record_interaction(&user, genesis_date + Duration::days(15));
    }

    // Month 2: 1 interaction (below threshold) → inactive
    tracker.record_interaction(&user, genesis_date + Duration::days(45));

    // Month 3: 4 interactions (above threshold) → active
    for _ in 0..4 {
        tracker.record_interaction(&user, genesis_date + Duration::days(75));
    }

    let active_months = tracker.active_month_count(
        &user,
        genesis_date,
        GENESIS_ACTIVE_MONTH_THRESHOLD,
    );

    assert_eq!(active_months, 2, "Only months with >= 3 interactions count as active");
}
```

### Test 7: Non-genesis user unaffected

```rust
#[test]
fn test_non_genesis_user_unaffected() {
    let mut reputation = UserReputation::new(UserId::new_v4());

    // User has no genesis weight
    assert!(reputation.genesis_weight().is_none());

    // Genesis decay should be a no-op
    let decay_calc = DecayCalculator::new();
    // Should not panic or error when applied to non-genesis user
    // Implementation should check genesis_weight.is_some() before applying
}
```

### Test 8: Backward compatibility — old decay still works for fully inactive

```rust
#[test]
fn test_backward_compatibility_with_old_formula() {
    let calc = DecayCalculator::new();

    // For fully inactive users, new formula should produce identical results to old formula
    for months in 1..=24u32 {
        let old_result = dec!(1.0) * dec!(0.9).powd(Decimal::from(months));
        let new_result = calc.apply_genesis_decay(dec!(1.0), months, 0);

        assert_eq!(old_result, new_result,
            "At {} inactive months, old and new formulas should match", months);
    }
}
```

---

## Validation Checklist

- [ ] `GENESIS_ACTIVE_MONTH_THRESHOLD` constant defined (value: 3)
- [ ] `apply_genesis_decay()` accepts `active_month_count` parameter
- [ ] Fully active user experiences zero decay (Test 1)
- [ ] Fully inactive user matches old formula (Test 2, Test 8)
- [ ] Partial activity works correctly (Test 3)
- [ ] Edge cases handled: overflow (Test 4), zero months (Test 5)
- [ ] ActivityTracker provides monthly counts (Test 6)
- [ ] Non-genesis users unaffected (Test 7)
- [ ] DecayJob updated to pass active month count
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## What NOT To Do

- **Do NOT** remove the genesis decay mechanism entirely — inactive accounts still need to decay
- **Do NOT** change the 0.9 decay factor — only change WHEN it applies
- **Do NOT** modify other decay mechanisms (CompetenceScore 90-day half-life is separate)
- **Do NOT** allow a single trivial action to count as "active" — threshold of 3 meaningful interactions is intentional

# Implementation Prompt: PatternObserved VouchContext Variant

> **Target**: tandang/markov-engine
> **Priority**: HIGH | **Effort**: LOW
> **Prerequisite**: None
> **Branch suggestion**: `feat/vouch-context-pattern-observed`

---

## Objective

Add a new `PatternObserved` variant to the `VouchContext` enum. This variant records that a vouch (Jamin) was preceded by a history of project-level support (Dukung) interactions, providing an evidence trail for the trust assertion.

**This is metadata only — it does NOT change PageRank calculation, trust graph structure, or any weight computation.**

---

## Implementation Steps

### Step 1: Add the enum variant

**File**: `crates/domain/src/vouch/vouch_entity.rs`

Add to the existing `VouchContext` enum:

```rust
pub enum VouchContext {
    /// Co-witnessing a claim
    CoWitness { witness_id: Uuid },
    /// Verifier accepted executor's solution
    VerifiedExecution { task_id: Uuid },
    /// Beneficiary attests to impact
    ImpactAttestation { witness_id: Uuid },
    /// Sensemaker criteria validated
    SensemakingValidation {
        witness_id: Uuid,
        criteria_hash: String,
    },
    /// Manual / direct vouch
    Direct {},

    // ---- NEW ----

    /// Vouch backed by observable project-support history from gotong-royong.
    /// The voucher has previously "Dukung'd" (supported) multiple projects
    /// by the vouchee before making this person-level trust assertion.
    PatternObserved {
        /// How many Dukung interactions preceded this Jamin
        dukung_count: u32,
        /// When the voucher first supported the vouchee's projects
        first_dukung_at: DateTime<Utc>,
        /// Project/witness IDs that were supported (for audit trail)
        witness_ids: Vec<Uuid>,
    },
}
```

### Step 2: Update serialization

Ensure serde derives handle the new variant. The existing enum likely uses `#[serde(tag = "type")]` or similar. Add the variant with matching tag:

```rust
// If using externally tagged (default):
// "PatternObserved" will serialize as:
// { "PatternObserved": { "dukung_count": 7, "first_dukung_at": "...", "witness_ids": [...] } }

// If using internally tagged:
// { "type": "PatternObserved", "dukung_count": 7, ... }
```

Verify the existing serde strategy and match it.

### Step 3: Update any match arms

Search the codebase for `match` on `VouchContext` and add the new arm. Since VouchContext is metadata-only, most match arms should be passthrough:

```rust
// Example pattern — find all existing matches:
match &vouch.context {
    VouchContext::CoWitness { witness_id } => { /* ... */ },
    VouchContext::VerifiedExecution { task_id } => { /* ... */ },
    // ... other arms ...
    VouchContext::PatternObserved { dukung_count, first_dukung_at, witness_ids } => {
        // Same behavior as other contexts — metadata recorded, no calc impact
    },
}
```

### Step 4: Database migration (if applicable)

If VouchContext is stored as a JSONB column, no schema migration needed — the new variant serializes naturally. If it's stored as an enum column, add the new value:

```sql
-- Only if using database enum type:
ALTER TYPE vouch_context_type ADD VALUE 'pattern_observed';
```

---

## Tests

### Test 1: Serialization roundtrip

```rust
#[test]
fn test_pattern_observed_serialization_roundtrip() {
    let context = VouchContext::PatternObserved {
        dukung_count: 7,
        first_dukung_at: Utc::now() - Duration::days(120),
        witness_ids: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
    };

    let serialized = serde_json::to_string(&context).unwrap();
    let deserialized: VouchContext = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        VouchContext::PatternObserved {
            dukung_count,
            first_dukung_at: _,
            witness_ids,
        } => {
            assert_eq!(dukung_count, 7);
            assert_eq!(witness_ids.len(), 3);
        }
        _ => panic!("Deserialized to wrong variant"),
    }
}
```

### Test 2: Vouch creation with PatternObserved context

```rust
#[test]
fn test_create_vouch_with_pattern_observed() {
    let voucher = UserId::new_v4();
    let vouchee = UserId::new_v4();
    let witness_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

    let vouch = Vouch::new(
        voucher,
        vouchee,
        VouchType::Positive,
        VouchContext::PatternObserved {
            dukung_count: 5,
            first_dukung_at: Utc::now() - Duration::days(90),
            witness_ids: witness_ids.clone(),
        },
    );

    assert_eq!(vouch.voucher(), &voucher);
    assert_eq!(vouch.vouchee(), &vouchee);
    assert!(matches!(vouch.vouch_type(), VouchType::Positive));
    assert!(matches!(
        vouch.context(),
        VouchContext::PatternObserved { dukung_count: 5, .. }
    ));
}
```

### Test 3: PatternObserved does NOT affect trust graph weight

```rust
#[test]
fn test_pattern_observed_does_not_change_weight() {
    let mut graph = TrustGraph::new();
    let a = UserId::new_v4();
    let b = UserId::new_v4();

    // Create two identical vouches — one Direct, one PatternObserved
    let vouch_direct = Vouch::new(
        a, b,
        VouchType::Positive,
        VouchContext::Direct {},
    );

    let vouch_pattern = Vouch::new(
        a, b,
        VouchType::Positive,
        VouchContext::PatternObserved {
            dukung_count: 10,
            first_dukung_at: Utc::now() - Duration::days(180),
            witness_ids: vec![Uuid::new_v4(); 5],
        },
    );

    // Both should produce the same effective weight
    let weight_direct = vouch_direct.weight().effective_weight();
    let weight_pattern = vouch_pattern.weight().effective_weight();

    assert_eq!(weight_direct, weight_pattern,
        "PatternObserved must not alter vouch weight — it is metadata only");
}
```

### Test 4: Backward compatibility — existing contexts still deserialize

```rust
#[test]
fn test_existing_contexts_still_deserialize() {
    // Simulate data written before PatternObserved existed
    let old_data = r#"{"CoWitness":{"witness_id":"550e8400-e29b-41d4-a716-446655440000"}}"#;
    let context: VouchContext = serde_json::from_str(old_data).unwrap();
    assert!(matches!(context, VouchContext::CoWitness { .. }));

    let old_direct = r#"{"Direct":{}}"#;
    let context2: VouchContext = serde_json::from_str(old_direct).unwrap();
    assert!(matches!(context2, VouchContext::Direct {}));
}
```

### Test 5: Edge case — empty witness_ids

```rust
#[test]
fn test_pattern_observed_empty_witness_ids() {
    // Valid but unusual: someone Jamin'd with PatternObserved but no specific witnesses listed
    let context = VouchContext::PatternObserved {
        dukung_count: 1,
        first_dukung_at: Utc::now(),
        witness_ids: vec![],
    };

    let serialized = serde_json::to_string(&context).unwrap();
    let deserialized: VouchContext = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        VouchContext::PatternObserved { witness_ids, .. } => {
            assert!(witness_ids.is_empty());
        }
        _ => panic!("Wrong variant"),
    }
}
```

---

## Validation Checklist

- [ ] `VouchContext::PatternObserved` variant compiles
- [ ] All existing `match` arms on `VouchContext` updated (no exhaustiveness warnings)
- [ ] Serde roundtrip works (Test 1)
- [ ] Vouch creation works (Test 2)
- [ ] Weight is NOT affected (Test 3)
- [ ] Old data still deserializes (Test 4)
- [ ] Edge cases handled (Test 5)
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## What NOT To Do

- **Do NOT** add any weight modifier or bonus for PatternObserved in this PR
- **Do NOT** change the trust graph structure
- **Do NOT** modify PageRank iteration logic
- **Do NOT** add any new API endpoints — this is a data model change only

A future PR may add an optional credibility bonus, but this PR is strictly additive metadata.

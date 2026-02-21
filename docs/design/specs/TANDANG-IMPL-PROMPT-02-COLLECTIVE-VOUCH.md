# Implementation Prompt: Collective VouchType

> **Target**: tandang/markov-engine
> **Priority**: HIGH | **Effort**: MEDIUM
> **Prerequisite**: None (independent of Prompt 01)
> **Branch suggestion**: `feat/vouch-type-collective`

---

## Objective

Add a new `VouchType::Collective` variant that represents corroborated trust — when multiple people vouch for the same person from the same witnessed context. This captures gotong-royong's communal trust formation pattern.

**This changes weight calculation (new weight formula for Collective) but does NOT change the trust graph structure (still UserId → UserId).**

---

## Implementation Steps

### Step 1: Add the enum variant

**File**: `crates/domain/src/types/vouch.rs`

```rust
pub enum VouchType {
    /// Standard positive vouch (default). Weight: 1.0
    Positive,
    /// Skeptical vouch (cautious endorsement). Weight: -0.3
    Skeptical,
    /// Conditional vouch (active only while conditions met). Weight: 0.5–1.0
    Conditional,
    /// Mentorship vouch (higher liability and reward). Weight: 0.8
    Mentorship,
    /// Project-scoped vouch (domain-limited). Weight: 0.6
    ProjectScoped,

    // ---- NEW ----

    /// Collective vouch — corroborated by multiple independent vouchers.
    /// Created when 3+ people vouch for the same person from the same context.
    /// Weight: base_weight × corroboration_bonus
    Collective {
        /// The base vouch type that each individual would have given
        base_type: Box<VouchType>,
        /// Number of corroborating vouchers (minimum 3)
        corroborator_count: u32,
    },
}
```

**Design choice**: `Collective` wraps a `base_type` so we know *what kind* of collective vouch it is (collective positive, collective mentorship, etc.). This preserves type information for analytics and future use.

**Alternative (simpler)**: If wrapping is too complex, make it a flat variant:

```rust
Collective,
// With corroborator_count stored in VouchContext or VouchWeight metadata
```

Choose whichever fits the existing pattern better.

### Step 2: Define weight calculation

**File**: wherever `VouchWeight` or weight computation lives (likely `vouch_entity.rs` or `types/vouch.rs`)

```rust
impl VouchType {
    pub fn base_weight(&self) -> Decimal {
        match self {
            VouchType::Positive => dec!(1.0),
            VouchType::Skeptical => dec!(-0.3),
            VouchType::Conditional => dec!(0.5), // starts at 0.5, can increase
            VouchType::Mentorship => dec!(0.8),
            VouchType::ProjectScoped => dec!(0.6),
            VouchType::Collective { base_type, corroborator_count } => {
                let base = base_type.base_weight();
                let bonus = corroboration_bonus(*corroborator_count);
                base * bonus
            }
        }
    }
}

/// Corroboration bonus: more independent vouchers = slightly more weight.
/// Conservative: starts at 1.05× for 3 people, caps at 1.20× for 6+.
///
/// Formula: 1.0 + min(0.20, (count - 2) × 0.05)
///   3 people = 1.05×
///   4 people = 1.10×
///   5 people = 1.15×
///   6+ people = 1.20× (cap)
fn corroboration_bonus(corroborator_count: u32) -> Decimal {
    let count = corroborator_count.max(3); // minimum 3 for collective
    let bonus = Decimal::from(count - 2) * dec!(0.05);
    dec!(1.0) + bonus.min(dec!(0.20))
}
```

### Step 3: Anti-collusion safeguards

The existing anti-collusion mechanisms already help, but add one new check:

**File**: wherever anti-collusion checks live (likely `trust_graph.rs` or a dedicated module)

```rust
/// Group staleness: if the same set of people always vouch collectively,
/// progressively reduce the corroboration bonus.
///
/// Tracks how often the same group vouches together.
/// After 3 identical-group collective vouches, bonus reduces by 0.05 per occurrence.
/// Floor: bonus of 1.0 (no bonus, but no penalty either).
pub fn group_staleness_dampening(
    group_hash: u64,  // hash of sorted voucher IDs
    previous_collective_count: u32,
) -> Decimal {
    if previous_collective_count <= 3 {
        dec!(1.0) // No dampening for first 3 collective vouches by same group
    } else {
        let reduction = Decimal::from(previous_collective_count - 3) * dec!(0.05);
        (dec!(1.0) - reduction).max(dec!(0.0))
        // At 7+ occurrences, bonus is fully zeroed (but base weight remains)
    }
}
```

### Step 4: Detection — who decides it's Collective?

**Recommended: GR-side detection (simpler)**

GR detects the pattern and sends the vouch pre-typed as `Collective`:

```
GR logic:
  When user triggers Jamin on person X from witness W:
    count = SELECT COUNT(*) FROM jamin
            WHERE vouchee = X AND witness_id = W AND created_at > now() - interval '7 days'
    IF count >= 2:  // this would be the 3rd person
      send to tandang as VouchType::Collective { base_type: Positive, corroborator_count: count + 1 }
      ALSO retroactively upgrade the previous 2 vouches to Collective (optional, see note below)
    ELSE:
      send as VouchType::Positive (normal)
```

**Note on retroactive upgrade**: When the 3rd person vouches and triggers Collective, the first 2 vouches were already sent as `Positive`. Options:
- **Option A**: Leave them as Positive, only the 3rd+ get the bonus. Simpler.
- **Option B**: Send an update/upgrade request to tandang for the first 2. More fair but more complex.
- **Recommendation**: Start with Option A. Revisit if fairness is a concern.

### Step 5: Update all match arms

Search for `match` on `VouchType` throughout the codebase and add the `Collective` arm.

---

## Tests

### Test 1: Corroboration bonus calculation

```rust
#[test]
fn test_corroboration_bonus_values() {
    // 3 people = 1.05×
    assert_eq!(corroboration_bonus(3), dec!(1.05));
    // 4 people = 1.10×
    assert_eq!(corroboration_bonus(4), dec!(1.10));
    // 5 people = 1.15×
    assert_eq!(corroboration_bonus(5), dec!(1.15));
    // 6 people = 1.20× (cap)
    assert_eq!(corroboration_bonus(6), dec!(1.20));
    // 10 people = still 1.20× (cap)
    assert_eq!(corroboration_bonus(10), dec!(1.20));
}
```

### Test 2: Minimum corroborator count enforcement

```rust
#[test]
fn test_collective_minimum_3_corroborators() {
    // Count below 3 should be clamped to 3
    assert_eq!(corroboration_bonus(1), dec!(1.05));
    assert_eq!(corroboration_bonus(2), dec!(1.05));
    assert_eq!(corroboration_bonus(0), dec!(1.05));
}
```

### Test 3: Collective weight calculation

```rust
#[test]
fn test_collective_positive_weight() {
    let collective = VouchType::Collective {
        base_type: Box::new(VouchType::Positive),
        corroborator_count: 4,
    };

    // Positive base = 1.0, bonus for 4 people = 1.10
    // Expected: 1.0 × 1.10 = 1.10
    assert_eq!(collective.base_weight(), dec!(1.10));
}

#[test]
fn test_collective_mentorship_weight() {
    let collective = VouchType::Collective {
        base_type: Box::new(VouchType::Mentorship),
        corroborator_count: 3,
    };

    // Mentorship base = 0.8, bonus for 3 people = 1.05
    // Expected: 0.8 × 1.05 = 0.84
    assert_eq!(collective.base_weight(), dec!(0.84));
}
```

### Test 4: Collective skeptical vouch

```rust
#[test]
fn test_collective_skeptical_weight() {
    let collective = VouchType::Collective {
        base_type: Box::new(VouchType::Skeptical),
        corroborator_count: 5,
    };

    // Skeptical base = -0.3, bonus for 5 = 1.15
    // Expected: -0.3 × 1.15 = -0.345
    // Note: negative weight gets MORE negative — collective skepticism is stronger
    assert_eq!(collective.base_weight(), dec!(-0.345));
}
```

### Test 5: Group staleness dampening

```rust
#[test]
fn test_group_staleness_dampening() {
    let group_hash = 12345u64;

    // First 3 collective vouches: no dampening
    assert_eq!(group_staleness_dampening(group_hash, 1), dec!(1.0));
    assert_eq!(group_staleness_dampening(group_hash, 3), dec!(1.0));

    // 4th: -0.05
    assert_eq!(group_staleness_dampening(group_hash, 4), dec!(0.95));

    // 5th: -0.10
    assert_eq!(group_staleness_dampening(group_hash, 5), dec!(0.90));

    // 7th: fully zeroed
    assert_eq!(group_staleness_dampening(group_hash, 7), dec!(0.80));

    // Beyond 23: floor at 0.0
    assert_eq!(group_staleness_dampening(group_hash, 25), dec!(0.0));
}
```

### Test 6: Trust graph — Collective creates normal UserId edges

```rust
#[test]
fn test_collective_vouch_creates_normal_edges() {
    let mut graph = TrustGraph::new();
    let vouchers = vec![UserId::new_v4(), UserId::new_v4(), UserId::new_v4()];
    let vouchee = UserId::new_v4();

    for voucher in &vouchers {
        let vouch = Vouch::new(
            *voucher,
            vouchee,
            VouchType::Collective {
                base_type: Box::new(VouchType::Positive),
                corroborator_count: 3,
            },
            VouchContext::CoWitness {
                witness_id: Uuid::new_v4(),
            },
        );
        graph.add_vouch(vouch);
    }

    // Should create 3 separate edges, all pointing to same vouchee
    // Graph structure is unchanged — still UserId → UserId
    assert_eq!(graph.edge_count(), 3);
    assert!(graph.node_count() == 4); // 3 vouchers + 1 vouchee
}
```

### Test 7: Serialization roundtrip

```rust
#[test]
fn test_collective_serialization_roundtrip() {
    let vouch_type = VouchType::Collective {
        base_type: Box::new(VouchType::Positive),
        corroborator_count: 5,
    };

    let json = serde_json::to_string(&vouch_type).unwrap();
    let deserialized: VouchType = serde_json::from_str(&json).unwrap();

    match deserialized {
        VouchType::Collective { base_type, corroborator_count } => {
            assert!(matches!(*base_type, VouchType::Positive));
            assert_eq!(corroborator_count, 5);
        }
        _ => panic!("Deserialized to wrong variant"),
    }
}
```

### Test 8: Backward compatibility

```rust
#[test]
fn test_existing_vouch_types_still_deserialize() {
    let positive = r#""Positive""#;
    let skeptical = r#""Skeptical""#;

    assert!(matches!(
        serde_json::from_str::<VouchType>(positive).unwrap(),
        VouchType::Positive
    ));
    assert!(matches!(
        serde_json::from_str::<VouchType>(skeptical).unwrap(),
        VouchType::Skeptical
    ));
}
```

### Test 9: Collective cannot nest (no Collective of Collective)

```rust
#[test]
fn test_collective_cannot_nest() {
    // This should either:
    // A) Be prevented at construction time via a validate() method
    // B) Flatten automatically (extract inner base_type)

    let nested = VouchType::Collective {
        base_type: Box::new(VouchType::Collective {
            base_type: Box::new(VouchType::Positive),
            corroborator_count: 3,
        }),
        corroborator_count: 4,
    };

    // Option A: validate rejects it
    // assert!(nested.validate().is_err());

    // Option B: flattening — weight should use inner Positive, outer count
    // let weight = nested.base_weight();
    // assert_eq!(weight, dec!(1.0) * corroboration_bonus(4));

    // CHOOSE ONE APPROACH and implement accordingly
    // Recommendation: Option A (reject at construction) is simpler
}
```

---

## Validation Checklist

- [ ] `VouchType::Collective` variant compiles with `base_type` and `corroborator_count`
- [ ] `corroboration_bonus()` function implemented and tested
- [ ] `base_weight()` handles Collective variant correctly
- [ ] `group_staleness_dampening()` function implemented
- [ ] All existing `match` arms on `VouchType` updated
- [ ] Serde roundtrip works (Test 7)
- [ ] Backward compatibility preserved (Test 8)
- [ ] Trust graph structure unchanged (Test 6)
- [ ] Nesting prevention decided and implemented (Test 9)
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## Design Decisions for Developer

1. **Flat vs wrapped enum**: The prompt shows `Collective { base_type, corroborator_count }`. If `Box<VouchType>` feels too complex for the existing patterns, make it flat and store `corroborator_count` elsewhere (e.g., in `VouchWeight`). Choose what fits.

2. **Detection side**: GR will send pre-typed Collective vouches. Tandang does NOT need to detect clusters automatically in v1.

3. **Retroactive upgrade**: Not required in v1. Only the 3rd+ voucher gets the Collective bonus. Previous vouchers stay as their original type.

4. **Nested prevention**: Recommend rejecting at construction (`Vouch::new()` validates `base_type` is not `Collective`).

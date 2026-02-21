# Implementation Prompt: GR-Native ESCO-ID Domain Extensions

> **Target**: tandang/markov-engine
> **Priority**: MEDIUM | **Effort**: LOW
> **Prerequisite**: None (independent)
> **Branch suggestion**: `feat/esco-id-gr-domains`

---

## Objective

Register gotong-royong-specific skill domains in the ESCO-ID extension system so that `CompetenceScore` can track per-domain expertise for GR-native activities. These activities (community coordination, citizen investigation, etc.) are not covered by standard ESCO taxonomy.

**This does NOT change any calculation logic — CompetenceScore already supports arbitrary SkillId keys via its HashMap. We are only registering new domain identifiers.**

---

## Context

Currently `CompetenceScore` is stored as:

```rust
competence: HashMap<SkillId, CompetenceScore>
```

Each `SkillId` maps to an ESCO code (or ESCO-ID extension for Indonesian skills). The decay system already operates per-domain — inactive domains decay independently. We need to add GR-specific domains so users build trackable competence in community activities.

---

## Proposed Domains

| Code | Indonesian Name | English Name | Description | Activity Examples |
|---|---|---|---|---|
| `ESCO-ID-GR-001` | Koordinasi Komunitas | Community Coordination | Organizing collective action, managing group logistics | Creating witnesses, organizing events, task assignment |
| `ESCO-ID-GR-002` | Investigasi Warga | Citizen Investigation | Gathering evidence, fact-finding, field reporting | Submitting evidence to witnesses, field documentation |
| `ESCO-ID-GR-003` | Mediasi Konflik | Conflict Mediation | Resolving disputes, facilitating dialogue | Jury participation, dispute resolution, sensemaking |
| `ESCO-ID-GR-004` | Verifikasi Lapangan | Field Verification | On-ground reality checking, Proof-of-Reality | PoR submissions, location verification, photo evidence |
| `ESCO-ID-GR-005` | Pendampingan Warga | Community Mentorship | Guiding new participants, knowledge transfer | Mentorship vouches received/given, onboarding help |

---

## Implementation Steps

### Step 1: Determine how SkillId domains are registered

**You need to answer this first**: How is the ESCO/ESCO-ID skill registry currently implemented?

**If config/seed file**:
```toml
# Add to existing skill registry config
[skills.esco-id-gr]
ESCO-ID-GR-001 = { name_id = "Koordinasi Komunitas", name_en = "Community Coordination", category = "gotong-royong" }
ESCO-ID-GR-002 = { name_id = "Investigasi Warga", name_en = "Citizen Investigation", category = "gotong-royong" }
ESCO-ID-GR-003 = { name_id = "Mediasi Konflik", name_en = "Conflict Mediation", category = "gotong-royong" }
ESCO-ID-GR-004 = { name_id = "Verifikasi Lapangan", name_en = "Field Verification", category = "gotong-royong" }
ESCO-ID-GR-005 = { name_id = "Pendampingan Warga", name_en = "Community Mentorship", category = "gotong-royong" }
```

**If database seed/migration**:
```sql
INSERT INTO skill_domains (code, name_id, name_en, category, created_at) VALUES
  ('ESCO-ID-GR-001', 'Koordinasi Komunitas', 'Community Coordination', 'gotong-royong', now()),
  ('ESCO-ID-GR-002', 'Investigasi Warga', 'Citizen Investigation', 'gotong-royong', now()),
  ('ESCO-ID-GR-003', 'Mediasi Konflik', 'Conflict Mediation', 'gotong-royong', now()),
  ('ESCO-ID-GR-004', 'Verifikasi Lapangan', 'Field Verification', 'gotong-royong', now()),
  ('ESCO-ID-GR-005', 'Pendampingan Warga', 'Community Mentorship', 'gotong-royong', now());
```

**If hardcoded enum**:
```rust
// Add to SkillId or wherever domains are defined
pub mod gr_domains {
    use super::SkillId;

    pub const KOORDINASI_KOMUNITAS: SkillId = SkillId::new("ESCO-ID-GR-001");
    pub const INVESTIGASI_WARGA: SkillId = SkillId::new("ESCO-ID-GR-002");
    pub const MEDIASI_KONFLIK: SkillId = SkillId::new("ESCO-ID-GR-003");
    pub const VERIFIKASI_LAPANGAN: SkillId = SkillId::new("ESCO-ID-GR-004");
    pub const PENDAMPINGAN_WARGA: SkillId = SkillId::new("ESCO-ID-GR-005");
}
```

### Step 2: Map GR activities to domains

This is for documentation / API contract — GR will send domain info when reporting contributions:

| GR Activity | Maps to Domain |
|---|---|
| Create a witness | GR-001 (Koordinasi) |
| Participate in a witness | GR-001 (Koordinasi) |
| Submit evidence | GR-002 (Investigasi) |
| Serve on jury | GR-003 (Mediasi) |
| Submit Proof-of-Reality | GR-004 (Verifikasi) |
| Give mentorship vouch | GR-005 (Pendampingan) |
| Receive mentorship vouch | GR-005 (Pendampingan) |
| Sensemaking validation | GR-003 (Mediasi) |
| Verify execution of task | GR-004 (Verifikasi) |

### Step 3: Ensure decay works for new domains

No code change needed — verify that:
- New SkillId keys are accepted by `CompetenceScore` HashMap
- `DecayCalculator` operates on any SkillId without filtering
- `ActivityTracker` can track activity for arbitrary domains

---

## Tests

### Test 1: GR domain SkillIds are valid

```rust
#[test]
fn test_gr_domain_skill_ids_are_valid() {
    let domains = vec![
        "ESCO-ID-GR-001",
        "ESCO-ID-GR-002",
        "ESCO-ID-GR-003",
        "ESCO-ID-GR-004",
        "ESCO-ID-GR-005",
    ];

    for code in domains {
        let skill_id = SkillId::from(code);
        assert!(skill_id.is_valid(), "SkillId {} should be valid", code);
    }
}
```

### Test 2: CompetenceScore works with GR domains

```rust
#[test]
fn test_competence_score_with_gr_domain() {
    let mut reputation = UserReputation::new(UserId::new_v4());

    let gr_domain = SkillId::from("ESCO-ID-GR-001");

    // Add competence in a GR domain
    reputation.update_competence(gr_domain.clone(), dec!(0.7));

    let score = reputation.competence(&gr_domain);
    assert!(score.is_some());
    assert_eq!(score.unwrap().score(), dec!(0.7));
}
```

### Test 3: Decay works independently per GR domain

```rust
#[test]
fn test_gr_domain_decay_independence() {
    let mut reputation = UserReputation::new(UserId::new_v4());

    let koordinasi = SkillId::from("ESCO-ID-GR-001");
    let investigasi = SkillId::from("ESCO-ID-GR-002");

    // Both start at 0.7
    reputation.update_competence(koordinasi.clone(), dec!(0.7));
    reputation.update_competence(investigasi.clone(), dec!(0.7));

    // Simulate: active in koordinasi, inactive in investigasi for 120 days
    // After 30-day trigger + 90-day half-life:
    // investigasi should decay to ~0.35
    // koordinasi should remain ~0.7

    reputation.record_activity(koordinasi.clone(), Utc::now());
    // No activity recorded for investigasi

    let decay_calc = DecayCalculator::new();
    decay_calc.apply_decay(&mut reputation, Utc::now() + Duration::days(120));

    let k_score = reputation.competence(&koordinasi).unwrap().score();
    let i_score = reputation.competence(&investigasi).unwrap().score();

    assert!(k_score > dec!(0.65), "Active domain should not decay significantly");
    assert!(i_score < dec!(0.50), "Inactive domain should decay after 120 days");
}
```

### Test 4: Multiple GR domains on same user

```rust
#[test]
fn test_multiple_gr_domains_on_same_user() {
    let mut reputation = UserReputation::new(UserId::new_v4());

    let all_domains = vec![
        ("ESCO-ID-GR-001", dec!(0.8)),
        ("ESCO-ID-GR-002", dec!(0.6)),
        ("ESCO-ID-GR-003", dec!(0.9)),
        ("ESCO-ID-GR-004", dec!(0.5)),
        ("ESCO-ID-GR-005", dec!(0.7)),
    ];

    for (code, score) in &all_domains {
        reputation.update_competence(SkillId::from(*code), *score);
    }

    // All 5 domains should coexist
    assert_eq!(reputation.competence_domain_count(), 5);

    // Each should have its own score
    for (code, expected_score) in &all_domains {
        let actual = reputation.competence(&SkillId::from(*code)).unwrap().score();
        assert_eq!(actual, *expected_score, "Domain {} score mismatch", code);
    }
}
```

### Test 5: GR domains coexist with standard ESCO domains

```rust
#[test]
fn test_gr_domains_coexist_with_esco() {
    let mut reputation = UserReputation::new(UserId::new_v4());

    // Standard ESCO domain
    let esco_programming = SkillId::from("http://data.europa.eu/esco/skill/abc123");
    reputation.update_competence(esco_programming.clone(), dec!(0.8));

    // GR domain
    let gr_koordinasi = SkillId::from("ESCO-ID-GR-001");
    reputation.update_competence(gr_koordinasi.clone(), dec!(0.7));

    // Both should exist independently
    assert!(reputation.competence(&esco_programming).is_some());
    assert!(reputation.competence(&gr_koordinasi).is_some());
    assert_eq!(reputation.competence_domain_count(), 2);
}
```

### Test 6: Domain-qualified vouch (optional, for future)

```rust
#[test]
fn test_project_scoped_vouch_with_gr_domain() {
    // If ProjectScoped vouches can be domain-qualified:
    let voucher = UserId::new_v4();
    let vouchee = UserId::new_v4();

    let vouch = Vouch::new(
        voucher,
        vouchee,
        VouchType::ProjectScoped,
        VouchContext::VerifiedExecution {
            task_id: Uuid::new_v4(),
        },
    );

    // The vouch should be associable with a GR domain for C-score routing
    // Implementation detail: how does the engine know which domain to credit?
    // Options:
    //   A) VouchContext carries domain info
    //   B) Witness/task metadata includes domain
    //   C) GR sends domain as separate parameter

    // This test documents the question — implement based on existing patterns
    assert!(vouch.voucher() == &voucher);
}
```

---

## Validation Checklist

- [ ] GR domain codes registered (config/DB/code — whichever mechanism exists)
- [ ] `SkillId::from("ESCO-ID-GR-001")` is accepted as valid
- [ ] `CompetenceScore` HashMap accepts GR domain keys (Test 2)
- [ ] Decay operates independently per GR domain (Test 3)
- [ ] Multiple GR domains can coexist on one user (Test 4)
- [ ] GR domains don't interfere with standard ESCO domains (Test 5)
- [ ] `cargo test` passes with zero new failures
- [ ] `cargo clippy` clean

---

## Questions for Developer

1. **Registry mechanism**: How are ESCO-ID extensions currently registered? Config file, DB seed, or code? This determines the exact implementation.

2. **Domain-qualified vouches**: When a `ProjectScoped` vouch comes in, how does the engine know which CompetenceScore domain to credit? Is there a field for this, or does it need to be added?

3. **Naming convention**: Should GR domains follow `ESCO-ID-GR-XXX` or a different prefix to distinguish them from official ESCO-ID Indonesian extensions?
